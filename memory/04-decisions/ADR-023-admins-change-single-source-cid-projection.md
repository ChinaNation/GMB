# ADR-023:admins-change 链上唯一真源 + CID 纯投影

状态:Accepted(2026-06-21)。**本 ADR 是「管理员真源」的唯一架构真源**,取代一切旧的 CID 直写/本地播种叙述。
关联:[[feedback_no_compatibility]]、[[feedback_pubkey_format_rule]]、[[feedback_chainspec_frozen]]、[[feedback_registry_regen_after_genesis]]、[[project_admin_single_source_admins_change_2026_06_21]]、ADR-008、ADR-015、ADR-017。

> **注(2026-06-22)**:本 ADR §1 描述的 `org`(`ORG_NRC..ORG_OTH` 6 类)分类模型**已被 [[ADR-025]] 取代**。机构分类唯一真源现为 CID 机构码 `institution_code`;ORG_PUP/ORG_OTH 已合并为机构账户码谓词 `is_institution_code`,固定治理档 NRC/PRC/PRB/FRG/NJD 走 `is_fixed_governance_code`,个人多签(原 ORG_REN)走 `is_personal_code`(PMUL)。下文 `org` 叙述仅作历史决策保留,新开发以 ADR-025 为准。

---

## 0. 一句话

所有管理员(治理机构 / 个人多签 / 公权机构 / 私权机构)的唯一真源 = 链上 `admins-change::AdminAccounts`。CID(citizencode)是它的**纯投影**:postgres `admins` 表只是登录缓存,不再是第二真源。china_zf 只喂链创世,绝不进 CID 运行时。

2026-07-12 当前目标补充：机构管理员真源只保存管理员钱包账户集合 `admins`；机构岗位定义与岗位任职归 `entity`。具体职责与授权由对应业务模块依据“机构 + 岗位 + 业务动作”的硬规则判定，不建立通用岗位权限字符串或枚举表。机构管理员是“管理员钱包账户与某机构岗位的有效绑定”，不再由 `AdminProfile` 同时承载姓名、CID、岗位、任期和来源。个人多签保持独立，不纳入机构岗位模型。

## 1. 钉死的模型(经多轮收敛,不得回退)

- **org 分类用现有 6 类**(`citizenchain/runtime/votingengine/src/types.rs:16-27`),不新增:
  `ORG_NRC=0` 国家储委会 / `ORG_PRC=1` 省储委会 / `ORG_PRB=2` 省储行 / `ORG_REN=3` 个人 / `ORG_PUP=4` 公权机构(政府/教育/司法/立法/监察)/ `ORG_OTH=5` 私权机构(公司/银行/基金)。
- **市注册局不是特例**:身份注册局是政府类公权机构 = `ORG_PUP`,与联邦注册局/公安局/教育局完全同构。**禁止**为它新增 org 或链上专属类型。
- **不新建账户**:所有机构(含每市的市注册局,`gov/service.rs` 的 `CITY_REGISTRY` 模板)都已由行政区生成 + `accounts/derive.rs::derive_account` 派生账户,链端/CID 同一派生口径。
- **递归 D1-a 授权**:上级注册局给它登记的机构设"初始管理员",之后该机构**自治**(`propose_admin_set_change`, `who ∈ 本机构 admins`)。上级不能事后改下级 admin 集。
  - 联邦注册局:创世内置(china_zf 215 管理员,org=PUP,account=`ZS001-GZF0P`)。
  - 市注册局:由联邦注册局创建并设初始管理员 → 之后市注册局自治。
  - 普通机构:由市/联邦注册局创建并设初始管理员 → 之后机构自治。
- **省份/城市 scope = 纯 CID 元数据,不上链**。链上联邦注册局就一个扁平账户、不带省份;"每省 5 人"只是 CID 用于管辖过滤(联邦看本省、市看本市),顺序源自 china_zf 注释。
- **registry_org_code(FEDERAL/CITY)是 CID 标签**(管理员挂在联邦注册局还是某市注册局),不是链上概念。
- **冷签**:所有管理员变更走 CitizenWallet QR_V1 冷签;`ONCHINA_SIGNING_SEED_HEX` 只是凭证签名密钥,**绝不代签**机构管理员动作。

