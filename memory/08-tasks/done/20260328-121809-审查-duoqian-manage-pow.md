# 任务卡：全面仔细的检查一遍 duoqian-manage-pow 这个模块有没有安全漏洞、有没有需要改进的地方、功能需求是否严格实现、中文注释技术文档是否完整、有没有要清理的残留

- 任务编号：20260328-121809
- 状态：done
- 所属模块：citizenchain/runtime/transaction
- 当前负责人：Codex
- 创建时间：2026-03-28 12:18:09

## 任务需求

全面仔细的检查一遍 duoqian-manage-pow 这个模块有没有安全漏洞、有没有需要改进的地方、功能需求是否严格实现、中文注释技术文档是否完整、有没有要清理的残留

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/chat-protocol.md
- memory/07-ai/requirement-analysis-template.md
- memory/07-ai/thread-model.md

## 模块模板

- 模板来源：memory/08-tasks/templates/citizenchain-runtime.md

### 默认改动范围

- `citizenchain/runtime`
- `citizenchain/governance`
- `citizenchain/issuance`
- `citizenchain/otherpallet`
- `citizenchain/transaction`
- 必要时联动 `primitives`

### 先沟通条件

- 修改 runtime 存储结构
- 修改资格模型
- 修改提案、投票、发行核心规则

## 模块级完成标准

- 审查安全边界、功能实现、中文注释、技术文档、残留清理
- 关键结论回写任务卡
- 跑完必要验证命令

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
- 已检查 `citizenchain/runtime/transaction/duoqian-manage-pow/src/lib.rs`
- 已检查 `citizenchain/runtime/src/configs/mod.rs`
- 已检查 `citizenchain/runtime/transaction/duoqian-manage-pow/src/weights.rs`
- 已检查 `citizenchain/runtime/transaction/duoqian-manage-pow/src/benchmarks.rs`
- 已检查 `memory/05-modules/citizenchain/runtime/transaction/duoqian-manage-pow/DUOQIAN_TECHNICAL.md`
- 已完成 `cargo test -p duoqian-manage-pow`
- 已完成 `cargo check -p citizenchain`

## 审查结论

1. 创建路径状态清理不完整
   - `propose_create` / `propose_create_personal` 会先写入 `DuoqianAccounts(status = Pending)`，但后续只在 `execute_create` 执行失败时清理
   - 对“提案被拒绝 / 超时否决”没有清理路径；技术文档也把这个风险列为已知问题
   - 若同一地址对应的创建提案没通过，后续重建可能被 `AddressAlreadyExists` 卡死
   - 位置：
     - `citizenchain/runtime/transaction/duoqian-manage-pow/src/lib.rs:581`
     - `citizenchain/runtime/transaction/duoqian-manage-pow/src/lib.rs:663`
     - `citizenchain/runtime/governance/voting-engine-system/src/internal_vote.rs:249`
     - `memory/05-modules/citizenchain/runtime/transaction/duoqian-manage-pow/DUOQIAN_TECHNICAL.md:198`

2. 关闭路径状态清理不完整
   - `propose_close` 会写入 `PendingCloseProposal`
   - 当前只在 `execute_close` 成功或执行失败时清理；提案被拒绝 / 超时否决后不会清理
   - 结果是同一多签地址后续再次发起关闭提案会一直命中 `CloseAlreadyPending`
   - 位置：
     - `citizenchain/runtime/transaction/duoqian-manage-pow/src/lib.rs:854`
     - `citizenchain/runtime/transaction/duoqian-manage-pow/src/lib.rs:900`
     - `citizenchain/runtime/governance/voting-engine-system/src/internal_vote.rs:249`

3. 技术文档明显过期
   - 文档仍写 `register_sfid_institution` 只接受 `sfid_id + register_nonce + signature`
   - 文档仍写验签 payload 是 `GMB_SFID_INSTITUTION_V1`
   - 文档仍写“当前代码实际提供 5 个公开入口”，漏掉 `propose_create_personal`
   - 文档仍写 `AddressRegisteredSfid.nonce += 1`，但代码已明确不再更新该字段
   - 位置：
     - `memory/05-modules/citizenchain/runtime/transaction/duoqian-manage-pow/DUOQIAN_TECHNICAL.md:20`
     - `memory/05-modules/citizenchain/runtime/transaction/duoqian-manage-pow/DUOQIAN_TECHNICAL.md:49`
     - `memory/05-modules/citizenchain/runtime/transaction/duoqian-manage-pow/DUOQIAN_TECHNICAL.md:165`
     - `citizenchain/runtime/transaction/duoqian-manage-pow/src/lib.rs:691`
     - `citizenchain/runtime/transaction/duoqian-manage-pow/src/lib.rs:930`
     - `citizenchain/runtime/transaction/duoqian-manage-pow/src/lib.rs:1150`

4. 有死字段残留
   - `AddressRegisteredSfid.nonce` 当前只在注册时初始化为 `0`
   - 代码里已明确标注这是“历史遗留，不再更新”，仓库内也没有实际读取用途
   - 这属于应清理或明确制度化的残留状态
   - 位置：
     - `citizenchain/runtime/transaction/duoqian-manage-pow/src/lib.rs:132`
     - `citizenchain/runtime/transaction/duoqian-manage-pow/src/lib.rs:740`
     - `citizenchain/runtime/transaction/duoqian-manage-pow/src/lib.rs:1150`

5. `weights.rs` 仍是占位实现
   - 文件头直接声明“占位 weights，后续由 benchmark 生成替换”
   - 但模块已经有 benchmark 文件，说明权重产物没有完成正式收口
   - 位置：
     - `citizenchain/runtime/transaction/duoqian-manage-pow/src/weights.rs:1`

## 验证记录

- `cargo test -p duoqian-manage-pow`
  - 结果：通过，15 个测试全部通过
- `cargo check -p citizenchain`
  - 结果：通过
