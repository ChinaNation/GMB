# 任务卡:卡⑤ ChainReadCache 余额/storage 共享缓存层(共享底座)

属 ADR-018 §三-2 + §九(2026-06-13)。落位 `governance/shared/`(三仓库共享底座)。

## 目标
新建 `governance/shared/chain_read_cache.dart`:按 `finalizedBlockHash : storageKey` 缓存,TTL 短(同块内复用,换块自然失效)。插在 ChainRpc 批量入口层(`页面 → ChainRpc → SmoldotClientManager`,缓存在 ChainRpc)。

## 任务清单
- [ ] 新建 `governance/shared/chain_read_cache.dart`,键 = `finalizedBlockHash:storageKey`,换块失效。
- [ ] 收编现有 `SfidMainPubkey` 内部缓存进统一层。
- [ ] 接入 §四D 单发 `fetchFinalizedBalance`:`wallet/.../wallet_onchain_balance_card.dart:59`、`governance/.../institution_account_info_page.dart:150/254`、`personal_manage_account_info_page.dart:171/274`、创建/关闭/转账前各页。
- [ ] 治理机构余额(精确整键 `System.Account[addr]`)走此层批量 + 缓存。
- [ ] **豁免死守**:交易提交管线(nonce/dry-run/submit/runtime-version/genesis)不走缓存(`feedback_extrinsic_submit_must_watch`);UI 倒计时 Timer 不动。

## 验收
- [ ] flutter analyze 0 + flutter test 全过(缓存命中/换块失效单测)
- [ ] 真机:同地址多次单查降为一次;logcat 验证 fetchStorage/fetchFinalizedBalance 调用数下降
- [ ] 旧代码/文档/注释清理无残留
