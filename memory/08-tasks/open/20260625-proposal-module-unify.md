# 20260625 提案统一模块 + lib 目录合并到 citizen(CitizenApp 大重构)

用户 2026-06-25 拍板:① 所有机构(治理/立法/选举/其他公权/今后私权/个人多签)共用同一个「发起提案」入口,进入后按主体能力显示不同提案种类 → **必须拆成独立 proposal 模块**;② lib 根目录的 `governance`/`legislation` 与 `citizen/` 下同名目录乱,**合并到 `citizen/` 下**;③ 新建 `lib/citizen/proposal/`,**公民 tab 下机构提案入口统一在此实现,禁止散落别处**;proposal 内**每种提案一个子文件夹**。

## 现状(已核查)

- `lib/governance/`(62 文件,被 63 文件 import)= 提案/治理实现主模块;`lib/citizen/governance/governance_tab.dart` 仅 tab 壳。
- `lib/legislation/`(6 文件,被 6 文件 import)= 法律数据+阅读页;`lib/citizen/legislation/legislation_tab.dart` 仅 tab 壳。
- 提案发起页还散在 `lib/transaction/`(multisig-transfer 转账/安全基金/归集、offchain/onchain 等)。
- 教训根因:同名目录两地 + 提案逻辑三处(governance/transaction/legislation)散落。

## 目标结构(2026-06-25 用户逐条定稿)

机构提案入口已从“只公权机构码”升级为“提案主体能力表”:个人多签、创世治理机构、普通公权机构、私权机构、非法人机构共用 `ProposalSubject + ProposalCapabilityRegistry`;机构码仍参与判断,但只允许在能力规则层集中使用。

```
lib/
├── transaction/                  根模块,大部分不动(= 多签管理 + 链上/链下交易)
│   ├── organization-manage/      机构多签管理(创建+关闭=一个模块,不拆) ← 从 citizen/governance 移入
│   ├── personal-manage/          个人多签管理 ← 从 citizen/governance 移入
│   ├── onchain-transaction/      现有保留
│   └── offchain-transaction/     清算行 + 链下支付,整模块留 lib/transaction(2026-06-25 用户定,不迁 my/)
├── votingengine/                 投票引擎(internal-vote/legislation-vote)= 保持独立不动
└── citizen/
    ├── citizen_tab_page.dart
    ├── 8964/                     广场 ← citizen/vote 改名;广场管理全在此实现
    ├── legislation/              法律阅读(data + 阅读页 + legislation_tab)
    ├── election/                 选举机构
    ├── governance/               治理机构专属(governance_tab + NRC/PRC/PRB 相关)
    ├── public/                   其他公权机构
    ├── shared/                   公权共用基础设施(institution_info / proposal context+store / account_derivation / 提案详情&投票页 institution_manage_detail_page / institution_account_list_page)
    └── proposal/                 机构/个人多签「发起提案」统一模块,每种提案一个子文件夹
        ├── proposal_entry_page.dart   通用入口(替代 GovernanceProposalsPage)
        ├── proposal_registry.dart     ProposalSubject → ProposalCapability
        ├── transaction/          公权机构资金管理(转账/安全基金/手续费/归集) ← lib/transaction/multisig-transfer 移入
        ├── admins-change/        换管理员 ← 从 citizen/governance 移入
        ├── runtime-upgrade/      协议升级(类B) ← 从 citizen/governance 移入
        ├── legislation-yuan/     发起立法/修法/废法(类B,保留名 legislation-yuan) ← 从 citizen/governance 移入
        ├── resolution-issuance/  决议发行(占位)
        ├── resolution-destroy/   决议销毁(占位)
        ├── grandpa-key/          验证密钥(占位)
        └── election/             发起选举(占位)
```

- 立法发起=类B,`proposal/legislation-yuan/` 放 LegislationIntroPage(节点端发起说明),不在阅读页放入口。
- 个人多签/私权后续接入同一能力入口;业务表单仍留在各自模块内,proposal 只负责统一入口和公民 tab 展示路由。

## 定稿(2026-06-25 用户确认)

- 公权机构资金管理 `multisig-transfer`(转账/安全基金/手续费/归集)→ `citizen/proposal/transaction/`。
- `lib/transaction/` 大部分不动,目标 = organization-manage(从 citizen/governance 移入)+ personal-manage(移入)+ onchain-transaction(留)+ offchain-transaction(留)。
- offchain-transaction(清算行 + 链下支付)**整模块留 lib/transaction**,不迁 my/(2026-06-25 用户定)。

## 分阶段(每阶段 flutter analyze 0 才进下一阶段)

- **P3a 移出/归位**:`transaction/multisig-transfer`→`citizen/proposal/transaction/`;`citizen/governance/{organization-manage,personal-manage}`→`lib/transaction/`(offchain-transaction 不动)。
- **P3b 建 proposal**:`citizen/governance/{admins-change,runtime-upgrade,legislation-yuan}`→`citizen/proposal/<同名>/`;新建 proposal_entry_page + proposal_registry + 4 占位文件夹(resolution-issuance/destroy/grandpa-key/election)。
- **P3c 拆 shared/governance**:`citizen/governance/{shared,institution_manage_detail_page,institution_account_list_page}`→`citizen/shared/`;`citizen/governance/` 只留 governance_tab + 治理专属。
- **P4 广场改名**:`citizen/vote/`→`citizen/8964/`。
- **P5 入口统一 + 收尾**:机构详情页「发起提案」→ proposal_entry_page(按主体能力 registry 过滤);撤之前误加的立法 entry;残留扫描、文档、注释、analyze。

