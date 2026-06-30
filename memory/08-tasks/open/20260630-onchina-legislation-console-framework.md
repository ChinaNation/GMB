# 任务卡：OnChina 立法与表决控制台框架（法律案核心 + 提案类型维度）

## 任务需求

在 OnChina 控制台落地「立法与表决」模块：立法机构管理员（议员/委员）在各自机构节点扫码登录后，**发起提案 → 院内/两院表决 → 查看进度**；大厅中央另设**只读大屏**展示在线议员席位与实时投票。提案分三类（**法律案 / 任免案 / 预算案**），以**提案类型（ProposalCategory）为可扩展维度**；本轮**法律案为核心全链路实现**，任免案/预算案**本轮只定义规范化数据结构**、提交链路预留。

结论性前提（已与用户确认）：**不修改宪法**。法律提案权宪法只给立法机关；政府的「人事任免」由宪法第 100/106 条直授（提交人事任免职书 → 参议会/立法会常规案表决任免），「预算」由《预算法》（普通法）授权，二者走**独立提案类型**，非法律案。

## 所属模块

`citizenchain/onchina`（OnChina Agent：后端 `domains/legislation/` + 鉴权/能力扩展点 + 前端 `frontend/legislation/`）。链端 `legislation-yuan`(idx27)/`legislation-vote`(idx28) **本轮只读核对，零改动**（改 = runtime 二次确认）。

## 输入文档

- memory/04-decisions/ADR-027-legislation-yuan.md
- memory/04-decisions/ADR-030-onchina-multi-institution-console.md
- memory/04-decisions/ADR-028-citizen-tab-public-power-unified.md
- 公民宪法第 8/12/18/19/44/45/46/53/55/57/59/64/66/75/90/100/101/104–113/125 条（提案主体、表决程序、任免、三权分立）

## 边界铁律（必须遵守）

- **本轮范围 = 法律案全链路 + 提案类型维度 + 大屏只读**；任免案/预算案只落 `personnel/model.rs`、`budget/model.rs` 结构，**不接链、不实现提交**。
- **不改宪法**；不改链端 `legislation-yuan`/`legislation-vote`（只读）。任何 runtime diff 需二次确认。
- **另开线程**（不在本卡）：① 行政签署人（总统/省长/市长）登录身份 + `executive_sign` 入口；② 议员 admins 灌入通道（election-vote → admins）；③ 双客户端 CitizenApp/CitizenWallet（卡 20260624-legislation-dual-client）。
- **后端源码根** = `citizenchain/onchina/src/`；链交互文件一律 `chain_` 前缀；冷签复用 `core/qr/` + `auth/action_sign.rs`；前端 API 走模块内 `api.ts`，通用 http 仅 `utils/http.ts`。
- **scope fail-closed**：立法 list 必过 `scope::filter_by_scope`；提案 `scope_code` 越权在写入边界（onchain_gate + prepare 预检）拒绝，读路径绝不放行。
- **投票职责边界**：onchina 只做「组织提案数据 + 扫码冷签 + 提交 extrinsic + 读链展示」，**绝不计票/推进状态机**（全归投票引擎）。
- **金额单位分**（u128 整数，禁浮点）；公钥内部 0x 小写 hex，展示 SS58(prefix=2027)。
- 全仓字段同名：沿用 `cid_number/houses/proposer_body/vote_type/current_house/content_hash/law_id/version/scope_code/admin_account` 等，禁造别名。
- 注释描述当前实现，禁「从 X 改 Y / 原来 / 之前」历史措辞。

## 已核实事实（宪法 + 链端源码）

### 提案主体 × 表决院 × 签署（真机构码，code.rs 核实）

