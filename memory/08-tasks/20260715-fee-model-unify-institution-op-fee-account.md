# 任务卡：费用模型统一五类 + 机构操作从费用账户扣 + 公民快照收归投票引擎

## 任务需求

来源:注册局占号报 `InvalidTransaction::Payment` 诊断([[project_chain_fee_model_and_payment_diagnosis]])延伸出的收费模型重构。用户拍板三件事合为一卡:

1. **费用模型统一严格 5 类 + 默认拒绝**:除框架自带系统交易免费外,所有投票=投票费、所有链上交易=链上交易费、所有链下交易=链下交易费,不在这 4 类的一律拒绝(`Unknown→InvalidTransaction::Call`)。
2. **机构操作签名与扣费分离**:注册局(及任一机构)对公民/机构数据的上链操作,由管理员签名,但**手续费从该机构的费用账户扣**,每笔收最低链上费(0.1 元);单源模型=机构唯一 CID→唯一费用账户(OP_FEE)→唯一管理员集。防伪造用 `is_admin_of` 卫语 + 入池期 SignedExtension。
3. **公民人口快照收归投票引擎内部生成**:删掉 3 个 public `prepare_*_population_snapshot` extrinsic,由发起公投的提案流程内联生成(仿 election-vote),快照退出交易层。

拨付不管(NRC 转账给各机构费用账户,机构自管余额)。runtime breaking,链开发期直接改 + 重新创世,不做迁移([[feedback_chain_dev_never_ask_migration]])。

## 所属模块

citizenchain runtime(收费层 + citizen-identity/joint-vote/legislation-vote)+ onchina(报错文案)+ CitizenWallet(删快照残桩)。

## 输入文档

- 收费单源 `runtime/primitives/src/fee_policy.rs`;适配器 `runtime/transaction/onchain/src/lib.rs`;分类器/付费方 `runtime/src/configs.rs`
- 账户派生 `runtime/primitives/src/account_derive.rs`(OP_FEE);`fee_account_of` `runtime/transaction/offchain/src/bank_check.rs:141`;`is_active_admin_of_account` `runtime/src/configs.rs:1476`
- 机构操作 call:`runtime/entity/public-manage/src/lib.rs`、`.../private-manage`;公民操作 `runtime/misc/citizen-identity/src/lib.rs`
- 快照:`runtime/votingengine/joint-vote/src/jointinternal.rs`、`.../legislation-vote/src/legislation/referendum.rs`、election-vote 内联范式 `election-vote/src/lib.rs:321`
- [[project_fee_policy_unified]] [[project_citizenwallet_call_registration_three_points]] [[project_cid_occupy_registry_2026_07_02]]

## 目标分类(五类终态)

**免费 Free(仅框架系统 + 全链维护)**:System / Timestamp / Grandpa / Assets(CallFilter 已拦);`cleanup_rejected_public_proposal` / `cleanup_rejected_private_proposal`(幂等 GC);Root/回调专用内部 call(不走签名费路径,分类防御性)。

**链上交易费 OnchainAmount = `max(金额×0.1%, ONCHAIN_MIN_FEE=0.1元)`**:
- 付费方=签名者本人:`transfer_with_remark`(按金额)、`SquarePost`(0金额=0.1元)、清算 deposit/withdraw(当前 CallFilter 禁用)。
- **付费方=机构费用账户(机构操作,`OnchainAmount(0)`=0.1元)**:
  - CitizenIdentity:`occupy_cid`/`occupy_cids_batch`/`revoke_cid`/`register_voting_identity`/`upgrade_to_candidate_identity`/`update_voting_identity`/`update_candidate_identity`/`revoke_identity`(acting=`registrar_account`)
  - public/private-manage:`register_cid_*`/`propose_create_*`/`propose_close_*`/`update_institution_info`/`add_institution_account`(acting=`issuer_main_account`)

**投票费 VoteFlat = 1元(付费方=签名者)**:InternalVote/JointVote/LegislationVote/ElectionVote 的 cast_*;VotingEngine 的 `finalize_proposal`(手动,自动结算有 on_initialize 兜底免费)/`retry_passed_proposal`/`cancel_passed_proposal`;MultisigTransfer/OnchainIssuance/LegislationYuan/RuntimeUpgrade/GrandpaKeyChange/PersonalManage/PersonalAdmins 的 propose_X;AddressRegistry。

**链下交易费 OffchainFee**:submit_offchain_batch(当前 CallFilter 禁用,保留分类)。

**拒绝 Unknown**:OnchainTransaction 非 transfer_with_remark、Balances 原生,及一切未归类(穷尽 match,新 pallet 漏分类编译报错)。

## 落地方案(文件级)

### 单源「机构操作」路由(不新建文件,因 RuntimeCall 只在 runtime crate)
`runtime/src/configs.rs` 新增共享函数 `acting_institution_of(call) -> Option<AccountId>`:逐一 match 机构操作 call、取出 acting 机构账户(registrar_account / issuer_main_account),其余返回 None。**一函数三处消费**(分类器/付费方/SignedExtension),杜绝漂移。协议常量仍在 `primitives::fee_policy`(复用 `ONCHAIN_MIN_FEE`,补策略注释),因依赖方向(runtime→primitives)禁止 primitives 反向 match RuntimeCall。

