# 链下清算打包器（OffchainPacker）接入 service worker

- **日期**: 2026-04-04
- **模块**: Blockchain Agent — node/src/service.rs, offchain_packer.rs
- **优先级**: 中（链下清算功能完整性）

## 背景

链下清算的收单链路已通：RPC 收交易 → 账本记账 → gossip 广播防双花。但"打包上链"环节缺失——`OffchainPacker` 代码已写好，未在 service.rs 中创建实例和启动 worker。

导致 offchain_packer.rs / offchain_keystore.rs / offchain_ledger.rs 共 12 个 dead_code warning。

## 目标

在 service.rs 中启动 packer worker，完成链下清算闭环：

1. 监听新区块事件
2. 调用 `should_pack()` 判断是否触发打包（笔数阈值 10 万笔 或 区块���隔 10 块）
3. 调用 `pack()` 取出待上链交易、签名��成 batch
4. 构造 `submit_offchain_batch` extrinsic 并提交到交易池
5. 上链成功后调用 `on_settled()` 清理账本 + 广播结算通知
6. 上链失败后调用 `on_pack_failed()` 将交易放���账本

## 涉及文件

- `node/src/service.rs` — 创建 OffchainPacker 实例，启动 essential task
- `node/src/offchain_packer.rs` — 已实现，接���后 12 个 warning 自动消除
- `node/src/offchain_keystore.rs` — `pair` 字段和 `remove_signing_key` 方法在 packer 接入后被使用
- `node/src/offchain_ledger.rs` — `take_all_pending`、`remove_settled`、`save_to_disk` 在 packer 接入后被调用

## 前置依赖

- 签名���理员配置（任务卡 20260401-step2b）
- 链下 pallet 密钥机制简化（任务卡 20260401-step2a）
