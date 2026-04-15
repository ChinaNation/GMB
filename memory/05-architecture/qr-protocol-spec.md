# WUMIN_QR_V1 统一二维码协议规范

- 版本:`WUMIN_QR_V1`
- 创建日期:2026-04-09
- 状态:唯一事实源(Single Source of Truth)
- 范围:全仓库所有二维码(CPMS 安装 4 码 QR1/QR2/QR3/QR4 除外)

## 1. 设计铁律

1. **唯一协议字符串**:`WUMIN_QR_V1`。不存在任何其他 proto 字符串。
2. **唯一 kind 枚举**:7 个值,见第 3 节。不存在任何 `type` / `purpose` / `msg_type` 字段。
3. **唯一字段命名**:见第 4 节字段字典。不存在任何别名、兼容读、`a ?? b`。
4. **唯一签名原文拼接**:见第 5 节。所有需要 sr25519 签名的 kind 共用一个拼接函数。
5. **固定码不出现时效字段**:`id` / `issued_at` / `expires_at` 三字段**直接不存在于 JSON**,不是 `null`,不是 `0`,不是 `""`。
6. **0 兼容**:删除所有历史字段别名代码,不保留任何过渡期。

## 2. 顶层 envelope 结构

```jsonc
{
  "proto": "WUMIN_QR_V1",
  "kind":  "<7 个 kind 之一>",
  "id":    "<临时码必填,固定码省略>",
  "issued_at":  <临时码必填,固定码省略,unix 秒>,
  "expires_at": <临时码必填,固定码省略,unix 秒>,
  "body":  { ... 按 kind 派发 ... }
}
```

**字段规则**:
- `proto`:恒为 `"WUMIN_QR_V1"`
- `kind`:恒为第 3 节 7 个值之一,snake_case
- `id`:临时码必填,字符长度 16-128,允许 `[a-zA-Z0-9_-]`;固定码**字段不出现**
- `issued_at` / `expires_at`:临时码必填,unix 秒级整数;固定码**字段不出现**
- `body`:必填,对象,字段集合由 kind 决定(第 4 节)

**严格规则**:
- 顶层字段**只有** `proto` / `kind` / `id` / `issued_at` / `expires_at` / `body` 六个
- `body` 里**绝对不重复**顶层字段
- 解析器遇到未知顶层字段:**报错**
- 解析器遇到 `proto != "WUMIN_QR_V1"`:**报错**
- 解析器遇到 `kind` 不在 7 值列表:**报错**

## 3. 7 个 kind 清单

| kind | 类型 | 生成者 | 扫描者 | 说明 |
|---|---|---|---|---|
| `login_challenge` | 临时 | SFID/CPMS 后端 | wumin | 登录挑战码 |
| `login_receipt` | 临时 | wumin | SFID/CPMS 后端 | 登录回执码 |
| `sign_request` | 临时 | wuminapp | wumin | 离线签名请求 |
| `sign_response` | 临时 | wumin | wuminapp | 离线签名回执 |
| `user_contact` | **固定** | wuminapp | wuminapp / citizenchain / sfid 前端 | 个人联系码 |
| `user_transfer` | 临时 | wuminapp | wuminapp / citizenchain | 临时收款码 |
| `user_duoqian` | **固定** | wuminapp | wuminapp | 多签账户码 |

## 4. body 字段字典(按 kind 派发)

### 4.1 `login_challenge`(临时)

```jsonc
"body": {
  "system":     "sfid" | "cpms",
  "sys_pubkey": "0x<hex>",
  "sys_sig":    "0x<hex>"
}
```

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `system` | string | 是 | 目标系统,`"sfid"` 或 `"cpms"` |
| `sys_pubkey` | string | 是 | 系统公钥,`0x` + hex |
| `sys_sig` | string | 是 | 系统对签名原文的 sr25519 签名,`0x` + hex |

### 4.2 `login_receipt`(临时)

```jsonc
"body": {
  "system":       "sfid" | "cpms",
  "pubkey":       "0x<hex>",
  "sig_alg":      "sr25519",
  "signature":    "0x<hex>",
  "payload_hash": "0x<hex>",
  "signed_at":    1712650010
}
```

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `system` | string | 是 | 原样回传自挑战码 |
| `pubkey` | string | 是 | 签名者(wumin 冷钱包)公钥,`0x` + hex |
| `sig_alg` | string | 是 | 固定 `"sr25519"` |
| `signature` | string | 是 | 对签名原文的签名,`0x` + hex |
| `payload_hash` | string | 是 | 签名原文字节的 SHA-256,`0x` + hex |
| `signed_at` | int | 是 | 签名完成时间,unix 秒 |

