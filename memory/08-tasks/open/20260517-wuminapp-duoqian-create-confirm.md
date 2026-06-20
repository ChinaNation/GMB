任务需求：
修复 wuminapp 创建个人/机构多签时余额不足仍被前端误判成功的问题：创建前必须按链端口径校验发起钱包余额，创建类交易必须等待入块并确认链上创建提案事件后才能写本地记录，不能再用 txHash 或预测 NextProposalId 作为成功依据。

所属模块：
wuminapp

输入文档：
- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/workflow.md
- memory/07-ai/unified-naming.md
- memory/01-architecture/wuminapp/WUMINAPP_TECHNICAL.md
- memory/05-modules/wuminapp/personal-manage/PERSONAL_MANAGE_WUMINAPP_TECHNICAL.md
- memory/05-modules/wuminapp/transaction/duoqian-transfer/DUOQIAN_TRANSFER_APP_TECHNICAL.md

必须遵守：
- 不把 `author_submitExtrinsic` 返回的 txHash 当成创建成功。
- 创建多签前必须校验发起钱包余额覆盖 `初始资金 + 链上创建手续费 + ED`。
- 创建类交易必须等待入块，并确认对应 runtime 事件存在后再写本地多签和本地提案。
- 不使用预测 `NextProposalId` 作为本地提案编号。
- 对已产生的未上链本地幽灵记录进行清理，不再展示为“已注销/未知提案”。
- 改代码后必须补中文注释、更新文档并清理残留。

预计修改目录：
- wuminapp/lib/governance/personal-manage/：修复个人多签创建余额校验、入块确认、事件确认和本地提案落库时机，属于 Flutter 业务代码。
- wuminapp/lib/governance/organization-manage/：修复机构多签创建余额校验、入块确认和事件确认，属于 Flutter 业务代码。
- wuminapp/lib/governance/：修复多签列表对未上链本地记录的误判，属于 Flutter 展示与本地状态代码。
- wuminapp/lib/rpc/：复用交易入块与 System.Events 读取能力，属于链 RPC 边界代码。
- memory/01-architecture/wuminapp/ 与 memory/05-modules/wuminapp/：同步 txHash 成功判定规则和创建余额规则，属于文档更新。

输出物：
- 创建前余额校验
- 创建提案入块确认
- 创建事件解析与真实 proposalId 回填
- 本地幽灵记录清理
- 中文注释
- 测试/静态检查结果
- 文档更新

验收标准：
- 发起钱包 198 元、初始资金 200 元时，创建前直接提示余额不足，不能进入签名/提交成功流程。
- 创建个人多签时，只有确认 `PersonalManage.PersonalDuoqianProposed` 后才写本地记录。
- 创建机构多签时，只有确认 `OrganizationManage.InstitutionCreateProposed` 后才显示提交成功。
- 本地不存在链上提案的幽灵创建记录不再显示为“已注销 + 未知提案”。
- 静态检查通过或明确记录无法运行原因。
- 文档已更新，残留已清理。

新增命名说明：
- 中文名：多签创建金额规则；English name：duoqian_create_amount_rules；类型：Dart 常量/工具；使用位置：`wuminapp/lib/governance/shared/duoqian_create_amount_rules.dart`；简介：按 runtime 口径计算创建多签所需的 `amount + fee + ED`。
- 中文名：多签创建确认任务卡；English name：wuminapp-duoqian-create-confirm；类型：任务卡；使用位置：`memory/08-tasks/open/20260517-wuminapp-duoqian-create-confirm.md`；简介：记录本次创建多签成功判定修复范围、输出物和验收标准。

执行记录：
- 已新增 `wuminapp/lib/governance/shared/duoqian_create_amount_rules.dart`，按 runtime 口径计算创建所需金额：`初始资金 + max(初始资金 * 0.1%, 0.10 元) + 1.11 元 ED`。
- 已在个人多签创建页和机构多签创建页签名前增加发起钱包余额预校验；余额不足时直接提示并停止，不进入签名/提交流程。
- 已将 `PersonalManage::propose_create` 和 `OrganizationManage::propose_create_institution` 改为等待入块。
- 已从入块区块的 `System.Events` 中确认 `PersonalManage.PersonalDuoqianProposed` / `OrganizationManage.InstitutionCreateProposed`，并使用事件中的真实 `proposal_id`。
- 已删除个人多签创建页基于 `VotingEngine.NextProposalId` 的预测提案号逻辑。
- 已修复旧版本本地幽灵记录：链上账户不存在且本地 create 提案仍为 voting、链上 proposal 不存在时，删除本地多签实体、提案快照和本地状态，不再显示为“已注销/未知提案”。
- 已同步更新 wuminapp 架构文档、personal-manage 技术文档和 governance 技术文档。

