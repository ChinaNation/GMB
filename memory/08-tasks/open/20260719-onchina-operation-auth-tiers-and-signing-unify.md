# OnChina 操作三档统一 + 删平台钥 + 冷签改签真实载荷 + 登录 QR 机构字段

- 创建日期：2026-07-19
- 状态：Phase 1+2 完成并全面复查通过。**平台钥彻底删除**(登录自证+机构 call1/6-9 全去凭证→管理员钱包签+链上 CID+岗位验);**账户增删 UI 已移进机构工作区**(`AccountManageSection` 挂 Private/Generic/Judicial workspace,注册局详情页 `canDelete=false` 只读);**残留清扫净**(6 个凭证死 Error 变体 InvalidCidInstitutionSignature/RegisterNonceAlreadyUsed/EmptyIssuerCidNumber/EmptyScopeProvinceName 删除、crypto/sr25519.rs+core/secret.rs 整删、注销 DB DROP、前端死类型清)。验证:cargo test public/private-manage 14+14、onchina 131、cargo check citizenchain+onchina 48.79s、tsc -b 全绿;live code 零残留。剩(另卡):Phase 3 冷签改签真实载荷、Phase 4 岗位码入链校验细化、Phase 5 登录 QR 机构字段。
- 范围：OnChina 全部管理员操作的鉴权分档与签名实现；登录 QR 与登录签名；平台系统签名钥；CitizenWallet 登录链路
- 所属模块：Blockchain Agent（OnChina 后端 + runtime）、Mobile Agent（CitizenWallet）
- 依赖：`memory/08-tasks/20260719-institution-role-permission-unify.md`（ADR-039 岗位码=权限载体）
- 关联：`memory/08-tasks/open/20260718-citizen-onchain-single-signature-flow.md`（公民一次签名）
- 触及 runtime：是（`configs.rs` 验签口径、`primitives/sign.rs` op_tag）。按既有约定，**runtime 改动在执行阶段单独二次确认**。

---

## 一、目标模型（定稿）

### 1.1 权限模型
权限 = 机构 CID + 岗位码 + 管理员。管理员持公私钥对，岗位码由 ADR-039 承载。
**唯一真源 = 链上管理员注册表**（`RuntimeInstitutionAdminQuery::is_institution_admin`，`runtime/src/configs.rs`）。onchina 本地库只作投影/缓存，不得成为第二真源。

### 1.2 操作三档（读 / 本地写 / 链上写）

| 档 | = 现 enum | 触发手段 | 覆盖操作 |
|---|---|---|---|
| 登录态 | `Session` | 仅会话 | 登录会话、**只读查询** |
| 本地写 | `Passkey` | 会话 + passkey 断言 | 只改 onchina 本地库、不动链上真源的写 |
| 链上写 | `PasskeyColdSign` | 会话 + passkey + 钱包对**真实链载荷**签一次 | 产生 extrinsic 或改链上真源集合 |

**铁律**：任何操作必须落入三档之一，否则拒绝；写操作 ≥ passkey；`Session` 仅限只读；passkey 首次登录绑定，未绑定不得开展业务。

### 1.3 两个签名原语（手段，正交于三档）
- **passkey 断言（WebAuthn）**：本地写的手段 + 链上写的 passkey 半。
- **钱包 sr25519，签真实链载荷本身**：链上写的链半，链上按 CID + 岗位码 验。

三档由两原语搭出：登录态 0 原语 / 本地写 1（passkey）/ 链上写 2（passkey + 钱包）。

### 1.4 一次签名（禁来回签）
一个操作 = passkey 一次 + 钱包一次 = 一次；涉及公民的操作再叠加公民本人钱包一次。删除以下"第二签名/第二真源"：
- **平台系统签名钥 `ONCHINA_SIGNING_SEED_HEX`**：登录自证 + 机构凭证两处用法，**整把删**。机构凭证改由注册局管理员本人钱包签名（链上验签口径 `is_institution_admin` 不变）。
- **`onchina_admin_governance` 治理文本 grant 冷签**：并入"钱包签真实链载荷"那一次；grant 退化为服务端一次性防重放记录，不再单独签一次文本。
- **登录文本签名第三套 `build_signature_message`**：并入 op_tag 统一域（新增 `OP_SIGN_ONCHINA_LOGIN`）。

---

## 二、目录架构（受影响模块 + 注释）

