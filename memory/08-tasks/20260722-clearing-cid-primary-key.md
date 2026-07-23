# 任务卡：清算体系以 CID 为唯一主键（binding/身份/签名全改 CID）

★★ 完成（2026-07-22）：五端 + 全测试套件 100% 绿 ★★
- cargo check --workspace = 0；cargo test -p offchain = 24 passed；cargo test -p node = 288 passed；dart analyze lib = 0 error；flutter test golden = All passed。
- 签名协议双端字节锁验证：Rust + Dart 对同一 CID intent 算出完全相同 signing_hash（fixture1=4c0c…886fe / fixture2=38ba…460e72 / fixture3=6240…052c9f）。
- 清算体系身份 = CID（机构唯一永久主键）：UserBank/DepositBalance/BankTotalDeposits/L2FeeRateBp/L2FeeRateProposed/LastClearingBatchSeq 全按 CID 键；PaymentIntent/OffchainBatchItem 携带 CID（Compact(len)||bytes 与 BoundedVec<u8> 等价）；pallet/node/runtime/onchina/Dart 五端对齐并各自测试通过。
- 后续独立任务：Step 2（清算账户资金流）、Step 3（手续费收链上费）——基于本 CID 主键落地，见对应设计。

★★ Review 修复补丁（2026-07-22，基于 Step 2/3 清算账户模型重基线后）★★
最终代码审查发现的问题已修复并各自验证：
- H1（生产缺陷）node 升级守卫探针 `runtime_policy.rs` CID/主账户字节混用：`storage_key::deposit/bank_total/last_batch` 改收 `&[u8]` CID（`Compact(len)‖bytes`）；探针用 `actor_cid_number` 播种 UserBank 值/deposit/bank_total/rate 键 + 补清算账户正反登记与注资 + item 填真实 CID（非空 Default）+ fee 账户注资覆盖 Step 3 链上费；探针返回 CID、`last_batch` 按 CID 读；新增字节锁单测 `cid_storage_keys_use_compact_len_prefix`（错回定长 32B 即红）。此前休眠（`OffchainTransaction` 被 BaseCallFilter 拦），放开入口即会硬阻断所有链升级——原“剩余仅 #[cfg(test)] 夹具不影响生产”结论订正。
- H2 跨行结算 e2e（`MockCid` 加 BANK2 独立 CID+主/费/清算账户）、H3 偿付拒绝+`>=` 边界两测试：此前 `cases.rs:99` 自认“跨行需 fixture 扩展”、全套无 `SolvencyProtected`，两条最易写错路径实为裸奔。
- M1 L2 ACK 变长 `bank_cid` 加 `Compact(len)` 前缀（对齐 `batch_signing_hash` 单一原语）。
- M3 Dart `scaleEncode` 加 release 安全的 CID 长度 `throw`（assert 被裁）。M4 `cargo fmt`。M5 `app_isar.dart` op_tag 0x06→0x07。
- 文档漂移全清：batch_item/bank_check/settlement/lib/rpc/ledger/listener/mod 按清算账户模型更正，裸字段名残留清零。
验证：`cargo test -p offchain`=28 passed；`cargo test -p node`=289 passed（含字节锁单测）；`cargo check -p node` 通过；`dart analyze`（改动文件）=0 issues；`cargo fmt --check -p offchain`=clean。
待办 M2：`submit_offchain_batch` 等走 benchmark 生成的 `weights.rs`，Step 2/3 新增 `clearing_account_of` 等 `find_account` 读未计入权重，须重跑 benchmark 生成（不臆造数值）；因并发线程正 churn entity/benchmarks/genesis，待 runtime 稳定后执行。



任务需求：
把 offchain 清算体系的"清算行身份"从主账户地址统一改为 CID（InstitutionCidNumber）——唯一、永久、全链主键。
UserBank/DepositBalance/BankTotalDeposits/L2FeeRateBp/L2FeeRateProposed/LastClearingBatchSeq 全部按 CID 为键；
L3 支付意图 PaymentIntent/OffchainBatchItem 携带 CID（做法 B，签名协议升级）；
bank_check/解析原语全部按 CID 入参；五端逐字节对齐，重生金标。

所属模块：
- citizenchain/runtime/transaction/offchain（存储/签名/bank_check/业务逻辑）
- citizenchain/runtime/src/configs.rs（费用路由）
- citizenchain/node/src/transaction/offchain（packer/submitter/ledger/listener/rpc）
- citizenapp/lib/transaction/offchain-transaction（payment_intent/rpc/scan_flow/pay_page + 金标）