## 硬规则

禁止兼容/留旧目录 / 公权提案全归 proposal/ / 每种提案独立子文件夹 / 机构管理不拆 / 命名照用户字面(legislation-yuan/8964)/ 每阶段 analyze 0 才继续。

## 进度

- [ ] P0 结构 + registry 定稿(待用户确认上面 4 点)
- [x] P1 legislation 迁移(2026-06-25):`lib/legislation/*` → `lib/citizen/legislation/`(data+law_list+law_reader),import 全局重写 `package:citizenapp/legislation/`→`.../citizen/legislation/`,残留 0,analyze 0。
- [x] P2 governance 迁移(2026-06-25):`lib/governance/*`(9 顶层条目/62 文件)→ `lib/citizen/governance/`,import 全局重写 `package:citizenapp/governance/`→`.../citizen/governance/`(63 文件),残留 0,analyze 0。
- [x] P3a(2026-06-25):`transaction/multisig-transfer`→`citizen/proposal/transaction`;`citizen/governance/{organization-manage,personal-manage}`→`lib/transaction/`(修 institution_account_list_page 相对 import);offchain-transaction 留 lib/transaction。analyze 0。
- [x] P3b(2026-06-25):`citizen/governance/{admins-change,runtime-upgrade,legislation-yuan}`→`citizen/proposal/<同名>/`;`governance_proposals_page`→`proposal/proposal_entry_page`(类名 GovernanceProposalsPage→ProposalEntryPage);新建 `proposal_registry.dart`(机构类型→提案种类单源)+ `proposal_placeholder.dart` + 4 占位文件夹。analyze 0。
- [x] P3c(2026-06-25):`citizen/governance/{shared,institution_manage_detail_page,institution_account_list_page}`→`citizen/shared/`(修 `'shared/multisig_discovery_coordinator'` 相对 import);citizen/governance/ 只剩 governance_tab。analyze 0。
- [x] P4(2026-06-25):`citizen/vote`→`citizen/8964`(广场)。analyze 0。
- [~] P5 入口统一 + 收尾(2026-06-25 部分):
  - [x] 撤掉误加的 institution_detail_page `_legislationProposeEntry`(+ 删 import);proposal_entry_page 3 占位卡接入占位页;残留扫描(旧根目录已删/旧 package 0/断裂相对 import 0);验证 analyze 0 + legislation 5/5 + governance·proposal·transaction 测试过(唯一失败=已知 Isar flaky,--concurrency=1 过)。
- [x] **E1 registry 机构码驱动(2026-06-25)**:proposal_registry 改为机构码键(删 ProposalInstitutionType 枚举);各提案→可发起机构码集合单源。此方案已在 2026-06-27 被主体能力表取代。
- [x] **E3 entry_page 改 registry 驱动(2026-06-25)**:删 `if(orgType==nrc||prc)` 硬门控,曾改为旧机构码能力表渲染卡片。此方案已在 2026-06-27 升级为 `ProposalSubject + ProposalCapabilityRegistry`,旧函数已删除。
  - [x] **E2+E4 全公权入口(2026-06-26 完成,防御式)**:institution_detail_page `_load` 改 `_govInfo = governanceInfo(cid) ?? _infoFromInstitution(inst)`——治理三类用静态档(含安全基金等专户),其余公权机构从 Institution 派生主/费账户构造 InstitutionInfo(`_infoFromInstitution`);`_govInfo` 全公权非空 → `_isGovernance` 全开 → 提案入口 + admins/adminWallets 加载对全公权生效;ProposalEntryPage 加 `institutionCode` 传 `_inst.institutionCode`。**防御**:`_accountIdentity` getter try/catch `ArgumentError`(非治理且非注册账户身份解析失败 → null,入口仍开但需激活),绝不崩。验证:analyze 0 + governance_tab 7/7 + proposal_local_store 2/2。**立法机构(NRP/NED/PRP/CLEG/CSLF/CEDU)现经 registry 显示「发起立法」卡 → LegislationIntroPage,UI 入口打通。**
  - [x] **E5 主体能力表(2026-06-27)**:proposal_registry 从旧裸机构码能力表升级为 `ProposalSubject -> ProposalCapability`,集中描述 pallet/call/voteEngine/启用状态;ProposalEntryPage 按主体能力渲染;InstitutionDetailPage 不再用 `_govInfo != null` 判定治理,固定治理按 `NRC/PRC/PRB`,普通注册机构也打开统一提案入口和可激活管理员列表。非法人不再自动归 PrivateAdmins,必须由 CID 注册归属显式 public/private kind;管理员读取和管理员变更 call data 优先按 `AdminAccount.kind` 路由;OrganizationManage 创建端上同步拒绝裸非法人机构码。验证:相关 focused tests 通过,organization-manage/admins-change 14/14 通过。
  - **小遗留**:① 非治理机构的真实链上 admins 是否齐全仍需真机逐类型 QA——入口/卡片已开,激活态依赖 admins-change 数据;② 选举、链上发行、链下清算费率 runtime 当前未启用,前端不展示可发起入口;③ 个人多签/私权的详情页仍需逐步接入统一能力入口,业务表单不迁移。