## 2. 真源:链上 AdminAccounts

`admins-change::AdminAccounts`(`citizenchain/runtime/admins/admin-management/src/lib.rs:227`),StorageMap key=机构主账户(32B),value=`AdminAccount { org, kind, admins: BoundedVec, status, threshold, creator, created_at, updated_at }`。`propose_admin_set_change`(call_index=0)一个 call 覆盖 add/remove/replace。链端授权天然防越权:`who ∈ 当前 admins`,CID 不持机构私钥。

## 3. CID 投影:双通道(缺一不可)

> 决定性事实:创世 `build()`(`lib.rs:306-356`)直写 `AdminAccounts` **零 `deposit_event`**。纯 indexer 永远看不到创世写入 → 必须配启动快照。

- **通道① 启动全量快照**:subxt `storage().at(finalized).iter("AdminsChange","AdminAccounts")` 全量迭代 → 以 AdminAccounts key 解析对应机构 CID 元数据,只投影 `registry_org_code=FEDERAL_REGISTRY/CITY_REGISTRY` 的注册局 → upsert。失败仅 warn 不阻断 serve。
- **通道② indexer 增量**:监听 finalized 的 `AdminSetChanged/AdminAccountActivated/AdminAccountClosed`(事件只含 `admins_len` 不含名单 → 必须按 account 做 storage 点查取最新 admins)→ upsert/删除。

**安全栏(评审阻塞项)**:reconcile 只在快照完整无错才删 + 阈值守卫(联邦 admin<200 放弃删除)+ `federal_registry_scope` CASCADE 走单事务;冲突键统一 `lower(admin_account)` + 0x 小写 hex;高危写动作 commit 前实时点查防窗口期提权;`delete_admin_runtime_state_conn` 包单事务。

## 4. 两类读,别混

- **登录表 `admins`**(postgres):只投影注册局操作员(FEDERAL/CITY)——他们才登录 CID 控制台。
- **机构详情"管理员 tab"**:对该机构 `AdminAccounts[account]` 实时点查显示(任何机构都一样,永远最新),不落登录表。普通 PUP 机构(公安局等)管理员不进登录表。

## 5. Bootstrap 与止血

- **~~P0 止血 seed-federal-admins~~ 已退役(2026-06-30 更新)**:`seed-federal-admins` CLI + 本地 215 平铺播种 + `federal_registry_scope` 表全删;FRG 管理员「全走链读」链上 `PublicAdmins::FederalRegistryProvinceGroups[省码]`(每节点单省),无本地 china_zf 播种兜底。
- **重新创世顺序(2026-07-03 更新)**:重生 chainspec(含所有公权机构写入 `PublicManage::Institutions/InstitutionAccounts`、FRG 43 省×5 写入 `FederalRegistryProvinceGroups`、NRC/PRC/PRB/NJD 写 `AdminAccounts`)→ 起链 → OnChina 执行 `sync-gov` 从链上同步公权机构投影 → 执行 `audit-chain-catalog` 全量验收 → 重跑 `generate_public_institution_bundle.mjs`([[feedback_registry_regen_after_genesis]])→ FRG 管理员经链上省组直接扫码登录(无需 seed)。

## 6. 链端:PUP 自治 + 机构注销 close + 创世封存(`20260621-admins-change-builtin-pup-selfgovern`)

**6.1 PUP 自治(方案A,admins-change 已落地 2026-06-21)**:PUP 内置保持 `BuiltinInstitution`,放宽 `ensure_account_kind_matches_org`(接受 ORG_PUP)+ `validate_admins_len_for_account`(PUP 走可变上限,NRC/PRC/PRB 仍精确)。联邦注册局可发 `propose_admin_set_change` 自治。cargo test 43/43。

**6.2 创世封存(已落地)**:`ProtectedGenesisAccounts` StorageMap,`build()` 写入全部 china/ 机构主账户(联邦注册局/治理/顶层政府司法监察教育立法=CID 根基,永不可注销);`is_genesis_protected` 供关闭入口调。

