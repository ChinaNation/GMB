# QR_V1 统一二维码协议规范

- 版本:`QR_V1`
- 更新日期:2026-07-29
- 状态:当前详细事实源,由 `memory/07-ai/unified-protocols.md` 统一管辖
- 范围:全仓库所有“生成二维码 -> 扫码识别 -> 签名/确认 -> 签名响应验签”的二维码流程

## 1. 设计铁律

1. 唯一协议字符串:`QR_V1`。不得恢复历史协议名、登录专用 QR kind 或任何第二套扫码协议名。
2. 唯一 envelope 字段:`p/k/i/e/b`。不得恢复 `proto/kind/id/issued_at/expires_at/body` 作为线上 QR 字段。
3. 唯一签名请求字段:`a/g/u/d`。业务场景放在 `a`,扫码流向放在 `k`。
4. 唯一签名响应字段:`u/s`。签名响应不携带 payload、payload hash、签名时间或展示字段。
5. 唯一验签真源:生成方按 `i` 找回本地 session 中的 action、payload、`signer_public_key` 和过期时间后验签。
6. 唯一展示真源:扫码端必须由 `a + d(review_payload)` 本地解码展示;QR 不携带 `display`、`summary`、`fields`。
7. 固定码不出现时效字段:`i/e` 直接不存在,不是 `null`、`0` 或空串。
8. 不兼容旧字段。解析器遇到旧字段、别名字段、未知字段必须报错。
9. 签名判定只有两种结果:`Normal/正常` 或 `Reject/拒绝`。不得引入未知、警告、部分识别、可忽略等第三状态。
10. 用户可见确认内容必须全部来自本地中文 action registry 和 payload decoder;动作名、字段名或枚举值缺少中文翻译时必须红色拒绝。

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
| 3 | `user_contact` | 固定 | CitizenApp 已绑定 CID 的身份账户 | CitizenApp / CitizenWallet / OnChina | **用户码**:声明永久 CID、当前绑定地址和公开昵称 |
| 4 | `user_transfer` | 临时 | 仅 CitizenApp | 付款方 | **收款码**:一笔收款请求,可带金额和备注 |
| 5 | `wallet_code` | 固定 | CitizenApp / CitizenWallet 任意账户 | CitizenApp / OnChina / citizenchain node | **钱包码**:只声明账户,不含任何身份字段 |

展示型二维码按「谁生成 + 表达什么」三分,入口即语义,禁止任何运行时分流:

| 码 | 表达 | `k` | 生成端 | 唯一入口 |
|---|---|---:|---|---|
| 用户码 | 人(永久 CID) | 3 | 仅 CitizenApp,且 CID↔AccountId 闭环命中 | 用户主页 |
| 钱包码 | 账户 | 5 | CitizenApp + CitizenWallet,任意账户无条件 | 钱包-账户详情 |
| 收款码 | 一笔收款请求 | 4 | 仅 CitizenApp | 聊天-加号-收付款 |

「是否带时效字段」由「生成端是否联网」推导:钱包码必须离线端也能出,因此固定;收款码只在联网端生成,因此允许 `i/e`。离线设备无 NTP,不得签发带绝对时间戳的凭证。

登录、公民签名确认、管理员确认、交易签名、运行时升级等都不新增 `k`;它们统一是 `k=1` 签名请求,具体业务由 `b.a` 区分。

`k=5` 曾用于已废止的 `chat_node_pairing`(桌面通信节点配对)。该流程整体取消后码值回收给钱包码,规格见第 8 节。不得恢复桌面区块链软件通信节点配对流程;旧字段 `node_peer_id`、`node_multiaddr`、`endpoint_kind` 不属于当前 QR_V1 可解析 body,携带它们的旧码会因 body 字段集不匹配被拒绝。

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
| `u` | string | 是 | 压缩传输键；内部唯一语义名为 `signer_public_key`，通常是期望签名者 32 字节公钥的 base64url 无填充编码；仅 `citizen_occupy/citizen_rebind` 必须为空字符串，由钱包在 `d` 的账户零槽原位填入所选账户 |
| `d` | string | 是 | `review_payload` 原始字节,base64url 无填充;除 Runtime 升级 hash-only 外,必须可被扫码端完整解码和中文展示 |

