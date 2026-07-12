# 20260629 organization-manage 拆分残留清理 + citizenapp 三分

- 状态:进行中(2026-06-29)
- 触发:用户身份系统重构把链端 `organization-manage` crate 删→拆 `public-manage`(idx32)/`private-manage`(idx33);全仓审计(6 面 33 agent)发现 node/citizenapp/文档/脚本多处残留。
- 关联:[[project_institution_admin_field_model_2026_06_28]]、ADR-030、`memory/07-ai/unified-protocols.md`、`qr-action-registry.md`。

## 背景事实(锁定)
- 链端:`OrganizationManage(17)` 已删(17 为空号);`PublicManage=32`/`PrivateManage=33`,`propose_create_{public,private}_institution`=call 5(15 字段逐字节相同,仅前缀);MODULE_TAG `org-mgmt`→`pub-mgmt`/`pri-mgmt`;storage `Institutions`/`InstitutionAccounts` 名不变但前缀随 pallet 名变。
- citizenwallet 冷钱包已完整迁移(0 残留);runtime/Cargo 工作区 0 残留。

## 验收
- node `cargo check -p node` 绿;citizenapp `dart analyze` 0 error;`grep OrganizationManage/organization-manage` 在各活跃面归零(历史 ADR/已完成卡保留)。
- 零残留:无指向已删 pallet/crate/目录的活跃代码或"当前真源"文档。

## 进度
### A. node 链端(CRITICAL,用户优先)—— ✅ 完成(2026-06-29,cargo check -p node 绿)
- [x] `chain.rs` 新增 `institution_manage_pallet(cid_number)`(经 `institution_code_from_cid_number`+`is_private_legal_code` 派生 PublicManage/PrivateManage),Institutions map_key + InstitutionAccounts 前缀(fetch_institution_accounts 加 `manage_pallet` 参)全切;doc 注释同步。
- [x] `proposal.rs` MODULE_TAG `b"org-mgmt"`→双常量 `TAG_PUBLIC_MANAGE=b"pub-mgmt"`/`TAG_PRIVATE_MANAGE=b"pri-mgmt"`,`is_organization_manage_proposal`→`is_institution_manage_proposal` 双前缀 OR(两调用点已改)。

### B. 文档 + 脚本 + onchina 注释 —— ✅ 权威文档完成(2026-06-29)
- [x] unified-protocols.md:P-TX-001(本轮早先)+ P-STORAGE-002 整节 + P-CRED-001 + P-STORAGE-004 真源/生产者/消费者/必跑测试改 public-manage/private-manage(顺手修 `runtime/private/personal-manage`→`entity/personal-manage`)。剩 12 处为**故意保留**的禁止/历史行 + citizenapp 目录(归 phase C)。
- [x] unified-naming.md:47/50 示例 + 145(机构管理行拆两 pallet)+ 456/457(删已删 node 目录)+ 471/472(改 entity/)。剩 1 处=citizenapp 目录(phase C)。
- [x] CITIZENCHAIN_TECHNICAL.md §9.3(私权模块→实体模块 entity/ 三 manage)+ §12.1.2 文档路径。
- [x] node 模块文档:GOVERNANCE_TECHNICAL(模块树/前端目录)、CID_CONFIG_TECHNICAL;NODE_CLEARING_BANK 加过时警告 banner(整篇属 B0 node→onchina 文档债,待 B0 对齐重写)。
- [x] scripts/load-context.sh:case 路由改 entity/public-manage|private-manage(拆两 case)。
- [x] onchina 注释 chain_runtime.rs:278、actions.rs:41 改真源(public-manage/private-manage / configs)。
- 保留:04-decisions/ADR-* 全部(历史决策记录,故意提旧名,不改);node admins-change / NODE_CLEARING_BANK 正文(B0 文档债)。

### C. citizenapp 机构重构(框架终定,2026-06-29)
**关键分析结论**:① 公民 tab 已是 ADR-028 统一模型(公权/私权同壳同详情同数据路径,仅 `institution_code`→pallet 路由不同,单源 `lib/citizen/shared/institution_code_label.dart`);② 端侧不做字面三目录(逆转 ADR-028+重复 codec),用"共享核心+路由器";③ **用户终定的层归属**:`transaction/`=交易ONLY(链下/链上/个人多签管理**保留**/多签转账);机构**管理**→`citizen/`(非交易);机构按[管理→citizen]/[业务:转账→transaction、管理员/立法/升级→citizen/proposal]拆分。完整分析见 workflow wf_22ea8321 + wf 公民tab分析。

