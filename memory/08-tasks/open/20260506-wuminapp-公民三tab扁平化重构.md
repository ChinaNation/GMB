任务需求：
wuminapp 公民域三个二级 tab（机构/治理/投票）改造为新结构（治理/投票/公权），
连同目录扁平化、文件改名、引言迁移、公权页占位一并落地。

所属模块：Mobile（wuminapp）

输入文档：
- memory/00-vision/...
- memory/01-architecture/...
- memory/07-ai/agent-rules.md
- memory/07-ai/dual-chat-entry.md
- memory/07-ai/feedback_user_naming_literal.md
- ADR-010 SubjectId 协议（institution_id 统一）

必须遵守：
- 不可突破模块边界（链端 0 改动，spec_version 不动）
- 不可绕过既有契约（链上术语保留：InstitutionInfo / Subjects / Institutions storage 名）
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通

## 范围

### 1. 目录扁平化
删除 `lib/citizen/` 一层，三个 tab 直接落到 lib 顶层：
- `lib/governance/` 治理 tab 入口
- `lib/vote/` 投票 tab
- `lib/public/` 公权 tab（占位）
- `lib/institution/` 通用机构域（治理/公权/注册共用）
- `lib/proposal/` 提案域顶层化
- `lib/citizen_tab_page.dart` 上移到 lib 根

### 2. tab 名 + 默认 tab
`_tabs = ['公权', '投票', '治理']`，默认 `_selectedTab = 1`（投票）。

### 3. 文件 + 类改名
- `AllProposalsView` → `VoteView`，文件 `vote/vote_view.dart`
- `InstitutionListPage` → `GovernanceListPage`，文件 `governance/governance_list_page.dart`
- `_InstitutionSection/_InstitutionCard` → `_GovernanceSection/_GovernanceCard`
- `VotePage` 删除
- 新增 `PublicPage`（`public/public_page.dart`，占位）
- 新增 `ConstitutionQuote`（`vote/constitution_quote.dart`，水印）
- `InstitutionInfo / InstitutionDetailPage / InstitutionAdminService / OrgType / kProvincialCouncils / kProvincialBanks / kNationalCouncil / formatProposalId` 全部保留（链上术语）

### 4. 文案
- citizen tab 字符串 3 个（同上）
- `'机构分类'` → `'治理机构'`（governance_list_page.dart）
- 删除"查看各级机构信息与治理提案"副标题

### 5. 引言水印
原 vote_page.dart 引言抽出为 `ConstitutionQuote` widget，
在 vote_view.dart 中以 Stack + Opacity 0.06 + IgnorePointer 始终显示，
若隐若现做底层背景。

### 6. 不在本卡范围（下卡再做）
- 公权机构数据扩容（lf/sf/jc/zf/jy 灌入 institution_data）
- 公权页省/市垂直导航栏
- 链端 primitives 公权机构上链

输出物：
- 代码（目录重构 + 类改名 + 文案 + 占位页 + 水印）
- 中文注释
- 测试（现有 widget test 保持全过）
- 文档更新（无 ADR 必要，纯 UI 重构）
- 残留清理（空目录 lib/citizen/、test/citizen/ 删除）

验收标准：
- `flutter analyze` 0 error 0 warning
- `flutter test` 全过（当前 65 widget test）
- 应用启动后切换底部"公民"tab，三个二级 tab "公权/投票/治理" 显示正常
- 投票 tab 提案列表 + 水印背景显示正常
- 治理 tab 机构列表 + 详情页进入路径不破
- 28 个外部 import 路径全部修复，0 红线
- 链端 0 改动，spec_version 不动

影响范围：
- wuminapp（lib/ + test/）
- 不影响 citizenchain / wumin / sfid / cpms
