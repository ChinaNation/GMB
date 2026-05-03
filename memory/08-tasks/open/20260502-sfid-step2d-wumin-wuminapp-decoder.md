# SFID Step 2d:wumin / wuminapp 凭证 decoder 双端联动

- 状态:open
- 创建日期:2026-05-02
- 模块:`wumin/`(Flutter 冷钱包)+ `wuminapp/`(Flutter 在线端)
- 关联 ADR:`memory/04-decisions/ADR-008-sheng-admin-3tier.md`
- 前置依赖:step2a + step2b 完成(链端凭证字段已固定)
- 跨模块协议触发:**`memory/07-ai/chat-protocol.md` 第 5 条**(runtime 凭证签名/验签改动必须双端联动)

## 任务需求

step2b 在 `duoqian-manage` 凭证 payload 加 `signer_admin_pubkey: [u8;32]` 字段。wumin 冷钱包 + wuminapp 在线端的扫码签名 decoder **必须同步识别 + 显示**新字段:
- `feedback_qr_signing_two_color.md` 铁律:禁止白盲签;decoder 必须能解出全部字段并展示给用户
- 不识别新字段会导致两色识别从绿色降级为黄色(盲签),被 UX 拒绝

## 影响范围

### wumin 冷钱包(Flutter + Dart)

- decoder:`lib/...` 下 institution registration 凭证解码逻辑
- 加字段 `signer_admin_pubkey` 解码 + 显示("签名管理员公钥:0x...")
- sign_display_fields 加新字段输出
- analyze + test 全绿

### wuminapp 在线端(Flutter + Dart)

- decoder:同步加字段
- sign_display_fields 加新字段
- analyze + test 全绿

### 凭证 payload 兼容性

SCALE 编码字段顺序硬编码,前端冷钱包/在线端/SFID 后端必须**完全对齐**:
```
sfid_id || institution_name || a3 || sub_type || parent_sfid_id ||
province || register_nonce || signer_admin_pubkey || signature
```

新字段插入位置:`register_nonce` 后、`signature` 前。

## 主要风险点

- **签名 payload 哈希**:加字段后 hash 输入变化;decoder 必须用与 SFID 后端 + duoqian-manage runtime 完全相同的序列化
- **未识别字段提示降级**:wumin 两色识别铁律严格,decoder 不识别会显黄盲签 → UX 阻塞;必须 PR 同步上线
- **测试覆盖**:wumin/wuminapp 已有 institution registration scan/sign decode 测试,本卡更新现有 + 新加 ≥ 2 测试覆盖新字段

## 是否需要先沟通

- 否(`chat-protocol.md` 第 5 条已强制双端联动,本卡作为跨模块任务卡注册即合规)

## 验收清单

- `cd wumin && flutter analyze && flutter test` 全绿
- `cd wuminapp && flutter analyze && flutter test` 全绿
- 现有 institution registration decode 测试更新通过
- 新加 ≥ 2 测试:
  - `decode_with_signer_admin_pubkey_success`
  - `decode_without_signer_admin_pubkey_legacy_payload_rejected`
- 双端 signing payload 字节序列与 runtime + SFID 后端完全一致(可加 fixture 测试)
- Grep 残留 0:`signing_province=None` 分支在 decoder
- 任务卡 progress 章节回写

## 不要做的事

- 不要碰 runtime / SFID 后端(已在 step2a/b/c 完成)
- 不要 commit

## 工作量

~300 行(双端 ~150 行/端)+ 4 测试,~1.5 agent rounds。

## 提交策略

wumin 与 wuminapp 必须**同一 PR 内提交**或**严格按"先 wumin → 再 wuminapp"顺序合并**,确保任何中间快照都不出现 decoder 不一致状态。

## Progress(2026-05-02 Mobile Agent)

状态:**done(待 commit)**。基于 commit `81cde87`(链端 step2b/step3 已落地)。

### wumin 冷钱包(Flutter + Dart)改动