`review_payload` 与签名字节必须分离:

| 名称 | 含义 | 规则 |
|---|---|---|
| `review_payload` | QR `b.d` 携带、给用户审阅和中文展示的完整载荷 | 普通交易必须完整可解码;不得用 32B hash 冒充 |
| `signing_bytes` | 钱包实际交给 sr25519 签名的字节 | 由扫码端根据 action 和 `review_payload` 本地计算 |

签名字节规则:

| 场景 | `a` 规则 | 签名字节 |
|---|---|---|
| 普通链交易 | `a = (pallet_index << 8) | call_index` | `d` 必须是生成方用当前 runtime 类型构造的完整 `SignedPayload` 原始三元组 SCALE 字节;长度 ≤256B 签原文,>256B 签 `blake2_256(review_payload)` |
| 登录 | `a = 1` | 签 payload 原文 |
| 公民链上身份确认 | `a = 2` | `d` 必须是 `VotingIdentityPayload` SCALE bytes,签 `blake2_256(GMB || 0x10 || d)` |
| 注册局首次绑定 | `a = 10 citizen_occupy` | `d = genesis_hash + bounded cid + 32B 零 account_id 槽 + revision=0(u64 LE) + expires_at(u64 LE)`；钱包严格解码且确认无尾字节后原位填入账户，签 `blake2_256(GMB || 0x12 || 完整授权)` |
| 注册局换绑 | `a = 11 citizen_rebind` | `d = genesis_hash + bounded cid + current_account_id + 32B 零 new_account_id 槽 + nonzero revision(u64 LE) + expires_at(u64 LE)`；钱包拒绝当前账户与新账户相同，原位填入新账户后签 `blake2_256(GMB || 0x1f || 完整授权)` |
| OnChina 管理员治理文本载荷 | `a = 3` | 签 payload 原文 |
| 管理员激活 / 解密 | `a = 5/6` | 签二进制 payload 原文 |
| Runtime 升级哈希签名 | `a = 7` 或已登记 RuntimeUpgrade hash-only action | `d` 允许是 32B signing bytes,签该 32B;这是 QR_V1 唯一 hash-only 例外 |

`citizen_occupy/citizen_rebind` 的外层 `e` 必须与内层 `expires_at` 完全相等。账户零槽
非零、CID 非规范 SCALE、revision 不符合上述约束、载荷截断或存在尾字节时，CitizenApp
与 CitizenWallet 都必须拒签；不得恢复旧版 `bounded_cid ++ account_id` 末尾拼接。

OnChina 登录生成 `a=1` 前必须先扫描 `k=5 wallet_code` 钱包码，取 `b.account_id`
作为唯一目标账户，再按 `account_id → CidByAccountId → CidRegistry Active →
管理员记录 Active` 单向反查确认其为在职管理员；三级中任一不成立必须拒绝登录，不得
「读不到就放行」。管理员私钥保管在离线的 CitizenWallet，登录第 1 步必须由
CitizenWallet 自己出示钱包码即可完成，不得要求联网热钱包参与——否则管理员只能在
「私钥落到联网设备」与「无法登录」之间二选一。

二维码不携带 CID 与昵称:CID 由链上 `CidByAccountId` 反查得到，管理员姓名由链上
记录读出，两者都比二维码自述更可靠，且「两字段互不匹配」这类攻击形态从根上不存在。

随后 `k=1` 请求的 `b.u` 必须编码该账户的 32 字节公钥，`b.d` 固定为 UTF-8
`onchina`；不得生成空 `b.u`，不得允许扫码钱包自行选择其它账户。登录响应仍使用
`k=2`，其 `b.u` 必须与请求目标和数据库目标账户一致。

链交易生成方不得手写拼接 `call_data/era/nonce/tip/additional_signed` 或 signed extrinsic。citizenchain node、CitizenApp 热钱包、OnChina 和其它链交易生成方必须统一使用当前 runtime 类型构造 `TxExtension`、`SignedPayload` 和 `UncheckedExtrinsic`；QR `b.d` 放入完整 `review_payload`，实际签名输入单独按 `SignedPayload::using_encoded` 计算。

