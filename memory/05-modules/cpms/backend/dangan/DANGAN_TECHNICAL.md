# CPMS Dangan 模块技术文档

## 1. 模块定位
`backend/src/dangan/` 负责 `SFID_CPMS_V1 / ARCHIVE` 档案二维码构建与签名、公民状态校验、有效期计算、档案生命周期硬删除和年度状态导出。

本模块不保存实名归属判断逻辑；省市归属来自 `initialize` 保存的 INSTALL 授权材料，并只写入加密 `geo_seal`。

## 2. 负责范围
- `build_archive_qr_payload(...)`：构造 ARCHIVE 二维码。
- `validate_citizen_status(...)`：校验 `NORMAL / REVOKED`。
- `archive_valid_from(...) / archive_valid_until(...) / archive_validity_years(...)`：计算电子护照有效期。
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
- 安装材料读取由 `initialize` 提供。
- 业务权限和请求校验由 `operator_admin` / `authz` 负责。

## 7. 档案生命周期

- 公民状态只允许 `NORMAL`（正常）和 `REVOKED`（注销）。
- 正常公民允许 `voting_eligible=true/false`；注销公民必须 `voting_eligible=false`。
- 公民档案详情页删除按钮对应注销软删除，后端保存 `status = DELETED`、`citizen_status = REVOKED`、`voting_eligible = false`、`deleted_at`、`deleted_by`、`delete_reason`。
- 软删除后的 100 年内，档案号和护照号仍在 `archives` 表中占用，不进入生成池。
- 从 `deleted_at` 的 UTC 日期起满 100 年后，`lifecycle` 在服务启动时和每日后台任务中扫描到期档案。
- 硬删除使用单事务：锁定到期档案、写入 `archive_number_recycle_pool`、写入 `archive_hard_delete_logs`、清理删除挑战和打印记录、物理删除 `archives` 行。
- `archive_hard_delete_logs` 只保存 `source_archive_id / archive_no / passport_no / deleted_at / hard_deleted_at / reason`，不保存姓名、出生日期、地址等实名原文。
- SFID 号不在 CPMS 回收；CPMS 后续通过年度导出向 SFID 更新档案号状态、公民状态和投票状态。

## 8. 年度状态导出

- 导出文件类型固定为 `SFID_CPMS_V1 / CPMS_STATUS_EXPORT`。
- 导出只生成离线 JSON 文件内容，不进行联网推送。
- 年度报告只能由超级管理员在每年 UTC 1 月 1 日到 1 月 10 日导出，导出内容为上一年度的更新数据。
- UTC 1 月 6 日到 1 月 10 日，如果上一年度仍未导出，`OPERATOR_ADMIN` 登录和已有会话都会被锁定；超级管理员不受影响，必须先导出年度报告。
- `cpms_status_exports` 记录每个年度最近一次导出批次、导出时间、记录数量和 `records_hash`，用于判断操作管理员是否需要锁定。
- `status_records` 只包含 `archive_no / citizen_status / voting_eligible / status_updated_at`，用于 SFID 更新公民状态和投票资格。
- `number_release_records` 只包含 `archive_no / passport_no / hard_deleted_at`，用于表达 100 年硬删除后号码可复用；该列表不表示公民状态变化。
- 导出文件不得包含姓名、出生日期、地址、钱包地址、钱包公钥等实名或绑定细节。
- 导出文件使用 CPMS ARCHIVE 签发密钥签名，签名原文为 `sfid-cpms-v1|cpms-status-export|{sfid_number}|{cpms_pubkey}|{export_batch_id}|{exported_at}|{records_hash}`。

## 9. 测试覆盖

- 普通 `cargo test` 覆盖 100 年截止日按 UTC 日期计算，避免用固定秒数近似导致闰年边界错误。
- 设置 `CPMS_TEST_DATABASE_URL` 后，数据库集成测试会套用当前 `db/schema.sql`，覆盖软删除未满 100 年不硬删除、不入回收池；软删除满 100 年后物理删除档案、写入号码回收池并写入硬删除日志。
- 生命周期逻辑必须继续保持单事务语义：号码回收池或硬删除日志写入失败时，不得删除 `archives` 行。
- 状态导出测试覆盖导出哈希稳定性、签名原文格式和注销状态无投票资格。
