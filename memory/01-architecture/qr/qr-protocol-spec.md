# QR_V1 统一二维码协议规范

- 版本:`QR_V1`
- 更新日期:2026-06-23
- 状态:当前详细事实源,由 `memory/07-ai/unified-protocols.md` 统一管辖
- 范围:全仓库所有“生成二维码 -> 扫码识别 -> 签名/确认 -> 签名响应验签”的二维码流程

## 1. 设计铁律

1. 唯一协议字符串:`QR_V1`。不得恢复历史协议名、登录专用 QR kind 或任何第二套扫码协议名。
2. 唯一 envelope 字段:`p/k/i/e/b`。不得恢复 `proto/kind/id/issued_at/expires_at/body` 作为线上 QR 字段。
3. 唯一签名请求字段:`a/g/u/d`。业务场景放在 `a`,扫码流向放在 `k`。
4. 唯一签名响应字段:`u/s`。签名响应不携带 payload、payload hash、签名时间或展示字段。
5. 唯一验签真源:生成方按 `i` 找回本地 session 中的 action、payload、公钥和过期时间后验签。
6. 唯一展示真源:扫码端必须由 `a + d(payload)` 本地解码展示;QR 不携带 `display`、`summary`、`fields`。
7. 固定码不出现时效字段:`i/e` 直接不存在,不是 `null`、`0` 或空串。
8. 不兼容旧字段。解析器遇到旧字段、别名字段、未知字段必须报错。

## 2. 顶层 Envelope

```jsonc
{
  "p": "QR_V1",
  "k": 1,
  "i": "req_01HXYZ4VQK8NRPM2G7FJD9TBC3",
  "e": 1780000000,
  "b": {}
}
```

| 字段 | 类型 | 必填 | 注释 |
|---|---|---|---|
| `p` | string | 是 | 协议版本,恒为 `QR_V1` |
| `k` | int | 是 | 扫码流向码,见第 3 节 |
| `i` | string | 临时码必填 | request/session id,16-128 字符,允许 `[A-Za-z0-9_-]` |
| `e` | int | 临时码必填 | 过期 unix 秒;固定码不出现 |
| `b` | object | 是 | body,字段由 `k` 决定 |

顶层字段只允许 `p/k/i/e/b`。临时码必须有 `i/e`;固定码禁止有 `i/e`。

## 3. k 扫码流向码

| k | 名称 | 类型 | 生成方 | 扫码方 | 注释 |
|---:|---|---|---|---|---|
| 1 | `sign_request` | 临时 | CitizenApp / CitizenWallet / CID / citizenchain node | 签名方 | 请求扫码方签名 `b.d` |
| 2 | `sign_response` | 临时 | 签名方 | 请求生成方 | 回传签名结果 |
| 3 | `user_contact` | 固定 | CitizenApp / CitizenWallet | 需要地址的一方 | 展示钱包地址和联系人名 |
| 4 | `user_transfer` | 临时 | 收款方 | 付款方 | 收款码,可带金额和备注 |
| 5 | `im_node_pairing` | 固定 | citizenchain node | CitizenApp | 通信节点配对 |

登录、公民身份确认、管理员确认、交易签名、运行时升级等都不新增 `k`;它们统一是 `k=1` 签名请求,具体业务由 `b.a` 区分。

## 4. k=1 sign_request

```jsonc
{
  "p": "QR_V1",
  "k": 1,
  "i": "req_01HXYZ4VQK8NRPM2G7FJD9TBC3",
  "e": 1780000000,
  "b": {
    "a": 515,
    "g": 1,
    "u": "1DWTxxX90xxhFBq9BKmf1oIshViFTM3jmlaE56Vton0",
    "d": "AgMA1DWTxxX90xxhFBq9BKmf1oIshViFTM3jmlaE56Vton1BnA"
  }
}
```

| 字段 | 类型 | 必填 | 注释 |
|---|---|---|---|
| `a` | int | 是 | 业务动作码,见 `qr-action-registry.md` |
| `g` | int | 是 | 签名算法码,当前只允许 `1 = sr25519` |
| `u` | string | 是 | 期望签名者 32 字节公钥,base64url 无填充 |
| `d` | string | 是 | 待签 payload 原始字节,base64url 无填充 |

签名字节规则:

| 场景 | `a` 规则 | 签名字节 |
|---|---|---|
| 普通链交易 | `a = (pallet_index << 8) | call_index` | `d` 必须是生成方用当前 runtime 类型构造的 `SignedPayload` SCALE 字节;长度 ≤256B 签原文,>256B 签 `blake2_256(payload)` |
| 登录 | `a = 1` | 签 payload 原文 |
| 公民链上身份确认 | `a = 2` | `d` 必须是 `VotingIdentityPayload` SCALE bytes,签 `blake2_256(GMB || 0x10 || d)` |
| OnChina 管理员治理文本载荷 | `a = 3` | 签 payload 原文 |
| 管理员激活 / 解密 | `a = 5/6` | 签二进制 payload 原文 |
| Runtime 升级哈希签名 | `a = 7` 或 RuntimeUpgrade 链 action | `d` 必须是同一 runtime `SignedPayload::using_encoded` 得到的 32B signing bytes,签该 32B |

