# 签名协议统一为单一原语 GMB+op_tag（折掉 7 个字符串域）

## 状态
**Phase 1 + Phase 2 完成并验证（2026-06-22）。** 见 [[ADR-026]]。全仓 `GMB_*_V1` 生产残留=0;签名 op_tag/signing_message 单源 primitives::sign;治理 5 个字节零变化(golden 137304f0);8 哈希域 Rust↔Dart 金标逐字节;ACTIVATE/DECRYPT 四方逐字节一致(payload 97B/108B);IM 两版本串单源集中。链端 workspace check + 后端 77 + citizenapp/citizenwallet flutter test 全绿。fixture 漂移已修(canonical↔dart 副本 8 向量逐字节一致)。**未提交(待授权)**;破坏式签名(运行时非创世状态,落同次重新创世前无迁移)。

## 一句话收口
整个仓库的签名协议现在是**一套**:`primitives::sign::signing_message(GMB||op_tag||scale)` 哈希原语(治理5 + L3_PAY/BATCH/L2_ACK)+ 统一 `GMB||op_tag` 二进制前缀(ACTIVATE/DECRYPT,保留原始字节可解析)+ IM 两版本串单源集中。零散落零重复。

## 任务需求
全仓签名消息收敛到唯一原语 `primitives::sign::signing_message(op_tag, scale_payload) = blake2_256(GMB || op_tag || scale_payload)`；7 个散落字符串域 `b"GMB_*_V1"` 折成 op_tag(0x15-0x1B)进同一注册表；删全部本地重复（2-5 份/域）。

## 关键设计（见 ADR-026）
- 字节证明：`SCALE((GMB,op_tag,fields))` == `GMB||op_tag||SCALE(fields)` → 治理 5 个(0x10-0x14)改调原语**字节不变、签名不变**（回归铁证）；7 个字符串域改 op_tag → 签名字节变（前缀 13B→4B）。
- op_tag 注册表 0x15-0x1B：L3_PAY/OFFCHAIN_BATCH/L2_ACK/ACTIVATE_ADMIN/DECRYPT/IM_NODE_PAIRING/IM_WALLET_BINDING。

## 爆炸半径（真实核对）
- 删/折字符串域定义处：runtime `offchain-transaction/src/batch_item.rs`(L3_PAY+BATCH)；node `ledger.rs`/`packer.rs`/`settlement/signer.rs`/`rpc.rs`(L2_ACK)/`governance/admins_change/activation.rs`/`offchain_transaction/settlement/admin_unlock.rs`/`settings/communication-node/mod.rs`/`im/binding.rs`。
- 治理路径(字节不变验证)：runtime `configs/mod.rs`5 处 + backend `core/chain_runtime.rs`。
- Dart 镜像：citizenapp `payment_intent.dart`/`offchain_clearing_rpc.dart`/`im/crypto/*`/`admins-change/.../admin_activation_service.dart`；citizenwallet `signer/payload_decoder.dart`(ACTIVATE_ADMIN/DECRYPT)。
- 测试：citizenapp `payment_intent_golden_test.dart` 等。
- 冷钱包 `'cid_admin_governance'` QR 域：评估并入/正名。

## 实施顺序
1. primitives::sign 新模块（signing_message + OP_SIGN_* 0x10-0x1B）
2. runtime+node 折域 + 删重复（citizenchain workspace）
3. backend 治理路径核对（字节不变）
4. Dart 镜像折 op_tag + 金标
5. Rust 导出 signing golden + Dart 断言 + 全量编译/测试/残留=0

## 验收
全仓 `GMB_*_V1`=0；签名域常量仅 primitives::sign；治理 5 个 signing golden 字节不变；7 协议新 op_tag Rust↔Dart 金标逐字节对齐；链端/node/backend/双钱包全绿。

## Phase 3：扫码签名统一为 QR_V1（2026-06-22 追加）

### 任务需求

按唯一真源、唯一协议、尽量精简原则，把所有“生成二维码 → 扫码识别 → 用户签名 → 返回签名结果 → 生成方验签/提交”的流程统一到一套 `QR_V1` 组件与字段契约中。重点修复 CID 通行密钥更新被公民钱包红色拒签、签名二维码过大导致扫码慢、链交易 `InvalidTransaction::BadProof(0x010004)` 三类问题。

### 预计修改目录