```text
citizenchain/
├── onchina/src/
│   ├── auth/
│   │   ├── operation_auth.rs        # ★三档定义 + 动作→档映射;把写操作移出 Session,Session 仅只读
│   │   ├── actions.rs               # ★冷签流程:删治理文本 grant 冷签(build_sign_request 那处),
│   │   │                            #   改为对真实链载荷冷签一次;grant 仅留服务端 nonce/consumed
│   │   ├── action_sign.rs           # ★signed_payload_text(domain="onchina_admin_governance") 删除,
│   │   │                            #   冷签对象改为真实链载荷(extrinsic SignedPayload / 凭证摘要)
│   │   ├── login/
│   │   │   ├── signature.rs         # ★删 build_login_qr_system_signature(平台钥自证)
│   │   │   └── qr_login.rs          # ★删 sys_pubkey/sys_sig;登录 QR 携带 机构CID+简称;
│   │   │                            #   登录验签仍走 onchain_gate(链上管理员集合)
│   │   └── passkey/mod.rs           # ○只读确认:passkey 在首次登录绑定;本地写/链上写消费断言
│   ├── core/
│   │   ├── chain_runtime.rs         # ★删 sign_runtime_digest + 机构凭证平台钥签名;
│   │   │                            #   凭证摘要改由注册局管理员钱包签(finish_institution_credential 等)
│   │   ├── chain_submit.rs          # ○唯一 extrinsic 提交通路(已"只签一次");链上写落点,基本不改
│   │   └── qr/
│   │       ├── mod.rs               # ★login_request_body 改(去 sys、带 CID/简称);
│   │       │                        #   build_signature_message 并入 op_tag 域
│   │       └── sign_request.rs      # ★冷签 QR 载荷改为真实链载荷,不再是治理文本
│   ├── institution/
│   │   ├── subjects/admin.rs        # ★update_institution 重判:改 cid_full_name(链上单源)→链上写;
│   │   │                            #   纯本地字段(文档/展示)→本地写;拆分核实
│   │   └── admins/mod.rs            # ★机构治理凭证调用点:改由注册局管理员签(原 build_..._credential)
│   └── main.rs                      # ★删 ONCHINA_SIGNING_SEED_HEX 加载与校验
├── runtime/
│   ├── src/configs.rs               # △verify_institution_*:signer 仍按 is_institution_admin 验,
│   │                                #   接入岗位码校验(依赖 ADR-039);runtime 改动二次确认
│   └── primitives/src/sign.rs       # △新增 OP_SIGN_ONCHINA_LOGIN;登录并入统一签名域
│
citizenwallet/lib/
├── login/login_qr_handler.dart      # ★删 verifySystemSignature;解析 CID/简称;展示岗位名(本端管理员档);
│                                     #   登录签名改走统一域(op_tag),四端逐字节一致
├── qr/signature_message.dart        # ★build_signature_message 并入统一域(或删,改 signing.dart op_tag)
└── ui/login_sign_page.dart          # ★删系统签名校验调用;登录页展示机构 CID/简称/岗位名

图例:★=改动  ○=只读核对/基本不动  △=触及 runtime,执行前二次确认
```

---

## 三、分阶段执行方案

> 每阶段独立可验收;runtime 阶段(P4 岗位码、P5 登录 op_tag)执行前单独二次确认。

### Phase 1 — 三档边界钉死 + 逐动作重新归档
- `operation_auth.rs`：`Session` 收窄为纯只读;所有写动作至少 `Passkey`。
- 按「四、逐动作归档表」重判每个 `AdminActionType`。
- `update_institution`（`subjects/admin.rs`）按写入目标重判并拆分（见归档表）。
- 验收：无任何写动作停留在 `Session`;新增动作漏标编译失败(穷尽 match 保持)。

### Phase 2 前置发现(登录信任根 + passkey 绑定现状)
- onchina **每个会话都由钱包签名换取**:两条登录 `handler.rs:320` / `qr_login.rs` 均 `verify_admin_signature`(sr25519)+ `onchain_gate`(链上管理员集合);无 passkey/密码登录路径(passkey 只有 register/assert,不签发会话)。
- **passkey 绑定已实现且已满足"绑定必须钱包签名"**:`register_begin`/`register_finish`(`passkey/mod.rs`)+ `getPasskeyStatus` + `usePasskeyRegistration`。因会话必自钱包冷签登录,绑定在该会话内完成 = 钱包签名已授权绑定。**这是既有功能,非本卡新增工作**(用户 2026-07-19 仅提示注意)。删平台钥后此不变量保持:登录/绑定信任根 = 钱包签名验链上管理员集合。