| 层级 | 提案主体(发起院) | houses 序列 | 终审院 | 签署/救济 | 依据 |
|---|---|---|---|---|---|
| 国家·非教育 | 众议会 NRP | [NRP, NSN] | 参议会 NSN | 总统签署→NLG+NSN+NRP 三人会签 | 第45/100/101 |
| 国家·教育 | 教委会 NED(本会先表决) | [NED, NSN] | 参议会 NSN | 总统签署 | 第75②/100 |
| 省·非教育 | 省众议会 PRP | [PRP, PSN] | 省参议会 PSN | 省长 PGV 签署→PLG+PSN+PRP 三人会签 | 第104–108 |
| 市·非教育 | 市立法会委员 CLEG / 市自治会委员 CSLF | [CLEG] | 市立法会 CLEG | 市长 CGOV 签署(无会签) | 第46/110–113 |
| 市·教育 | 市教委会委员 CEDU | [CLEG] | 市立法会 CLEG | 市长 CGOV 签署 | 第46④ |

- `proposer_body` 与 `houses` 解耦（链端 `ensure_routing` 已固化）；教育↔教委会(NED/CEDU)双向硬绑；特别案禁教育；省级无教委会（无 PED 码）；不可修改条款 = {1,2,3,17,19,24,34,42}（`count_const.rs:49`，与宪法一致）。
- 国家级行政签署人=总统（总统府机构码 **待核实**，code.rs 国家级未见，第53条总统府存在）。

### 三类提案的表决程序

| 类型 | 表决类型 | 表决机构 | 流程 |
|---|---|---|---|
| 法律案 | 5 种(常规/常规教育/重要/重要教育/特别) | 两院/单院 | 院内→[公投]→签署→[会签]→[护宪] |
| 任免案 | 常规案(默认)/重要案(3 次驳回升级,第55/57/64条) | 参议会/市立法会 单院 | 院内表决→通过即任免(无公投/签署/护宪) |
| 预算案 | 常规案 | 立法机关 单院 | 院内表决→通过(细节由《预算法》定) |

### 部署模型

OnChina 每节点单机构绑定（`onchain_gate.rs` + `NodeInstitutionBinding`），`https://onchina.local:8964` + mDNS。每立法机构一节点（国家众议会/参议会/立法院/教委会… 各一），议员席位电脑扫码登录本机构节点。现有前端**无只读大屏、无实时订阅**（会话+分页查询）→ 大屏为新建，**同一前端两路由（operator/display），本节点会话 + 轮询**。

## 框架设计（已确认）

### 后端目录 `citizenchain/onchina/src/domains/legislation/`

```
mod.rs                  域导出 + /api/legislation/* 路由
model.rs                统一信封 + ProposalCategory + HouseRef/LegProposalState
category.rs             提案类型维度:本节点机构码→可发起 category×tier×vote_type 候选(扩展点)
handler.rs / service.rs HTTP 入口 / 组织提案数据(不计票)
chain_read_proposal.rs  通用链读 LegProposalState + 各 tally(三类共用)
law/{mod,model,chain_propose,chain_house_vote,chain_referendum_vote,chain_executive_sign,chain_override_sign,chain_guard_vote,chain_read}.rs   法律案(本轮实现)
personnel/{mod,model}.rs   任免案(本轮:PersonnelDecision 结构,链路预留)
budget/{mod,model}.rs      预算案(本轮:BudgetPlan 结构,链路预留)
display/{mod,handler,service}.rs   大屏只读聚合(在线席位+实时计票+提案进度)
```
扩展点(改既有)：`platform/capability.rs`、`auth/operation_auth.rs`、`auth/catalog.rs`、`scope/rules.rs`。

### 前端目录 `citizenchain/onchina/frontend/legislation/`

```
api.ts / types.ts / index.ts
operator/  LegislationView.tsx / ProposeMenu.tsx(唯一发起入口)
  law/     LawEditorModal / ProposeLawModal / HouseVotePanel / SignActionModal / ProposalProgressView / LawListTable
  personnel/ PersonnelDecisionModal.tsx(预留)
  budget/    BudgetPlanModal.tsx(预留)
display/   LegislationBoardView / SeatGrid / LiveTallyBoard / ProposalTicker(只读,独立路由)
```
接入点：`App.tsx`（ActiveView 增 `legislation`；大屏走 `?view=legislation-display`）、`platform/capabilityMap.ts`。

