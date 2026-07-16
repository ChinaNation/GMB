# ADR-018:citizenapp 全应用统一字段查询 + 降低全节点依赖

- 状态:当前。提案统一查询、批量读取、finalized 缓存、管理员扫描和机构目录降载均已落地。
- **2026-07-12 最终修订**:公权机构目录采用“发布期 finalized 链快照 + Isar 本地索引”。生成器直接读取 `PublicManage::Institutions/InstitutionAccounts`；公民、管理员和清算行读取各自链上 storage。
- 关联:[[ADR-017]](ADR-017-finalized-unification.md)
- 触发:机构详情提案列表为空(根因实测:轻节点对长前缀 keysPaged 返回空);user 要求把"统一字段 + 降载"作为基础规则,整改整个 citizenapp。

---

## 一、关键技术结论(审计后确诊,纠正常见误判)

轻节点 smoldot 的两种访问模式必须分清:

| 访问模式 | 轻节点表现 | 例子 |
|---|---|---|
| **精确整键 `fetchStorage(完整key)`** | ✅ 正常(单 key Merkle 证明) | `ActiveProposalsBySubject[ProposalSubject]`、`InternalVotesByAccount[pid,account]`、`System.Account[account]` 余额 |
| **keysPaged 前缀扫描,前缀嵌长 K1(ProposalSubject / blake2+cid)** | ❌ 返回空(证明拉不全,静默空) | `ProposalsByCid[cid_number]`、`InstitutionAccounts[blake2(cid)+cid,...]` |
| **keysPaged 短前缀(整表 / ≤2B K1)** | ✅ 正常 | `ProposalsByYear[year]`、`ProposalsByCode[institution_code]`(机构码反向索引,见 [[ADR-025]])、`AdminAccounts` 整表 |

**所以"功能性坏"只有长前缀 keysPaged 这一类(2 处);其余全是"能用但费节点"的负载问题。** 不需要改链端 storage 结构。

---

## 二、三条基础规则(本 ADR 确立,写入 agent-rules)

- **R1 统一字段查询**:列表类一律"短 key 索引(`ProposalsByYear`/整表)取一次 → 客户端按已解码字段过滤"。**禁止**业务代码对嵌 32B account / `blake2(x)+x` 的长 K1 做 keysPaged 前缀扫描。精确整键读取不受限,可继续用。
- **R2 降低全节点依赖**:① 多 key 一律 `fetchStorageBatch`/`fetchFinalizedBalances`,禁止循环内逐条;② 同一数据跨页面取一次进共享缓存复用;③ 链状态页用 finalized 订阅驱动,禁止 Timer 轮询查链;④ 能本地算的不联网。豁免=交易提交管线(nonce/dry-run/submit/runtime-version/genesis)+ UI 倒计时。
- **R3 外部 HTTP 缓存**:仅 Cloudflare Worker、GitHub 更新等真实外部服务允许按业务设置 TTL；公民、机构、管理员和清算行身份不得以 HTTP 缓存替代链读取。

---

## 三、统一架构(三个收口层)

### 1. `ProposalFeedCache`(新增,提案统一查询层)
所有提案页面共用一条通路,替代散落的 4 个反向索引:
```
ProposalsByYear[currentYear](短key,可用) → getKeysPagedFinalized → ids
   → _fetchProposalsForIds(ids)(已全批量:meta+displayId+detail)
   → 共享缓存(TTL 20s)
   → 提案页 filter defaultCodes ∪ subscribedCidNumbers / 机构详情 filter subject_cid_numbers 包含机构 CID / 个人多签 filter PersonalAccount
```
依据:`ProposalsByYear` 链端对每个提案无条件写、终态清理时移除；`ProposalMeta` 已解码 `kind/stage/status/internal_code/actor_cid_number/execution_account/subject_cid_numbers`，客户端过滤零联网。
**载荷:广场(原 3 次 ByOrg)+ 机构详情(原坏的 ByInstitution)+ 个人 → 同周期共用一份缓存 = 1 次按年取 + 1 次批量详情。**

