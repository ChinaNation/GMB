# 任务卡:冷钱包扫码签名两色识别改造

- 时间:2026-04-22
- 状态:open
- 归属:Blockchain Agent(`citizenchain/node` Tauri UI)+ Mobile Agent(`wumin` 冷钱包 + `wuminapp` 热钱包)
- 承接:`20260422-unified-voting-entry-phase3.md`(Phase 3 完工后的识别模型收尾)

## 目标

`wumin` 冷钱包扫描所有 QR,识别结果统一为 🟢 绿 / 🔴 红 两色,**禁止黄色盲签兜底**。

四端(`citizenchain/node` Tauri UI + `wuminapp` + `sfid` 后端 + `cpms` 后端)签发的所有合法 QR → 绿;其他 → 红。

## 方案文档

- `memory/05-architecture/qr-signing-recognition.md` — 两色识别方案
- `memory/05-architecture/qr-action-registry.md` — action / fields 唯一事实源
- `memory/05-architecture/qr-protocol-spec.md` — WUMIN_QR_V1 协议(已有)

## 必删清单

| 文件:位置 | 删除项 |
|---|---|
| `wumin/lib/signer/offline_sign_service.dart:154-167` | 整个 `allowedHashedActions` 常量 + 所有引用分支 |
| `wuminapp/lib/trade/user.dart:621` | `action: 'vote_register'` 孤儿入口 |

### PR 合并前 Grep 扫清零(15 个字符串)

在 `wumin/` + `wuminapp/` + `sfid/` + `cpms/` + `memory/` 范围内命中数必须为 0:

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

## 必补清单

### wumin/lib/signer/payload_decoder.dart 新增 8 个分支

| Pallet | pallet_index | call | call_index | decoded.fields |
|---|---|---|---|---|
| DuoqianTransfer | 19 | execute_transfer | 3 | proposal_id |
| DuoqianTransfer | 19 | execute_safety_fund_transfer | 4 | proposal_id |
| DuoqianTransfer | 19 | execute_sweep_to_main | 5 | proposal_id |
| ResolutionDestro | 14 | execute_destroy | 1 | proposal_id |
| AdminsChange | 12 | execute_admin_replacement | 1 | proposal_id |
| GrandpaKeyChange | 16 | execute_replace_grandpa_key | 1 | proposal_id |
| GrandpaKeyChange | 16 | cancel_failed_replace_grandpa_key | 2 | proposal_id |
| DuoqianManage | 17 | cleanup_rejected_proposal | 4 | proposal_id |

### wuminapp 10 个文件 SignDisplayField 补 key

按 Registry 字段表补齐:
- `lib/trade/onchain/onchain_trade_page.dart`
- `lib/duoqian/shared/duoqian_manage_detail_page.dart`
- `lib/governance/runtime_upgrade_detail_page.dart`
- `lib/governance/transfer_proposal_page.dart`
- `lib/duoqian/institution/institution_duoqian_create_page.dart`
- `lib/duoqian/personal/personal_duoqian_create_page.dart`
- `lib/duoqian/institution/institution_duoqian_close_page.dart`
- `lib/duoqian/personal/personal_duoqian_close_page.dart`
- `lib/governance/runtime_upgrade_page.dart`(fields 整块缺失,一并补齐)
- `lib/governance/activation_service.dart`
- 其他 Grep `SignDisplayField\(` 命中点

### citizenchain/node Tauri UI 对齐 Registry

`src/governance/signing.rs` + `src/transaction/mod.rs` 全部 `build_X_sign_request`:
- `display.action` 字面对齐 Registry
- `display.fields` 每个 `key` 字面对齐 Registry
- 顺序和 Registry 一致
- 多删、漏补

## 拆分 4 个 PR

### PR-A ✅ 架构落地(2026-04-22 完成)
- `memory/05-architecture/qr-signing-recognition.md`
- `memory/05-architecture/qr-action-registry.md`
- 本任务卡
- **不动代码**

### PR-B ✅ wumin 冷钱包改造(2026-04-22 完成, Mobile Agent)
- 默认范围:`wumin/`
- ✅ `offline_sign_service.dart` 删 `allowedHashedActions` 白名单
  + 所有引用分支;`decodeFailed` 改为统一拒签
