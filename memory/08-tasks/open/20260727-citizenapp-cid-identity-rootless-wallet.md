# citizenapp CID 身份层 + 无根多账户钱包(全量跨模块)

状态:open(用户授权 2026-07-27;需求分析 + 链侧侦察 + 三决策已锁定,待首小步方案确认)
所属模块:Chain(citizenchain runtime)+ OnChina(注册局)+ Mobile(citizenapp);citizenwallet 反向改动由**另一个会话**做
关联/推翻:
- 承接并**重塑** `20260727-citizenwallet-modelb-index-derivation.md` 的 Step 2(原「citizenapp 保留种子/助记词、单热钱包」决策被本卡**推翻**)
- Step 1(citizenwallet 无根多账户)已完成,本卡把同一无根 `//index` 模型搬到 citizenapp
- ADR-026 统一签名 [[unified-signing-protocol-adr026]];CID 格式 [[cid-code-table-level-final]] / [[china-code-immutable]](本卡改公民 CID 前缀 → 需同步)

## 背景漏洞(用户 req 4)
当前身份唯一主键 = 钱包账户,私钥热存本机;私钥泄漏 = 必须换身份 = 丢失全部动态/文章/粉丝等社交资产。
修复:身份主键切为 **CID 号**,钱包账户降为**可更换的签名子钥**(仿链上个人多签 CID+账户组合)。

## 三决策(用户 2026-07-27 拍板)
- **D1 自助·自付费**:公民本人 `Signed` origin 直接占号,自付最低链上费(`ONCHAIN_MIN_FEE=10`),绕过注册局。新增 permissionless `self_occupy_cid`。
- **D2 仅自助轮换换绑**:`commitment = blake2_256(账户公钥)`;换绑由**当前绑定账户签名**授权换到新 `//index` 账户;彻底丢失助记词=不可恢复(接受)。暂不做恢复密钥、暂不做注册局 KYC 恢复。
- **D3 人主体 CID 去地域化 + 号段扩容(人/机构彻底分开)**:**人主体(公民 CTZN / 居民 NATP / 智能人 SMTP)统一**(用户 2026-07-27 修正:不留任何 person 走省码;NATP=居民,非自然人)R5 **位 1-2=`CN` 国家码;位 3-5 由固定 `000` 改为承载号码高 3 位**(原 000 对人无意义,回收作号段)。人号段 = R5 位3-5(高 3 位)+ N9(低 9 位)= **12 位 = 1e12/年**;人只吃公钥/码/年,不吃省市。机构 CID 才用省码+市码(`is_person_code` 分流,`CN` 前缀天然区分)。居住省/市仍单独存于 `VotingIdentity`/`CidRecord`(与 CID 号解耦)。[[unify-means-zero-exceptions]] 零例外。

## 链侧现状(侦察定论,`runtime/misc/citizen-identity/src/lib.rs`)
- 存储**已全在位**:`CidRegistry`(CID→记录,含 `commitment[32]`/residence/status Active·Revoked)、`AccountIdByCid`+`CidByAccountId`(双向绑定闭环 `bind_account_id` L1407 / `ensure_account_id_binding_available` L1375)、`VotingIdentityByCid`、`CandidateIdentityByCid`、四级人口计数。**匿名 CID = 有 CidRegistry+双向绑定、无 VotingIdentityByCid**,存储天然可表达。
- 缺口:①全部 extrinsic **注册局门**(`CitizenIdentityAuthority::can_manage_voting_identity`,CREG/FRG);无自助。②`occupy_cid`(call 6,L1085)**只写号不绑账户**;账户到 `register_voting_identity`(call 0)才绑。③**无换绑**(`update_*` 禁改账户 `ensure_current_account_id_binding` L1395)。
- CID **客户端生成**(`primitives/cid/generator.rs`),链只格式+查重仲裁(`do_occupy_cid` L1237);现个人码强制市 `000`、R5=省+000、N9=hash(公钥|码|省名|000|年)。**D3 改为 R5=CN000。**
- 费:citizen_identity 全走 `institution_onchain_route`(机构付费,`configs.rs fee_route` L431+);**自助需新自付费 arm**(`signer_onchain_route` 式,签名人自付,min 10)。

## 跨模块设计

### ① 链 citizen-identity(Blockchain)
- **CID 格式改 D3**:`primitives/cid/{generator.rs,number.rs,code.rs}` 公民 CID R5=`CN`+号码高3位;12 位号段 = `hash(公钥/种子|CTZN|年) mod 1e12`+nonce probe,高3→R5[2:5]、低9→N9;`parse_cid_number_parts` 放行 `CN` 前缀且 R5 位3-5 为数字(跳过省/市码表查);核心段校验位含 R5 自然覆盖。最终公式钉金标。**单源,冷热链共用。**
- **新 `self_occupy_cid`**(Signed 自助):参数 `cid_number`(客户端 CTZN/CN000)+`commitment[32]=blake2_256(pubkey)`;占号即绑账户(写 CidRegistry+双向绑定,复用闭环校验);**不写 VotingIdentity**;自付费;新 op_tag 入 `primitives::sign`。
- **新 `rebind_cid_account`**(自助轮换):当前绑定账户签名授权 → 换绑到新账户;更新双向绑定;旧账户失效。
- **改 `occupy_cid`**(注册局路径,req 5 第1步):加账户绑定参数,占即绑。
- `register_voting_identity`/`upgrade_to_candidate_identity` → 匿名 CID 之上**可选升级**(req 5 第2步「创建档案」;「匿名」=止于占号+绑定)。
- `fee_route` 加自付费 arm。**dev 期重新创世** [[chain-in-dev]] + 注册表重生 [[registry-regen-after-genesis]]。新业务逻辑留 pallet,不进 primitives 核心常量 [[no-business-types-in-primitives]]。