### 2. `ChainReadCache`(新增,余额/storage 共享缓存,挂在 ChainRpc 层)
- 按 `finalizedBlockHash : storageKey` 缓存,TTL 短(同块内复用),换块自然失效。
- 余额、固定 storage(CidMainPubkey 已有内部缓存,纳入统一)走此层。
- 审计确认收口层级:`页面 → ChainRpc(批量入口) → SmoldotClientManager(原生窗口)`,缓存插在 ChainRpc。

### 3. 批量与订阅(改造现有,不新增类)
- 所有"循环内逐条链读"改 `fetchStorageBatch`/`fetchFinalizedBalances`。
- 所有"Timer 轮询查链"改 finalized 头订阅(复用 `ChainEventSubscription`)。

---

## 四、全应用整改清单(按类别,含确切位置)

### A. R1 必改 — 长前缀 keysPaged(功能性坏,2 处)
| 位置 | 索引 | 改法 |
|---|---|---|
| `transaction/multisig-transfer/multisig_transfer_service.dart:290` fetchProposalIdsByInstitution | `ProposalsByCid[cid_number]` | 删,机构详情改走 ProposalFeedCache 按年取+过滤(**本次 bug 收口**) |
| 已删除的 `listCidAccounts` | 旧重复 CID→账户表 | 审计确认是死代码；目标态账户清单只读 `InstitutionAccounts[(cid_number, account_name)]` 正向真源，不恢复旧枚举接口 |

### B. R2 必改 — N+1 循环逐条链读(费节点,核心降载)
| 位置 | 现状 | 改法 |
|---|---|---|
| `rpc/chain_tx_monitor.dart`(余额回调) | 每事件/每钱包逐条 `fetchFinalizedBalance` | 批量 `fetchFinalizedBalances([钱包])` |
| `governance/.../institution_detail_page.dart:508` | 逐管理员查余额 | 收集后一次 `fetchFinalizedBalances` |
| `governance/.../institution_detail_page.dart:796` | 逐转账受益人查余额 | 同上批量 |
| `governance/organization-manage/institution_manage_service.dart:342/368/380/547/556/564/593` | listCidAccounts/账户详情逐条 `fetchStorage` ×7 | `fetchStorageBatch(keys)` |
| `governance/personal-manage/personal_manage_service.dart:180/197/205/357` | 个人账户逐条 `fetchStorage` ×4 | `fetchStorageBatch` |
| `citizen/vote/vote_view.dart:308` hasUnvotedWallet | 循环内逐提案查投票 | 新增 `hasUnvotedWalletsBatch` 批量 |
| 调 `fetchAdminVote`(单)/`balance_guard`(单)的循环点 | 逐条 | 强制走已存在的 `fetchAdminVotesBatch`/批量余额 |

### C. R2 应改 — 轮询改订阅 / 降频
| 位置 | 现状 | 改法 |
|---|---|---|
| `ui/chain_progress_banner.dart:74/104` | 6s 轮询 `fetchChainProgress` | 订阅 finalized 头 / 仅 syncing 时轮询且延长 |
| `transaction/multisig-transfer/multisig_transfer_detail_page.dart:142/520` | 20s 轮询待投票状态 | finalized 头订阅驱动刷新 |
| `governance/institution_manage_detail_page.dart:455` | 20s 轮询 | 同上 |
| (qr_sign_session 1s、pin_input 1s = UI 倒计时,**不查链,豁免**) | — | 保留 |

### D. R2 应改 — 共享缓存 / 去重
| 位置 | 现状 | 改法 |
|---|---|---|
| 广场 `citizen/vote/vote_view.dart:156-158` | 3 次 `ProposalsByCode`(机构码反向索引,见 [[ADR-025]]) | 接 ProposalFeedCache,与机构详情共用 |
| `wallet/.../wallet_onchain_balance_card.dart:59` 单账户总额 | 单查无缓存 | 接 ChainReadCache |
| `governance/.../institution_account_info_page.dart:150/254`、`personal_manage_account_info_page.dart:171/274` | 同地址多次单查 | 去重/缓存 |
| 创建/关闭/转账前各页 `fetchFinalizedBalance` 单查 | 无缓存 | ChainReadCache |

