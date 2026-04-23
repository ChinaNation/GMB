# 冷钱包扫码签名两色识别方案

- 版本:2026-04-22
- 状态:唯一事实源(Single Source of Truth)
- 范围:wumin 冷钱包扫描 QR 的识别与签名颜色判定
- 依赖:
  - `memory/05-architecture/qr-protocol-spec.md` — WUMIN_QR_V1 协议定义
  - `memory/05-architecture/qr-protocol-fixtures/` — 七种 kind 的 golden JSON
  - `memory/05-architecture/qr-action-registry.md` — action / field 事实源

## 一、四条铁律

1. **WUMIN_QR_V1 是唯一协议字符串**。冷钱包扫到非此协议、非七种 kind 之一、或过期的 QR,直接拒签(🔴 红)。
2. **两色终态**:🟢 绿(识别通过,允许签名)/ 🔴 红(识别失败,禁止签名)。**禁止黄色盲签兜底**。
3. **"识别" = 结构识别**。envelope 合法 + payload 可完整解码展示 → 绿;其他 → 红。这是 Substrate 冷钱包的安全模型天花板:冷钱包只负责 "让用户看清要签什么",不负责 "QR 来路对不对"。客户端程序(node 桌面 exe / wuminapp 手机 apk)反编译即可泄漏任何 issuer 私钥,协议层无法做密码学 provenance。
4. **不残留、不兼容、不过渡**。所有 Phase 3 已删字符串、`allowedHashedActions` 白名单、孤儿 action 必须一次性清除,不保留任何过渡逻辑(`feedback_no_compatibility`)。

## 二、冷钱包扫描的 kind

按 `qr-protocol-spec.md` §3 的 7 种 kind,冷钱包仅消费:

| 扫入的 kind | 冷钱包产出 |
|---|---|
| `login_challenge` | `login_receipt` |
| `sign_request` | `sign_response` |

其他 5 种 kind(`login_receipt` / `sign_response` / `user_contact` / `user_transfer` / `user_duoqian`)不由冷钱包消费,冷钱包扫到视为不在职责范围 → 🔴 红。

## 三、识别路径

### 3.1 envelope 层前置校验(两种 kind 共享)

全部为真才进下一步:

- `envelope.proto == 'WUMIN_QR_V1'`
- `envelope.kind ∈ {login_challenge, sign_request}`
- `envelope.id` 长度 16-128,字符集 `[a-zA-Z0-9_-]`
- `envelope.issued_at` 与 `envelope.expires_at` 均存在(临时码必填)
- `now <= envelope.expires_at`
- `envelope.body` 字段集完全匹配 spec §4 该 kind 的定义,**无多余字段、无别名**

任一失败 → 🔴 红。

### 3.2 kind = login_challenge

补充校验:

- `body.system` 非空字符串
- `body.sys_pubkey` 是合法 `0x<64hex>` (32 字节 sr25519 公钥)
- `body.sys_sig` 是合法 `0x<128hex>` (64 字节 sr25519 签名)
- 按 spec §5 拼接签名原文,用 `sys_pubkey` 验证 `sys_sig` 通过

全过 → 🟢 绿,允许生成 `login_receipt`。
UI 必须醒目展示 `system` 字段值(`"sfid"` / `"cpms"` / ...),由用户自己判断是否是本人预期登录的系统。

### 3.3 kind = sign_request

补充校验:

- `body.address` (SS58,prefix=2027)、`body.pubkey` (`0x<64hex>`)、`body.sig_alg == 'sr25519'`、`body.payload_hex` (`0x<hex>`) 均非空
- `body.spec_version` 在 `wumin/lib/signer/pallet_registry.dart` 的 `supportedSpecVersions` 集合内
- `payload_decoder.decode(payload_hex, spec_version)` 返回 `decoded != null`
- `decoded.action == body.display.action`(**字面相等,无大小写变体,无别名**)
- 对 `decoded.fields` 每一项 `(key, value)`,在 `body.display.fields` 中按同名 `key` 查到的 value,与 decoded 侧 value **字面相等**

全过 → 🟢 绿,允许生成 `sign_response`。

## 四、action / fields 对齐规则

唯一事实源:`memory/05-architecture/qr-action-registry.md`

