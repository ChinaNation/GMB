# SFID step2e:冷钱包签 4 个 chain push extrinsic 端到端通路

- 状态:open
- 创建日期:2026-05-02
- 模块:`sfid/backend` + `wumin` + `wuminapp`
- 关联 ADR:`memory/04-decisions/ADR-008-sheng-admin-3tier-and-key-admin-removal.md`
- 上游:phase7(commit 81cde87)+ step2d(commit b4bb76e)
- 跨模块协议触发:`memory/07-ai/chat-protocol.md` 第 5 条(runtime 凭证签名/验签改动必须双端联动)

## 任务需求

phase7 后 SFID 后端推 4 个 unsigned extrinsic 用 `sheng_signer_cache` 假签;链端 ValidateUnsigned 要求 sig 由 `ShengAdmins[province][slot]` 对应的 admin 私钥签发(冷钱包独占)。本卡接通**扫码签名通路**:SFID 后端 prepare payload → wuminapp 显示 QR → wumin 冷钱包扫码签 → wuminapp 扫回 sig → SFID 后端 submit-sig 真推链。

## 4 个 extrinsic(pallet_sfid_system,pallet_index=10)

| call_index | 名 | payload 字段(对齐 step2a domain 常量与顺序)|
|---|---|---|
| 2 | `add_sheng_admin_backup` | province / slot(1B 0/1/2)/ new_pubkey [u8;32] / nonce [u8;32] |
| 3 | `remove_sheng_admin_backup` | province / slot / nonce |
| 4 | `activate_sheng_signing_pubkey` | province / admin_pubkey [u8;32] / signing_pubkey [u8;32] / nonce [u8;32] |
| 5 | `rotate_sheng_signing_pubkey` | province / admin_pubkey / new_signing_pubkey / nonce |

domain 常量(step2a 已固化):
- `b"add_sheng_admin_backup_v1"` / `b"remove_sheng_admin_backup_v1"`
- `b"activate_sheng_signing_pubkey_v1"` / `b"rotate_sheng_signing_pubkey_v1"`

签名 payload:`domain || province.encode() || ...其他字段 || nonce`,然后 `blake2_256` → sr25519。

## 改造范围(三方协同)

### A. SFID 后端 — 4 endpoint 拆双步

每个 chain push 流程改为 prepare + submit-sig 两步:

```
旧:
POST /api/v1/admin/sheng-admin/roster/add-backup     (单步,假签推链)

新:
POST /api/v1/admin/sheng-admin/roster/prepare-add-backup
   → 200 { payload_hex, nonce_hex, qr_url, expires_at }
POST /api/v1/admin/sheng-admin/roster/submit-add-backup-sig
   → 入参 { nonce_hex, sig_hex }
   → 200 { tx_hash }
```

四组(add-backup / remove-backup / activate / rotate)都拆。

`chain/sheng_admin/{add_backup,remove_backup}.rs` + `chain/sheng_signer/{activation,rotation}.rs`:
- 拆原 `add_backup(...)` 函数为 `prepare_add_backup(...)`(返回 payload + nonce)+ `submit_add_backup_sig(payload_id, sig)`(实际推链)
- nonce 缓存(`storage/sheng_pending_signs/<nonce_hex>.json`):TTL 5 分钟,落盘防止进程重启丢失
- `submit-X-sig` 接到 sig → 拼回 unsigned extrinsic → 调 phase7 `submit_immortal_paysno` 真推链

### B. wumin 冷钱包 — decoder + 4 sign body

`lib/signer/payload_decoder.dart` 加 4 分支:
- pallet_index=10 + call_index 2/3/4/5
- 解出字段并展示中文 label(action_labels.dart 加 4 标签)
- admin 私钥签 → 输出 sig QR

`lib/qr/bodies/` 加 4 个 SignRequest body type(若 envelope 已支持泛型,可复用)。

### C. wuminapp 在线端 — 管理后台 + 扫码桥接

`lib/admin/sheng_admin_console/`(新建)或就近:
- 名册管理页:链上 ShengAdmins[province][3 slot] 状态展示
- "添加 Backup" 按钮:输入 backup admin pubkey → 调 SFID `prepare-add-backup` → 显示 QR → wumin 扫码签 → 扫回 sig → 调 SFID `submit-sig`
- "激活签名密钥" / "rotate" / "remove backup" 同流程

## 关键约束

- `feedback_qr_signing_two_color.md`:冷钱包必须能解出全部字段(绿色),禁止白盲签
- `feedback_no_compatibility.md`:不留 phase7 假签兜底
- `feedback_pubkey_format_rule.md`:0x 小写 hex
- `feedback_scale_domain_must_be_array.md`:domain 常量数组
- 死规则:文档/注释/残留三件套

## 验收

### SFID 后端

- `cargo check -p sfid-backend` 全绿
- `cargo test -p sfid-backend` ≥ baseline 79 + 6 新测试(prepare 返回一致性 / submit-sig 校验 / nonce 重放拒绝 / TTL 过期 / 并发 / 4 endpoint 各 1)
- 残留 grep:phase7 假签 `signing_pair` 在 chain/sheng_admin/* + chain/sheng_signer/* = 0

### wumin

- `flutter analyze` 0 issues
- `flutter test` ≥ baseline 107 + 8 新测试(每分支 1 decode + 1 sign roundtrip)
- decoder 4 分支识别 + 中文 label

### wuminapp

- `flutter analyze` 0 issues
- `flutter test` ≥ baseline 112 + 6 新测试
- 管理后台 widget tests 通过
- 扫码桥接:模拟 wumin sig QR → 调 SFID submit-sig

### 双端字节一致性

- 4 fixture 凭证(每 extrinsic 1 组)记录:wumin / wuminapp / SFID 后端 / 链端 4 处 SCALE 字节序列对齐

## 工作量

- SFID 后端 4 endpoint 拆双端 + nonce 落盘:~300 行 + 6 测试
- wumin decoder + 4 sign body:~250 行 + 8 测试
- wuminapp 管理后台 + 扫码桥接:~400 行 + 6 测试
- 共 ~950 行 + 20 测试,**~3 agent rounds**(可并发派,但 SCALE 字节对齐后序合)

## 提交策略

- 三端同 PR 提交;或严格按"先 SFID 后端 → 再 wumin → 再 wuminapp"顺序合并
- 任何中间快照不得出现 4 endpoint 的"假签 / 真签"混合状态
