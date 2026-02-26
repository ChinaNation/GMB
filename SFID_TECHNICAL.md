# SFID 技术文档

## 1. 登录方案定位

- SFID 系统统一采用 `WUMINAPP_LOGIN_V1`。
- 登录方式固定为“离线双向扫码”：
  - 第一次：`wuminapp` 扫描 SFID 登录挑战二维码。
  - 第二次：SFID 软件扫描 `wuminapp` 回执二维码。
- 用户公钥即账户标识，私钥仅在 `wuminapp` 手机端本地离线签名。

## 2. 角色与授权规则

- 系统内置 45 个超级管理员账户。
- 超级管理员可新增 n 个操作管理员账户。
- 登录准入规则：
  - 验签通过且账户在“超级管理员/操作管理员”授权表中，允许登录。
  - 不在授权表中，拒绝登录。

## 3. 协议与数据结构

挑战二维码（SFID -> 手机）：

```json
{
  "proto": "WUMINAPP_LOGIN_V1",
  "system": "sfid",
  "request_id": "uuid",
  "challenge": "base64-32bytes",
  "nonce": "uuid",
  "issued_at": 1760000000,
  "expires_at": 1760000060,
  "aud": "sfid-local-app",
  "origin": "sfid-device-id"
}
```

签名原文：

```text
WUMINAPP_LOGIN_V1|sfid|aud|origin|request_id|challenge|nonce|expires_at
```

回执二维码（手机 -> SFID）：

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

## 4. 校验与风控

- `request_id` 一次性消费，禁止重放。
- 挑战有效期建议 60 秒，超时拒绝。
- 验签算法固定 `sr25519`，签名与账户公钥必须一致。
- `system` 必须为 `sfid`，`aud/origin` 必须在 SFID 白名单。
- 登录审计字段：`request_id/account/role/device/result/time`。

## 5. 错误码建议

- `1101`：协议头无效
- `1102`：挑战过期
- `1103`：挑战已消费
- `1201`：签名验签失败
- `1202`：账户与公钥不一致
- `2202`：账户不在 SFID 授权名单
