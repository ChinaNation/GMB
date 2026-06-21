# CPMS Dangan 模块技术文档

## 1. 模块定位
`backend/dangan/` 负责 CPMS 档案业务：档案创建/查询、列表游标分页、`SFID_CPMS_V1 / ARCHIVE` 档案二维码构建与签名、公民状态校验、有效期计算、公民资料库、档案操作记录聚合、档案生命周期硬删除和年度状态导出。

本模块不保存实名归属判断逻辑；CPMS 所属省市来自 `initialize` 保存的 INSTALL 授权材料。公民居住地由该省市 + 镇 + 地址段 + 详细地址输入段组成；出生地由全国省、市、镇组成，来自 CPMS 随包的 SFID 行政区唯一真源 `china.sqlite` 只读拷贝，保存后不得在编辑接口中修改。

## 2. 负责范围
- `build_archive_qr_payload(...)`：构造 ARCHIVE 二维码。
- `clear_archive_qr_payload(...)`：统一清空旧 ARCHIVE 二维码。
- `validate_citizen_status(...)`：校验 `NORMAL / REVOKED`。
- `effective_voting_eligible(...) / resolve_voting_eligible(...)`：按公民状态和 16 周岁年龄线计算或校验选举资格。
- `archive_valid_from(...) / archive_valid_until(...) / archive_validity_years(...)`：计算电子护照有效期。
- `routes::router(...)`：提供档案创建、详情、编辑、投票账户绑定、删除签名、打印、操作记录和列表接口。
- `routes::ensure_archive_qr_ready(...)`：生成或打印 ARCHIVE 前校验档案完整性。
- `routes::list_archives(...)`：使用 `created_at DESC, archive_id DESC` 游标分页返回档案列表。
- `routes::list_archive_audit_logs(...)`：按档案 ID、档案号和审计 detail 聚合档案操作记录。
- `stats::adjust_archive_stats(...)`：在档案创建和注销软删除事务中维护 `archive_stats`。
- `materials::router(...)`：提供公民资料库上传、列表、下载和删除接口。
- `lifecycle::run_due_archive_hard_delete(...)`：硬删除软删除满 100 年的档案资料，并释放档案号和护照号。
- `export::build_and_record_cpms_status_export(...)`：生成 CPMS 给 SFID 手工导入的年度状态更新文件，并记录导出批次。

档案号和护照号生成统一归属 `memory/05-modules/cpms/backend/number/NUMBER_TECHNICAL.md`
与 `cpms/backend/number/`。

## 3. 有效期规则

- `valid_from` 使用创建档案当天的 UTC 日期。
- 创建档案当天未满 16 周岁时，`valid_until` 为 5 周年前一天。
- 创建档案当天已满 16 周岁时，`valid_until` 为 10 周年前一天。
- 生日当天视为已满对应周岁。

## 4. ARCHIVE 载荷

```json
{
  "proto": "SFID_CPMS_V1",
  "type": "ARCHIVE",
  "archive_no": "K8M4ZP7W2Q1C9T6R5N3X8V2Y1A-7H",
  "citizen_status": "NORMAL",
  "voting_eligible": true,
  "valid_from": "2026-05-24",
  "valid_until": "2036-05-23",
  "status_updated_at": 1779580800,
  "cpms_pubkey": "0x...",
  "geo_seal": "g1.<nonce_hex>.<cipher_hex>",
  "wallet_address": "5...",
  "wallet_pubkey": "0x...",
  "wallet_sig_alg": "sr25519",
  "sig": "0x..."
}
```

二维码明文字段不得出现 `sfid_number / province_code / city_code / town_code` 或中文行政区名称。归属密文 `geo_seal` 加密 CPMS 机构 `sfid_number`、居住地代码、出生地代码和 `election_scope_level`，由 SFID 根据安装授权中的 `install_secret` 解密。
ARCHIVE 不包含 `code_id` 或使用次数；重复绑定由 SFID 的 `archive_no / sfid_number / wallet_pubkey` 三者唯一关系约束。
生成和打印 ARCHIVE 前必须满足完整性门槛：姓氏、名字、性别、身高、出生日期、护照号、有效期、居住省市、出生省市镇、公民状态、选举资格、投票账户、照片和出生纸齐全；公民状态必须为 `NORMAL`，选举资格必须为 `true`，照片和出生纸各至少 1 张。

