# ADR-040 全仓库钱包账户字段命名统一 + 扫码确认页 SS58/中文/单位展示统一

任务需求(用户报障 3 项 + 全仓库统一诉求,直接修):
1. 区块链软件-交易-钱包管理-增加钱包:扫公民钱包二维码报「字段 address 必填非空字符串」
2. 交易发起转账:扫公民签名二维码报「公钥格式无效」
3. 冷钱包扫码确认页:字段名有英文(`yuan金额`)、金额显示 `xxxx.xx GMB`、账户显示公钥 hex 而非 SS58
   + 全仓库诉求:相同字段命名以 substrate 官方为准统一;人看的 UI 一律 SS58,系统用 hex;
     公钥/哈希要么都带 0x 要么都不带,不能混;金额单位统一「元」(白皮书:算用分/展示用元)

所属模块:citizenchain/node(前端 QR 解析 + 后端提交)、citizenwallet(冷钱包解码/展示)、citizenapp(货币单位)

根因:
1. node QR 解析器 `citizenQr.ts`/`parseAddressQr.ts` 是唯一残留旧字段名 `address`;钱包生产端
   (`user_contact_body.dart`/`user_transfer_body.dart`)早已发 `ss58_address` → node 校验失配报错。
2. node 后端 `transaction/onchain/mod.rs` 对 `expected_signer_public_key` 多做了一次
   `trim_start_matches("0x")`,而 `verify_and_submit`→`normalize_public_key` 契约要求带 0x → 归一后失配。
3a. `yuan金额`:`field_labels.dart` 的 `amount_` 前缀影子规则把已登记的 `amount_yuan`(生成表=金额)
    误拆成 `amount_`+`yuan` → 泄漏英文。
3b. `GMB`:货币符号常量/默认值散落多处仍为 `GMB`(与签名域常量 `core_const::GMB` 同名但语义不同)。
3c. 账户显示公钥:解码器 `reviewFields` 账户字段值用 `_bytesToLowerHex`(hex),渲染页 `fieldValueText`
    未做 hex→SS58 转换。

修复:
1. node `citizenQr.ts`/`parseAddressQr.ts`:`address`→`ss58_address`、`pubkey`→`signer_public_key`,
   5 个消费方(TransferForm/WalletManagerModal/RewardAccountSection/CreateProposalPage/SafetyFundProposalPage)同步。
2. node `mod.rs`:删多余 `trim_start_matches("0x")`,直接传 `&expected_signer_public_key`。
3a. `field_labels.dart`:删 `amount_` 前缀影子;标签全走 crates 生成表(`amount_yuan`→金额、`amount_raw`→资产数量(raw));
    未登记 amount_* 一律拒绝,无英文兜底。
3b. 货币 `GMB`→`元`:钱包 `amount_format.dart`+`payload_decoder.dart`(全分支);
    app `amount_format.dart`+`onchain_payment_page.dart`+`transaction_history_page.dart`+`wallet_onchain_balance_card.dart`。
    ★ 只改货币符号,绝不动签名域 `core_const::GMB`(blake2_256(GMB‖op_tag‖SCALE))。
3c. 账户 hex→SS58 集中在值格式化单点 `fieldValueText`:按 ADR-040 命名约定,`account_id`/`*_account_id`
    的 `0x`+64hex 值渲染成 SS58;`*_public_key`/`*_hash` 按明确标注保持 0x hex。
    机器层 `fields` 仍是 hex(跨端逐字节验真不变)→ 金标锁步测试全绿。

0x 一致性核查(全仓库统一):显示层哈希/公钥全 0x —— `_bytesToLowerHex`(带0x)、`signerPublicKeyHex`(带0x)、
onchina `before_hash/after_hash/actor_public_key` 均 0x、`_isCanonicalHex32`=`^0x[0-9a-f]{64}$`。

验收(全部通过):
- node:前端 tsc 通过;后端 cargo check 0 error
- 钱包:`field_labels_test`+`payload_decoder_test` +116 全绿(含新增账户 SS58 展示用例);dart analyze No issues
- app:`wallet_onchain_balance_card_test` +4 绿

状态:1/2/3 全部完成并验证(2026-07-24)

## 收尾两项(2026-07-24 完成)

### chat MLS FFI 账户键统一(非 FRB,是 serde JSON-over-FFI)
发现:Dart 侧(`mls_native.dart`)早已迁移 ADR-040,发 `account_id`/`recipient_account_id`/`member_account_ids`;
Rust 侧(`chat_mls.rs`)仍用 `owner_account`/`recipient_account`/`member_accounts` —— **两侧键失配,整个
chat MLS 原生路径当前是坏的**(开发期零用户没端到端跑过,故未暴露)。非命名洁癖,是真 bug。
修复(让 Rust 对齐已迁移的 Dart 线格式):
- `chat_mls.rs` 4 个 token 全仓改名(word-boundary,无冲突):`owner_account`→`account_id`(52 行)、
  `recipient_account`→`recipient_account_id`、`member_accounts`→`member_account_ids`、
  输出键 `removed_accounts`→`removed_account_ids`。含结构体字段/require_non_empty 标签/JSON 收发键/测试/注释。
  ★ MLS BasicCredential 内容 `format!("{}:{}", account, device_id)` 只改变量名、**值/线格式不变**。
