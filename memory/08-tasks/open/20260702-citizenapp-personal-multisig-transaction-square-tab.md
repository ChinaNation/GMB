任务需求：
- 删除公民 App 底部“多签”tab 及该 tab 内所有机构多签 / 机构账户展示、发现、同步和详情入口。
- 保留个人多签功能，并迁移到“交易”tab：将交易页现有扫码支付入口改为左右两个入口，左侧“扫码支付”，右侧“多签账户”。
- 扫码支付保持现有扫码能力，但入口不再显示箭头。
- “多签账户”进入纯个人多签账户列表页，顶部标题显示“多签账户”，右上角保留加号，点击加号直接进入创建个人多签页面。
- 在原底部“多签”tab 位置新增“广场”tab，作为未来功能入口。
- 将 `citizenapp/lib/citizen/8964` 目录改名为 `citizenapp/lib/citizen/all`，并将公民 tab 内原“广场”子 tab 改为“提案”。
- 待用户单独确认后，在 `citizenapp/lib/` 下新建 `8964` 目录，用于今后广场功能代码。

所属模块：
- citizenapp

输入文档：
- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/chat-protocol.md
- memory/07-ai/requirement-analysis-template.md
- memory/07-ai/thread-model.md
- memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md
- memory/07-ai/module-checklists/citizenapp.md
- memory/07-ai/module-definition-of-done/citizenapp.md

原导航阶段必须遵守：
- 不修改 `citizenchain/runtime/`。
- 不保留旧机构多签入口、旧机构多签文案、旧机构多签列表展示或旧机构发现触发路径。
- 不删除公民 App 内合法的机构目录、机构详情、机构账户业务能力；本任务只清理原“多签”tab 中的机构功能。
- 新建 `citizenapp/lib/8964/` 目录前，必须再次列出完整路径、用途、原因、是否会被 Git 跟踪，并取得用户明确确认。
- 代码必须补中文注释。
- 改代码后必须更新文档并清理残留。
- 完成前必须做真实运行态验收；仅编译、分析或单元测试不算完成。

当前轮追加确认（2026-07-02）：
- 用户已确认执行“机构 CID 作为提案归属唯一真源”的链端 + CitizenApp 改造，本轮允许并要求修改 `citizenchain/runtime/`。
- 机构码只用于提案分类/路由；机构类提案归属、订阅、活跃限制、互斥锁和反向索引统一按机构 CID。
- 个人多签没有 CID，仍按个人多签账户 `AccountId` 作为 `ProposalSubject::PersonalAccount`。

预计修改目录：
- `citizenapp/lib/`：调整底部导航，把原“多签”tab 替换为“广场”tab；待确认后新增未来广场功能目录。
- `citizenapp/lib/transaction/`：调整交易 tab 页面入口，接入个人多签账户列表。
- `citizenapp/lib/transaction/personal-manage/`：复用个人多签创建页、服务和本地实体，必要时调整列表入口文案。
- `citizenapp/lib/citizen/shared/`：从原混合多签列表中剥离机构逻辑，只保留个人多签列表；清理机构多签发现、状态同步和详情跳转残留。
- `citizenapp/lib/citizen/`：将 `8964` 改名为 `all`，更新公民 tab 子 tab 文案和 import。
- `memory/01-architecture/citizenapp/`：同步记录导航结构和个人多签入口目标态。
- `memory/08-tasks/open/`：记录本任务执行、验收和残留清理。

输出物：
- 代码
- 中文注释
- 文档更新
- 依赖搜索记录
- 真实运行态验收记录