### Phase 2 — 删平台钥(登录自证 + 机构凭证)
- 删 `build_login_qr_system_signature`、`sign_runtime_digest`、`main.rs` env 加载。
- 机构注册/创建/治理/注销凭证：摘要(`institution_*_message`)改由**注册局管理员钱包签名**;链上 `verify_institution_*` 口径不变(`is_institution_admin`)。
- 钱包侧删 `verifySystemSignature` 及调用。
- 验收：全仓无 `ONCHINA_SIGNING_SEED_HEX` 残留;机构注册/注销链上验签用注册局管理员签名回归通过;passkey 绑定不变量(钱包签名登录会话内绑定)保持。

**Phase 2 实建(设计修正 + 执行结果):**

原"凭证改管理员冷签"方案被验证行不通(钱包无 0x13/0x14 哈希域签名分支,且治理路径会造成同一操作两签)。**改为:去掉机构操作的独立凭证,链上按 extrinsic 签名者的 CID+岗位授权。**

- **2a 登录自证删除**:✅ 已完成(去 `build_login_qr_system_signature` + 登录 QR sys 字段 + 钱包 `verifySystemSignature`)。`cargo check` + `tsc` 通过。
- **2b 机构操作去凭证 + 删平台钥(runtime)**:✅ **部分完成(2026-07-19,已验证)**——
  - call 6/7/8/9(改名/加账户/治理/登记管理员):删嵌入凭证 + 平台钥,改管理员钱包签一笔 extrinsic + 链上 `is_institution_admin(who)`(+岗位 call8 proposer_role_code、call9/6/7 FRG省专员)。原凭证证实是"submitter=注册局在册管理员"的冗余,删除不弱化。runtime `verify_institution_registration/creation/governance` + `institution_*_message(除close)` + `can_register_institution`(凭证版)全删,留 `can_register_institution_origin`。onchina 两 builder + `finish_institution_credential` 删。`cargo test public/private-manage/onchina` 全绿;`cargo check` 0.56s 过;安全回归测试改写覆盖保留的 close 验签。
  - **call 1(账户注销)也已去凭证(2026-07-19 收尾,已验证)**:模型定案——自定义账户增删归**机构自管**(注册局只管注册),协议账户永久不可删(`is_closable_institution_account` 守卫已强制)。故 close 的注册局审批凭证是多余层,删之非弱化(保留 `is_institution_admin(who)`+协议守卫+内部投票+beneficiary)。runtime 删 `verify_institution_account_close`+整个 `CidInstitutionVerifier` trait+`institution_account_close_message`+close 4 凭证参数+`UsedDeregisterNonce`;onchina 删 `sign_runtime_digest`/`build_institution_deregistration_credential`/`runtime_signing_context`/`crypto/sr25519.rs`(整删)/`core/secret.rs`(整删)/`InstitutionAccountDeregister` 动作/注销凭证 DB(DROP TABLE)/main.rs 加载/scripts。前端删死类型 `INSTITUTION_ACCOUNT_DEREGISTER`。**平台钥 live code 零残留;cargo test public/private-manage/onchina 全绿;`cargo check` 10.91s 过;`tsc -b` 0 错。**
  - 残留常量 `OP_SIGN_INST`/`OP_SIGN_DEREGISTER`:无 message 构造入口,属四端金标向量注册表成员,删需四端同步,保留并注明。

**平台钥彻底删除已达成。** 剩:账户增删 UI 从注册局详情页(`PrivateDetailLayout`/`GovDetailPage`)移进机构自己工作区(`frontend/workspace/PrivateInstitutionWorkspace`),注册局详情页降只读——后端已按 `is_institution_admin` 授权,纯搬 UI。

### Phase 3 — 冷签改签真实链载荷(一次签名)
- 删 `signed_payload_text(onchina_admin_governance)` 治理文本冷签;`actions.rs` 冷签 QR 载荷改为真实链载荷(extrinsic SignedPayload 或凭证摘要)。
- `security grant` 退化为服务端一次性 nonce/consumed 记录,不再承载"第二次签名"。
- 验收：每个链上写操作只需管理员冷签一次;grant 与链上签名同源。

### Phase 4 — 岗位码入链上校验(对接 ADR-039)
- `configs.rs` `verify_institution_*` / 动作授权接入岗位码校验:某动作要求某岗位码,验签时确认签名者在该机构持该岗位码。
- **不在本卡重复实现岗位码存储/模型**(归 ADR-039),本卡只在验签处调用其接口。
- runtime 改动，执行前二次确认。验收：错误岗位码的签名被链上拒绝。

