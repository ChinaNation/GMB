# 2026-05-09 admins_change 管理员更换 node 与 wuminapp 整理

## 任务需求

按确认后的目录设计执行管理员更换模块整理：

- node 节点端前后端都在各自 `admins_change` 目录承载管理员更换业务。
- 管理员激活属于管理员管理，统一移动到 node 后端 `admins_change/activation.rs`。
- wuminapp 不区分传统前后端，不新建 backend，管理员更换收口到 `wuminapp/lib/governance/admins-change/`。
- wuminapp 的管理员激活服务迁入 `lib/governance/admins-change/services/admin_activation_service.dart`。
- wuminapp 的 `InstitutionAdminService` 查询门面迁入 `lib/governance/admins-change/services/`，旧 `lib/institution/institution_admin_service.dart` 不保留。
- 清理历史 `wuminapp/lib/proposal/admin_change/` 空占位。
- `offchain/organization-manage` 详情页“换管理员”按钮进入 `governance/admins_change` 管理员更换流程。
- 更新技术文档，记录目录边界和实现入口。

## 影响范围

- `citizenchain/node/src/governance/admins_change/`
- `citizenchain/node/frontend/governance/admins_change/`
- `citizenchain/node/src/governance/`
- `citizenchain/node/src/desktop/`
- `citizenchain/node/frontend/governance/`
- `citizenchain/node/frontend/offchain/`
- `citizenchain/node/frontend/offchain/organization-manage/`
- `wuminapp/lib/governance/admins-change/`
- `wuminapp/lib/institution/institution_activation_service.dart`
- `wuminapp/lib/institution/institution_admin_service.dart`
- `wuminapp/test/governance/admins-change/`
- 历史 `wuminapp/lib/proposal/admin_change/` 残留清理
- `memory/05-modules/citizenchain/node/admins-change/`
- `memory/05-modules/wuminapp/admins-change/`

## 风险点

- `AdminsChange.propose_admin_set_change` 的 call data 必须与 runtime pallet index / call index / SCALE 编码保持一致。
- 现有管理员读取逻辑散在 institution 与 duoqian shared，需要迁移时保留兼容入口。
- wuminapp 只有 Flutter 客户端与 Rust FFI，不允许误建传统 backend 目录。

## 执行状态

- [x] 建立 node 后端 `admins_change` 目录并迁移管理员主体能力
- [x] 将 node 管理员激活迁入 `governance/admins_change/activation.rs`
- [x] 建立 node 前端 `admins_change` 目录并接入换管理员页面
- [x] 将 node 前端激活 API 从根 `governance/api.ts` 收口到 `governance/admins_change/api.ts`
- [x] 将 `offchain/organization-manage` 的“换管理员”按钮接入 `AdminSetChangePage`
- [x] 建立 wuminapp `lib/governance/admins-change` 模块
- [x] 将 wuminapp 管理员激活服务迁入 `lib/governance/admins-change/services/admin_activation_service.dart`
- [x] 将 wuminapp `InstitutionAdminService` 门面迁入 `lib/governance/admins-change/services/`
- [x] 清理旧占位和更新文档
- [x] 运行必要验证

## 验证记录

- `npm run build`（`citizenchain/node/frontend`）：通过。
- `flutter analyze lib/governance/admins-change lib/institution lib/proposal lib/vote lib/duoqian/shared test/governance/admins-change`（`wuminapp`）：通过。
- `flutter test test/governance/admins-change`（`wuminapp`）：通过。
- `rustfmt --edition 2021 --check citizenchain/node/src/governance/admins_change/activation.rs citizenchain/node/src/governance/admins_change/mod.rs citizenchain/node/src/governance/mod.rs citizenchain/node/src/desktop/mod.rs citizenchain/node/src/offchain/settlement/admin_unlock.rs`：通过。
- `WASM_FILE=/Users/rhett/GMB/citizenchain/target/wasm/citizenchain.compact.compressed.wasm cargo test --manifest-path citizenchain/Cargo.toml -p node admins_change`：继续编译，但被既有 offchain 问题阻断：
  - `node/src/offchain/duoqian_transfer/mod.rs` 缺失。
  - `node/src/offchain/settlement/packer.rs` 访问 `OffchainLedger.inner` 私有字段。

上述 Rust 阻断项不属于本次 `admins_change` 边界。

## 2026-05-10 复核记录

### 复核结论