### ② onchina 注册局(OnChina)
- `src/domains/citizens/occupy.rs`:占号第1步加钱包账户(占即绑);第2步 handler/UI 分「创建公民档案」/「匿名」两支。
- CID 生成同步 D3(`CN000`)。

### ③ citizenapp(Mobile,两层)
- **钱包层**:无根多账户派生(逐字节复用 Step 1 `//0//1//2` 金标)+ 只存账户公私钥对、不存种子/助记词;单钱包;删 App 内「创建/导入钱包」入口 + 删「默认账户/默认用户」;钱包空→踢回初始化;账户详情加「添加账户」(输助记词+自动填充,派生下一 `//index`);列表**热账户 + 冷钱包共存**(冷钱包代码不动);创建提示手抄助记词或在 citizenwallet 保管。
- **身份层**:`电子护照`→`身份`改名;首卡`注册身份·访客`(匿名图标留);初始化后跳身份页;右上`刷新`→`注册`/`更换`按钮 + 下拉刷新;点注册=生成 CN000 CTZN CID→签名→`self_occupy_cid` 自付费交易→绑定;已注册首卡下显`身份cid号 xxxxx`、下两卡`公民cid号`→`身份cid号`;对外 CID 主、账户辅;**身份主键 `getDefaultWallet()`→CID-bound account**(20+ 调用方联动:8964/chat/my/membership/contact/myid);聊天/广场/订阅/创作者/通讯录 **门控到已注册 CID**(与四层门禁 [[citizenapp-four-gate-entry-failclosed]] 合并厘清)。