### 1. `runtime/src/configs.rs`
- `RuntimeFeeKindClassifier::fee_kind`:先 `if acting_institution_of(call).is_some() → OnchainAmount(0)`;`finalize_proposal→VoteFlat`;`cleanup_rejected_*→Free`;Free 收紧到框架自带;删占号=Free 特例。**每条中文注释重写为五类原则**。
- `RuntimeFeePayerExtractor::fee_payer`:`if let Some(acc)=acting_institution_of(call){ if is_active_admin_of_account(&acc,who){ return fee_account_of(&acc).ok() } }`;否则 None(回落签名者)。保留 submit_offchain_batch 分支。

### 2. `runtime/src/lib.rs`
- 新增 `CheckInstitutionOpAuth` SignedExtension,插入 `TxExtension`(`ChargeTransactionPayment` 之前):`validate` 阶段对 `acting_institution_of(call)=Some(acc)` 的交易校验 `is_active_admin_of_account(acc, signer)`,不符 → `InvalidTransaction::BadSigner`(冒用别家 registrar 交易进不了池、不打包、不扣费)。

### 3. 公民快照收归引擎(删 3 extrinsic)
- `runtime/misc/citizen-identity/src/lib.rs`:删 `prepare_population_snapshot` extrinsic(保留 trait `create_population_snapshot`)。
- `runtime/votingengine/joint-vote/src/lib.rs` + `jointinternal.rs`:删 `prepare_joint_population_snapshot` extrinsic + `PendingPopulationSnapshots` storage;`do_create_joint_proposal` 增 `scope` 入参、内部调 `create_population_snapshot(scope)`(仿 election-vote)。
- `runtime/votingengine/legislation-vote/src/lib.rs` + `legislation/referendum.rs`:同样删 `prepare_population_snapshot` extrinsic,提案内联生成。
- 快照从此不进费用分类器。

### 4. `runtime/transaction/onchain/src/lib.rs` / `primitives/src/fee_policy.rs`
- 逻辑不改;头注措辞更新(免费=框架系统+维护;机构操作=最低链上费从费用账户扣)。

### 5. `citizenchain/onchina/src/domains/citizens/occupy.rs`·`chain_identity.rs`·机构创建关闭 handler
- 签名不变;`InvalidTransaction::Payment` 映射「注册局费用账户余额不足」文案;可加提交前费用账户余额预检。占号/机构操作注释:免费→0.1元费用账户扣。

### 6. CitizenWallet 残桩清理(快照删除连带)
- `citizenwallet/lib/signer/payload_decoder.dart` + `pallet_registry.dart` + `qr/qr_protocols.dart`:删 `prepare_joint_population_snapshot`(0x1502)、`prepare_legislation_snapshot`(0x1a00)解码/常量/`fromDecodedAction`;删对应测试用例。([[project_citizenwallet_call_registration_three_points]] 反向:删 call 同样三处清)

### 7. 测试
- runtime:分类穷尽性、机构操作→OnchainAmount(0)、付费方=机构费用账户、SignedExtension 拒冒用(is_admin_of 假→BadSigner)、框架 Free 不变、finalize→VoteFlat、cleanup→Free、费用账户不足→Payment。
- 快照:提案内联生成快照、删 extrinsic 后编译与公投流程 GREEN。
- onchina:费用账户不足报错路径。

### 8. `docs/decisions/` 新 ADR
- 记「五类费用统一 + 机构操作从费用账户扣(签名/扣费分离,is_admin_of + SignedExtension 双层防伪)+ 快照收归引擎」。

## 铁律
- 单源:`acting_institution_of` 一处匹配;机构唯一 CID→费用账户(OP_FEE)/管理员集。
- 穷尽分类 + 默认拒绝(已在),两层安全正交:①无 call 绕过收费 ②无签名者盗刷别家费用账户。
- 只在主检出 `/Users/rhett/GMB` 操作,不碰 worktree([[feedback_user_evaluates_in_main_checkout]]);改动留工作区不提交供 review。无残留([[feedback_no_remnants]])。

## 验收
- `cargo test`(分类穷尽 + 机构操作扣费用账户 + SignedExtension 拒冒用 + 快照内联)GREEN;`cargo clippy` 零新增。
- CitizenWallet `flutter test`/`analyze` GREEN(快照残桩清零)。
- 重新创世后:注册局费用账户注资后占号/机构操作 0.1 元成功;空账户扣不到别家;冒用交易入池即拒。

## 待确认边界
- MultisigTransfer/OnchainIssuance 等**其它机构的业务 propose_X 保持 VoteFlat**(不并入机构操作 0.1 元);机构操作仅限 CitizenIdentity + public/private-manage(=注册局对公民/机构数据的登记管理)。
