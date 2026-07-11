# 任务卡：CitizenApp 命名精简统一 + 残留清理

## 任务需求

执行 2026-07-11 全仓库命名审计中「公民 CitizenApp」一类的 68 条方案(见 [[project_naming_audit_2026_07_11]]),对 `citizenapp/` 做彻底的精简+统一+清除:

- **精简**:只缩短/去冗余,绝不加长(遵守 [[feedback_simplify_means_shorter]] [[feedback_dir_separator_depth_caps]])。
- **统一**:全仓同名字段零例外一致(如 SS58 前缀单源、`sigAlg`→`alg`、`*Tab` 顶级页族)。
- **清除**:所有旧名/重复/死代码/空目录残留一律删除(遵守 [[feedback_no_remnants]] [[feedback_no_compatibility]])。
- 不管链上/迁移/兼容,彻底改。
- 完成后更新文档、完善注释、清理残留。

**用户指定例外:`8964/` 与 `test/8964/` 目录名保持不变(不改 square)。** 其内文件/类名的规范化不受影响(目录名保留 8964)。

## 执行批次

1. 删除死残留:app_theme 死令牌(headerGradient/subtleGradient/radiusXl/elevatedCard)、`qr/signature_message.dart`、`signer/signer.dart` 桶文件、`QrSigner.maxPayloadChars`、`_SimpleScanner`、空测试目录(test/proposal、offchain/onchain-transaction、多余 .gitkeep)。
2. 纯符号改名(~30 项):*Tab 页族、CitizenBadge→IdentityBadge、LegislationVoteMeta→LegMeta、OnchainRpc→TransferRpc、proposerPubkey→proposerSs58、VotersCountResponse→CitizenCountResponse 等。
3. 文件/目录改名:onchain.dart→transfer_rpc.dart、wallet_isar.dart→app_isar.dart、hw_vault_harness→seed_vault_harness、citizen/all→citizen/feed;smoldot-dart→smoldotdart、smoldot-pow→smoldotpow(第2级目录禁分隔,含 build/pubspec/FFI 引用同步)。
4. 语义合并:InstitutionInfo→Institution、OrgType/orgType→InstitutionClassification、Institution.status 字符串→InstitutionStatus 枚举、PersonalProposalStatus→枚举、identityLevel→tier、_ss58Format/_ss58Prefix→单源 kSs58Prefix、surfaceWhite→surfaceCard、SignRequestBody.sigAlg→alg、_truncateWalletAddress→_truncateAddress、重复 _ScanOverlayPainter/_ScanCornerPainter 抽单份。
5. 注释/文档:resolution_destro→resolution_destroy、InstitutionCodeLabel「104/88」→「92」、AppLockService._hash 类注释 SHA-256→PBKDF2;同步 memory/05-modules 相关模块文档。

## 验收

- `flutter analyze` 零新增错误(以本任务前基线为准)。
- 全仓无 `InstitutionInfo`/`OrgType`/死令牌/被删文件残留引用。
- `8964`/`test/8964` 目录名保留。
- 任务卡回写验收结果。

## 预计修改目录

- `citizenapp/lib/`(citizen/institution·shared、ui、qr、signer、wallet、transaction、my、im、votingengine、rpc、isar、security、smoldotpow、smoldotdart 等)
- `citizenapp/test/`
- `memory/05-modules/`、`memory/08-tasks/`(验收)

## 验收结果（2026-07-11 完成）

`flutter analyze` = 2 issues(与改动前基线完全一致的既有 info-lint,零新增错误)。`flutter test` = 482 通过,39 失败——全部为 `bootstrap_test`/`wallet_manager_test` 的 `Isar.initializeIsarCore` 原生库 TimeoutException(headless VR 环境不能起 isar native),非本次改动引入(零编译/符号错误,所有被改名区域测试全绿)。改动 493 文件(删 5 / 改 153 / 改名 328)。

**已执行:** 全部删除(死令牌 4、死文件 signature_message/signer 桶、死常量 maxPayloadChars、空测试目录/gitkeep)+ 约 30 项符号改名(*Tab 页族、CitizenBadge→IdentityBadge、OnchainRpc→TransferRpc、LegMeta、AdminSetChange→AdminsChange、kNationalCouncil→kNrc/kProvincialCouncils→kPrcs 含生成器、proposerPubkey→proposerSs58 定向)+ 文件/目录改名(onchain→transfer_rpc、wallet_isar→app_isar、hw_vault_harness→seed_vault_harness、citizen/all→feed、widget_test→bootstrap_test、profile_test_doubles→fake_profile;**smoldot-dart→smoldotdart、smoldot-pow→smoldotpow**,pubspec/analysis/Cargo/FFI 全部改到位,`flutter pub get` + `cargo metadata` 均通过)+ 安全合并(surfaceWhite→surfaceCard、org_models→institution_models、sigAlg→alg)+ 注释/文档(resolution_destroy、PIN 哈希 PBKDF2、机构码 104→92、模块文档 smoldot 路径)。**`8964`/`test/8964` 目录名按用户要求保留。**

**未执行(经读源码核实为错误建议或行为级重构,不属命名精简,已向用户报告):**
1. `OrgType→InstitutionClassification`——错:前者是 NRC/PRC/PRB/account 分类常量类,后者是 tab 分组谓词工具,概念不同。
2. `InstitutionInfo→Institution`——错:Institution 由 InstitutionInfo 构造(institution.dart:130),是不同模型非重复。
3. `_truncateWalletAddress→_truncateAddress`——错:刻意不同截断宽度(钱包 8+8 vs 哈希 6+6),合并会截短地址显示。
4. `Institution.status String→enum`、`PersonalProposalStatus→enum`——行为级重构(String→枚举,牵连序列化)。
5. `MyIdState.identityLevel→tier`——identityLevel 是跨 ~15 文件的 String + JSON 键 `identity_level`,非冗余 getter,收敛需序列化重构。
6. `_ScanOverlayPainter/_ScanCornerPainter` 抽共享、`_SimpleScanner`→复用 QrScanPage——真重复但属 UI 抽取/替换重构,建议单独去重任务。
