# SFID Step 3:citizenchain runtime BindCredential / VoteCredential / PopSnapshotCredential 双层 verifier 接通

- 状态:open
- 创建日期:2026-05-02
- 模块:`citizenchain/runtime/`(otherpallet/sfid-system + governance/voting-engine + issuance/citizen-issuance + issuance/resolution-issuance + governance/runtime-upgrade + src/configs/mod.rs)
- 关联 ADR:`memory/04-decisions/ADR-008-sheng-admin-3tier.md`(Step 2 章节)
- 前置依赖:step2a / step2b / step2c(commit 6e1779c)全部完工
- 阻塞下游:step2d(wumin / wuminapp decoder 必须同步加 (province, signer_admin_pubkey) 字段)

## 任务需求

step2b 期间链上三处 SFID verifier(`RuntimeSfidVerifier` / `RuntimeSfidVoteVerifier` /
`RuntimePopulationSnapshotVerifier`)被改为 stub 一律返回 false,
本卡接通真实双层 (province, admin_pubkey) 验签。

## 改动范围

### 1. sfid-system pallet 凭证结构

`BindCredential<Hash, Nonce, Signature>` 加字段:
```rust
pub struct BindCredential<Hash, Nonce, Signature> {
    pub binding_id: Hash,
    pub bind_nonce: Nonce,
    pub province: BoundedVec<u8, ConstU32<64>>,    // ★新
    pub signer_admin_pubkey: [u8; 32],             // ★新
    pub signature: Signature,
}
```

### 2. trait 签名扩展

- `SfidVerifier::verify` 走凭证内 `province + signer_admin_pubkey`
- `SfidVoteVerifier::verify_vote` 入参 + `province` + `signer_admin_pubkey`
- `PopulationSnapshotVerifier::verify_population_snapshot` 入参 + `province` + `signer_admin_pubkey`
- `SfidEligibilityProvider::verify_and_consume_vote_credential` 入参 + `province` + `signer_admin_pubkey`
- `JointVoteEngine::create_joint_proposal*` 入参 + `province` + `signer_admin_pubkey`

### 3. runtime configs/mod.rs 三处 verifier 接通真实双层验签

走 `sfid_system::Pallet::sheng_signing_pubkey_for_admin(province, admin_pubkey)` 查派生公钥,
payload 走 `(DUOQIAN_DOMAIN, OP_SIGN_BIND/VOTE/POP, block_hash(0), <业务字段>, province, admin_pubkey)`。

### 4. 业务 pallet caller

- `voting_engine::citizen_vote` extrinsic 加 2 参
- `resolution_issuance::propose_resolution_issuance` extrinsic 加 2 参
- `runtime_upgrade::propose_runtime_upgrade` extrinsic 加 2 参

### 5. 测试

step2b 留下 2 条 active 测试改为正向断言:
- `runtime_sfid_verifiers_reject_until_step3_credential_carries_admin_pubkey` → `runtime_sfid_verifiers_double_layer_verify_succeeds`
- `runtime_sfid_eligibility_binding_lookup_works_but_vote_verify_blocked_until_step3` → `runtime_sfid_eligibility_binding_and_vote_full_path`

新加测试覆盖 main / backup / outsider / cross-province 路径。

## 不要做的事

- 不要 commit
- 不要碰 SFID 后端(并行 phase7 agent 范围)
- 不要碰 wumin / wuminapp(step2d 范围)

## Progress(2026-05-02 Blockchain Agent)

状态:**done(待 commit)**。spec_version 未动(留 step2c/step2d 协调)。

### 三处 verifier 接通情况

