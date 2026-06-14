# ADR-018:wuminapp 全应用统一字段查询 + 降低全节点依赖

- 状态:实施中(2026-06-13)。卡①③④⑦ 已完工并真机验证(机构详情提案显示/广场去重/投票确认/降载全部生效);**卡②⑤ 代码完工**(②双发现服务合一+批量反查+删死代码;⑤ ChainReadCache 挂 fetchStorageBatch 咽喉+按 finalizedHash 块内缓存+tx_monitor 即时失效+forceFresh 旁路;analyze 0 + test 204/204,真机 logcat 待 user 跑);卡⑥ 待做。
- **修订(2026-06-13)**:经全栈架构审计,账户发现按"三仓库 + 共享底座"重构(详见 §九)。卡② 由"listSfidAccounts 整表化"重定义为"删死代码 + 合并双发现服务单次扫描 + AddressRegisteredSfid 反查命名";卡⑥ 增列"公权机构目录走 SFID 后端 catalog";新增 [[ADR-019]](ADR-019-adminaccounts-by-member-index.md)(链端成员反向索引,L2)。
- **公权机构界面定案(2026-06-13,混合模式)**:① 数据来源=发布期生成数据包打底 + SFID 公开接口版本/增量同步(不是纯接口、不是纯数据包);② 落点修订——功能域归 `wuminapp/lib/citizen/public/`(不放 governance,公权≠治理),仅借 `governance/shared/` 共享原语(派生/账户卡/缓存/提案查询);③ SFID 侧新建 `sfid/backend/wuminapp/` BFF 目录(匿名只读,薄 handler;领域逻辑留 gov),公权目录接口落此;④ 账户发现 100% 本地派生,目录行带 `custom_account_names`(空占绝大多数,近零成本),只余额联网批量+ChainReadCache 缓存;⑤ v1 只做浏览+订阅+动态展示,发起提案/换管理员下一期。拆 5 张卡:`20260613-sfid-wuminapp-bff-public-catalog`(跨模块前置)+ `card0-derivation-base` + `cardA-data-sync` + `cardB-nav-ui` + `cardC-detail`。
- **混合模式 v2 修订(2026-06-13,量级实测后定案)**:实测确定性目录到镇级,**单省机构上万、全国约 40 万**(广东省 9000–15000)。据此:① **省导航始终全显**——左栏 43 省来自 `data/public_provinces.dart` 复用治理 `kProvincialCouncils` 同一行政区(去 `公民储备委员会` 后缀,**保留"省"** → 与 china.sqlite/SFID `province` 字段对齐;展示去"省",匹配用全名);关注钉顶不滚。② **真实版本号** = `MAX(updated_at)` RFC3339(subjects 有 updated_at),非 null;**增量** = `WHERE updated_at > since`,客户端带本地版本只拉变化(通常空/几条)。③ **keyset 翻页**(`after_sfid`,`sfid_number > $after`)替代 OFFSET(深翻 O(n²)),供生成器高效全量导出。④ **客户端本地优先**:进省先读本地秒显、在线增量丢后台(TTL 节流 + 超时),**消除"阻塞式整省全量下载→一直转圈"**;首启后台分批(putAllBySfidNumber chunk 2000)灌数据包基线。⑤ 150MB 量级可打进安装包,**真瓶颈是首次灌 40 万条进 Isar**(后台分批,真 isolate 留 follow-up)。⑥ **运维前置**:用新二进制重启 SFID 后端 + 跑 `tools/generate_public_institution_bundle.mjs`(已内嵌 43 省全名)铺完整包。删除检测 updated_at 抓不到→低频全量对账兜底(follow-up)。
- 关联:[[ADR-017]](ADR-017-finalized-unification.md)
- 触发:机构详情提案列表为空(根因实测:轻节点对长前缀 keysPaged 返回空);user 要求把"统一字段 + 降载"作为基础规则,整改整个 wuminapp。

---

## 一、关键技术结论(审计后确诊,纠正常见误判)

