# 任务卡:ADR-017 卡2 citizenapp 全面 finalized 口径

## 方案(收口点改造)
1. `ChainRpc.fetchStorage/fetchStorageBatch/fetchStorageBatchChunked` 底层切 finalized 原生变体;删 `fetchFinalizedStorage*` 重复入口(调用方改用平名,平名=finalized 成为唯一约定);删废弃 best 余额接口。
2. 索引扫描 `getKeysPagedAtBest` → `getKeysPagedFinalized`(ensureSynced 后取快照钉 finalizedBlockHash,缺失抛错),4 调用点跟随。
3. `ChainTxMonitor` 删 best 链扫描,只扫 finalized 链;交易状态收敛 已提交→已确认 两态;业务刷新只挂 finalizedHeads。
4. 残留扫描:业务代码零裸 best 读取;豁免区(nonce/runtime version/genesis/dry-run/submit)保持。

## 验收
- [ ] flutter analyze 0 + flutter test 全过
- [ ] 模拟器 E2E:机构详情提案列表显示、余额、广场点击、转账两态

## 完工记录(2026-06-12)

- 收口切换:`fetchStorageBatch/fetchStorage/fetchStorageBatchChunked` 底层切 finalized 原生变体,平名=finalized 成为唯一约定(ADR-017 注释);删除重复入口 `fetchFinalizedStorageBatch/fetchFinalizedStorage/fetchFinalizedStorageBatchChunked`(内部引用同步切平名)。
- 删除废弃 best 接口:`fetchBalance/fetchTotalBalance/fetchBalances/fetchConfirmedNonce`(全部零调用方)。
- 索引扫描:`getKeysPagedAtBest` → `getKeysPagedFinalized`(ensureSynced 后取快照钉 finalizedBlockHash,缺失抛错),4 调用点跟随。
- `ChainTxMonitor` 重构:删 best 链扫描(`_syncBestUnfinalized` 全family + `_TransferEventStatus` 枚举 + `_maxUnfinalizedBlocksPerRun`),只扫 finalized 链;newHeads 事件显式忽略(注释说明);写入状态恒 statusFinalized——交易状态两态化(已提交→已确认),statusInBlock 全仓零写入方(页面标签映射保留容错历史行)。
- 豁免区注释补齐:`fetchLatestBlock` 标注 ADR-017 豁免(仅支付凭证构造);`fetchCurrentSfidMainPubkeyHex` 切 finalized。
- 残留扫描:业务代码零裸 best 读取(getStorageValueHex/getStorageValuesHex 仅剩 smoldot_client 封装层定义);`flutter analyze` 0;`flutter test --concurrency=1` 196/196。
- 模拟器 E2E 待链端部署后与卡1一并验证(finalized 推进后机构详情提案列表/余额/转账两态)。