`geo_seal` 明文结构只在 CPMS 签发端与 SFID 解密端存在：

```json
{
  "sfid_number": "GD001-GZF06-123456789-2026",
  "residence": { "province_code": "GD", "city_code": "001", "town_code": null },
  "birthplace": { "province_code": "GD", "city_code": "001", "town_code": null },
  "election_scope_level": "CITY"
}
```

- `election_scope_level=PROVINCE`：居住地和出生地只写 `province_code`，`city_code / town_code` 为空。
- `election_scope_level=CITY`：居住地和出生地写 `province_code / city_code`，`town_code` 为空。
- `election_scope_level=TOWN`：居住地和出生地写 `province_code / city_code / town_code`；前端在“设置投票账户”弹窗打开“注册镇选举公民”时必须同时打开“注册市选举公民”。

## 5. 签名与加密
- `geo_seal` 使用 AES-256-GCM。
- `geo_seal` 密钥：`blake2b_256(install_secret)`。
- `geo_seal` AAD：`sfid-cpms-v1|geo-seal|{archive_no}|{cpms_pubkey}`。

- ARCHIVE 签名原文：

```text
sfid-cpms-v1|archive|{archive_no}|{citizen_status}|{voting_eligible}|{valid_from}|{valid_until}|{status_updated_at}|{cpms_pubkey}|{geo_seal_hash}|{wallet_address}|{wallet_pubkey}
```

- ARCHIVE 签名上下文：`substrate`。

## 6. 模块边界
- 本模块承载档案领域业务；操作员只是 `authz` 中的一种角色，不得作为档案业务实现模块。
- 公民资料库文件存储、元数据和硬删除清理归属本模块。
- 安装材料读取由 `initialize` 提供。
- 业务权限由 `authz::require_archive_admin` 统一校验，允许 `ADMIN / OPERATOR` 调用档案接口。

## 6.1 列表分页与检索

- `GET /api/v1/archives` 不接收 `page / page_size / q` 参数，也不接收 `archive_no / passport_no / name` 选择器式查询参数。
- 默认 `limit=50`，最大 `100`；前端可选 `20 / 50 / 100`。
- 翻页使用不透明 cursor，cursor 内部只包含排序所需的 `created_at / archive_id`。
- SQL 固定按 `created_at DESC, archive_id DESC` 排序，并用 `(created_at, archive_id) < (cursor_created_at, cursor_archive_id)` 取下一页。
- 响应字段固定为 `items / limit / next_cursor / has_next / total_active`，不返回 `total_pages`。
- 前端列表固定显示“序号、档案号、姓名、性别、年龄、市镇、公民状态、创建时间”，整行点击进入详情，不保留单独操作列。
- `total_active` 读取 `archive_stats.active_count`；列表请求不得实时 `COUNT(*) FROM archives`。
- 统一检索字段为 `search`，后端精确匹配 `archive_no = search OR passport_no = search OR (last_name || first_name) = search`，禁止恢复前端字段选择器。
- 检索字段只允许 `search / birth_date / town_code / address_unit_id / citizen_status` 的索引化精确过滤，禁止恢复 `%keyword% LIKE` 全表模糊搜索。

## 7. 档案生命周期

