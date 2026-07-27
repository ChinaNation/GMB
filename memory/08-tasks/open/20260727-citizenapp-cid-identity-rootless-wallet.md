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
- **D3 公民 CID 去地域化**:公民 CID 生成**统一 `CN` 国家码开头 + 3·4·5 位固定 `000`**(R5=`CN000`)。不再用省码。居住省/市仍单独存于 `VotingIdentity`/`CidRecord` 供公民档与投票作用域用(与 CID 号解耦)。[[unify-means-zero-exceptions]] 零例外。

## 链侧现状(侦察定论,`runtime/misc/citizen-identity/src/lib.rs`)
- 存储**已全在位**:`CidRegistry`(CID→记录,含 `commitment[32]`/residence/status Active·Revoked)、`AccountIdByCid`+`CidByAccountId`(双向绑定闭环 `bind_account_id` L1407 / `ensure_account_id_binding_available` L1375)、`VotingIdentityByCid`、`CandidateIdentityByCid`、四级人口计数。**匿名 CID = 有 CidRegistry+双向绑定、无 VotingIdentityByCid**,存储天然可表达。
- 缺口:①全部 extrinsic **注册局门**(`CitizenIdentityAuthority::can_manage_voting_identity`,CREG/FRG);无自助。②`occupy_cid`(call 6,L1085)**只写号不绑账户**;账户到 `register_voting_identity`(call 0)才绑。③**无换绑**(`update_*` 禁改账户 `ensure_current_account_id_binding` L1395)。
- CID **客户端生成**(`primitives/cid/generator.rs`),链只格式+查重仲裁(`do_occupy_cid` L1237);现个人码强制市 `000`、R5=省+000、N9=hash(公钥|码|省名|000|年)。**D3 改为 R5=CN000。**
- 费:citizen_identity 全走 `institution_onchain_route`(机构付费,`configs.rs fee_route` L431+);**自助需新自付费 arm**(`signer_onchain_route` 式,签名人自付,min 10)。

## 跨模块设计

### ① 链 citizen-identity(Blockchain)
- **CID 格式改 D3**:`primitives/cid/{generator.rs,number.rs,code.rs}` 公民 CID R5 统一 `CN000`;新增/放行 `CN` 国家前缀通过 `parse_cid_number_parts` 校验;N9 hash 输入去省名(`hash(公钥|CTZN|CN|000|年)`,最终公式落地时钉金标)。**单源,冷热链共用。**
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
- **S1 链**:公民 CID 统一 CN000(primitives/cid + 链上校验)+ 金标向量。**首小步候选**(最小、共享地基、无需创世)。
- **S2 链**:`self_occupy_cid`(自助+自付费+占即绑+匿名)+ op_tag + 测试。
- **S3 链**:`rebind_cid_account`(自助轮换,commitment=hash(pubkey))+ 测试。
- **S4 链**:`occupy_cid` 注册局占即绑 + register_voting_identity 变可选升级 + 测试。
- **S5 链**:重新创世 + 注册表重生 + 冷热链一致性。
- **S6 onchina**:占号绑账户 + 第2步两选项。
- **S7 citizenapp 钱包层**:无根多账户重构(独立于链,可并行)。
- **S8 citizenapp 身份层**:改名 + 注册按钮 + 自助占号交易 + CID 主键切换 + 门控。

## 待办 / 未决
- citizenwallet 反转(改成存种子/助记词)= **另一个会话**;需与本卡协同,避免对 citizenwallet 反复横跳(Step 1 刚改无根)。
- 访客可用面 + CID 注册门与四层门禁合并口径(S8 细化)。
- 身份主键切换的过渡期(CID 未注册时 myid/square/chat 身份源)——S8 定。

## 验收
冷热链逐字节一致(citizenapp `//0//1//2` == Step 1 金标 == 链读);CID CN000 金标钉死;自助占号/换绑链上闭环;门控 fail-closed;全端 analyze/test 绿;残留复扫 0。
