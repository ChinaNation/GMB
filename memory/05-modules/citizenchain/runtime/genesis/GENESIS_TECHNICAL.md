# GenesisPallet 技术文档

## 1. 模块职责

`genesis-pallet` 只负责：

- 保存 `Genesis` / `Operation` 链阶段；
- 保存开发者能否直接升级 runtime 的一次性开关；
- 在 block#0 写入创世宣言、国家宣言和创世人口；
- 调用 runtime 注入的机构 seeder 写入创世机构和管理员。

本模块不提供 extrinsic，不保存 PoW 出块时间，也不向节点提供出块时间 Runtime API。
PoW 六分钟是 `primitives::pow_const::POW_TARGET_BLOCK_TIME_MS` 固定的难度调整平均目标，
与链阶段无关；有效工作量证明找到后立即出块，没有最短等待或最晚期限。

## 2. 五个受守卫字段

| 存储项 | 类型 | 创世 RAW 形态 | 永久规则 |
|---|---|---|---|
| `Phase` | `ChainPhase` | 缺省即 `Genesis` | 只允许一次切换为 `Operation` |
| `DeveloperUpgradeEnabled` | `bool` | 缺省即 `true` | 只允许与阶段同步切换为 `false` |
| `CitizensDeclaration` | `BoundedVec<u8, MaxDeclarationLen>` | `CITIZENS` 的准确 UTF-8 字节 | 永久逐字冻结 |
| `CountryDeclaration` | `BoundedVec<u8, MaxDeclarationLen>` | `COUNTRY` 的准确 UTF-8 字节 | 永久逐字冻结 |
| `CitizenMax` | `u64` | `1_443_497_378` | 永久冻结 |

FRAME `StorageVersion` 必须保持 0。旧 `TargetBlockTimeMs` 已删除，同前缀未知 RAW key
（包括该旧字段）由 NodeGuard fail-closed 拒绝，不保留兼容或影子状态。

## 3. 一次性阶段状态机

合法创世状态：

```text
(Phase, DeveloperUpgradeEnabled) = (Genesis, true)
```

唯一合法目标状态：

```text
(Phase, DeveloperUpgradeEnabled) = (Operation, false)
```

约束：

- 两个字段只能在同一个包含 `:code` 变化的 runtime 升级区块中原子写入；
- 禁止普通区块修改、部分修改、显式写回创世默认值、反向切换和重新启用开发者直升；
- 转为 `Operation` 后永久冻结；
- 本轮没有自动执行阶段切换，正式切换仍需单独确认迁移和治理授权。

## 4. 公共接口

模块只保留：

```rust
pub trait DeveloperUpgradeCheck {
    fn is_enabled() -> bool;
}
```

`runtime-upgrade` 使用该接口选择当前允许的升级授权路径。旧 `GenesisPalletApi`、
`TargetBlockTime` trait、`TargetBlockTimeChanged` 事件以及未被调用的阶段事件已经删除。

## 5. 创世固定真源

固定值来自 `runtime/primitives/src/genesis.rs`：

- `CITIZENS`：创世宣言；
- `COUNTRY`：国家宣言；
- `GENESIS_CITIZEN_MAX = 1_443_497_378`；
- `GENESIS_ISSUANCE = 14_434_973_780_000` 分。

`runtime/src/genesis.rs` 把前三项写入 `GenesisConfig`。NodeGuard 使用相同的节点编译期
真源重新构造 RAW key 和 SCALE 值，不信任 runtime metadata、getter 或 Runtime API。

## 6. NodeGuard 执法

`node/src/core/node_guard/genesis_pallet.rs` 在四条路径执行：

1. 节点启动：读取 block#0 的整个 `GenesisPallet` 前缀，确认创世事实和缺省阶段状态；
2. 普通区块：三个创世事实和 StorageVersion 任何触碰都拒绝；
3. runtime 升级：只接受两字段唯一原子单向转换，并在 `:code` 后复核完整状态；
4. 完整状态导入：整个 pallet 前缀进入共享单遍分区，未知 key、缺失值、错误 SCALE、
   尾随字节和非规范状态全部拒绝。

## 7. 测试与验收

- `genesis-pallet` 单元测试：默认阶段、开发者开关、trait、阶段模拟和创世配置；
- NodeGuard 策略测试：RAW key、两种规范状态、三个固定事实、未知旧字段、畸形 SCALE、
  非规范默认写回、合法原子转换、无 `:code`、部分转换、反向转换和固定事实触碰；
- NodeGuard 真实 runtime 创世完整状态测试确认五字段策略参与共享扫描和拒绝链路；
- 最终结果以任务卡中的本轮编译、WASM 和 fresh 节点真实验收记录为准。

## 8. 文件索引

- `citizenchain/runtime/genesis/src/lib.rs`：类型、存储、创世构建和开发者升级查询；
- `citizenchain/runtime/genesis/src/tests/mod.rs`：pallet 单元测试；
- `citizenchain/runtime/primitives/src/genesis.rs`：三个创世事实的固定真源；
- `citizenchain/runtime/src/genesis.rs`：真实 runtime genesis patch；
- `citizenchain/node/src/core/node_guard/genesis_pallet.rs`：节点独立永久规则。