- 公民状态只允许 `NORMAL`（正常）和 `REVOKED`（注销）。
- 正常且已满 16 周岁的公民允许 `voting_eligible=true/false`；注销公民或未满 16 周岁的公民必须 `voting_eligible=false`。
- 创建/编辑档案、修改公民状态和年度导出时，后端按同一规则计算选举资格；前端只允许正常且已满 16 周岁的公民选择“有选举资格”。
- 出生日期必须早于当前 UTC 日期；当天出生和未来日期都不得录入，保存后编辑档案接口不得接收或改写 `birth_date`。
- 出生地在创建档案时必填 `birth_province_code / birth_city_code / birth_town_code`，从 CPMS 随包的 SFID 行政区真源只读接口选择；保存后编辑档案接口不得接收或改写出生地字段。
- 居住地址保存 `town_code / address_unit_id / address_detail / address_full_snapshot`；`address_unit_id` 必须属于当前安装城市和所选镇，`address_detail` 是管理员录入的自由详细地址段。
- 公民详情页投票账户下方只读展示“注册市选举公民 / 注册镇选举公民”的结果；修改入口只在“设置投票账户”弹窗内。
- 只有 `voting_eligible=true` 的公民允许设置投票账户；前端必须禁用投票账户输入和扫码操作，后端 `设置投票账户` 接口必须拒绝无选举资格或非正常状态档案。
- “设置投票账户”接口同时保存 `wallet_address` 和 `election_scope_level`，并清空旧 `archive_qr_payload`，等待重新签发 ARCHIVE。
- 前端列表搜索和创建出生日期统一使用 `cpms/frontend/components/DateInput.tsx`，
  页面内不得散落直接的日期输入实现。
- 公民档案详情页删除按钮对应注销软删除，后端保存 `status = DELETED`、`citizen_status = REVOKED`、`voting_eligible = false`、`deleted_at`、`deleted_by`、`delete_reason`。
- 编辑实名字段、修改公民状态、设置投票账户、上传资料或删除资料会调用 `clear_archive_qr_payload(...)` 清空旧 `archive_qr_payload`；旧档案码不得继续作为当前档案状态展示。
- 软删除后的 100 年内，档案号和护照号仍在 `archives` 表中占用，不进入生成池。
- 从 `deleted_at` 的 UTC 日期起满 100 年后，`lifecycle` 在服务启动时和每日后台任务中扫描到期档案。
- 硬删除使用单事务：锁定到期档案、写入 `archive_number_recycle_pool`、写入 `archive_hard_delete_logs`、清理删除挑战和打印记录、物理删除 `archives` 行，并清理该档案的资料库文件目录。
- `archive_number_recycle_pool` 只对 `used_at IS NULL` 的档案号和护照号建立唯一约束；同一号码复用后再次满 100 年硬删除，可以生成新的历史池项。
- `archive_hard_delete_logs` 只保存 `source_archive_id / archive_no / passport_no / deleted_at / hard_deleted_at / reason`，不保存姓名、出生日期、地址等实名原文。
- SFID 号不在 CPMS 回收；CPMS 通过年度导出向 SFID 更新档案号状态、公民状态和投票状态。

## 8. 公民资料库

- 公民资料库命名为“公民资料库”，在公民档案详情页“资料库”左侧导航 tab 展示。
- 后端入口固定在 `backend/dangan/materials.rs`，路由由 `dangan::router()` 挂载；不得把资料库存储和生命周期逻辑放进角色模块。
- 资料类型固定为 `PHOTO / BIRTH_CERTIFICATE / COPY / VIDEO / OTHER`，分别对应照片、出生纸、复印件、视频和其他资料。
- `archive_materials` 只保存元数据：资料类型、原始文件名、本机存储文件名、MIME、大小、SHA-256、备注、上传人、上传时间和软删除字段。
- 开发默认文件正文保存到 `data/archive-materials/<archive_id>/`；正式离线安装包固定设置
  `CPMS_MATERIALS_DIR=/var/lib/cpms/materials`，备份脚本必须同步备份该目录。
- 单文件上限 100 MB；后端按资料类型校验 MIME，拒绝类型不匹配或空文件。
- 上传入口增加本机 IP 级限流，避免内网脚本误传或连续大文件请求压垮主机。
- 前端资料上传只通过“上传”按钮打开弹窗，弹窗内展示资料类型、文件、备注和提交按钮；资料库卡片标题区不重复显示数量。
- 软删除档案可以查看和下载已有资料，不能新增或删除资料；100 年硬删除档案时同步删除资料目录。
- 上传、下载、删除分别记录 `ARCHIVE_MATERIAL_UPLOAD / ARCHIVE_MATERIAL_DOWNLOAD / ARCHIVE_MATERIAL_DELETE` 审计事件。
- 上传或删除照片、出生纸等公民资料会清空旧 `archive_qr_payload`；重新更新档案码时仍要求照片和出生纸各至少 1 张。

