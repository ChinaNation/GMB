# 任务卡：治理提案解码统一入口（共享 resolver）

- 状态：done（2026-04-15）
- 归属：Blockchain Agent（citizenchain node UI backend）

## 背景 / 根因

治理页面"全部提案"列表中，**手续费划转**（`propose_sweep_to_main`）、
**安全基金**（`propose_safety_fund_transfer`）两类提案卡片显示"（无详情数据）"，
而点进详情页后能正常看到内容。

定位到根因：`citizenchain/node/src/ui/governance/proposal.rs` 的
`fetch_proposal_display`（列表 summary 唯一来源）开头：

```rust
let raw = match fetch_proposal_data_raw(proposal_id)? {
    Some(r) => r,
    None => return Ok(ProposalDisplayInfo { summary: "（无详情数据）".into(), ... }),
};
if raw.is_empty() {
    return Ok(ProposalDisplayInfo { summary: "（无详情数据）".into(), ... });
}
```

该函数只读 `VotingEngine::ProposalData` 一个存储项。
手续费划转/安全基金/费率提案的业务 detail 存储在各自独立的 pallet storage：

- `DuoqianTransfer::SweepProposalActions`
- `DuoqianTransfer::SafetyFundProposalActions`
- `OffchainTransaction::RateProposalActions`

`ProposalData` 对它们为 None/空 → 直接早返回 "（无详情数据）" →
后续**死代码** `fetch_sweep_proposal_action` / `fetch_safety_fund_proposal_action`
/ `fetch_rate_proposal_action` 永远走不到。

详情页走的 `fetch_proposal_full` 不是这条路径，它独立做了 rate/safety_fund/sweep
三个 fallback 查询，所以详情页正常。**列表和详情各自维护一套解码顺序，易漂移**。

## 修复方案

**共享解析器（shared resolver）模式**：抽出一个 `ProposalAction` 枚举 +
`resolve_proposal_action()` 函数作为**唯一解码入口**，列表和详情都调它。

### 改动

| 文件 | 动作 |
|---|---|
| `citizenchain/node/src/ui/governance/proposal.rs` | 核心重构 |

#### 新增

- `enum ProposalAction`（7 个业务动作变体 + `Unknown`，用 `Box` 控制大小）
- `fn resolve_proposal_action(proposal_id, meta) -> Result<ProposalAction>`
  - 查找顺序：`ProposalData`（按 kind 分流 4 种）→ `RateProposalActions`
    → `SafetyFundProposalActions` → `SweepProposalActions` → `Unknown`
  - 命中即返回，不重复查询
- 7 个纯函数 `format_*_summary(&detail) -> String`：列表 summary 的唯一格式化器
- `fn truncate_chars(s, max)`：按 Unicode 字符数安全截断（避免切中多字节 UTF-8）
- `fn split_action_into_details(action)`：把 action 展开成 7 个 Option detail 字段

#### 重写

- `fetch_proposal_display`：用 resolver + `match ProposalAction` + `format_*_summary` 生成 summary
- `fetch_proposal_full`：用 resolver + `split_action_into_details`；tally 查询维持原有条件

#### 删除

- `struct ProposalDetailSet`（被 `ProposalAction` 替代）
- `fn fetch_proposal_details`（被 resolver 替代）
- `fn fetch_proposal_summary`（从未被外部调用的冗余包装）
- `fetch_proposal_display` 中 rate/safety_fund/sweep 三段 fallback **死代码**

### 一致性保证

两条路径共用 resolver → 列表 summary 与详情 `*_detail` 字段必然描述同一份数据。
新增提案类型时，只需：
1. 给 `ProposalAction` 加一个变体
2. 写一个 `format_*_summary`
3. 在 resolver 里加一个 fetch 步骤
4. 在 `split_action_into_details` 加一个映射

列表/详情自动拿到正确展示，无需分别改两处。

## 验证

- `cargo check -p node` — ✅ 通过（需 WASM_FILE 环境变量，前端 dist 可用占位目录临时补齐）
- `cargo test -p node -- format_summary_tests` — ✅ **12 个单测全部通过**
  - `truncate_chars`：基础 / 英文超限截断 / 多字节 UTF-8 安全截断
  - 7 个 `format_*_summary`：转账（基础 + 长备注）、runtime 升级（长原因）、
    决议发行（分配数+金额）、决议销毁（未知机构 fallback）、费率（百分比）、
    安全基金（金额）、手续费划转（金额+机构）
  - `split_action_into_details_maps_each_variant`：变体分发正确性

## 性能影响

| 场景 | 改前 | 改后 |
|---|---|---|
| 转账/升级/销毁/发行（常见） | 1 次 RPC | 1 次 RPC |
| **手续费划转** | 1 次（然后显示"无详情"） | 4 次（3 空查 + 1 命中） |
| **安全基金** | 1 次 | 3 次 |
| 费率提案 | 1 次 | 2 次 |

这 3 类提案频率极低，可接受。若后续成为瓶颈，可在 `ProposalMeta` 增加
`subtype_hint` 字段（需 runtime 升级）实现一次命中，独立立项。

## 后续

- Frontend（`ProposalListView.tsx`）无需改动：Tauri 命令签名和返回结构完全不变
- 前端 build（`npm run build`）不是本次作用范围，CI/run.sh 已有流程

## 相关任务卡

- `20260415-wumin-institutions-unified-registry.md` — wumin 冷钱包机构注册表合并（同日完成，HA000 省份名统一为"海滨"）
