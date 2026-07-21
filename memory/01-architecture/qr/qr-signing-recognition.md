# QR_V1 扫码签名两色识别方案

- 更新日期:2026-07-19
- 状态:当前详细事实源,由 `memory/07-ai/unified-protocols.md` 统一管辖
- 范围:CitizenWallet / CitizenApp 扫描 QR 后的识别、展示、签名放行和签名响应验签规则
- 依赖:
  - `memory/01-architecture/qr/qr-protocol-spec.md`
  - `memory/01-architecture/qr/qr-action-registry.md`

## 1. 两色终态

| 颜色 | 含义 | 行为 |
|---|---|---|
| 绿色 | QR 结构合法,action 已登记,`review_payload` 可独立解码,动作/字段/枚举值已完整中文翻译,签名者匹配 | 允许签名或提交 |
| 红色 | 任一校验、解码、匹配或中文翻译失败 | 禁止签名或提交 |

扫码端不得因为 QR 里有文字摘要而放行。QR_V1 不携带摘要;所有展示内容必须由本地 decoder 从 `a + d` 推导。

签名判定只有 `Normal/正常` 与 `Reject/拒绝` 两种结果。不得出现未知、警告、部分识别、可忽略、继续确认等第三状态。红色拒绝状态不得显示可触发签名的按钮或入口。

## 2. k=1 签名请求校验

进入签名页前必须全部通过:

1. `p == QR_V1`
2. `k == 1`
3. `i` 合法且未过期
4. `b.a` 是已登记动作码
5. `b.g == 1`
6. `b.u` 解码为 32 字节公钥
7. `b.d` 解码为非空 `review_payload`
8. 当前钱包公钥等于 `b.u`

业务识别:

1. 对链交易 action,`a` 必须等于 `review_payload` 解码出的 pallet/call 动作码。
2. 对文本/二进制专用 action,decoder 必须识别 payload domain 或固定前缀。
3. Runtime 升级哈希直签只允许 32B payload,且 action registry 必须标记 `hash_only_allowed=true`。
4. decoder 失败或 `a` 与 payload 不一致,红色拒签。
5. action 名、字段名或枚举值缺少中文翻译,红色拒签。
6. 普通链交易 `b.d` 只有 32B signing bytes 且不能完整解码,红色拒签。
7. 固定展示值必须来自 `qr-protocol/registry/fields.yaml` 的 `field_value_zh`；decoder 只允许填动态变量，不允许在移动端另写“默认岗位”“制度账户”“费用付款账户”等第二套展示值。

签名字节:

1. 链交易 `review_payload` 必须来自生成方的当前 runtime `SignedPayload` 原始三元组 SCALE 字节。
2. 链交易 `review_payload` 长度 ≤256B:签 payload 原文。
3. 链交易 `review_payload` 长度 >256B:签 `blake2_256(review_payload)`。
4. `a=2 citizen_identity`:签 `blake2_256(GMB || 0x10 || VotingIdentityPayload SCALE bytes)`。
5. 其它非链文本/二进制 payload:签原文。
6. Runtime hash-only:签同一 runtime `SignedPayload::using_encoded` 得到的 32B signing bytes 原文。

## 3. k=2 签名响应校验

生成方收到签名响应后必须从本地 session 取回原请求,不得信任签名响应携带的业务信息。

1. `p == QR_V1`
2. `k == 2`
3. `i == session.request_id`
4. `e` 未过期
5. `b.u == session.expected_pubkey`
6. `b.s` 解码为 64 字节
7. 生成方用本地 session 重新计算 `review_payload` hash,必须等于 session.expected_payload_hash
8. 按 session 的 `a + review_payload` 计算签名字节后 sr25519 验签通过

CitizenWallet 对同一已扫描请求只允许调用一次钱包密钥：签名进行中时忽略重复点击，首个响应二维码生成后彻底禁用再次签名。同一业务操作不叠加“字段确认签名”“周期确认签名”或其它第二签名。

签名响应中不得出现 payload、payload hash、签名时间、摘要字段。若出现旧字段,解析器应报错。

citizenchain node 的链交易冷签路径和热钱包路径必须统一调用 `citizenchain/crates/chain-signing` 构造 runtime `TxExtension`、`SignedPayload`、`UncheckedExtrinsic`。普通链交易 QR 只放完整 `review_payload`，实际签名字节由同一真源按 Substrate `using_encoded` 规则计算。任何模块手写拼接 SCALE 字节都属于第二真源,会导致扫码端绿签但链端 `BadProof`。

## 4. 登录签名

登录不再有独立 QR kind。CID 生成 `k=1,a=1`:

| 字段 | 规则 |
|---|---|
| `b.u` | 系统公钥 |
| `b.d` | UTF-8 `system|system_signature` |
| 用户签名 | CitizenWallet 对 `b.d` 原文字节签名 |
| 签名响应 | `k=2` 的 `u/s` |

