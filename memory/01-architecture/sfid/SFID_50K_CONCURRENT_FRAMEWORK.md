# SFID 5 万并发框架

## 当前结论

SFID 的高并发目标必须建立在结构化数据库、索引和省市范围分区之上。业务主数据只落 PostgreSQL,应用进程不保存第二套主数据。

## 负载来源

高峰主要来自注册局工作人员集中使用浏览器后台:

- 联邦注册局机构 admins 查询本省公权机构、私权机构、CPMS 和审计。
- 市注册局机构 admins 查询本市机构、公民绑定和资料库。
- citizenapp 公开查询电子护照状态、机构身份和投票凭证。

## 数据分区策略

从第一版目标结构开始按 `province_code` 做省分区:

- `subjects`
- `citizens`
- `gov`
- `private`
- `accounts`
- `docs`
- `audit`

广州注册局机构管理员查询广州数据时,SQL 必须携带 `province_code='GD' AND city_code='<广州代码>'`。联邦注册局机构 admins 查询本省数据时,SQL 必须携带 `province_code`。禁止全量读取后过滤。

## 索引策略

必备索引:

- `subjects(province_code, city_code, kind, status, sfid_number)`
- `subjects(province_code, city_code, name)`
- `citizens(province_code, city_code, created_at DESC, id DESC)`
- `citizens(province_code, city_code, archive_no, sfid_number, wallet_pubkey, wallet_address)`
- `gov(province_code, city_code, town_code, institution_code)`
- `gov(province_code, city_code, town_code, org_code)`
- `private(province_code, city_code, kind, code)`
- `accounts(province_code, sfid_number)`
- `docs(province_code, sfid_number, uploaded_at DESC)`
- `audit(province_code, city_code, created_at DESC)`
- `admins(registry_org_code, city_name)`
- `admins(lower(admin_account))`
- `admins(lower(created_by))`
- `federal_registry_scope(province_name)`

## 启动策略

`sfid-backend serve` 只做 schema 初始化、分区创建、内置联邦注册局机构管理员初始化和后台索引 worker 启动。确定性公权机构目录不得在每次服务启动时全量写入。

部署流程必须把确定性目录初始化从普通服务启动中拆开:部署入口先执行 `ensure-gov`,已初始化且完整则跳过,缺失或不完整才初始化;随后再启动 `serve`。页面列表只读已持久化数据,不得同步触发全量对账。

自动生成机构采用显式对账:

1. 根据 `china` 行政区划和宪法常量生成目标目录。
2. 对目标范围做差异写入。
3. 只更新变化的行政区或机构。
4. 保持已有 `sfid_number` 不变。

## 查询策略

后台列表:

- 公权机构:按 `gov + subjects + accounts` 查询确定性列表。
- 私权机构:按 `private + subjects` 查询注册局新增数据。
- 公民:按 `citizens` 精确查询或分页查询。
- CPMS:按 `cpms_sites` 查询授权状态。
- 资料库:按 `docs` 查询机构资料。
- 注册局管理员目录:联邦注册局机构 admins 按 `federal_registry_scope.province_name` 查询,市注册局机构 admins 按 `created_by` 归属省和 `city_name` 查询。禁止查询全部管理员后在 Rust 或前端过滤。

公开接口:

- 机构搜索和详情读取 `subjects/accounts`。
- SFID 不提供清算行搜索;清算行属于链上组织治理概念。
- 投票人数快照使用 `citizens` 聚合计数。
- 投票凭证和电子护照状态按钱包公钥精确查询。

## 扩展策略

当单省数据继续增长时:

- 优先补充组合索引和只读副本。
- 高频公开查询可增加 Redis 短 TTL 缓存,缓存只保存查询结果,不得成为主数据。
- 写操作保持 PostgreSQL 事务边界,不引入双写旧格式。

## 验收要求

- 后端源码不得出现旧聚合快照入口。
- 架构文档不得继续描述旧快照表为目标方案。
- 所有省市范围列表必须在 SQL 层带范围条件。
- `cargo check` 和 `cargo check --tests` 必须通过。