轻节点 smoldot 的两种访问模式必须分清:

| 访问模式 | 轻节点表现 | 例子 |
|---|---|---|
| **精确整键 `fetchStorage(完整key)`** | ✅ 正常(单 key Merkle 证明) | `ActiveProposalsByInstitution[account]`、`InternalVotesByAccount[pid,account]`、`System.Account[account]` 余额 |
| **keysPaged 前缀扫描,前缀嵌长 K1(32B account / blake2+sfid)** | ❌ 返回空(证明拉不全,静默空) | `ProposalsByInstitution[account]`、`SfidRegisteredAddress[blake2(sfid)+sfid]` |
| **keysPaged 短前缀(整表 / ≤2B K1)** | ✅ 正常 | `ProposalsByYear[year]`、`ProposalsByOrg[org]`、`AdminAccounts` 整表 |

**所以"功能性坏"只有长前缀 keysPaged 这一类(2 处);其余全是"能用但费节点"的负载问题。** 不需要改链端 storage 结构。

---

## 二、三条基础规则(本 ADR 确立,写入 agent-rules)

- **R1 统一字段查询**:列表类一律"短 key 索引(`ProposalsByYear`/整表)取一次 → 客户端按已解码字段过滤"。**禁止**业务代码对嵌 32B account / `blake2(x)+x` 的长 K1 做 keysPaged 前缀扫描。精确整键读取不受限,可继续用。
- **R2 降低全节点依赖**:① 多 key 一律 `fetchStorageBatch`/`fetchFinalizedBalances`,禁止循环内逐条;② 同一数据跨页面取一次进共享缓存复用;③ 链状态页用 finalized 订阅驱动,禁止 Timer 轮询查链;④ 能本地算的不联网。豁免=交易提交管线(nonce/dry-run/submit/runtime-version/genesis)+ UI 倒计时。
- **R3 外部后端(SFID/HTTP)缓存**:health/catalog/机构注册证/电子护照状态等读取加 Isar + TTL 缓存,不每次现查。

---

## 三、统一架构(三个收口层)

### 1. `ProposalFeedCache`(新增,提案统一查询层)
所有提案页面共用一条通路,替代散落的 4 个反向索引:
```
ProposalsByYear[currentYear](短key,可用) → getKeysPagedFinalized → ids
   → _fetchProposalsForIds(ids)(已全批量:meta+displayId+detail)
   → 共享缓存(TTL 20s)
   → 广场 filter internalOrg∈{0,1,2} / 机构详情 filter institutionBytes==account ∪ kind==1 / 个人 filter institutionBytes==account
```
依据:`ProposalsByYear` 链端对每个提案无条件写、终态清理时移除;`ProposalMeta` 已解码 `kind/stage/status/internalOrg/institutionBytes`,客户端过滤零联网。
**载荷:广场(原 3 次 ByOrg)+ 机构详情(原坏的 ByInstitution)+ 个人 → 同周期共用一份缓存 = 1 次按年取 + 1 次批量详情。**

### 2. `ChainReadCache`(新增,余额/storage 共享缓存,挂在 ChainRpc 层)
- 按 `finalizedBlockHash : storageKey` 缓存,TTL 短(同块内复用),换块自然失效。
- 余额、固定 storage(SfidMainPubkey 已有内部缓存,纳入统一)走此层。
- 审计确认收口层级:`页面 → ChainRpc(批量入口) → SmoldotClientManager(原生窗口)`,缓存插在 ChainRpc。

### 3. 批量与订阅(改造现有,不新增类)
- 所有"循环内逐条链读"改 `fetchStorageBatch`/`fetchFinalizedBalances`。
- 所有"Timer 轮询查链"改 finalized 头订阅(复用 `ChainEventSubscription`)。

---

## 四、全应用整改清单(按类别,含确切位置)