验收标准：
- 公民 App 底部导航不再显示“多签”，原位置显示“广场”。
- 交易 tab 中同一行左侧显示“扫码支付”、右侧显示“多签账户”。
- 点击“扫码支付”仍进入现有扫码支付流程，入口不显示箭头。
- 点击“多签账户”进入纯个人多签账户列表页，标题为“多签账户”。
- 个人多签账户列表右上角加号直接进入创建个人多签页面。
- 个人多签列表不再读取、发现、同步或展示任何机构账户。
- 公民 tab 内原“广场”子 tab 改为“提案”。
- `citizenapp/lib/citizen/8964` 路径残留清理完毕，引用统一为 `citizenapp/lib/citizen/all`。
- 待确认后，`citizenapp/lib/8964/` 作为未来广场功能目录存在且入口可达。
- `flutter analyze` 或等效检查通过。
- 真机或模拟器运行态验证导航、扫码入口、个人多签入口和新增广场 tab 行为符合目标。

追加需求（2026-07-02）：
- 公民 tab「提案」子 tab 保持单一列表，不增加“默认 / 订阅”切换或分组控件。
- 提案列表默认显示机构码改为：`NRC/NLG/NSN/NRP/NED/NJD/NSP/PRS`。
- `PRC/PRB` 不再默认进入提案列表；省储会、省储行只有在当前钱包订阅对应机构时才显示其提案。
- 其它公权机构提案按当前热钱包订阅机构 CID 精确命中 `subject_cid_numbers`，不按机构码放大到同类全部机构。
- 链端 `Proposal` 保存机构归属 CID 列表：`subject_cid_numbers`；多机构关联提案写入多个机构 CID。
- 机构活跃提案索引改为 `ActiveProposalsBySubject`，机构类主体 key 为 `InstitutionCid(cid_number)`，个人多签主体 key 为 `PersonalAccount(account_id)`。
- 机构提案反向索引改为 `ProposalsByCid`，按机构唯一 CID 反查提案。

执行记录：
- 已全仓搜索旧多签入口、交易页入口、`citizen/8964`、广场/提案文案和机构发现残留。
- 已删除底部“多签”tab 挂载，原第 2 个底部 tab 改为“广场”，入口为 `citizenapp/lib/8964/square_tab_page.dart`。
- 已将 `citizenapp/lib/citizen/8964/vote_view.dart` 迁移为 `citizenapp/lib/citizen/all/proposal_view.dart`，并将公民 tab 子 tab 文案从“广场”改为“提案”。
- 已删除原 `citizenapp/lib/citizen/shared/institution_account_list_page.dart` 混合列表，新增 `citizenapp/lib/transaction/personal-manage/personal_account_list_page.dart` 作为纯个人多签账户列表。
- 已将交易 tab 的扫码入口改为同一行双入口：左侧“扫码支付”、右侧“多签账户”；扫码支付入口不再显示箭头。
- 已删除 `MultisigDiscoveryCoordinator`、`institution_discovery_service.dart` 和对应旧测试，个人多签列表直接扫描 `PersonalAdmins.AdminAccounts` 并只交给个人多签发现服务处理。
- 已将 `AdminAccountsScanService` 收窄为只扫描 `PersonalAdmins.AdminAccounts`；个人发现按 `kind=Personal`、`institution_code=PMUL`、本机管理员钱包过滤。
- 已更新交易页测试、启动冒烟测试、AdminAccounts 过滤测试。
- 已同步更新 CitizenApp 架构文档、personal-manage 技术文档和 governance 技术文档。
- 原导航阶段未修改 `citizenchain/runtime/`；本轮 CID 真源追加改造已按用户确认修改 runtime。