决策（用户 2026-07-22 确认"执行"，采用推荐默认）：
- 做法 B：PaymentIntent 直接携带 CID，不再带主账户；重生五端金标。
- batch_signing_hash 保留 institution_account（批次是对某账户的操作，身份主键归 CID）。
- 无迁移：开发期零用户，直接重建存储 + 重生金标。

必须遵守：
- 五端签名逐字节对齐（唯一原语 primitives::sign::signing_message）
- 省标识从 CID 前 2 字节直接取（替换反查 account_info）
- 需要具体账户时由 CID 派生 find_account(cid, 保留名)

输出物：代码 + 中文注释 + 金标重生 + 文档更新 + 残留清理

落地进度（2026-07-22，编译驱动，每步 cargo check 验证）：
- 已完成并绿：offchain pallet 全部（6 存储键 + 签名结构 + bank_check + 业务逻辑 + 事件 + Call），`cargo check -p offchain` = 0 错误。
- 已完成 node ≈90%：NodePaymentIntent/NodeBatchItem/PendingPayment（CID=Vec<u8>，SCALE 与 BoundedVec 等价）、ledger（my_bank/on_payment_settled/accept_payment_with_chain_state 按 CID）、rpc 读层（UserBank/DepositBalance/L2FeeRateBp 存储键按 CID 重编 Compact(len)‖bytes、L2 ACK 按 CID）、listener（分发 + EventListener 身份）、packer、submitter。
- 决策（用户 2026-07-22）：节点自身 CID 来源 = **节点配置新增 CID 字段**（不做运行时反查）。

里程碑（2026-07-22）：**链端 cargo check --workspace = 0 错误全绿**（offchain pallet + node + runtime + onchina 全部以 CID 为唯一主键）。node 已完成:mod.rs 复用 actor_cid_number 作节点身份(无需新配置字段)、LastClearingBatchSeq/L2FeeRateBp/UserBank/DepositBalance 存储键按 CID 重编、EventListener/rpc 身份、RPC trait query_user_bank(返回 CID 文本)/query_fee_rate(入参 CID)、runtime_policy storage_key::rate。

Dart 进度（2026-07-22）：payment_intent.dart 已改（bank 字段→payerBankCid/recipientBankCid，scaleEncode 用 Compact(len)||bytes，与 BoundedVec 逐字节等价，去掉 204 定长断言）。
Dart 剩余：offchain_pay_page.dart(bank 来源从 SS58 账户改为 queryUserBank 返回的 CID 文本，intent 传 payerBankCid/recipientBankCid)、offchain_clearing_rpc.dart(queryUserBank 解析改 CID 文本、queryFeeRate 入参改 CID)、offchain_scan_flow.dart(收款方 CID 来源)。

Dart lib 已完成（2026-07-22，dart analyze lib 侧 0 error，残留已清）：
- payment_intent.dart：payerBankCid/recipientBankCid，scaleEncode 用 Compact(len)||bytes（与 BoundedVec 逐字节等价）。
- offchain_pay_page.dart：付款方 CID 来自 queryUserBank、收款方 CID 来自 widget.recipientBankCidNumber，intent 传 *BankCid，queryFeeRate 入参 CID，显示区同/跨行按 CID；删残留 import/_ensure0x。
- offchain_clearing_rpc.dart：queryUserBank 返回 CID 文本、queryFeeRate 入参 bankCid。

金标进度（2026-07-22）：batch_item.rs 三 golden fixture 已改 CID + 加 cid() helper + 改 hash-only 断言(占位 __FILL1/2/3__),两个 deterministic 测试已改 CID。跑 `cargo test -p offchain` 发现 pallet 测试套件(tests/mod.rs、tests/cases.rs)仍用旧 API(payer_bank 等)构造批次,阻塞编译 → 必须先按 CID 更新这些测试夹具,金标才能跑起来捕获真值(__FILL__)。

tests/cases.rs 消歧规则（关键,bank_main() 双重语义,逐行判断,勿盲替换）：
- 已完成:intent/item 的 payer_bank/recipient_bank 字段 → *_cid: bank_cid()（6 处 replace_all）。
- 改 bank_cid():bind_clearing_bank/switch_bank 第 2 参、UserBank/DepositBalance/BankTotalDeposits/L2FeeRateBp/LastClearingBatchSeq 的键、UserBank 值 Some(bank_main())、seed_fee_rate 参数(helper 122-124 改 &InstitutionCidNumber)、UserBank::insert(&bob, AccountId32::new(OTHER_BANK_BYTES))→CID。
- 保持 bank_main():submit_offchain_batch 的 institution_account 参、sign_batch 的 institution_account 参、Balances::free_balance(&bank_main())。
- 说明:institution_account=主账户(发 extrinsic/余额),CID=身份主键(绑定/账本键)。

