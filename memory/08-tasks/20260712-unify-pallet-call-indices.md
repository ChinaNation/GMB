# 任务卡:全端 pallet 号连续化 + call 号分歧修正 + system_version→1

- 日期:2026-07-12
- 来源:创世阻塞审计后续(用户点 3 + 点 4)
- 关联决策:[[project_genesis_single_grandpa_authority_accepted]](点 1=不改)

## 背景/目标
用户要求「全仓库所有端 pallet 号/call 号完全统一且连续排序」。现状 runtime pallet_index = 0..36 带空洞(12、17 空缺、顺序被打乱);node / CitizenApp / CitizenWallet 三端硬编码号;citizenweb、chain-signing、scripts 号中立(0 改)。

## 已确认决策
- D1 = ①保序去洞:抹掉空洞 12/17,idx≥13 整体下移,得连续 0..34,相对顺序不变(23 个 pallet 换号)。
- D2 = ①保留 call 语义分带:call 的洞是刻意 ABI 分带,pallet 内 u8 不跨 pallet 漂移;只修误路由/各端分歧,不连续化 call。
- 点 3:`system_version` 0→1(StateVersion V0→V1),随本次重生创世生效。

## 权威新号表(name → 新 index,连续 0..34)
System0 Timestamp1 Balances2 TransactionPayment3 OnchainTransaction4 ProvincialBankInterest5 FullnodeIssuance6 PersonalManage7 ResolutionIssuance8 VotingEngine9 CitizenIdentity10 CitizenIssuance11 RuntimeUpgrade12 ResolutionDestroy13 Grandpa14 GrandpaKeyChange15 PowDifficulty16 MultisigTransfer17 GenesisPallet18 OffchainTransaction19 InternalVote20 JointVote21 ElectionVote22 OnchainIssuance23 Assets24 LegislationYuan25 LegislationVote26 PublicAdmins27 PrivateAdmins28 PersonalAdmins29 PublicManage30 PrivateManage31 Campaign32 AddressRegistry33 SquarePost34

旧→新(仅变化的 23 个):13→12 14→13 15→14 16→15 18→16 19→17 20→18 21→19 22→20 23→21 24→22 25→23 26→24 27→25 28→26 29→27 30→28 31→29 32→30 33→31 34→32 35→33 36→34。0–11 不变。

## 捆绑修正(不得带进新号)
- **PersonalAdmins 误路由 bug**:三端把 `propose_admin_set_change` 编成 `[7,3]`(pallet7=PersonalManage 无 call3,越界,现在必失败)。真源 = PersonalAdmins call_index(0)。改后正确编码 = **[29,0]**。
  - node: [admins/management/storage.rs:22](../../citizenchain/node/src/admins/management/storage.rs) `PERSONAL_ADMINS.pallet_index 7→29` + [call_data.rs](../../citizenchain/node/src/admins/management/call_data.rs) `call 3→0`
  - CitizenApp: [admin_set_change_call_codec.dart](../../citizenapp/lib/citizen/proposal/admins-change/codec/admin_set_change_call_codec.dart) `7→29 / 3→0`;chain_rpc.dart `7=>PersonalAdmins` 修正;qr `personalAdminsChange 0x0703→0x1d00`
  - CitizenWallet: [pallet_registry.dart](../../citizenwallet/lib/signer/pallet_registry.dart) `7→29 / 3→0`;qr `0x0703→0x1d00`
- **删死码**:CitizenApp [chain_rpc.dart:907](../../citizenapp/lib/rpc/chain_rpc.dart) 对空缺 index 12 的旧 `_adminSetChangeErrorHint`。

