# ADR-018:wuminapp 全应用统一字段查询 + 降低全节点依赖

- 状态:实施中(2026-06-13)。卡①③④⑦ 已完工并真机验证(机构详情提案显示/广场去重/投票确认/降载全部生效);卡②⑤⑥ 待独立窗口续做。
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
| `governance/organization-manage/institution_manage_service.dart:327` listSfidAccounts | `SfidRegisteredAddress[blake2(sfid)+sfid]` | 改整表扫 `SfidRegisteredAddress`(短前缀)+ 客户端按 sfid 过滤(落地前手机实测确认空) |

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
- 卡②(A)`listSfidAccounts` 整表化(先手机验证)。
- 卡③(B)全部 N+1 改批量(余额/storage/投票)。
- 卡④(C)三处轮询改 finalized 订阅。
- 卡⑤(D)`ChainReadCache` 余额/storage 共享缓存层。
- 卡⑥(E)HTTP/SFID 后端 Isar+TTL 缓存。
- 卡⑦ 规则 R1/R2/R3 写入 `memory/07-ai/agent-rules.md` + `memory/05-modules/wuminapp/`。

## 八、风险与回滚
- ProposalsByYear 跨年窗口:取 currentYear,必要时并 currentYear-1(提案 90 天清理,1 年窗口足够)。
- 整表扫 SfidRegisteredAddress / AdminAccounts:注册规模上去需分页;dev 期无虞。
- 纯客户端改动,逐卡可独立回滚(git revert),不涉链、不涉创世。