任何一端(`citizenchain/node` Tauri UI / `wuminapp` / `wumin` decoder)新增或修改 action / field key,必须:

1. 先改 Registry
2. 再改三端代码(decoder + 签发方)
3. 补 golden fixture + 端到端测试
4. CI 门禁扫 Registry vs 三端实现

## 五、必删残留清单

### 5.1 wumin 冷钱包

| 文件 | 删除项 |
|---|---|
| `wumin/lib/signer/offline_sign_service.dart:154-167` | 整个 `allowedHashedActions` 常量 |
| 同文件所有位置 | 所有引用 `allowedHashedActions` 的判断分支 |
| 同文件 | 所有 "decode 失败但允许签名" 的代码路径 |

### 5.2 wuminapp

| 文件:位置 | 删除项 |
|---|---|
| `wuminapp/lib/trade/user.dart:621` | `action: 'vote_register'` 孤儿入口(无链上 extrinsic、无 decoder 分支) |

### 5.3 PR 合并前 Grep 扫清零

以下 15 个 Phase 3 已废字符串,在 `wumin/` + `wuminapp/` + `sfid/` + `cpms/` + `memory/` 命中数必须为 0 才能合并:

```
vote_create
vote_transfer
vote_safety_fund_transfer
vote_sweep_to_main
finalize_create
finalize_transfer
finalize_safety_fund
finalize_sweep
OP_SIGN_CREATE
OP_SIGN_TRANSFER
OP_SIGN_SAFETY_FUND
OP_SIGN_SWEEP
CreateVoteIntent
TransferVoteIntent
make_transfer_sigs
```

(链端 `citizenchain/runtime` 的残留已在 Phase 3 清理,此次扫描不含 runtime Rust 源码。)

## 六、必补清单

### 6.1 payload_decoder 新增 8 个分支(PR-B 已落地 2026-04-22)

`wumin/lib/signer/payload_decoder.dart` 对照 Phase 3 链端所有"人工可触发" extrinsic:

| Pallet | pallet_index | call | call_index | decoded.fields |
|---|---|---|---|---|
| DuoqianTransferPow | 19 | execute_transfer | 3 | proposal_id |
| DuoqianTransferPow | 19 | execute_safety_fund_transfer | 4 | proposal_id |
| DuoqianTransferPow | 19 | execute_sweep_to_main | 5 | proposal_id |
| ResolutionDestroGov | 14 | execute_destroy | 1 | proposal_id |
| AdminsOriginGov | 12 | execute_admin_replacement | 1 | proposal_id |
| GrandpaKeyGov | 16 | execute_replace_grandpa_key | 1 | proposal_id |
| GrandpaKeyGov | 16 | cancel_failed_replace_grandpa_key | 2 | proposal_id |
| DuoqianManagePow | 17 | cleanup_rejected_proposal | 4 | proposal_id |

(`register_sfid_institution` 由 sfid 后端 ShengSigningPubkey 直签,不走冷钱包 ⇒ 不补 decoder。)

### 6.1.1 propose_runtime_upgrade / developer_direct_upgrade 字段对齐(PR-B 已落地)

Registry 增补 `wasm_hash` + `eligible_total` 字段,三端共用 **sha256(wasm_bytes)**
(不是 blake2_256)。冷钱包 decoder 用 `package:crypto` 的 `sha256.convert()`,
节点 Tauri UI 用 `sha256_hash(&wasm_code)`,两边输出逐字相等 → 🟢 绿色识别。

### 6.2 wuminapp SignDisplayField 补 key

10 个文件的所有 `SignDisplayField` 构造必须填 `key` 参数,按 Registry 字段表:

- `lib/trade/onchain/onchain_trade_page.dart`
- `lib/governance/duoqian_manage_detail_page.dart`
- `lib/governance/runtime_upgrade_detail_page.dart`
- `lib/governance/transfer_proposal_page.dart`
- `lib/governance/duoqian_create_proposal_page.dart`
- `lib/governance/personal_duoqian_create_page.dart`
- `lib/governance/duoqian_close_proposal_page.dart`
- `lib/governance/runtime_upgrade_page.dart`(当前整块 display.fields 为空,一并补齐)
- `lib/governance/activation_service.dart`
- 其他 Grep `SignDisplayField` 命中点