验收结果：
- `flutter analyze`：通过。
- `flutter test test/ui/transaction_tab_page_test.dart test/widget_test.dart test/governance/shared/admin_accounts_scan_service_test.dart`：通过。
- `flutter test test/governance/personal-manage/personal_manage_service_test.dart test/governance/personal-manage/personal_manage_storage_codec_test.dart test/governance/personal-manage/personal_pending_create_lookup_test.dart test/governance/personal-manage/personal_proposal_history_service_test.dart`：通过。
- `flutter test -j 1`：通过，301 项测试通过，4 项原生 OpenMLS / smoldot 宿主库相关测试按既有条件跳过。
- `git diff --check`：通过。
- Android 模拟器 `Medium_Phone_API_36.1` 真实运行态验收通过：
  - 首次权限引导点击“稍后再说”后进入交易页。
  - 底部导航显示 `公民 / 广场 / 信息 / 交易 / 我的`，不再显示底部“多签”。
  - 交易页链上支付表单上方同一行显示“扫码支付”和“多签账户”。
  - 点击底部“广场”进入 `广场` 页面，显示当前入口壳。
  - 点击“多签账户”进入标题为“多签账户”的个人多签列表，右上角为“新增个人多签”，空态文案只提个人多签。
  - 点击右上角加号直接进入“创建个人多签”页面，未出现机构多签创建选项。
  - 点击“公民”tab 后二级 tab 显示 `提案 / 立法 / 选举 / 治理 / 公权`。

残留清理：
- 代码、测试和相关文档残留搜索确认不再出现：
  - `InstitutionAccountListPage`
  - `institution_account_list_page`
  - `MultisigDiscoveryCoordinator`
  - `multisig_discovery_coordinator`
  - `institution_discovery_service`
  - `citizen/8964`
  - `package:citizenapp/citizen/8964`
  - `VoteView`
  - `多签 Tab`
  - `新增个人多签或机构多签`
  - `公民-广场`
- 已删除空目录 `citizenapp/lib/citizen/8964`。
- 已停止 Android 模拟器和运行中的 CitizenApp debug 会话。
- 已按本轮 CID 方案再次调整 `citizenapp/lib/citizen/all/proposal_view.dart`：公民-提案使用默认 8 个机构码 + 当前钱包订阅机构 CID 合并过滤，保持单一列表，不增加顶部切换。
- 已扩展 `MultisigTransferService.filterCitizenProposalFeedIds()` 和 `MultisigTransferProposalFeed.fetchCitizenProposalFeedIds()`，默认范围按机构码命中，订阅范围按 `subject_cid_numbers` 中的机构 CID 精确命中并按 proposal_id 倒序去重。
- 已扩展 `ProposalContextResolver.resolveBatch()` 支持传入 accountHex→InstitutionInfo 映射，避免订阅公权机构和默认国家机构在提案列表/详情中退回“机构账户 xxxx”。
- 已删除 `ProposalLocalStore` 的全局治理索引 API 和对应测试，公民-提案不再读取/保存 `governance.proposal.index.global`。
- 已更新 `multisig_transfer_decode_test.dart` 覆盖默认码、订阅 CID、`PRC/PRB` 非默认和同码未订阅不误入。
- 已更新 CitizenApp 架构文档和 governance 技术文档，清理“公民-提案只按 NRC/PRC/PRB / 广场提案列表 / 全局治理索引是真源”的旧描述。

CID 真源改造执行记录（2026-07-02）：
- 已修改 `citizenchain/runtime/votingengine`：`Proposal` 新增 `subject_cid_numbers` 与 `account_context`；`ActiveProposalsBySubject`、`InternalProposalMutexes`、`ProposalsByCid` 全部以 `ProposalSubject` / CID 为主体真源。
- 已修改 internal-vote / joint-vote / legislation-vote / election-vote / admins / manage / governance / multisig-transfer 等链端创建路径：机构类提案必须写入 CID；多机构提案写入多个 CID；个人多签提案写入空 CID 并使用 `PersonalAccount`。
- 已修改 `entity-primitives::InstitutionMultisigQuery` 增加 `lookup_cid`，runtime 通过公权/私权注册表查机构 CID。
- 已修改 CitizenApp 提案解码：`ProposalMeta`、本地摘要、运行时升级和多签转账提案解码全部读取 `subject_cid_numbers`。
- 已修改 CitizenApp 提案列表/机构详情/活跃提案查询：机构详情按 `subject_cid_numbers` 包含本机构 CID 过滤；订阅按 CID 命中；个人多签活跃提案按 `ProposalSubject::PersonalAccount` 查询。
- 已修改 OnChina 立法大屏只读链路：`Proposal` 镜像解码加入 `subject_cid_numbers`；活跃提案读取改为 `ActiveProposalsBySubject[InstitutionCid(cid_number)]`；节点绑定身份携带 `institution_cid_number`。
- 已更新 memory 技术文档、ADR 和任务卡，清理旧 storage 名、旧字段名和“订阅按主账户”旧口径。

