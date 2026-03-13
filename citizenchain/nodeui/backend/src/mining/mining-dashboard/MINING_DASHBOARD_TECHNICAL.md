# Mining Dashboard 模块技术文档

## 0. 功能需求

- 页面需要展示四项挖矿收益指标：收益总额、累计手续费收益、累计挖矿奖励、今日收益。
- 页面需要展示最近出块记录，包括区块高度、时间、手续费、铸块奖励和区块作者。
- 页面需要展示节点资源监控，包括 CPU 占用、内存占用、磁盘占用和节点数据大小。
- 前端会定时轮询该接口，模块需要支持高频读取而不重复全链重扫。
- 当节点未完成追块时，模块需要返回当前已统计结果，并明确提示还有多少区块待补算。
- 当部分链上字段读取失败时，模块需要尽量返回已有统计结果，并通过 `warning` 提示不完整原因。
- 当 RPC 不可用、链指纹不匹配或矿工账号暂时无法读取时，模块应优先返回最近缓存，避免页面整体空白或统计被误重置。
- 模块需要保证收益统计只归属于当前本节点矿工账号，链切换或矿工账号切换时必须自动重建缓存。

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
    - `cache_version`
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
- 旧版本地缓存文件不会因为版本号变化直接丢弃；已知旧版本会迁移到当前 `cache_version` 后回写。
- RPC 链指纹与 genesis hash 按请求实时校验，不跨请求永久缓存，避免节点重启或端口复用后误信任旧结果。

## 6. 错误与告警策略

- RPC 不可用/链指纹不匹配：返回空统计 + `warning`。
- 增量刷新失败：返回最近缓存 + `warning`。
- 本节点矿工账号读取失败：返回最近缓存 + `warning`，不清空既有统计。
- `payment_queryInfo` 或时间戳读取部分失败：返回统计结果，并在 `warning` 标注失败计数。

## 7. RPC 健壮性

- RPC 通过共享模块 `nodeui/backend/src/shared/rpc.rs` 发起（`rpc::rpc_post`），与 Network Overview 复用同一连接池实现。
- 共享 RPC 客户端使用 `OnceLock<Client>` + 初始化互斥锁：
  - 首次成功后复用连接池；
  - 初始化失败不会缓存错误，后续调用会重试；
  - 初始化互斥保证并发下只会有一个线程执行初始化。
- 使用 connect + request timeout。
- 响应读取上限 4MB（含 Content-Length 预检查与流式读取限流）。
- 检查 HTTP 状态码必须为 200。
- JSON-RPC `error` 字段统一转错误。
- 共享 HTTP RPC URL 会复用当前本地 RPC 端口，而不是硬编码到单一端口。

## 8. 资源采样优化

- 资源采样结果做短 TTL 缓存（5 秒），减少高频执行开销。
- CPU / 内存采样通过 `sysinfo` crate（`System::process(pid)`）直接读取进程统计，不再依赖外部 `ps` 命令。
- 磁盘占用通过 `sysinfo::Disks` 获取，不再依赖外部 `df` 命令。
- 节点数据目录大小通过 Rust `fs::metadata` 递归计算，不再依赖外部 `du` 命令。
- 仍优先读取节点 PID 定向资源；无 PID 时回退整机视角。

## 9. 依赖关系

- 依赖 `home/process` 的 `current_status` 获取节点 PID（用于定向资源统计）。
- 依赖 `shared/keystore::node_data_dir` 获取节点数据目录路径。
- 依赖 `shared/security` 提供应用数据目录路径。
- 依赖 `sysinfo` crate 进行跨平台进程和磁盘资源采样。