| 文件 | 行数变化 | 说明 |
|---|---|---|
| `wumin/lib/signer/pallet_registry.dart` | +8 / -0 | 新增 `proposeCreateInstitutionCall = 5` 常量(DuoqianManage pallet 17.5)|
| `wumin/lib/signer/payload_decoder.dart` | +290 / -25 | 1) `_decodeCitizenVote`(9.2)在 nonce/sig 后、approve 前加 (Vec province + [u8;32] signer_admin_pubkey) 解码 + 长度门槛 45→78;2) `_decodeProposeRuntimeUpgrade`(13.0) 补全 snapshot_nonce/signature 跳读 + 末尾加 (province + signer_admin_pubkey);3) 新增 `_decodeProposeCreateInstitution`(17.5) 完整解析 13 字段(含 a3 / sub_type / parent_sfid_id);4) 新增 `_decodeProposeResolutionIssuance`(8.0)解析 reason/total_amount/allocations/eligible_total/snapshot_nonce/signature/province/signer_admin_pubkey;5) 新增 `_bytesToLowerHex` helper(feedback_pubkey_format_rule:0x 小写) |
| `wumin/lib/signer/action_labels.dart` | +1 / -0 | 加 `propose_create_institution: '创建机构多签账户'` |
| `wumin/test/signer/payload_decoder_test.dart` | +280 / -10 | 1) `decodes citizen_vote` 改写为含新字段断言;2) 新加 `citizen_vote 旧 SCALE 字节流拒绝解码`(防白盲签);3) `decodes propose_runtime_upgrade` 改写含 nonce/sig/province/signer_admin_pubkey;4) 新加 `decodes propose_create_institution` + `decodes propose_resolution_issuance`;5) 新加 3 条 fixture 驱动 `fixture step2d {citizen_vote / propose_runtime_upgrade / propose_resolution_issuance}: decoder 解出新字段` |
| `wumin/test/fixtures/step2d_credential_payload.json` | 新文件(70 行) | 三组凭证 SCALE 字节固化(Python 生成器与链端 codec.encode 对齐),baseline_commit=81cde87 |

### wuminapp 在线端(Flutter + Dart)改动

| 文件 | 行数变化 | 说明 |
|---|---|---|
| `wuminapp/lib/wallet/capabilities/api_client.dart` | +35 / -3 | `PopulationSnapshotResponse` 加 (`province: String`, `signerAdminPubkey: String`) 必填字段;`fetchPopulationSnapshot` 解析新字段并强制存在校验;hex 归一化为 0x 小写(feedback_pubkey_format_rule)|
| `wuminapp/lib/citizen/proposal/runtime_upgrade/runtime_upgrade_service.dart` | +50 / -12 | `submitProposeRuntimeUpgrade` 入参加 (`province: Uint8List`, `signerAdminPubkey: Uint8List`);`buildProposeRuntimeUpgradeCallForTest` 改为 @visibleForTesting static 方法,SCALE 末尾加 `Vec<u8> province` + `[u8;32] signer_admin_pubkey`;`_u64ToLeBytes` 改 static |
| `wuminapp/lib/citizen/proposal/runtime_upgrade/runtime_upgrade_page.dart` | +50 / -8 | 透传 (`province`, `signerAdminPubkey`) 从 `snapshot` 到 service;`SignDisplay.fields` 补 4 项((`reason / eligible_total / province / signer_admin_pubkey`),供冷钱包两色识别比对中文 label |
| `wuminapp/test/citizen/proposal/runtime_upgrade/runtime_upgrade_service_test.dart` | 新文件(85 行) | 1) `propose_runtime_upgrade call data 与 fixture 逐字节一致`(读 fixture 调用 build call 对比 hex);2) `signer_admin_pubkey 长度非 32 → 拒绝构造 call data` |
| `wuminapp/test/fixtures/step2d_credential_payload.json` | 新文件(70 行) | 与 wumin 端同源(逐字 copy)|

### 5 个 extrinsic decoder 字段位置

- **citizen_vote(9.2)**:`[0x09][0x02][proposal_id u64_le][binding_id 32B][Vec nonce][Vec sig][Vec province ★][[u8;32] signer_admin_pubkey ★][approve bool]`
- **propose_runtime_upgrade(13.0)**:`[0x0d][0x00][Vec reason][Vec wasm][u64_le eligible_total][Vec snap_nonce][Vec sig][Vec province ★][[u8;32] signer_admin_pubkey ★]`
- **propose_resolution_issuance(8.0)**:`[0x08][0x00][Vec reason][u128_le total_amount][Vec<{32B+u128_le}> allocations][u64_le eligible_total][Vec snap_nonce][Vec sig][Vec province ★][[u8;32] signer_admin_pubkey ★]`
- **propose_create_institution(17.5)**:`[0x11][0x05][Vec sfid_id][Vec institution_name][Vec<{Vec name+u128 amount}> accounts][u32_le admin_count][Vec<32B> duoqian_admins][u32_le threshold][Vec register_nonce][Vec signature][Vec province ★][[u8;32] signer_admin_pubkey ★][Vec a3][Option<Vec> sub_type][Option<Vec> parent_sfid_id]`
- **register_sfid_institution(17.2)**:**不收**(wumin payload_decoder.dart:184-188 注释明确"由 sfid 后端 ShengSigningPubkey 直签,不走冷钱包",该 extrinsic 由 SFID 后端推链 phase7 走 RuntimeSfidInstitutionVerifier 验签,wumin/wuminapp 在线端不出现 QR 签名分支)