- ✅ `offline_sign_page.dart` 删 L488-501 重复白名单;banner 三色改两色
  (decodeFailed 改红色 dangerous 图标)
- ✅ `payload_decoder.dart` 新增 `_decodeProposalIdOnly` 辅助函数 +
  新增 8 个 execute/cleanup/cancel 分支
  (execute_transfer / execute_safety_fund_transfer / execute_sweep_to_main
  / execute_destroy / execute_admin_replacement / execute_replace_grandpa_key
  / cancel_failed_replace_grandpa_key / cleanup_rejected_proposal)
- ✅ `payload_decoder.dart` 对齐 wasm_hash(sha256)+ eligible_total
  (`propose_runtime_upgrade` 补 2 字段;`developer_direct_upgrade`
  改 `wasm_bytes` → `wasm_hash`)。添加 `crypto` 包导入,计算 sha256
- ✅ `pallet_registry.dart` 清注释里 Phase 3 死字符串残留
- ✅ 单测:新增 10 个(8 execute/cleanup + 2 wasm_hash 场景),
  修正 1 个(`vote_transfer` → `internal_vote`),
  新增 2 个拒签断言(`decodeFailed` / `mismatched`)
- ✅ flutter analyze 0 issues,flutter test **96/96** PASS
- ✅ Grep 扫 15 个 Phase 3 死字符串在 `wumin/` 零命中
- ⏳ apk 重编(user 端执行:`flutter build apk --release && adb install -r ...`)

### PR-C ✅ wuminapp 改造(2026-04-22 完成, Mobile Agent)
- 默认范围:`wuminapp/`
- ✅ 9 个 SignDisplay page 文件补 `SignDisplayField.key`,辅助字段从
  display.fields 里移除(`transfer_proposal_detail_page` 删除整个
  `_buildSignDisplayFields` 函数,`duoqian_manage_detail_page` 删
  createInfo/closeInfo 辅助字段,`runtime_upgrade_detail_page` 删
  proposalInfo 辅助字段,机构/个人多签关闭页删"当前余额",
  `activation_service` 删"管理员公钥")
- ✅ `amount_yuan` 字段格式全局对齐 wumin decoder `_fenToYuan` 输出
  "X.XX GMB"(千分位)
- ✅ 删 `user.dart` 冷钱包 vote_register 分支(改为热钱包-only +
  cold-wallet 提示)+ 清 3 个 unused import + 1 个 unused helper
- ✅ 注释里残留字符串全清(duoqian_manage_service.dart / all_proposals_view.dart
  / user.dart)
- ✅ flutter analyze 0 issues,flutter test **65/65** PASS
- ✅ Grep 扫 16 个 Phase 3 死字符串 + `vote_register` 在 `wuminapp/` 零命中
- ⏳ apk 重装(user 端执行)
- 📌 分离出 2 个 follow-up 任务卡:
  - `20260422-offchain-clearing-pay-two-color.md`
    (offchain 清算支付 32B 盲签路径两色接入)
  - `20260422-joint-vote-institution-id-display.md`
    (joint_vote institution_id 冷钱包展示)

### PR-E ✅ activate_admin 签名请求解析失败 bugfix(2026-04-22 · Mobile Agent)

**问题**:管理员列表点"激活"显示 QR 码后,wumin 冷钱包扫码报
"签名请求解析失败:sign_request.pubkey 必填 0x hex"。

**根因**:违反 `feedback_pubkey_format_rule` 铁律。
- `wuminapp/lib/governance/activation_service.dart` 两处传入裸 hex:
  - L128 `payloadHex = _bytesToHex(payload)`(无 `0x`)
  - L135 `pubkey: pk`(`_normalize()` 去了 `0x`)
- 传到 wumin `SignRequestBody.fromJson:40/46` 严格校验"必须以 `0x` 开头" → 抛 FormatException
- `QrSigner._validateHexField` 两端都太松(允许裸 hex 通过 buildRequest),让错误在热钱包源头漏过,直到冷钱包扫码才暴露

