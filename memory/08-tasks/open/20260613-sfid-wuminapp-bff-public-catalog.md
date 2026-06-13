# 任务卡:SFID 新建 wuminapp BFF 目录 + 公权机构公开目录接口(跨模块前置)

属 ADR-018 §九(2026-06-13 混合模式修订)。**公权机构界面的跨模块关键路径,wuminapp 卡 A/B/C 全部依赖本卡先落。**

状态:**代码完工(2026-06-13,v1)**。`sfid/backend/wuminapp/` BFF 落地:`list_public_institutions` + `public_institutions_version` 两个匿名 handler 挂 app_routes;公开 DTO `PublicInstitutionRow` 白名单(显式丢 created_by_name/created_by_role/cpms_status/install_token_status/identity_service_status/private_type/partnership_kind);复用 `Db::list_official_institutions_scope` 领域查询不改;`custom_account_names` 批量单查 accounts 表(非 5 保留名,保留名数组从 accounts/derive.rs 暴露单一源);manifest_version 复用 gov::service。cargo build 干净 + 安全白名单单测 2/2(断言敏感字段不泄露)。

**v1 范围决策(card 原列与实现的差异,已对齐)**:
- `since_version` 行级增量**暂不实现**:v1 同步 = version 接口比对各省 manifest_version,变化则**全量重拉该省**(省份有界、确定性目录极少变),足够且零行级游标复杂度。真行级 delta 留 v1.x。
- 独立导出 CLI **不单做**:card A 数据包生成器直接调 `list_public_institutions` 分页全量拉取即省导出,无需另写 CLI。
- OpenAPI 契约见下「契约」节(card A 接 mock 依据)。

## 背景
- 现有 `/api/v1/institutions/official` 是**管理员专用**(`require_admin_any`),wuminapp 公民端拉不到。
- SFID backend 已有"非 admin 路由组"(main.rs:2301,钱包交易索引 indexer::api + 电子护照状态 citizens/),但 handler 散落、无统一归口。
- 公权机构目录数据来自 SFID 自己的 Postgres(subjects + gov + accounts 表),是**确定性目录**,gov/service.rs 已有生成器 + reconcile 维护 `manifest_version`。**与链交互无关**(不碰 core/chain_* 与 indexer/worker)。

## 架构决定
- 新建 `sfid/backend/wuminapp/` 作为 **BFF(Backend-For-Frontend)**:只放薄 handler + DTO + 路由组装,挂进现有非 admin 路由组(匿名只读 + 复用 global_rate_limit_middleware + CORS)。
- **领域逻辑留在 gov/**:扩 `gov/service.rs` 的目录查询,顺带 emit `custom_account_names` + 稳定 `catalog_version` + 各省 `manifest_version`。`wuminapp/` 调 gov,不复制 gov 逻辑。
- 安全铁律:只透出白名单公开字段,**严禁**带 created_by / cpms token / passkey 等管理员/PII 字段。审计点=整个 `wuminapp/` 目录即匿名可读边界。

## 完工清单
- [ ] 新建 `sfid/backend/wuminapp/`(mod + handler + dto),wire 进 main.rs 非 admin 路由组。
- [ ] `GET /api/v1/wuminapp/public-institutions/version` → `{ catalog_version, provinces:[{province, manifest_version}] }`,极小载荷、匿名、可缓存。
- [ ] `GET /api/v1/wuminapp/public-institutions?province=&city=&since_version=&cursor=&page_size=`:
  - 无 `since_version` = 全量(按省/市 scope 分页);带 `since_version` = 增量(返回 upsert 行 + 独立 `deleted:[sfid_number]` 列表)。
  - 行字段(公开白名单):`sfid_number / institution_name / sfid_name / short_name / province / city / town / institution_code / org_code / status / account_count / custom_account_names[]`。
  - **`custom_account_names`**:gov/service.rs 同一查询触碰 accounts 表时顺带收 op_tag=0x06 的账户名列表,绝大多数机构为空数组(近零成本)。主/费(0x00/0x01)不带,客户端本地派生。
- [ ] 导出能力:CLI 或 admin 命令,把全量目录 dump 成 JSON(喂 wuminapp 卡 A 数据包生成器),复用同一查询。
- [ ] `catalog_version` 单调递增定义(reconcile 时间戳或序列),各省 `manifest_version` 复用现有。
- [ ] OpenAPI 契约文档(分页游标 + scope + 增量语义 + 字段),供 wuminapp 卡 A 接 mock 先行。

## 单测
- [ ] 匿名访问通过、无 token 不被 require_admin 拦。
- [ ] 公开字段白名单:断言响应不含 created_by / token / passkey 等敏感字段。
- [ ] 增量:since_version 只回变化行 + deleted 列表;custom_account_names 空/非空两路。
- [ ] scope:按 province/city 过滤正确。

## 验收
- [ ] cargo build + cargo test 全过;新接口匿名可拉,管理员接口不受影响。
- [ ] 与 wuminapp 卡 A 联调:全量载入 + 一次增量同步闭环。
- [ ] 旧代码/文档/注释清理无残留。

## 契约(card A 接 mock 依据,已落地)
- `GET /api/v1/app/public-institutions?province=<必填>&city=&q=&org_code=&cursor=<offset 整数>&page_size=<1..300,默认300>`
  → `ApiResponse{code,message,data: PageResult<PublicInstitutionRow>}`,`PageResult` 含 `items/page_size/next_cursor/has_more/manifest_version/catalog_status`。
  `PublicInstitutionRow{sfid_number, institution_name?, sfid_name?, short_name?, status, category, subject_property, p1, province, city, town, institution_code, org_code?, has_legal_personality?, parent_sfid_number?, account_count, custom_account_names[], created_at}`。
- `GET /api/v1/app/public-institutions/version?province=<必填>&city=` → `data:{province, city?, manifest_version?}`。
- 匿名(非 admin),挂 global_rate_limit + CORS;province/city 用中文名,后端 province_code_by_name/city_code_by_name 解析,未知名 400。

## 不做(边界)
- 不碰 `core/chain_*`、`indexer/worker`(链交互)。
- 不迁移现有钱包交易索引/电子护照/清算行搜索接口(那是**单列重构卡**:把散落的 wuminapp-facing handler 分期搬进 `wuminapp/`,行为中性、不阻塞本卡)。
- 不动 gov 管理员接口口径。

## 改动目录(中文注释)
- 新增 `sfid/backend/wuminapp/`:wuminapp BFF 薄层(handler/dto/route),代码。
- 改 `sfid/backend/gov/service.rs`:目录查询扩 custom_account_names + catalog_version,代码。
- 改 `sfid/backend/main.rs`:wire 新路由进非 admin 组,代码。
- 文档:新增 OpenAPI 契约 + 中文注释。
