# ADR-017：出块即固化 + 全端 finalized 单一口径

- 状态:Accepted / 代码已实施(2026-06-12 卡1链端+卡2 citizenapp+卡3桌面端全部完工,cid 后端经盘点本就达标零改动;**待 user 重建节点二进制并全网滚动重启后生效**,实施明细见 20260612-adr017-card1/2/3 三张任务卡)
- 背景事故:2026-06-10~12 机构详情提案列表为空/余额不动/转账看似成功不上链系列排查,最终根因 = 难度 100 分叉风暴 + GRANDPA 默认投票规则(best−2)+ 跳空块三者叠加,轻节点链视图漂移(详见任务卡 20260611-citizenapp-keyspaged-pin-finalized 三次诊断)。

## 决策

1. **出块即固化**:节点 GRANDPA 投票规则从 substrate 默认(`BeforeBestBlockBy(2)` + `ThreeQuartersOfTheUnfinalizedChain`)改为**无约束**(允许投到链尾)。出块后秒级固化;死水期尾块同样可固化。
2. **全端 finalized 单一口径**:除交易提交管线豁免区外,一切链上状态读取(余额/提案/交易记录/机构/治理/索引扫描/事件)统一钉 finalized。**禁止业务/展示读取使用 best。**
3. **豁免区(唯一)**:交易构造与提交管线——签名 nonce(`accountNextIndex`,池视图)、runtime version/metadata/genesis(签名参数)、dry-run、submit、提交成功判定(InBestBlock 等待)、提交后 nonce 后台核对。这些是提交协议的物理需要,不是"高度状态展示"。
4. **跳空块设计不变**:两层强制(runtime `pow-difficulty` 空块 assert(lib.rs:142-149,共识级)+ 矿工侧池空不挖)。
5. **难度问题降级**:分叉只在链尖秒级窗口出生即被裁决,全端只读 finalized 后不可观察;难度卡 20260608 降为运行期前优化项(矿工奖励公平/防刷块,后期难度调整另行处理)。
6. **权威**:开发期单权威 = 国家储委会节点(中枢),它是全网 finality 心脏,须存活监控;运行期 44 权威需 2/3+1=30 票在线,**部署算术问题必须在 SwitchToProduction 之前定案**(6 台服务器装不下 30 个 voter 身份)。

## 各端改动清单(2026-06-12 全仓盘点)

### 链端(citizenchain/node)——1 处,不动 runtime/创世,滚动重启生效
- `core/service.rs:723` `voting_rule` → 无约束规则。

### citizenapp ——收口点改造,~40+ A 类散点自动生效
- **收口 P0**:`ChainRpc.fetchStorage/fetchStorageBatch/fetchStorageBatchChunked` 底层从 `getStorageValuesHex`(best)切 `getFinalizedStorageValuesHex`;废弃的 fetchBalance/fetchTotalBalance/fetchBalances(best)删除。
- **索引扫描**:`getKeysPagedAtBest` → 钉 finalized 哈希(ensureSynced 之后取快照),改名 `getKeysPagedFinalized`;4 调用点跟随。
- **交易监控**:`ChainTxMonitor` 删 best 链扫描(`_syncBestUnfinalized`/`fetchLatestBlock` 路径),只扫 finalized 链;`_processBlock` 的 `state_getStorage` 改带 finalized 块哈希;交易状态三态收敛两态(已提交→已确认);`chain_event_subscription` 业务刷新只挂 finalizedHeads。
- **杂项 A 类**:`fetchConfirmedNonce`(监控用)、`fetchCurrentCidMainPubkeyHex`、clearing_bank_directory 等随收口自动/逐点切换。
- **豁免保持**:`fetchNonce`/`getAccountNextIndexAsync`/`fetchRuntimeVersion`/`fetchMetadata`/`fetchGenesisHash`/`submitExtrinsic*`/`signed_extrinsic_builder`。
- **诊断面**:`getStatusSnapshot` 同时含 best/finalized 高度,仅供链况横幅诊断展示,业务逻辑不得取 best 字段。

### 桌面端(citizenchain/node Tauri)——15 处 A 类
- `shared` 层新建统一收口:`fetch_finalized_head()` + `fetch_finalized_storage(key)` + `fetch_finalized_keys_paged(prefix,…)`;合并现有 4 份重复 `fetch_finalized_head`。
- A 类 15 处机械替换:governance/proposal.rs(9)、admins_change/storage.rs(1)、organization-manage/chain.rs(3)、offchain_transaction/endpoint.rs(1)、settings/fee-address(1)。
- B 类豁免:signing.rs 提交管线(nonce/dry-run/submit/后台核对)。C 类 8 处已 finalized 不动。

### cid 后端——基本已达标
- indexer 已 `subscribe_finalized` + finalized head(C 类)✓;推链三件套保持"只等 InBestBlock"(豁免;可选升级等 Finalized,P2 决策项)。

### 原生层(smoldot)——0 改动
- `chain_finalized_storage_values` 等 finalized 原生变体已齐;Dart 路由切换即可。无按任意哈希钉块的原生读取能力(本方案不需要;留备忘)。

## 验收

1. 链端:改规则滚动重启后,发一笔交易 → 数秒内 `finalized == best`;静默期最后一块也被固化(历史死结场景)。
2. citizenapp:analyze 0 + test 全过;模拟器 E2E(机构详情提案列表/余额/广场/点击/转账两态)。
3. 桌面端:cargo check/test + UI 冒烟(提案列表/总览/机构页)。
4. 残留扫描:业务代码零裸 best 读取(grep getStorageValuesHex/fetchStorage 旧入口/state_getStorage 无 at)。

## 风险与备忘

- finality 停 = 全网数据冻结(单权威期中枢 voter 必须稳;监控+自动拉起)。
- 44 权威部署算术(SwitchToProduction 前置硬条件)。
- 附带小卡:广场列表重复显示同一提案(缓存 dup)。
