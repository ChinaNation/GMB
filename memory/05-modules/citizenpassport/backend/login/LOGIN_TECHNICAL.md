# CPMS 登录模块技术文档

## 1. 模块定位
`backend/login/` 是 CPMS 的登录模块，负责管理员登录与会话建立。

本模块只处理登录链路，不负责业务权限判定和业务数据操作。

## 2. 职责范围
### 2.1 本模块负责
- QR 登录挑战码下发
- CitizenWallet 登录回执签名验签
- 登录结果轮询，且结果领取一次后立即删除
- 登录会话创建与登出
- 当前登录管理员查询；`auth/me` 和 QR 登录成功结果返回 `user_id / user_group / admin_display_name`
- 登录相关状态持久化（`sessions/login_challenges/qr_login_results`）

### 2.2 本模块不负责
- 角色权限控制（`admins` / `operators` 的业务授权）
- 管理员管理（创建/删除）
- 公民档案与二维码业务
- 审计以外的业务领域逻辑

## 3. 路由清单
由 `router()` 统一注册并由主路由 `merge` 挂载：

- `POST /api/v1/admin/auth/qr/challenge`
- `POST /api/v1/admin/auth/qr/complete`
- `GET /api/v1/admin/auth/qr/result`
- `GET /api/v1/admin/auth/me`
- `POST /api/v1/admin/auth/logout`

## 4. 持久化表（PostgreSQL）
- `sessions`
- `login_challenges`
- `qr_login_results`

短期登录状态由 CPMS 本机 PostgreSQL 承载。`sessions.access_token` 只写入
`cpms_session` HttpOnly Cookie，前端不再保存 token。Cookie 默认不设置浏览器 Max-Age，
实际空闲过期以数据库 `sessions.expires_at` 为准；正式 `citizenpassport-ubuntu24-amd64.run` 或
`citizenpassport-ubuntu24-arm64.run` 离线安装通过 nginx 提供 `https://www.citizenpassport.com/`，并设置
`CPMS_COOKIE_SECURE=true` 给 Cookie 增加 `Secure`。正式安装脚本只创建数据库和权限，登录相关
表由后端启动时的 `MIGRATOR.run()` 统一创建。定时清理逻辑删除过期 session、过期 challenge
和超时二维码登录结果。

## 5. 安全约束
- QR challenge 由后端生成 `challenge_id/session_id`，前端不得自带会话 ID。
- challenge 与 `session_id` 绑定，带有效期，完成登录后标记消费，防重放。
- `qr_login_results` 中的成功结果领取一次后立即删除。
- 管理员被删除后，对应 session 被清理；无管理员记录的 session 视为无效。
- 签名必须通过 `sr25519` 验签，且签名公钥必须属于管理员。
- 管理员 15 分钟无活动过期，操作员 30 分钟无活动过期；每次鉴权成功按角色滑动续期。
- 登录 challenge、登录完成和登录结果轮询均有本机 IP 级限流，超限返回 `429 / CPMS_RATE_LIMITED`。
- UTC 每年 1 月 11 日起，如果存在已超过 1 月 10 日仍未导出的 `CPMS_STATUS_EXPORT` 年度报告，`operators` 登录完成和登录结果领取都会被拒绝；已有操作员会话在鉴权时也会被清理。`admins` 不受该锁定影响，用于补导年度报告。

## 6. 扫码登录协议（与 CitizenWallet 对齐）
### 6.1 挑战二维码字段
登录二维码使用统一 envelope：

```json
{
  "proto": "CITIZEN_QR_V1",
  "kind": "login_challenge",
  "id": "login_xxx",
  "issued_at": 1779990000,
  "expires_at": 1779990090,
  "body": {
    "system": "cpms",
    "sys_pubkey": "0x...",
    "sys_sig": "0x..."
  }
}
```

后端保存的 `session_id` 只用于浏览器轮询绑定，不进入二维码主载荷。

### 6.2 验签拼串（后端与 CitizenWallet 一致）
```text
CITIZEN_QR_V1|login_challenge|{challenge_id}|cpms|{expires_at}|{cpms_pubkey_without_0x}
```

CitizenWallet 返回 `login_receipt` 后，CPMS 校验回执公钥、签名算法和签名值；通过后创建
`sessions` 记录，并在 `qr/result` 的成功响应中设置 HttpOnly Cookie。

## 7. 依赖边界
本模块依赖主模块提供通用能力：
- 统一响应封装与错误结构
- 管理员公钥查询
- PostgreSQL 连接池
- 审计日志写入
- 通用签名工具方法

## 8. 后续拆分建议
如继续模块化，可在 `backend/` 下独立出：
- `authz/`：权限与权限校验
- `archive/`：档案业务
- `qr/`：业务二维码签发与打印
