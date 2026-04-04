# 任务卡：空块不提交（skip-empty-blocks）

- 任务编号：20260324-skip-empty-blocks
- 状态：in-progress
- 所属模块：citizenchain-node
- 当前负责人：Claude
- 创建时间：2026-03-24

## 任务需求

节点在交易池无任何交易时不出块。只有候选块中包含至少 1 笔非 inherent 交易时才出块。

## 技术方案

在 `service.rs` 中包装 `ProposerFactory`，候选块生成后检查是否有非 inherent extrinsic，没有则跳过。

### 改动范围

- `citizenchain/node/src/service.rs`（唯一改动文件）

### 判断规则

- inherent（如 timestamp.set）= 系统注入，不算交易
- 非 inherent = 所有通过交易池提交的 extrinsic 都算交易
- 候选块只有 inherent → 不出块
- 候选块有 ≥1 笔非 inherent → 正常出块

## 实施记录

### 2026-03-24 实现完成

改动文件（共 2 个）：

| 文件 | 改动 |
|------|------|
| `citizenchain/node/src/service.rs` | `start_cpu_miner` 增加 `pool_ready` 参数；`new_full` 中构造 `pool_ready` 闭包并传给 CPU/GPU 矿工 |
| `citizenchain/node/src/gpu_miner.rs` | `try_start` 增加 `pool_ready` 参数；外层循环增加空池检查 |

实现方式：
- 在 `new_full` 中用交易池构造 `Arc<dyn Fn() -> usize>` 闭包，返回 `pool.status().ready`
- CPU/GPU 矿工外层循环在获取 metadata 后检查 `pool_ready() == 0`
- 为空则 sleep 500ms 后 continue，不进入哈希搜索
- 编译通过（`cargo check --package citizenchain`）

### 2026-03-25 历史方案：曾为避免创世引导死锁临时放行前 10 个空块

问题：清链重启后交易池为空 → 矿工跳过挖矿 → 不出块 → 无法提交交易 → 启动死锁。

修复：在 `pool_ready` 闭包中增加创世引导期判断（`best_number < 10` 时返回 1），允许前 10 个空块出块，引导期后恢复空块跳过逻辑。仅改动 `service.rs` 中闭包构造处，CPU/GPU 矿工代码无需修改。

### 2026-03-26 增加“必须先同步网络”门控

问题：上述创世引导期判断会让清库后的普通节点在尚未接入现网前，自己先从 genesis 出块，形成离线分叉。

修复：在同一 `pool_ready` 门控中补充网络优先规则：
- 无同步 peer 时返回 `0`
- 处于 `major sync` 时返回 `0`
- `best_number == 0` 时返回 `0`，要求先导入至少 1 个网络块
- 只有已经接入网络并完成首块导入后，前 10 个块阶段才允许继续空块引导

### 2026-03-26 删除历史引导残留

现网已存在长期运行节点与高度 10 的链，不再需要“前 10 个空块自动引导”能力。

当前最终规则：
- 无同步 peer 时返回 `0`
- 处于 `major sync` 时返回 `0`
- `best_number == 0` 时返回 `0`
- 其他情况按交易池 `pool.status().ready` 判断，只有有待打包交易时才挖矿
