# Step 1：SFID 端清算行资格白名单与候选搜索 API

任务需求：
作为清算行三阶段实施（[ADR-007](../../04-decisions/ADR-007-clearing-bank-three-phase.md)）的第 1 阶段，在 SFID 端落地"清算行资格白名单"判定能力 + 收紧已有 `/clearing-banks/search` API + 新增 `/clearing-banks/eligible-search` 候选搜索 API + SFID 前端 badge / hint。

资格定义：仅 `(a3=SFR ∧ sub_type=JOINT_STOCK) ∨ (a3=FFR ∧ parent.a3=SFR ∧ parent.sub_type=JOINT_STOCK)` 的机构有资格成为清算行。规则参见 [memory/05-modules/sfid/clearing-bank-eligibility.md](../../05-modules/sfid/clearing-bank-eligibility.md)。

所属模块：SFID Agent（sfid/backend + sfid/frontend；零 runtime / wumin / wuminapp 改动）

## 输入文档

- [memory/04-decisions/ADR-007-clearing-bank-three-phase.md](../../04-decisions/ADR-007-clearing-bank-three-phase.md)
- [memory/05-modules/sfid/clearing-bank-eligibility.md](../../05-modules/sfid/clearing-bank-eligibility.md)
- [memory/08-tasks/templates/sfid-backend.md](../templates/sfid-backend.md)
- [memory/08-tasks/templates/sfid-frontend.md](../templates/sfid-frontend.md)

## 必须遵守

- 不可突破模块边界：本步只动 sfid/backend + sfid/frontend，零 runtime / wumin / wuminapp 改动
- 不可绕过既有契约：`/clearing-banks/search` 路由不变，仅收紧返回集合 + 扩展响应字段（新字段加在末尾，不改原字段顺序）
- 不可擅自修改安全红线：资格判定为只读判定，不影响机构创建/激活/账户操作流程
- 不清楚逻辑时先沟通

## 变更范围（文件级）

### 后端（sfid/backend）

- `src/institutions/service.rs`
  - 新增 `is_clearing_bank_eligible(inst, parent) -> bool` 纯函数（参数：机构 + 可选 parent 引用，简化调用方）
  - 新增 6 case 单元测试（SFR-JOINT_STOCK / SFR-LIMITED_LIABILITY / SFR-NON_PROFIT / FFR-合规 parent / FFR-不合规 parent / FFR-缺 parent）
- `src/institutions/handler.rs`
  - 修改 `AppClearingBankRow`：新增 `sub_type` / `parent_sfid_id` / `parent_institution_name` / `parent_a3` 字段（追加在末尾）
  - 修改 `app_search_clearing_banks`：先跨 43 省收集 SFR-JOINT_STOCK 的 sfid_id 全国 HashSet，再二轮收集候选并按资格白名单过滤
  - 新增 `EligibleClearingBankRow` + `app_search_eligible_clearing_banks` 函数（不要求主账户已激活）
- `src/main.rs`
  - 注册新路由 `GET /api/v1/app/clearing-banks/eligible-search` → `app_search_eligible_clearing_banks`

### 前端（sfid/frontend）

- `src/utils/clearingBankEligible.ts` — 新建，SFR-JOINT_STOCK 单机构判定（FFR 需 parent 信息时由调用方提供）
- `src/views/institutions/InstitutionListTable.tsx` — 表格新增"清算行资格"列，对 SFR-JOINT_STOCK 显示蓝色 badge `可作为清算行`
- `src/views/institutions/InstitutionDetailPage.tsx` — 详情页头部对 SFR-JOINT_STOCK / FFR(已设 parent_sfid_id) 显示 badge
- `src/views/institutions/PrivateInstitutionLayout.tsx` — sub_type=JOINT_STOCK 选项加 hint 文案"股份公司可参与清算业务"

### 文档

- 新建 `memory/04-decisions/ADR-007-clearing-bank-three-phase.md`
- 新建 `memory/05-modules/sfid/clearing-bank-eligibility.md`
- 更新 `memory/MEMORY.md` 索引（如有 ADR-007 提及）
- 任务卡归档

## 输出物

- 代码：上述前后端改动
- 中文注释：service 新函数完整中文 doc-comment；handler 新结构体字段中文注释
- 测试：service 6 case 单测；handler 集成测试若有现成 fixture 也补
- 文档：ADR-007 + 资格规则文档 + 任务卡
- 残留清理：无遗留 `is_clearing_bank` 字段 / 无注释 TODO / 无未引用代码

## 验收标准

- 资格判定：6 case 单测全绿
- `/clearing-banks/search`：
  - 返回结果只含 SFR-JOINT_STOCK + FFR(parent.SFR ∧ parent.JOINT_STOCK) ∩ 主账户已激活
  - SFR-LIMITED_LIABILITY 即使主账户已激活也不返回
  - 响应字段含 sub_type / parent_sfid_id / parent_institution_name / parent_a3
- `/clearing-banks/eligible-search`：
  - 返回所有满足资格白名单的机构（包括未激活）
  - 含 `main_chain_status` 字段
  - 无 province/city 过滤
- 前端：
  - SFR-JOINT_STOCK 列表行显示"可作为清算行" badge
  - SFR-LIMITED_LIABILITY 不显示
  - PrivateInstitutionLayout 第二步选 JOINT_STOCK 显示提示文案
- 后端：`cargo check -p sfid_backend`（或对应 crate）通过；`cargo test -p sfid_backend institutions::service::tests` 全绿
- 前端：`npm run build` 通过
- 残留：Grep `is_clearing_bank` 全仓零结果（除 ADR/规则文档/历史任务卡历史描述）

## 落地顺序

1. 写 ADR-007 + 规则文档 ✅（与本任务卡同时建立）
2. service.rs 加 `is_clearing_bank_eligible` + 6 case 单测
3. handler.rs 修改 `app_search_clearing_banks`（含跨省 SFR 集合预收集）
4. handler.rs 新增 `app_search_eligible_clearing_banks`
5. main.rs 注册新路由
6. cargo build + cargo test 验证
7. 前端 utils/clearingBankEligible.ts 新建
8. 前端 InstitutionListTable / InstitutionDetailPage / PrivateInstitutionLayout 改造
9. npm run build 验证
10. 文档完整性检查 + 任务卡归档