### A. R1 必改 — 长前缀 keysPaged(功能性坏,2 处)
| 位置 | 索引 | 改法 |
|---|---|---|
| `transaction/duoqian-transfer/duoqian_transfer_service.dart:290` fetchProposalIdsByInstitution | `ProposalsByInstitution[account]` | 删,机构详情改走 ProposalFeedCache 按年取+过滤(**本次 bug 收口**) |
| `governance/organization-manage/institution_manage_service.dart:327` listSfidAccounts | `SfidRegisteredAddress[blake2(sfid)+sfid]` | **~~改整表扫~~ 已被 §九 取代**:审计确认此方法为死代码(全仓零调用),且多签账户清单应走 `AddressRegisteredSfid` 精确反查而非正向枚举 → **直接删除**,不整表化 |

### B. R2 必改 — N+1 循环逐条链读(费节点,核心降载)
| 位置 | 现状 | 改法 |
|---|---|---|
| `rpc/chain_tx_monitor.dart`(余额回调) | 每事件/每钱包逐条 `fetchFinalizedBalance` | 批量 `fetchFinalizedBalances([钱包])` |
| `governance/.../institution_detail_page.dart:508` | 逐管理员查余额 | 收集后一次 `fetchFinalizedBalances` |
| `governance/.../institution_detail_page.dart:796` | 逐转账受益人查余额 | 同上批量 |
| `governance/organization-manage/institution_manage_service.dart:342/368/380/547/556/564/593` | listSfidAccounts/账户详情逐条 `fetchStorage` ×7 | `fetchStorageBatch(keys)` |
| `governance/personal-manage/personal_manage_service.dart:180/197/205/357` | 个人账户逐条 `fetchStorage` ×4 | `fetchStorageBatch` |
| `citizen/vote/vote_view.dart:308` hasUnvotedWallet | 循环内逐提案查投票 | 新增 `hasUnvotedWalletsBatch` 批量 |
| 调 `fetchAdminVote`(单)/`balance_guard`(单)的循环点 | 逐条 | 强制走已存在的 `fetchAdminVotesBatch`/批量余额 |

### C. R2 应改 — 轮询改订阅 / 降频
| 位置 | 现状 | 改法 |
|---|---|---|
| `ui/chain_progress_banner.dart:74/104` | 6s 轮询 `fetchChainProgress` | 订阅 finalized 头 / 仅 syncing 时轮询且延长 |
| `transaction/duoqian-transfer/duoqian_transfer_detail_page.dart:142/520` | 20s 轮询待投票状态 | finalized 头订阅驱动刷新 |
| `governance/institution_manage_detail_page.dart:455` | 20s 轮询 | 同上 |
| (qr_sign_session 1s、pin_input 1s = UI 倒计时,**不查链,豁免**) | — | 保留 |

### D. R2 应改 — 共享缓存 / 去重
| 位置 | 现状 | 改法 |
|---|---|---|
| 广场 `citizen/vote/vote_view.dart:156-158` | 3 次 `ProposalsByOrg` | 接 ProposalFeedCache,与机构详情共用 |
| `wallet/.../wallet_onchain_balance_card.dart:59` 单账户总额 | 单查无缓存 | 接 ChainReadCache |
| `governance/.../institution_account_info_page.dart:150/254`、`personal_manage_account_info_page.dart:171/274` | 同地址多次单查 | 去重/缓存 |
| 创建/关闭/转账前各页 `fetchFinalizedBalance` 单查 | 无缓存 | ChainReadCache |

### E. R3 应改 — HTTP/SFID 后端缓存(37 处无缓存)
| 位置 | 改法 |
|---|---|
| `wallet/capabilities/api_client.dart`(health/admin catalog/机构注册证 等) | 各接口加 Isar+TTL(health 5min / catalog 1d / 证书 7d) |
| `my/myid/myid_api.dart:50` 电子护照状态 | Isar 缓存 + 15min 刷新 |
| `rpc/sfid_public.dart:51` 清算行搜索 | 已部分缓存,补 TTL |
| `update/app_update_service.dart:92` GitHub release | 低频,加短缓存即可 |

