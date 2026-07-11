# 公权机构详情页对齐治理机构

## 任务需求

citizenapp「公民 → 公权 → 公权机构 → 机构详情」页版式与交互重构，对齐治理机构详情页：

1. 机构信息卡：删「账户数」行，改显「法定代表人」(取 CID `legal_rep_name`，无则留空)；「所属」标签改「所属地」(值不变 省·市)；每行之间加横线分隔。
2. 「账户与余额」整卡收为单行「机构账户(N)」+ 右箭头，点击进现有「全部账户页」；删主账户/费用账户余额展示，缩小行高。
3. 「管理员(N)」行右侧加右箭头，点击进入公权专用只读管理员列表页。
4. 提案拆为两块：原「提案」卡 → 改为「发起入口」(治理同款 badge+箭头，本期先占位)；管理员行下方新增「提案列表」(治理同款卡片样式)。

调整后版面顺序：机构信息 → 机构账户 → 提案发起入口 → 管理员 → 提案列表。

## 硬边界

- 发起入口本期**只占位**：公权机构提案类型为 转账/费用划转/更换管理员，但不接真实发起流程(不做 PublicInstitutionEntity→InstitutionInfo 桥接)，点击给占位反馈。后续单独任务卡接发起流程。
- 管理员列表本期为**轻量只读**：公权管理员来自 CID 系统、尚未对接，列表空时显占位文案，不做冷钱包导入/扫码激活。后续单独任务卡接 CID 管理员来源。
- 不动链端、CID 号生成、账户派生、订阅逻辑、账户页本身。
- `legal_rep_name` 经确认可对外公开(公权机构法定代表人为公开信息)，加入公权 BFF 公开白名单。

## 预计修改目录

- `citizencode/backend/citizenapp/public_institution.rs`：`PublicInstitutionRow` + SQL SELECT 暴露 `s.legal_rep_name`(白名单)，更新安全红线注释与测试。
- `citizenapp/lib/citizen/public/data/public_institution_dto.dart`：DTO + Isar 实体映射加 `legalRepName`。
- `citizenapp/lib/isar/wallet_isar.dart`：`PublicInstitutionEntity` 加 `legalRepName`，重生成 `.g.dart`。
- `citizenapp/lib/citizen/public/public_institution_detail_page.dart`：五段重排(主战场)。
- `citizenapp/lib/citizen/public/public_institution_admin_list_page.dart`：**新增**公权轻量只读管理员列表页。
- `citizenapp/test/citizen/public/public_institution_detail_test.dart`：同步断言(机构账户(N)/提案列表/管理员入口)。
- `memory/05-modules/`：更新公权机构相关技术文档，清理旧文案残留。

## 验收计划

- CID 后端：`cargo fmt && cargo check`，`cargo test -p citizencode-backend`(public_institution 测试)。
- Mobile：`dart run build_runner build --delete-conflicting-outputs`(重生成 Isar)，`dart analyze`，`flutter test`。
- 版面核对：机构信息(法定代表人/所属地/分隔线)、机构账户单行入口、提案发起占位、管理员入口、提案列表。
- 文档更新、中文注释完善、旧文案(账户数/账户与余额/更多账户/所属)残留扫描。

## 执行记录

- CID 后端 `citizenapp/public_institution.rs`:`PublicInstitutionRow` 加 `legal_rep_name`(`#[serde(skip_serializing_if = "Option::is_none")]`),SQL SELECT 末列追加 `s.legal_rep_name`(idx17),`from_pg_row` 读取;更新安全红线注释标注其为已确认可公开字段;测试 `sample_row` 补字段 + 断言 JSON 携带。`cargo fmt && cargo check` 通过,`cargo test public_institution` 2/2 绿。
- 前端 DTO `public_institution_dto.dart`:加 `legalRepName` 字段 + 构造参数 + `fromJson('legal_rep_name')` + `toEntity` 映射。
- Isar 实体 `wallet_isar.dart::PublicInstitutionEntity`:加 `String? legalRepName`,`dart run build_runner build --delete-conflicting-outputs` 重生成 `.g.dart`(exit 0)。
- 新增 `public_institution_admin_list_page.dart`:公权专用轻量只读管理员列表,hex→SS58(prefix2027,非法 hex 兜底原样),空态占位"管理员数据待与 CID 系统对接"。
- 重构 `public_institution_detail_page.dart`:五段版式(机构信息/机构账户入口/提案发起入口/管理员入口/提案列表);删余额展示+余额拉取;抽 `_entryRow` 单行入口组件;`_row` 加分隔线、label 宽 80。
- 数据包生成器 `citizenapp/tools/generate_public_institution_bundle.mjs` 整行透传 BFF items,无字段白名单 → 重生成即自动带 `legal_rep_name`,无需改。
- 测试:`public_institution_detail_test.dart` 断言对齐新版式(机构账户(3)/发起提案/管理员(1)入口/提案列表)+ 新增管理员入口点击进列表页用例;`public_institution_dto_test.dart` 补 `legalRepName` 解析/落库/缺省 null。`dart analyze` 0 issues;`flutter test test/citizen/public` 18/18 绿。
- 残留扫描:`账户数/账户与余额/更多账户/'所属'` 在 `lib/citizen/public` + `test/citizen/public` 零命中。
- 文档:更新记忆笔记 `project_public_institution_feature_2026_06_13`(补 2026-06-14 详情页重构 + legal_rep_name 白名单链路)。

## 数据包重生成(2026-06-14,后端已重启)