- `memory/07-ai/`：更新统一协议入口与命名入口，登记 `QR_V1`、`k`、`a`、短字段和禁止旧协议残留；属于文档修改和残留清理。
- `memory/01-architecture/qr/`：重写扫码协议、签名识别、action 注册表和 fixture 口径，只保留本次统一扫码签名方案；属于文档修改和旧协议清理。
- `memory/05-modules/`：同步 CitizenApp、CitizenWallet、CID、CPMS、CitizenChain 模块技术文档；属于文档修改和跨端边界说明。
- `citizenwallet/lib/qr/`、`citizenwallet/lib/signer/`：实现公民钱包扫码解析、签名前展示、签名消息选择和 `QR_V1` 签名响应；属于代码修改、中文注释完善和旧格式清理。
- `citizenapp/lib/qr/`、`citizenapp/lib/signer/`、`citizenapp/lib/rpc/`：实现公民端生成/识别 `QR_V1`、链交易签名响应识别和 Substrate payload 签名规则；属于代码修改、中文注释完善和旧格式清理。
- `citizenchain/node/src/`：统一节点生成签名二维码、验签、交易提交的 session/签名消息规则，修复 BadProof；属于代码修改、中文注释完善和旧提交参数残留清理。
- `citizenchain/runtime/primitives/src/`：如需补充扫码签名 action/op_tag 常量或注释，仅限 primitives 签名真源；本项已由用户在本任务中二次确认允许修改 runtime。
- `citizencode/backend/`、`citizencode/frontend/`：统一 CID 登录、通行密钥更新、公民绑定、管理员动作的 QR 生成/验签字段；属于代码修改、中文注释完善和旧字段残留清理。
- `citizenpassport/backend/`、`citizenpassport/frontend/`：统一 CPMS 登录和档案删除签名二维码字段；属于代码修改、中文注释完善和旧字段残留清理。

### 执行要求

- 不保留历史协议名、登录专用 QR kind、QR 内 display 展示真源、兼容分支或旧文档口径。
- `QR_V1` 顶层字段压缩为 `p/k/i/e/b`；业务场景统一由 `a` 表示；签名请求/响应只携带扫码验签所需最小字段。
- 扫码端展示内容必须由 `a+d` 解码得到，禁止把 display 文案作为验真真源。
- 链交易签名必须按 Substrate `SignedPayload::using_encoded` 规则处理：payload 长度大于 256 字节时签 blake2_256(payload)，避免 BadProof。
- 完成后必须运行跨端解析/签名/验签测试，并尽可能做真实本地扫码或 HTTP/页面验收；未能运行的验收必须记录原因。

### 完成记录

- 已把扫码签名协议统一为 `QR_V1`：顶层只保留 `p/k/i/e/b`，签名请求 body 只保留 `a/g/u/d`，签名响应 body 只保留 `u/s`。
- 已把登录、CID 通行密钥更新、公民绑定、CPMS 档案删除、CitizenChain 治理/交易签名统一为 `k=1` 签名请求和 `k=2` 签名响应；业务场景由 `a` 唯一区分。
- 已把链交易 `a` 统一为 `(pallet_index << 8) | call_index`，非链动作码登记到 `memory/01-architecture/qr/qr-action-registry.md` 和 runtime primitives。
- 已修复 Substrate payload 签名规则：链交易 payload 长度大于 256 字节时签 `blake2_256(payload)`，避免 `InvalidTransaction::BadProof(0x010004)`。
- 已删除旧登录二维码 body、旧展示适配器、旧登录二维码 fixture，并将 UI 文案统一为“签名响应”。
- 已同步 CID/CPMS 后端、前端和数据库命名：登录二维码入口统一为 `qr/sign-request`，本地 `payload_hash` 只作为生成方 session 校验，不进入二维码或签名响应。
- 已更新 QR 协议文档、action registry、模块技术文档、统一协议/命名规则和相关历史任务卡残留表述。

### 已执行验收

- `citizenchain`: `cargo check -p primitives`、`cargo check -p node`
- `citizencode/backend`: `cargo check`
- `citizenpassport/backend`: `cargo check`
- `citizenapp`: `flutter test test/signer/qr_signer_test.dart test/qr/qr_router_test.dart test/qr/qr_sign_session_test.dart test/wallet/widgets/wallet_qr_dialog_test.dart test/wallet/pages/wallet_list_tile_test.dart`
- `citizenwallet`: `flutter test test/signer/qr_signer_test.dart test/signer/offline_sign_service_test.dart test/signer/payload_decoder_test.dart`
- `citizencode/frontend`: `npm run build`
- `citizenpassport/frontend`: `npm run build`
- `citizenchain/node/frontend`: `npm run build`
- `website`: `npm run build`
- 残留扫描：旧 QR 协议名、旧登录二维码 kind、旧展示字段口径、旧签名响应文案和旧激活/解密字符串域均无命中。

### 验收限制

- 当前线程完成了本地编译、构建、Flutter 单测和前端 build；未接入真实手机摄像头做物理扫码。二维码内容已通过生成、解析、签名、验签单测覆盖，真实设备扫码速度需在真机联调时复验。