未填 key 视为 bug,PR 合并前 Grep `SignDisplayField\(` 每处 review。

### 6.3 citizenchain/node Tauri UI 对齐 Registry

审计 `src/ui/governance/signing.rs` + `src/ui/transaction/mod.rs` 全部 `build_X_sign_request` 函数,与 Registry 逐条对齐:

- `display.action` 字面相等
- `display.fields` 每个 `key` 字面相等
- 顺序和 Registry 一致
- 多删、漏补

## 七、实施步骤(拆 4 个 PR)

### PR-A 架构落地(本轮)
- `memory/05-architecture/qr-signing-recognition.md`(本文档)
- `memory/05-architecture/qr-action-registry.md`
- 任务卡 `memory/08-tasks/open/20260422-cold-wallet-two-color-recognition.md`
- **无代码改动**

### PR-B 冷钱包 wumin 改造(Mobile Agent)
- 删 `allowedHashedActions` + 所有引用
- `payload_decoder.dart` 新增 8 个分支
- 单测覆盖 8 个分支 + 三色判定
- apk 重编

### PR-C wuminapp 改造(Mobile Agent)
- 10 个文件 `SignDisplayField` 补 key
- 删 `vote_register` 孤儿入口
- 和 Registry 对齐 action / fields
- 合并前 Grep 扫 §5.3 的 15 个残留字符串

### PR-D 节点 UI 对齐(Blockchain Agent)
- `src/ui/governance/signing.rs` + `src/ui/transaction/mod.rs` 对齐 Registry
- 必要时补漏项
- 合并前 Grep 扫残留

## 八、验证清单(PR-B/C/D 全合并后)

### 业务场景(全部应 🟢 绿)

1. 链节点 Tauri UI 发起 `internal_vote`
2. 链节点 Tauri UI 发起 `joint_vote`
3. 链节点 Tauri UI 发起 `transfer` (Balances.transfer_keep_alive)
4. 链节点 Tauri UI 发起 `propose_transfer` / `propose_safety_fund_transfer` / `propose_sweep_to_main`
5. wuminapp 发起 `transfer`
6. wuminapp 发起 `propose_transfer` / `propose_safety_fund_transfer` / `propose_sweep_to_main`
7. wuminapp 发起 `propose_create` / `propose_create_personal` / `propose_close`
8. wuminapp 发起 `internal_vote` / `joint_vote`
9. wuminapp 发起 `activate_admin`
10. sfid 后端发起 `login_challenge` → 冷钱包回 `login_receipt`
11. cpms 后端发起 `login_challenge` → 冷钱包回 `login_receipt`

### 对抗场景(全部应 🔴 红)

12. 手工构造 `proto='WUMIN_QR_V2'` 的 envelope → 红
13. 手工构造 `kind='sign_request'` 但 `payload_hex` 是随机字节 → 红
14. 手工构造 `kind='sign_request'` 但 `spec_version=999` → 红
15. 手工构造 `expires_at` 已过去的 envelope → 红

## 九、拒绝的替代方案

### 9.1 加密 provenance (envelope 内嵌 issuer_sig)

- citizenchain/node Tauri UI (桌面 exe)、wuminapp (开源 apk) 都是客户端程序,反编译可直接拿到打包进去的 issuer 私钥
- sfid/cpms 后端虽可安全持有,但它们只发 `login_challenge`,不发 `sign_request`
- 四端无法共用同一套 provenance 模型,会退化成两套色判定,违反 "协议统一" 铁律
- **拒绝**

### 9.2 保留 `allowedHashedActions` 黄色盲签兜底

- 白名单靠 action 字符串命名约定猜来源,无任何密码学强度
- 攻击者构造 `display.action='propose_transfer'` 的垃圾 payload,当前走黄色兜底 → 用户盲签 → 实际资金损失
- 违反 `feedback_no_compatibility` 铁律
- **拒绝**

### 9.3 使用 `--- compatibility mode ---` 处理老版 apk

- 老版 apk 不在维护范围,Phase 3 完工后必须重编重装
- 任何兼容分支都是技术债滋生地
- **拒绝**
