# SFID Login 模块技术文档

- 最后更新:2026-05-02
- 任务卡:
  - `memory/08-tasks/done/20260502-sfid-cleanup残留整改.md`

## 1. 模块目标

`sfid/backend/login` 负责 SFID 管理员认证链路,统一处理登录挑战生成、
签名验签、会话签发、二维码登录和会话校验。

本模块只负责 Authentication 和登录态守卫。业务权限、数据范围、链交互动作
必须留在各业务模块内继续校验,不得把业务 handler 塞回登录目录。

## 2. 当前目录

```text
sfid/backend/login/
├── mod.rs        # 模块聚合与对外 API re-export
├── model.rs      # 登录 challenge、session、二维码结果、请求/响应 DTO
├── handler.rs    # 普通登录接口:check/logout/identify/challenge/verify
├── qr_login.rs   # WUMIN_QR_V1 扫码登录 challenge/complete/result
├── guards.rs     # require_admin_* 鉴权守卫、session 校验、签名 key bootstrap
└── signature.rs  # sr25519 验签、公钥解析、challenge 清理、展示名辅助
```

## 3. 对外接口

- `GET /api/v1/admin/auth/check`
- `POST /api/v1/admin/auth/logout`
- `POST /api/v1/admin/auth/identify`
- `POST /api/v1/admin/auth/challenge`
- `POST /api/v1/admin/auth/verify`
- `POST /api/v1/admin/auth/qr/challenge`
- `POST /api/v1/admin/auth/qr/complete`
- `GET /api/v1/admin/auth/qr/result`

## 4. 认证模型

- 管理员标识:`admin_pubkey`
- 角色模型:当前只保留 `ShengAdmin` / `ShiAdmin`
- 会话载体:`Bearer <access_token>`
- 会话缓存:`admin_sessions`,登录后同步写入 GlobalShard
- 挑战缓存:`login_challenges`
- 二维码登录结果缓存:`qr_login_results`

## 5. 核心流程

### 5.1 普通 Challenge 登录

1. `identify` 根据管理员身份二维码解析 `admin_pubkey` 并检查管理员状态。
2. `challenge` 生成带 `origin/domain/session_id/nonce` 的 challenge。
3. `verify` 校验 sr25519 签名,一次性消费 challenge 并签发 8 小时会话。
4. ShengAdmin 登录成功后会触发本省本人 signing keypair 本地 bootstrap。

### 5.2 二维码登录

1. `qr/challenge` 生成 WUMIN_QR_V1 登录挑战和 SFID 系统签名。
2. 手机扫码后按 `login_receipt` 原文签名并提交 `qr/complete`。
3. 后端验签成功后写入 `qr_login_results` 并签发会话。
4. 网页轮询 `qr/result` 获取 `PENDING / SUCCESS / EXPIRED`。

二维码登录统一遵循 `WUMIN_QR_V1`:

- 系统签名由 `SFID_SIGNING_SEED_HEX` 派生的 SFID main signer 产出。
- 手机端验签原文由 `qr::build_signature_message` 生成。
- 登录协议禁止重新引入 `aud` 作为移动端扫码验签字段。

## 6. 守卫函数

- `require_admin_any`:读取登录态,返回 `AdminAuthContext`。
- `require_admin_write`:当前与 `require_admin_any` 等价,保留写接口语义入口。
- `require_sheng_admin`:只放行 `ShengAdmin`,并要求存在省域 scope。
- `require_admin_session_middleware`:Axum 路由层会话校验中间件。

## 7. 边界规则

- `login` 不承载机构、公民、CPMS、省管理员治理等业务 handler。
- 业务模块不得直接读取 session cache,只能通过 `require_admin_*` 获取认证上下文。
- 角色范围过滤放在 `scope`,不放回 `login`。
- 省管理员一主两备治理放在 `sheng_admins`,登录目录只负责本人登录后的本地签名密钥加载。
