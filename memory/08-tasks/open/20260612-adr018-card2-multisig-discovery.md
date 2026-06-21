# 任务卡:卡② 多签发现降载(三仓库+共享底座)

属 ADR-018 §九(2026-06-13 架构修订)。原标题"listSfidAccounts 长前缀整表化"已废弃。

状态:**代码完工(2026-06-13)**,analyze 0 + flutter test 197/197 全过;真机 logcat 验证待 user 跑。

## 背景(纠正原卡定义)
审计确认:
- `listSfidAccounts` 是**死代码**(全仓零调用),且 `SfidRegisteredAccount` 正向枚举是 R1 禁区。
- citizenapp 真正发现机构/个人多签走**全表扫 `AdminsChange::AdminAccounts`**(两个发现服务各扫一遍,同一张表扫两次=纯浪费)。
- 多签 = "我的"(我的钱包某钱包是管理员才显示);治理机构走注册表、公权机构走后端 catalog(卡⑥),均与本卡无关。

## 完工清单
- [x] 新建 `governance/shared/admin_accounts_scan_service.dart`:单次 `getKeysPagedFinalized`(AdminAccounts 短前缀整表,翻页 256)+ 批量 `fetchStorageBatch`(chunk 100)+ `AdminAccountStorageCodec.tryDecode` + 提 addr → emit `ScannedAdminAccount{addr,org,kind,admins}`;含纯函数 `filterMine`(kind+org 白名单+本地钱包)供两模块复用。
- [x] 新建 `governance/shared/multisig_discovery_coordinator.dart`:统一节流(30min,单 key `multisig_discovery_last_at_ms`)+ 本地钱包读取 + **单次扫描** + 分发两类 `processScanned`。
- [x] `InstitutionDiscoveryService` / `PersonalManageDiscoveryService` 重构为纯 `processScanned(scan, myPubkeys)`:筛分流 + 反查 + upsert + 孤儿校验;删除各自的扫描/节流/钱包/hex 重复段。两模块目录边界不动。
- [x] 机构账户命名走 `AccountRegisteredSfid` **精确批量反查**(新增 `fetchRegisteredInstitutionRefsBatch`),不正向枚举。
- [x] 个人多签元数据同样批量反查(新增 `fetchPersonalMetasBatch`),消除个人侧 N+1。两处批量均加"整体失败不误删孤儿"保护。
- [x] 删除死代码:`listSfidAccounts` + 孤儿 `_utf8Decode` + 无用 `smoldot_client` import;`fetchRegisteredInstitutionRef`(单)/`fetchPersonalMeta`(单)被批量取代后亦删除。
- [x] 列表页 `_runBackgroundDiscovery` 改调协调器一次(替代两服务各调一次);进度文案合并为"多签扫描"。
- [ ] **(顺延卡⑤)** 列表余额批量化 + ChainReadCache:发现路径只填 Isar 实体(name/sfid/admins),余额读属卡⑤ 范畴,本卡不引入缓存层。

## 单测
- [x] `test/governance/shared/admin_accounts_scan_service_test.dart`:`filterMine` 分流(kind/org 白名单/多钱包 any/空集)。
- [x] `test/governance/shared/multisig_discovery_coordinator_test.dart`:空钱包短路 / 30min 节流 / lastDiscoveryAt 持久化。
- [x] 删除两份过时服务测试(throttle/empty 逻辑已迁协调器);`admin_account_storage_codec_test.dart` 保留。
- [ ] 反查聚合单测:批量反查走真链 storage,留端到端覆盖(分流逻辑已由 filterMine 覆盖)。

## 验收
- [x] flutter analyze 0 + flutter test 197/197 全过
- [x] 旧代码/文档/注释清理无残留(`listSfidAccounts`/`discoverByMyWallets` 零引用)
- [ ] 真机:机构多签/个人多签列表正常显示(仅我参与的);logcat 验证 AdminAccounts 全表扫 **2 次→1 次**(待 user 装机)

## 不做(边界)
- 不做公权机构目录(卡⑥);不做 `AdminAccountsByMember` 链上反向索引(ADR-019);不动链端;不整表化 `SfidRegisteredAccount`。

## 改动文件
- 新增:`lib/governance/shared/admin_accounts_scan_service.dart`、`lib/governance/shared/multisig_discovery_coordinator.dart`、`test/governance/shared/{admin_accounts_scan_service,multisig_discovery_coordinator}_test.dart`
- 改:`institution_discovery_service.dart`、`personal_manage_discovery_service.dart`、`institution_manage_service.dart`、`personal_manage_service.dart`、`institution_account_list_page.dart`
- 删:`test/governance/organization-manage/institution_discovery_service_test.dart`、`test/governance/personal-manage/personal_manage_discovery_service_test.dart`