验证记录：
- `dart format wuminapp/lib/governance/shared/duoqian_create_amount_rules.dart wuminapp/lib/governance/personal-manage/personal_duoqian_create_page.dart wuminapp/lib/governance/personal-manage/personal_manage_service.dart wuminapp/lib/governance/personal-manage/personal_proposal_history_service.dart wuminapp/lib/governance/organization-manage/institution_duoqian_create_page.dart wuminapp/lib/governance/organization-manage/duoqian_manage_service.dart wuminapp/lib/governance/duoqian_account_list_page.dart`：通过。
- `cd wuminapp && dart analyze lib test`：通过。
- `cd wuminapp && flutter test test/widget_test.dart`：通过。
- `cd wuminapp && flutter test test/governance/personal-manage test/governance/organization-manage`：通过。
- `git diff --check`：通过。
- `cd wuminapp && flutter build apk --debug`：通过。

2026-05-17 补充执行记录：
- 已确认链上旧问题不是余额未清空：关闭执行会扣手续费并把剩余余额转入用户提供的收款地址。
- 已修复 `admins-change` 动态主体关闭逻辑：个人/机构多签注销成功后删除 `Subjects[subject]` 当前状态，不再保留 `Closed` 墓碑。
- 已新增 `admins-change` storage v4 迁移：升级时清理旧链上遗留的 Closed 动态主体；历史事件和提案不删除。
- 已在个人多签关闭执行阶段复核 reserved 余额，避免提案后新增锁定资金导致销户不彻底。
- 已在机构账户关闭成功后清理 `InstitutionAccounts`、`SfidRegisteredAccount`、`AccountRegisteredSfid` 和管理员主体当前状态，与个人多签关闭口径对齐。
- 已在 wuminapp `ChainRpc` 增加 `System.ExtrinsicFailed` 模块错误解析，个人/机构创建确认在找成功事件前先显示真实链上失败原因。
- 已更新 runtime 与 wuminapp 技术文档，并补充测试覆盖。

2026-05-17 补充验证记录：
- `cargo test --manifest-path citizenchain/Cargo.toml -p admins-change --lib`：通过，44 passed。
- `cargo test --manifest-path citizenchain/Cargo.toml -p personal-manage --lib`：通过，23 passed。
- `cargo test --manifest-path citizenchain/Cargo.toml -p organization-manage --lib`：通过，24 passed。
- `cd wuminapp && flutter test test/governance/personal-manage test/governance/organization-manage`：通过。
- `cd wuminapp && dart analyze lib test`：通过。
- `cd wuminapp && flutter test test/governance/personal-manage/personal_manage_service_test.dart test/governance/organization-manage/duoqian_manage_service_test.dart`：通过。
- `git diff --check`：通过。

2026-05-17 阈值显示补充执行记录：
- 已修复个人/机构多签详情读取管理员主体时错把 `AdminsChange::AdminAccounts` 中 creator 字段解成 threshold 的问题。
- 个人/机构多签当前账户详情现在只从 `AdminsChange::AdminAccounts` 读取 org 和管理员列表，普通动态阈值改从 `InternalVote.ActiveDynamicThresholds[(org, subject)]` 读取，查不到 active 时再查 pending。
- `DuoqianAccountInfo.threshold` 改为可空；阈值查询不到时 UI 只显示管理员人数，不再显示错位大数字。
- 已补充个人/机构 storage codec 和 service 测试，覆盖 `AdminsChange::AdminAccounts` 不再携带 threshold 的布局。

2026-05-17 阈值显示补充验证记录：
- `cd wuminapp && dart analyze lib test`：通过。
- `cd wuminapp && flutter test test/governance/personal-manage/personal_manage_service_test.dart test/governance/personal-manage/personal_manage_storage_codec_test.dart test/governance/organization-manage/duoqian_manage_storage_test.dart test/governance/organization-manage/duoqian_storage_codec_test.dart`：通过。
- `cd wuminapp && flutter test test/governance/personal-manage test/governance/organization-manage`：通过。