### 字段（snake/camel 三端一致，详见对话定稿）

- 通用：`proposal_category`(law/personnel/budget)、`HouseRef{code, account_hex}`、`LegProposalState{proposal_id, proposal_category, tier, scope_code, proposer_body, houses, vote_type, current_house, stage, status, content_hash}`。
- 法律案：`ProposeLawInput{law_action, tier, scope_code, proposer_body, houses, executive, legislature, vote_type, title, title_en, chapters, effective_at}`；`LawChapter>LawSection>LawArticle>LawClause`(章>节>条>款,`number/title/title_en/body/body_en/text/text_en`)。
- 任免案：`PersonnelAction{Appoint,Dismiss,Replace}`；`PersonnelDecision{action, office_institution_code, office_title, office_seat, nominee_cid_number, nominee_name, term_index, term_years, reason}`；`ProposePersonnelInput{tier, scope_code, proposer_body, houses, vote_type, decision}`。
- 预算案：`BudgetPlan{budget_entity_code, fiscal_year, categories, total_revenue, total_expenditure}`；`BudgetClass>BudgetSection>BudgetItem>BudgetSubitem`(类>款>项>目,`code/name`，`revenue/expenditure` u128 分)；`ProposeBudgetInput{...}`。
- 大屏：`LegislationBoard{institution_code, online_seats, live_tally, proposals}`、`SeatPresence{seat, admin_account, admin_name, online}`、`HouseTallyRow{proposal_id, current_house, approve, reject}`。
- 能力位：`can_view_legislation/can_propose_legislation/can_cast_house_vote/can_sign_legislation/can_propose_personnel/can_propose_budget`(后两预留)。
- 动作：`ProposeEnactLaw/ProposeAmendLaw/ProposeRepealLaw/CastHouseVote/CastReferendumVote/ExecutiveSign/OverrideSign/GuardVote`(写类 PASSKEY_COLD_SIGN)，`ProposePersonnel/ProposeBudget`(预留)。

## 分步卡

### Phase 0 · 地基
- **01** `domains/legislation/{mod,model}.rs`：统一信封 + `ProposalCategory` 枚举 + `HouseRef`/`LegProposalState` + 路由壳。依赖:无。风险:低。
- **02** 扩展点：`platform/capability.rs` 加 6 立法能力位 + 立法机构能力矩阵；`auth/operation_auth.rs` 加 10 动作 variant（穷尽 match）+ `catalog.rs` 档位；前端 `capabilityMap.ts` 镜像。依赖:01。风险:中。
- **03** `category.rs`：本节点机构码 → 可发起 category×tier×vote_type×proposer_body 候选（proposer≠voting_body 解耦在此）。依赖:01。风险:中。

### Phase 1 · 法律案后端
- **04** `law/model.rs`：`ProposeLawInput` + `LawChapter/Section/Article/Clause`（章节条款）。依赖:01。风险:低。
- **05** `law/chain_propose.rs` + `service.rs`：组织 houses/proposer_body/executive/vote_type，对齐链端 `propose_enact/amend/repeal_law` SCALE 逐字段 + `precheck_legislation_scope`（写入边界 fail-closed）。依赖:03/04。风险:高。
- **06** `law/chain_house_vote.rs` + `chain_referendum_vote.rs` + `chain_*_sign.rs` + `chain_guard_vote.rs`：提交各表决/签署 extrinsic（冷签）。依赖:05。风险:中。
- **07** `law/chain_read.rs` + `chain_read_proposal.rs`：链读 get_law/list_laws + LegProposalState/tally（过 scope）。依赖:01。风险:中。