普通链交易若 `b.d` 只有 32 字节且不能按 action registry 解码为完整业务 payload,扫码端必须红色拒绝,不得展示“载荷 32 字节”后继续签名。

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
| `u` | string | 是 | 压缩传输键；内部唯一语义名为 `signer_public_key`，值是实际签名者 32 字节公钥的 base64url 无填充编码 |
| `s` | string | 是 | 64 字节 sr25519 签名,base64url 无填充 |

生成方验签必须使用本地 session:

1. `p == QR_V1`
2. `k == 2`
3. `i == 本地请求 id`
4. `e` 未过期
5. `b.u == 本地 expected signer_public_key`
6. 按本地 session 重新计算 payload hash,必须等于生成请求时保存的 `expected_payload_hash`
7. 按本地 session 的 `a + payload` 计算签名字节后验证 `b.s`

同一业务操作只允许一次密钥签名。扫码端进入签名中状态后必须阻止重复触发；生成出首个 `k=2` 响应二维码后不得对同一 `i` 再次调用密钥或生成第二个响应。需要重试时必须由发起方创建新的请求 id，而不是在原请求上追加确认签名。

## 6. k=3 user_contact

固定码,不带 `i/e`。

```jsonc
{
  "p": "QR_V1",
  "k": 3,
  "b": {
    "cid_number": "CN001-CTZN-000000001-2026",
    "ss58_address": "w5FhUDLW4BxsE1QXK4sNjPZ8rqSnK2QeVpUfXzqczpWdxChxV",
    "display_name": "晨光寻路者"
  }
}
```

| 字段 | 类型 | 必填 | 注释 |
|---|---|---|---|
| `cid_number` | string | 是 | 永久公民身份号；无首尾空格，UTF-8 长度 1–32 字节 |
| `ss58_address` | string | 是 | 当前 CID 绑定账户的本链规范 SS58 地址，prefix 固定 2027；只用于展示和边界输入输出 |
| `display_name` | string | 是 | 公开昵称；无首尾空格、1–40 个 Unicode 字符，只作展示 |

用户码只能由 CitizenApp 在链上 CID↔AccountId 闭环命中的身份账户生成，唯一入口是
用户主页；链读失败从严拒绝生成，不得降级成其它码。未注册账户与其它钱包子账户
不出用户码——它们在钱包-账户详情出 `k=5` 钱包码。CitizenWallet 是离线钱包，没有
CID 真源，永远只解析 `k=3`、不生成。解析器必须拒绝旧 `contact_name`、缺失字段、
未知字段、非 2027 SS58 及非规范 CID/昵称。

用户码是唯一能写入通讯录的码：通讯录关系必须锚永久 CID，不能锚会换绑的账户。
钱包码与收款码进通讯录必须拒绝。

## 7. k=4 user_transfer(收款码)

临时码,带 `i/e`。唯一入口是 CitizenApp 聊天-加号-收付款,唯一生成端是 CitizenApp
(联网,时钟可信)。CitizenWallet 永远不生成、不解析收款码。

**当前状态:预留,生成方待实现。** 落地任务卡
`memory/08-tasks/open/20260729-qr-three-code-classification.md`。零生成方是已规划
未实现,不是残桩,清扫审计不得删除。生成方落地时必须同批补两件事:金额/备注输入
UI(否则 `amount` 恒为空串,该码与钱包码无差别),以及扫码侧 `e` 过期即拒绝(当前全仓
`e` 的校验只有签名请求/响应一处,收款码路径无校验)。

