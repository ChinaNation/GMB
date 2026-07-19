# CitizenApp QR 技术说明

- 更新日期:2026-07-19
- 唯一事实源:`memory/01-architecture/qr/qr-protocol-spec.md`
- Action 注册表:`memory/01-architecture/qr/qr-action-registry.md`

## 1. 边界

CitizenApp 只使用 `QR_V1`。所有扫码 envelope 顶层字段固定为 `p/k/i/e/b`。

| k | 名称 | CitizenApp 职责 |
|---:|---|---|
| 1 | `sign_request` | 生成需要外部签名的请求二维码 |
| 2 | `sign_response` | 扫描外部签名设备返回的签名响应并验签 |
| 3 | `user_contact` | 生成/扫描联系人钱包码 |
| 4 | `user_transfer` | 生成/扫描收款码 |

CitizenApp 不处理管理员扫码登录。登录签名请求由 OnChina 页面生成,由 CitizenWallet 公民钱包扫码签名。

`k=5 chat_node_pairing` 已删除。CitizenApp 扫到旧通信节点配对码时按未知 `k` 拒绝，不再保存桌面区块链软件通信节点信息。

## 2. 签名请求

CitizenApp 生成签名请求时只能使用:

```json
{"p":"QR_V1","k":1,"i":"...","e":1780000000,"b":{"a":515,"g":1,"u":"...","d":"..."}}
```

字段含义:

| 字段 | 注释 |
|---|---|
| `a` | 动作码。链交易为 `(pallet_index << 8) | call_index` |
| `g` | 签名算法,当前固定 `1 = sr25519` |
| `u` | 期望签名者 32B 公钥,base64url 无填充 |
| `d` | 待签 payload 原始字节,base64url 无填充 |

二维码内不得携带 `display`、`summary`、`payload_hash` 或旧字段别名。签名页面展示内容必须由扫码端按 `a+d` 解码得到。

## 3. 签名响应

CitizenApp 扫描 `k=2` 签名响应:

```json
{"p":"QR_V1","k":2,"i":"...","e":1780000000,"b":{"u":"...","s":"..."}}
```

验签必须使用本地会话保存的请求:

1. `i` 等于本地请求 id。
2. `e` 未过期。
3. `b.u` 等于当前请求期望公钥。
4. `b.s` 为 64B sr25519 签名。
5. 按本地 `a + payload` 计算签名字节并验签。

链交易 payload 长度大于 256 字节时,签名字节必须是 `blake2_256(payload)`；否则签 payload 原文。这是防止 `InvalidTransaction::BadProof(0x010004)` 的唯一规则。

## 4. 用户码

`user_contact` 是固定码,不带 `i/e`,body 只包含联系人所需字段。

`user_transfer` 是收款临时码,带 `i/e`,body 可以包含 `memo`。备注仅用于付款方展示和业务填充,不得参与签名协议真源。

## 5. 统一实现入口

- Dart 协议常量:`citizenapp/lib/qr/qr_protocols.dart`
- Envelope 解析:`citizenapp/lib/qr/envelope.dart`
- 签名请求 body:`citizenapp/lib/qr/bodies/sign_request_body.dart`
- 签名响应 body:`citizenapp/lib/qr/bodies/sign_response_body.dart`
- 签名会话:`citizenapp/lib/signer/qr_signer.dart`
- 签名页面:`citizenapp/lib/qr/pages/qr_sign_session_page.dart`

任何新增扫码签名场景必须先登记 action,再复用这些入口。

管理员人员字段的协议顺序固定为 `admin_account + family_name + given_name`。CitizenApp 构造个人多签创建、个人管理员更换或机构相关链交易时，必须把三个字段完整写入 `review_payload`；CitizenWallet 只在确认页合并显示姓名，不按姓名授权。同一次业务操作只生成一个签名请求并接收一个签名响应，不叠加第二次确认签名。

## 6. 测试要求

- `test/qr/qr_router_test.dart`
- `test/qr/qr_sign_session_test.dart`
- `test/signer/qr_signer_test.dart`
- 钱包码相关 widget/page 测试

测试必须覆盖:短字段往返、未知 `k/a` 拒绝、签名响应 id/pubkey 错配拒绝、链 payload 大于 256 字节哈希签名。
