# 公权机构详情页对齐治理机构

## 任务需求

wuminapp「公民 → 公权 → 公权机构 → 机构详情」页版式与交互重构，对齐治理机构详情页：

1. 机构信息卡：删「账户数」行，改显「法定代表人」(取 SFID `legal_rep_name`，无则留空)；「所属」标签改「所属地」(值不变 省·市)；每行之间加横线分隔。
2. 「账户与余额」整卡收为单行「机构账户(N)」+ 右箭头，点击进现有「全部账户页」；删主账户/费用账户余额展示，缩小行高。
3. 「管理员(N)」行右侧加右箭头，点击进入公权专用只读管理员列表页。
4. 提案拆为两块：原「提案」卡 → 改为「发起入口」(治理同款 badge+箭头，本期先占位)；管理员行下方新增「提案列表」(治理同款卡片样式)。

调整后版面顺序：机构信息 → 机构账户 → 提案发起入口 → 管理员 → 提案列表。

## 硬边界

- 发起入口本期**只占位**：公权机构提案类型为 转账/费用划转/更换管理员，但不接真实发起流程(不做 PublicInstitutionEntity→InstitutionInfo 桥接)，点击给占位反馈。后续单独任务卡接发起流程。
- 管理员列表本期为**轻量只读**：公权管理员来自 SFID 系统、尚未对接，列表空时显占位文案，不做冷钱包导入/扫码激活。后续单独任务卡接 SFID 管理员来源。
- 不动链端、SFID 号生成、账户派生、订阅逻辑、账户页本身。
- `legal_rep_name` 经确认可对外公开(公权机构法定代表人为公开信息)，加入公权 BFF 公开白名单。

## 预计修改目录

- `sfid/backend/wuminapp/public_institution.rs`：`PublicInstitutionRow` + SQL SELECT 暴露 `s.legal_rep_name`(白名单)，更新安全红线注释与测试。
- `wuminapp/lib/citizen/public/data/public_institution_dto.dart`：DTO + Isar 实体映射加 `legalRepName`。
- `wuminapp/lib/isar/wallet_isar.dart`：`PublicInstitutionEntity` 加 `legalRepName`，重生成 `.g.dart`。
- `wuminapp/lib/citizen/public/public_institution_detail_page.dart`：五段重排(主战场)。
- `wuminapp/lib/citizen/public/public_institution_admin_list_page.dart`：**新增**公权轻量只读管理员列表页。
- `wuminapp/test/citizen/public/public_institution_detail_test.dart`：同步断言(机构账户(N)/提案列表/管理员入口)。
- `memory/05-modules/`：更新公权机构相关技术文档，清理旧文案残留。

## 验收计划

- SFID 后端：`cargo fmt && cargo check`，`cargo test -p sfid-backend`(public_institution 测试)。
- Mobile：`dart run build_runner build --delete-conflicting-outputs`(重生成 Isar)，`dart analyze`，`flutter test`。
- 版面核对：机构信息(法定代表人/所属地/分隔线)、机构账户单行入口、提案发起占位、管理员入口、提案列表。
- 文档更新、中文注释完善、旧文案(账户数/账户与余额/更多账户/所属)残留扫描。

## 执行记录

- SFID 后端 `wuminapp/public_institution.rs`:`PublicInstitutionRow` 加 `legal_rep_name`(`#[serde(skip_serializing_if = "Option::is_none")]`),SQL SELECT 末列追加 `s.legal_rep_name`(idx17),`from_pg_row` 读取;更新安全红线注释标注其为已确认可公开字段;测试 `sample_row` 补字段 + 断言 JSON 携带。`cargo fmt && cargo check` 通过,`cargo test public_institution` 2/2 绿。
- 前端 DTO `public_institution_dto.dart`:加 `legalRepName` 字段 + 构造参数 + `fromJson('legal_rep_name')` + `toEntity` 映射。
- Isar 实体 `wallet_isar.dart::PublicInstitutionEntity`:加 `String? legalRepName`,`dart run build_runner build --delete-conflicting-outputs` 重生成 `.g.dart`(exit 0)。
- 新增 `public_institution_admin_list_page.dart`:公权专用轻量只读管理员列表,hex→SS58(prefix2027,非法 hex 兜底原样),空态占位"管理员数据待与 SFID 系统对接"。
- 重构 `public_institution_detail_page.dart`:五段版式(机构信息/机构账户入口/提案发起入口/管理员入口/提案列表);删余额展示+余额拉取;抽 `_entryRow` 单行入口组件;`_row` 加分隔线、label 宽 80。
- 数据包生成器 `tools/generate_public_institution_bundle.mjs` 整行透传 BFF items,无字段白名单 → 重生成即自动带 `legal_rep_name`,无需改。
- 测试:`public_institution_detail_test.dart` 断言对齐新版式(机构账户(3)/发起提案/管理员(1)入口/提案列表)+ 新增管理员入口点击进列表页用例;`public_institution_dto_test.dart` 补 `legalRepName` 解析/落库/缺省 null。`dart analyze` 0 issues;`flutter test test/citizen/public` 18/18 绿。
- 残留扫描:`账户数/账户与余额/更多账户/'所属'` 在 `lib/citizen/public` + `test/citizen/public` 零命中。
- 文档:更新记忆笔记 `project_public_institution_feature_2026_06_13`(补 2026-06-14 详情页重构 + legal_rep_name 白名单链路)。

## 数据包重生成(2026-06-14,后端已重启)

- 确认运行中二进制 `target/debug/sfid-backend`(mtime 13:32 > 改动 13:27)`strings` 含新 SQL 片段 `s.created_at, s.legal_rep_name` → 新二进制在跑。
- `SFID_BASE_URL=http://127.0.0.1:8899 node tools/generate_public_institution_bundle.mjs`:43 省、287,790 机构、version=2026-06-14T20:42:56Z,无 429,exit 0。
- **数据现状**:全包 0 处 `legal_rep_name`。库内 PUBLIC/PRIVATE ACTIVE 共 294,162,仅 **1** 个录了法定代表人(`ZS003-SGF1I-761573855-2026`,中枢省巫溪市,程伟),但它是 **PRIVATE 私法人**(category=PRIVATE_INSTITUTION/subject_property=S),不匹配公权目录 `GOV_FROM_WHERE` 任一分支,本就不在公权目录。→ 当前公权机构详情页法定代表人一律留空,**符合预期**;待有公权机构录入法定代表人后,重跑生成器或在线增量同步即自动带值。

## 待续(follow-up)

- 发起提案真实流程(占位 → 接转账/费用划转/更换管理员发起页,需 PublicInstitutionEntity→InstitutionInfo 桥接 + 管理员钱包/激活态)。
- 管理员来源接 SFID 系统(当前链读 AdminsChange 为空;公权管理员权威来源在 SFID)。
