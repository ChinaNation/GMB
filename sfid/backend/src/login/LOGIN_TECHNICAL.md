# SFID Login 模块技术文档

## 1. 模块目标
`src/login` 负责 SFID 管理员认证链路，统一处理“登录挑战生成、签名验签、会话签发、会话校验”。

该模块只负责认证（Authentication），不负责业务权限动作本身（业务接口由 `key-admins`、`super-admins`、`operator-admins` 接手）。

## 2. 目录与职责
```text
src/login/
├── mod.rs                # 登录接口、会话中间件、验签与鉴权函数
└── LOGIN_TECHNICAL.md    # 本文档
```

## 3. 对外接口（管理员登录）
- `GET /api/v1/admin/auth/check`
- `POST /api/v1/admin/auth/identify`
- `POST /api/v1/admin/auth/challenge`
- `POST /api/v1/admin/auth/verify`
- `POST /api/v1/admin/auth/qr/challenge`
- `POST /api/v1/admin/auth/qr/complete`
- `GET /api/v1/admin/auth/qr/result`

## 4. 认证模型
- 账户标识：`admin_pubkey`
- 会话载体：`Bearer <access_token>`
- 会话缓存：`admin_sessions`
- 挑战缓存：`login_challenges`
- 二维码登录结果缓存：`qr_login_results`

## 5. 核心流程
### 5.1 二维码登录
1. 前端调用 `qr/challenge` 生成挑战。
2. 手机扫码签名后回传签名回执（`qr/complete`）。
3. 后端验签成功后签发 `access_token`。
4. 前端轮询 `qr/result` 获取登录成功状态与会话。

二维码登录统一遵循 `WUMINAPP_LOGIN_V1` 协议规范：

- 挑战字段：`proto/system/request_id/challenge/nonce/issued_at/expires_at/sys_pubkey/sys_sig`
- SFID 场景：`sys_cert` 可空
- 手机签名原文：`WUMINAPP_LOGIN_V1|system|request_id|challenge|nonce|expires_at`
- 回执兼容字段：`request_id|challenge_id`、`pubkey|admin_pubkey|public_key`

### 5.2 普通 challenge 登录
1. 先 `identify` 校验管理员身份。
2. `challenge` 生成带上下文的挑战串。
3. `verify` 验签通过后签发会话。

## 6. 安全策略
- 挑战短时效：默认 `90` 秒。
- 挑战一次性消费：`consumed=true` 后不可复用。
- 会话过期与空闲超时双重校验。
- 登录签名算法：`sr25519`。
- 兼容部分钱包的 `<Bytes>...</Bytes>` 包裹签名。
- 角色限制函数集中在本模块：
  - `require_admin_any`
  - `require_admin_write`
  - `require_super_admin`
  - `require_super_or_key_admin`
  - `require_key_admin`
  - `require_super_or_operator_or_key_admin`
- 省域防御（`SUPER_ADMIN`）：
  - `require_super_admin` 要求 `admin_province` 非空。
  - `require_super_or_key_admin` 在 `ctx.role == SUPER_ADMIN` 时同样要求 `admin_province` 非空。

## 7. 与业务模块边界
- `src/login`：只做认证和会话校验。
- `src/key-admins` / `src/super-admins` / `src/operator-admins` / `src/business`：
  - 通过 `require_*` 函数拿到认证上下文。
  - 按角色继续执行业务权限与数据范围校验。

## 8. 关键实现约束
- 登录相关函数统一位于 `src/login/mod.rs`。
- `main.rs` 仅保留路由装配与模块接线，不再持有登录业务实现。
- 新增登录需求必须先更新本文件再改代码，避免认证口径分叉。
- 登录协议禁止再引入 `aud` 作为移动端扫码验签字段；网页上下文如需保留，只能作为服务端会话上下文单独保存。
- SFID 自身系统身份信任来源为区块链当前登记公钥；二维码中的 `sys_pubkey` 必须与链上当前值一致。