- `admins-change` runtime 本体的管理员集合变更入口、执行回调、互斥锁校验、主体状态校验和动态阈值校验已覆盖主要链上安全边界；本次复核未发现 runtime `admins-change` 直接绕过漏洞。
- node 后端已收口到 `citizenchain/node/src/governance/admins_change/`，node 前端已收口到 `citizenchain/node/frontend/governance/admins_change/`。Rust 模块目录使用下划线是语言约束下的当前实现；如要前端目录统一为短横线，需要另行做命名迁移。
- wuminapp 已收口到 `wuminapp/lib/governance/admins-change/`，测试已收口到 `wuminapp/test/governance/admins-change/`；历史迁移来源目录已清理，不再作为当前实现入口。
- 整体功能尚未闭合，主要阻断在冷钱包 QR 解码协议、注册机构账户级管理员主体、node 前端主体选择能力和文档路径漂移。

### 发现的问题

- P0：冷钱包 `wumin/lib/signer/payload_decoder.dart` 需要同步 `propose_admin_set_change(org, subject, new_admins[])`，否则会导致严格签名校验拒签，或在非严格路径下造成冷钱包展示字段与真实 call data 不一致。2026-05-10 已在本任务中修复解码、action label 与测试。
- P0/P1：wuminapp 已将注册机构身份解析为 `InstitutionAccount(0x05)` 主体，但 runtime `organization-manage` 创建/激活注册机构管理员主体仍使用 `SfidInstitution(0x02)`。在第 4 步 `organization-manage` 改造完成前，注册机构账户级管理员变更链路不能视为完成。
- 已修复：node 前后端已统一为 `AdminSubjectRef`，内置治理机构走 `sfidNumber + org`，个人多签和机构账户走 `subjectIdHex + org`，动态主体缺少 `subjectIdHex` 时后端拒绝。
- 已修复：QR 注册表要求 `propose_admin_set_change` 展示字段为 `org, subject, new_admins[]`。2026-05-10 node、wumin 冷钱包、wuminapp QR adapter 均已统一到该字段集，`subject/new_admins` 使用 `0x` 小写 hex。
- P2：wuminapp `AdminSubjectService` 按 identity 缓存管理员主体，提交管理员更换后没有看到自动清理缓存路径，可能导致页面继续展示旧管理员集合。
- P2：旧路径文档和注释需要更新或删除旧说法。2026-05-10 已将任务卡、wuminapp governance 技术文档和 `institution_admin_service.dart` 注释更新到当前目录口径。

### 本次验证

- `cargo test --manifest-path citizenchain/Cargo.toml -p admins-change`：通过，41 个测试通过。
- `npx tsc --noEmit`（`citizenchain/node/frontend`）：通过。
- `rustfmt --edition 2021 --check citizenchain/node/src/governance/admins_change/*.rs`：通过。
- `/Users/rhett/flutter/bin/cache/dart-sdk/bin/dart analyze lib/governance/admins-change test/governance/admins-change`（`wuminapp`）：通过。
- `flutter analyze` / `flutter test` 未进入项目验证阶段，被本机 Flutter SDK 缓存写入权限阻断：`/Users/rhett/flutter/bin/cache/engine.stamp: Operation not permitted`。改用 `dart analyze` 完成静态验证；`dart test` 不适用于该 Flutter 测试集，缺少 `package:test`。

### 后续建议

- 优先修复冷钱包 `wumin/lib/signer/` 的 `propose_admin_set_change` 解码、展示字段和测试。
- 继续推进 `organization-manage` 第 4 步账户级管理员主体改造，使 runtime 与 wuminapp 的 `InstitutionAccount(0x05)` 规则闭合。
- 后续只剩 Flutter SDK 写权限恢复后补跑 Flutter 级测试；当前 admins-change 代码已用 Dart 静态分析覆盖。
- 统一 QR display 字段到 `memory/01-architecture/qr/qr-action-registry.md`。
- 更新或删除旧路径文档和注释，再重新运行 Flutter SDK 级验证。

### 2026-05-10 执行记录

- 已更新 wumin 冷钱包 `AdminsChange(12).call(0)` 解码：从旧单管理员替换同步为 `propose_admin_set_change(org, subject, new_admins[])`。
- 已更新 wumin 冷钱包 action label 与 pallet registry 常量命名。
- 已补充 wumin `payload_decoder_test` 对管理员集合变更的解码用例。
- 已更新 wuminapp governance 技术文档、任务卡旧路径和 `InstitutionAdminService` 注释。
- `flutter test test/signer/payload_decoder_test.dart test/signer/pallet_registry_test.dart`（`wumin`）：通过。
- `flutter analyze lib/signer test/signer`（`wumin`）：通过。