### 豁免(不改)
交易提交管线:`fetchNonce`/`fetchRuntimeVersion`/`fetchGenesisHash`/`fetchLatestBlock`/`fetchFinalizedBlock`(提交用)/`submitExtrinsic*`/dry-run;`ChainTxMonitor` finalized 订阅;UI 倒计时 Timer。精确整键 `fetchStorage`(ActiveProposalsByInstitution/InternalTallies/AdminSnapshot/clearing-bank 等)正常,只在被循环调用时归入 B 类批量,本身不动口径。

---

## 五、不动链端
`ProposalsByInstitution` 等反向索引保留在 runtime(桌面端全节点可正常用);仅约束轻节点客户端不查长前缀。纯 Dart 改动,不动 runtime、不重新创世。

## 六、验证
1. `flutter analyze` 0 + `flutter test --concurrency=1` 全过;新增 ProposalFeedCache 过滤单测(多机构/多 org/joint 夹具)。
2. 落地前手机 logcat 实测确认 A 类 2 处确返回空(验后删诊断)。
3. 真机 E2E:机构详情(国储会)显示提案;广场/个人多签一致;**logcat 统计改造前后 getKeysPaged/fetchStorage/fetchFinalizedBalance/HTTP 调用计数,验证显著下降**。

## 七、任务拆卡
- 卡①(A+D 提案)`ProposalFeedCache` + 机构详情/广场/个人多签统一接入(收口本次 bug + 最大降载)。
- 卡②(A,**已按 §九 重定义**)多签发现降载:删 `listSfidAccounts` 死代码 + 合并机构/个人多签双发现服务为共享单次 AdminAccounts 扫描 + `AddressRegisteredSfid` 精确反查命名。纯客户端零链改。
- 卡③(B)全部 N+1 改批量(余额/storage/投票)。
- 卡④(C)三处轮询改 finalized 订阅。
- 卡⑤(D)`ChainReadCache` 余额/storage 共享缓存层。
- 卡⑥(E + §九)HTTP/SFID 后端 Isar+TTL 缓存;**增列**公权机构目录走 SFID 后端 catalog(分页+搜索)+ Isar/TTL,轻节点不扫链(公权机构界面"下一步再做",随界面落地)。
- 卡⑦ 规则 R1/R2/R3 写入 `memory/07-ai/agent-rules.md` + `memory/05-modules/wuminapp/`。

## 八、风险与回滚
- ProposalsByYear 跨年窗口:取 currentYear,必要时并 currentYear-1(提案 90 天清理,1 年窗口足够)。
- 整表扫 SfidRegisteredAddress / AdminAccounts:注册规模上去需分页;dev 期无虞。
- 纯客户端改动,逐卡可独立回滚(git revert),不涉链、不涉创世。

---

## 九、账户发现分层架构(2026-06-13 全栈审计修订)

### 0. 背景:三类机构本质不同,不能共用一条发现通路
全栈审计(链端 storage + 派生原语 + wuminapp 客户端)确认:账户地址全部由唯一原语
`derive_duoqian_account(op_tag, ss58, payload)`(`primitives/src/core_const.rs:89`)确定性派生,
同输入永远同地址、可离线算。但"枚举账户清单"的来源与范围语义三类完全不同:

| 类别 | 范围语义 | 发现来源 | 缓存模型 | 新鲜度 | 链改空间 |
|---|---|---|---|---|---|
| 治理机构(87:国储会1+省储会43+省储行43) | 目录·全集(与用户无关) | 编译期注册表 `governance_institution_registry.generated.dart` | 静态写死 | 永不变 | 无 |
| 公权机构(动态注册) | 目录·全集(与用户无关) | **SFID 后端 catalog**(分页+搜索) | HTTP+Isar/TTL(天级) | 慢 | 无 |
| 多签(机构多签+个人多签) | **我的**(我的钱包某钱包是管理员才显示) | 链上扫 `AdminsChange::AdminAccounts` | Isar 永久+增量 | 随钱包/出块变 | ADR-019 反向索引 |

