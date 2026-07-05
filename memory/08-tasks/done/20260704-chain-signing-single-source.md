# 2026-07-04 Rust 端链交易签名唯一真源

## 目标

- 新增 `citizenchain/crates/chain-signing/`，统一构建 Substrate extrinsic 的 `TxExtension`、`SignedPayload`、签名字节和 `UncheckedExtrinsic`。
- 替换 node、OnChina、结算提交器、benchmark 中重复实现的链交易签名材料构建逻辑。
- 保持 CitizenApp 现有 polkadart 转账签名不变；本任务只收敛 Rust host 端链交易签名实现。
- 不修改 `citizenchain/runtime/**`。

## 范围

- `citizenchain/crates/chain-signing/`：链交易签名材料唯一真源。
- `citizenchain/node/`：治理冷签、PoWR 热签、结算提交、benchmark 改为调用共享 crate。
- `citizenchain/onchina/`：管理员扫码回签提交改为调用共享 crate。
- `memory/`：更新架构与任务记录，清理旧文档表述。

## 验收

- 全仓源码中 `SignedPayload::from_raw` 只保留在 `chain-signing`。
- node 与 OnChina 不再各自实现 `build_tx_extension` / `build_signing_material`。
- 运行 Rust 格式化与目标包检查。

## 完成记录

- 已新增 `citizenchain/crates/chain-signing/`，统一 Rust host 端 extrinsic 签名材料。
- 已替换 node 治理冷签、PoWR 热签 RPC、benchmark、清算行结算 submitter、OnChina 扫码回签提交路径。
- 已更新架构图谱、QR 签名说明、治理签名文档、清算行 submitter 文档和 node 技术文档。
- 已确认 `citizenchain/runtime/**` 无 diff。