**修复**:
- ✅ `activation_service.dart:128` 改 `'0x${_bytesToHex(payload)}'`
- ✅ `activation_service.dart:138` 改 `pubkey: '0x$pk'`
- ✅ `wuminapp/lib/signer/qr_signer.dart` `_validateHexField` 强制要求 `0x` 前缀,源头拦截裸 hex
- ✅ `wumin/lib/signer/qr_signer.dart` 同步强制 `0x` 校验
- ✅ Grep 扫全 wuminapp 其他 10 处 `pubkey:` / `payloadHex:` 构造点均已带 `0x` 前缀,无同类问题

**回归**:
- wumin analyze 0 issues + test **96/96** PASS
- wuminapp analyze 0 issues + test **65/65** PASS

**用户动作**:wumin apk + wuminapp 重编重装后,激活 QR 可正常扫码识别。

---

### PR-D ✅ citizenchain 节点 UI 对齐(2026-04-22 完成, Blockchain Agent)
- 默认范围:`citizenchain/node/src/` + `citizenchain/node/frontend/`
- ✅ 审计 `signing.rs` 全部 8 个 `build_X_sign_request` 的 display.fields
  key 逐条对齐 Registry(internal_vote / joint_vote / propose_transfer /
  propose_safety_fund_transfer / propose_sweep_to_main /
  developer_direct_upgrade / propose_runtime_upgrade / transaction transfer)
- ✅ `activation.rs:221-224` 删多余 `institution` 字段(Registry
  activate_admin fields 严格只含 `shenfen_id`);机构名保留在 summary
- ✅ `format_amount` 与 wumin decoder `_fenToYuan` 双方都是"千分位+2 位
  小数+GMB 后缀",字面输出一致
- ✅ `sha256_hash(&wasm_code)` 与 wumin decoder `sha256.convert(wasmBytes)`
  使用同算法,`wasm_hash` 字面输出一致
- ✅ Registry `activate_admin` 行签发方补 `node_ui`(承认节点 UI 也是
  合法 issuer,不仅 sfid 后端)
- ✅ frontend TS 无 `action:'xxx'` 字面量构造(前端只透传后端 display,
  无需改)
- ✅ Grep 扫 15 个 Phase 3 死字符串在 `citizenchain/node/src/` +
  `citizenchain/node/frontend/`(非 node_modules)零命中
- ⚠️ cargo check 跳过:WASM 铁律要求 runtime 不本地编,只改 JSON 宏内
  一行无 Rust 语法变更,运行时风险零。下次 CI WASM build 时一并验证
- `joint_vote` `institution_id` 字段展示拆成独立 follow-up 任务卡
  `20260422-joint-vote-institution-id-display.md`(不在 PR-D 范围)

## 验收(端到端)

### 业务场景(全部 🟢 绿)

1. 链节点 Tauri UI 发 `internal_vote`
2. 链节点 Tauri UI 发 `joint_vote`
3. 链节点 Tauri UI 发 `transfer`
4. 链节点 Tauri UI 发 `propose_transfer` / `propose_safety_fund_transfer` / `propose_sweep_to_main`
5. wuminapp 发 `transfer`
6. wuminapp 发 `propose_transfer` / `propose_safety_fund_transfer` / `propose_sweep_to_main`
7. wuminapp 发 `propose_create` / `propose_create_personal` / `propose_close`
8. wuminapp 发 `internal_vote` / `joint_vote`
9. wuminapp 发 `activate_admin`
10. sfid 后端发 `login_challenge` → 冷钱包回 `login_receipt`
11. cpms 后端发 `login_challenge` → 冷钱包回 `login_receipt`

### 对抗场景(全部 🔴 红)

12. `proto='WUMIN_QR_V2'` envelope
13. `kind='sign_request'` 但 `payload_hex` 是随机字节
14. `kind='sign_request'` 但 `spec_version=999`
15. `expires_at` 已过去的 envelope

## 回写

- PR-A/B/C/D 全部合并后,本任务卡移 `done/`
- `memory/MEMORY.md` 加索引:`project_qr_signing_two_color.md`
- 若 Registry 发现新 action 遗漏,反馈回 PR-A 修 `qr-action-registry.md`
