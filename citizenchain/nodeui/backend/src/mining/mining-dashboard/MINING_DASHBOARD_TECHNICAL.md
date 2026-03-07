# Mining Dashboard 模块技术文档

## 1. 模块位置

- 路径：`nodeui/backend/src/mining/mining-dashboard/mod.rs`
- 对外命令：
  - `get_mining_dashboard`

## 2. 模块职责

- 汇总挖矿收益看板数据：总收益、手续费收益、奖励收益、今日收益。
- 生成最近 20 个区块的出块记录。
- 返回节点资源占用（CPU、内存、磁盘、数据目录大小）。
- 在数据不完整或 RPC 异常时返回告警信息（`warning`），避免静默错误。

## 3. 数据模型

- 对外返回：
  - `MiningDashboard { income, records, resources, warning }`
- 进程内缓存：
  - `MiningComputationCache`
    - `chain_genesis_hash`
    - `last_processed_height / last_processed_hash`
    - `total_fee_fen / total_reward_fen`
    - `income_by_utc_day`
    - `recent_records`（最近 20 条）

## 4. 统计流程

1. 校验 RPC 目标链指纹（`ss58Format == 2027` + `system_name` 非空）。
2. 读取最新区块高度。
3. 按增量区间 `last_processed_height+1..=best_height` 处理新区块：
   - `chain_getBlockHash`
   - `chain_getBlock`
   - `payment_queryInfo`
   - `state_getStorage(Timestamp.Now)`
4. 更新累计收益、按 UTC 天收益桶、最近 20 条记录。
5. 返回聚合结果。

说明：已移除“每次请求全链重扫”的实现，改为增量刷新，避免高度变大后接口退化。

## 5. 一致性控制

- 通过 `chain_genesis_hash` 识别链上下文变化。
- 启动时若发现：
  - 链 genesis hash 变化，或
  - 缓存高度高于当前链高，或
  - 缓存末块 hash 与链上不一致
  会自动重置缓存后重建。

## 6. 错误与告警策略

- RPC 不可用/链指纹不匹配：返回空统计 + `warning`。
- 增量刷新失败：返回最近缓存 + `warning`。
- `payment_queryInfo` 或时间戳读取部分失败：返回统计结果，并在 `warning` 标注失败计数。

## 7. RPC 健壮性

- 使用 connect/read/write timeout。
- 响应读取上限 4MB。
- 检查 HTTP 状态行必须为 200。
- JSON-RPC `error` 字段统一转错误。

## 8. 资源采样优化

- 资源采样结果做短 TTL 缓存（5 秒），减少 `ps/top/du/df` 高频执行开销。
- 仍优先读取节点 PID 定向资源；无 PID 时回退整机视角。

## 9. 依赖关系

- 依赖 `home/home-node` 的 `current_status` 获取节点 PID（用于定向资源统计）。
- 依赖 `settings/security` 提供应用数据目录路径。
