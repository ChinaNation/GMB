# 任务卡:ADR-017 卡3 桌面端 finalized 口径统一

## 方案
1. 新建统一收口 `governance/chain_query.rs`:`fetch_finalized_head()/fetch_finalized_storage(key)/fetch_finalized_keys_paged(prefix,…)`;合并 4 份重复 fetch_finalized_head(institution/proposal/duoqian_transfer/home)。
2. 15 处 A 类读取机械替换:governance/proposal.rs(9)、admins_change/storage.rs(1)、organization-manage/chain.rs(3)、offchain_transaction/endpoint.rs(1)、settings/fee-address(1)。
3. B 类豁免不动(signing.rs 提交管线);C 类 8 处已 finalized 不动。

## 验收
- [x] cargo check + node 测试通过;残留扫描无裸 state_getStorage(无 at)业务读取

## 完工记录(2026-06-12)

### 改动文件清单(11 文件)
1. `node/src/governance/chain_query.rs` **新建**:三函数收口 `fetch_finalized_head()` / `fetch_finalized_storage(&str) -> Result<Option<String>,_>` / `fetch_finalized_keys_paged(prefix,count,start_key)`,底层复用 signing.rs `rpc_post`;模块头注明 ADR-017 死规则(业务读取禁止 best)。
2. `node/src/governance/signing.rs`:`rpc_post` 放宽 `fn` → `pub(crate) fn`(仅可见性,提交管线豁免区零改动)。
3. `node/src/governance/mod.rs`:注册 `pub(crate) mod chain_query;`;`build_chain_query_context` 钉块来源改 `chain_query::fetch_finalized_head`。
4. `node/src/governance/institution.rs`:删本地 `fetch_finalized_head`(4 份重复之一),`fetch_balance` 改走收口;`fetch_balance_at` 保留(全部调用方均传 Some(finalized hash),C 类)。
5. `node/src/governance/proposal.rs`:删本地 `fetch_finalized_head`/`fetch_finalized_storage`/`rpc_post`;9 处 A 类全部改经 chain_query(next_proposal_id / active_proposal_ids / proposal_meta / proposal_data_raw / internal_tally / joint_tally / referendum_tally / proposal_display_id / option_bool + keysPaged 反向索引)。
6. `node/src/governance/admins_change/storage.rs`:1 处 A 类(AdminAccounts)改收口,删本地 rpc_post。
7. `node/src/governance/organization-manage/chain.rs`:删本地 `fetch_finalized_head`;实勘 3 处 state_getStorage 原本已传 finalized_hash(C 类保留),真正裸读是 `state_getKeysPaged` 翻页循环 1 处——补钉 `fetch_institution_detail` 同一 finalized 快照哈希(不经 chain_query 单页函数,保证 key 列举与 storage 读取钉同一块)。
8. `node/src/transaction/duoqian_transfer/proposal.rs`:删本地 `fetch_finalized_head`/`fetch_finalized_storage`/`rpc_post`,SafetyFund/Sweep 两处读取改收口(原已 finalized,仅合并实现)。
9. `node/src/transaction/offchain_transaction/endpoint.rs`:1 处 A 类(ClearingBankNodes)改收口,删本地 rpc_post。
10. `node/src/settings/fee-address/mod.rs`:1 处 A 类(RewardWalletByMiner 绑定状态查询)改收口。
11. `node/src/home/rpc/mod.rs`:删本地 `finalized_block_hash`(4 份重复之一),3 处调用(finalized 高度/总发行/永久质押)改收口。

### A 类替换计数
- 真实裸 best 读取(无 at 参数)共 **13 处** 改钉 finalized:proposal.rs 9 + admins_change 1 + organization-manage keysPaged 1 + offchain endpoint 1 + fee-address 1。
- 任务卡预估 15 处中,organization-manage/chain.rs "3 处" 实勘有 2 处早已传 finalized_hash(盘点口径按 grep 行号粗算),按"已 finalized 不动"处理。
- 4 份重复 fetch_finalized_head(institution/proposal/duoqian_transfer/home)全部删除合并入 chain_query。

### 残留扫描结论
- `grep -rn "state_getStorage" src/ | grep -v signing.rs | grep -v chain_query` 剩余命中全部显式携带块哈希:home/rpc 2 处(finalized_hash)、mining/dashboard 2 处(按块号哈希,C 类)、institution.rs fetch_balance_at(调用方传 finalized hash)、organization-manage 3 处(同函数 finalized 快照)。
- `state_getKeysPaged` 仅剩 organization-manage 翻页循环 1 处,已钉 finalized 快照哈希。
- 零裸 best 业务读取。

### 编译/测试
- `cargo check -p node`:通过,0 warning;`core/service.rs` `voting_rule: ()`(卡1)一并编译通过,无需改 VotingRulesBuilder。
- `cargo test -p node`:**153 passed / 1 failed**。唯一失败 `transaction::onchain_transaction::tests::compact_u128_big_integer` 为既有失败(git stash 后复跑同样失败,与本卡无关):测试断言 1_000_000 走 big-integer 模式,实际 SCALE 正确编码为 four-byte 模式,测试取值写错,已另立修复任务。