### 4.3 `sign_request`(临时)

```jsonc
"body": {
  "address":      "<SS58>",
  "pubkey":       "0x<hex>",
  "sig_alg":      "sr25519",
  "payload_hex":  "0x<hex>",
  "spec_version": 123,
  "display": {
    "action":  "transfer",
    "summary": "转账 100 GMB 给 5Grw...",
    "fields":  [
      { "label": "收款方", "value": "5Grw..." },
      { "label": "金额", "value": "100 GMB" }
    ]
  }
}
```

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `address` | string | 是 | 签名者 SS58 地址 |
| `pubkey` | string | 是 | 签名者公钥,`0x` + hex |
| `sig_alg` | string | 是 | 固定 `"sr25519"` |
| `payload_hex` | string | 是 | 待签 payload 字节,`0x` + hex,≤32768 字符 |
| `spec_version` | int | 是 | 链 runtime spec_version |
| `display` | object | 是 | 人可读摘要,见下 |
| `display.action` | string | 是 | 动作 key,`transfer` / `bind_clearing` / `duoqian_propose` 等 |
| `display.summary` | string | 是 | 一句话摘要,离线端必须显示 |
| `display.fields` | array | 否 | 结构化字段列表,每项 `{label, value}` |

### 4.4 `sign_response`(临时)

```jsonc
"body": {
  "pubkey":       "0x<hex>",
  "sig_alg":      "sr25519",
  "signature":    "0x<hex>",
  "payload_hash": "0x<hex>",
  "signed_at":    1712650010
}
```

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `pubkey` | string | 是 | 签名者公钥,必须与请求一致 |
| `sig_alg` | string | 是 | 固定 `"sr25519"` |
| `signature` | string | 是 | 对 `payload_hex` 原字节的签名 |
| `payload_hash` | string | 是 | `payload_hex` 原字节的 SHA-256 |
| `signed_at` | int | 是 | 签名完成时间 |

**验证规则**:在线端接收后,必须:
1. `id == request.id`
2. `pubkey == request.body.pubkey`
3. `payload_hash == sha256(hex_decode(request.body.payload_hex))`
4. sr25519 验证 `signature` 对 `hex_decode(payload_hex)`

### 4.5 `user_contact`(**固定**,无时效)

```jsonc
{
  "proto": "WUMIN_QR_V1",
  "kind":  "user_contact",
  "body": {
    "address": "<SS58>",
    "name":    "<昵称>"
  }
}
```

**注意顶层无 `id` / `issued_at` / `expires_at`**。

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `address` | string | 是 | 用户 SS58 地址 |
| `name` | string | 是 | 用户昵称 |

### 4.6 `user_transfer`(临时)

```jsonc
"body": {
  "address": "<SS58>",
  "name":    "<收款方昵称>",
  "amount":  "",
  "symbol":  "GMB",
  "memo":    "",
  "bank":    ""
}
```

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `address` | string | 是 | 收款方 SS58 地址 |
| `name` | string | 是 | 收款方昵称,允许空串 |
| `amount` | string | 是 | 建议金额,字符串避免浮点精度,空串表示由付款方输入 |
| `symbol` | string | 是 | 币种,默认 `"GMB"` |
| `memo` | string | 是 | 备注,允许空串 |
| `bank` | string | 是 | 清算省储行标识,允许空串 |

### 4.7 `user_duoqian`(**固定**,无时效)

```jsonc
{
  "proto": "WUMIN_QR_V1",
  "kind":  "user_duoqian",
  "body": {
    "address":     "<多签 SS58>",
    "name":        "<多签账户名>",
    "proposal_id": 0
  }
}
```

**注意顶层无 `id` / `issued_at` / `expires_at`**。

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `address` | string | 是 | 多签账户 SS58 地址 |
| `name` | string | 是 | 多签账户名 |
| `proposal_id` | int | 是 | 关联提案 ID,`0` 表示无 |

## 5. 签名原文拼接(统一函数)

所有需要 sr25519 签名的 kind(`login_challenge` 的 `sys_sig`、`login_receipt` 的 `signature`、`sign_response` 的 `signature`)共用这一个拼接:

