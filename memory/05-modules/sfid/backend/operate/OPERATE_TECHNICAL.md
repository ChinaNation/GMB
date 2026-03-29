# OPERATE 模块技术文档

## 1. 模块定位

- 路径：`backend/src/operate`
- 职责：承载“操作业务”实现，统一管理管理员操作流程与 CPMS 二维码验签能力。
- 来源：由原 `business` 中以下能力整合而来：
  - 绑定操作（原 2）
  - 状态操作（原 3）
  - CPMS 二维码验签工具（原 6）

## 2. 模块结构

- `binding.rs`
  - `admin_bind_scan`
  - `admin_bind_confirm`
  - `admin_unbind`
- `status.rs`
  - `admin_cpms_status_scan`
- `cpms_qr.rs`
  - `canonical_citizen_qr_text`
  - `canonical_status_qr_text`
  - `verify_cpms_qr_signature`
- `mod.rs`
  - 操作业务子模块导出

## 3. 路由接线

- `POST /api/v1/admin/bind/scan` -> `operate::binding::admin_bind_scan`
- `POST /api/v1/admin/bind/confirm` -> `operate::binding::admin_bind_confirm`
- `POST /api/v1/admin/bind/unbind` -> `operate::binding::admin_unbind`
- `POST /api/v1/admin/cpms-status/scan` -> `operate::status::admin_cpms_status_scan`

## 4. 依赖与边界

- 依赖：
  - `business::scope`（省域范围判断）
  - 全局公共能力（鉴权、审计、状态存储、签名封装）
- 边界：
  - `operate` 仅负责“管理员操作业务”。
  - 区块链接口业务在 `backend/src/chain`。
  - SFID 生成业务在 `backend/src/sfid/admin.rs` 与 `backend/src/sfid/mod.rs`。

## 5. 关键一致性约束

- `admin_bind_confirm` 在创建 `RewardStateRecord(Pending)` 时采用“先 DB 后内存”顺序：
  - 先调用 `persist_reward_state_db(..., None)` 持久化；
  - DB 成功后才写入 `store.reward_state_by_pubkey`。
- 若奖励状态持久化失败（含未生效），接口返回 `1502 reward state persistence failed`，不保留仅内存成功的中间状态。

## 6. 审计事件

| 事件 | 触发场景 | 关键字段 |
|------|---------|---------|
| `BIND_SCAN` | 管理员扫描公民 QR 码 | account_pubkey, archive_no, site_sfid |
| `BIND_CONFIRM` | 管理员确认绑定 | account_pubkey, archive_index |
| `BIND_CONFIRM_META` | 绑定确认元数据 | request_id, actor_ip |
| `BIND_UNBIND` | 管理员解绑 | account_pubkey, archive_index |
| `BIND_UNBIND_META` | 解绑元数据 | request_id, actor_ip, reason |
