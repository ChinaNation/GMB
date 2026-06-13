# 任务卡:卡② 多签发现降载(三仓库+共享底座)

属 ADR-018 §九(2026-06-13 架构修订)。原标题"listSfidAccounts 长前缀整表化"已废弃,见下。

## 背景(纠正原卡定义)
审计确认:
- `listSfidAccounts`([institution_manage_service.dart:298-357](../../../wuminapp/lib/governance/organization-manage/institution_manage_service.dart)) 是**死代码**(全仓零调用),且 `SfidRegisteredAddress` 正向枚举是 R1 禁区。
- wuminapp 真正发现机构/个人多签走**全表扫 `AdminsChange::AdminAccounts`**(`institution_discovery_service.dart:111` + `personal_manage_discovery_service.dart:89`),同一张表扫两遍 = 纯浪费。
- 多签 = "我的"(我的钱包某钱包是管理员才显示);治理机构走注册表、公权机构走后端 catalog(卡⑥),均与本卡无关。
- 共享解码器 `governance/shared/admin_account_storage_codec.dart` 已存在,两发现服务已共用。

## 目标(L1 纯客户端零链改)
把"机构多签 + 个人多签"发现收敛为**共享单次扫描 + 精确反查命名**,删死代码。机构/个人两模块目录边界不动(`project_wuminapp_module_boundary_2026_05_09`)。

## 任务清单
- [ ] 新建 `governance/shared/admin_accounts_scan_service.dart`:单次 `getKeysPagedFinalized`(`twox128(AdminsChange)+twox128(AdminAccounts)` 前缀,翻页 256,ensureSynced 后钉 finalized 哈希 — `feedback_smoldot_keyspaged_pin_hash`)+ 批量 `fetchStorageBatch`(chunk 100)→ 用现有 `AdminAccountStorageCodec.tryDecode` 解码 → emit `{addr(取 key 末 32B), org, kind, adminPubkeysHex}`。一次扫描两类共用。
- [ ] `organization-manage` 机构多签 repo / `personal-manage` 个人多签 repo 改为**订阅共享扫描**,各自按现有过滤口径分流(机构:kind=2 && org∈{4,5};个人:kind=1/org=3;均要求 adminPubkeysHex 含本地任一钱包 pubkey),各写各的 Isar(`InstitutionEntity`/`PersonalDuoqianEntity`)。扫描服务只负责"取+解码+emit",过滤与持久化各 repo 自理 → 不破模块边界。
- [ ] 机构账户命名走 `AddressRegisteredSfid[addr]` **精确批量反查**(`fetchStorageBatch`)→ 解码 (sfid_number, account_name) → 按 sfid_number 聚合。**不正向枚举** `SfidRegisteredAddress`。
- [ ] 删除 `institution_manage_service.dart` 的 `listSfidAccounts`(约 298-357 行,含 doc 注释)及一切相关注释(`feedback_no_compatibility`,Grep 确认零引用)。
- [ ] 列表余额批量化(`fetchFinalizedBalances`),接卡⑤ `ChainReadCache`;若卡⑤ 未先行,留接口占位 TODO。
- [ ] Isar 永久缓存 + 节流(现 30min)+ 增量;首屏本地、后台**单次**刷新(而非两服务各刷)。

## 单测
- [ ] `admin_accounts_scan` 分流:混合 kind 夹具 → 机构/个人正确分流 + 本地钱包(多钱包)匹配过滤。
- [ ] 反查聚合:多账户同 sfid → 按 sfid_number 聚合正确、account_name 还原正确。
- [ ] 删除后 `listSfidAccounts` 零引用。

## 验收
- [ ] flutter analyze 0 + flutter test 全过
- [ ] 真机:机构多签/个人多签列表正常显示(仅我参与的);logcat 验证 AdminAccounts 全表扫 **2 次→1 次**
- [ ] 旧代码/文档/注释清理无残留

## 不做(边界)
- 不做公权机构目录(卡⑥);不做 `AdminAccountsByMember` 链上反向索引(ADR-019);不动链端;不整表化 `SfidRegisteredAddress`。