**6.3 机构注销 close(organization-manage 已落地 2026-06-21)**:统一模型——**管理员属于机构不属于账户**(`resolve_admin_account_for_account` 任意账户解析到机构主账户的管理员集),故一个 `propose_close` 按被关账户 role 分流:Main=注销整机构(级联关全部账户余额→同一 beneficiary + 关 AdminAccount)、非主=只删该账户(不动 AdminAccount)。真源方向不循环——注册局在 CID 设注册局域注销态(区别于链投影 RevokedOnChain)+ 签发注销凭证(对称创建凭证,身份注册局签,payload 用 `OP_SIGN_DEREGISTER=0x14`,target+scope 入签名防重放);机构【自己的管理员】冷签发起 propose_close 带凭证(链上提案只能机构 admin 发起);链验:发起人∈机构admins + 凭证有效 + `UsedDeregisterNonce` 防重放 + `ensure_closeable` 三层硬闸(`is_genesis_protected` / `org∈{NRC,PRC,PRB}` / 724)。注销动作走身份注册局 PasskeyChallenge 最严档。机构 admin 拒签则链上账户滞留(非强制处置,强制吊销需另设治理通道,本期不做)。链端 cargo test 29/29 + 全 runtime check 通过。**身份注册局后端已实现**(2026-06-21):`OP_SIGN_DEREGISTER=0x14` 凭证签发器(golden 字节锁)+ `InstitutionDeregister/...AccountDeregister`(PasskeyChallenge)动作派发(查存/管辖/拒根基/derive target/建凭证/写注销态)+ `institution_deregistrations` 表 + `/deregistration-info` 下发路由;cargo test 64/64、0 warning、表实测落库。**CitizenWallet decoder + 身份注册局前端入口已实现**(2026-06-21):decoder 机构 `propose_close`(pallet 17/call 1)走专用 `_decodeProposeCloseInstitution`(解 scope+account+beneficiary 后 skip 3 个 Vec<u8> 凭证字段 + issuer_main_account/signer_pubkey + 签名尾,个人 66 字节解码不动,dart analyze 0);身份注册局前端 `gov/GovDetailPage.tsx` 机构信息卡加 `<Popconfirm>注销机构` 入口(门控 `canWrite && ACTIVE && created_by!=='SYSTEM'`,复用 `runPasskeyChallengeGrant('INSTITUTION_DEREGISTER')`),`AdminActionType` 补 `INSTITUTION_DEREGISTER/INSTITUTION_ACCOUNT_DEREGISTER`(tsc 0)。**仅剩 follow-up**:node 调用面随 propose_close 签名适配、runtime 级集成测试、运行期端到端冒烟。

**6.4 CID 前端登录/列表两处修复(2026-06-21)**:① 无 passkey 管理员登录默认落【机构信息】tab 看不到待绑红点 → `InstitutionDetailNavLayout` 加 `initialActiveKey` prop,`GovDetailPage` 在 `passkey_bound===false` 时传 `'admins'` 直达管理员列表。② 管理员姓名列裸显哈希(0xd641…) → 根因 `backend/admins/seed.rs` 联邦 215 admin `admin_display_name` 写空串,改 `format!("{province}联邦注册局管理员{seat}")`,开发库实测哈希→人名。

## 7. 实施阶段

- Phase 0(done):seed CLI 止血。
- Phase 1(open `20260621-admins-chain-sync`):双通道投影 + 链端 kind 修复 + 省份反查改 `city_name→province`。
- Phase 2(open `20260621-admins-action-deprecate`):废弃 CID 直写改冷签 QR + 前端 + CitizenWallet decoder。

## 8. 对抗自检留存的两个先堵的洞

1. 投影只 upsert + reconcile 阈值守卫,防误删市注册局/联邦行致 scope JOIN 断裂掉线。
2. 窗口期被撤销管理员仍可登录(可接受陈旧缓存),但**高危写动作**必须 commit 前实时点查链上 admins,链不可用则降级拒绝。