- Dart `mls_native.dart:260` 读取键 `removed_accounts`→`removed_account_ids` 同步。
- 与 `qr-protocol/tests/registry_consistency.rs` 的 `REMOVED_AMBIGUOUS_ACCOUNT_FIELDS` 禁用名守卫
  (含 `owner_account`/`wallet_account`)方向一致,守卫保留。
验证:cargo test --lib chat_mls **3 passed 0 failed**;dart analyze mls_native.dart 无错。

### 命名文档对齐 ADR-040(memory/07-ai/unified-naming.md)
代码已全面迁移(onchina/citizenapp 统一 `account_id`/`ss58_address`/`public_key`),`wallet_*` 仅剩文档残留。
改 5 处:电子护照钱包地址/公钥、CitizenSubject 组成、投票账户地址/公钥 →
`ss58_address`/`public_key`/`account_id`。文档无 `wallet_address|wallet_pubkey|wallet_account` 残留。

## 全仓复审(2026-07-24,按 audit-recipe 每条回原文核验)

覆盖:公民(Android/iOS/rust/Dart/cloudflare)、公民钱包、公民链(runtime/node/onchina)、公民官网、公民控制台。

新修真字段漂移(2 处,内部 SCALE 位置解码结构体,不透传前端、不改协议字节):
- onchina `domains/legislation/chain_read_proposal.rs` `execution_account`→`execution_account_id`
  (runtime 规范 `proposal.execution_account_id`,见 personal-admins/personal-manage lib.rs)。
- onchina `domains/legislation/display/chain_read.rs` `voter_account`→`voter_account_id`
  (ADR-040 §4 `InstitutionVoteTicket{voter_account_id}`)。
- 验证:onchina cargo check + legislation 测试通过。

确认已干净:
- 全仓强禁用名(`wallet_account`/`owner_account`/`*_pubkey`/`wallet_address`)活代码复扫 **0 命中**。
- 原生层(Android/iOS kotlin/swift/java)0 命中。
- node/onchina 前端 tsx 账户一律走 ss58 编码渲染,hex 仅"账户 Hex"明确标注处。
- 公民控制台已移出 Git(gitignore,0 跟踪),本地 src 无禁用名。
- 货币:钱包/App 金额展示已统一「元」。

审计撤回(false-positive,非违规,留原样):
- `AdminAccount` 类型 / `personal_account` 注释措辞:类型名+注释,真字段是 `personal_account_id`,非字段违规。
- 局部变量/测试助手:`target_account`(reward_account.rs var)、`admin_account`(seeder.rs var)、
  `subscriber_account()`/`creator_account()`/`beneficiary_account`(test 助手)—— ADR-040 管跨界字段非局部命名。
- `citizen_account: &ResolvedCitizenAccount`:类型参数非账户id字段。

用户定夺结果(2026-07-24):
- ①代币符号 GMB:**不改**(GMB=公民币币种符号,非金额单位)。
- ②聊天群成员展示:归入测试统一范畴处理;聊天层 accountId 不透明串不擅动展示。
- ③测试也统一 + 清理注释:已执行(见下)。

### 测试范畴命名统一 + 注释清理(2026-07-24)
测试助手/局部改名(均命名/持有 AccountId32,word-boundary 无冲突):
- `runtime/misc/square-post/tests/mod.rs`:`subscriber_account()`→`subscriber_account_id()`、
  `creator_account()`→`creator_account_id()`。
- `runtime/entity/public-manage/tests/cases.rs`:局部 `beneficiary_account`→`beneficiary_account_id`。
- `runtime/entity/personal-manage/tests/cases.rs`:助手 `proposed_account()`→`proposed_account_id()`。
- `citizenapp/test/**`:2 处测试名串 `personal_account`→`personal_account_id`。
- `node/transaction/multisig/proposal.rs`:测试 fn `..._funding_account`→`..._funding_account_id`。
补充(全仓复扫二轮又捞到的 test/benchmark/注释,同批统一):
- `runtime/src/tests/cases.rs`:`representative_account`→`representative_account_id`(测试)。
- `runtime/transaction/multisig/src/benchmarks.rs`:`beneficiary_account()`→`beneficiary_account_id()`(基准工具)。
- `citizenapp/.../personal_proposal_history_service.dart:4`:注释 `PersonalAccount(personal_account)`→`(personal_account_id)`。
留原样(非账户 id,核验后判非违规):`delete_account`(动作名)、`pending_account`(持 PersonalAccount 记录)、
`PersonalAccounts`(存储名)、`stake_account`(派生描述串)、`registry_consistency.rs` 禁用名守卫清单。
注释清理:`node/governance/proposal.rs:777`、`node/multisig/proposal.rs:146` 旧账户名 → `_account_id`。

生产局部变量(经用户确认后也统一,2026-07-24):
- `genesis/src/institution/seeder.rs`:局部 `admin_account`→`admin_account_id`。
- `node/src/settings/reward_account.rs`:按 ADR-040 惯用法拆分——账户字节 `target_account`→`target_account_id`
  ([u8;32] 账户身份),hex 文本 `target_account_id`→`target_account_id_hex`(其 0x-hex 文本编码,feeds `account_id` 字段)。
- 验证:node `cargo check`、genesis `cargo check` 均 Finished。

验证(全绿):square-post 23 / public-manage 19 / personal-manage 23 / node 1 `cargo test`;
multisig `--features runtime-benchmarks` cargo check Finished;citizenchain runtime `cargo test --no-run` Finished;
Dart 2 文件 +16。全仓活代码禁用名+裸 role_account 复扫,除上述保留项外 0 残留。
