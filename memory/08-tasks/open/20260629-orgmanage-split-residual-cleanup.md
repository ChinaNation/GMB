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
### A. node 链端(CRITICAL,用户优先)
- [ ] `node/src/transaction/offchain_transaction/institution_read/chain.rs` 把 `OrganizationManage` storage 前缀改路由 `PublicManage(32)`/`PrivateManage(33)`(line 319 Institutions map_key + line 428 InstitutionAccounts 前缀);双查或按 cid 派生。
- [ ] `node/src/governance/proposal.rs` MODULE_TAG `b"org-mgmt"`→双 TAG `b"pub-mgmt"`/`b"pri-mgmt"`(line 30 常量 + 234 函数,改名 is_institution_manage_proposal)。
- [ ] `api_client.dart:376` 注释旧 call 名(随 citizenapp 一并)。

### B. 文档 + 脚本 + onchina 注释(机械)
- [ ] unified-protocols.md:P-STORAGE-002 整节、P-CRED-001、P-STORAGE-004 真源/生产者/消费者/必跑测试改 public-manage/private-manage。
- [ ] unified-naming.md:145/456/471 命名登记改两 pallet/删已删 node 目录。
- [ ] CITIZENCHAIN_TECHNICAL.md §9.3;node 模块文档(NODE_CLEARING_BANK / GOVERNANCE / admins-change / CID_CONFIG)。
- [ ] scripts/load-context.sh:134/140 case 路由改 entity/public-manage|private-manage。
- [ ] onchina 注释 chain_runtime.rs:278、actions.rs:41 改真源 crate 名。

### C. citizenapp 三分重构(设计锁定)
**决策**:① 3 薄模块 + 1 共享核心;② 机构创建/关闭(公权+私权)**全部删**(收归 onchina),public/private-manage 仅读/浏览/转账。个人多签反查标注拆到 personal-manage。
- [ ] **删写入**:`institution_multisig_create_page.dart`、`institution_multisig_close_page.dart`、service 的 submitProposeCreate/Close+build call data+create 事件确认+_signAndSubmit+写编码 helper+InstitutionInitialAccountInput;qr_protocols `organizationCreate/Close/CleanupRejected`(0x1105/0x1101/0x1104)+ fromDecodedAction 三映射;写测试 institution_manage_service_test.dart;入口 `_openCreateInstitution`(account_list_page)+ close 入口(account_info_page)。
- [ ] **共享核心**(`lib/citizen/shared/institution/` 或 institution-core):institution_manage_models、multisig_storage_codec(**修 pallet 前缀路由 32/33**)、读服务(fetchAccount(sBatch)/fetchRegisteredInstitutionRefsBatch/decodeManageProposalData 修 org-mgmt→pub/pri-mgmt)、institution_discovery_service、findInstitutionByAccountId 主体。
- [ ] **public-manage**:governance_institution_registry.generated + institution_registry(国储会/省储会/省储行)。
- [ ] **private-manage**:私权机构浏览特有部分。
- [ ] **personal-manage**(已存)+=findInstitutionByAccountId 的 `个人多签` 反查标注分支。
- [ ] 重接 ~16 个 consumer 的 import;`dart analyze` 绿。
