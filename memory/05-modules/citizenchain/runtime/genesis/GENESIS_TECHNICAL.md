# GENESIS_PALLET Technical Notes

## 0. 功能需求

`genesis-pallet` 的功能需求是：
- 存储链的当前运行阶段（创世期 Genesis / 运行期 Operation）及对应参数。
- 存储创世常量（创世宣言、国名宣言、创世人口），在创世区块中初始化。
- 为节点层（矿工门控、难度调整）提供 Runtime API 读取链上动态出块时间。
- 为 `runtime-root-upgrade` 提供开发者直升开关的 trait 查询接口。

## 1. 设计边界

- 本模块是 pallet，但**不暴露任何 extrinsic**。
- 阶段切换仅通过 `OnRuntimeUpgrade` 迁移一次性写入，不设链上调用。
- 其他模块（难度调整、runtime-root-upgrade）各自读本模块的链上值。
- 创世常量在创世区块写入后不再变更。

## 2. 核心类型

### 2.1 ChainPhase
```
Genesis    — 单权威、30 秒出块、开发者可直升 runtime
Operation  — 44 权威、6 分钟出块、升级必须走联合投票
```

## 3. 存储项

| 存储项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `Phase` | `ChainPhase` | `Genesis` | 当前链阶段 |
| `TargetBlockTimeMs` | `u64` | `30_000` | 出块目标时间（毫秒） |
| `DeveloperUpgradeEnabled` | `bool` | `true` | 开发者直升 runtime 开关 |
| `CitizensDeclaration` | `BoundedVec<u8, MaxDeclarationLen>` | 空 | 创世宣言 |
| `CountryDeclaration` | `BoundedVec<u8, MaxDeclarationLen>` | 空 | 国名宣言 |
| `CitizenMax` | `u64` | `0` | 创世人口 |

## 4. 公共接口

### 4.1 Runtime API
```rust
pub trait GenesisPalletApi {
    fn target_block_time_ms() -> u64;
}
```
节点层矿工门控通过此 API 读取链上动态出块时间，替代编译期常量。

### 4.2 DeveloperUpgradeCheck trait
```rust
pub trait DeveloperUpgradeCheck {
    fn is_enabled() -> bool;
}
```
供 `runtime-root-upgrade` 通过关联类型读取开发者直升开关。

## 5. 创世配置

`GenesisConfig` 字段：
- `citizens_declaration: Vec<u8>` — 创世宣言 UTF-8 字节
- `country_declaration: Vec<u8>` — 国名宣言 UTF-8 字节
- `citizen_max: u64` — 创世人口

默认值来源（`primitives::genesis`）：
- `CITIZENS` — 创世宣言文本
- `COUNTRY` — 国名宣言文本
- `GENESIS_CITIZEN_MAX = 1_443_497_378` — 第七次全国人口普查
- `GENESIS_ISSUANCE = 14_434_973_780_000` — 每人 100 元（分为单位）

## 6. 阶段切换流程

阶段从 `Genesis` 切换到 `Operation` 时需同步写入：
1. `Phase::put(ChainPhase::Operation)`
2. `TargetBlockTimeMs::put(360_000)` — 6 分钟
3. `DeveloperUpgradeEnabled::put(false)` — 关闭开发者直升

切换由 runtime 升级迁移（`OnRuntimeUpgrade`）执行，不是链上 extrinsic。

**当前状态（待办）**：迁移代码尚未实现。全仓无生产代码写入 `Phase`、`TargetBlockTimeMs`、`DeveloperUpgradeEnabled`，链将一直停在 Genesis 默认值（30 秒出块、开发者直升启用）。需要在正式切换到运行期前实现 `OnRuntimeUpgrade` 迁移。

## 7. 上下游依赖

### 被读取方
| 模块 | 读取内容 |
|------|----------|
| `pow-difficulty-module` | `target_block_time_ms()` — 每 600 块调整难度 |
| `runtime-root-upgrade` | `DeveloperUpgradeCheck::is_enabled()` — 判断升级路径 |
| 节点层（矿工） | `GenesisPalletApi::target_block_time_ms()` — 出块间隔 |

## 8. Events

| 事件 | 说明 |
|------|------|
| `PhaseChanged { from, to }` | 链阶段已切换 |
| `TargetBlockTimeChanged { old_ms, new_ms }` | 出块目标时间已变更 |
| `DeveloperUpgradeToggled { enabled }` | 开发者直升开关已变更 |

说明：Events 预留给迁移代码触发，本模块无 extrinsic。当前迁移未实现，Events 暂为死代码。

## 9. 测试覆盖

`cargo test -p genesis-pallet` 覆盖（6 个用例）：
- 默认阶段为 Genesis
- 默认出块时间 30 秒
- 默认开发者直升启用
- DeveloperUpgradeCheck trait 读取 storage
- 模拟迁移切换到 Operation 阶段
- GenesisConfig 初始化创世宣言和人口

## 10. 文件索引

- 入口与存储定义：`runtime/genesis/src/lib.rs`
- Weight（空实现）：`runtime/genesis/src/weights.rs`
- 创世常量：`runtime/primitives/src/genesis.rs`
- 创世配置构建：`runtime/src/genesis_config_presets.rs`
- Runtime 配置：`runtime/src/configs/mod.rs`（lines 1060-1063）
- Runtime API 实现：`runtime/src/apis.rs`（lines 281-285）

## 11. 已知待办

1. **迁移未实现**：`on_runtime_upgrade` 迁移代码尚未编写。链将一直停在 Genesis 默认值。正式切换到运行期前必须实现。
2. **Events 暂为死代码**：`PhaseChanged`/`TargetBlockTimeChanged`/`DeveloperUpgradeToggled` 预留给迁移使用，当前无代码触发。
3. **创世常量链上无消费者**：`CitizensDeclaration`/`CountryDeclaration`/`CitizenMax` 仅本模块测试和创世配置使用，生产运行时无其他模块读取。这些存储项的设计意图是链上可审计的创世记录，非功能性数据。