### Phase 5 — 登录 QR 机构字段 + 登录签名统一
- 登录 QR 载荷 `d` 携带 机构 CID + 简称(节点绑定已知);钱包解析展示 + 展示本端岗位名。
- 新增 `OP_SIGN_ONCHINA_LOGIN`,登录签名从文本第三套并入 `signing_message(op_tag)`;机构 CID + 岗位绑入签名原文,简称仅展示。
- 四端逐字节一致(onchina Rust + node/onchina 前端 TS + citizenwallet Dart)。
- runtime 触及(primitives/sign.rs),执行前二次确认。验收：登录金标向量四端一致;篡改机构字段导致验签失败。

---

## 四、逐动作归档表(现状 → 目标)

| AdminActionType | 现档 | 目标档 | 写入目标 | 备注 |
|---|---|---|---|---|
| 只读查询(列表/详情) | Session | 登录态 | 无 | 一致 |
| InstitutionUpdate | Session | **链上写 / 本地写(拆)** | cid_full_name=链上单源;文档/展示=本地 | ★错配,必拆分 |
| InstitutionUploadDocument | Session | **本地写** | onchina 本地存储 | ★写操作需 passkey |
| InstitutionCreate | ColdSign | 链上写 | 链上(凭证) | 一致 |
| InstitutionCreateAccount | ColdSign | 链上写 | 链上 | 一致 |
| InstitutionDeleteAccount | ColdSign | 链上写 | 链上 | 一致 |
| InstitutionAccountDeregister | ColdSign | 链上写 | 链上(凭证) | 一致;凭证改管理员签 |
| CreateCityRegistry / DeleteCityRegistry | ColdSign | 链上写 | 链上 Active 集 | 一致 |
| InstitutionDeleteDocument | ColdSign | **核实(本地→本地写)** | onchina 本地存储 | ○删本地文档是否需冷签,可降 passkey |
| NodeBindingUnbind | ColdSign | **核实(本地→本地写)** | onchina 本地绑定 | ○本地解绑,可降 passkey |
| ProposeEnactLaw/AmendLaw/RepealLaw | ColdSign | 链上写 | 链上 extrinsic | 一致 |
| CastRepresentativeVote/ReferendumVote | ColdSign | 链上写 | 链上 | 一致 |
| ExecutiveSign/OverrideSign/GuardVote | ColdSign | 链上写 | 链上 | 一致 |
| ProposePersonnel/ProposeBudget | ColdSign | 链上写 | 链上 | 一致 |
| CitizenOnchainPush | Passkey | 本地写(管理员) + 公民 overlay | 链上(公民签) | 管理员 passkey 授权 + 公民钱包签一次 |

---

## 五、签名统一后的最终形态

- 删除前(碎):extrinsic 签名 + op_tag 业务签名 + 登录文本签名 + 治理文本 grant 冷签 + 平台钥凭证 + passkey。
- 统一后(两原语):
  1. **passkey 断言** —— 本地写 + 链上写的 passkey 半。
  2. **钱包 sr25519 签真实链载荷** —— 链上写的链半;真实载荷 = Substrate extrinsic SignedPayload 或链上验的凭证摘要(op_tag 域);登录也并入 op_tag。
- 平台钥、治理文本 grant、登录文本第三套 —— 全删。真源唯一 = 链上管理员注册表(CID + 岗位码 + 管理员钥)。

---

## 六、必须遵守
- 三档之外一律拒绝;写操作 ≥ passkey;`Session` 仅只读。
- 一个操作只签一次(passkey 一次 + 钱包一次);公民操作叠加公民钱包一次。
- 平台钥整把删,不保留任何自动签名兜底。
- onchina 本地库不得成为链上真源的第二写入点(update_institution 重判即为此)。
- 登录签名四端逐字节一致;不新增 QR kind、不新增协议名,登录续用 a=1。
- 链开发期无用户:彻底改零残留,不做兼容/迁移。
- runtime 改动(P4/P5)执行前单独二次确认。

## 七、输出物
- 代码 + 中文注释。
- 测试:三档鉴权分支、机构凭证改管理员签回归、冷签一次到位、登录金标向量四端一致、错误岗位码链上拒绝。
- `memory/` 回写:本卡进度、`qr-protocol-spec.md` 登录字段、与 ADR-039 对接说明。
- 残留清理:`ONCHINA_SIGNING_SEED_HEX` / `sys_pubkey` / `sys_sig` / `verifySystemSignature` / `signed_payload_text(onchina_admin_governance)` 全删。

## 执行进度

