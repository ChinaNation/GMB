# CPMS Dangan 模块技术文档

## 1. 模块定位
`backend/src/dangan/` 负责 `SFID_CPMS_V1 / ARCHIVE` 档案二维码构建与签名、公民状态校验、有效期计算、公民资料库、档案生命周期硬删除和年度状态导出。

本模块不保存实名归属判断逻辑；省市归属来自 `initialize` 保存的 INSTALL 授权材料，并只写入加密 `geo_seal`。

## 2. 负责范围
- `build_archive_qr_payload(...)`：构造 ARCHIVE 二维码。
- `validate_citizen_status(...)`：校验 `NORMAL / REVOKED`。
- `archive_valid_from(...) / archive_valid_until(...) / archive_validity_years(...)`：计算电子护照有效期。
- `materials::router(...)`：提供公民资料库上传、列表、下载和删除接口。
- `lifecycle::run_due_archive_hard_delete(...)`：硬删除软删除满 100 年的档案资料，并释放档案号和护照号。
- `export::build_and_record_cpms_status_export(...)`：生成 CPMS 给 SFID 手工导入的年度状态更新文件，并记录导出批次。

档案号和护照号生成统一归属 `memory/05-modules/cpms/backend/number/NUMBER_TECHNICAL.md`
与 `cpms/backend/src/number/`。

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

二维码明文字段不得出现 `sfid_number / province_code / city_code`。归属密文 `geo_seal` 只加密 `sfid_number`，由 SFID 根据安装授权中的 `install_secret` 解密。
ARCHIVE 不包含 `code_id` 或使用次数；重复绑定由 SFID 的 `archive_no / sfid_code / wallet_pubkey` 三者唯一关系约束。

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
- 本模块只提供 ARCHIVE 算法和电子护照有效期规则。
- 公民资料库文件存储、元数据和硬删除清理归属本模块；`operator_admin` 只负责页面入口和档案管理调用。
- 安装材料读取由 `initialize` 提供。
- 业务权限和请求校验由 `operator_admin` / `authz` 负责。

## 7. 档案生命周期

- 公民状态只允许 `NORMAL`（正常）和 `REVOKED`（注销）。
- 正常公民允许 `voting_eligible=true/false`；注销公民必须 `voting_eligible=false`。
- 公民档案详情页删除按钮对应注销软删除，后端保存 `status = DELETED`、`citizen_status = REVOKED`、`voting_eligible = false`、`deleted_at`、`deleted_by`、`delete_reason`。
- 软删除后的 100 年内，档案号和护照号仍在 `archives` 表中占用，不进入生成池。
- 从 `deleted_at` 的 UTC 日期起满 100 年后，`lifecycle` 在服务启动时和每日后台任务中扫描到期档案。
- 硬删除使用单事务：锁定到期档案、写入 `archive_number_recycle_pool`、写入 `archive_hard_delete_logs`、清理删除挑战和打印记录、物理删除 `archives` 行，并清理该档案的资料库文件目录。
- `archive_number_recycle_pool` 只对 `used_at IS NULL` 的档案号和护照号建立唯一约束；同一号码复用后再次满 100 年硬删除，可以生成新的历史池项。
- `archive_hard_delete_logs` 只保存 `source_archive_id / archive_no / passport_no / deleted_at / hard_deleted_at / reason`，不保存姓名、出生日期、地址等实名原文。
- SFID 号不在 CPMS 回收；CPMS 通过年度导出向 SFID 更新档案号状态、公民状态和投票状态。

## 8. 公民资料库

- 公民资料库命名为“公民资料库”，在公民档案详情页下半部分展示。
- 后端入口固定在 `backend/src/dangan/materials.rs`，路由由 `dangan::router()` 挂载；不得把资料库存储和生命周期逻辑放进 `operator_admin`。
- 资料类型固定为 `PHOTO / BIRTH_CERTIFICATE / COPY / VIDEO / OTHER`，分别对应照片、出生纸、复印件、视频和其他资料。
- `archive_materials` 只保存元数据：资料类型、原始文件名、本机存储文件名、MIME、大小、SHA-256、备注、上传人、上传时间和软删除字段。
- 文件正文默认保存到 `data/archive-materials/<archive_id>/`；部署时可用 `CPMS_MATERIALS_DIR` 指向专用资料盘。
- 单文件上限 100 MB；后端按资料类型校验 MIME，拒绝类型不匹配或空文件。
- 软删除档案可以查看和下载已有资料，不能新增或删除资料；100 年硬删除档案时同步删除资料目录。
- 上传、下载、删除分别记录 `ARCHIVE_MATERIAL_UPLOAD / ARCHIVE_MATERIAL_DOWNLOAD / ARCHIVE_MATERIAL_DELETE` 审计事件。

## 9. 年度状态导出

- 导出文件类型固定为 `SFID_CPMS_V1 / CPMS_STATUS_EXPORT`。
- 导出只生成离线 JSON 文件内容，不进行联网推送。
- 超级管理员从每年 UTC 1 月 1 日起可导出上一年度数据；如果存在多年未导出，系统按最早未导出年度依次补导。
- 首个需要导出的年度从 `system_install.initialized_at` 所在年份开始，避免新装系统误要求历史年度。
- 年度报告不再在 UTC 1 月 10 日后关闭导出窗口；只要存在待导出年度，超级管理员一直可以导出。
- UTC 每年 1 月 11 日起，如果存在已超过 1 月 10 日仍未导出的年度报告，`OPERATOR_ADMIN` 登录和已有会话都会被锁定；超级管理员不受影响，必须先补导年度报告。
- `GET /api/v1/archives/status-export/state` 返回待导出年度、按钮可用状态、角标状态和操作管理员锁定状态，供前端系统设置页展示。
- `cpms_status_exports` 记录每个年度首次导出的批次、导出时间、记录数量、`records_hash` 和完整已签名 JSON；重复点击导出时返回同一份文件，不重新生成签名批次。
- `status_records` 只包含 `archive_no / citizen_status / voting_eligible / status_updated_at`，用于 SFID 更新公民状态和投票资格。
- `number_release_records` 只包含 `archive_no / passport_no / hard_deleted_at`，用于表达 100 年硬删除后号码可复用；该列表不表示公民状态变化。
- 导出文件不得包含姓名、出生日期、地址、钱包地址、钱包公钥等实名或绑定细节。
- 导出文件使用 CPMS ARCHIVE 签发密钥签名，签名原文为 `sfid-cpms-v1|cpms-status-export|{sfid_number}|{cpms_pubkey}|{export_batch_id}|{exported_at}|{records_hash}`。

## 10. 测试覆盖

- 普通 `cargo test` 覆盖 100 年截止日按 UTC 日期计算，避免用固定秒数近似导致闰年边界错误。
- 设置 `CPMS_TEST_DATABASE_URL` 后，数据库集成测试会套用当前 `db/schema.sql`，覆盖软删除未满 100 年不硬删除、不入回收池；软删除满 100 年后物理删除档案、写入号码回收池并写入硬删除日志。
- 生命周期逻辑必须继续保持单事务语义：号码回收池或硬删除日志写入失败时，不得删除 `archives` 行。
- 公民资料库接口需覆盖上传类型/MIME 校验、软删除档案禁止新增或删除、下载审计和硬删除清理资料目录。
- 状态导出测试覆盖导出哈希稳定性、签名原文格式、最早未导出年度选择、1 月 11 日锁定边界和注销状态无投票资格。