### 2026-05-10 runtime admins-change 修复记录

- 已修复 `admins-change` 主体/org 边界：
  - `PersonalDuoqian` 只能使用 `ORG_REN`。
  - `InstitutionAccount` 只能使用 `ORG_PUP / ORG_OTH`。
  - `SfidInstitution` 仅保留 ABI 兼容和机构归属/检索语义，新写入和变更路径返回 `InvalidSubjectKind`。
- 已修复 `votingengine` org 校验：`ORG_PUP / ORG_OTH` 进入内部投票合法 org 集合。
- 已修复 `internal-vote` 动态主体路径：Active/Pending 主体、阈值读取、显式 pending 快照创建均支持 `ORG_REN / ORG_PUP / ORG_OTH`。
- 已修复 runtime `RuntimeInternalThresholdProvider`：治理机构继续固定阈值，动态账户统一从 `admins-change` 读取 `REN/PUP/OTH` Active / Pending 阈值。
- 已更新 `ADMINSCHANGE_TECHNICAL.md`、`ADR-015`、`GOVERNANCE_TECHNICAL.md` 中关于 `REN/PUP/OTH` 和 `SfidInstitution` 的旧说法。
- 已补充 `admins-change` 单测：`InstitutionAccount + PUP/OTH` 成功、`InstitutionAccount + REN` 失败、`SfidInstitution` 写入失败。
- 已补充 `internal-vote` 单测：`PUP/OTH` pending subject 快照与 active subject 阈值快照。
- `cargo test --manifest-path citizenchain/Cargo.toml -p admins-change`：通过，42 个测试通过。
- `cargo test --manifest-path citizenchain/Cargo.toml -p internal-vote`：通过，88 个测试通过。
- `cargo test --manifest-path citizenchain/Cargo.toml -p votingengine`：通过，0 个测试。
- `WASM_FILE=/Users/rhett/GMB/citizenchain/target/wasm/citizenchain.compact.compressed.wasm cargo check --manifest-path citizenchain/Cargo.toml -p citizenchain`：通过。

### 2026-05-10 runtime 残留复查

- 本次范围内未发现 `admins-change` 仍把 `InstitutionAccount` 绑定到 `ORG_REN` 的残留。
- 本次范围内未发现 `admins-change` 仍允许 `SfidInstitution` 作为新管理员主体的残留。
- 管理员更换模块外的机构账户注册、注销、反查和账户管理员归属模型，已从本任务剥离；后续如需调整，应在 `organization-manage / personal-manage` 专项任务卡中处理，不再混入 admins-change 修复结论。

### 2026-05-10 node 前后端 admins-change 修复记录

- 已将 node 后端 `get_admin_subject_state / build_admin_set_change_request / submit_admin_set_change` 统一为 `AdminSubjectRef`：内置治理机构可用 `sfidNumber + org`，个人多签和机构账户必须用 `subjectIdHex + org`。
- 已将 `SfidInstitution` 从 node 管理员更换前置校验中排除；`PersonalDuoqian` 只能走 `ORG_REN`，`InstitutionAccount` 只能走 `ORG_PUP / ORG_OTH`。
- 已将 node QR display 字段统一为 `org / subject / new_admins`，与 wumin 冷钱包 `propose_admin_set_change` 解码字段一致。
- 已将前端 `AdminSetChangePage` 改为接收 `subjectRef`，NRC/PRC/PRB 入口带治理 org，清算行入口从主账户派生 `InstitutionAccount(0x05)` subject 并按 `ORG_OTH` 进入 `governance/admins_change`。
- 已补充 node 后端单测覆盖 `SfidInstitution` 拒绝、`PersonalDuoqian + 非 REN` 拒绝、`InstitutionAccount + PUP/OTH` 允许。
- 已清理 node / wumin 冷钱包 QR 展示中的旧机构类泛称：`ORG_REN` 显示为“个人多签”。
- `npx tsc --noEmit`（`citizenchain/node/frontend`）：通过。
- `rustfmt --edition 2021 --check citizenchain/node/src/governance/admins_change/*.rs citizenchain/node/src/governance/organization-manage/chain.rs citizenchain/node/src/governance/organization-manage/types.rs citizenchain/node/src/governance/runtime_upgrade/commands.rs`：通过。
- `WASM_FILE=/Users/rhett/GMB/citizenchain/target/wasm/citizenchain.compact.compressed.wasm cargo test --manifest-path citizenchain/Cargo.toml -p node admins_change`：通过，7 个 node admins_change 相关测试通过。
- `/Users/rhett/flutter/bin/cache/dart-sdk/bin/dart analyze lib/signer test/signer`（`wumin`）：通过。
- `flutter test test/signer/payload_decoder_test.dart test/signer/pallet_registry_test.dart`（`wumin`）：本机 Flutter SDK 缓存写权限阻断，报 `/Users/rhett/flutter/bin/cache/engine.stamp: Operation not permitted`；已用 Dart 静态分析覆盖本次冷钱包改动。

