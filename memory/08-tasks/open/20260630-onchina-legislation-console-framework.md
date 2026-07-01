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
- ~~国家级行政签署人（总统府）机构码~~ —— **已核实 = PRS**（`code.rs:338`），Phase 1B 路由表已落地。
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
- [x] **Phase 0 地基完成（2026-06-30）**：
  - **01** 新建 `domains/legislation/{mod,model}.rs`：`ProposalCategory{Law/Personnel/Budget}`（serde snake_case 对齐前端 `'law'|'personnel'|'budget'`，`as_u8` 0/1/2）。**细化**：统一信封读模型 `HouseRef`/`LegProposalState` 移至 Phase 1（随 `chain_read_proposal` 消费方一并落地，避免悬空结构）；本轮 model.rs 只落 `ProposalCategory`。
  - **03** 新建 `domains/legislation/category.rs`：`legislation_role`（机构码→发起院/复议院/仅提案，单源）+ `proposable_candidates`（机构码→法律案候选 tier×vote_type；参议会空、政府空待 Phase 4）。候选为 Phase 1 发起菜单 API 数据源（`#[allow(dead_code)]` 标注 Phase 1 消费 + 单测覆盖）。
  - **02** 扩展点：`platform/capability.rs` 新增 6 立法能力位 + `legislation_capabilities` 按角色下发（发起院=发起+表决 / 参议会=只表决 / 教委会自治会=只提案；签署/任免/预算位本轮恒 false）；`auth/operation_auth.rs` 新增 10 动作 variant（穷尽 match 全归 PasskeyColdSign）+ `as_str`/`parse_action_type` 往返；前端 `platform/capabilityMap.ts` 镜像 6 位 + EMPTY 兜底。
  - **验收**：`cargo test -p onchina` 88 passed（+8 新测，0 回归）· `cargo check -p onchina` 零警告 · 前端 `tsc --noEmit` 0 err · 改动仅 onchina（未触 runtime）。
- [x] **Phase 1A 法律案链交互编码器完成（2026-06-30）**：
  - 新建 `domains/legislation/law/{mod,chain_propose,chain_vote}.rs`：裸 SCALE call-data 编码器,**复用 `core::institution_call` 的「构造 call data → origin 冷签 → CitizenWallet 提交」通道**(onchina 不拼签名尾、不提交)。
  - `chain_propose.rs`：`propose_enact/amend/repeal_law`(pallet **27** call 0/1/2)+ 章>节>条>款 SCALE 镜像(`ChapterArg` 等派生 Encode,字段顺序锁死链端 Chapter/Section/Article/Clause)。
  - `chain_vote.rs`：`cast_house_vote`/`cast_referendum_vote`/`executive_sign`/`override_sign`/`guard_vote`(pallet **28** call 1–5,均 `(proposal_id:u64, approve:bool)`)。`prepare_population_snapshot`(call 0,`PopulationScope`)随公投增量。
  - **交叉校验**:`tests` 用链端真实 `legislation_yuan::{Tier,VoteType}` + codec `.encode()` 逐字节比对(新增 dev-dep `legislation-yuan`);确认 enact 全参数、amend/repeal 前缀+law_id、vote `(u64,bool)` 形态字节级一致。
  - **验收**:`cargo test -p onchina` **93 passed**(+5,0 回归)· `cargo check` **零警告** · 改动仅 onchina + Cargo.lock(未触 runtime)。
- [x] **Phase 1B 组织逻辑层完成（2026-06-30）**：
  - 新建 `law/model.rs`：`ProposeLawInput`（houses/executive/legislature **不收前端**,后端按宪法路由解析,防越权）+ `LawChapter/Section/Article/Clause`（serde camelCase）+ `to_chapter_args`（DTO→编码器入参)。
  - 新建 `law/routing.rs`：宪法路由单源（国家[NRP,NSN]/教育[NED,NSN]/省[PRP,PSN]/市[CLEG];executive=总统府 **PRS**/省政府 PGV/市政府 CGOV;legislature=NLG/PLG/None;宪法案按国家;省教育案 None）。
  - 新建 `law/service.rs`：`build_propose_law_call`（请求+本节点机构+路由+`resolve_account` 闭包注入 → `ChainCall`,消费 Phase 1A 编码器）+ `precheck_legislation_scope`（写入边界 fail-closed:层级越权/区码不符拒）+ `build_house_vote_call` + `LegislationError`。
  - **验收**:`cargo test -p onchina` **107 passed**（+14,0 回归)· 零警告 · 含市教委会发起 proposer≠houses[0] 解耦、scope 越权拒、省教育案无路由等用例 · 改动仅 onchina（runtime 未触）。
  - **解决**:国家行政签署人机构码 = 总统府 **PRS**（`code.rs:338`),原「待核实」项闭合。
  - **细节**:account 解析以闭包注入保持纯函数可测;真实链读 `resolve_account` 在 1B-2 接入。
