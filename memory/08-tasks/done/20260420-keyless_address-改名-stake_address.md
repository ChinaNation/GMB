---
title: 省储行质押地址字段改名 keyless_address → stake_address
status: done
owner: Blockchain Agent
created: 2026-04-20
completed: 2026-04-20
---

# 执行结果（2026-04-20）

- [primitives/china/china_ch.rs](citizenchain/runtime/primitives/china/china_ch.rs) 44 处 `keyless_address → stake_address`
- runtime 调用方：[configs/mod.rs](citizenchain/runtime/src/configs/mod.rs)（字段访问 + `is_keyless_account → is_stake_account` / `is_keyless_multi_address → is_stake_multi_address` / 测试辅助 `keyless_account → stake_account` / 3 个测试函数名 + 局部变量 `keyless/keyless_raw` → `stake/stake_raw`）、[genesis_config_presets.rs](citizenchain/runtime/src/genesis_config_presets.rs)、[runtime/src/lib.rs](citizenchain/runtime/src/lib.rs)
- 节点 UI：[node/src/ui/governance/mod.rs](citizenchain/node/src/ui/governance/mod.rs) + [node/src/ui/home/rpc/mod.rs](citizenchain/node/src/ui/home/rpc/mod.rs)
- institution-asset-guard [注释](citizenchain/runtime/transaction/institution-asset-guard/src/lib.rs) 分类记法更新
- 文档：GMB_WHITEPAPER、CITIZENCHAIN_TECHNICAL、CROSS_MODULE_INTEGRATION、INSTITUTION_ASSET_GUARD_TECHNICAL、DUOQIAN_TRANSFER_TECHNICAL、2 份任务卡注释
- 验证：`cargo check -p primitives ...` 9 crate 全通过；`cargo test -p primitives` 7/7 通过
- 全仓库 `rg keyless` 残留：仅本任务卡本身（叙事保留）

# 机构账户三元模型已清晰

| 角色 | 字段名 | 适用机构 |
|---|---|---|
| 主账户 | `main_address` | 所有多签机构 |
| 费用账户 | `fee_address` | 所有多签机构 |
| 质押账户 | `stake_address` | 仅省储行 |

# 补遗：CheckNonKeylessSender → CheckNonStakeSender（2026-04-20 全仓库复查发现）

交易扩展 `CheckNonKeylessSender` 内部已调用 `configs::is_stake_account`，类型名与内部逻辑语义不一致，属改名残留。一并改为 `CheckNonStakeSender`，涉及：

- [runtime/src/lib.rs](citizenchain/runtime/src/lib.rs)：struct 定义 + `IDENTIFIER` 常量字符串 + `TxExtension` 元组位置 + impl
- [node/src/rpc.rs](citizenchain/node/src/rpc.rs) + [offchain/pool_submitter.rs](citizenchain/node/src/offchain/pool_submitter.rs) + [benchmarking.rs](citizenchain/node/src/benchmarking.rs)：扩展元组引用
- [node/src/ui/governance/signing.rs](citizenchain/node/src/ui/governance/signing.rs)：5 条 SCALE 编码顺序注释
- [RPC_TECHNICAL.md](memory/05-modules/wuminapp/rpc/RPC_TECHNICAL.md) + [STEP2B_II_B_2_A_SUBMITTER.md](memory/05-modules/citizenchain/node/offchain/STEP2B_II_B_2_A_SUBMITTER.md) + [20260324-nodeui-governance-tab.md](memory/08-tasks/done/20260324-nodeui-governance-tab.md)：文档

共 15 处跨 8 文件。Metadata IDENTIFIER 字符串同步变更（`feedback_chain_in_dev` + `feedback_no_compatibility`，无需兼容旧值）。

# 背景

`primitives/china/china_ch.rs` 的 `keyless_address` 字段描述的是**机制**（无私钥），但在新的机构账户三元模型下，其**角色**是质押账户。与 `duoqian_address → main_address` 对齐，把 `keyless_address` 统一改名为 `stake_address`，使省储行三个账户语义清晰：