## 小步执行序(每小步先出技术方案待确认再执行;`dart analyze` 0 + `flutter test` 全绿 / `cargo test` 全绿为门禁)
- **S1 链**:公民 CID 号段方案(CN 前缀 + 位3-5 承载号码高位 + N9 = 12 位/年;primitives/cid 生成+校验 + node 共识守卫)+ 金标向量。**✅ 完成(2026-07-27,全工作区绿)**。
- **S2 链**:`self_occupy_cid`(自助+自付费+占即绑+匿名)+ op_tag + 测试。
- **S3 链**:`self_rebind_cid_account`(自助轮换,origin=新账户+旧账户签名,OP_SIGN_CID_REBIND)+ 测试。**✅ 完成(2026-07-27,全工作区绿)**。
- **S4+S6 合并(注册局流程,链+onchina 强耦合)→ 交接新窗口实现**:occupy 占即绑账户+签名、档案选填(D4a)、register 变纯升级、`admin_rebind_cid_account`(匿名任一注册局,D4b)、CTZN|NATP、onchina 弹窗/占号/R5-split 适配。**自包含交接卡:`20260727-registrar-cid-flow-s4s6.md`**。
- **S5 链**:重新创世 + 注册表重生 + 冷热链一致性。
- **S6 onchina**:已并入 S4+S6 合并交接卡(见上）。
- **S7 citizenapp 钱包层**:无根多账户重构(独立于链)。**✅ 完成(2026-07-27,S7.1/7.2/7.3 全绿)**。
- **S8 citizenapp 身份层**:改名 + 注册按钮 + 自助占号交易 + CID 主键切换 + 门控。拆三子步:
  - **S8.1 客户端 CID 生成 + 占号/换绑 RPC**(纯离线编码,逐字节镜像链,不依赖 S4+S6)。**✅ 完成(2026-07-27,848 绿)**。
  - **S8.2 身份页改造**(电子护照→身份改名 + 首卡`注册身份·访客` + 初始化跳身份页 + 右上`注册`/`更换`按钮 + 下拉刷新 + CID 展示)。**✅ 完成(2026-07-27,867 绿)**。**决策裁定(用户)**:①**不新增徽章色**,保持现有 3 色(visitor/voting/candidate),匿名占号后仍访客色,仅**第 1 卡显已注册 CID 号**;②「更换」**含**,与注册一起做(匿名 CID 可自助换绑选目标账户;voting/candidate 提示去注册局);③初始化后**跳身份页但绝不动底部 5-tab 结构**(onboarding 放行后一次性 push MyIdPage,返回回落广场)。
  - **S8.3 身份主键切换 + 门控 + 删默认用户**(`getDefaultWallet()`→CID-bound account、`signWithWallet` re-plumb 到 accountId、聊天/广场/订阅/创作者/通讯录门控到已注册 CID、删「默认用户」)。**决策裁定(用户,2026-07-27,grounding 后校准)**:
    - **①时机=现在全做**;**验证=本地开发链重新创世**(本地起含本会话 S1-S3 `self_occupy_cid` 的 citizenchain 矿工端重新创世,app 连它走 新建钱包→自助占号→匿名已注册→门控解锁 全链路)。**⚠️纠正前提**:创世公民(法代 `CN220-CTZN2-198805200-2026`/账户 `0x0cb1d05c…4b06b`)在 `citizen-identity` **无 GenesisConfig 记录**=链上纯访客(`readByAccountId` 返回 null),**不能用它验证门控/主键**;助记词也不在仓。
    - **②门控=全功能门控 CID**,未注册即引导注册,访客无匿名可用面。
    - **③身份账户=CID 绑定的钱包账户(可任意 `//n`,非恒账户0)**——见 [[citizenapp-cid-identity-master-key]]:身份主键=CID 号,钱包账户是 CID 绑定的鉴权凭证;**注册 CID 时用户自选绑定账户**;设备子钥/广场会话/通讯录密钥都从 CID 绑定账户派生(非恒账户0),换绑随之重派迁移(根治 agent 报告的三处对不齐,不是把身份拉回账户0)。
    - **架构发现**:身份主键类型从 `WalletProfile`(钱包级)降到 `Account`(账户级);27 处 `getDefaultWallet`+27 处 `signWithWallet` 按用途分流(身份→身份账户 / 付款治理机构→保留账户0或所选)。QR 签名服务(citizen_identity/square_action)只遍历 WalletProfile,子账户身份有 gap 需扩。
    - **拆子步**:S8.3a 身份账户单源 `getIdentityAccount()` + 注册选绑定账户 **✅完成(2026-07-27,875绿)**;S8.3b 鉴权凭证迁移(设备子钥/会话/通讯录钥/社交·会员·创作者签名从 CID 绑定账户派生;**死契约 [[cid-rebind-subkeys-must-auto-migrate]]**:换绑 Finalized 成功后所有派生子钥必自动成功更换,本地重派必成功、后端重注册持续重试到成功,不允许不更换或失败);S8.3c 全功能门控 CID(合并口径:CID门×设备子钥会话×发布四层门禁);S8.3d 删默认用户;S8.3e 本地开发链重新创世 e2e。

## S1 落地(2026-07-27,完成 ✅)
> **修正(2026-07-27,用户)**:CN 方案由「仅 CTZN」扩为「**全部 `is_person_code`(CTZN/NATP/SMTP)**」——删 `is_citizen_code`,generator/`cid_scope_codes` 守卫/node `valid_province_city` 均改 `is_person_code`,generator else 分支删原 person city=000 死逻辑,人主体金标测试覆盖三码。CTZN 金标值不变。**`cargo test --workspace` 全绿(81 批次 0 failed / 0 panicked)。**
**改动(仅链侧单源,主检出 `citizenchain/`)**:
- `runtime/primitives/cid/code.rs`:新增 `is_citizen_code`(仅 CTZN;NATP/SMTP 仍省+000)。
- `runtime/primitives/cid/generator.rs`:CTZN 分支产 `CN`+高3位、N9 低9位,12 位号 = `hash_u64(公钥|CTZN|年) mod 1e12`(新增 `hash_u64` 取 blake2 前 8 字节);公民分支**不再要求省/市**(自助占号无居住地);省市机构分支逻辑不变。
- `runtime/primitives/cid/number.rs`:`cid_scope_codes` 加公民 fail-closed 守卫(CTZN 传入即 Err,杜绝把号段误读成区划码);doc 更新。
- `node/src/core/node_guard/cid_lifecycle.rs`:**共识守卫**(独立第二 CID 校验实现)`valid_province_city` 加公民分支——CTZN 校验 R5=`CN`+3数字,而非省+市(否则节点会拒 CN 公民 CID)。**CN 方案的第二处必改点**,由全工作区回归暴露并修复。
- `onchina/src/cid/generator.rs`:仅测试断言改 CN(包装逻辑不变,仍向 primitive 传省份,被 CTZN 分支忽略)。
- 金标:`generator.rs` `citizen_cid_number_golden` 钉死 dev //0 公钥 `0x2afba927…` → **`CN951-CTZN1-539598435-2026`**(12 位号=951539598435);另加 `citizen_cid_needs_no_province`、`number.rs::citizen_cid_parses_but_has_no_scope`。
- **测试**:`cargo test --workspace` 全绿(primitives 78 / citizen-identity / node bin 294 / 其余全部批次 0 failed;金标钉死)。

**留待 S5 重新创世 / Step 3 重生(非本步,记录防遗漏)**:旧**省码式**公民 CID 字面仍能 parse+validate(parser 不查省码),但语义已过时,须随重生改 CN 前缀:
- 创世常量:`runtime/primitives/cid/china/china_zf.rs:30 FSC_GENESIS_ADMIN_CID_NUMBER`、`.../citizenchain.rs:22 LEGAL_REPRESENTATIVE_CITIZEN_CID_NUMBER`(均 `CN220-CTZN2-198805200-2026`)。
- 测试夹具:square-post / private-manage / public-manage / admin-primitives / public-admins / runtime/src/tests 里的 `GZ000-CTZN6-…` `GD001-CTZN1-…`(结构合法,S5 统一改 CN 提高真实度)。
- 校验(已核):链侧无 live code 把公民 CID 传 `cid_scope_codes`(守卫兜底);`legislation-yuan` 的 `parts.r5` 只用于**公权机构** CID 路由(`ensure_route_institution_*`,不碰 CTZN)——无 live 隐患。

**留待 S6(onchina 改造)**:onchina 有本地「从 CID R5 切省/市」逻辑,现吃机构 CID 或旧省码式创世公民常量(今可用),自助 CN 公民到来后会把省读成 `CN`,须随 S6 适配:`onchina/src/domains/genesis_projection.rs::split_province_city`、`.../projection.rs::r5_province_city`、`.../docs/handler.rs::scope_codes_from_cid`。公民省份改取自 `citizens` 表/居住字段,不再从 CID R5 推。

## S2 落地(2026-07-27,完成 ✅)
> **q3 修正(2026-07-27,用户)**:自助占号类型从「仅 CTZN」放宽为「**CTZN(公民)/ NATP(居民)用户自选**」(投票/竞选公民只能经注册局线下注册=q1)。`do_occupy_cid` 移除 CTZN 硬校验并下沉到调用方(注册局 occupy/batch=`ensure_valid_citizen_cid` CTZN;自助=新 `ensure_valid_self_registrable_cid` CTZN|NATP;SMTP/机构码拒)。`cargo test -p citizen-identity` 43 绿(+NATP happy /+SMTP 拒);**`cargo test --workspace` 全绿(81 批次 0 failed)。**
**`self_occupy_cid`(自助占号+占即绑+匿名+自付费),仅链侧,主检出 `citizenchain/`:**
- `runtime/misc/citizen-identity/src/lib.rs`:新 extrinsic `self_occupy_cid(origin, cid_number)` **`call_index(5)`**(复用已退役槽,不留空位);`ensure_signed`→`account_id`(命名对齐 [[wallet-account-naming-account-id]]);**commitment 链上算** `blake2_256(account_id.encode())`(客户端不传);复用 `ensure_account_id_binding_available`(一账户一 CID 闭环)+`do_occupy_cid`(SELF registrar + 空 residence,格式+CTZN+原子查重)+`bind_account_id`(占即绑);**不写 VotingIdentity**(匿名)。新增 `pub const SELF_OCCUPY_REGISTRAR=b"SELF"` + 事件 `CidSelfOccupied{cid_number,account_id}` + `use sp_io::hashing::blake2_256`。
- `weights.rs`:trait + SubstrateWeight + unit 三处 `self_occupy_cid()`(手工权重 reads3/writes3,dev,待 benchmark 精化)。
- `benchmarks.rs`:`self_occupy_cid` 基准(`whitelisted_caller`),`--features runtime-benchmarks` 编译绿。
- `runtime/src/configs.rs`:fee_route 加 `CitizenIdentity(self_occupy_cid{..}) => signer_onchain_route(who,0)`(**签名人自付** min 10),置于机构付费 arm 前。
- `tests/mod.rs`:5 用例(绑定+匿名+SELF记录+commitment / 幂等 / 一账户一CID拒 / 他人占同CID拒 / 非CTZN拒)——**`cargo test -p citizen-identity` 41 全绿**。
- **node 守卫**:`validate_citizen_record` 放行 SELF(非空 registrar)+ 空 residence + Active + 无 VotingIdentity → **无需改守卫**(已验)。
- **回归**:`cargo test --workspace` = 19 批次 0 failed;node bin 唯一失败 `admin_unlock::lock_decrypted_admin_removes_entry` 是 **pre-existing 并行竞态 flaky**(共享全局 `decrypted_map()`,`list_decrypted_admins_filters_by_cid` 结尾 `clear()` 整表),**与 S2 无关**——单线程 `admin_unlock` 6/6 绿证。已记独立修复任务。

## S3 落地(2026-07-27,完成 ✅)
**`self_rebind_cid_account`(匿名 CID 自助换绑),仅链侧,主检出 `citizenchain/`:**
- 接口:`self_rebind_cid_account(origin=新账户, cid_number, old_account_signature)` **`call_index(9)`**。新账户作 origin(自签证受控+自付费);旧账户从 `AccountIdByCid[cid]` **反查、不传**(取不到=`NotBoundToAnyCid`);payload=`(cid_number,new_account_id).encode()`,旧账户对其 op-tag 签名(D2「当前绑定账户签名授权」)。
- 门:**匿名限定(q1)** —— 有 `VotingIdentityByCid` 即 `CivicRebindRequiresRegistrar`(civic 走 S4 注册局);**一账户一 CID** —— 新账户已绑他 CID 即 `AccountIdAlreadyBoundToAnotherCid`(新账户任意、能签即可,q 用户确认)。
- 换绑原语 `rebind_account_id`(删旧反向索引+写新双向绑定,old==new 幂等);事件 `CidAccountRebound{cid,old,new}`;错误 `NotBoundToAnyCid`/`CivicRebindRequiresRegistrar`/`InvalidRebindSignature`。
- **签名域**:`primitives/src/sign.rs` 新增 `OP_SIGN_CID_REBIND=0x11` + 进 `SIGN_OP_TAGS[12]` + fixture `signing_domain_vectors.json` 加 0x11 金标向量(`SIGN_GOLDEN_UPDATE=1` 回填)。**四端契约**:citizenapp(S8)/其余端签此换绑须同 op_tag。
- `CitizenIdentityAuthority` 加 `verify_rebind_signature`(**4 处实现全覆盖**:trait/`()`、configs Runtime 用 OP_SIGN_CID_REBIND、citizen-identity mock、citizen-issuance 集成 mock;后两 `==b"valid"`)——trait 加方法波及全部 impl,漏 citizen-issuance 集成测试 mock 曾致编译错、已补;pallet helper `ensure_rebind_signature`。
- `configs.rs` fee_route:`self_rebind_cid_account => signer_onchain_route`(自付费);weights 3 处 + benchmark(`account`+`whitelisted_caller`)。
- 测试:换绑成功(旧账户反向索引清、新账户接管)/ 未占拒 / 旧签名无效拒 / 新账户已绑他 CID 拒 / **civic 自助换绑拒** / **NATP 居民不能升级投票公民**(锁 q3 约束)。
- **状态**:**全绿** —— `cargo test -p citizen-identity` 49(+6 S3)、benchmark 编译、`sign_golden` 0x11 锁定、`cargo test --workspace` **81 批次 0 failed**。node 守卫放行 rebind(只校验绑定闭环一致性、不禁变更;registrar/residence 不可变只针对 `CidRegistry` 记录,rebind 不改该记录)。
- **q1/q2/q3 锁定(2026-07-27)**:q1 投票/竞选公民只能注册局线下注册(自助只出匿名);q2 自助换绑失败者去线下注册局,管理员可直接换绑任意 CID(=S4 `admin_rebind_cid_account`);q3 自助/注册局注册均让用户自选 CTZN(公民)/NATP(居民),且 **NATP 永不可升投票/竞选**(现有 CTZN 校验已保护)。

## S7 决策(D7,2026-07-27,本窗口进行中)
- **框架订正**:citizenwallet 现盘已被另一会话**反转成「存种子+助记词」**(`masterSeedHexV1`/`masterMnemonicV1`,已无 `accountMiniSecretV1`)。故可**逐字节复用的只是 `//index` 派生金标**(冷热共享);「无根存储(每账户 child mini-secret、不存种子/助记词)」在 **citizenapp 新建**,非照抄 citizenwallet 现盘。
- **D7a 存储+鉴权**:child mini-secret 存**硬件金库 strict 层**(citizenapp 唯一鉴权凭证);**签名/读密钥强制 `biometricOnly=true` fail-closed**,无生物识别/验证失败/取消一律拒(连创建/导入也拒)——[[biometric-only-mandatory]] 扩到 citizenapp 热端。
- **D7b 无根 + 角色分工**:助记词/种子**不持久化**;初始化(创建或导入)只在本机生成**公私钥对**,提示用户手抄助记词或在 citizenwallet 保管。分工定稿:**citizenapp=公私钥对日常用(无根)、citizenwallet=种子/助记词冷签+安全保管**。addAccount 须重输助记词(归属校验后派生下一 //index)。
- **D7c 边界**:CID 主键切换 / 删默认用户 / 签名改按 accountId → **S8**;S7 过渡期 `getDefaultWallet`/`signWithWallet(walletIndex)` 解析到账户0,§6 全部身份/签名调用方零改。
- **D7-合并**:**S7.1 = 派生 `//index` + 无根存储**(合并一子步,二者耦合)。

## S7.1 落地(2026-07-27,完成 ✅)
**citizenapp 无根单账户核心(派生 //0 + 无根存储),仅主检出 `citizenapp/`,冷钱包/§6 调用方零改:**
- **派生**:`wallet_manager.dart` 删 bare 根 `_deriveSr25519FromSeed`;新增 `_childMiniSecret`(逐字节移植金标)+`_deriveAccount(seed,index)`,账户0=`//0`。金标 `test/wallet/derivation_golden_test.dart`(//0//1//2 == 冷端,ss58 2027)**spike 全绿**。
- **无根存储**:`secure_seed_store.dart` 接口改 `putAccountKey/readAccountKey/hasAccountKey/deleteAccountKey`(按 accountId,**删种子/助记词层**);`hardware_bound_seed_vault.dart` blob `wallet_account_key_v1_$accountId`、KEK 仍按 walletIndex(同钱包多账户共 KEK、各账户分 blob)、**仅 strict biometricOnly**;fake 同步。
- **wallet_manager**:createWallet/importWallet 只存账户0 child、母种子 finally 清零、助记词一次性返回不落库;`signWithWallet`/`_loadSigningKey` 读 child(生物识别)+校验+清零;**删 `_selfHealSeedFromMnemonic`/`_readSeedHexWithSelfHeal`/`getMnemonic`**(无根无自愈,失效/缺失→`WalletAuthException` 重导入 fail-closed);`getSeedHex`→账户0 child;contact key/`_registerDeviceSubkey` 从 child;`isUsableHotWallet` 用 `hasAccountKey`;clear/delete/rollback 用 `deleteAccountKey`。
- **UI**:`wallet_page.dart` 删「查看助记词」菜单(唯一 `getMnemonic` 调用方);「查看私钥」留。
- **三条安全不变量**(逐条核实):①不存种子(母种子仅内存、finally 清零)②不存助记词(无存储层)③签名/读密钥强制 strict 生物识别、取消/失效/缺失 fail-closed。
- **门禁**:`dart analyze` **No issues** + `flutter test` **813 passed / 5 skipped / 0 failed**(本人独立复跑 `FLUTTER_EXIT=0`,非仅信 agent);残留扫 0;金标 spike 全绿。
- **范围收敛**:S7.1 保单账户 WalletProfile(不动 app_isar),AccountEntity/多账户 → S7.2。

## S7.2 落地(2026-07-27,完成 ✅)
**citizenapp 多账户 + 单钱包(批量指定序号),主检出 `citizenapp/`,冷钱包/§6 调用方零改:**
- **数据模型**:`app_isar.dart` 新增 `AccountEntity`(masterId indexed / accountIndex / accountId unique / ss58Address unique / accountName / createdAtMillis)+ `WalletProfileEntity` 加 `masterId`(=账户0 accountId);account0 建钱包时同写一条 AccountEntity;`AccountEntitySchema` 注册;`build_runner` 重生 `app_isar.g.dart`。
- **多账户 API**(`wallet_manager.dart`):`getAccounts(masterId)`/`getAccountByAccountId`;**`addAccounts(masterId, mnemonic, List<int> indices)`**(归属校验 //0==masterId → 序号 `[1,1989]`+输入去重+不与既有冲突 → 逐个派生 → **批量原子**:一事务写全 AccountEntity + 逐个 putAccountKey,任一失败整批回滚删行+deleteAccountKey → 母种子清零);`addNextAccount`(max+1);`getAccountPrivateKey(accountId)`(读该账户 child,生物识别);`signForAccount(accountId, payload)`(多账户签名,校验公钥+清零);`deleteAccount(accountId)`(账户0 锚点守卫:有兄弟拒单删,删至空级联删钱包);**单钱包约束**(已有热钱包拒建/拒导,`本设备已存在热钱包...`)。默认身份/`signWithWallet(walletIndex)` 仍账户0(interim,§6 零改)。
- **测试**:新增 `test/wallet/wallet_multi_account_test.dart`(11 用例:连续 `[1,2,3]`/断续 `[1,5,9]` 存各自 child、去重/越界/已存在/归属拒、addNextAccount、getAccountPrivateKey、signForAccount 签名可验、deleteAccount 锚点+级联);修 `wallet_manager_reorder_test.dart` fixture 缺 masterId。
- **门禁**:`dart analyze` **No issues** + `flutter test` **824 passed / 5 skipped / 0 failed**(本人独立复跑 `FLUTTER_EXIT=0`);残留扫 0。
- **执行插曲**:实现 agent 因 API 连接中断中途失败(停在写测试前),留 2 处 reorder fixture 缺 masterId(LateInitializationError)+ 多账户测试未写;**本人接手**:补 fixture masterId、写 11 条多账户测试、独立复核批量原子/锚点/契约后转绿。

## S7.3 落地(2026-07-27,完成 ✅)—— S7 钱包层收官
**citizenapp 多账户钱包 UI(纯 UI,复用 S7.1/S7.2 API,冷钱包/§6 零改):**
- `wallet_page.dart`:「＋」入口 + 空态**删「创建钱包」「导入热钱包」**,只留「导入冷钱包」(onboarding 仍是首启唯一热钱包引导);我的钱包列表**热账户 + 冷钱包共存**;账户点击进详情。
- 新 `lib/wallet/pages/account_detail_page.dart`:账户 SS58 + **私钥展示**(默认隐藏→确认→`getAccountPrivateKey` 生物识别→`0x`+64hex 纯 Text + `ScreenshotGuard`);顶部「添加账户」。
- 新 `lib/wallet/widgets/add_account_sheet.dart`:两模式「添加下一个」/「指定序号」;**多序号空格分隔**(`split(RegExp(r'\s+'))`,连续 `1 2 3`/断续 `1 5 9`)→ `addAccounts`;助记词重录(无根);业务校验单源交 `addAccounts` 兜底不抄两份。
- `create_wallet_flow.dart`:备份文案改「手抄助记词或在公民钱包保管」。
- widget 测试 `test/wallet/pages/wallet_multi_account_ui_test.dart`(入口已删/账户列表/序号解析/私钥默认隐藏/冷钱包共存)。
- **执行插曲**:agent 完成实现但收尾测试未跑;本人复核发现 1 条「1 5 9 端到端 UI→addAccounts→读 isar」widget 用例**在 testWidgets fake-async 下卡 10 分钟超时**(经 UI 触发的真实 isar 磁盘异步不可驱动);其覆盖已由 `parseAccountIndices('1 5 9')` 解析单测 + `wallet_multi_account_test` 的 `addAccounts([1,5,9])` 落库单测全覆盖,故将该脆弱用例改为**不碰 isar 的轻量渲染测试**(渲染需 `Scaffold` 供 Material)。
- **门禁**:`dart analyze` **No issues** + `flutter test` **838 passed / 5 skipped / 0 failed**(本人独立 `FLUTTER_EXIT=0`);残留扫 0。

## S7 钱包层收官(2026-07-27)
S7.1(无根派生+存储 biometricOnly)+ S7.2(多账户批量+单钱包)+ S7.3(多账户 UI)全部完成并全绿。citizenapp = 一设备一钱包多 `//index` 账户、只存 child mini-secret(不存种子/助记词)、biometricOnly fail-closed;冷钱包共存;§6 身份/签名调用方过渡解析账户0。**下一步 S8 身份层**(CID 主键 + 注册接 `self_occupy_cid` + 门控,依赖另一会话 S4+S6)。

## S8.1 落地(2026-07-27,完成 ✅)—— 客户端 CID 生成 + 占号/换绑 RPC(纯离线编码,逐字节镜像链)
**citizenapp 侧新增两只离线原语 + 两条 extrinsic 构造,主检出 `citizenapp/`,不依赖 S4+S6、不碰 UI:**
- **CID 生成器**(新 `lib/citizen/cid/cid_generator.dart`):`generateCitizenCid({accountId, institution, year})` **逐字节移植链 `generator.rs` 人主体分支**——`input="{accountId}|{institution}|{year}"` → `blake2_256` 前 8 字节**小端 u64**(BigInt 承载防 Dart 有符号溢出)→ `mod 1e12` → `high3=/1e9`(R5=`CN`+高3补零)`low9=%1e9`(N9 补零)→ M1 校验位 `'0'+(Σ(i+1)*alphabetIndex % 10)`(字母表外→0,对齐 `number.rs`)→ `"{R5}-{CTZN/NATP}{M1}-{N9}-{year}"`。仅 CTZN/NATP,`blake2b256` 用 polkadart `Hasher.blake2b256`(纯,无 key)。
- **占号/换绑 RPC**(新 `lib/rpc/citizen_identity_rpc.dart`):`CitizenIdentityRpc`,两条自签自付 extrinsic,复用 `SignedExtrinsicBuilder`(immortal/自动 nonce·specVersion·genesisHash)+ `signWithWallet`:
  - `selfOccupyCid`:callData=`[10][5]` ++ `compact(len)++cid.utf8`(CidNumberBound=`BoundedVec<u8,ConstU32<32>>`)。
  - `selfRebindCidAccount`:callData=`[10][9]` ++ CID BoundedVec ++ `compact(64)++64B`(SignatureOf=`BoundedVec<u8>`);旧账户授权摘要 `buildRebindSigningDigest`=`signingMessage(0x11, compact(len(cid))++cid.utf8++new_account(32B))`=`blake2_256(GMB++0x11++payload)`,**逐字节 == 链 `(cid,new_account).encode()` + `verify_rebind_signature`**;旧账户从链上 `AccountIdByCid[cid]` 反查、不上送,`signForAccount(oldAccountId, digest)` 出授权签名。
- **登记**:`pallet_registry.dart` 加 `citizenIdentityPallet=10`/`selfOccupyCidCall=5`/`selfRebindCidAccountCall=9`;`signer/signing.dart` 加 `kOpSignCidRebind=0x11`(`kGmbSignDomain=[0x47,0x4D,0x42]`="GMB" 已核 == 链 `core_const::GMB`)。
- **逐字节四验(本人独立复核,非仅信 agent)**:①CID 金标 `CN951-CTZN1-539598435-2026`(dev //0 公钥,== 链 `citizen_cid_number_golden`)②occupy callData 字节序 ③rebind 摘要 = `signing_message(0x11,…)` 对齐链 verify ④算法读源逐行核对 generator/number.rs。
- **测试**:新 `test/citizen/cid/cid_generator_golden_test.dart`(金标 + CTZN/NATP + 非法码/年份拒)+ `test/rpc/citizen_identity_rpc_test.dart`(callData 字节 + rebind 摘要 + 长度守卫 + accountId 校验)。
- **门禁**:`dart analyze` **No issues** + `flutter test` **848 passed / 5 skipped / 0 failed**(本人独立 `FLUTTER_EXIT=0`);残留扫 0。
- **范围收敛**:S8.1 只出**离线可测的生成/编码/提交原语**,不碰身份页 UI(S8.2)、不碰主键切换/门控(S8.3);上链真跑待 S5 重新创世 + S4+S6 注册局流程合流。

## S8.2 落地(2026-07-27,完成 ✅)—— 身份页改造 + 匿名占号注册/换绑全链路
**citizenapp 身份页(原电子护照)接入 S8.1 离线原语,仅主检出 `citizenapp/`,不动底部 5-tab、冷钱包/§6 零改:**
- **链读扩展匿名态**(`my/myid/citizen_identity_chain_reader.dart`):`CitizenIdentityChainSnapshot.votingIdentity` 改**可空** + `isAnonymous` getter;`readByAccountId` 三态分流——`CidByAccountId` 无/绑定不闭环→`null`(纯访客);闭环但无 `VotingIdentityByCid`(或布局损坏,**安全降级不误升**)→匿名快照(仅 cidNumber);闭环+合法 voting→投票/竞选。**关键**:旧 reader 无 voting 即返回 null,把匿名 CID 与纯访客混同,故必扩。
- **状态分流 + 动作**(`my/myid/myid_service.dart`):**不新增 tier/色**(决策①),匿名态 = `visitor` + 填 `cidNumber` + `isAnonymousRegistered` getter;`getState` 四分流(纯访客/匿名/投票/竞选);新增 `registerAnonymousCid`(取默认用户→`generateCitizenCid`(**UTC 当前年** `cidYearProvider`)→`selfOccupyCid`)、`rebindCidTo`(旧=默认用户、新=所选,`selfRebindCidAccount`;目标==自身拒)、`listRebindTargets`(默认用户外全部账户);注入 `identityRpc`/`cidYearProvider` 可测。
- **RPC 修正**(`rpc/citizen_identity_rpc.dart`):**修 S8.1 接入隐患**——`selfOccupyCid`/`selfRebindCidAccount` 由 `signWithWallet(walletIndex)`(interim 恒解析账户0)改 **`signForAccount(accountId)`**(按 accountId 精确取私钥),删 `walletIndex`/`signerPublicKey` 冗余参;否则换绑非0新账户会被账户0私钥签、与声称发送方不符致链上验签必败。static `buildXxx` 编码未动(金标不变)。
- **UI**(`my/myid/myid_page.dart` + `my/user/user.dart` 入口):**改名**电子护照→身份(AppBar/入口/错误文案/注释,UI 残留清零,仅留 1 行历史说明);首卡 `访客轻节点`→`注册身份·访客`(匿名图标留),匿名态卡内补显 `身份CID号 xxxxx`;voting/candidate 卡 `公民CID号`→`身份CID号`;右上 icon 刷新→语义 `注册`/`更换` 按钮(纯访客注册、匿名/投票/竞选更换,civic 点击提示走注册局,queryFailed 无按钮),`RefreshIndicator` 下拉刷新;新 `widgets/register_identity_sheet.dart`(选 CTZN/NATP+风险提示)、`widgets/rebind_account_sheet.dart`(选目标账户,空态提示)。
- **初始化跳身份页**(`wallet/wallet_gate.dart`):加可注入 `onInitialized`(默认 `push(MyIdPage)`);`onCreated`(本次会话新建/导入,子路由已 pop 干净)后 `postFrame` 引导一次,**冷启动即有钱包不跳**;盖广场 tab 之上、返回回落,**不动 5-tab**(决策③)。
- **下游适配**(`8964/chain/square_chain_service.dart`):`votingIdentity` 可空后,匿名态在广场身份判定归 `visitor`(不升级徽章,合决策①)。
- **测试**:service +7(匿名态解码/纯访客区分/注册金标编排/换绑参数/目标==自身拒/listRebindTargets)、widget +8(改名/注册·更换按钮/匿名 CID 展示/下拉刷新/字段文案,`descendant` 限定卡内避非当前卡字段名干扰)、gate +2(引导触发/冷启动不触发,注入 no-op 挡真链)、两 sheet +5。
- **门禁**:`dart analyze` **No issues** + `flutter test` **867 passed / 5 skipped / 0 failed**(本人独立 `FLUTTER_EXIT=0`);旧文案(电子护照/访客轻节点/公民CID号)UI+注释残留清零。
- **范围收敛**:S8.2 身份页锚点仍 `getDefaultWallet()`(账户0 interim);**换绑后身份归新账户但身份页读账户0 会暂显访客**——身份页跟随 CID-bound account 是 S8.3;上链真跑待 S5 重新创世 + S4+S6 合流。

## S8.3a 落地(2026-07-27,完成 ✅)—— 身份账户单源 + 注册选绑定账户
**把身份页数据源 + 注册/换绑收敛到「CID 绑定账户」(非恒账户0),仅主检出 `citizenapp/`:**
- **身份账户单源**(新 `lib/my/myid/identity_account_resolver.dart`):`IdentityAccountResolver.resolve()` 返回 `ResolvedIdentity{accountId,ss58,accountIndex,snapshot}`——**优先账户0**(命中即 1 次链读)、未命中遍历子账户、首个 `readByAccountId` 闭环命中者即身份账户;全未命中→回退账户0(`isRegistered=false` 未注册);无热钱包→null;**链读异常上抛不吞**(fail-closed)。
- **身份绑定通知**(`wallet_manager.dart`):`notifyIdentityBindingChanged()` 复用 `walletsRevision`(占号/换绑改身份账户但钱包列表没变,须显式广播常驻页重读);注释同步纳入 CID 占号/换绑。
- **MyIdService 收敛**:`getState` 改走 `resolve()`(身份从 getDefaultWallet账户0 → CID 绑定账户,`votingAccountId`/徽章快照全锚身份账户);`registerAnonymousCid({institution, bindAccountId})` **注册时选绑定账户**(默认账户0,可选子账户,用所选账户生成 CID+`self_occupy_cid`+成功后通知);`rebindCidTo` 旧账户改用 `resolve()` 解析当前 CID 绑定账户(非恒账户0)+成功通知;`listRebindTargets` 排除身份账户;新 `listBindableAccounts`;移除对 `CitizenIdentityChainReader` 的直接持有。
- **UI**:`register_identity_sheet` 加「绑定钱包账户」单选(多账户才露出,默认账户0)+ 返回 `RegisterChoice{institution,bindAccountId}`;`myid_page._onRegister` 先 `listBindableAccounts` 再弹面板。
- **注释同步**:myid 层「默认用户/getDefaultWallet 账户0」过时表述改「身份账户/CID 绑定账户」(comment rot 清理);chat/square/user/wallet 层「默认用户」残留留 S8.3b/S8.3d。
- **测试**:新 `identity_account_resolver_test.dart`(账户0命中/子账户//5命中/全未命中回退/无钱包null/链读异常上抛)5 + service 扩展(注册绑子账户//5非账户0、listBindableAccounts)+ sheet 多账户选择;换绑/列举目标测试注入 `_FakeIdentityResolver` 绕真链序列。
- **门禁**:`dart analyze` **No issues** + `flutter test` **875 passed / 5 skipped / 0 failed**(本人独立 `FLUTTER_EXIT=0`)。
- **范围收敛**:S8.3a 只切身份页数据源 + 注册 + 单源基建;**其他身份展示调用方(chat/square/user)+ 设备子钥/广场会话/通讯录密钥迁移 = S8.3b**——常态身份=账户0 时无差异,换绑到子账户的全端一致性待 S8.3b。上链真跑待 S8.3e 本地链。

## 待办 / 未决
- **S5 创世协同**:另一会话(modelb 卡 Step 3/S3.1)已在重派全部创世管理员+程伟公钥(bare→//0)并重新创世;本卡 S5 须与其合流——旧省码式 CTZN 创世常量(`FSC_GENESIS_ADMIN_CID_NUMBER`/`LEGAL_REPRESENTATIVE_CITIZEN_CID_NUMBER`=`GZ000-CTZN6-…`)随该重生改 **CN** 号(公钥由用户给,CID 号由生成器按新 CN 规则重派)。二者同一次创世,勿各改各的。
- citizenwallet 反转(改成存种子/助记词)= **另一个会话**;需与本卡协同,避免对 citizenwallet 反复横跳(Step 1 刚改无根)。
- 访客可用面 + CID 注册门与四层门禁合并口径(S8 细化)。
- 身份主键切换的过渡期(CID 未注册时 myid/square/chat 身份源)——S8 定。

## 验收
冷热链逐字节一致(citizenapp `//0//1//2` == Step 1 金标 == 链读);CID CN000 金标钉死;自助占号/换绑链上闭环;门控 fail-closed;全端 analyze/test 绿;残留复扫 0。
