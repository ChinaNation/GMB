# 任务卡：创世引导期允许空块（genesis-bootstrap，历史方案，已废弃）

- 任务编号：20260325-genesis-bootstrap
- 状态：deprecated
- 所属模块：citizenchain-node
- 当前负责人：Claude (Blockchain Agent)
- 创建时间：2025-03-25
- 关联任务：20260324-skip-empty-blocks

## 任务需求

该卡记录的是历史阶段为避免清链后启动死锁而引入的临时方案。

当前现网要求已变更为：普通节点必须先同步现网，不再允许自动引导前 10 个空块。

清链重启后区块链不出块，原因是「空块不提交」功能在交易池为空时跳过挖矿，导致创世启动死锁。

需要在创世引导期（best_number < 10）允许空块，引导期结束后恢复空块跳过逻辑。

## 技术方案

在 `service.rs` 的 CPU/GPU 矿工 `pool_ready` 检查前，获取当前最佳块高度，若 `best_number < 10` 则跳过空池检查，允许出空块。

### 改动范围

| 文件 | 改动 |
|------|------|
| `citizenchain/node/src/service.rs` | CPU 矿工循环增加创世引导期判断 |
| `citizenchain/node/src/gpu_miner.rs` | GPU 矿工循环增加创世引导期判断 |

### 判断规则

- `best_number < 10`：引导期，允许空块
- `best_number >= 10`：正常期，空池时跳过挖矿

## 废弃说明

- 该方案已被 `20260326-103500` 与 `20260326-104800` 两张任务卡中的新规则取代。
- 当前实现不再保留“前 10 个空块自动放行”逻辑。