**层归属(终定)**:
- `lib/transaction/`=交易ONLY:offchain ✅ + onchain ✅ + **personal-manage ✅保留不动**(个人多签非机构、链上独立、未入 onchina,含创建/关闭自助)+ **multisig-transfer ←从 citizen/proposal/transaction 移入**(公私个共用)+ shared ✅。
- `lib/citizen/institution/`=机构管理:统一模型(ADR-028 保留)+ **吸收机构链访问核心**(←lib/transaction/organization-manage)。
- 删除仅限**机构**(公权+私权)创建/关闭;**个人多签创建/关闭保留**。

**Chunk 1 — 机构管理核心 → citizen/institution + 删机构写**:
- [x] `institution_pallet_router.dart` 建好并在 `lib/citizen/institution/`。路由设计已核实(cid 键按 forInstitutionCode;account 键反查双查 managePallets)。
- [x] **结构搬迁完成(2026-06-29,`dart analyze lib`=No issues found)**:organization-manage 只读核心 8 文件 `git mv`→`lib/citizen/institution/` 并改名(institution_manage_service→**institution_chain_service**、manage_models→**institution_models**、institution_registry→**governance_registry**(+.generated,part 指令对齐)、codec/discovery/account_info_page/admin_list_page);类 `InstitutionManageService`→`InstitutionChainService` 全仓改;~9 consumer 的 package import + 移动文件间相对 import 全部重接。
- [x] **删机构写入口**:删 institution_multisig_create_page/close_page;account_list_page 创建入口改只进个人多签(_openCreateInstitution 删、菜单简化);account_info_page 删关闭(_confirmClose/_openClosePage/关闭菜单项);qr organizationCreate(0x1105)/Close(0x1101)/CleanupRejected(0x1104)+三映射删。
- [x] **死写代码已清(2026-06-29)**:`institution_chain_service` 整体重写为**只读**(删 submitProposeCreate/Close+buildCallData+create事件确认+_signAndSubmit+写 helper/常量+InstitutionInitialAccountInput + 未用 import);保留读方法(fetchAccount(sBatch)/fetchRegisteredInstitutionRefsBatch/fetchCidRegisteredAccount/decodeManageProposalData)。删 `test/governance/organization-manage/`(写测试 + 旧前缀 codec 测试,属旧结构残留;新路由读测试列 follow-up)。
- [x] **codec 路由已落(2026-06-29)**:`multisig_storage_codec` 的 institutionKey/institutionAccountKey/accountRegisteredCidKey 加 `palletName` 参;`institution_chain_service` 反查 `AccountRegisteredCid` 对 `managePallets` **双查取首命中**、cid 键贯穿命中 pallet;`decodeManageProposalData` 的 `org-mgmt` → 双 tag `pub-mgmt`/`pri-mgmt`。`OrganizationManage` 字面在 citizenapp 代码侧归零(仅剩 3 条"取代/删除"说明注释)。

**Chunk 2 — 多签转账 → transaction(完成,2026-06-29)**:
- [x] `lib/citizen/proposal/transaction/`(10 文件)→ `git mv`→`lib/transaction/multisig-transfer/`;全 package 路径 `citizen/proposal/transaction/`→`transaction/multisig-transfer/` 重写(自引 + 外部 consumer + test);转账读账户经 citizen/institution 读核心(import 不变,package 路径)。
- [x] personal-manage 不动(留 transaction)。
- [x] **`dart analyze`(lib+test)=No issues found**。

**文档(2026-06-29)**:unified-naming.md(citizenapp 目录登记行重写为 citizen/institution + transaction/multisig-transfer)、unified-protocols.md(多签转账归口 + 禁令路径)、CITIZENAPP_TECHNICAL.md(目录树/导航/创建入口/错误模块名)、小模块文档(ONCHAIN/MULTISIG_TRANSFER_APP/PERSONAL_MANAGE_CITIZENAPP)已更。**遗留**:`05-modules/citizenapp/governance/GOVERNANCE_TECHNICAL.md`(30 引用,大描述文档)已加过时 banner 指向新结构,整篇按新结构重写列 follow-up。

## 总结(2026-06-29 完成)
node 链端(CRITICAL)+ onchina 注释 + 脚本 + 权威文档(unified-protocols/naming)+ CITIZENCHAIN_TECHNICAL + node 模块文档全部清理;citizenapp 按"机构管理(citizen/institution)/ 交易(transaction:multisig-transfer+personal-manage+on/off-chain)"重组,机构创建/关闭收归 onchina,公私权机构链访问按机构码路由 PublicManage(32)/PrivateManage(33)。`cargo check -p node` 绿 + `dart analyze` 绿。残留 follow-up:① citizenapp 路由读核心单测;② GOVERNANCE_TECHNICAL 整篇重写;③ NODE_CLEARING_BANK 整篇按 B0 重写(已加 banner)。
