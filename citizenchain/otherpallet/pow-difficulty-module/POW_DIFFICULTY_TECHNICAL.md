# PoW Difficulty Module Technical Notes

## 0. 功能需求
### 0.1 核心职责
`pow-difficulty-module` 的功能需求是：
- 链上保存当前 PoW 挖矿难度，供 Runtime API 与节点侧共识逻辑读取。
- 难度必须按固定窗口自动调整，不能每块抖动。
- 调整结果必须落链并可通过事件审计。

### 0.2 调整规则需求
- 首个有效窗口必须从 `block 1` 开始，到 `interval + 1` 首次结算。
- 新难度必须按 `old_difficulty * target_window_ms / actual_window_ms` 计算。
- 单次调整幅度必须限制在 `[old/4, old*4]` 区间内。
- 极端情况下不得因为算术溢出、分母为 0 或强转回绕导致异常难度值。

### 0.3 生命周期与计重需求
- 模块不提供外部 extrinsic，全部逻辑由 hook 驱动。
- `on_finalize` 必须在读取到当前块时间戳后执行难度调整。
- Runtime 必须在 `on_initialize` 预申报对应区块路径的 weight，覆盖：
  - 调整块路径
  - 首次建立窗口路径
  - 普通空转路径

---

## 1. 模块定位
`pow-difficulty-module` 是一个 FRAME pallet，用于在链上动态维护 PoW 挖矿难度。

核心目标：
- 难度不再固定常量，而是按实际出块速度自动调节。
- 采用窗口化调整，避免每块抖动。
- 单次调整有上下限，避免难度剧烈跳变。
- 节点侧通过 Runtime API 读取链上当前难度，实现共识参数链上治理化。

代码位置：
- `/Users/rhett/GMB/citizenchain/otherpallet/pow-difficulty-module/src/lib.rs`

---

## 2. 关键常量
常量统一来自 `primitives::pow_const`：
- `POW_INITIAL_DIFFICULTY`：创世默认难度。
- `DIFFICULTY_ADJUSTMENT_INTERVAL`：难度调整间隔（当前 600）。
- `DIFFICULTY_TARGET_WINDOW_MS`：目标窗口时长（`interval * MILLISECS_PER_BLOCK`）。
- `DIFFICULTY_MAX_ADJUST_FACTOR`：单次上调倍率上限（当前 4）。
- `DIFFICULTY_MIN_ADJUST_FACTOR`：单次下调倍率下限（当前 4，对应最低为 `old/4`）。
- `MILLISECS_PER_BLOCK`：目标出块时间（当前 6 分钟）。

---

## 3. 存储结构
- `CurrentDifficulty: StorageValue<u64>`
  - 当前生效难度。
  - 默认值由 `DefaultInitialDifficulty` 提供（`POW_INITIAL_DIFFICULTY`）。
- `WindowStartMs: StorageValue<u64, OptionQuery>`
  - 当前调整窗口起点时间戳（毫秒）。
  - `None` 表示尚未建立窗口起点。

---

## 4. 事件与 Runtime API
### 4.1 事件
- `DifficultyAdjusted { block, old_difficulty, new_difficulty, actual_window_ms, target_window_ms }`
  - 在调整块触发，记录本次调整的核心审计字段。

### 4.2 Runtime API
在本模块声明：
- `PowDifficultyApi::current_pow_difficulty() -> u64`

Runtime 中实现后，节点可读取链上难度用于 PoW 校验和挖矿目标计算。

---

## 5. 生命周期逻辑（on_initialize + on_finalize）
核心执行逻辑位于 `Pallet::<T>::on_finalize(n)`，对应预算由 `Pallet::<T>::on_initialize(n)` 预申报。

### 5.1 触发判定
- 先读取：
  - `block_num: u32`
  - `now_ms: u64`（来自 `pallet_timestamp::Pallet::<T>::now()`）
- `now_ms == 0` 直接返回（跳过无时间戳场景）。

### 5.2 调整块条件（已修复）
当前条件为：

```text
block_num > 1 && (block_num - 1) % interval == 0
```