- [x] **Phase 1B-2 解码层完成（2026-06-30）**：
  - `chain_propose.rs`:章节 SCALE 镜像(`ChapterArg` 等)改为双向 `Encode + Decode`,供发起编码与链读解码共用(单源)。
  - 新建 `law/chain_read.rs`:`OnChainLaw`/`OnChainLawVersion` **Decode 镜像**(字段顺序锁死链端 `Law`/`LawVersion`,Tier/LawStatus/VoteType 作单字节枚举)+ `decode_law`/`decode_law_version` + `build_law_view`(字节→String、账户→0x hex、章节→可读)。
  - `law/model.rs`:读模型 `HouseRef`/`LawView`(serde camelCase)+ 逆向转换 `to_law_chapters`/`house_ref`/`institution_code_text`;章节 DTO 改 `Serialize+Deserialize`(输入+展示共用)。
  - **交叉校验**:`tests` 用链端真实 `Tier`/`LawStatus`/`VoteType` 编码 golden → onchina 镜像 decode 回读字段一致(对称 Phase 1A 编码校验);+ to_chapter_args↔to_law_chapters 往返、机构码去尾 \0。
  - **验收**:`cargo test -p onchina` **111 passed**(+4,0 回归)· 零警告 · fmt clean · runtime 未触。
  - **细节(本步未含,转 live)**:account 解析需「机构码+scope → cid_number(链读)→ OP_MAIN 派生」,且 subxt 取数(`fetch_*`)需运行态链验收 → 归 Phase 1B-2-live;`chain_read_proposal`(LegProposalState)同 decode 镜像范式,归 Phase 1B-2c。
- [x] **Phase 1B-2-live 账户派生 + 链取数完成（2026-06-30）**：
  - `law/chain_read.rs`:`derive_house_account(cid_number)→[u8;32]`(复用 `institution::accounts::derive::derive_account(cid,"主账户")`,SS58=2027/OP_MAIN/GMB 单源)——**金标校验**:对 `primitives` fixture 向量 `LN001-NRC0G-944805165-2026·主账户 = b38e86de…` 逐字节一致(离线可验证,锁死派生口径)。
  - `law/chain_read.rs`:subxt `fetch_law`/`list_laws_by_scope`/`fetch_law_version`——**整表 iterate + 镜像 decode + 客户端按已解码字段过滤**(ADR-018;law_id/tier/scope_code 均在 value 内,**无需 storage key 反解 / dynamic Value key / storage_key_suffix**,零 chain_runtime 改动),复用既有读链范式,compile-verified。
  - **验收**:`cargo test -p onchina` **112 passed**(+1 派生金标,0 回归)· 立法模块**零警告** · runtime 未触。
  - **待续(转 runtime 验收)**:`resolve_house_account` 全链路 = 「机构码+scope → cid_number(subjects 表既有查询)→ `derive_house_account`」;subjects 查 + scope_code↔省市码换算在 handler(1B-5)组合,需运行态 onchina+DB+链实测(读出 genesis 宪法、解出立法机构账户)。
  - 注:本轮全库另有 citizens/db/main/runtime-citizen-identity 等**非本任务外部改动**(并发进程/hook),未触碰。
- [x] **Phase 1B-2c 提案进度链读完成（2026-06-30）**：
  - 新建 `legislation/chain_read_proposal.rs`:`OnChainProposal`/`OnChainLegMeta`/`OnChainVoteCount32/64` **Decode 镜像**(字段序锁死 votingengine `Proposal`、legislation-vote `LegislationMeta`、`VoteCountU32/U64`)+ `LegProposalState`/`VoteTally` 只读 DTO(serde camelCase)+ `build_leg_proposal_state`(**只搬运,绝不计票**)。
  - **PopulationScope 规避**:`referendum_scope: Option<PopulationScope>` 是 `LegislationMeta` 末字段且投影不需要 → 用**前缀解码镜像**(SCALE decode 只读声明字段、忽略尾部字节),无需引入 `PopulationScope` 结构。单测 `leg_meta_prefix_mirror_ignores_trailing_referendum_scope` 验证成立。
  - subxt `fetch_proposal_state(proposal_id)`:泛型 `fetch_value_by_proposal_id::<V>`(iterate + `storage_key_suffix::<8>` 取 u64 key + 镜像 decode)读 Proposal/LegMeta/两 tally 装配;`chain_runtime::storage_key_suffix` 改 `pub(crate)` 复用(onchina 内,零 runtime 触碰)。compile-verified。
  - **验收**:`cargo test -p onchina` **115 passed**(+3 提案 golden,0 回归)· 立法模块**零警告** · fmt clean · runtime 未触。