### Phase 1 归档决定(用户已确认 + 前端事实修正)
- `InstitutionUpdate`:改的是 `cid_full_name`/法人/所属法人(链上注册凭证签名字段=链上单源),且前端本就走冷签 → 归**链上写(PasskeyColdSign)**。纯本地展示字段(若有)在 Phase 2/3 再拆出为本地写。⚠️ 修正:此前误标 Passkey,已改回 PasskeyColdSign;同时修好后端(Session)/前端(冷签)既存不一致(该操作原状前端会抛错)。
- `InstitutionUploadDocument`:本地写(Passkey)。同样原为后端 Session / 前端冷签不一致,改 Passkey + 前端 passkey 一并修好。
- `InstitutionDeleteDocument` / `NodeBindingUnbind`:确认降为本地写(Passkey);二者均纯本地(`apply_node_binding_unbind_conn` 只动本地库、删文档只动本地存储)。

### Phase 1 后端已完成(已验证)
- `onchina/src/auth/operation_auth.rs`:三档语义改为 读/本地写/链上写;`auth_type()`:`InstitutionUploadDocument`(Session→Passkey)、`InstitutionDeleteDocument`、`NodeBindingUnbind`(ColdSign→Passkey)归本地写;`InstitutionUpdate`(Session→PasskeyColdSign)归链上写;删 `is_session()`;保留三档 enum + `operation_auth_has_exactly_three_tiers` 测试。
- `onchina/src/auth/actions.rs`:删 `is_session()` 三处调用;commit 流程保留对 `challenge.auth_type == Session` 的防御性拒绝;`require_admin_security_grant` 去掉只会话分支,写动作一律 ≥ passkey。
- 验证:`cargo check -p onchina --tests` 通过;`cargo test -p onchina operation_auth` 4 测试全绿。

### Phase 1 前端已完成(已验证)
- `admins/securityApi.ts`:新增 `passkeySubmitHeaders(auth)`(本地写档:只带 `X-Passkey-Assertion`,不走 prepare/扫码/commit)。
- `docs/api.ts`:`uploadDocument`/`deleteDocument` 去掉 `securityGrant` 参数,改 `passkeySubmitHeaders`。
- `docs/DocsLibrary.tsx`:上传/删除去冷签直接调 api;移除 `createScanSignGrant` prop 及 `AdminActionType/AdminSecurityGrantOutput` import。
- `private/PrivateDetailLayout.tsx`、`gov/GovDetailPage.tsx`:两处 `<DocsLibrary>` 去掉 `createScanSignGrant` prop(`PrivateDetailLayout` 自身机构更新仍用冷签,prop 保留)。
- `NodeBindingUnbind`:全仓无前端冷签调用点,后端改档即可,无前端配套。
- 验证:`tsc -b` EXIT=0;docs 前端无 `createScanSignGrant/securityGrant` 残留。

### Phase 1 收尾:剩余(留 Phase 2/3)
- `InstitutionUpdate` 链上字段拆分(纯本地展示字段拆出为 Passkey):随 Phase 2/3 链上写通路一并落地。

### Phase 2a 完成(删登录自证,已验证)
- 后端已删除平台系统签名；后续登录流程进一步收口为先扫描 `k=3 user_contact` 确定目标账户，再生成 `u` 非空且指向该账户的定向登录请求，payload 仅为 `system=onchina`。
- 钱包(citizenwallet):`login/login_qr_handler.dart` 删 `verifySystemSignature` + `_verifySr25519Utf8`/hex 工具,`_loginData`→`_loginSystem`(只取 system);`ui/login_sign_page.dart` 删系统签名校验分支。
- 验证:onchina `cargo check` 0 警告;citizenwallet 登录文件 `flutter analyze` 无问题(全量剩 2 个既存 test 问题,与登录无关)。
- 信任根不变:登录 = 管理员钱包签名验链上管理员集合(`handler.rs`/`qr_login.rs` `verify_admin_signature` + onchain_gate);passkey 绑定不变量保持。
- 注:`ONCHINA_SIGNING_SEED_HEX` 仍被机构凭证用(`chain_runtime.rs`),整把删在 2b。

## 八、验收标准
- 每个 `AdminActionType` 落入正确档;无写动作停留在 `Session`。
- 平台钥删除后,机构注册/注销链上验签(管理员签名)回归通过。
- 每个链上写操作仅需一次冷签;grant 与链上签名同源。
- 岗位码错误的签名被链上拒绝。
- 登录 QR 携带并展示机构 CID/简称/岗位名;登录签名四端字节一致;篡改机构字段验签失败。
- `flutter analyze` / `flutter test`(citizenwallet)、`cargo check` / `cargo test`(onchina、runtime 相关 crate)通过。

