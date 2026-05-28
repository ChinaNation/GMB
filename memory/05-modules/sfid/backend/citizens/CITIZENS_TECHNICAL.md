# CITIZENS 模块技术文档

## 1. 模块定位

- 路径：`sfid/backend/citizens`
- 职责：承载公民电子护照绑定、CPMS 状态扫码、公民投票凭证签发和联合投票人口快照凭证签发。
- 电子护照绑定边界：CPMS 档案码提供 `archive_no / archive_status / valid_from / valid_until / status_updated_at / wallet_address / wallet_pubkey / wallet_sig_alg`；SFID 验档案码后生成 `WUMIN_QR_V1 / sign_request`；wuminapp 使用对应钱包签名；SFID 验签通过后直接写入本地绑定结果并向 wuminapp 状态接口返回。

## 2. 模块结构

- `binding.rs`
  - `citizen_bind_challenge`：验 CPMS `ARCHIVE` 档案码并签发 wuminapp 签名请求。
  - `citizen_bind`：验 wuminapp `sign_response` 并完成电子护照绑定。
- `vote.rs`
  - `app_myid_status`：wuminapp 查询电子护照绑定状态。
- `chain_vote.rs`
  - `app_vote_credential`：公民投票凭证签发接口。
- `chain_joint_vote.rs`
  - `app_voters_count`：联合投票人口快照凭证签发接口。
- `model.rs`
  - 公民电子护照记录、`bind_status`、绑定 DTO、状态扫码 QR 载荷。
- `handler.rs`
  - `admin_list_citizens`：后台公民列表。
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
- `POST /api/v1/admin/cpms-status/scan` -> `citizens::status::admin_cpms_status_scan`
- `GET  /api/v1/admin/citizens` -> `citizens::handler::admin_list_citizens`
- `GET  /api/v1/public/identity/search` -> `citizens::handler::public_identity_search`

## 4. 依赖与边界

- 依赖：
  - `cpms::verify_cpms_archive_qr`：验 CPMS 档案码和归属密文。
  - `login::parse_sr25519_pubkey_bytes`：解析 wuminapp 钱包公钥。
  - 全局公共能力：鉴权、审计、状态存储。
- 边界：
  - 电子护照绑定必须以 CPMS `ARCHIVE` 档案码为入口。
  - 绑定必须使用 wuminapp 对 SFID challenge 的 sr25519 签名。
  - `citizens` 不实现投票流程；公民投票只调用投票凭证签发接口。
  - 公民 DTO 放 `citizens/model.rs`，不得塞入全局 `models`。

## 5. 关键一致性约束

- 三端字段统一：`archive_no / archive_status / identity_status / valid_from / valid_until / status_updated_at / wallet_address / wallet_pubkey / wallet_sig_alg / sfid_code / bind_status`。
- `bind_status` 只表达电子护照绑定状态：`PENDING / BOUND`；`identity_status` 表达身份 ID 当前有效状态。
- `citizen_bind_challenge` 必须锁定 `ARCHIVE` 中的钱包字段；前端提交绑定时不得重新传钱包地址或档案字段。
- `citizen_bind` 必须校验 `sign_response.pubkey` 等于 challenge 锁定的 `wallet_pubkey`，并校验 `payload_hash` 等于 challenge 原文哈希。
- `archive_no / sfid_code / wallet_pubkey` 三者保持一对一唯一关系。
- `status_updated_at` 参与 CPMS `ARCHIVE` 签名原文；旧时间戳档案码不得覆盖新状态。

## 6. 审计事件

| 事件 | 触发场景 | 关键字段 |
|------|---------|---------|
| `CITIZEN_BIND` | 管理员完成电子护照绑定 | wallet_pubkey, archive_no, sfid_code |
| `CPMS_STATUS_SCAN` | CPMS 站点扫公民状态 | sfid_number, qr_id, new_status |
| `CPMS_STATUS_SCAN_META` | 状态扫码元数据 | request_id, actor_ip |
| `APP_VOTE_CREDENTIAL` | wuminapp 请求公民投票凭证 | wallet_pubkey, proposal_id |
