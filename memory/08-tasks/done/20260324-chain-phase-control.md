# 任务卡：开发期/运行期阶段切换（chain-phase-control）

- 任务编号：20260324-chain-phase-control
- 状态：open（挂起，等空块不提交完成后再启动）
- 所属模块：citizenchain-runtime / citizenchain-node
- 当前负责人：待分配
- 创建时间：2026-03-24

## 任务需求

把"开发期 vs 运行期"的 3 个编译期常量改为链上状态，实现同一条链从开发期平滑切换到运行期：

| 参数 | 开发期 | 运行期 |
|------|--------|--------|
| 开发者直升 runtime | 开启 | 关闭 |
| 出块目标时间 | 30 秒 | 6 分钟 |
| GRANDPA 权威集 | 1 个 | 44 个 |

## 技术方案

### 新增 pallet

- 路径：`citizenchain/runtime/otherpallet/chain-phase-control/`
- pallet index：20
- 链上存储：
  - `Phase`：Development / Production
  - `TargetBlockTimeMs`：30000 / 360000
  - `DeveloperUpgradeEnabled`：true / false

### 需改动的 7 个现有文件

| # | 文件 | 改什么 |
|---|------|--------|
| 1 | `genesis_config_presets.rs` | 主网创世 GRANDPA 从 44 个改为 1 个（只放第 1 个国储会） |
| 2 | `grandpakey-change/src/lib.rs` | 创世初始化只写 1 个 key，与 pallet_grandpa 保持一致 |
| 3 | `runtime-upgrade/src/lib.rs` | 新增 `developer_direct_upgrade` extrinsic + 开关控制 |
| 4 | `configs/mod.rs` | 新增 `DeveloperUpgradeOrigin`；phase-control pallet Config |
| 5 | `pow_const.rs` | 去掉 `dev-chain` 编译特性，目标时间改为从链上读取 |
| 6 | `pow-difficulty-module/src/lib.rs` | 难度目标窗口从链上 `TargetBlockTimeMs` 读取 |
| 7 | `node/src/service.rs` | 矿工提交门控通过 Runtime API 读当前目标时间 |

### 不需要改

- `chain_spec.rs` 的 44 个 P2P bootnodes 保留不动
- `pallet_timestamp::MinimumPeriod` 固定为低值不再动，30秒/6分钟只靠难度目标 + 矿工门控控制

### 最后一次开发者升级（一次性 on_runtime_upgrade 迁移）

1. `DeveloperUpgradeEnabled = false`
2. `TargetBlockTimeMs = 360000`
3. `Phase = Production`
4. `pallet_grandpa::schedule_change(44 个权威)`
5. 同步 `grandpakey-change` 的 `CurrentGrandpaKeys` / `GrandpaKeyOwnerByKey` 为 44 个

## 前置依赖

- 空块不提交（任务卡 20260324-skip-empty-blocks）需先完成

## 必须遵守

- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通
- 改代码后必须更新文档和清理残留

## 实施记录

（待填写）
