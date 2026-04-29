# 链下清算打包器（OffchainPacker）接入 service worker

- **日期**: 2026-04-04
- **模块**: Blockchain Agent — node/src/offchain, node/src/service.rs
- **优先级**: 中（链下清算功能完整性）

## 背景

链下清算的收单链路已通：RPC 收交易 → 账本记账 → gossip 广播防双花。但"打包上链"环节缺失——`OffchainPacker` 代码已写好，未在 service.rs 中创建实例和启动 worker。

历史上该任务指向根级 offchain 文件；2026-04-29 目录收口后,对应实现统一迁入
`node/src/offchain/` 功能域。

## 目标

在 service.rs 中启动 packer worker，完成链下清算闭环：

1. 监听新区块事件
2. 调用 `should_pack()` 判断是否触发打包（笔数阈值 10 万笔 或 区块���隔 10 块）
3. 调用 `pack()` 取出待上链交易、签名��成 batch
4. 构造 `submit_offchain_batch` extrinsic 并提交到交易池
5. 上链成功后调用 `on_settled()` 清理账本 + 广播结算通知
6. 上链失败后调用 `on_pack_failed()` 将交易放���账本

## 涉及文件

- `node/src/service.rs` — 委托 `offchain/bootstrap.rs` 启动清算行 worker
- `node/src/offchain/settlement/packer.rs` — 清算行批量打包器
- `node/src/offchain/keystore.rs` — 加密私钥存储
- `node/src/offchain/ledger.rs` — 清算行本地账本

## 前置依赖

- 签名���理员配置（任务卡 20260401-step2b）
- 链下 pallet 密钥机制简化（任务卡 20260401-step2a）
