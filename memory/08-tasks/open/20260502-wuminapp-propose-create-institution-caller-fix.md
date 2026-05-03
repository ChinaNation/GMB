# wuminapp propose_create_institution caller 切到 call_index=5

- 状态:open
- 创建日期:2026-05-02
- 模块:`wuminapp`
- 关联 ADR:`memory/04-decisions/ADR-008-sheng-admin-3tier.md`
- 上游:step2d(commit b4bb76e),wumin decoder 已支持 17.5 但 wuminapp caller 仍调旧 17.0

## 任务需求

`wuminapp/lib/duoqian/institution/duoqian_manage_service.dart:54` 当前编码 `call_index=0`(`propose_create` 旧 12 参版),但链端 step2b 已升级到 `propose_create_institution` call_index=5 / 14 参(加 `province` + `signer_admin_pubkey`)。本卡切到新版本。

## 改造

### `duoqian_manage_service.dart`

- 常量 `_proposeCreateCallIndex = 0` → `_proposeCreateInstitutionCallIndex = 5`
- SCALE payload 14 字段顺序(对齐链端 step2b):
  ```
  sfid_id / institution_name / accounts / admin_count / duoqian_admins / threshold
  / register_nonce / signature / province ★新 / signer_admin_pubkey ★新
  / a3 / sub_type / parent_sfid_id
  ```
- 注意 `province + signer_admin_pubkey` 插入位置:`signature` 后、`a3` 前(对齐 step3 的 BindCredential 模式)

### `institution_duoqian_create_page.dart`

- 调用 service 时透传 `province` + `signerAdminPubkey`(从 SFID 后端 institution_credential 接口返回值取)

### `api_client.dart`(若需)

- institution_credential 响应解析加 `signer_admin_pubkey`(0x 小写 hex)

### SFID 后端(若 institution_credential 接口未返回 admin pubkey)

- `sfid/backend/chain/institution_info/handler.rs` 或 `chain/institution_info/dto.rs` 凭证响应加字段
- 数据来源:登录 session `unlocked_admin_pubkey`

## 验收

- `cd wuminapp && flutter analyze` 0 issues
- `cd wuminapp && flutter test` ≥ baseline 112 + 2 新测试
- 字节一致性:wuminapp call data hex == 链端 SCALE 编码 fixture
- wumin decoder 17.5 分支被实际触发(end-to-end 联调)

## 工作量

~150 行 + 2 测试,~0.5 round。

## Progress(2026-05-01)

### 实际改动文件

| 文件 | 关键变更 |
|---|---|
| `wuminapp/lib/duoqian/shared/duoqian_manage_service.dart` | 1) 常量 `_proposeCreateCallIndex = 0` → `_proposeCreateInstitutionCallIndex = 5`;2) `submitProposeCreateInstitution` 入参从 7 个扩到 14 个,删除 `accountName/amountFen` 单字段,改成 `institutionName + accounts: List<InstitutionInitialAccountInput>` + 6 个 SFID 凭证字段(register_nonce / signature / province / signer_admin_pubkey / a3 / sub_type / parent_sfid_id);3) 抽出静态 `@visibleForTesting` 工具 `buildProposeCreateInstitutionCallForTest` 纯函数式产 call data,与链端 step2b 14 参 SCALE 顺序逐字对齐;4) 新增 `InstitutionInitialAccountInput` 入参 DTO(对应链端 `InstitutionInitialAccount<AccountName, Balance>`);5) `signer_admin_pubkey` 长度强校验 `[u8;32]`,违反 throw `ArgumentError`(避免 decoder 17.5 落黄牌)。 |
| `wuminapp/lib/duoqian/institution/institution_duoqian_create_page.dart` | service 调用切到新 14 参签名,`institution_name` 走 `_institutionName ?? selectedAccount.accountName`;`accounts` 暂用单条 `[(account_name, amount)]`;`register_nonce / signature / province / signer_admin_pubkey / a3 / sub_type / parent_sfid_id` 7 字段以零字节占位 + 显式 `TODO(step2e)` 注释,等 SFID 后端补 `signer_admin_pubkey` 后改为调 `app_get_institution`(`InstitutionDetailWithCredential`)拼数据。 |
| `wuminapp/test/duoqian/duoqian_manage_service_test.dart` | 3 条新测试:① 14 参 SCALE 字节序列与手工组装 expected 字节级对齐(call_index=5,2 admins / 2 accounts / signature 64B / sub_type=Some / parent=None);② `signer_admin_pubkey` 长度 31 → throw `ArgumentError`;③ `parent_sfid_id=Some(...)` 时末尾 SCALE 编码为 `0x01 + Compact len + bytes`。 |

### 验收数字

- `flutter analyze`:**0 issues**
- `flutter test`:**115 passed**(baseline 112 + 新增 3 条;比卡上 +2 多 1 条:覆盖 `parent_sfid_id=Some` 分支)
- 头 2 字节断言 `[0x11, 0x05]`(pallet=17 / call_index=5)通过,与链端 step2b 一致

### SFID 后端 `institution_credential` 接口现状

`/api/v1/app/institutions/:sfid_id` 返回 `InstitutionDetailWithCredential`,**已包含** institution_name / a3 / sub_type / parent_sfid_id / province / register_nonce / signature 7 字段;**未包含** `signer_admin_pubkey`。本卡按规约不动 SFID 后端,wuminapp 端用零字节占位 + `TODO(step2e)` 显式标注:

- `register_nonce / signature / province / a3 / sub_type / parent_sfid_id`:虽然 SFID 已返回,但本页面 `_apiClient.fetchInstitutionAccounts(...)` 拿到的是 `AppInstitutionAccounts` 脱敏列表,**不**含这些字段;切到 `app_get_institution` 才拿得到。
- `signer_admin_pubkey`:SFID `dto.rs::InstitutionDetailWithCredential` 当前完全没有这个字段,需 SFID 后端开新口子(从 `province_pair.public()` 取 32B → 0x 小写 hex)。

### 任务卡微调建议

- 将 step2e 拆成 2 张子卡更清晰:
  - **step2e-1(SFID Agent)**:`InstitutionDetailWithCredential` 加 `signer_admin_pubkey: String`,handler 从 `province_pair.public()` 填充。
  - **step2e-2(Mobile Agent)**:wuminapp `api_client.fetchInstitutionDetail(sfidId)` 新方法 → `institution_duoqian_create_page` 删除 7 个 `TODO(step2e)` 占位,接入真数据,端到端跑通冷钱包扫码 → 链端 verifier 通过。
- 本卡的 SCALE 编码层(call_index=5 + 14 参顺序 + signer_admin_pubkey 长度校验)已稳定,step2e 后续只换数据源,不再动 service。