系统签名原文使用 `QR_V1|1|i|system|e|system_pubkey_without_0x`。

## 5. OnChina 非链载荷

| action | domain / 前缀 | 必须展示 |
|---:|---|---|
| 2 | `VotingIdentityPayload` SCALE bytes | 身份CID、钱包地址、年龄、有效期、公民状态、居住地 |
| 3 | `onchina_admin_governance` | 动作类型、注册局、省份、操作者/目标账户 |

文本载荷内部的 `payload_hash` 只用于生成方本地 session 或 API 防重放,不进入 QR 签名响应。

## 6. 应绿色通过

1. `a=1` 登录
2. `a=2` 公民链上身份确认
3. `a=3` OnChina 管理员治理冷钱包确认
4. `a=5/6` 管理员激活/解密
5. `a=7` Runtime 32B hash
6. 所有 `qr-action-registry.md` 登记的链交易 action

## 7. 应红色拒绝

1. 旧协议名或旧字段名。
2. 未登记 `k` 或 `a`。
3. 过期请求。
4. 当前钱包公钥与 `b.u` 不一致。
5. `a` 与 payload 解码出的动作不一致。
6. payload 无法解码。
7. action、字段或枚举值缺少中文翻译。
8. 普通交易 QR 只携带 32B hash/signing bytes,导致扫码端无法完整中文展示。
9. 链 payload >256B 却签原文而不是 `blake2_256(payload)`。
10. UI 只能显示动作数字、英文 action key、原始 hex 或“载荷 N 字节”。
11. 生成方重算的 session payload hash 与请求保存值不一致。
12. 签名响应签名校验失败。

## 8. 已拒绝方案

1. QR 内携带 `display` 摘要。会导致第二真源和伪造摘要风险,已删除。
2. 登录独立 kind。会增加协议分支,已统一为 `k=1,a=1`。
3. 旧协议兼容解析。会让同一扫码链路出现两套真源,禁止恢复。
4. 黄色/警告/未知但允许签名。会形成第三状态,禁止恢复。

## 9. CitizenWallet 当前落地状态

公民钱包扫码签名入口已落地两态模型:

```text
SignDecisionStatus.normal = 绿色,允许签名
SignDecisionStatus.reject = 红色,禁止签名
```

钱包侧不得再恢复 `matched / mismatched / decodeFailed` 等第三状态。任一 action 未登记、动作缺少中文名、payload 无法解码、字段缺少中文名、普通链交易只携带 32B hash/signing bytes,均返回 `reject` 并禁用签名按钮。

管理员人员载荷按目标类型严格分流：公权只接受 `admin_account + cid_number + family_name + given_name`，身份字段当前允许为空、非空 CID 必须为 CTZN 结构；私权机构与个人多签只接受 `admin_account + family_name + given_name` 且姓名非空。旧纯账户、旧合并姓名、非法 UTF-8 和重复账户一律红色拒签。

已落地文件:

- `citizenwallet/lib/signer/offline_sign_service.dart`
- `citizenwallet/lib/ui/offline_sign_page.dart`
- `citizenwallet/lib/signer/action_labels.dart`
- `citizenwallet/lib/signer/field_labels.dart`
- `citizenwallet/lib/signer/payload_decoder.dart`

## 10. CitizenApp 当前落地状态

公民端当前承担两类 QR 签名职责:

1. 生成普通链交易/身份确认签名请求,交由公民钱包扫码签名。
2. 作为热钱包签名方处理广场账户动作 `square_account_action`。

已落地规则:

- `citizenapp/lib/qr/generated/qr_action_registry.g.dart` 是由 `qr-protocol` 生成的 action/字段中文产物；`citizenapp/lib/qr/qr_protocols.dart` 只消费生成产物并保留调用方常量别名。
- `citizenapp/lib/signer/square_action_payload.dart` 对广场账户 payload 做本地解码,字段名必须来自生成产物。未知子动作、布局错误、字段中文缺失均返回 null。
- `citizenapp/lib/signer/square_action_sign_service.dart` 在触发钱包私钥签名前完成 action 登记、中文动作名、payload 中文展示、账户匹配和冷钱包边界校验。任一失败只返回拒绝,不得触发签名。
- `citizenapp/lib/qr/scan_dispatch_flow.dart` 和 `citizenapp/lib/qr/pages/qr_sign_response_page.dart` 只展示中文动作名和中文字段列表,不得恢复英文 action key、动作数字或原始 payload 展示。

公民端广场动作签名不使用 CitizenWallet 的离线 decoder；但两端都必须以 `citizenchain/crates/qr-protocol/registry/*` 为唯一登记真源同步 action code 和中文文案。当前 Dart 产物由 `citizenchain/crates/qr-protocol/src/bin/export_registry.rs` 生成，禁止恢复移动端手写 action/字段表。