★ 重大里程碑（2026-07-22）：**金标双端字节锁验证通过 + offchain 全测试套件绿**
- offchain tests/cases.rs 夹具已按 CID 消歧改完;`cargo test -p offchain` = 24 passed, 0 failed。
- batch_item 三 golden fixture signing_hash 真值：fixture1=4c0c5252…886fe、fixture2=38ba8205…460e72、fixture3=62405346…052c9f。
- Dart payment_intent_golden_test.dart 用同一 CID fixture + 同一 hash;`flutter test` = All passed。**Rust 与 Dart 对同一 CID intent 算出完全相同 signing_hash → 签名协议逐字节对齐锁定。**

剩余（仅 node crate #[cfg(test)] 夹具,50 处机械改,不影响生产/金标）：
- runtime_policy.rs 测试(805/807/831/833 PaymentSettled 事件构造 → *_cid;814 storage_key::rate probe)、ledger.rs 测试(563/565 NodePaymentIntent 构造、626/658/689 on_payment_settled 调用参数 payer_bank/recipient_bank/my_bank → CID)、packer.rs(340/549)、rpc.rs(465-580 存储键单测按 CID 键 + 580 intent 构造)、submitter.rs(252)。
- 模式与 cases.rs 同:NodePaymentIntent/OffchainBatchItem 的 payer_bank→payer_bank_cid(值用 CID Vec<u8>/字符串字节)、on_payment_settled 三个 bank 参数改 &[u8] CID、storage_key 按 CID 键、探针 item Default CID。
- cargo test -p node 通过后本卡收尾。

历史剩余条目（已被上面里程碑覆盖）：
- 先更新 offchain tests/mod.rs、tests/cases.rs、benchmarks.rs 的批次/intent 构造为 CID(payer_bank_cid 等)+ 各 storage 键/断言。
- 跑 cargo test -p offchain 捕获 fixture1/2/3 的 signing_hash 真值,回填 batch_item.rs 的 __FILL1/2/3__。
- Dart payment_intent_golden_test.dart:同一 CID fixture(cid 值须与 Rust 完全一致)+ 同一期望 hex,flutter test 验证逐字节。
- rpc.rs 存储键单测、mod.rs batch_seq 键单测、node_guard 探针、ledger/packer 夹具按 CID 刷新。
1. node/transaction/offchain/mod.rs：start_clearing_bank_components(_with_noop) 加 `clearing_bank_cid: Vec<u8>` 参数;read_last_clearing_batch_seq 的 LastClearingBatchSeq 键改按 CID(Compact(len)‖bytes);EventListener::new / OffchainClearingRpcImpl::new 传 clearing_bank_cid;packer/reserve_monitor 仍用 institution_account(主账户,发 extrinsic 用)。
2. 节点服务启动配置(mod.rs 的调用方):新增 CID 配置字段并透传。
3. node/core/node_guard/runtime_policy.rs：storage_key::rate() 费率键构造器按 CID 重编;PaymentSettled 匹配 recipient_bank→recipient_bank_cid;探针 item payer_bank_cid/recipient_bank_cid=Default。
4. rpc RPC-trait(OffchainClearingRpcServer):query_user_bank 返回 CID、query_fee_rate 入参 CID —— 与 Dart 一并改。
5. Dart ×5：payment_intent.dart 字段改 CID + rpc/scan_flow/pay_page；RPC 入参/出参改 CID。
6. 双端金标重生：Rust batch_item.rs 三 fixture(intent 布局变长)+ Dart payment_intent_golden_test.dart，逐字节对齐;L2 ACK 消息(bank_cid)Dart 镜像同步。
7. node/rpc.rs 存储键单测(465-526)与各端 #[cfg(test)] 夹具:按 CID 键/字段刷新。

验收标准：
- cargo check --workspace 通过；offchain/node 全绿
- PaymentIntent SCALE + signing_hash 金标 Rust 三 fixture + Dart 逐字节对齐
- 绑定 CID → 充值 → 结算(同行/跨行) → 提现全链路按 CID 记账，偿付守住
- dart analyze / Dart 测试通过
- 落地顺序：本卡(CID 主键) → Step 2(清算账户资金流) → Step 3(手续费收链上费)
