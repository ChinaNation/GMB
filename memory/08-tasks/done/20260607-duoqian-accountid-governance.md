任务需求：

将链上治理主体统一为多签账户 `AccountId`，删除历史主体包装治理协议；将 DUOQIAN 账户派生协议统一为 `DUOQIAN` 单一真源，所有主账户、费用账户、质押账户、安全基金账户、两和基金账户、个人多签账户、机构自定义账户都必须调用同一派生入口。

所属模块：

- citizenchain runtime/primitives
- citizenchain runtime/votingengine
- citizenchain runtime/governance
- citizenchain runtime/issuance
- SFID
- wumin
- wuminapp
- memory 文档系统

必须遵守：

- 不保留历史主体包装协议、旧命名、旧注释或旧文档残留。
- 不保留历史 DUOQIAN domain 常量、旧版本口径或第二套派生实现。
- 投票引擎、管理员模块、发行模块的治理主体统一为多签账户 `AccountId`。
- `asset_id` 只表示资产编号，不能承担治理身份。
- DUOQIAN 账户派生唯一真源在 `citizenchain/runtime/primitives/src/core_const.rs`。
- 其他模块只允许调用唯一真源或严格镜像该协议，不得自行定义第二套 domain/preimage 规则。
- 改代码后必须更新文档、完善中文注释、清理残留。

目标协议：

```text
AccountId = BLAKE2-256(
  DUOQIAN
  || op_tag
  || ss58.to_le_bytes()
  || payload
)
```

目标 op_tag：

- `OP_MAIN = 0x00`，payload = `sfid_number`
- `OP_FEE = 0x01`，payload = `sfid_number`
- `OP_STAKE = 0x02`，payload = `sfid_number`
- `OP_AN = 0x03`，payload = `sfid_number`
- `OP_HE = 0x04`，payload = `sfid_number`
- `OP_PERSONAL = 0x05`，payload = `creator || account_name`
- `OP_INSTITUTION = 0x06`，payload = `sfid_number || account_name`

验收标准：

- runtime 不再以历史主体包装协议或 `asset_id` 承担投票身份、管理员身份。
- 需要投票的主体只使用多签账户 `AccountId`。
- onchain-issuance 的发行人/治理主体为机构多签 `AccountId`，`asset_id` 只保留资产编号语义。
- 全仓不再存在历史 DUOQIAN domain 或第二真源派生公式。
- 内置账户地址已按目标 DUOQIAN 协议重算。
- SFID、wumin、wuminapp、tools 与文档全部同步。
- 残留扫描、必要测试和构建通过。

执行记录：

- runtime/primitives：保留 `core_const.rs` 作为 DUOQIAN 唯一真源，`derive_duoqian_account(op_tag, ss58, payload)` 内部使用 `ss58.to_le_bytes()` 写入 preimage，删除旧主体包装派生模块。
- runtime/votingengine/governance/transaction/issuance：投票主体、管理员主体、转账支出主体、发行主体统一为 `AccountId`；onchain-issuance 的 `asset_id` 只作为资产编号。
- SFID backend/tools：SFID 后端引用 runtime primitives 的唯一真源；`tools/duoqian.py` 改为读取 `core_const.rs`，并重算创世内置账户地址。
- wumin/wuminapp：冷钱包和移动端管理员账户、资产发行、转账、发现服务统一为 32 字节 `AccountId`；删除旧主体包装 codec、页面、服务和测试入口。
- memory/docs/frontend generated docs：ADR、模块文档、open/done 任务卡和前端内置文档产物已同步目标态。

验证结果：

- `cargo check -p duoqian-transfer -p internal-vote -p personal-manage -p organization-manage -p admins-change -p resolution-destro -p grandpakey-change -p onchain-issuance --tests`
- `cargo check -p duoqian-transfer -p admins-change -p resolution-destro -p grandpakey-change --features runtime-benchmarks`
- `cargo check -p node --bin citizenchain`
- `cargo check`（`sfid/backend`）
- `npm run build`（`citizenchain/node/frontend`，同步 `generated/local-docs.generated.ts` 与 `dist`）
- `flutter analyze`（`wuminapp`、`wumin`）
- `flutter test`（wuminapp 管理员/机构/个人多签相关测试）
- `flutter test test/signer/payload_decoder_test.dart`（`wumin`）
- `python3 tools/duoqian.py --dry-run`：main/fee/stake/anquan 全部 0 变更，408 个保留地址唯一。
- 残留扫描：旧主体协议、旧 storage 名、旧 DUOQIAN 口径、前端 generated/dist 产物均无命中；`tools/duoqian.py` 仅保留读取 `core_const.rs` 的正则，不是第二真源。

独立复核（2026-06-07，提交 `0bd3ba9f 重构多签账户体系`）：

- 全仓残留扫描:`SubjectId / SubjectKind / subject_id_from / primitives::derive / DUOQIAN_V1 / DUOQIAN_DOMAIN / AdminSubject / ss58_prefix_le` 全部 0 命中。
- `cargo test`(duoqian-transfer / admins-change / internal-vote / joint-vote / personal-manage / organization-manage / onchain-issuance):**216 passed / 0 failed**。
- onchain-issuance Phase-1 stub 内一处过期注释 `NRC_SUBJECT_ID` 已改为 `NRC 机构多签 AccountId`。
- 任务卡归档至 `memory/08-tasks/done/`。
- 遗留:onchain-issuance `propose_*` 的「机构多签 + proposer 是管理员 + reserve 押金」授权逻辑属 Phase-2(ADR-011 / `project_onchain_issuance_phase1`),不在本卡范围。