- 确认运行中二进制 `target/debug/citizencode-backend`(mtime 13:32 > 改动 13:27)`strings` 含新 SQL 片段 `s.created_at, s.legal_rep_name` → 新二进制在跑。
- `ONCHINA_BASE_URL=http://127.0.0.1:8899 node citizenapp/tools/generate_public_institution_bundle.mjs`:43 省、287,790 机构、version=2026-06-14T20:42:56Z,无 429,exit 0。
- **数据现状**:全包 0 处 `legal_rep_name`。库内 PUBLIC/PRIVATE ACTIVE 共 294,162,仅 **1** 个录了法定代表人(`ZS003-SGF1I-761573855-2026`,中枢省巫溪市,程伟),但它是 **PRIVATE 私法人**(category=PRIVATE_INSTITUTION/subject_property=S),不匹配公权目录 `GOV_FROM_WHERE` 任一分支,本就不在公权目录。→ 当前公权机构详情页法定代表人一律留空,**符合预期**;待有公权机构录入法定代表人后,重跑生成器或在线增量同步即自动带值。

## UI 尺寸对齐治理机构(2026-06-14 二改,user review 后)

一改只对齐了"分几段"的顺序,UI 度量没对齐治理机构,二改按治理组件尺寸逐一重做:
- 机构信息卡:删机构名标题(名称只在 AppBar,对齐治理);扁平 `_row` 换成治理 `_buildAccountInfoTile` 同款 `_infoTile`(32×32 图标 + 上标签11下数值13),Divider(18) 夹在高 tile 间不再显挤;卡 padding h14/v12、radius12、border primary0.18。
- 机构账户/管理员入口:压缩的 `_entryRow`(32图标/v11/单行)换成治理 `_buildAdminEntry` 同款 `_entryCard`(36×36 图标 primaryDark、标题15、副标题12"共N个账户/共N位管理员"、v12、radius12)。
- 提案发起入口:换成治理 `_buildHeader` 同款 44×44 hero(图标22 + 「提案」徽章 + 副文「发起提案」+ 箭头),本期仍占位(点击 SnackBar)。
- 提案列表:标题16/w700/primaryDark;空态换治理同款 surfaceMuted 大盒(ballot 图标+「暂无提案」);提案卡标题15/primaryDark。
- 测试断言改副标题双行("共3个账户"/"共1位管理员");`dart format`+`analyze 0`+detail 4/4 绿。**未做真机截图核对**。

## 治理⇄公权详情页统一(2026-06-14 三改,提交=统一治理和公权机构详情页)

目标:两端详情页版面/组件/尺寸统一。统一后版面均为 机构信息(身份ID/主账户/主账户余额/法定代表人/所属地) → 机构账户(独立行→全部账户页) → 提案(发起入口) → 管理员(入口) → 提案列表。
- **R0 发起提案行齐高**:公权 `_proposalEntry` + 治理 `_buildHeader` 图标 44→36 / size 22→18 / radius 12→10(公权底色 alpha 0.12→0.08);padding 本就 v12 不动。两端发起提案行与机构账户/管理员行齐高。
- **R1 公权 += 主账户 + 主账户余额**:`_loadDynamics` 重新加主账户余额拉取(`balances([mainHex])`);信息卡插「主账户」(SS58)/「主账户余额」(读取中/未激活/N 元)两 tile。
- **R2 治理 += 法定代表人 + 所属地**:新增 `citizen/public/data/cid_directory_lookup.dart`(`CidDirectoryLookup`/`CidDirectoryInfo`),按 cid 反查公权目录 Isar 库(治理内置机构都带真实 CID 号且在确定性目录内,已验证 GD/ZS/LN 储会均在包内)。治理 `_loadDirectory` 异步反查回填,信息卡插「法定代表人」/「所属地」两 tile;反查不到(注册机构账户)留空。库空时 `ensureBundleLoaded` 一次性兜底。
- **R3 治理更多账户 → 独立行**:删内联展开整套(`_buildMoreAccountsToggle`/`_buildExpandedAccounts`/`_buildExpandedAccountItem`/`_extraAccountSources`/`_toggleExtraAccounts`/`_loadExtraAccounts`/`_InstitutionAccountView`/`_chainRpc`);新增 `governance/organization-manage/institution_accounts_page.dart`(`GovernanceInstitutionAccountsPage`,对标公权全部账户页,列主+费+安全+两和+质押,批量余额);信息卡下方加「机构账户」独立行(治理 `_buildAdminEntry` 同款 36px)→ 跳该页。
- 验证:`dart analyze`(governance + citizen/public)0;`dart format` 过;公权 18/18 + 反查单测 3/3。治理无 detail widget 测试(不牵连)。**未做真机截图核对**。
- 提交边界:工作区有**另一窗口的 Chat/P2P 功能**(im/ 目录、main.dart、citizenchain node、ADR-020 等)在并行,本次只按显式路径提交本窗口详情页统一相关文件,不碰其他。CID 后端 `legal_rep_name` BFF 改动已在更早的提交 `6e77a3a5 更新公权机构中的信息显示` 落地。

## 待续(follow-up)

- 发起提案真实流程(占位 → 接转账/费用划转/更换管理员发起页,需 PublicInstitutionEntity→InstitutionInfo 桥接 + 管理员钱包/激活态)。
- 管理员来源接 CID 系统(当前链读 AdminsChange 为空;公权管理员权威来源在 CID)。
- 两端详情页深度统一(抽共享组件/统一 view-model)——当前为结构/视觉统一,代码仍两文件。