### E. R3 应改 — 外部 HTTP 缓存
| 位置 | 改法 |
|---|---|
| `update/app_update_service.dart:92` GitHub release | 低频,加短缓存即可 |

公民、机构、管理员和清算行不属于 E 类：它们已经统一为 finalized 链读取，目录型机构数据由发布期链快照和 Isar 提供首屏。

### 豁免(不改)
交易提交管线:`fetchNonce`/`fetchRuntimeVersion`/`fetchGenesisHash`/`fetchLatestBlock`/`fetchFinalizedBlock`(提交用)/`submitExtrinsic*`/dry-run;`ChainTxMonitor` finalized 订阅;UI 倒计时 Timer。精确整键 `fetchStorage`(ActiveProposalsBySubject/InternalTallies/AdminSnapshot/clearing-bank 等)正常,只在被循环调用时归入 B 类批量,本身不动口径。

---

## 五、链端边界
`ProposalsByCid` 等反向索引保留在 runtime(桌面端全节点可正常用);仅约束轻节点客户端不查长前缀。2026-07-02 起提案归属真源统一为 `subject_cid_numbers`,机构码只做分类,订阅过滤按机构 CID 命中。

## 六、验证
1. `flutter analyze` 0 + `flutter test --concurrency=1` 全过;新增 ProposalFeedCache 过滤单测(多机构/多 org/joint 夹具)。
2. 落地前手机 logcat 实测确认 A 类 2 处确返回空(验后删诊断)。
3. 真机 E2E:机构详情(国家储委会)显示提案;广场/个人多签一致;**logcat 统计改造前后 getKeysPaged/fetchStorage/fetchFinalizedBalance/HTTP 调用计数,验证显著下降**。

## 七、任务拆卡
- 卡①(A+D 提案)`ProposalFeedCache` + 机构详情/广场/个人多签统一接入(收口本次 bug + 最大降载)。
- 卡②(A,**已按 §九 重定义**)多签发现降载:删 `listCidAccounts` 死代码 + 合并机构/个人多签双发现服务为共享单次 AdminAccounts 扫描 + `AccountRegisteredCid` 精确反查命名。纯客户端零链改。
- 卡③(B)全部 N+1 改批量(余额/storage/投票)。
- 卡④(C)三处轮询改 finalized 订阅。
- 卡⑤(D)`ChainReadCache` 余额/storage 共享缓存层。
- 卡⑥ 已被 2026-07-12 链数据统一方案取代：机构目录使用 finalized 链快照，管理员与清算行直接读链。
- 卡⑦ 规则 R1/R2/R3 写入 `memory/07-ai/agent-rules.md` + `memory/05-modules/citizenapp/`。

## 八、风险与回滚
- ProposalsByYear 跨年窗口:取 currentYear,必要时并 currentYear-1(提案 90 天清理,1 年窗口足够)。
- 整表扫 `InstitutionAccounts` / `AdminAccounts`：注册规模上去需分页；dev 期无虞。
- 纯客户端改动,逐卡可独立回滚(git revert),不涉链、不涉创世。

---

## 九、账户发现分层架构(2026-06-13 全栈审计修订)

### 0. 背景:三类机构本质不同,不能共用一条发现通路
全栈审计(链端 storage + 派生原语 + citizenapp 客户端)确认:账户地址全部由唯一原语
`derive_account(op_tag, ss58, payload)`(`primitives/src/core_const.rs:89`)确定性派生,
同输入永远同地址、可离线算。但"枚举账户清单"的来源与范围语义三类完全不同:

| 类别 | 范围语义 | 发现来源 | 缓存模型 | 新鲜度 | 链改空间 |
|---|---|---|---|---|---|
| 治理机构(87:国家储委会1+省储委会43+省储行43) | 目录·全集(与用户无关) | 编译期注册表 `governance_institution_registry.generated.dart` | 静态写死 | 永不变 | 无 |
| 公权机构(动态注册) | 目录·全集(与用户无关) | 发布期 finalized 链快照 | Isar 本地索引 | 随 App 快照发布 | 无 |
| 多签(机构多签+个人多签) | **我的**(我的钱包某钱包是管理员才显示) | 链上扫 `AdminsChange::AdminAccounts` | Isar 永久+增量 | 随钱包/出块变 | ADR-019 反向索引 |

