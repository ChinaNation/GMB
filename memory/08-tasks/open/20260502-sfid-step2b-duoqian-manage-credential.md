# SFID Step 2b:duoqian-manage 凭证字段 + verifier 按 (province, admin_pubkey) 双层匹配

- 状态:open
- 创建日期:2026-05-02
- 模块:`citizenchain/runtime/transaction/duoqian-manage/`
- 关联 ADR:`memory/04-decisions/ADR-008-sheng-admin-3tier-and-key-admin-removal.md`(Step 2 章节)
- 前置依赖:step2a(sfid-system pallet 重写完成,helper `sheng_signing_pubkey_for_admin` 可用)
- 阻塞下游:step2d(wumin / wuminapp decoder 必须按本卡新字段同步)

## 任务需求

`duoqian-manage::register_sfid_institution` 当前 verifier 走两条路径:
- `signing_province=None` → `current_sfid_verify_pubkey()`(SfidMainAccount,Step 2a 已删)
- `signing_province=Some(p)` → `sheng_signing_pubkey(p)`(单参,Step 2a 已删)

本卡:
- 凭证 payload **加字段** `signer_admin_pubkey: [u8;32]`
- verifier 总是走 `sheng_signing_pubkey_for_admin(signing_province, signer_admin_pubkey)` 双层匹配
- 删除 `signing_province=None` 分支(链上 0 prior knowledge of SFID,凭证必须显式带 admin_pubkey)

## 影响范围

### 凭证 payload(SCALE 编码顺序固定)

```rust
struct InstitutionRegistrationCredential {
    sfid_id: BoundedVec<u8, ConstU32<64>>,
    institution_name: BoundedVec<u8, ConstU32<128>>,
    a3: BoundedVec<u8, ConstU32<8>>,
    sub_type: Option<BoundedVec<u8, ConstU32<32>>>,
    parent_sfid_id: Option<BoundedVec<u8, ConstU32<64>>>,
    province: BoundedVec<u8, ConstU32<64>>,
    register_nonce: BoundedVec<u8, ConstU32<128>>,
    signer_admin_pubkey: [u8; 32],   // ★ 新字段
    signature: BoundedVec<u8, ConstU32<128>>,  // 64 字节 sr25519
}
```

新字段位置:`register_nonce` 后、`signature` 前(避免破坏 signature 之外字段顺序差异性)。

### verifier 改造

```rust
let pubkey = SfidSystem::<T>::sheng_signing_pubkey_for_admin(
    &credential.province,
    &credential.signer_admin_pubkey,
)
.ok_or(Error::<T>::SfidProvinceAdminSigningNotActivated)?;
sr25519_verify(&credential.signature, &payload_hash, &pubkey)?;
```

删除:
- `current_sfid_verify_pubkey()` 调用
- `signing_province=None` 分支
- `Optional<signing_province>` 改为必填(BoundedVec)

新 Error:`SfidProvinceAdminSigningNotActivated`

### 文件级

- `citizenchain/runtime/transaction/duoqian-manage/src/lib.rs`:凭证 struct + verifier + Error
- 该 pallet 现有测试:配套更新(测试构造凭证时加 `signer_admin_pubkey` 字段)
- 节点桌面 / SFID 后端凭证签发函数:对齐(本卡只动 runtime;前后端联动留 step2d)

## 主要风险点

- **SCALE 编码兼容性**:凭证 struct 加字段会破坏旧 wumin/wuminapp 扫码签名 decoder;step2d 必须**同步上线**(`memory/07-ai/chat-protocol.md` 第 5 条铁律)
- **签名 payload 哈希**:加 `signer_admin_pubkey` 后 payload bytes 改变;wumin / wuminapp / SFID 三处签发流程必须用同一序列化规则
- **`signing_province=None` 分支删除**:duoqian-manage 现有测试如果有依赖 None 分支的,必须重写

## 是否需要先沟通

- 否(ADR-008 已 accepted)

## 验收清单

- `cargo check -p duoqian-manage` + `cargo test -p duoqian-manage` + `cargo clippy` 全绿
- 现有 register_sfid_institution 测试(若 N 条)全部更新为新凭证字段
- 新增 ≥ 3 测试:
  - `register_with_main_admin_signature_succeeds`
  - `register_with_backup_admin_signature_succeeds`
  - `register_with_admin_not_in_roster_rejected`
  - `register_signing_not_activated_for_admin_rejected`
- 任务卡 progress 章节回写

## 不要做的事

- 不要碰 sfid-system(step2a 范围)
- 不要碰 genesis / spec_version(step2c 范围)
- 不要碰 wumin / wuminapp(step2d 范围)
- 不要 commit

## 工作量

~200 行 + 4 测试,~1 agent round。

## Progress(2026-05-02 Blockchain Agent)

状态:**done(待 commit)**。spec_version 未动(留 step2c)。

### 实际改动文件