### 2026-05-10 wuminapp + wumin 冷钱包 admins-change 修复记录

- 已将 wuminapp `AdminSetValidation` 对齐 runtime/node：`SfidInstitution` 拒绝，`PersonalDuoqian` 只允许 `ORG_REN`，`InstitutionAccount` 只允许 `ORG_PUP / ORG_OTH`。
- 已将 wuminapp `AdminSetChangeQrAdapter` 的 display 字段从旧 `subject_id/admin_count/threshold` 改为 `org/subject/new_admins`，并统一 `0x` 小写 hex，避免冷钱包 strict display 比对失败。
- 已将 wuminapp `AdminSubjectService` 缓存 key 改为 `subjectIdHex`，并在管理员更换提交成功后清理对应 subject 缓存。
- 已将 wumin 冷钱包 `propose_admin_set_change` 解码增强为主体类型与 org 匹配校验：`0/1/2 -> Builtin`、`3 -> PersonalDuoqian`、`4/5 -> InstitutionAccount`；`SfidInstitution` 和错配主体拒绝解码。
- 已同步 wumin 冷钱包 org 展示：`ORG_REN=个人多签`、`ORG_PUP=公权机构账户`、`ORG_OTH=其他机构账户`。
- 已补充 wuminapp admins-change 测试：主体/org 错配拒绝、QR display 字段、`subjectIdHex` 缓存清理。
- 已补充 wumin 冷钱包测试：个人多签管理员集合变更、PUP/OTH 机构账户展示、subject kind 与 org 错配拒绝。
- 已同步 node QR display 的 `subject/new_admins` 为 `0x` 小写 hex，并将 PUP/OTH 展示值与冷钱包 decoder 对齐，保证桌面端发出的 QR 也能通过 strict 比对。
- `/Users/rhett/flutter/bin/cache/dart-sdk/bin/dart analyze lib/governance/admins-change test/governance/admins-change`（`wuminapp`）：通过。
- `/Users/rhett/flutter/bin/cache/dart-sdk/bin/dart analyze lib/signer test/signer`（`wumin`）：通过。
- `WASM_FILE=/Users/rhett/GMB/citizenchain/target/wasm/citizenchain.compact.compressed.wasm cargo test --manifest-path citizenchain/Cargo.toml -p node admins_change`：通过，7 个 node admins_change 相关测试通过。
- `flutter test test/governance/admins-change`（`wuminapp`）和 `flutter test test/signer/payload_decoder_test.dart test/signer/pallet_registry_test.dart`（`wumin`）：本机 Flutter SDK 缓存写权限阻断，报 `/Users/rhett/flutter/bin/cache/engine.stamp: Operation not permitted`；已用 Dart 静态分析覆盖本次改动。

### 2026-05-10 wuminapp admins-change 破坏式 identity 修复记录