## 9. 操作记录

- 公民档案详情页固定提供“操作记录”tab，前端入口为 `cpms/frontend/dangan/ArchiveDetail.tsx`，后端入口为 `GET /api/v1/archives/:archive_id/audit-logs`。
- 操作记录数据来源只允许使用 CPMS 本机 `audit_logs`；不得读取或依赖外部系统日志。
- 审计记录按 `target_id = archive_id`、`detail->>'archive_id' = archive_id`、`detail->>'archive_no' = archive_no` 聚合，并按 `created_at DESC` 返回最近 100 条。
- 接口返回 `operator_account`，由 `operator_user_id` 关联 `admin_users.admin_pubkey` 后转换为管理员账户地址；前端表格列固定为“操作、操作者账户、详情、时间”。
- 查询操作记录是只读行为，不额外写入审计，避免打开详情页本身制造噪声。

## 10. 年度状态导出

- 导出文件类型固定为 `SFID_CPMS_V1 / CPMS_STATUS_EXPORT`。
- 导出只生成离线 JSON 文件内容，不进行联网推送。
- 管理员从每年 UTC 1 月 1 日起可导出上一年度数据；如果存在多年未导出，系统按最早未导出年度依次补导。
- 首个需要导出的年度从 `system_install.initialized_at` 所在年份开始，避免新装系统误要求历史年度。
- 年度报告不再在 UTC 1 月 10 日后关闭导出窗口；只要存在待导出年度，管理员一直可以导出。
- UTC 每年 1 月 11 日起，如果存在已超过 1 月 10 日仍未导出的年度报告，`OPERATOR` 登录和已有会话都会被锁定；管理员不受影响，必须先补导年度报告。
- `GET /api/v1/archives/status-export/state` 返回待导出年度、按钮可用状态、角标状态和操作员锁定状态，供前端系统设置页展示。
- `cpms_status_exports` 记录每个年度最近一次导出的批次、导出时间、绑定记录数量、释放记录数量、`records_hash` 和完整已签名 JSON；重复点击导出时必须从当前档案数据重新生成并覆盖同年度记录。
- `citizen_binding_records` 是当前仍有钱包绑定的档案快照，包含 `archive_no / wallet_address / wallet_pubkey / wallet_sig_alg / wallet_bound_at / citizen_status / voting_eligible / status_updated_at`，用于 SFID 按档案号覆盖本地绑定状态；`voting_eligible` 导出前按公民状态和 16 周岁年龄线重新计算。
- `binding_release_records` 只包含 `archive_no / released_at / release_reason`，用于表达 100 年硬删除后 SFID 可以释放该档案号与身份 ID、钱包地址的绑定关系。
- 导出文件不得包含姓名、出生日期、地址、护照号等实名或 CPMS 内部号码。
- 导出文件使用 CPMS ARCHIVE 签发密钥签名，签名原文为 `sfid-cpms-v1|cpms-status-export|{sfid_number}|{cpms_pubkey}|{export_batch_id}|{exported_at}|{records_hash}`。

## 11. 测试覆盖

- 普通 `cargo test` 覆盖 100 年截止日按 UTC 日期计算，避免用固定秒数近似导致闰年边界错误。
- 设置 `CPMS_TEST_DATABASE_URL` 后，数据库集成测试会套用当前 `db/schema.sql`，覆盖软删除未满 100 年不硬删除、不入回收池；软删除满 100 年后物理删除档案、写入号码回收池并写入硬删除日志。
- 生命周期逻辑必须继续保持单事务语义：号码回收池或硬删除日志写入失败时，不得删除 `archives` 行。
- 公民资料库接口需覆盖上传类型/MIME 校验、软删除档案禁止新增或删除、下载审计和硬删除清理资料目录。
- 操作记录接口需覆盖档案 ID、档案号和资料操作 detail 的聚合边界。
- 状态导出测试覆盖导出哈希稳定性、签名原文格式、最早未导出年度选择、1 月 11 日锁定边界和注销状态无投票资格。