- **`RuntimeSfidVerifier`(BindCredential)**:单签结构;`BindCredential` 加 `province + signer_admin_pubkey` 两字段 SCALE 顺序固定为 `binding_id → bind_nonce → province → signer_admin_pubkey → signature`;payload `(DUOQIAN_DOMAIN, OP_SIGN_BIND, block_hash(0), account, binding_id, bind_nonce, province, signer_admin_pubkey)` blake2_256 → sr25519_verify。
- **`RuntimeSfidVoteVerifier`(VoteCredential)**:单签结构;trait 入参补 `(province, signer_admin_pubkey)`,`citizen_vote` extrinsic 加两参;payload `(DUOQIAN_DOMAIN, OP_SIGN_VOTE, block_hash(0), account, binding_id, proposal_id, nonce, province, signer_admin_pubkey)`。
- **`RuntimePopulationSnapshotVerifier`(PopSnapshotCredential)**:单签结构(非聚合);trait 入参补 `(province, signer_admin_pubkey)`;`JointVoteEngine::create_joint_proposal*` 三个入口同步加两参;`propose_resolution_issuance` / `propose_runtime_upgrade` extrinsic 各加两参;payload `(DUOQIAN_DOMAIN, OP_SIGN_POP, block_hash(0), who, eligible_total, nonce, province, signer_admin_pubkey)`。

### 实际改动文件清单

| 文件 | 行数变化 | 说明 |
|---|---|---|
| `runtime/otherpallet/sfid-system/src/lib.rs` | +60 / -10 | `BindCredential` 加 2 字段;`SfidVoteVerifier` / `SfidEligibilityProvider` trait 补 (province, signer_admin_pubkey) 入参;`Pallet::verify_and_consume_vote_credential` 透传 |
| `runtime/otherpallet/sfid-system/src/tests.rs` | +30 / -10 | 测试 helper `bind_credential` + `vote_credential_*` 测试调用同步 |
| `runtime/otherpallet/sfid-system/src/benchmarks.rs` | +14 / -2 | bind_sfid benchmark payload 加 province + signer_admin_pubkey 进 hash |
| `runtime/governance/voting-engine/src/lib.rs` | +120 / -25 | `PopulationSnapshotVerifier` / `JointVoteEngine` 三个入口 trait 补 2 参;`citizen_vote` extrinsic 补 2 参;`do_create_joint_proposal` 透传;`TestPopulationSnapshotVerifier` / `TestSfidEligibility` test mock + 全部 13 个 citizen_vote / 21 个 create_joint_proposal 测试调用同步;新加 `province_ok` / `signer_admin_pubkey_ok` helper |
| `runtime/governance/voting-engine/src/citizen_vote.rs` | +25 / -5 | `SfidEligibility` trait 补 2 参;`do_citizen_vote` 透传 |
| `runtime/governance/voting-engine/src/joint_vote.rs` | +12 / -3 | `do_create_joint_proposal` 加 2 参 + 透传 |
| `runtime/issuance/resolution-issuance/src/lib.rs` | +6 / -0 | extrinsic 加 2 参 |
| `runtime/issuance/resolution-issuance/src/proposal.rs` | +6 / -0 | `create_resolution_issuance_proposal` 加 2 参 + 透传 |
| `runtime/issuance/resolution-issuance/src/tests.rs` | +25 / -0 | TestJointVoteEngine / TestSfidEligibility / TestPopulationSnapshotVerifier mock 同步;`province_ok` / `signer_admin_pubkey_ok` helper;12 处 propose_resolution_issuance 测试调用同步 |
| `runtime/governance/runtime-upgrade/src/lib.rs` | +30 / -5 | extrinsic 加 2 参;TestJointVoteEngine mock + 4 处 propose_runtime_upgrade 测试调用同步;`province_ok` / `signer_admin_pubkey_ok` helper |
| `runtime/issuance/citizen-issuance/tests/integration_bind_sfid.rs` | +12 / -0 | TestSfidVoteVerifier mock + `make_credential` 同步 |
| `runtime/transaction/duoqian-manage/src/lib.rs` | +6 / -0 | TestSfidEligibility / TestPopulationSnapshotVerifier mock 同步 |
| `runtime/src/configs/mod.rs` | +210 / -100 | 三处 stub verifier 接通真实双层验签;`RuntimeSfidEligibility::verify_and_consume_vote_credential` 加 2 参透传到 sfid_system;**两条 step2b active 测试改名 + 重写为 step3 active 路径**(`runtime_sfid_verifiers_double_layer_verify_succeeds` + `runtime_sfid_eligibility_binding_and_vote_full_path`),新增 helper `setup_step3_test_admins` / `build_bind_credential` / `build_vote_signature` / `build_pop_signature`,新增 6 条独立测试 |