链交易生成方不得手写拼接 `call_data/era/nonce/tip/additional_signed` 或 signed extrinsic。citizenchain node、CitizenApp 热钱包和其它链交易生成方必须统一使用当前 runtime 类型构造 `TxExtension`、`SignedPayload` 和 `UncheckedExtrinsic`，再把 `SignedPayload` 的 SCALE 字节放入 `b.d`。

## 5. k=2 sign_response

```jsonc
{
  "p": "QR_V1",
  "k": 2,
  "i": "req_01HXYZ4VQK8NRPM2G7FJD9TBC3",
  "e": 1780000000,
  "b": {
    "u": "1DWTxxX90xxhFBq9BKmf1oIshViFTM3jmlaE56Vton0",
    "s": "qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqg"
  }
}
```

| 字段 | 类型 | 必填 | 注释 |
|---|---|---|---|
| `u` | string | 是 | 实际签名者 32 字节公钥,base64url 无填充 |
| `s` | string | 是 | 64 字节 sr25519 签名,base64url 无填充 |

生成方验签必须使用本地 session:

1. `p == QR_V1`
2. `k == 2`
3. `i == 本地请求 id`
4. `e` 未过期
5. `b.u == 本地 expected pubkey`
6. 按本地 session 重新计算 payload hash,必须等于生成请求时保存的 `expected_payload_hash`
7. 按本地 session 的 `a + payload` 计算签名字节后验证 `b.s`

## 6. k=3 user_contact

固定码,不带 `i/e`。

```jsonc
{
  "p": "QR_V1",
  "k": 3,
  "b": {
    "address": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    "contact_name": "张三"
  }
}
```

| 字段 | 类型 | 必填 | 注释 |
|---|---|---|---|
| `address` | string | 是 | SS58 钱包地址 |
| `contact_name` | string | 是 | 联系人名,允许空串仅在 UI 层兜底 |

## 7. k=4 user_transfer

临时码,带 `i/e`。

```jsonc
{
  "p": "QR_V1",
  "k": 4,
  "i": "pay_01HXYZ4VQK8NRPM2G7FJD9TBC3",
  "e": 1780000000,
  "b": {
    "address": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    "recipient_name": "张三",
    "amount": "100.50",
    "symbol": "GMB",
    "memo": "房租",
    "bank": ""
  }
}
```

| 字段 | 类型 | 必填 | 注释 |
|---|---|---|---|
| `address` | string | 是 | 收款方 SS58 钱包地址 |
| `recipient_name` | string | 是 | 收款方显示名,允许空串 |
| `amount` | string | 是 | 建议金额,字符串避免浮点精度,空串表示付款方输入 |
| `symbol` | string | 是 | 币种,当前 `GMB` |
| `memo` | string | 是 | 备注,允许空串 |
| `bank` | string | 是 | 清算行/清算网络标识,允许空串 |

## 8. k=5 im_node_pairing

固定码,不带 `i/e`。

```jsonc
{
  "p": "QR_V1",
  "k": 5,
  "b": {
    "node_peer_id": "12D3KooWNode",
    "node_multiaddr": "/ip4/127.0.0.1/tcp/30333/ws/p2p/12D3KooWNode",
    "endpoint_kind": "ip4"
  }
}
```

| 字段 | 类型 | 必填 | 注释 |
|---|---|---|---|
| `node_peer_id` | string | 是 | 通信节点 libp2p PeerId |
| `node_multiaddr` | string | 是 | 配对 multiaddr,不携带 RPC URL |
| `endpoint_kind` | string | 是 | `ip4` 或 `ip6` |

## 9. 签名原文拼接

只有系统对 QR envelope 元信息签名时使用该函数。普通交易签名响应不签 envelope,只签请求 payload。

```
QR_V1|<k>|<i>|<system 或空>|<e 或 0>|<principal>
```

| 字段 | 注释 |
|---|---|
| `k` | 数字扫码流向码 |
| `i` | 请求 id |
| `system` | `onchina` / 空串；`onchina` 是链上中国平台登录签名 payload 常量 |
| `e` | 过期 unix 秒;无则为 `0` |
| `principal` | 去掉 `0x` 的小写 hex 公钥 |

## 10. Fixture 契约

当前 fixture:

```text
memory/01-architecture/qr/qr-protocol-fixtures/sign_request.json
memory/01-architecture/qr/qr-protocol-fixtures/sign_response.json
memory/01-architecture/qr/qr-protocol-fixtures/user_contact.json
memory/01-architecture/qr/qr-protocol-fixtures/user_transfer.json
```

不得新增登录专用 fixture。登录统一复用 `sign_request.json` / `sign_response.json`,业务含义由 `b.a=1` 表达。

## 11. 修改流程

1. 先改本文件和 `qr-action-registry.md`。
2. 同步 fixtures。
3. 同步 Rust / TS / Dart 的解析、生成、验签入口。
4. 跑真实扫码签名链路或对应端到端测试。
5. 更新任务卡和模块文档。
