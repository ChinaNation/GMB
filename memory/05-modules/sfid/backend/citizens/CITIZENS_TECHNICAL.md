# CITIZENS 模块技术文档

> 历史:本模块由 phase23d(2026-05-01)从 `backend/operate/` 整体迁入,
> 业务上聚焦"公民身份"业务族,与 `sheng_admins` / `shi_admins` 等管理员模块平行。

## 1. 模块定位

- 路径:`backend/citizens`
- 职责:承载公民身份相关业务,包括公民身份绑定凭证签发、链上绑定推送、
  CPMS 站点扫码状态更新,以及 wuminapp 自有的投票账户登记/查询。
- 来源:phase23d 由原 `backend/operate/` 物理搬迁;原 `operate/` 整目录已删除。

## 2. 模块结构

- `binding.rs`
  - `citizen_bind_challenge` 绑定/解绑 challenge 签发
  - `citizen_bind` / `citizen_unbind` 公民身份绑定/解绑
  - `citizen_push_chain_bind` / `citizen_push_chain_unbind` 推链
  - `app_vote_account_register` / `app_vote_account_status` wuminapp 投票账户
- `chain_binding.rs`
  - 公民绑定 / 解绑链上 extrinsic 提交 helper
- `chain_vote.rs`
  - wuminapp 公民投票凭证签发接口
- `chain_joint_vote.rs`
  - 联合投票人口快照凭证签发接口
- `status.rs`
  - `admin_cpms_status_scan` CPMS 站点扫公民状态
- `cpms_qr.rs`
  - `canonical_citizen_qr_text`
  - `canonical_status_qr_text`
  - `verify_cpms_qr_signature`(签名链路废弃,保留 canonical 工具供复用)
- `handler.rs` 占位骨架,后续 Phase 拆 axum handler 时启用
- `vote.rs`   占位骨架,后续 Phase 把 `app_vote_account_*` 拆出至此模块
- `mod.rs`    子模块注册入口

## 3. 路由接线

- `POST /api/v1/admin/citizen/bind/challenge` -> `citizens::binding::citizen_bind_challenge`
- `POST /api/v1/admin/citizen/bind` -> `citizens::binding::citizen_bind`
- `POST /api/v1/admin/citizen/unbind` -> `citizens::binding::citizen_unbind`
- `POST /api/v1/admin/citizen/bind/push-chain` -> `citizens::binding::citizen_push_chain_bind`
- `POST /api/v1/admin/citizen/unbind/push-chain` -> `citizens::binding::citizen_push_chain_unbind`
- `POST /api/v1/app/vote-account/register` -> `citizens::binding::app_vote_account_register`
- `GET  /api/v1/app/vote-account/status` -> `citizens::binding::app_vote_account_status`
- `POST /api/v1/app/vote/credential` -> `citizens::chain_vote::app_vote_credential`
- `GET  /api/v1/app/voters/count` -> `citizens::chain_joint_vote::app_voters_count`
- `POST /api/v1/admin/cpms-status/scan` -> `citizens::status::admin_cpms_status_scan`
  (经由 `shi_admins::mod.rs` 转发)

## 4. 依赖与边界

- 依赖:
  - `scope`(省域范围判断)
  - 全局公共能力(鉴权、审计、状态存储、签名封装)
  - `citizens::chain_binding`(链上 `bind_sfid` / `unbind_sfid` extrinsic 推送)
  - `sheng_admins::institutions`(`resolve_site_province_via_shard`)
- 边界:
  - `citizens` 仅负责公民身份业务。
  - 链上交互能力在 `backend/citizens/chain_*`。
  - SFID 号生成入口在 `backend/sfid/generator.rs`。

## 5. 关键一致性约束

- 旧 `admin_bind_confirm` + `RewardStateRecord(Pending)` 双写顺序约束已随老绑定流程下线;
  当前 `citizen_bind` 走 challenge + signature 模式,详见 `binding.rs` 内联注释。
- CPMS QR 签名链路已废弃(SFID-CPMS QR v1 走 archive_import 端点),
  `cpms_qr::verify_cpms_qr_signature` 仅保留向后兼容的 canonical 文本拼装能力。

## 6. 审计事件

| 事件 | 触发场景 | 关键字段 |
|------|---------|---------|
| `CITIZEN_BIND` | 管理员确认绑定 | account_pubkey, archive_no |
| `CITIZEN_UNBIND` | 管理员解绑 | account_pubkey, archive_no |
| `CITIZEN_PUSH_CHAIN_*` | 推链 extrinsic 状态 | block_hash, ext_index |
| `CPMS_STATUS_SCAN` | CPMS 站点扫公民状态 | site_sfid, qr_id, new_status |
| `CPMS_STATUS_SCAN_META` | 状态扫码元数据 | request_id, actor_ip |
| `APP_VOTE_ACCOUNT_REGISTER` | wuminapp 注册投票账户 | account_pubkey, archive_no |