### 验收数字

- `cargo check -p sfid-system -p voting-engine -p resolution-issuance -p runtime-upgrade -p citizen-issuance -p duoqian-manage`:**全绿**
- `cargo check -p citizenchain`(WASM_FILE 设):**绿**(0 error)
- `cargo check -p citizenchain --tests`:**绿**
- `cargo test -p sfid-system`:**31 / 31 passed**(baseline 不动)
- `cargo test -p duoqian-manage`:**26 / 26 passed**(baseline 不动)
- `cargo test -p voting-engine`:**73 / 73 passed**(baseline 不动)
- `cargo test -p citizen-issuance`:**7 / 7 passed**(集成测试 4 + 内部 3,baseline 不动)
- `cargo test -p resolution-issuance`:**28 / 28 passed**
- `cargo test -p runtime-upgrade`:**16 / 16 passed**
- `cargo test -p citizenchain --tests`:**37 / 37 passed**(原 31 + 2 重写为正向路径 + 6 新加 = 39 起步,与 31 baseline + 6 新加 = 37 一致;计数差源于 setup_step3_test_admins 是 helper 不计入 test 计数)
- `cargo clippy`:与 baseline 持平(`too_many_arguments` 已存量;无新类型 warning)

### ignored 测试改 active 数量 + 新加测试数量

- step2b 残留 active 但断言 `assert!(!verify(...))` 的 2 条 → 重写为 active 正向断言 2 条:
  - `runtime_sfid_verifiers_reject_until_step3_credential_carries_admin_pubkey` → `runtime_sfid_verifiers_double_layer_verify_succeeds`
  - `runtime_sfid_eligibility_binding_lookup_works_but_vote_verify_blocked_until_step3` → `runtime_sfid_eligibility_binding_and_vote_full_path`
- 新加测试 6 条:
  - `bind_with_main_admin_signature_succeeds`
  - `bind_with_backup_admin_signature_succeeds`
  - `bind_with_admin_not_in_roster_rejected`
  - `vote_double_layer_verify_succeeds`
  - `vote_cross_province_admin_rejected`
  - `population_snapshot_per_province_signature_verifies`
- `#[ignore]` runtime/ 下扫描 = 0(沿袭 step2b 状态)

### 残留扫描

- `grep -n "return false" runtime/src/configs/mod.rs` 三处 verifier 区段:仅余 6 处 `None => return false` / sig 长度校验 `return false`(均为合法验签短路,非 stub 兜底)
- `grep -rn "step3" runtime/src/configs/mod.rs`:全部为 ADR-008 注释 / 测试名 / 测试 seed 字符串
- `grep -rn '#\[ignore\]' runtime/`:0 命中

### 后续任务卡微调建议

- **step2d**(wumin / wuminapp):decoder 必须同步加:
  - `BindCredential`:`province` + `signer_admin_pubkey` 字段(扫码 payload SCALE 序列已变)
  - `VoteCredential`(wuminapp citizen_vote):`province` + `signer_admin_pubkey`
  - `PopSnapshotCredential`(链下签发处):`province` + `signer_admin_pubkey`
  - 业务 extrinsic 调用参数列表:`citizen_vote` 加 2 参;`propose_resolution_issuance` 加 2 参;`propose_runtime_upgrade` 加 2 参
  - 签名 payload 哈希前缀必须与 runtime verifier 完全一致:`(DUOQIAN_DOMAIN, OP_SIGN_BIND/VOTE/POP, block_hash(0), <业务字段>, province, signer_admin_pubkey)`
- **SFID 后端**(phase7 并行):`bind_sfid` / `vote_sfid` / `population_snapshot` 三处签发函数对齐新 payload 与新 SCALE 顺序