CID 真源改造验收结果（2026-07-02）：
- `cargo test --manifest-path citizenchain/Cargo.toml -p internal-vote -p public-admins -p private-admins -p legislation-vote -p election-vote`：通过。
- `cargo test -p onchina --manifest-path citizenchain/Cargo.toml`：通过，131 项测试通过。
- `cargo check -p citizenchain --manifest-path citizenchain/Cargo.toml`：通过。
- `flutter analyze`：通过。
- `flutter test test/transaction/multisig-transfer/multisig_transfer_decode_test.dart test/governance/proposal_local_store_test.dart test/citizen/institution/institution_detail_test.dart`：通过。
- `flutter test --concurrency=1`：通过，303 项测试通过，4 项原生 OpenMLS / smoldot 宿主库相关测试按既有条件跳过。

全仓残留与宪法一致性复查（2026-07-02）：
- 已再次全仓搜索旧机构主账户提案索引、`internalOrg/internal_org`、`citizenTally`、`ProposalsByInstitution`、`ActiveProposalsByInstitution`、`institutionHexToCidNumber` 等残留；代码与文档中的旧主账户归属口径已清理。
- Node 治理反向索引命令已从 `list_proposals_by_institution` / `fetch_proposals_by_institution` 改为 `list_proposals_by_cid` / `fetch_proposals_by_cid`；`listProposalsByInstitutionCode` 仅保留机构码分类查询语义。
- 已清理 CitizenWallet、CitizenApp 和 memory 文档中非 runtime 的旧流程与旧条文号表述。
- 已按 SCALE 解析当前 `constitution.scale`：共 7 章、141 条；第 19/21/45/46/75/79/100/106 条与当前投票引擎状态机逐项核对。
- 核对结论：投票引擎逻辑与当前公民宪法规则一致。五类立法表决、常规/重要/特别阈值、特别案公投、总统/省长/市长签署、国家/省级三人会签、市级超时通过、护宪终审多数通过与超时否决均与宪法一致。
- 用户已二次确认允许修改 `citizenchain/runtime/` 注释；已清理 runtime 注释中的旧条文号、旧流程措辞、旧冻结条款号和过期反对率口径。
- 已同步清理 memory 历史任务卡中会误导当前目标态的旧条文号、旧冻结条款号和旧流程措辞；总统选举对应宪法第44条为当前有效条文，按现状保留。
- runtime 与全仓旧关键词复扫确认无命中；任务卡不再复写旧词清单,避免记录文本本身成为残留。

全仓残留与宪法一致性复查验收（2026-07-02）：
- `cargo test -p legislation-vote -p legislation-yuan --manifest-path citizenchain/Cargo.toml`：通过。
- `cargo check -p node --manifest-path citizenchain/Cargo.toml`：通过。
- `npx tsc --noEmit`（`citizenchain/node/frontend`）：通过。
- `flutter analyze`（`citizenapp`）：通过。
- `flutter test test/governance/proposal_local_store_test.dart`：通过。
- `flutter test --concurrency=1`（`citizenapp`）：通过，303 项测试通过，4 项原生 OpenMLS / smoldot 宿主库相关测试按既有条件跳过。
- `cargo fmt --manifest-path citizenchain/Cargo.toml --package node --check`：通过。
- `cargo fmt --manifest-path citizenchain/Cargo.toml --package votingengine --package legislation-vote --package legislation-yuan --check`：通过。
- `git diff --check`：通过。