账户类型注记(用户口径,2026-06-13):
- 治理机构账户固定不变:国家储委会 4(主/费/安全基金/两和)、省储委会 2(主/费)、省储行 3(主/费/永久质押)。
- 公权机构:主/费 + N 个自定义(cid_number+account_name 组合),界面"下一步再做"。
- 个人多签:发起人钱包 + account_name,无 cid,仅链上注册。

### 1. 架构:三仓库 + 一共享底座
上层按"发现来源"拆三个 Repository(语义/来源/缓存/新鲜度四维全不同,强行统一 = 全是
`if orgType` 的漏抽象);下层共享派生/余额/账户卡/反查,杜绝重复。

```
lib/governance/
├── shared/                              ← 共享底座(三类都用,单一源)
│   ├── account_derivation.dart          ← derive_account 唯一 Dart 实现(OP_* 全枚举)
│   │                                       现状:仅 personal-manage/personal_account_derive.dart,需归位/补全
│   ├── chain_read_cache.dart(卡⑤)      ← finalizedBlockHash 命名空间的余额/storage 共享缓存
│   │     （实现落 lib/rpc/chain_read_cache.dart:消费者 ChainRpc 在 rpc 层,避免 rpc→governance 层级倒挂;
│   │      挂在 fetchStorageBatch 咽喉透明覆盖全部 finalized 状态读,豁免管线不经此入口天然隔离）
│   ├── account_card / balance UI        ← 账户卡、余额展示(三类同一套)
│   ├── account_registered_cid 反查     ← AccountRegisteredCid[addr]→(cid,name) 精确整键
│   └── admin_account_storage_codec.dart ← 已存在(两发现服务共用解码器)
├── organization-manage/                 ← 治理机构目录 repo(读注册表,零链查) + 机构多签 repo(kind=2)
├── citizen/public/                     ← 公权机构 finalized 链快照 + Isar 目录
└── personal-manage/                     ← 个人多签 repo(kind=1/org=3)
    （shared/admin_accounts_scan_service.dart：单次全表扫 AdminAccounts,emit {kind,addr,org,admins};
      organization-manage 与 personal-manage 各自订阅过滤,不再各扫一遍——既消双扫,又不破模块边界）
```

### 2. 关键结论(纠正原 §四A 对卡②的判断)
- **治理机构目录**:已 100% 注册表驱动,零链查(最优,不动);仅余额走精确整键批量 + 卡⑤ 缓存。
- **公权机构目录**:App 运行时不全量扫链。发布期生成器在同一个 finalized 块读取机构与账户表，生成分省快照；App 以 Isar 建立本地索引。点进详情和关键操作使用已知 cid_number 精确读取链状态。
- **多签(我的)**:
  - L1(卡②,纯客户端零链改):① `InstitutionDiscoveryService` + `PersonalManageDiscoveryService` 对
    同一张 `AdminAccounts` 的双扫合并为 `shared/admin_accounts_scan_service.dart` 单次扫,按 kind 分流;
    ② 已知地址的机构账户命名走 `AccountRegisteredCid[addr]` 精确批量反查；已知 CID 的账户集合读取 `InstitutionAccounts` 真源；
    ③ 删死代码 `listCidAccounts`。
  - L2(ADR-019,需 runtime 升级):加 `AdminAccountsByMember` 成员反向索引,把全表扫降为按钱包精确读
    (O(n)→O(1))。这是全系统最高价值的一处链改,ADR-018"不动链端"范围外,单列。

### 3. `listCidAccounts` 定性
死代码(全仓零调用);其设计目标(正向枚举某 cid 下账户名)被 `AccountRegisteredCid` 精确反查取代,
且正向枚举是 R1 禁区 → 直接删除,不整表化重写(免去 BoundedVec 长前缀 SCALE 解析坑)。
