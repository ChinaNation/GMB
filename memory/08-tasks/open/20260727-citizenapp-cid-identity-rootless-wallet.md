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
- **S7 citizenapp 钱包层**:无根多账户重构(独立于链)。**← 本窗口进行中(2026-07-27,grounding agent 摸底中)**。
- **S8 citizenapp 身份层**:改名 + 注册按钮 + 自助占号交易 + CID 主键切换 + 门控。

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

## 待办 / 未决
- **S5 创世协同**:另一会话(modelb 卡 Step 3/S3.1)已在重派全部创世管理员+程伟公钥(bare→//0)并重新创世;本卡 S5 须与其合流——旧省码式 CTZN 创世常量(`FSC_GENESIS_ADMIN_CID_NUMBER`/`LEGAL_REPRESENTATIVE_CITIZEN_CID_NUMBER`=`GZ000-CTZN6-…`)随该重生改 **CN** 号(公钥由用户给,CID 号由生成器按新 CN 规则重派)。二者同一次创世,勿各改各的。
- citizenwallet 反转(改成存种子/助记词)= **另一个会话**;需与本卡协同,避免对 citizenwallet 反复横跳(Step 1 刚改无根)。
- 访客可用面 + CID 注册门与四层门禁合并口径(S8 细化)。
- 身份主键切换的过渡期(CID 未注册时 myid/square/chat 身份源)——S8 定。

## 验收
冷热链逐字节一致(citizenapp `//0//1//2` == Step 1 金标 == 链读);CID CN000 金标钉死;自助占号/换绑链上闭环;门控 fail-closed;全端 analyze/test 绿;残留复扫 0。