## 逐端范围
- runtime:lib.rs 23 行 pallet_index 改号 + votingengine/src/types.rs:43 注释 + system_version 0→1;configs.rs 符号变体自动跟随(0 改)。call_index 不动。
- node:onchain/offchain/multisig/governance 的 signing 与 call_data、runtime_upgrade、admins/management/storage.rs(含 bug 修)。
- CitizenApp:9 个 service pushByte 常量、qr_protocols.dart(~40 条 0xPPCC 只改高字节=pallet)、chain_rpc.dart switch + 删死码、onchain_asset_constants(25/26→23/24)、cloudflare square_event.ts(36→34)。
- CitizenWallet:pallet_registry.dart、qr_protocols.dart(与 App 逐字节镜像)、citizenwallet-run.sh(sed 同步须扩全量或废弃)、signer 测试字节向量。
- 夹具:memory/06-quality/fixtures/、memory/01-architecture/qr/qr-protocol-fixtures/*.json 的 0xPPCC 重生。
- citizenweb / chain-signing / scripts:0 改(执行时二次确认无漏网硬编码)。

## 死规则
- 四端逐字节同批改齐,漏任一端→冷钱包两色校验红拒/误放行([[feedback_tauri_rename_needs_frontend_sync]])。
- 链开发期:重新创世即可,无 migration、无 spec_version bump、无兼容层([[feedback_chain_in_dev]] [[feedback_chain_dev_never_ask_migration]])。
- storage prefix 由 pallet **名** twox128 决定,改号不动任何 storage key(本次不改名)。
- chainspec 重生用 CI 成功的 WASM(用户点 2),本轮不 bake。

## 验证
runtime+node `cargo check`;App/Wallet `dart analyze` + 字节向量测试(pallet_registry_test / payload_decoder_test / account_derive·sign golden);交叉核对每端硬编码 vs 新表;跑通个人多签管理员变更确认 [29,0] 入块。

## 执行记录(2026-07-12,未提交/未 bake)
- runtime:35 pallet 连续 0..34 + system_version→1;node:signing/call_data/storage 全改齐 + PersonalAdmins 误路由修复 [7,3]→[29,0]。
- **onchina 曾漏改(scoping 失误,已补)**:institution create PublicManage 32→30 / PrivateManage 33→31(动作码 0x2005/0x2105→0x1e05/0x1f05)、LegislationYuan 27→25、LegislationVote 28→26、AddressRegistry 35→33;含 7 个 golden 前缀测试([27,0]/[28,1]/[27,2] 等)。教训:改号必含 citizenchain/onchina/,验证必跑 `cargo test`(非仅 check)。
- 客户端:CitizenApp/CitizenWallet 活常量 + qr 0xPPCC + 注释;canonical `qr-action-registry.md` 第2节整表重写;~9 个 memory 技术文档同步。
- 验证结果:node `cargo test` 248/0;onchina 131/0;CitizenWallet signer 106/106;primitives golden(account_derive/sign)通过;runtime/node `cargo check` 通过;全 rust `*PALLET_INDEX*` 常量逐一核对=新表。
- CitizenApp:dart analyze 全绿;链相关测试(signer/governance/rpc/qr/transaction/legislation/citizen)通过。批量跑有 2 个失败(institution_detail「订阅按钮」等),经隔离复跑 All tests passed → 确认为已知 smoldot hermetic flaky([[project_citizenapp_test_smoldot_hermetic]]),与本次改号无关。
- 2026-07-12 后续全量验收发现并清理 3 条漏改测试：`SquarePost 36→34`、pallet `7` 错标为 `PersonalAdmins`、pallet `29` 错标为 `PublicAdmins`。修正后又增加安装包六节点守卫测试，CitizenApp 全量 `flutter test` 为 511 passed / 5 skipped / 0 failed。
- 待办:chainspec 用 CI WASM 重生(点 2)。

## 收尾清残 + ADR 订正(2026-07-12,独立审计 7 员发现后清理)
- **独立审计抓出功能性漏改**(已全修+验证):runtime `cases.rs:80` system_version 断言 0→1;node/frontend `VoteSigningFlow.tsx:160` 兜底 22→20;cloudflare `square_event.ts:6`+`chain_confirm.test.ts:331` square 36→34(App agent曾误报已改实未改)。另修 pre-existing 命名 bug:`personal_manage_storage_codec.dart:44` PersonalAccounts 存储键 PersonalAdmins→PersonalManage。
- **注释/文档清残**(A,工作流 4 区域):node/onchina rust 注释、CitizenWallet `payload_decoder`/`action_labels`、CitizenApp `personal_manage_*` 称谓、memory 文档(unified-protocols/CITIZENCHAIN_TECHNICAL/unified-naming/citizenapp-vs-citizenwallet/ADR-030);终检又补 4 处审计漏网(multisig benchmarks、votingengine lib.rs、institution_pallet_router、multisig_storage_codec)+ ADR-030:63。
- **flag1 已闭环**:qr/mod.rs 注释改对为 FRG=0x1b02(PublicAdmins.propose_federal_registry_province_admin_set_change)、CREG=公权机构创建 0x1e05。
- **flag2 已闭环**:ADR-022 account-keys 预留 27→35、ADR-027 立法 27/28→25/26、ADR-030 号订正,均注明 2026-07-12 重排;下一空号=35。
- 验证:runtime 35/0、node 248/0、onchina 131/0、CitizenWallet 106/106、cloudflare 6/6、primitives golden、dart analyze;全仓终检 grep 活代码(rs/dart/ts)零残留。
- 任务卡旧 idx 也已订正(用户要求不删只修):6 张卡(20260711、done/20260630、open/20260629、open/20260628、open/20260626-runtime-admins、open/20260626-election-vote)内旧 idx(24/27/28/32/33/34)与 PersonalAdmins[7,3] 历史描述全部改为新号;文件保留。
- **最终全仓终检 grep:活代码 + 文档 + 任务卡零残留**(仅本迁移记录卡与上游 smoldot 夹具保留旧→新对照,属正常)。