账户类型注记(用户口径,2026-06-13):
- 治理机构账户固定不变:国储会 4(主/费/安全基金/两和)、省储会 2(主/费)、省储行 3(主/费/永久质押)。
- 公权机构:主/费 + N 个自定义(sfid_number+account_name 组合),界面"下一步再做"。
- 个人多签:发起人钱包 + account_name,无 sfid,仅链上注册。

### 1. 架构:三仓库 + 一共享底座
上层按"发现来源"拆三个 Repository(语义/来源/缓存/新鲜度四维全不同,强行统一 = 全是
`if orgType` 的漏抽象);下层共享派生/余额/账户卡/反查,杜绝重复。

```
lib/governance/
├── shared/                              ← 共享底座(三类都用,单一源)
│   ├── account_derivation.dart          ← derive_duoqian_account 唯一 Dart 实现(OP_* 全枚举)
│   │                                       现状:仅 personal-manage/personal_duoqian_derive.dart,需归位/补全
│   ├── chain_read_cache.dart(卡⑤)      ← finalizedBlockHash 命名空间的余额/storage 共享缓存
│   │     （实现落 lib/rpc/chain_read_cache.dart:消费者 ChainRpc 在 rpc 层,避免 rpc→governance 层级倒挂;
│   │      挂在 fetchStorageBatch 咽喉透明覆盖全部 finalized 状态读,豁免管线不经此入口天然隔离）
│   ├── account_card / balance UI        ← 账户卡、余额展示(三类同一套)
│   ├── address_registered_sfid 反查     ← AddressRegisteredSfid[addr]→(sfid,name) 精确整键
│   └── admin_account_storage_codec.dart ← 已存在(两发现服务共用解码器)
├── organization-manage/                 ← 治理机构目录 repo(读注册表,零链查) + 机构多签 repo(kind=2)
├── public-institution/(下一步,卡⑥)     ← 公权机构目录 repo(SFID 后端 catalog + Isar/TTL)
└── personal-manage/                     ← 个人多签 repo(kind=1/org=3)
    （shared/admin_accounts_scan_service.dart：单次全表扫 AdminAccounts,emit {kind,addr,org,admins};
      organization-manage 与 personal-manage 各自订阅过滤,不再各扫一遍——既消双扫,又不破模块边界）
```

### 2. 关键结论(纠正原 §四A 对卡②的判断)
- **治理机构目录**:已 100% 注册表驱动,零链查(最优,不动);仅余额走精确整键批量 + 卡⑤ 缓存。
- **公权机构目录**:不扫链。SFID 后端是机构身份签发方,"列出全部公权机构"是其天职 → 后端 catalog +
  Isar/TTL;点进详情用已知 sfid_number 本地派生主/费地址 + 精确整键读余额/状态;自定义账户清单由
  catalog 带出。**不碰 `SfidRegisteredAddress` 长前缀**。(归卡⑥)
- **多签(我的)**:
  - L1(卡②,纯客户端零链改):① `InstitutionDiscoveryService` + `PersonalManageDiscoveryService` 对
    同一张 `AdminAccounts` 的双扫合并为 `shared/admin_accounts_scan_service.dart` 单次扫,按 kind 分流;
    ② 机构账户命名走 `AddressRegisteredSfid[addr]` 精确批量反查聚合,**不正向枚举** `SfidRegisteredAddress`;
    ③ 删死代码 `listSfidAccounts`。
  - L2(ADR-019,需 runtime 升级):加 `AdminAccountsByMember` 成员反向索引,把全表扫降为按钱包精确读
    (O(n)→O(1))。这是全系统最高价值的一处链改,ADR-018"不动链端"范围外,单列。

### 3. `listSfidAccounts` 定性
死代码(全仓零调用);其设计目标(正向枚举某 sfid 下账户名)被 `AddressRegisteredSfid` 精确反查取代,
且正向枚举是 R1 禁区 → 直接删除,不整表化重写(免去 BoundedVec 长前缀 SCALE 解析坑)。
