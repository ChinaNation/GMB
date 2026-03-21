# CPMS 登录模块技术文档

## 1. 模块定位
`backend/src/login/` 是 CPMS 的登录模块，负责管理员登录与会话建立。

本模块只处理登录链路，不负责业务权限判定和业务数据操作。

## 2. 职责范围
### 2.1 本模块负责
- 管理员身份识别（`identify`）
- 登录挑战码下发（普通签名登录与二维码登录）
- 登录签名验签
- 登录结果轮询
- 登录会话创建与登出
- 登录相关状态持久化（`sessions/login_challenges/qr_login_results`）

### 2.2 本模块不负责
- 角色权限控制（`SUPER_ADMIN` / `OPERATOR_ADMIN` 的业务授权）
- 管理员管理（创建/更新/禁用）
- 公民档案与二维码业务
- 审计以外的业务领域逻辑

## 3. 路由清单
由 `router()` 统一注册并由主路由 `merge` 挂载：

- `POST /api/v1/admin/auth/identify`
- `POST /api/v1/admin/auth/challenge`
- `POST /api/v1/admin/auth/verify`
- `POST /api/v1/admin/auth/qr/challenge`
- `POST /api/v1/admin/auth/qr/complete`
- `GET /api/v1/admin/auth/qr/result`
- `POST /api/v1/admin/auth/logout`

## 4. 持久化表（PostgreSQL）
- `sessions`
- `login_challenges`
- `qr_login_results`

## 5. 安全约束
- challenge 一次性消费，防重放
- challenge 带有效期，过期拒绝
- 校验 challenge 与 `session_id` 绑定关系
- 管理员状态必须为 `ACTIVE`
- 签名必须通过 `sr25519` 验签

## 6. 扫码登录协议（与 wuminapp 对齐）
### 6.1 挑战二维码字段
- `proto`: 固定 `WUMINAPP_LOGIN_V1`
- `system`: 固定 `cpms`
- `request_id`: 挑战 ID（即 `challenge_id`）
- `challenge`: 随机挑战串
- `nonce`: 随机串
- `issued_at`: 秒级时间戳
- `expires_at`: 秒级时间戳（TTL=90 秒）
- `sys_pubkey`: CPMS 当前登录系统公钥（0x + hex）
- `sys_sig`: CPMS 对挑战原文的签名（0x + hex）
- `sys_cert`: SFID 对该 CPMS 公钥的背书签名（0x + hex，必填）

说明：

- `origin`/`domain`/`session_id` 可以保留为网页侧会话上下文字段，但不进入移动端扫码挑战协议主载荷
- CPMS 不与区块链交互；`sys_cert` 由 SFID 背书提供信任链

### 6.2 验签拼串（后端与移动端一致）
```text
WUMINAPP_LOGIN_V1|system|request_id|challenge|nonce|expires_at
```

### 6.3 CPMS 证书链规范

CPMS 登录二维码中的 `sys_cert` 对应以下固定原文：

```text
CPMS_CERT_V1|cpms_pubkey|site_sfid|issued_at|expires_at
```

要求：

- `cpms_pubkey` 必须等于挑战码中的 `sys_pubkey`
- `site_sfid` 为当前 CPMS 实例绑定的机构编号
- `sys_cert` 由 SFID 当前有效私钥签发
- WuminApp 使用区块链当前 SFID 公钥验证 `sys_cert`

## 7. 依赖边界
本模块依赖主模块提供通用能力：
- 统一响应封装与错误结构
- 管理员公钥查询
- PostgreSQL 连接池
- 审计日志写入
- 通用签名工具方法

## 8. 后续拆分建议
如继续模块化，可在 `backend/src` 下独立出：
- `authz/`：权限与角色校验
- `admin/`：管理员管理
- `archive/`：档案业务
- `qr/`：业务二维码签发与打印