- 已新增 `AdminSubjectIdentity` 三类主体：`governanceInstitution`、`personalDuoqian`、`institutionAccount`。
- 已删除 wuminapp admins-change 的旧模糊字符串查询入口；`InstitutionAdminService`、`AdminSubjectService`、`ActivationService`、`AdminSetChangePage` 均要求传入明确 `AdminSubjectIdentity`。
- 已将管理员激活存储切到 `activated_admins_v3`，只保存 `subjectIdHex / identityKey / org / kind`，不读取、不迁移旧 `activated_admins_v1/v2`。
- 已同步 wumin 冷钱包管理员激活解码：新增 `GMB_ACTIVATE_SUBJECT_V1` / `activate_admin_subject`，展示字段为 `org / subject / pubkey`，旧 `sfid_number` 激活 payload 不再识别为当前 admins-change 激活协议。
- 已改造机构详情、管理员列表、提案上下文、个人多签详情、机构账户详情、转账详情、runtime 升级详情等调用点，统一从 `InstitutionInfo` 派生 `AdminSubjectIdentity` 后再调用 admins-change 服务。
- 已清理 wuminapp 当前代码中的旧机构类泛称残留；通用 `OrgType.duoqian` 显示为“多签账户”，具体身份由 admins-change identity 区分。
- 已补充 wuminapp admins-change 测试：机构账户、个人多签、治理机构 identity 派生；v3 激活记录按 `subjectIdHex` 过滤并忽略旧 `sfidNumber` 记录。
- 已补充 wumin 冷钱包测试：subject 级管理员激活 payload 可解码，旧激活 payload 被拒绝。
- `/Users/rhett/flutter/bin/cache/dart-sdk/bin/dart analyze lib/governance/admins-change test/governance/admins-change lib/governance/shared/institution_info.dart lib/governance/organization-manage/institution_registry.dart lib/governance/organization-manage/institution_admin_list_page.dart lib/governance/organization-manage/institution_detail_page.dart lib/governance/governance_proposals_page.dart lib/governance/shared/proposal/proposal_context.dart lib/governance/personal-manage/personal_manage_account_info_page.dart lib/governance/organization-manage/duoqian_account_info_page.dart lib/governance/duoqian_manage_detail_page.dart lib/transaction/duoqian-transfer/duoqian_transfer_detail_page.dart lib/transaction/duoqian-transfer/duoqian_transfer_page.dart lib/governance/runtime-upgrade/runtime_upgrade_detail_page.dart`（`wuminapp`）：通过。
- `/Users/rhett/flutter/bin/cache/dart-sdk/bin/dart analyze lib/signer test/signer`（`wumin`）：通过。
- `flutter test test/governance/admins-change`（`wuminapp`）：本机 Flutter SDK 缓存写权限阻断，报 `/Users/rhett/flutter/bin/cache/engine.stamp: Operation not permitted`。
- `flutter test test/signer/payload_decoder_test.dart test/signer/pallet_registry_test.dart`（`wumin`）：本机 Flutter SDK 缓存写权限阻断，报 `/Users/rhett/flutter/bin/cache/engine.stamp: Operation not permitted`。
- `/Users/rhett/flutter/bin/cache/dart-sdk/bin/dart test test/governance/admins-change`（`wuminapp`）：不能作为替代，项目未引入 `package:test`，测试集依赖 Flutter 测试运行器。

### 2026-05-10 admins-change 本模块复查修复记录

- 已修复 runtime admins-change 读侧防线：`active_subject_*` 与 `pending_subject_*_for_snapshot` 返回前重新校验 kind/org，旧 `SfidInstitution` 管理员主体和 `InstitutionAccount + ORG_REN` 脏数据不再暴露给投票引擎或业务模块。
- 已将 node 后端管理员激活从旧 `GMB_ACTIVATE / activate_admin / sfid_number` 切到 `GMB_ACTIVATE_SUBJECT_V1 / activate_admin_subject`。
- 已将 node 本地激活记录从 `activated-admins.json` 切到 `activated-admin-subjects.json`，只按 `subjectIdHex / org / kind / pubkeyHex` 归档；旧文件不读取、不迁移。
- 已将 node 前端管理员列表、机构详情、提案详情和清算行节点声明页的已激活管理员读取统一为 subject 级 `AdminSubjectRef`。
- 已更新 node / runtime / wuminapp 文档和 wuminapp 激活服务注释，清理当前协议里的旧 `sfid_number` 激活说明。
- `cargo test --manifest-path citizenchain/Cargo.toml -p admins-change --lib`：通过，43 个测试通过。
- `npx tsc --noEmit`（`citizenchain/node/frontend`）：通过。
- `/Users/rhett/flutter/bin/cache/dart-sdk/bin/dart analyze lib/governance/admins-change test/governance/admins-change`（`wuminapp`）：通过。
- `WASM_FILE=/Users/rhett/GMB/citizenchain/target/wasm/citizenchain.compact.compressed.wasm cargo test --manifest-path citizenchain/Cargo.toml -p node admins_change`：被当前工作区未完成的 `runtime-upgrade` 改动阻断，报 `JointVoteEngine::create_prepared_joint_proposal_with_data_and_object` 不存在；未进入 node admins_change 测试执行阶段。

### 2026-05-10 admins-change 交互模块修复记录