### Phase 2 · 法律案前端 operator
- **08** `operator/LegislationView.tsx` + `ProposeMenu.tsx`（按本机构码 + category 渲染；唯一发起入口；能力位门控）。依赖:02/03。风险:中。
- **09** `operator/law/LawEditorModal.tsx`（章节条款编辑→contentHash）+ `ProposeLawModal.tsx`（冷签复用 CitizenSignatureModal→propose）。依赖:05/08。风险:中。
- **10** `operator/law/HouseVotePanel.tsx`（cast_house_vote）+ `SignActionModal.tsx`（只读展示签署/会签/护宪进度，本轮不做发起入口）+ `ProposalProgressView.tsx` + `LawListTable.tsx`。依赖:06/07。风险:中。

### Phase 3 · 大屏 display
- **11** 后端 `display/{handler,service}.rs`：`/api/legislation/display/*` 聚合在线席位 + 实时 tally + 提案进度（纯读链，会话只读）。依赖:07。风险:中。
- **12** 前端 `display/`：`LegislationBoardView`(独立路由) + `SeatGrid` + `LiveTallyBoard` + `ProposalTicker`（轮询刷新，零操作入口）。依赖:11。风险:中。

### Phase 4 · 提案类型预留结构
- **13** `personnel/model.rs`：`PersonnelDecision`/`PersonnelAction`/`ProposePersonnelInput`（仅结构 + 前端 `PersonnelDecisionModal` 占位灰显，不接链）。依赖:01。风险:低。
- **14** `budget/model.rs`：`BudgetPlan` + 类款项目 + `ProposeBudgetInput`（仅结构 + `BudgetPlanModal` 占位，不接链）。依赖:01。风险:低。

### Phase 5 · 收尾
- **15** 注释补全 + 文档（ADR-027 补 onchina 落地节 + 提案类型维度决议）+ 残留清理（核查 education 模块未误承载教委会立法）。依赖:全部。风险:低。

## 待确认 / 待后续（不阻塞本轮）

- 任免案升级路径（3 次驳回→重要案→立法院独立决议）字段化（`reject_count`/`escalated`）—— 随 Personnel 链路上线时定。
- **职位码表 `office`**（任免案职位真源）—— 待用户定清单或我按宪法整理草案。
- 预算 `类/款/项/目` code 编码规则（国标政府收支分类 vs 自定义）—— 待用户定。
- 国家级行政签署人（总统府）机构码 —— 待核实/补码。
- 任免案/预算案链端 `PROPOSAL_KIND_PERSONNEL/BUDGET` —— 另卡（含 runtime 二次确认）。

## 验收标准

- 每卡 `cargo build -p onchina` + 相关单测通过；`npm --prefix citizenchain/onchina/frontend run build`/tsc 通过。
- 三档鉴权穷尽 match，新增动作漏标分档则编译失败。
- **真实运行态**：真实本地 onchina 服务 + PostgreSQL + 真实页面，跑通「立法机构管理员扫码登录 → 发起法律案(章节条款) → 院内/两院表决 → 查看进度」，大屏只读展示在线席位与实时投票。
- scope fail-closed：省管理员不能发起全国法律、市不能发起省法律（写入边界拒绝）。
- 法律案 SCALE 与链端逐字节一致（冷签可过）。
- 任免案/预算案结构编译通过、UI 占位灰显，不可误触发链提交。
- 零残留：无未用别名、无历史化注释、education 模块无立法残留。

## 进度

- [x] 需求分析（立法院管理员权限/功能/UI）
- [x] 宪法核实（三级两院、提案主体、表决程序、三权分立、任免/预算路径）
- [x] 政府提案权分析结论：不改宪法，建模提案类型（法律案/任免案/预算案）
- [x] 框架设计定稿（目录/文件/字段/中文注释，用户确认）
- [x] 主任务卡创建（本卡）
- [ ] Phase 0–5 实现
