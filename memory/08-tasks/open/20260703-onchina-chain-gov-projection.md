# 任务卡：OnChina 公权机构链上唯一真源投影

## 当前状态

已完成。

## 任务背景

OnChina 启动期仍保留 `china.sqlite × 公权机构模板` 的本地生成/对账路径。当前目标态已经明确：当前国家/省/市公权机构在创世时写入链上;镇级和新增机构由注册局运行期注册上链。本地节点软件启动时不得再生成公权机构，只能从链上唯一真源读取并投影到本地缓存。

## 任务目标

- 公权机构真源统一为链上 `PublicManage::Institutions` / `InstitutionAccounts`。
- 公权机构管理员真源统一为链上 `PublicAdmins::AdminAccounts` / `FederalRegistryProvinceGroups`。
- OnChina PostgreSQL 只保存链上投影缓存，不再生成公权机构。
- 删除或废弃 `ONCHINA_GOV_AUTO_RECONCILE` 启动期自动生成路径。
- 启动时链不可达、创世哈希不匹配或链上目录不可读时 fail-closed。
- 更新文档、完善中文注释、清理旧本地生成口径。

## 预计修改目录

- `citizenchain/onchina/src/core/`
  - 用途：扩展链上公权机构读取、账户读取和本地投影同步能力。
  - 边界：只读链上真源，写本地缓存；不得按行政区模板生成业务数据。
  - 类型：后端代码修改。
- `citizenchain/onchina/src/domains/gov/`
  - 用途：把旧公权机构目录生成/对账入口改成链上投影同步入口，保留链上 audit。
  - 边界：不再使用 `china.sqlite × 模板` 作为运行期写库来源。
  - 类型：后端代码修改与残留清理。
- `citizenchain/onchina/src/main.rs`
  - 用途：替换启动期 `gov_manifest` 守卫和自动生成分支。
  - 边界：启动流程只能链读投影或 fail-closed。
  - 类型：后端启动流程修改。
- `citizenchain/onchina/src/core/db.rs`
  - 用途：按需调整本地投影缓存表或同步游标。
  - 边界：字段只表达缓存同步状态，不表达真源版本。
  - 类型：数据库 schema 修改。
- `memory/04-decisions/`
  - 用途：更新 ADR 中公权机构真源与 OnChina 投影规则。
  - 边界：不改行政区真源和码表真源定义。
  - 类型：文档修改。
- `memory/05-modules/citizenchain/onchina/`
  - 用途：更新 OnChina 后端技术文档、启动验收和旧命令口径。
  - 边界：删除启动期本地生成公权机构描述。
  - 类型：文档修改与残留清理。

## 验收标准

- 普通启动不再调用 `reconcile_changed_gov_catalog_db` 或 `write_targets` 生成公权机构。
- 搜索不到运行期依赖 `ONCHINA_GOV_AUTO_RECONCILE` 的自动生成分支。
- 新空库启动只能从链上同步公权机构投影，链不可达时拒绝进入工作台。
- 链上创世 `PublicManage::Institutions` 数量为 49,593 时，本地投影缓存同步为 49,593;后续新增机构按链投影增量进入缓存。
- 后端编译、前端无关检查、真实本地服务启动验收通过。
- 文档已更新，旧本地生成口径已清理。

## 执行记录

- 已移除 OnChina 启动期 `ensure-gov` / `reconcile-gov` / `check-gov` 本地生成目录入口，新增 `sync-gov` 链投影命令。
- 已将公权机构本地缓存切换为链上 `PublicManage::Institutions` / `PublicManage::InstitutionAccounts` 全量投影，投影状态写入 `chain_projection_state(public-gov)`。
- 已删除 `gov_manifest` 表初始化路径，`gov.source` 收敛为 `CHAIN` / `MANUAL`，旧 `GENERATED` 数据启动时清理为 `CHAIN`。
- 已把 CitizenApp 公权机构 BFF 限定到 `gov.source='CHAIN'` 的链投影，版本游标改为 `chain_projection_state(public-gov)`。
- 已把联邦注册局详情入口改为从链投影缓存按 `FRG` 机构码定位唯一 CID，不再依赖本地创世常量查询。
- 已将旧公权 CID 生成函数退出运行态导出，仅保留测试校验用途。
- 已更新 ADR-018、ADR-021、ADR-023、OnChina 后端技术文档和数据安全技术文档。

## 验收记录

- `cargo check -p onchina`：通过，无警告。
- `onchina sync-gov` 真实链读同步：旧创世资产验收记录已废弃;本轮需在 49,593 创世机构链上重跑。
- 数据库校验：旧数量记录已废弃;本轮重跑后记录 `chain_projection_state(public-gov)=OK` 与 `gov.source='CHAIN'` 行数。
- 临时 `serve` 启动验收：`ONCHINA_BIND_ADDR=127.0.0.1:8972` 启动成功；启动同步报告二次变化为 `0`；`GET /api/v1/health` 返回 `status=UP`。
- 公权机构公开接口验收：`/api/v1/app/public-institutions/version?province_name=中枢省` 返回链投影版本；`/api/v1/app/public-institutions?province_name=中枢省&page_size=2` 返回链投影机构列表。
