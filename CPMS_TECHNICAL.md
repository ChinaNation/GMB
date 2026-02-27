# CPMS 技术文档（扫码登录实现对齐）

## 1. 方案定位与状态

- CPMS 为完全离线软件，统一使用 `WUMINAPP_LOGIN_V1` 离线双向扫码。
- 流程：`手机扫挑战码 -> 用户确认签名 -> 手机出回执码 -> CPMS 扫回执并验签授权`。
- 当前状态：协议、字段、签名原文与 `wuminapp` 已对齐，可按同一核心层接入。

## 2. 角色与授权规则

- 初始化生成 3 个超级管理员账户。
- 超级管理员可新增 n 个操作管理员。
- 登录判定：先验签后授权。
- 授权规则：账户在 CPMS 本地 RBAC 名单中才允许登录。

## 3. 协议对齐（与实现保持一致）

挑战二维码（CPMS -> 手机）：

```json
{
  "proto": "WUMINAPP_LOGIN_V1",
  "system": "cpms",
  "request_id": "uuid",
  "challenge": "string",
  "nonce": "uuid",
  "issued_at": 1760000000,
  "expires_at": 1760000060,
  "aud": "cpms-local-app",
  "origin": "cpms-device-id"
}
```

签名原文（固定顺序）：

```text
WUMINAPP_LOGIN_V1|cpms|aud|origin|request_id|challenge|nonce|expires_at
```

回执二维码（手机 -> CPMS）：

```json
{
  "proto": "WUMINAPP_LOGIN_V1",
  "request_id": "uuid",
  "account": "ss58-address",
  "pubkey": "0x...",
  "sig_alg": "sr25519",
  "signature": "0x...",
  "signed_at": 1760000020
}
```

## 4. CPMS 端验签与授权步骤

1. 解析回执并读取 `request_id/account/signature`。
2. 用挑战缓存重建签名原文。
3. `sr25519` 验签通过后，检查挑战时效与一次性消费状态。
4. 消费 `request_id`，进入 RBAC 授权判定。
5. 授权通过进入系统，失败返回拒绝原因。

## 5. 风控与错误码

- `1101`：协议头无效
- `1102`：挑战过期
- `1103`：挑战已消费（重放）
- `1201`：签名验签失败
- `1202`：账户与公钥不一致
- `2201`：账户不在 CPMS 授权名单