```jsonc
{
  "p": "QR_V1",
  "k": 4,
  "i": "pay_01HXYZ4VQK8NRPM2G7FJD9TBC3",
  "e": 1780000000,
  "b": {
    "ss58_address": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
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
| `ss58_address` | string | 是 | 收款方 SS58 展示地址，不作为授权主键 |
| `recipient_name` | string | 是 | 收款方显示名,允许空串 |
| `amount` | string | 是 | 建议金额,字符串避免浮点精度,空串表示付款方输入 |
| `symbol` | string | 是 | 币种,当前 `GMB` |
| `memo` | string | 是 | 备注,允许空串 |
| `bank` | string | 是 | 清算行/清算网络标识,允许空串 |

## 8. k=5 wallet_code(钱包码)

固定码,不带 `i/e`。

```jsonc
{
  "p": "QR_V1",
  "k": 5,
  "b": {
    "account_id": "0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"
  }
}
```

| 字段 | 类型 | 必填 | 注释 |
|---|---|---|---|
| `account_id` | string | 是 | 账户唯一标识；小写 `0x` 加 64 位十六进制，即 sr25519 公钥原字节 |

body 只有 `account_id` 一个字段。**不得**携带钱包账户名、公开昵称、CID、SS58 或
任何时效字段：本机钱包标签用户可随意改写、无任何链上或服务端约束，一旦进入二维码
就会被扫码端当成对方公开身份显示。展示用 SS58(prefix 2027)由扫码端从 `account_id`
自行派生。

生成端为 CitizenApp 与 CitizenWallet 的钱包-账户详情，任意账户无条件生成,包括
CID 已绑定的身份账户——账户详情表达的是「账户」，身份由用户主页的用户码表达。

`account_id` 与 `k=3` 用户码的 `ss58_address` 口径不同：钱包码的主要消费方是 OnChina
管理员登录与「扫码识别账户」，它们要的就是 `account_id`(登录请求 `b.u`、`same_account_id`
比对均为 `account_id` 语义)，用 SS58 会让每个扫码端多一次 decode 与 prefix 校验。
用户码保持 `ss58_address` 不变,该差异是有意保留的已知项。

钱包码的合法用途只有三类:按账户转账、OnChina 管理员登录第 1 步、OnChina 与
citizenchain node 前端的「扫码识别账户」。**写入通讯录必须拒绝**(无 CID 真源)。

`k=5` 是回收码值:旧 `chat_node_pairing` 已随桌面通信节点配对流程整体废止,其
`node_peer_id`、`node_multiaddr`、`endpoint_kind` 三字段不属于当前 QR_V1 可解析 body。
无需为旧码写专门的拒绝逻辑——顶层字段集精确匹配(`p/k/b`)之后,body 字段集精确匹配
会拒掉它。

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

## 12. Action registry 代码真源

当前唯一代码真源:

```text
citizenchain/crates/qr-protocol/registry/actions.yaml
citizenchain/crates/qr-protocol/registry/fields.yaml
citizenchain/crates/qr-protocol/registry/reject_reasons.yaml
```

`memory/01-architecture/qr/qr-action-registry.md` 是人类可读登记表和审查入口;代码、测试、生成和跨端校验必须以 `citizenchain/crates/qr-protocol/registry/*` 为准。各端不得再手写第二套 action 常量、中文标签、字段标签或签名判定分支。

### 12.1 载荷字段编解码代码真源(2026-07-30 收归)

QR body 里的定长字段(公钥 `b.u` 32 字节、签名 `b.s` 64 字节)统一用 base64url(no padding)
承载,进入 Rust 侧后一律转成 ADR-040 规范文本(小写 `0x` + 十六进制)。host 侧唯一实现:

```text
citizenchain/crates/qr-protocol/src/codec.rs
  b64_to_prefixed_hex(value, expected_len, field)  # 解码 + 长度校验
  bytes_to_b64(bytes)                              # 编码
  public_key_b64(bytes, field)                     # 32 字节公钥编码(长度不符即拒)
  PUBLIC_KEY_BYTES = 32 / SIGNATURE_BYTES = 64
```

`node/src/governance/signing.rs` 与 `onchina/src/core/qr/mod.rs` 原各有一份自写实现
(行为相同但错误类型与文案不同),已于创世前审计收归到上面这一份,两侧只保留
错误类型映射。**禁止再各自实现**:两份解码一旦在长度校验或大小写上出现分毫差异,
同一个二维码就会在一端通过、另一端拒绝。

当前已知散落实现只允许作为待收归对象,不得继续作为协议真源:

- `citizenapp/lib/qr/*`
- `citizenwallet/lib/qr/*`
- `citizenchain/onchina/src/core/qr/*`
- `citizenchain/node/src/governance/signing.rs`
- `citizenchain/node/frontend/shared/qr/citizenQr.ts`
