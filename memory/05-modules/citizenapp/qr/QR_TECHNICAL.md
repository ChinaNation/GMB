# CitizenApp QR 技术说明

- 更新日期:2026-07-28
- 唯一事实源:`memory/01-architecture/qr/qr-protocol-spec.md`
- Action 注册表:`memory/01-architecture/qr/qr-action-registry.md`

## 1. 边界

CitizenApp 只使用 `QR_V1`。所有扫码 envelope 顶层字段固定为 `p/k/i/e/b`。

| k | 名称 | CitizenApp 职责 |
|---:|---|---|
| 1 | `sign_request` | 生成需要外部签名的请求二维码 |
| 2 | `sign_response` | 扫描外部签名设备返回的签名响应并验签 |
| 3 | `user_contact` | **用户码**：仅为链上 CID↔AccountId 闭环命中的身份账户生成，唯一入口用户主页；扫描时复核二维码 CID 与链上绑定 |
| 4 | `user_transfer` | **收款码**：唯一入口聊天-加号-收付款。当前预留，生成方待实现（任务卡 `20260729-qr-three-code-classification`），该入口暂出钱包码 |
| 5 | `wallet_code` | **钱包码**：唯一入口钱包-账户详情，任意账户无条件生成，body 只有 `account_id` |

展示型二维码按入口分类，不做任何运行时分流：用户主页出用户码（`lib/8964/profile/user_qr_page.dart`），
钱包-账户详情出钱包码（`lib/wallet/pages/wallet_qr_page.dart`），两者共用展示外壳
`lib/qr/widgets/qr_display_scaffold.dart`。

扫码判据：`contact` 模式只接受用户码，扫到钱包码/收款码给出明确原因并拒绝（通讯录关系必须锚
永久 CID）；`transfer`/`dispatch` 模式接受用户码、钱包码、收款码与裸地址。

CitizenApp 不处理管理员扫码登录。登录签名请求由 OnChina 页面生成,由 CitizenWallet 公民钱包扫码签名；
登录第 1 步的目标账户由 CitizenWallet 出示的钱包码提供。

`k=5` 由已废止的 `chat_node_pairing` 回收给钱包码。旧 `node_peer_id`/`node_multiaddr`/`endpoint_kind`
载荷靠 body 字段集精确匹配拒绝，不再保存桌面区块链软件通信节点信息。

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

`user_contact` 是固定身份码，不带 `i/e`，body 严格只含
`cid_number + ss58_address + display_name`。CID 和公开昵称必须无首尾空格，
SS58 必须是 prefix 2027 的规范地址；旧 `contact_name` 和任何未知字段直接拒绝。

身份账户生成 `k=3` 前必须命中链上 CID↔AccountId 闭环。未注册账户和其它钱包账户
只生成五分钟 `k=4 user_transfer`；链读失败从严拒绝生成。扫码添加联系人时必须把
SS58 还原为 AccountId，再按链上绑定解析 CID，并与二维码声明的 `cid_number`
精确比较；`display_name` 只作公开展示，不写入私人 `contact_remark`。

`user_transfer` 是收款临时码,带 `i/e`,body 可以包含 `memo`。备注仅用于付款方展示和业务填充,不得参与签名协议真源。

## 5. 统一实现入口

- Dart 协议常量:`citizenapp/lib/qr/qr_protocols.dart`
- Envelope 解析:`citizenapp/lib/qr/envelope.dart`
- 签名请求 body:`citizenapp/lib/qr/bodies/sign_request_body.dart`
- 签名响应 body:`citizenapp/lib/qr/bodies/sign_response_body.dart`
- 签名会话:`citizenapp/lib/signer/qr_signer.dart`
- 签名页面:`citizenapp/lib/qr/pages/qr_sign_session_page.dart`

任何新增扫码签名场景必须先登记 action,再复用这些入口。

管理员人员字段的协议顺序固定为 `account_id + family_name + given_name`。CitizenApp 构造个人多签创建、个人管理员更换或机构相关链交易时，必须把三个字段完整写入 `review_payload`；CitizenWallet 只在确认页合并显示姓名，不按姓名授权。同一次业务操作只生成一个签名请求并接收一个签名响应，不叠加第二次确认签名。

## 6. 测试要求

- `test/qr/qr_router_test.dart`
- `test/qr/qr_sign_session_test.dart`
- `test/signer/qr_signer_test.dart`
- 钱包码相关 widget/page 测试

测试必须覆盖：短字段往返、未知 `k/a` 拒绝、签名响应
`request_id / signer_public_key` 错配拒绝、链 payload 大于 256 字节哈希签名。
