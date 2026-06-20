# 任务卡：SFID 公权机构按行政区划名录重构与公安局业务入口校正

## 任务需求

按最新边界重构 SFID 公权机构设计与实现：公权机构不做上下级机构，国家/部/省/市/镇只作为目录分类和行政区范围；公安局 tab 只作为市公安局业务入口，因为公安局需要承接 CPMS 安装码和身份码业务；CPMS 安装码只能由联邦管理员签发；旧省域管理员对外改为联邦管理员，市管理员保持原名；机构列表只展示简要名称，详情页展示全称和简称；代码完成后更新文档、补中文注释、清理残留。

## 核心原则

- 公权机构不做上下级机构。
- 国家/部/省/市/镇只作为公权机构目录分类和行政区范围，不作为上下级字段。
- 公安局单独 tab，只显示市公安局。
- 公安局独立，是因为它要承接 CPMS 安装码和身份码业务。
- CPMS 安装码只能由联邦管理员签发。
- `sfid/backend/cpms` 核心业务不重构。
- 旧省域管理员对外改为联邦管理员；市管理员不改名。
- 不区分自动生成/人工新增，生成只是初始化录入。
- `sfid_number` 永久不可改。

## 数据模型

`subjects`：

- `sfid_number`：身份 ID，不可变。
- `kind`：`PUBLIC / PRIVATE / CITIZEN`。
- `name`：列表展示名，默认用简称。
- `full_name`：全称。
- `sfid_short_name`：简称。
- `province_code`：省代码。
- `city_code`：市代码，可空。
- `town_code`：镇代码，可空。
- `province`：省名。
- `city`：市名，可空。
- `town`：镇名，可空。
- `status`：`ACTIVE / REVOKED`。

`gov`：

- `sfid_number`。
- `institution_code`：`ZF / LF / SF / JC / JY / CB / CH`。
- `org_code`：机构细类，例如 `PRESIDENT_OFFICE`、`FINANCE_BUREAU`、`CITY_POLICE`。

初始化记录只放 `gov_manifest`，不得把初始化来源放进机构业务字段。

## 列表展示

公权机构列表只显示简要信息：

- 身份 ID。
- 机构名称：只显示 `name`，默认等于简称。
- 行政区。
- 机构类型。
- 状态。
- 账户数。

列表不得同时显示全称和简称。

详情页显示：

- 全称。
- 简称。
- 身份 ID。
- 行政区。
- 机构类型。
- 状态。
- 账户。
- 资料库。
- 操作记录。

## 公安局 Tab

公安局 tab 只显示每个市一个公安局：

```sql
WHERE s.category = 'PUBLIC_SECURITY'
AND g.org_code = 'CITY_POLICE'
AND s.city_code IS NOT NULL
```

公安局列表显示：

- 身份 ID。
- 公安局名称。
- 所属行政区。
- CPMS 状态。
- 安装码状态。
- 身份码业务状态。

“生成 CPMS 安装码”按钮只给联邦管理员显示。市管理员不能签发，只能按权限查看或办理本市业务。

## 管理员角色

- `FEDERAL_ADMIN`：联邦管理员。
- `CITY_ADMIN`：市管理员。

前端文案：

- 联邦管理员列表。
- 市管理员列表。

说明：当前底层角色枚举如仍使用既有 `FEDERAL_ADMIN` 存储值，本任务必须至少把用户可见文案、权限说明、技术文档和安全动作语义全部收口到“联邦管理员”；不得把旧省域管理员名称作为对外名称继续展示。

## 初始化

首次部署执行：

```bash
init-gov
```

一次性录入国家/部/省/市/镇目录分类覆盖的公权机构，以及每个市一个公安局。录入后就是普通公权机构数据。

行政区变化时按范围对账：

```bash
reconcile-gov --province
reconcile-gov --city
reconcile-gov --town
```

只更新名称、简称、行政区字段，不改 `sfid_number`。

## 权限与性能

按省分区继续保留，查询必须 SQL 层限定范围：

```sql
WHERE province_code = $1
AND city_code = $2
```

镇范围：

```sql
WHERE province_code = $1
AND city_code = $2
AND town_code = $3
```

核心索引：

- `subjects(province_code, city_code, town_code, kind, status)`。
- `subjects(province_code, name)`。
- `gov(province_code, city_code, town_code, institution_code)`。
- `gov(province_code, city_code, town_code, org_code)`。

## 建议模块

- `sfid/backend/china`
- `sfid/backend/gov`
- `sfid/backend/admins`
- `sfid/backend/subjects`
- `sfid/backend/core`
- `sfid/frontend/gov`
- `sfid/frontend/admins`
- `sfid/frontend/china`
- `sfid/backend/cpms`
- `memory/01-architecture/sfid`

## 预计修改目录