即首个调整块是 `interval + 1`。  
以 `interval = 600` 为例：
- `block 1`：记录首窗口起点
- `block 601`：首次调整（窗口跨度为 `t601 - t1`，正好覆盖 600 个区块间隔）
- `block 1201`：第二次调整（跨度为 `t1201 - t601`）

### 5.3 难度计算

```text
new_difficulty_raw = old_difficulty * target_window_ms / actual_window_ms
```

其中：
- `actual_window_ms = max(now_ms - start_ms, 1)`，防止分母为 0。
- 若出块过快（`actual < target`），难度上升。
- 若出块过慢（`actual > target`），难度下降。

### 5.4 单次变更夹紧
- 上限：`old * DIFFICULTY_MAX_ADJUST_FACTOR`
- 下限：`max(old / DIFFICULTY_MIN_ADJUST_FACTOR, 1)`

最终：
- `new_diff_u128` 先做 `saturated_into::<u64>()`（防截断回绕）
- 再 `clamp(min, max)`

### 5.5 窗口推进（已修复）
调整完成后：
- 直接写 `WindowStartMs = now_ms` 作为下一窗口起点。

这避免了“清空后下个块再重建窗口”导致少算一个区块间隔的问题。

---

## 6. 算法边界与安全性
- 使用 `saturating_*` 防算术溢出。
- 使用 `max(1)` 保证分母安全。
- 使用 `saturated_into::<u64>()` 防止 `u128 -> u64` 强转回绕。
- 使用区间夹紧防极端网络抖动导致难度暴涨/暴跌。

---

## 7. 跨组件接线
### 7.1 Runtime 挂载
- `/Users/rhett/GMB/citizenchain/runtime/src/lib.rs`
  - `pub type PowDifficulty = pow_difficulty_module;`

### 7.2 Runtime 配置
- `/Users/rhett/GMB/citizenchain/runtime/src/configs/mod.rs`
  - `impl pow_difficulty_module::Config for Runtime`
  - `type WeightInfo = pow_difficulty_module::weights::SubstrateWeight<Runtime>`

### 7.3 Runtime API 实现
- `/Users/rhett/GMB/citizenchain/runtime/src/apis.rs`
  - `current_pow_difficulty()` 返回 `PowDifficulty::current_difficulty()`

### 7.4 Node 侧消费
- `/Users/rhett/GMB/citizenchain/node/src/service.rs`
  - `SimplePow::difficulty()` 调用 Runtime API 获取链上难度。
  - 调用失败时回退到 `POW_INITIAL_DIFFICULTY`。

---

## 8. Weight 与 Benchmark
- `WeightInfo` 当前拆成 3 条预算路径：
  - `on_initialize_adjustment`
  - `on_initialize_start_window`
  - `on_initialize_idle`
- 预算语义是“本 pallet 在该区块内 `on_initialize + on_finalize` 的总成本”，而不是只计 `on_initialize` 自身。
- `runtime-benchmarks` 已提供 3 个对应 benchmark 入口，分别覆盖：
  - 调整块路径
  - 首次建立窗口路径
  - 普通空转路径
- 当前 `weights.rs` 仍是保守手写值，后续可用 benchmark CLI 产物替换。

---

## 9. 测试覆盖（当前）
`cargo test -p pow-difficulty-module` 当前覆盖 7 项：
- `first_adjustment_happens_at_interval_plus_one_and_window_is_exact`
- `raises_difficulty_when_blocks_are_too_fast`
- `lowers_difficulty_when_blocks_are_too_slow`
- `clamps_to_adjustment_bounds`
- `saturating_cast_prevents_u128_to_u64_wraparound`
- `test_genesis_config_builds`
- `runtime_integrity_tests`

另外做过 runtime 侧回归：
- `cargo test -p gmb-runtime pow`

---

## 10. 运维观察与审计建议
- 监控 `DifficultyAdjusted` 事件，重点看：
  - `actual_window_ms / target_window_ms` 比值趋势
  - `new_difficulty` 连续窗口变化幅度
- 若长期触发上下限夹紧，说明网络算力/出块时钟与目标参数偏离较大，应复核：
  - `MILLISECS_PER_BLOCK`
  - `DIFFICULTY_ADJUSTMENT_INTERVAL`
  - 节点时钟同步状态（NTP）