### 机构自定义账户 增/删 改为链上内部投票提案(完成,已验证)
- 契约:`onchina/src/core/institution_call.rs` 新增 `encode_propose_add_institution_account`(runtime call 7:`cid_number → account_names:Vec<Vec<u8>> → proposer_role_code`)、`encode_propose_close_institution`(call 1:`actor_cid_number → proposer_role_code → 32B账户 → 32B受益人`);公权码→pallet 30、私权码→pallet 31;逐字节金标 3 例(含 `CFIN`/`SFAS` 分流)。
- 后端:`accounts/handler.rs` `create_account`/`delete_account` 删本地直写 + `require_admin_security_grant`,改镜像 `prepare_institution_governance`(共用 `authorize_own_institution_proposal`:node 绑定 + 只能操作本机构 + 岗位码 1..64 + scope + `code_bytes`)→ 编码提案 → `build_chain_sign_output`(提为 `pub(crate)`)→ 复用通用消费端 `/api/v1/admin/chain/submit`。`list_accounts` + 公开 `app_list_accounts` 读侧切链上真源 `institution_accounts_lookup`(按 cid 前缀读 `PublicManage/PrivateManage::InstitutionAccounts`)。`occupy.rs submit_chain_sign` 把两个新 PURPOSE 并入"提交后只记审计、读侧从链读"分支。
- close 受益人固定=本机构主账户(`derive_account_bytes(cid,"主账户")`);待关闭账户地址本端派生;`derive_account` 改基于新增 `derive_account_bytes`。
- 清残:删 `Db::upsert_institution_account_row`/`upsert_target_account_row`/`delete_institution_account_row`(grep 确认零调用)、`CreateAccountOutput`;`DeleteAccountInput{proposer_role_code}` 新增(DELETE 带 Json body)。
- 前端:`accounts/api.ts`/`CreateAccountModal.tsx`/`AccountManageSection.tsx` 去 securityGrant,改 `useChainSign` prepare→扫码签→`submitChainSign`;新增"发起岗位码"输入;列表改 `listAccounts` 链读;文案"发起提案,机构内部投票通过后生效";`AccountList.tsx` `created_at` 链上无时间戳→空值显 `-`(不再 1970),类型 `created_at: string|null` + `account_kind` 补 `'clearing'`。
- 验证:`cargo check` 0 警告;`cargo test --bin onchina institution` 40 passed;前端 `tsc -b` EXIT=0。
- 边界:工作树同时含另一线程 ADR-040(`20260722-account-id-official-unify.md`)对 `account_id` 命名统一的改动(primitives/admin-primitives/entity-primitives/`ONCHINA_TECHNICAL.md`/`src/cid/generator.rs`/`chain_runtime.rs` 610·626 行 `admin_account→account_id` 等),本会话一律不碰,仅在同一 `chain_runtime.rs` 新增了 `institution_accounts_lookup`。

### 平台签名钥删除的注释/文档残留清理(完成,已验证)
本节收口第六节验收项「全仓无 `ONCHINA_SIGNING_SEED_HEX` 残留」的最后一段:符号级早已清零,残的是**墓碑注释与失效文档**。判据 = 只删「指向已删除符号的历史句」,保留「当前不变量陈述」;历史留痕的载体是本任务卡与 ADR,不是源码注释。

