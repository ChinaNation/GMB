# PoW Difficulty 技术文档

## 1. 固定时间语义

全链唯一口径：

```text
POW_TARGET_BLOCK_TIME_MS = 360_000
```

六分钟是 PoW 难度调整追踪的长期平均目标，不是最短出块间隔，也不是最晚出块期限：

- 有效 PoW 提前找到就立即提交；
- 晚于六分钟找到仍是合法区块；
- 协议无法在矿工离线或算力不足时保证六分钟内出块；
- `pallet_timestamp::MinimumPeriod = 1ms`，时间戳只负责严格递增，不承担节流职责；
- Genesis 和 Operation 使用完全相同的 PoW 目标。

旧 `MILLISECS_PER_BLOCK = 30_000`、GenesisPallet 动态时间存储、目标时间 Runtime API、
CPU/GPU 提交等待和 runtime 的 `MINUTES/HOURS/DAYS` 派生常量已删除。

## 2. 难度状态

| 存储项 | 类型 | 说明 |
|---|---|---|
| `CurrentDifficulty` | `u64` | 当前 PoW 难度，只能由算法推进，必须大于 0 |
| `WindowStartMs` | `Option<u64>` | 当前调整窗口起点时间戳 |
| `ActiveParams` | `PowDifficultyParams` | 当前生效的版本化难度参数 |
| `PendingParams` | `Option<PendingPowDifficultyParams>` | runtime 升级暂存的下一块生效参数 |
| `LastAdjustment` | `Option<DifficultyAdjustmentAudit>` | 最近一次难度调整审计 |

本 pallet 没有 extrinsic。节点直接读取 `CurrentDifficulty` RAW storage；读不到、解码失败或
难度为 0 均 fail-closed，不再保留 Runtime API 或固定难度兜底。

## 3. 版本化参数与调整规则

创世默认值来自 `primitives::pow_const`：

- `POW_INITIAL_DIFFICULTY = 100`；
- `DIFFICULTY_ADJUSTMENT_INTERVAL = 600`；
- `POW_TARGET_BLOCK_TIME_MS = 360_000`；
- `DIFFICULTY_TARGET_WINDOW_MS = 216_000_000`，即 60 小时；
- 单次调整范围 `[old / 4, old × 4]`，最低难度为 1；
- `POW_PARAMS_VERSION` 与 `POW_ALGORITHM_VERSION` 固定描述当前参数结构和算法版本。

运行期唯一允许通过 runtime 升级原子变更的是 `PowDifficultyParams`：

- `params_version` 必须随参数值变化而递增；
- `algorithm_version` 必须等于当前 runtime 支持的 `POW_ALGORITHM_VERSION`；
- `target_block_time_ms`、`adjustment_interval`、`max_adjust_up_factor`、
  `max_adjust_down_divisor` 必须一次性随 runtime code 一起表决；
- `CurrentDifficulty` 不能被治理直接设置，只能由算法按 `ActiveParams` 推进；
- 参数在升级块暂存到 `PendingParams`，下一块激活并重置窗口，不修改当前难度。

首窗口在 block#1 建立，block#601 首次调整，此后每 600 个区块调整一次：

```text
actual_window_ms = max(now_ms - window_start_ms, 1)
target_window_ms = target_block_time_ms × adjustment_interval
raw = old_difficulty × target_window_ms / actual_window_ms
new = clamp(raw, max(old / 4, 1), old × 4)
```

实际窗口短于 60 小时则提高难度，长于 60 小时则降低难度。计算使用饱和算术和
`saturated_into::<u64>()`，避免除零、溢出和截断回绕。旧难度异常为 0 时修复到至少 1。

## 4. 生命周期与空块共识边界

- `on_initialize` 按参数激活、调整、建窗、普通四条真实路径预申报权重；
- `on_finalize` 在 timestamp inherent 已执行后读取时间戳并更新状态；
- 调整完成后立即把当前时间戳写为下一窗口起点；
- 在任何窗口或难度状态写入前，runtime 要求 extrinsic count 大于 1，即 timestamp inherent 之外
  至少存在一笔交易；否则断言失败，整块成为共识无效块，难度状态不推进；
- NodeGuard 同时在 runtime 执行前返回 `KnownBad`，本地 mining worker 和 CPU/GPU 再以 ready
  交易池门控 proposal；最佳块变化后先跳过一轮等待交易池维护完成，无本地矿工的节点不构造
  proposal。节点前置防护不能替代 runtime 最终拒绝。

## 5. 节点挖矿行为

CPU 和 GPU 使用同一规则：

```text
取得当前 proposal 与难度
→ 搜索 nonce
→ 找到有效工作量证明
→ 确认 proposal 版本仍有效
→ 使用 powr 密钥签名
→ 立即提交
```

节点不再读取目标时间，不保存上次提交时刻，也不 sleep 补齐六分钟。六分钟只参与下一次
难度计算，不能被矿工本地配置解释成提交门控。

## 6. 权重与测试

`weights.rs` 由真实 benchmark 重新生成。删除 `GenesisPallet::TargetBlockTimeMs` 读取后，
调整路径不再包含旧 GenesisPallet storage proof；新增 `on_initialize_activate_params`
覆盖参数激活路径。

单元测试覆盖：

- block#601 首次调整和精确窗口；
- 快块提高难度、慢块降低难度；
- 4 倍上下限；
- 饱和转换和零难度修复；
- 只有 timestamp 的空块被 runtime 独立拒绝，且失败前不改变窗口和当前难度；
- timestamp 加一笔交易的非空块正常完成；
- 参数暂存下一块激活且不改变当前难度；
- 不支持的算法版本拒绝暂存；
- benchmark 四条路径。

NodeGuard 已纳入 PoW 动态难度守卫：逐块复算 `CurrentDifficulty`、窗口推进、参数激活和
runtime 升级审计，禁止普通区块篡改难度、参数或窗口；`:code` 变化必须同时具备
`RuntimeUpgradeAudit` 与 `PendingParams` 的原子绑定。当前自动化基线：
`pow-difficulty` runtime-benchmarks 17/17、NodeGuard 76/76、ConstitutionGuard 40/40。
2026-07-12 真实运行态复核：普通 release WASM 的 fresh 双节点临时链中，无交易时持续停在
block#0；Alice 提交真实 `System::remark` signed extrinsic 后产出 block#1
`0xaaf286249a775bcac3bb107b7e7f4c15ccb3fb2eaebb8d0cf87e81464d7ae7fb`，
包含 timestamp + remark 两条 extrinsics，pending 清零且陪跑节点同步到 block#1。

## 7. 文件索引

- `citizenchain/runtime/primitives/src/pow_const.rs`：固定 PoW 常量；
- `citizenchain/runtime/misc/pow-difficulty/src/lib.rs`：难度算法；
- `citizenchain/runtime/misc/pow-difficulty/src/benchmarks.rs`：三条 benchmark；
- `citizenchain/runtime/misc/pow-difficulty/src/weights.rs`：生成权重；
- `citizenchain/node/src/core/service.rs`：CPU 挖矿和 PoW 验证；
- `citizenchain/node/src/mining/gpu_miner.rs`：GPU 挖矿。
