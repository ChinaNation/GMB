# 任务卡：全面仔细检查 institution-asset-guard 模块是否存在安全漏洞、改进点、功能需求实现偏差、中文注释/技术文档缺失，以及需要清理的残留。

- 任务编号：20260328-123104
- 状态：open
- 所属模块：citizenchain/runtime/transaction/institution-asset-guard
- 当前负责人：Codex
- 创建时间：2026-03-28 12:31:04

## 任务需求

全面仔细检查 institution-asset-guard 模块是否存在安全漏洞、改进点、功能需求实现偏差、中文注释/技术文档缺失，以及需要清理的残留。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/05-modules/citizenchain/runtime/transaction/institution-asset-guard/INSTITUTION_ASSET_GUARD_TECHNICAL.md

## 必须遵守

- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通
- 改代码后必须更新文档和清理残留

## 输出物

- 审查结论
- 风险点
- 改进建议
- 文档/残留清单

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
- 已审查代码文件：
  - citizenchain/runtime/transaction/institution-asset-guard/src/lib.rs
- 已审查关联接线：
  - citizenchain/runtime/src/configs/mod.rs
  - citizenchain/runtime/transaction/duoqian-manage-pow/src/lib.rs
  - citizenchain/runtime/transaction/duoqian-transfer-pow/src/lib.rs
  - citizenchain/runtime/transaction/offchain-transaction-pos/src/lib.rs
- 已审查文档：
  - memory/05-modules/citizenchain/runtime/transaction/institution-asset-guard/INSTITUTION_ASSET_GUARD_TECHNICAL.md
  - memory/08-tasks/open/20260325-130439-新增-institution-asset-guard-公共模块-并接入机构账户资金操作白名单边界.md

## 审查结论

- 未发现当前 runtime 下可直接利用的高危安全漏洞。
- 模块职责、动作枚举、调用方接入点和技术文档整体一致，功能需求基本实现。
- 主要问题不在“逻辑错误”，而在“默认安全姿态偏松”和“真实 runtime 规则缺少直接回归测试”。

## 发现问题

1. `InstitutionAssetGuard` 的 `()` 默认实现是全放行，属于 fail-open 兜底。
   - 证据：`src/lib.rs` 中 `impl InstitutionAssetGuard<AccountId> for ()` 直接返回 `true`。
   - 影响：当前 production runtime 没这么配，但如果后续新 runtime、mock 或基准环境误接成 `()`, 这条资金白名单边界会静默失效。

2. 模块测试只覆盖了 `()` 的“全放行默认行为”，没有直接覆盖真实 runtime 的拒绝/放行矩阵。
   - 证据：当前模块只有一个单测 `default_guard_allows_all_actions`；`RuntimeInstitutionAssetGuard` 的 `keyless / reserved duoqian / fee_account / 普通账户` 分支没有独立测试。
   - 影响：功能现在是对的，但后续改 `configs/mod.rs` 时容易无声回归。

3. `load-context.sh` 未登记 `institution-asset-guard`，属于上下文装载残留。

## 注释与文档

- 中文注释完整，技术文档整体齐全且与当前代码基本一致。
- 未发现文档中的明显旧接口漂移、`TODO`、调试打印或临时代码残留。

## 验证记录

- `cargo test -p institution-asset-guard`：通过
- `cargo check -p institution-asset-guard`：通过
- `cargo check -p duoqian-manage-pow`：通过
- `cargo check -p duoqian-transfer-pow`：通过
- `cargo check -p offchain-transaction-pos`：通过
- `cargo check -p citizenchain`：通过
- `rustfmt --check citizenchain/runtime/transaction/institution-asset-guard/src/lib.rs`：通过