- `onchina/src/crypto/mod.rs`:删模块文档尾句「原 sr25519 种子派生工具仅供已删除的 OnChina 平台签名钥使用,已随之整体删除」(纯墓碑;前四行已完整说明本目录当前用途)。
- `onchina/src/main.rs`:保留「OnChina 后端不持有任何链上签名钥 + 鉴权真源是链上 Active 管理员集合」不变量句,删「原平台签名钥 `ONCHINA_SIGNING_SEED_HEX`(仅用于签发注销凭证)已随注销凭证链路整体删除」。
- `onchina/src/core/chain_runtime.rs`:保留三要素鉴权不变量句并入「也不持有任何平台签名钥」,删「原注销凭证签发链路(`build_institution_deregistration_credential` 等)连同平台签名钥 `ONCHINA_SIGNING_SEED_HEX` / `ONCHAIN_CREDENTIAL_SIGNER_PUBKEY` 已整体删除」。
- `onchina/src/institution/admins/mod.rs` 750·834、`onchina/src/core/institution_call.rs` 43·59:「已收敛为…不再由平台钥签发独立凭证 / call 不再嵌…」→ 正向当前态「机构治理(管理员登记)= …直接冷签这笔 extrinsic」「call 不嵌独立凭证签名/公钥/nonce/作用域」,与同文件 72 行既有正向口径对齐。技术信息(call 里没有这些字段)一字未减。
- 删除整份失效文档 `memory/feedback_cid_sheng_signing_keyring.md`:自称「铁律」,但所声明的 11 个符号(`ShengSigningPubkey`/`ProvinceBySigningPubkey`/`set_sheng_signing_pubkey`/`encrypted_signing_privkey`/`resolve_business_signer`/`submit_register_cid_institution_extrinsic`/`FederalAdminsView`/`cid_system`/`OP_SIGN_BIND`/`DUOQIAN_CID_V1`/`derive_account_from_cid_number`)**代码命中全部为 0**,且三条内容与现行死规则直接冲突:要求「确保 `ONCHINA_SIGNING_SEED_HEX` 环境变量正确」(本卡已整把删)、`DUOQIAN` 签名域(`derive-domain-rename-gmb` / `unified-signing-optag-only` 已换 `GMB` + `op_tag` 单源)、`signing_province = None` 回退 CID MAIN 的「向后兼容模式」(`feedback_no_compatibility`)。仅 `08-tasks/done/` 两张历史卡引用它作为「Phase 1 产出」,按审计铁律 done/ 不动;文件历史留在 Git。
- 复扫:`node/onchina/crates`(排除 vendor)墓碑注释正则命中归零;`ONCHINA_SIGNING_SEED_HEX` / `ONCHAIN_CREDENTIAL_SIGNER_PUBKEY` 全仓代码级命中归零。
- 验证:`cd citizenchain && cargo check -p onchina --all-targets` → `Finished`,0 警告 0 错误。
- 协议文档同批修正 `unified-protocols.md` P-TX-012(已做):字段列表照 `institution_call.rs` 132·151 的真实编码顺序重写 —— `propose_institution_governance(30/31.8)` = `cid_number → InstitutionGovernanceAction → actor_cid_number → proposer_role_code`;`register_institution_admins(30/31.9)` = `cid_number → admins → actor_cid_number`。同时修掉两处硬规则违规:`admins` 成员写成 `admin_account`(应为 `account_id`,且该名在 `registry_consistency.rs` 的 `REMOVED_AMBIGUOUS_ACCOUNT_FIELDS` 禁用表内)、漏 `Admin.cid_number`(admin-primitives 55 行真实结构为 `account_id + cid_number + family_name + given_name`)。删尾部 5 个已废凭证字段,补一行 origin 鉴权说明。状态行「仅适用于…凭证；…已退出本凭证」改为正向当前态。
- 未做(阻塞在下条 P0):`unified-protocols.md` P-CRED-001(1214·1220·1221·1225)与 `unified-naming.md` 202·203·204·205·643·664·665·666·667 的凭证字段行。它们描述的 `register_nonce`/`credential_signer_public_key`/`scope_*`/`signature` **在 node 端死结构里仍然存在**,不能单方面判废;须与下条一并定夺。仅 `credential_issuer_cid_number` 已确认全仓零命中。

