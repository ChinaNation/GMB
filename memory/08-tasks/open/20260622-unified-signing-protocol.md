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
