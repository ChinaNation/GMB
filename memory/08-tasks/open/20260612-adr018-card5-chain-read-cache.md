# 任务卡:卡⑤ ChainReadCache 余额/storage 共享缓存层

属 ADR-018 §三-2 + §九(2026-06-13)。

状态:**代码完工(2026-06-13)**,analyze 0 + flutter test 204/204 全过;真机 logcat 验证待 user 跑。

## 落位修正
ADR §九 图把它列在 `governance/shared/`,但消费者 `ChainRpc` 在 `lib/rpc/`。为避免 rpc→governance 层级倒挂,实现落 **`lib/rpc/chain_read_cache.dart`**(与 ChainRpc 同层)。

## 设计(透明缓存挂咽喉)
- 全部 finalized 状态读(余额/storage/反查/多签扫描)都汇入 `ChainRpc.fetchStorageBatch` → 缓存挂这一个咽喉即全覆盖,**§四D 单查接入点零改动**。
- 命名空间 = `finalizedBlockHash`;同 finalized 块内状态不可变 → 块内缓存零陈旧,换块整体失效(GMB ~6 分钟出块,块内复用收益大)。
- 失效驱动:① `ChainTxMonitor` 收到新 finalized 头即 `invalidate()`(即时);② `read` 内 finalizedHash 门控复查(15s 兜底)。
- 单例(ChainRpc 各处 new,必须进程级共享)+ in-flight 合并 + 负缓存。
- 豁免:提交管线(nonce/dry-run/submit/runtimeVersion/genesis/fetchLatestBlock)走各自原生调用,**结构性不经 fetchStorageBatch**,天然免缓存。

## 完工清单
- [x] 新建 `lib/rpc/chain_read_cache.dart`:单例;`read(keys, finalizedHashProvider, fetchMissing, forceFresh, now)`;命名空间换块清空 + in-flight 合并 + 负缓存 + `invalidate()` + 15s 门控复查。
- [x] `ChainRpc.fetchStorageBatch` 接缓存:取 finalizedHash(`getStatusSnapshot().finalizedBlockHash`)→ ChainReadCache,未命中才 `_rawFetchFinalizedStorage`(原下沉逻辑);加 `forceFresh` 参数。
- [x] 单发 `fetchFinalizedBalance` 改委托 `fetchFinalizedBalances([pubkey])`(行为等价 free 口径),离开 `getFinalizedSystemAccountSnapshot`,自动经 batch+缓存;`fetchFinalizedBalances` 加 `forceFresh` 透传。
- [x] `fetchCurrentCidMainPubkeyHex` 改走 `fetchStorageBatch`,**删** `_cachedCurrentCidMainPubkeyHex` 永久缓存字段(改块内缓存,密钥轮换后自动失效更正确)。
- [x] `ChainTxMonitor._onEvent` 的 `newFinalizedBlock` 分支调 `ChainReadCache.instance.invalidate()`(即时失效)。
- [x] `DuoqianTransferBalanceGuard` 转账前余额校验走 `forceFresh: true`(旁路缓存读最新)。
- [x] 清残留:删孤儿 `_normalizeAccountHex`(原仅旧单发余额使用)。`getFinalizedSystemAccountSnapshot` 改道后无调用方,系 smoldot 底层能力绑定保留(非业务残留)。

## 单测
- [x] `test/rpc/chain_read_cache_test.dart`:命中/换块/15s 门控/负缓存/forceFresh/in-flight 合并/invalidate(8 例,纯逻辑注入 fake,无 smoldot)。

## 验收
- [x] flutter analyze 0 + flutter test 204/204 全过
- [x] 旧代码/注释清理无残留(永久缓存字段、孤儿函数已删)
- [ ] 真机:同地址多次单查降为一次;logcat 验证 fetchStorage/balance 调用数下降(待 user 装机)

## 改动文件
- 新增:`lib/rpc/chain_read_cache.dart`、`test/rpc/chain_read_cache_test.dart`
- 改:`lib/rpc/chain_rpc.dart`、`lib/rpc/chain_tx_monitor.dart`、`lib/transaction/duoqian-transfer/duoqian_transfer_balance_guard.dart`

## 边界
- 不动提交管线豁免接口;不动 smoldot 底层绑定;链端 0 改动。