### P0:node 桌面端指向已删除端点的死调用链(已清理,已验证)
本轮核验 P-CRED-001 时发现,**不是文档问题,是运行时必然失败的代码残留**:
- OnChina 后端路由表(`onchina/src/main.rs` 2372·2376·2380)只有 `/api/v1/app/institutions/search`、`/:cid_number`、`/:cid_number/accounts` 三条;**没有 `/:cid_number/registration-info`**。`build_institution_registration_info_credential`、`InstitutionRegistrationInfoCredential`、`register_nonce`、`credential_signer_public_key` 在 onchina 侧全部零命中(凭证签发链路已随平台钥整体删除)。
- 但 node 桌面端保留完整调用链:`institution_read/types.rs` 95-124(`InstitutionRegistrationInfoResp` + `InstitutionRegistrationCredentialResp`,含 `genesis_hash`/`register_nonce`/`actor_cid_number`/`credential_signer_public_key`/`scope_province_name`/`scope_city_name`/`signature`/`meta` 八字段)→ `institution_read/cid.rs` 151-160 `fetch_institution_registration_info` 硬拼该 URL → `institution_read/commands.rs` 67-74 Tauri command `fetch_clearing_bank_institution_registration_info` → `desktop/mod.rs` 117 注册 → 前端 `node/frontend/transaction/offchain/institution/api.ts` 26 + `types.ts` 65 invoke。
- 判据(动手前逐条回原文核验):① onchina 路由为明文字符串无变量拼接,`registration` 在 onchina 侧命中全为 passkey / `subjects::registration` 无关模块;② 前端 UI 对 `fetchInstitutionRegistrationInfo` **零调用**(仅 api.ts 定义处);③ 两个响应类型的引用集完全自包含,无外部消费者;④ runtime 侧 `private-manage:142` / `public-manage:144` 只是**类型别名注释**提到 `registration-info`,不构成运行时依赖。
- 已删(node 端,全链 6 文件):`institution_read/types.rs` 两个响应结构 + `use serde::Deserialize`(删后成唯一未用 import,由 `cargo check` 抓出);`institution_read/cid.rs` 整节「拉机构注册信息」(`InstitutionRegistrationInfoEnvelope` + `fetch_institution_registration_info`,保留共用的 `onchina_client`/`ONCHINA_REQUEST_TIMEOUT`/`cid_config`);`institution_read/commands.rs` Tauri command + 模块文档「CID 注册凭证」措辞;`desktop/mod.rs` 117 注册行;前端 `institution/api.ts` 方法与 import;前端 `institution/types.ts` 两个 TS 类型。
- 文档同批清理:`unified-protocols.md` **P-CRED-001 整条协议删除**(唯一真源 `chain_runtime.rs` 已不产出该凭证,端点两端零命中);`unified-naming.md` 删 `credential_issuer_cid_number` / `credential_signer_pubkey` / `register_nonce` 共 5 行(两份重复表各一份),`account_names` 用途列由 `CID registration-info API` 改 `call data / storage`;`NODE_CLEARING_BANK_TECHNICAL.md` 修正过时 banner 路径 + 第 0 节能力 3 改为节点声明 + Tauri 命令表删 2 行已删命令并把路径列从 `governance/organization_manage/` / `transaction/offchain_transaction/` 改为真实目录 + 第 4 节删 registration-info 来源约束 + 第 5 节时序整节改写为节点声明时序。
- **核验中挡下三处误删**(粗看像同批残留,实为活字段):`scope_province_name` / `scope_city_name` 被 onchina 前端登录态大量在用(`auth.scope_province_name`,`App.tsx` 61·62 等),只是 `unified-naming.md` 用途列过时 → **改描述不删行**;`credential_signer_pubkey` 在 `registry_consistency.rs:24` 的 `REMOVED_AMBIGUOUS_ACCOUNT_FIELDS` 防复活禁用表内 → **保留**。
- 未动(按审计铁律):`NODE_CLEARING_BANK_TECHNICAL.md` 第 7「验收」/第 8「变更记录」历史快照、`08-tasks/done/` 引用;第 6 节 follow-up 表 F3/F4 指向已随机构创建移出 node 的能力,属待办不擅自处置,待 owner 确认。
- 验证:`cargo check -p node --all-targets` → `Finished`,**0 警告 0 错误**;`cargo check -p onchina --all-targets` → 0 警告;node 前端 `npx tsc --noEmit -p tsconfig.json` → **EXIT=0**;`cargo test -p qr-protocol` → 8 passed(确认未影响另一线程的 registry 改动)。全仓复扫 6 个死符号 + `registration-info` 当前态描述归零。
- runtime 收尾(已获用户二次确认后执行):`runtime/entity/private-manage/src/lib.rs:142`、`runtime/entity/public-manage/src/lib.rs:144` 的 `InstitutionAccountNamesOf` 类型别名注释,由「注册凭证里的账户名列表,顺序必须与 CID `registration-info` 返回一致」改为真实用途「「新增机构自定义命名账户」提案(call 7 `propose_add_institution_account`)的待新增账户名列表」。改前核验该类型唯一消费者:`lib.rs` 623/645 的 call 7 参数 + `add.rs:34 do_propose_add_institution_account`,与已删注册凭证无关。**仅 2 行文档注释,无逻辑/常量/权重/存储/生成物改动**;`cargo check -p public-manage -p private-manage --all-targets` → `Finished`,0 警告 0 错误。
- 收口复扫:`registration-info` / `registration_info` 在 `citizenchain/` `citizenapp/` `citizenwallet/` 全部代码路径**归零**;memory 侧仅剩 `08-tasks/done/` 历史卡与 `NODE_CLEARING_BANK_TECHNICAL.md` 第 7/8 节历史快照(按审计铁律不动)。
- 边界:未摘任何 `#[allow(dead_code)]`,未动 `citizenchain/runtime/`,未碰另一线程正在改的 `qr-protocol/registry/` 与两端生成物。
