# SFID 技术文档（扫码登录实现态）

## 1. 方案定位与状态

- SFID 登录统一使用 `WUMINAPP_LOGIN_V1` 离线双向扫码。
- 流程：`手机扫挑战码 -> 用户确认签名 -> 手机出回执码 -> SFID 扫回执并验签登录`。
- 当前状态：已与 `wuminapp` 联调通过，实测可登录成功。

## 2. 角色与授权规则

- 系统内置 45 个超级管理员账户。
- 超级管理员可新增 n 个操作管理员账户。
- 登录判定：先验签，后授权。
- 授权规则：账户在 SFID 管理员名单中则允许登录；否则拒绝。

## 3. 协议对齐（与实现一致）

挑战二维码（SFID -> 手机）：

```json
{
  "proto": "WUMINAPP_LOGIN_V1",
  "system": "sfid",
  "request_id": "uuid",
  "challenge": "string",
  "nonce": "uuid",
  "issued_at": 1760000000,
  "expires_at": 1760000060,
  "aud": "sfid-local-app",
  "origin": "sfid-device-id"
}
```

签名原文（固定顺序）：

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

## 4. SFID 端验签与登录步骤

1. 解析回执字段，提取 `request_id/account(pubkey)/signature`。
2. 用 `request_id` 查找原挑战并重建签名原文。
3. 使用 `sr25519` 验签；失败返回 `signature verify failed`。
4. 校验挑战未过期且未消费。
5. 消费 `request_id`（一次性）。
6. 通过管理员名单做授权与角色判定，签发会话。

## 5. 手机端约束（SFID 依赖）

- 手机端必须先展示确认弹窗，用户点击后才签名。
- 手机端本地白名单必须允许 `system=sfid` 且 `aud/origin` 匹配。
- 手机端本地防重放：同 `request_id` 不重复签名。

## 6. 风控与错误码

- `1101`：协议头无效
- `1102`：挑战过期
- `1103`：挑战已消费（重放）
- `1201`：签名验签失败
- `1202`：账户与公钥不一致
- `2202`：账户不在 SFID 授权名单