- 主账户 `main_address`（已完成）
- 费用账户 `fee_address`（原名保留）
- 质押账户 `stake_address`（本次改名）

# 铁律

- 字节值不变，仅字段名变，不触 chainspec 冻结、不需要链重启
- 与字段名强耦合的 Rust 包装器同步改名（按 duoqian 改名先例）：
  - `is_keyless_account` → `is_stake_account`
  - `is_keyless_multi_address` → `is_stake_multi_address`
  - 测试辅助 `keyless_account()` / 局部 `keyless` / `keyless_raw` / 三个测试函数名一并改
- 注释里的"keyless 账户/keyless 地址"统一改成"质押账户/stake"，保持语义一致

# 执行清单

## 第 1 步：primitives 源头
- [ ] [china_ch.rs](citizenchain/runtime/primitives/china/china_ch.rs)：struct 字段 + 43 条数据

## 第 2 步：runtime 调用方
- [ ] [configs/mod.rs](citizenchain/runtime/src/configs/mod.rs)：`n.keyless_address` / `CHINA_CH[...].keyless_address` + 包装器 `is_keyless_account` / `is_keyless_multi_address` + 测试辅助与测试函数名
- [ ] [genesis_config_presets.rs](citizenchain/runtime/src/genesis_config_presets.rs)：`bank.keyless_address` + 注释
- [ ] [runtime/src/lib.rs](citizenchain/runtime/src/lib.rs)：`configs::is_keyless_account` 调用

## 第 3 步：节点 UI
- [ ] [node/src/ui/governance/mod.rs](citizenchain/node/src/ui/governance/mod.rs)
- [ ] [node/src/ui/home/rpc/mod.rs](citizenchain/node/src/ui/home/rpc/mod.rs)

## 第 4 步：文档/注释
- [ ] [GMB_WHITEPAPER.md](memory/00-vision/GMB_WHITEPAPER.md)
- [ ] [INSTITUTION_ASSET_GUARD_TECHNICAL.md](memory/05-modules/citizenchain/runtime/transaction/institution-asset-guard/INSTITUTION_ASSET_GUARD_TECHNICAL.md)
- [ ] [DUOQIAN_TRANSFER_TECHNICAL.md](memory/05-modules/citizenchain/runtime/transaction/duoqian-transfer-pow/DUOQIAN_TRANSFER_TECHNICAL.md)
- [ ] [CITIZENCHAIN_TECHNICAL.md](memory/01-architecture/citizenchain/CITIZENCHAIN_TECHNICAL.md)
- [ ] [CROSS_MODULE_INTEGRATION.md](memory/05-modules/citizenchain/runtime/CROSS_MODULE_INTEGRATION.md)
- [ ] [20260325-130439-...md](memory/08-tasks/done/20260325-130439-新增-institution-asset-guard-公共模块-并接入机构账户资金操作白名单边界.md)
- [ ] [20260328-123104-...md](memory/08-tasks/done/20260328-123104-全面仔细检查-institution-asset-guard-模块是否存在安全漏洞-改进点-功能需求实现偏差-中文注释-技术文档缺失-以及需要清理的残留.md)
- [ ] [institution-asset-guard/src/lib.rs](citizenchain/runtime/transaction/institution-asset-guard/src/lib.rs) line 12 注释

## 第 5 步：验证
- [ ] `cargo check -p primitives -p institution-asset-guard -p duoqian-transfer-pow -p shengbank-stake-interest -p onchain-transaction-pow -p offchain-transaction-pos -p resolution-destro-gov -p resolution-issuance-gov -p duoqian-manage-pow`
- [ ] `cargo test -p primitives`

# 验收

全仓库 `rg "keyless_address|is_keyless"` 应为 0 处，所有 `keyless` 残留仅在保留的语义描述中（无，本次全部清理）。
