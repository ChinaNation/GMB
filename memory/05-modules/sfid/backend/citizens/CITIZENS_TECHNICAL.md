# CITIZENS 模块技术文档

- 最后更新:2026-05-31
- 任务卡:
  - `memory/08-tasks/done/20260530-sfid-admin-permission-step2.md`

## 1. 模块定位

- 路径：`sfid/backend/citizens`
- 职责：承载公民电子护照绑定、CPMS 状态扫码、公民投票凭证签发和联合投票人口快照凭证签发。
- 电子护照绑定边界：CPMS 档案码提供 `archive_no / citizen_status / voting_eligible / valid_from / valid_until / status_updated_at / wallet_address / wallet_pubkey / wallet_sig_alg`；SFID 验档案码后生成 `WUMIN_QR_V1 / sign_request`；wuminapp 使用对应钱包签名；SFID 验签通过后直接写入本地绑定结果并向 wuminapp 状态接口返回。

## 2. 模块结构

- `binding.rs`
  - `citizen_bind_challenge`：验 CPMS `ARCHIVE` 档案码并签发 wuminapp 签名请求。
  - `citizen_bind`：消费管理员 Passkey grant,验 wuminapp `sign_response` 并完成电子护照绑定。
- `vote.rs`
  - `app_myid_status`：wuminapp 查询电子护照绑定状态。
- `chain_vote.rs`
  - `app_vote_credential`：公民投票凭证签发接口。
- `chain_joint_vote.rs`
  - `app_voters_count`：联合投票人口快照凭证签发接口。
- `model.rs`
  - 公民电子护照记录、`bind_status`、绑定 DTO、状态扫码 QR 载荷。
- `handler.rs`
  - `admin_list_citizens`：后台公民精确查询和游标分页。
  - `public_identity_search`：公开身份查询。
- `status.rs`
  - `admin_cpms_status_scan`：CPMS 站点扫公民状态。
- `cpms_qr.rs`
  - 状态扫码 canonical 文本拼装能力。
- `mod.rs`
  - 子模块注册入口。

## 3. 路由接线

- `POST /api/v1/admin/citizen/bind/challenge` -> `citizens::binding::citizen_bind_challenge`
- `POST /api/v1/admin/citizen/bind` -> `citizens::binding::citizen_bind`
- `GET  /api/v1/app/myid/status?wallet_address=<walletAddress>` -> `citizens::vote::app_myid_status`
- `POST /api/v1/app/vote/credential` -> `citizens::chain_vote::app_vote_credential`
- `GET  /api/v1/app/voters/count` -> `citizens::chain_joint_vote::app_voters_count`
- `POST /api/v1/admin/citizens/cpms-status-export/import` -> `citizens::status_export_import::admin_import_cpms_status_export`
- `GET  /api/v1/admin/citizens` -> `citizens::handler::admin_list_citizens`
- `GET  /api/v1/public/identity/search` -> `citizens::handler::public_identity_search`

## 4. 依赖与边界

- 依赖：
  - `cpms::verify_cpms_archive_qr`：验 CPMS 档案码和归属密文。
  - `admins::actions::require_admin_security_grant`：确认 `PASSKEY` 写操作已经通过 Passkey。
  - `login::parse_sr25519_pubkey_bytes`：解析 wuminapp 钱包公钥。
  - 全局公共能力：鉴权、审计、状态存储。
- 边界：
  - 电子护照绑定必须以 CPMS `ARCHIVE` 档案码为入口。
  - 绑定必须使用 wuminapp 对 SFID challenge 的 sr25519 签名。
- `citizens` 不实现投票流程；公民投票只调用投票凭证签发接口。
- 公民 DTO 放 `citizens/model.rs`，不得塞入全局 `models`。
- CPMS 年度报告导入实现放 `citizens/status_export_import.rs`，不再保留旧 CPMS 状态扫码入口。

## 5. 关键一致性约束

- 三端字段统一：`archive_no / citizen_status / voting_eligible / vote_status / identity_status / valid_from / valid_until / status_updated_at / wallet_address / wallet_pubkey / wallet_sig_alg / sfid_number / bind_status`。
- `bind_status` 只表达电子护照绑定状态：`PENDING / BOUND`；`identity_status` 表达身份 ID 当前有效状态；`vote_status` 由 `citizen_status + voting_eligible` 计算。
- `citizen_status` 当前只允许 `NORMAL / REVOKED`；`REVOKED` 表示 CPMS 软删除注销，必须对应 `voting_eligible=false`。
- CPMS 年度 `CPMS_STATUS_EXPORT` 导入时，`citizen_binding_records` 按 `archive_no` 覆盖已有 SFID 绑定记录的钱包地址、公民状态和投票资格，但不自动生成新的身份 ID；`binding_release_records` 用于释放档案号、身份 ID、钱包地址三者绑定关系，不处理 CPMS 护照号。
- SFID 导入年度报告前必须校验 CPMS 授权处于 `ACTIVE`、CPMS 公钥已经由档案码验真绑定、`records_hash` 与签名均正确；同一 CPMS 同一年度只允许导入相同 `records_hash`。
- 后台公民精确查询、绑定 challenge、年度报告导入均按管理员省/市 scope 过滤:
  联邦管理员只看本省,市管理员只操作本市。
- 管理员端公民查询不默认返回任何全量列表；必须输入档案号、身份ID、投票账户地址或投票账户公钥，后端返回 `{ items, page_size, next_cursor, has_more }`。
- SFID 公民模块不保存公民姓名，任何公民检索都不得按姓名匹配。
- `citizens` 是管理员浏览器查询用分区表；绑定完成和 CPMS 年度报告导入后同步写入，`archive_no / sfid_number / wallet_pubkey` 三者一对一由公民绑定流程强制。
- 完成绑定和年度报告导入属于 `PASSKEY` 写操作,必须携带 Passkey 换取的一次性
  `x-sfid-security-grant`。
- `citizen_bind_challenge` 必须锁定 `ARCHIVE` 中的钱包字段；前端提交绑定时不得重新传钱包地址或档案字段。
- `citizen_bind` 必须校验 `sign_response.pubkey` 等于 challenge 锁定的 `wallet_pubkey`，并校验 `payload_hash` 等于 challenge 原文哈希。
- wuminapp 扫描 `citizen_bind` 请求时必须先独立解析 `payload_hex`，确认 action、档案号、公民状态、选举权利和钱包地址与 display 一致后才签名。
- `archive_no / sfid_number / wallet_pubkey` 三者保持一对一唯一关系。
- `status_updated_at` 参与 CPMS `ARCHIVE` 签名原文；旧时间戳档案码不得覆盖新状态。

## 6. 审计事件

| 事件 | 触发场景 | 关键字段 |
|------|---------|---------|
| `CITIZEN_BIND` | 管理员完成电子护照绑定 | wallet_pubkey, archive_no, sfid_number |
| `CPMS_STATUS_EXPORT_IMPORT` | 管理员导入 CPMS 年度报告 | sfid_number, export_year, records_hash |
| `APP_VOTE_CREDENTIAL` | wuminapp 请求公民投票凭证 | wallet_pubkey, proposal_id |
