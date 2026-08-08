# CitizenApp Signer 技术说明

- 更新日期:2026-08-06
- 唯一事实源:`memory/01-architecture/qr/qr-protocol-spec.md`
- Action 注册表:`memory/01-architecture/qr/qr-action-registry.md`
- 签名域注册表:`citizenchain/runtime/primitives/src/sign.rs`(权威源)
  与 [ADR-026](../../../04-decisions/ADR-026-unified-signing-protocol.md)

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
5. `u` 解码出的 `signerPublicKey` 必须转换为预期 `signerAccountId`。
6. `s` 可解码为 64B。
7. 使用本地 request 的 payload 计算签名字节后验签通过。

链交易 payload 长度大于 256 字节时签 `blake2_256(payload)`；否则签 payload 原文。非链 payload 签原文。

## 4. 业务边界

- 公民签名确认：“聊天 → 扫一扫”按请求公钥自动定位本机热账户；“我的 → 钱包 →
  我的钱包 → 账户卡片 → 扫码图标”把签名账户锁定为该卡片的 `account_id`。两个入口统一
  复用 `CitizenIdentitySignService`，独立解码投票/参选身份载荷并校验请求、载荷、本机
  账户三方公钥一致；账户不一致必须在读取私钥前拒绝，旧的独立 `MyIdSignPage` 已删除。
- 广场账户动作与公民占号/换绑也使用同一账户维度签名模型，最终统一调用
  `WalletManager.signForAccountId()`；不得把非0账户的请求回退到账户0签名。
- 账户卡片入口只接收签名请求，扫码页面仍复用既有 `QrScanPage`，不得修改蓝色对准框、
  提示小字、相册、手电筒或其它扫描 UI。
- CitizenWallet 继续通过统一离线签名服务处理同一 `citizen_identity` action；动作标题只读取生成的 QR action registry，统一为“公民签名确认”。
- 链交易:CitizenApp 生成交易 payload 签名请求,扫描签名响应后广播交易。
- 管理员登录:不属于 CitizenApp,由 CitizenWallet 公民钱包处理。

## 5. 金标向量镜像(安全关键)

`citizenapp/test/signer/fixtures/signing_domain_vectors.json` 是**镜像**，真源在
`citizenchain/runtime/primitives/tests/fixtures/`。跨端门禁
`.github/scripts/check-golden-vectors-sync.mjs` 对签名域组设了 `requireAll: true`：

> **本镜像必须完整覆盖真源全部签名域，缺一条直接失败**，即使该域 CitizenApp 本身不签。

这条是刻意的：镜像只允许是真源子集的规则对签名域**不适用**，因为「本端不签这个域」
今天成立不代表明天成立，缺条目会让新增域在本端悄悄没有任何字节约束。冷钱包不设镜像
文件、直读真源，所以只有本端会被这条门禁挡住 —— 2026-08-06 新增
`OP_SIGN_ONCHINA_ADMIN=0x20` 时正是漏了本端镜像才触发。

改真源后必须同步本文件并跑 `node .github/scripts/check-golden-vectors-sync.mjs`。

## 6. 公民身份授权载荷的防重放三件套

`VotingIdentityConsentPayload.decode` 接收的是**完整授权字节**，不是内层载荷：

```
genesis_hash(32) ++ VotingIdentityPayload ++ expected_identity_version(8) ++ expires_at(8)
```

唯一写入端是 `citizenchain/onchina/src/domains/citizens/chain_identity.rs`
的 `build_citizen_identity_authorization_bytes`。只喂内层载荷一律解不出 → 按
「无法独立验证」拒签。**测试夹具必须照写入端真实字节形态构造**：本端夹具曾只造内层载荷，
在防重放三件套落地后长期红着，而「解码通过」在那种夹具下证明不了任何跨端一致性。

## 7. 测试

必须覆盖:

1. `QR_V1` 短字段请求/响应往返。
2. id 错配、公钥错配、签名错误均拒绝。
3. 链 payload 大于 256 字节时按 Substrate 规则签 hash。
4. 未登记 action 或旧字段进入解析器时拒绝。
5. 公民签名与广场动作入口对 action 不符、载荷不可完整展示、账户不在本机或卡片账户
   不匹配全部拒绝，并断言实际调用 `signForAccountId()` 的账户正是目标 `account_id`。
6. 公民身份授权载荷缺防重放三件套(只给内层载荷)必须拒签；解码成功时必须断言
   `genesisHashHex` / `expectedIdentityVersion` / `authorizationExpiresAt` 三个字段
   原样解出，而不只是"跳过了外层字节"。
