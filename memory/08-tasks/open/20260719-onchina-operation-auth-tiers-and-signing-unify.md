# OnChina 操作三档统一 + 删平台钥 + 冷签改签真实载荷 + 登录 QR 机构字段

- 创建日期：2026-07-19
- 状态：方案待确认（用户确认后执行；未确认前不写码）
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

### Phase 2 — 删平台钥(登录自证 + 机构凭证)
- 删 `build_login_qr_system_signature`、`sign_runtime_digest`、`main.rs` env 加载。
- 机构注册/创建/治理/注销凭证：摘要(`institution_*_message`)改由**注册局管理员钱包签名**;链上 `verify_institution_*` 口径不变(`is_institution_admin`)。
- 钱包侧删 `verifySystemSignature` 及调用。
- 验收：全仓无 `ONCHINA_SIGNING_SEED_HEX` 残留;机构注册/注销链上验签用注册局管理员签名回归通过。

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

## 八、验收标准
- 每个 `AdminActionType` 落入正确档;无写动作停留在 `Session`。
- 平台钥删除后,机构注册/注销链上验签(管理员签名)回归通过。
- 每个链上写操作仅需一次冷签;grant 与链上签名同源。
- 岗位码错误的签名被链上拒绝。
- 登录 QR 携带并展示机构 CID/简称/岗位名;登录签名四端字节一致;篡改机构字段验签失败。
- `flutter analyze` / `flutter test`(citizenwallet)、`cargo check` / `cargo test`(onchina、runtime 相关 crate)通过。
