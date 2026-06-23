# CitizenApp Signer 技术说明

- 更新日期:2026-06-22
- 唯一事实源:`memory/01-architecture/qr/qr-protocol-spec.md`
- Action 注册表:`memory/01-architecture/qr/qr-action-registry.md`

## 1. 模块职责

`citizenapp/lib/signer/` 负责在线端签名会话:

1. 构造 `QR_V1 k=1` 签名请求。
2. 展示请求二维码给外部签名设备扫描。
3. 扫描 `QR_V1 k=2` 签名响应。
4. 用本地会话保存的 action、payload、期望公钥验签。
5. 把通过验签的签名交给业务模块提交。

CitizenApp 不在 QR 内写入展示摘要,也不接收 QR 内的 payload hash。所有业务展示由签名端按 `a+d` 解码。

## 2. 请求字段

签名请求 body 固定为:

| 字段 | 注释 |
|---|---|
| `a` | 动作码。链交易为 `(pallet_index << 8) | call_index` |
| `g` | 签名算法,当前固定 `1 = sr25519` |
| `u` | 期望签名者 32B 公钥,base64url 无填充 |
| `d` | 待签 payload 原始字节,base64url 无填充 |

`QrSigner.buildRequest()` 必须只输出上述字段。禁止恢复 `display`、`summary`、`payload_hash`、地址 hex 冗余字段或旧字段别名。

## 3. 响应验签

签名响应 body 固定为:

| 字段 | 注释 |
|---|---|
| `u` | 实际签名者 32B 公钥,base64url 无填充 |
| `s` | 64B sr25519 签名,base64url 无填充 |

`QrSigner.parseResponse()` 必须校验:

1. `p == QR_V1`。
2. `k == 2`。
3. `i == request.id`。
4. `e` 未过期。
5. `u == expectedPubkey`。
6. `s` 可解码为 64B。
7. 使用本地 request 的 payload 计算签名字节后验签通过。

链交易 payload 长度大于 256 字节时签 `blake2_256(payload)`；否则签 payload 原文。非链 payload 签原文。

## 4. 业务边界

- CID 绑定:CitizenApp 生成/展示绑定签名请求,扫描签名响应后提交 CID。
- 链交易:CitizenApp 生成交易 payload 签名请求,扫描签名响应后广播交易。
- 管理员登录:不属于 CitizenApp,由 CitizenWallet 公民钱包处理。

## 5. 测试

必须覆盖:

1. `QR_V1` 短字段请求/响应往返。
2. id 错配、公钥错配、签名错误均拒绝。
3. 链 payload 大于 256 字节时按 Substrate 规则签 hash。
4. 未登记 action 或旧字段进入解析器时拒绝。