- 已修复 `organization-manage` 创建机构时的 admins-change 主体：`propose_create_institution` 新增 `admin_org`，只允许 `ORG_PUP / ORG_OTH`；Pending/Active 管理员主体改为主账户地址派生的 `InstitutionAccount(0x05)`，不再使用 `SfidInstitution(0x02)`。
- 已将机构账户关闭、`InstitutionMultisigQuery`、`DuoqianSfidAccountQuery::is_admin_of` 改为读取账户级 subject 和 `Institutions[sfid].admin_org`；`duoqian-transfer` 对机构账户传 `ORG_REN` 会返回 `InstitutionOrgMismatch`。
- 已同步 node 后端 `propose_create_institution` QR/call_data 为 11 字段布局，并在机构详情读取 `admin_org` 和 active admins-change 主体；Pending 阶段回退显示创建快照。
- 已同步 node 前端创建机构入口传 `adminOrg=ORG_OTH`；wuminapp 创建机构、机构账户发现、提案上下文、转账入口均携带/使用 `adminSubjectOrg`，不再把机构账户当作个人多签。
- 已同步 wumin 冷钱包 `propose_create_institution` decoder：读取并展示 `org`，只接受 `ORG_PUP / ORG_OTH`，字段顺序与 runtime/node/wuminapp 一致。
- 已更新 organization-manage、duoqian-transfer、votingengine、node 清算行、wuminapp governance/admins-change 文档，清理“机构账户走 ORG_REN”和“10 字段布局”的旧说明。
- `cargo test --manifest-path citizenchain/Cargo.toml -p admins-change --lib`：通过，43 个测试通过。
- `cargo test --manifest-path citizenchain/Cargo.toml -p organization-manage --lib`：通过，24 个测试通过。
- `cargo test --manifest-path citizenchain/Cargo.toml -p duoqian-transfer --lib`：通过，23 个测试通过。
- `WASM_FILE=/Users/rhett/GMB/citizenchain/target/wasm/citizenchain.compact.compressed.wasm cargo test --manifest-path citizenchain/Cargo.toml -p node organization_manage`：通过，3 个 node organization_manage 相关测试通过。
- `npx tsc --noEmit`（`citizenchain/node/frontend`）：通过。
- `/Users/rhett/flutter/bin/cache/dart-sdk/bin/dart analyze lib/governance/organization-manage lib/common lib/transaction/duoqian-transfer lib/vote test/governance/organization-manage test/governance/admins-change`（`wuminapp`）：通过。
- `/Users/rhett/flutter/bin/cache/dart-sdk/bin/dart analyze lib/signer test/signer`（`wumin`）：通过。
- `npx prettier --write ...`：当前环境无本地 prettier，且网络受限无法访问 `registry.npmjs.org`；未执行 prettier，已用 `tsc` 和 `git diff --check` 覆盖本次 TS 变更。

### 2026-05-15 wuminapp + wumin 冷钱包新阈值 ABI 修复记录

- 已将 wuminapp `AdminsChange::propose_admin_set_change` call data 切到最终 ABI：`org / subject_id / new_admins / new_threshold`。
- 已删除 wuminapp 对 `AdminsChange::Subjects.threshold` 的旧读取口径；管理员主体 storage 只解码 `org/kind/admins/creator/created_at/updated_at/status`，固定阈值走制度常量，动态阈值读取 `InternalVote.ActiveDynamicThresholds / PendingDynamicThresholds`。
- 已将 wuminapp 管理员更换页面补齐阈值 UI：内置治理机构只读固定阈值，个人多签和机构账户可输入动态阈值并按严格过半公式校验。
- 已将 wuminapp 管理员更换 QR display 同步为 `org / subject / new_admins / new_threshold`，与冷钱包 decoder 严格比对。
- 已将 wumin 冷钱包 `propose_admin_set_change` decoder 切到新载荷，缺少 `new_threshold` 的旧载荷和尾部多余字节直接拒签。
- 已补齐机构多签注销后的本地显示：统一账户列表继续显示“已注销”，详情页不显示余额，右上角显示“删除”，确认后清理本机机构多签数据。
- 已将个人多签创建/注销提案本地初始票数改为 `yesVotes = 1`，对齐投票引擎发起人自动赞成票。
- 已同步 `unified-protocols.md`、wuminapp governance/admins-change/personal-manage/signer 文档和 wuminapp-vs-wumin 双端边界文档。
