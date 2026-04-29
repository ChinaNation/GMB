# 任务卡：全面仔细检查 citizen-issuance 模块是否存在安全漏洞、改进点、功能需求实现偏差、中文注释/技术文档缺失，以及需要清理的残留。

- 任务编号：20260328-095223
- 状态：open
- 所属模块：citizenchain/runtime/issuance/citizen-issuance
- 当前负责人：Codex
- 创建时间：2026-03-28 09:52:23

## 任务需求

全面仔细检查 citizen-issuance 模块是否存在安全漏洞、改进点、功能需求实现偏差、中文注释/技术文档缺失，以及需要清理的残留。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/05-modules/citizenchain/runtime/issuance/citizen-issuance/CITIZENISS_TECHNICAL.md

## 必须遵守

- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通
- 改代码后必须更新文档和清理残留

## 输出物

- 代码
- 中文注释
- 文档更新
- 残留清理

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
- 已读取启动协议、模块技术文档、上游 `sfid-system` 接线、常量来源、runtime 配置与 benchmark/weight 文件
- 已执行 `cargo test -p citizen-issuance`
- 已执行 `cargo check -p citizen-issuance`
- 已执行 `cargo check -p citizen-issuance --features runtime-benchmarks`
- 已执行 `cargo test -p sfid-system`
- 已执行 `cargo check -p citizenchain`

## 审查结论

### 已确认

- 未发现 `citizen-issuance` 模块内可直接利用的高危安全漏洞
- 核心需求已实现：一次性奖励、账户级与绑定标识级双重防重、总量上限、阶段奖励、成功/跳过事件、无外部补发入口
- 模块核心中文注释基本完整，关键发奖路径、去重逻辑、weight 接线意图均有中文说明
- 模块单测、benchmark feature 编译、runtime 编译均通过

### 已确认问题与改进点

1. 上游 `sfid-system` 的 `bind_sfid` 权重文件与当前代码实现明显漂移，影响本模块回调接线后的总 weight 可信度。
   - `sfid-system/src/weights.rs` 仍引用 `UsedCredentialNonce`、`SfidToAccount`、`AccountToSfid`、`CredentialNoncesByExpiry`
   - 当前实际代码已是 `UsedBindNonce`、`BindingIdToAccount`、`AccountToBindingId`，且没有 `CredentialNoncesByExpiry`
   - 这说明上游 weight 很可能未按当前实现重新 benchmark；本模块自身 weight 正常，但集成后的 `bind_sfid` 总 weight 可信度不足

2. 缺少真正覆盖 `bind_sfid -> OnSfidBound -> reward issuance` 的集成测试。
   - `sfid-system` 测试里 `type OnSfidBound = ()`
   - 当前只能证明两个 crate 分别自洽，不能证明 runtime 接线后的真实行为不会回归

3. 模块技术文档存在漂移。
   - `CITIZENISS_TECHNICAL.md` 中 `SkipReason` 写的是 `DuplicateSfid`，代码实际是 `DuplicateBindingId`
   - 文档记录的 weight 生成日期为 `2026-03-12`，实际 `src/weights.rs` 文件头是 `2026-03-17`
   - 测试覆盖章节未完整反映现有测试用例

4. AI 上下文装载脚本未登记该子模块，导致自动装载时提示“未识别的模块”。
   - 这不影响链上逻辑，但会影响后续审查、实现和文档治理效率

### 建议处理顺序

1. 先重新 benchmark 并更新 `sfid-system/src/weights.rs`
2. 增加至少一条跨模块集成测试，覆盖 `bind_sfid` 成功后奖励发放与跳过事件
3. 同步更新 `CITIZENISS_TECHNICAL.md`
4. 给 `memory/scripts/load-context.sh` 增加 `citizenchain/runtime/issuance/citizen-issuance` 映射