- [x] **Phase 1B-4 冷签 sign_request 完成（2026-06-30）**：
  - **核实纠正**:立法提案/院内表决是**链上 extrinsic**(议员 origin 冷签提交),走**链交易 QR 路径**(`b.a=chain_action_code`、`b.d=SCALE call_data`,`build_sign_request_bytes`),**不走** `onchina_admin_governance` 文本路径 → **不经 `auth/actions.rs` 的 prepare/commit 治理流,零改 actions/action_sign/operation_auth**。范式与 `institution::subjects::registration::build_institution_create_sign_request` 完全一致。
  - 新建 `law/action.rs`:`build_propose_law_sign_request`(input+proposer_code+actor_pubkey+`resolve_account` 注入 → `build_propose_law_call` → `build_sign_request_bytes`)+ `build_house_vote_sign_request`。`resolve_account` 闭包注入保持**与 DB 解耦、可单测**。
  - **验收(单测)**:`cargo test -p onchina` **118 passed**(+3,0 回归)——sign_request 承载正确动作码(enact 0x1B00 / house-vote 0x1C01)+ 非空 b.d(call_data base64)+ 越权路由 422 早拒;立法模块**零警告** · fmt clean · runtime 未触。
  - **待续(1B-5 组合)**:handler 注入真实 `resolve_account` = 「机构码+scope → subjects 查 cid_number → `derive_house_account`」;`precheck_legislation_scope` 在 handler 先拦截。
- [x] **Phase 1B-5 handler + 路由 + DB 解析 + 脚手架清理完成（2026-06-30,compile-verified）**：
  - 新建 `legislation/handler.rs`:`/api/legislation/{propose,house-vote,laws,laws/:id,proposals/:id}` 五端点。**后端三重边界**:① 登录绑定机构 ② `proposable_candidates` 校验本机构能否发起该 tier×vote_type(参议会/非立法机构拒)③ `precheck_legislation_scope` 越权前置(层级/行政区)。发起/表决产 sign_request,读法律/提案直读链。
  - `law/chain_read.rs`:`resolve_house_account(db, code, province_code, city_code)` = `SELECT cid_number FROM subjects WHERE code+省+市` → `derive_house_account`;handler 以「自开连接」闭包按院逐个解析(保持 `Fn`)。scope↔省市码用 `cid::china::{province_code_by_name,city_code_by_name}`。
  - `main.rs`:挂载 `/api/legislation/*`(admin_routes,login 中间件)。
  - **脚手架清理**:移除全 `legislation/*` 模块级 `#![allow(dead_code)]`;`fetch_*` 改用 `decode_law/decode_law_version`(DRY,decoder 转生产消费);仅对**真预留** API 加定点 `allow`(公投/签署/护宪编码器+call index、`as_u8`、decode 镜像布局字段、候选 `category` 字段),各附「预留原因」注释。
  - **验收**:`cargo test -p onchina` **118 passed**(0 回归)· 立法模块**零警告** · fmt clean · runtime 未触(chain_runtime 仅 `storage_key_suffix` 转 pub(crate))。
  - **🔴 待真实运行态验收(需环境)**:running onchina + PostgreSQL + 链 + genesis 宪法——`GET /laws` 读出宪法(law_id=0)、`resolve_house_account` 解出立法机构账户、`POST /propose` 产正确 sign_request、`GET /proposals/:id` 读活跃 stage/tally。首个需核对:subjects 对国家/省/市机构的 `province_code/city_code` 取值(空/`000`)。端到端上链等 CitizenWallet(既定)。
- [x] **Phase 2A 前端数据层完成（2026-06-30）**：新建 `frontend/legislation/{types,api,index}.ts`——`types.ts` camelCase 逐字镜像后端 DTO(LawView/HouseRef/LawChapter…/ProposeLawInput/LegProposalState/VoteTally)+ 层级/表决/状态/阶段数值常量(对齐链端);`api.ts` 五个客户端(`listLaws`/`getLaw`/`getProposalState`/`proposeLegislation`/`castHouseVote`,走 `utils/http.ts::adminRequest`,发起/表决返回 sign_request)。`tsc --noEmit` **0 error**。
- [ ] Phase 2B 前端界面壳 + 路由（`operator/LegislationView` + `App.tsx` ActiveView/Tab,能力位 `canProposeLegislation` 门控)
- [ ] Phase 2C 法律读界面（`LawListTable` + 法律详情 章节条款渲染)
- [ ] Phase 2D 发起(`ProposeMenu` 唯一入口 + `LawEditorModal` 章节编辑 + `ProposeLawModal` + 冷签弹窗接 sign_request)
- [ ] Phase 2E 院内表决(`HouseVotePanel` + 冷签)
- [ ] Phase 2F 提案进度(`ProposalProgressView` 六阶段 + tally)
- [ ] Phase 3 大屏 display（11–12）
- [ ] Phase 4 任免/预算预留结构（13–14）
- [ ] Phase 5 收尾（15）
