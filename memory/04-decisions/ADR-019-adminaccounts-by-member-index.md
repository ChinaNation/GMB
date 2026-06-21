# ADR-019:AdminAccounts 按成员反向索引(轻节点多签发现 O(n)→O(1))

- 状态:提案(2026-06-13)。**需 runtime 升级**(走链上 setCode),ADR-018"不动链端"范围外单列。
- 关联:[[ADR-018]](ADR-018-citizenapp-unified-query-low-load.md) §九 L2;[[ADR-017]](ADR-017-finalized-unification.md)
- 触发:citizenapp"我的多签"(机构多签 + 个人多签)发现当前只能**全表扫** `AdminsChange::AdminAccounts`。

## 一、问题
- `AdminAccounts: StorageMap<Blake2_128Concat, AccountId(账户地址), AdminAccount{org,kind,admins[],...}>`(`runtime/governance/admins-change/src/lib.rs:223`)。
- key 是账户地址,admins(管理员钱包 pubkey 列表)藏在 value 里 → "某钱包是哪些账户的管理员"**无反向索引**。
- 客户端只能 `getKeysPagedFinalized` 拉全表 + 逐条 `fetchStorageBatch` 解码 + 匹配 `admins` 含本地 pubkey。轻节点 O(n),随账户总量线性增长。
- ADR-018 §九 L1(卡②)已把机构/个人双扫合一(降一半),但仍是全表扫——治标不治本。

## 二、方案(待评审)
- 新增反向索引,二选一:
  - `AdminAccountsByMember: StorageDoubleMap<Blake2_128Concat admin_pubkey, Blake2_128Concat account_addr, ()>`(枚举友好,无上限压力);或
  - `MemberAdminAccounts: StorageMap<admin_pubkey, BoundedVec<account_addr, MaxAccountsPerMember>>`(单键取全列表,有上限)。
  倾向 DoubleMap(无上限、增删 O(1)、轻节点对 `admin_pubkey` 短前缀枚举即可)。
- 写路径:`register` / `create` / `execute` / `close` / admins-change 增删管理员时,同步维护索引(管理员变更 = 旧成员删项 + 新成员加项)。
- 读路径:轻节点对每个本地钱包 `getKeysPaged(前缀=AdminAccountsByMember[admin_pubkey])` 直接拿到其管理的账户地址 → 再精确读 `AdminAccounts[addr]` / `AccountRegisteredSfid[addr]` 取详情。**全程无全表扫**。

## 三、迁移
- 全新创世无历史数据,index 随新写入自然建立(`feedback_chain_in_dev`,无需 migration)。
- 若上线后才加:走 pallet `StorageVersion` migration 回填(遍历 `AdminAccounts` 反建索引),`feedback_no_spec_version_questions` / `feedback_storage_rename_needs_migration` 适用。

## 四、影响
- 链端:admins-change pallet 加 1 storage + 5 处写路径 + try-runtime hook + spec_version bump(setCode,chainspec 不重生 — `feedback_chainspec_frozen`)。
- 客户端(citizenapp):卡② 的 `admin_accounts_scan_service` 全表扫改为按本地钱包精确枚举;删全表扫路径。

## 五、待办
- [ ] 评审反向索引形态(DoubleMap vs Map<BoundedVec>)与上限取舍。
- [ ] 确认写入点全覆盖(register / create / execute / close / admins-change 各增删管理员处)。
- [ ] benchmark 权重 + try-runtime 校验。
- [ ] 立任务卡 + runtime 升级流程(spec_version、setCode)。
- [ ] 客户端切读路径(依赖卡② 已落地的 scan service)。

## 六、风险
- 写路径遗漏 = 索引与主表漂移(管理员显示缺漏)→ 必须覆盖全部增删点 + 单测每条路径。
- 上限方案(BoundedVec)下成员管理账户数封顶;DoubleMap 无此问题,优先。