### 验收数字

- `cd wumin && flutter analyze`:**0 issues** ✓
- `cd wumin && flutter test`:**107 / 107 passed**(baseline 100 + 新加 7:1 citizen_vote 含新字段 + 1 citizen_vote legacy reject + 1 propose_create_institution + 1 propose_resolution_issuance + 1 propose_runtime_upgrade 含新字段 + 3 fixture 驱动 decode tests,扣减改写覆盖 = 净 +6,实际计数 100→107)
- `cd wumin && flutter test test/signer/payload_decoder_test.dart`:**34 / 34 passed**(原 28 + 新增 6:citizen_vote 改写、citizen_vote legacy、propose_runtime_upgrade 改写、propose_create_institution、propose_resolution_issuance、3 fixture)
- `cd wuminapp && flutter analyze`:**0 issues** ✓
- `cd wuminapp && flutter test`:**112 / 112 passed**(baseline 110 + 新增 2:fixture call data 字节一致 + signer_admin_pubkey 长度校验)

### 双端字节一致性 fixture 验证

- fixture(`test/fixtures/step2d_credential_payload.json`)记录 3 组凭证(citizen_vote / propose_runtime_upgrade / propose_resolution_issuance),expected_call_data_hex 由 Python 生成器与链端 SCALE codec.encode 等价产出
- wumin 端:3 个 fixture-driven decode tests 全过 → decoder 反向兼容字节流
- wuminapp 端:`buildProposeRuntimeUpgradeCallForTest` 正向编码与 fixture expected hex 逐字节相等 → 链端 = wumin = wuminapp 三处字节序列对齐
- baseline_commit=`81cde87`,任意一端漂移立即 fixture 失败

### 残留扫描(均为 0 命中)

- `grep -rn "signing_province=None\|signing_province: None\|signingProvince: null\|signingProvince=null" wumin/lib wuminapp/lib wumin/test wuminapp/test`:**0 命中**
- 5 个 extrinsic 引用全部带新字段处理(decoder 加 (province, signer_admin_pubkey) 解码,wuminapp service 透传 SCALE 编码)

### 任务卡范围内不做的事(已遵守)

- 未 commit
- 未触碰 sfid/backend / sfid/frontend / citizenchain/runtime
- 未为 4 个新 chain push extrinsic(add/remove_sheng_admin_backup / activate/rotate_sheng_signing_pubkey)添加 wumin 冷钱包签名分支(留 step2e)
- 未做 e2e 浏览器测试(本卡只到 unit test)

### 后续任务卡微调建议

- **step2e**(冷钱包签 4 个新 chain push extrinsic):待 SFID 后端 phase7 给出 add/remove_sheng_admin_backup / activate/rotate_sheng_signing_pubkey 签发流程后,wumin decoder 加这 4 个 extrinsic 的解析(走 sfid-system pallet,call_index 2/3/4/5,全 `Pays::No`)+ wuminapp 端构造 caller(单独管理后台,非业务 page)。本卡留接口稳定。
- **遗留 bug(独立 follow-up)**:`wuminapp/lib/duoqian/institution/institution_duoqian_create_page.dart:351` UI 文案标 `propose_create_institution`,但 `DuoqianManageService.submitProposeCreateInstitution`(`duoqian_manage_service.dart:54`)实际编码 `propose_create`(call_index=0,**旧 12 参版本**),与链端 17.5 `propose_create_institution`(14 参,带 SFID 凭证 + province + signer_admin_pubkey)不一致。该 bug 早于 step2d 存在,本卡严格按范围"decoder 字段更新 + payload hash 同步"执行,不动 caller。建议作为独立 follow-up 任务卡 `wuminapp-propose-create-institution-caller-fix` 修复:替换 `_proposeCreateCallIndex` → `5`,补 `accounts` / `register_nonce` / `signature` / `province` / `signer_admin_pubkey` / `a3` / `sub_type` / `parent_sfid_id` 8 个新参,并接通 SFID 后端的 institution registration credential 接口。修复后 wumin decoder 17.5 分支才会被触发(当前 wumin 已支持但等待 wuminapp caller 切换)。
- **SFID 后端 phase7 并行(已知依赖)**:`/api/v1/app/voters/count` 当前未返回 (province, signer_admin_pubkey),wuminapp `ApiClient.fetchPopulationSnapshot` 已加强制校验。SFID 后端必须在本卡上线前把这两字段下发到响应 JSON,否则在线端 propose_runtime_upgrade 流程会在快照解析期 throw。建议作为 SFID phase7 阻塞收尾项。