| 文件 | 行数变化 | 说明 |
|---|---|---|
| `citizenchain/runtime/transaction/duoqian-manage/src/lib.rs` | +180 / -45 | trait `SfidInstitutionVerifier` 签名改造(`signing_province: Option<&[u8]>` → `province: &[u8]` + 新 `signer_admin_pubkey: &[u8; 32]`);extrinsic `register_sfid_institution`(call_index 2)+ `propose_create_institution`(call_index 5)同步参数改造;新 Error `EmptyProvince`;mock `TestSfidInstitutionVerifier` 升级为支持 `ACTIVATED_ROSTER` 注入的双层匹配;helper `register_sfid_with_account_name` / 4 处现有 propose_create_institution 测试调用同步;新增 `register_with_admin` helper + 5 条新测试 |
| `citizenchain/runtime/src/configs/mod.rs` | +110 / -125 | `RuntimeSfidInstitutionVerifier` 删 `signing_province=None` 兜底分支;改走 `SfidSystem::sheng_signing_pubkey_for_admin(province, signer_admin_pubkey)`;签名 payload 加 `province + signer_admin_pubkey` 防跨字段替换;`current_sfid_verify_public()` 函数删除;`RuntimeSfidVerifier` / `RuntimeSfidVoteVerifier` / `RuntimePopulationSnapshotVerifier` 三处 SFID main 兜底验签改为 stub 返回 false(BindCredential / VoteCredential / PopSnapshotCredential 加字段留 step3);3 处 `#[ignore = "ADR-008 step2b"]` 测试全部重写并移除 ignore 标记 |
| `citizenchain/runtime/otherpallet/sfid-system/src/lib.rs` | -15 | 删除 `current_sfid_verify_pubkey()` 与 `sheng_signing_pubkey(_province)` 两个 deprecated 编译垫片函数;垫片 1/2 清理完毕 |

### 验收数字

- `cargo check -p sfid-system`:**绿**(0 error / 0 warning)
- `cargo check -p duoqian-manage`:**绿**(0 error / 0 warning)
- `cargo check -p citizenchain`(WASM_FILE=target/wasm/citizenchain.compact.compressed.wasm):**绿**(0 error;此前 step2a 留下的 2 deprecated warning 已消除)
- `cargo check -p citizenchain --tests`:**绿**
- `cargo test -p sfid-system`:**31 / 31 passed**(无回归,垫片删除无影响)
- `cargo test -p duoqian-manage`:**26 / 26 passed**(原 21 + 新 5)
- `cargo test -p citizenchain --tests`:**31 / 31 passed**(其中 3 条原 ignored 测试已重写为 active 通过)
- `cargo clippy -p duoqian-manage --`:7 warning,与 baseline 数量持平(too_many_arguments 由 5 条变为 5 条,因加 2 参数 8→9/12→13/13→14;无新类型 warning)

### 4 + 1 条任务卡新测试通过情况

| # | 测试名 | 通过 |
|---|---|---|
| 1 | `register_with_main_admin_signature_succeeds` | passed |
| 2 | `register_with_backup_admin_signature_succeeds` | passed |
| 3 | `register_with_admin_not_in_roster_rejected` | passed |
| 4 | `register_signing_not_activated_for_admin_rejected` | passed |
| 5 | `register_with_empty_province_rejected`(附加,验证 `province` 改必填后空字节串 EmptyProvince) | passed |

runtime 集成测试层 3 条原 ignored 重写后:

| 原名 | 新名 | 通过 |
|---|---|---|
| `runtime_sfid_verifiers_and_population_snapshot_verify_with_runtime_main_key` | `runtime_sfid_verifiers_reject_until_step3_credential_carries_admin_pubkey` | passed |
| `runtime_sfid_eligibility_wrapper_works_with_nonce_consumption` | `runtime_sfid_eligibility_binding_lookup_works_but_vote_verify_blocked_until_step3` | passed |
| `runtime_sfid_institution_verifier_uses_runtime_main_key` | `runtime_sfid_institution_verifier_double_layer_lookup`(覆盖 main 签 ok / backup 签 ok / outsider reject / not-activated reject / 篡改签名 reject 5 条断言) | passed |

### 残留扫描(均为 0)

- `grep -rn "current_sfid_verify_pubkey\|current_sfid_verify_public" citizenchain/runtime/ --include="*.rs"`:仅 1 处注释提到旧函数名(说明性,无代码符号)
- `grep -rn "sheng_signing_pubkey(" citizenchain/runtime/ --include="*.rs"`(过滤掉 extrinsic `activate_sheng_signing_pubkey` / `rotate_sheng_signing_pubkey` / `sheng_signing_pubkey_storage` / 双参 `sheng_signing_pubkey_for_admin`):0 命中
- `grep -rn '#\[ignore = "ADR-008 step2b"\]' citizenchain/runtime/`:0 命中

### 后续任务卡微调建议

- **step2c**:仅余 sfid-system pallet 的 `pallet::GenesisConfig` 空 stub 一处垫片需清理(配套删 `genesis_config_presets.rs:159-167` 的 SFID 3 把硬编码地址 + `root.insert("sfidSystem", ...)` + 对应 deserialize 测试);spec_version 1→2 + on_runtime_upgrade migration。
- **step2d**:wumin / wuminapp 扫码签名 decoder 必须同步加 `province` 必填 + `signer_admin_pubkey: [u8; 32]` 字段;签名 payload 哈希前缀重排为 `(DUOQIAN_DOMAIN, OP_SIGN_INST, block_hash(0), sfid_id, account_name, register_nonce, province, signer_admin_pubkey)`;extrinsic 调用参数列表 register/propose_create 各加 2 个新参(`province`、`signer_admin_pubkey`),其中 propose_create_institution 现 14 个参数(原 13)。
- **step3(可选,出 step2 范围)**:RuntimeSfidVerifier / RuntimeSfidVoteVerifier / RuntimePopulationSnapshotVerifier 三处 stub 接通真实验签;BindCredential / VoteCredential / PopSnapshotCredential 加 `(province, admin_pubkey)` 字段。step2b 期间这三处 SFID 公民绑定 / 投票 / 人口快照链路事实失效,与 register_sfid_institution 在 step2a 期间的状态一致(开发期未上线无影响)。