```
WUMIN_QR_V1|<kind>|<id>|<system 或空>|<expires_at 或 0>|<principal>
```

字段之间用 `|` 分隔;缺失字段以空串占位(对 `system`)或 `0` 占位(对 `expires_at`);`<principal>` 按 kind 取值:

| kind | `<principal>` 取值 |
|---|---|
| `login_challenge` | `body.sys_pubkey`(去掉 `0x` 前缀) |
| `login_receipt` | `body.pubkey`(去掉 `0x` 前缀) |
| `sign_response` | `body.pubkey`(去掉 `0x` 前缀) |

**Dart / Rust / TS 三端必须逐字节一致**。

**`sign_request` 不在此列** —— 离线端对 `payload_hex` 的原字节签名(不是对 envelope 签名),仅 `signed_at`/`payload_hash` 进 `sign_response.body`。

## 6. 一次性 ID 规则

- 生成时机:临时码生成时分配,固定码无 ID
- 格式:`[a-zA-Z0-9_-]{16,128}`,推荐 nanoid(22 字符)或 32 hex
- 消费:解析器必须对 `login_challenge.id` 和 `sign_request.id` 执行一次性消费(防重放),存储键名 `qr.used_ids`

## 7. 时效规则

- `issued_at` ≤ 当前时间 + 30 秒(时钟偏差容限)
- `expires_at` > 当前时间(解析时)
- `expires_at - issued_at`:
  - `login_challenge`:90 秒(固定)
  - `sign_request`:最大 300 秒
  - `login_receipt` / `sign_response`:跟随对应请求的 `expires_at`
  - `user_transfer`:默认 600 秒(10 分钟),可配

## 8. 字段命名铁律(grep 0 命中清单)

**绝对不允许在全仓库出现的字段名**(CPMS 安装 4 码目录除外):

| 旧名 | 新名 |
|---|---|
| `to`(作为地址字段) | `address` |
| `account`(作为地址字段) | `address` |
| `account_pubkey` | `pubkey` |
| `admin_pubkey` | `pubkey` |
| `public_key` | `pubkey` |
| `user_address` | `address` |
| `request_id` | `id`(顶层) |
| `challenge_id` | `id`(顶层) |
| `challenge`(作为字段名) | `id`(顶层) |
| `nickname` | `name` |
| `type`(作为顶层消息类型) | `kind`(顶层) |
| `purpose` | `kind`(顶层) |
| `msg_type` | `kind`(顶层) |

**绝对不允许出现的旧协议字符串**:
```
WUMIN_QR_V1
WUMIN_QR_V1
WUMIN_QR_V1
WUMINAPP_USER_CARD_V1
```

**绝对不允许出现的旧类型名**:
```
TransferQrPayload
UserQrPayload
LoginChallenge
LoginReceipt
QrSignRequest
QrSignResponse
```

## 9. 测试契约

所有 wuminapp / wumin / citizenchain / sfid / cpms 的 QR 相关测试必须读取 `memory/05-architecture/qr-protocol-fixtures/*.json` 作为 golden 样本:

- 序列化测试:`toJson(body) + envelope` 必须**逐字节**等于对应 fixture
- 反序列化测试:`parse(fixture)` 必须解出预期字段,字段数量、类型、值全相等
- 解析器负向测试:改一个字段名 / 多一个字段 / 少一个字段 —— 必须全部报错

fixture 文件命名:
```
memory/05-architecture/qr-protocol-fixtures/login_challenge.json
memory/05-architecture/qr-protocol-fixtures/login_receipt.json
memory/05-architecture/qr-protocol-fixtures/sign_request.json
memory/05-architecture/qr-protocol-fixtures/sign_response.json
memory/05-architecture/qr-protocol-fixtures/user_contact.json
memory/05-architecture/qr-protocol-fixtures/user_transfer.json
memory/05-architecture/qr-protocol-fixtures/user_duoqian.json
```

## 10. 修改规范的流程

本 spec 是唯一事实源。改规范前必须:

1. 先改本文件
2. 同步改 fixtures(两者永远一致)
3. 再改所有端的代码
4. 跑全量测试(所有端的测试都读取 fixture,任一端不同步 → 测试红)
5. 更新任务卡

**绝不允许**"先在代码里改,再回来补 spec" —— 这是字段散乱的历史根源。