- `sfid/backend/china`：读取镇行政区划，作为 `town/town_code` 和镇目录公权机构生成输入，涉及代码。
- `sfid/backend/gov`：公权机构初始化、对账、列表、详情和公安局聚合接口，涉及代码。
- `sfid/backend/admins`：旧省域管理员角色对外改名为联邦管理员，市管理员不改名，涉及代码和用户可见文案。
- `sfid/backend/subjects`：全称、简称、镇字段和状态对外模型收口，删除机构上下级字段残留，涉及代码。
- `sfid/backend/core`：表字段、索引、分区和旧字段残留清理，涉及代码。
- `sfid/backend/cpms`：不重构核心业务，只适配联邦管理员权限语义和公安局安装码状态读取，涉及代码。
- `sfid/frontend/gov`：公权机构列表只显示一个机构名称，公安局 tab 单独展示 CPMS/身份码状态，详情显示全称和简称，涉及代码。
- `sfid/frontend/admins`：文案改为联邦管理员列表、市管理员列表，涉及代码。
- `sfid/frontend/china`：增加镇行政区缓存，涉及代码。
- `memory/01-architecture/sfid`：更新 SFID 架构、权限、数据模型、初始化和展示规则，涉及文档。
- `memory/08-tasks`：记录执行进度、验证结果和残留清理，涉及文档。

## 影响范围

- 公权机构和公安局确定性目录生成。
- 公权机构列表和详情字段。
- 镇行政区划读取。
- 联邦管理员文案和权限提示。
- 公安局 CPMS 安装码签发按钮权限。
- SFID 架构文档。

## 主要风险点

- 不得把公安局 tab 扩展成镇公安分支或公安系统列表。
- 不得重构已经完成的 `cpms` 核心业务。
- 不得把市管理员改名。
- 不得把自动生成/人工新增做成机构业务属性。
- 不得改变既有机构或个人的 `sfid_number`。
- 列表不得同时显示全称和简称。
- 不得把初始化来源作为机构业务字段。
- 不得重构 `sfid/backend/cpms` 核心业务。
- 不得恢复旧 `backend/src`、独立链目录或独立前端业务 API 目录。

## 验收标准

- 公权机构按行政区划名录展示，不做上下级机构。
- 公安局 tab 只显示市公安局。
- CPMS 安装码签发只允许联邦管理员。
- 市管理员名称不被改写。
- 公权机构列表只显示一个机构名称。
- 详情页显示全称和简称。
- `subjects` 对外模型包含 `name/full_name/sfid_short_name/town_code/town/status`。
- `gov` 对外模型只包含 `institution_code/org_code` 等机构类型字段，不再包含旧上下级字段。
- 公安局列表包含 CPMS 状态、安装码状态和身份码业务状态。
- 前端用户可见旧省域管理员文案统一改为“联邦管理员”，市管理员不改名。
- 代码和文档不得残留镇公安分支作为公安局方案。
- 文档已同步，残留已清理，构建测试通过。

## 执行记录

- 2026-06-06：按用户完整技术方案补齐任务卡，准备进入代码和文档落地。
- 2026-06-06：完成后端主体/公权模型收口，`subjects` 移除初始化来源业务字段，`gov` 只承接 `institution_code/org_code`，公安局列表限定 `category=PUBLIC_SECURITY AND org_code=CITY_POLICE AND city_code IS NOT NULL`。
- 2026-06-06：完成前端公权列表简要展示、公安局 tab 独立列、详情全称/简称/行政区/状态/资料库/操作记录展示，并清理私权详情页误带的 CPMS 分支。
- 2026-06-06：按最新要求删除机构上下级字段和旧等级展示概念；修正市立法会为 `xx市立法会 / xx市公民立法委员会`，公安局为 `xx市公安局 / xx市公民安全局`，市国防局为 `xx市国防局 / xx市国家防务局`，公民自治委员会简称为 `自治会`。
- 2026-06-06：完成联邦管理员文案、角色值、CPMS 权限说明和 SFID 文档同步；残留扫描未发现旧角色名、旧公安局方案、旧初始化来源业务字段残留。
- 2026-06-06：修复 `ensure-gov` 全量目录统计超大数组问题，已有目标计数按 10000 条分块，批量写入按 5000 条分块，避免 294161 条目录重刷时再次失败。
- 2026-06-06：本机数据库已直接删除旧上下级列和旧索引，执行新版 `ensure-gov` 成功：`updated=294161`、`account_inserted=588322`、`total_after=294161`。
- 2026-06-06：严格校验通过 `check-gov --strict`：`ok=true`、`missing=0`、`mismatched=0`、`missing_accounts=0`、`obsolete=0`。
- 2026-06-06：数据库样本验证通过：旧锦程市错误名称计数为 0；合肥市公安局、合肥市立法会、合肥市自治会、合肥市国防局，以及国家众议会、联邦审计署、联邦人事局、交通部全称/简称符合目标命名。
- 2026-06-06：验证通过 `cargo fmt --manifest-path sfid/backend/Cargo.toml`、`cargo check --manifest-path sfid/backend/Cargo.toml`、`cargo check --tests --manifest-path sfid/backend/Cargo.toml`、`npm run build --prefix sfid/frontend`。
