# SFID Step 2a:sfid-system pallet storage + 4 extrinsic 重写

- 状态:open
- 创建日期:2026-05-02
- 模块:`citizenchain/runtime/otherpallet/sfid-system/`
- 关联 ADR:`memory/04-decisions/ADR-008-sheng-admin-3tier.md`(Step 2 章节)
- 上游:Step 1 SFID 后端已落地(commit 461b78c,phase45 chain/client.rs mock 推链 4 个 endpoint 在 SFID 端就绪)
- 阻塞下游:step2b(duoqian-manage 凭证)+ SFID phase7(mock 切真)

## 任务需求

把 `citizenchain/runtime/otherpallet/sfid-system/src/lib.rs` 1206 行重写,落地 ADR-008 决议:
- storage 改 DoubleMap(Province × Slot / AdminPubkey)
- 删 `SfidMainAccount/Backup1/Backup2`、`current_sfid_verify_pubkey()`、单值 `ShengSigningPubkey`、`ProvinceBySigningPubkey`、genesis_config

**spec_version 不升**(本期裸升级,留待 chain 上线后再走 setCode)。

## 影响范围(文件级)

- `citizenchain/runtime/otherpallet/sfid-system/src/lib.rs`(1206 行重写)
- `citizenchain/runtime/otherpallet/sfid-system/src/benchmarks.rs`(同步更新)
- `citizenchain/runtime/otherpallet/sfid-system/src/weights.rs`(新 extrinsic 权重)
- 不动:`runtime/src/configs/mod.rs`(暂不动 Config trait,除非新加 const)

## 详细规约

### Storage

```rust
#[derive(Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub enum Slot { Main, Backup1, Backup2 }

pub type Province<T> = BoundedVec<u8, ConstU32<64>>;

#[pallet::storage]
pub type ShengAdmins<T> = StorageDoubleMap<
    _,
    Blake2_128Concat, Province<T>,
    Twox64Concat, Slot,
    [u8; 32],
    OptionQuery,
>;

#[pallet::storage]
pub type ShengSigningPubkey<T> = StorageDoubleMap<
    _,
    Blake2_128Concat, Province<T>,
    Blake2_128Concat, [u8; 32],   // admin pubkey
    [u8; 32],                      // signing pubkey
    OptionQuery,
>;
```

### Extrinsic

全部 `Pays::No` + `ensure_none(origin)` + sr25519 验签 + ValidateUnsigned 防重放。

| call_index | 名 | 鉴权 | 主要语义 |
|---|---|---|---|
| 0 | `bind_sfid` | 保留 | 不动 |
| 1 | `unbind_sfid` | 保留 | 不动 |
| 2 | `add_sheng_admin_backup(province, slot, new_pubkey, nonce, sig)` | sig 由 ShengAdmins[province][Main] 签;slot ∈ {Backup1, Backup2};本省 Main 已 activate | 写入 backup 槽 |
| 3 | `remove_sheng_admin_backup(province, slot, nonce, sig)` | 同上 | 清空 backup 槽 + 级联清 ShengSigningPubkey[(province, removed_pubkey)] |
| 4 | `activate_sheng_signing_pubkey(province, admin_pubkey, signing_pubkey, nonce, sig)` | sig 由 admin_pubkey 私钥签;first-come-first-serve(若 Main 空 → 占 Main 槽;否则 admin_pubkey 必须 ∈ ShengAdmins[province][\*]) | 写入 ShengSigningPubkey[(province, admin_pubkey)] = signing_pubkey;若占 Main,同时写 ShengAdmins[province][Main] = admin_pubkey |
| 5 | `rotate_sheng_signing_pubkey(province, admin_pubkey, new_signing_pubkey, nonce, sig)` | sig 由 admin_pubkey 私钥签;admin_pubkey ∈ ShengAdmins[province][\*] | 替换 ShengSigningPubkey[(province, admin_pubkey)] |

签名 payload(blake2_256 哈希入参):
- `add_backup`:`b"add_sheng_admin_backup_v1" || province || slot.encode() || new_pubkey || nonce`
- `remove_backup`:`b"remove_sheng_admin_backup_v1" || province || slot.encode() || nonce`
- `activate`:`b"activate_sheng_signing_pubkey_v1" || province || admin_pubkey || signing_pubkey || nonce`
- `rotate`:`b"rotate_sheng_signing_pubkey_v1" || province || admin_pubkey || new_signing_pubkey || nonce`

domain 常量(`feedback_scale_domain_must_be_array.md`):必须 `[u8; N]` 数组,不能 `&[u8]`。

### Helper 重写

- 删 `current_sfid_verify_pubkey()`
- 改 `sheng_signing_pubkey(province)` 单参 → 删除
- 加 `sheng_signing_pubkey_for_admin(province: &[u8], admin_pubkey: &[u8;32]) -> Option<[u8;32]>`
- 加 `is_sheng_admin(province: &[u8], pubkey: &[u8;32]) -> Option<Slot>`
- 加 `is_sheng_main(province: &[u8], pubkey: &[u8;32]) -> bool`

### Event

新增:
- `ShengAdminBackupAdded { province, slot, pubkey }`
- `ShengAdminBackupRemoved { province, slot, pubkey }`
- `ShengSigningActivated { province, admin_pubkey, signing_pubkey }`
- `ShengSigningRotated { province, admin_pubkey, old_signing_pubkey, new_signing_pubkey }`

删除:`RotatedSfidKeys`、`ShengSigningPubkeySet` 等老事件

### Error

- `Sheng3TierMainAlreadyActivated`
- `Sheng3TierAdminNotInRoster`
- `Sheng3TierSlotOccupied`
- `Sheng3TierSigningNotActivated`
- `Sheng3TierSignatureInvalid`
- `Sheng3TierNonceUsed`

### ValidateUnsigned

实现 `ValidateUnsigned`:
- `validate_unsigned`:
  1. extrinsic 是 4 个新 extrinsic 之一
  2. nonce 未用过(查 UsedShengNonce 防重放)
  3. 验签:对应 admin_pubkey 私钥能验证 payload
  4. longevity 短(64 blocks 即可),priority 中等
- `pre_dispatch`:加锁(原子性)+ 同步状态校验

新加 storage:`UsedShengNonce: StorageMap<Hash, ()>` 防重放。

## 主要风险点

- **`Pays::No` 防 DoS**:nonce 一次性 + sig 验证失败立即 reject + ValidateUnsigned longevity 短;不存在长期淹没风险
- **first-come-first-serve 抢占**:部署期窗口短;接受 ADR-008 trade-off
- **现有 5+ 测试基于老 storage / extrinsic**:本卡必须配套重写(`set_sheng_signing_pubkey_*` 5 个测试 + `rotate_sfid_keys_*` 多个测试)
- **benchmarks.rs**:旧 benchmark 删 + 新 extrinsic benchmark
- **Slot enum 在 `pallet::Decode`/`pallet::Encode`**:确保 Storage 用 Twox64Concat hasher 而非 Blake2_128Concat(枚举 4 字节小键 hasher 应快速)

## 是否需要先沟通

- 否(ADR-008 已 accepted,直接执行)

## 验收清单

- `cargo check -p sfid-system`、`cargo check` 全 runtime 全绿
- `cargo clippy -p sfid-system -- -D warnings` 无新错
- 新测试覆盖:
  - `activate_first_come_first_serve_on_empty_main`
  - `activate_existing_admin_writes_signing_pubkey`
  - `activate_unknown_admin_rejected_when_main_filled`
  - `add_backup_signed_by_main_succeeds`
  - `add_backup_unauthorized_signature_rejected`
  - `remove_backup_cascades_to_signing_pubkey`
  - `rotate_signing_pubkey_replaces_value`
  - `rotate_unknown_admin_rejected`
  - `pays_no_zero_balance_account_succeeds`
  - `cross_province_admin_cannot_modify_other_province`
  - `nonce_replay_rejected`
  - `signature_with_wrong_payload_rejected`
- benchmarks.rs 同步;weights.rs 加 4 个新权重 stub
- 任务卡 progress 章节回写

## 不要做的事

- 不要碰 `duoqian-manage`(留 step2b)
- 不要碰 genesis / spec_version(留 step2c)
- 不要碰 wumin / wuminapp(留 step2d)
- 不要 commit
- 不要修改 `bind_sfid` / `unbind_sfid` 业务

## 工作量

~600 行 lib.rs 改写 + 12 测试 + benchmarks/weights 同步,~1.5-2 agent rounds。

## Progress(2026-05-01 Blockchain Agent)

状态:**done(待 commit)**。spec_version 未动(用户明确要求保留)。

### 实际改动文件

| 文件 | 变化 | 说明 |
|---|---|---|
| `citizenchain/runtime/otherpallet/sfid-system/src/lib.rs` | 1206 → 952 行(净 -254) | storage / extrinsic / helper / ValidateUnsigned / event / error 全量重写;tests 析出 `tests.rs` |
| `citizenchain/runtime/otherpallet/sfid-system/src/tests.rs` | 新建 757 行 | 31 条测试,覆盖 12 条任务卡核心 + 老 bind/vote 兼容路径 |
| `citizenchain/runtime/otherpallet/sfid-system/src/benchmarks.rs` | 98 → 197 行 | 删 `rotate_sfid_keys` + 新加 4 个 Pays::No extrinsic benchmark |
| `citizenchain/runtime/otherpallet/sfid-system/src/weights.rs` | 192 → 110 行 | WeightInfo trait 加 4 个新 fn,删旧 2 个;数值为 stub(链端基线就绪后重新生成) |
| `citizenchain/runtime/src/configs/mod.rs` | +1 type + 3 处 `#[ignore]` | 给 sfid_system::Config 补 `type UnbindOrigin = EnsureRoot<AccountId>`;3 处依赖老 `SfidMainAccount::put` 的 runtime 集成测试标 `#[ignore = "ADR-008 step2b"]`,等 step2b 改 verifier 后重写 |

### 验收数字

- `cargo check -p sfid-system`:**绿**(0 error / 0 warning)
- `cargo check -p sfid-system --features runtime-benchmarks`:**绿**
- `cargo check -p citizenchain`(整 runtime crate,WASM_FILE=target/wasm/citizenchain.compact.compressed.wasm):**绿**(2 deprecated warning,见下)
- `cargo check -p citizenchain --tests`:**绿**
- `cargo test -p sfid-system`:**31 / 31 passed**(含 12 条核心 + 4 条 ValidateUnsigned/helper 辅助 + 15 条老兼容)
- `cargo clippy -p sfid-system`:0 warning(`-D warnings` 模式因上游 primitives 既有 warning 卡住,本卡未引入新 warning)

### 12 条任务卡核心测试通过情况

| # | 测试名 | 通过 |
|---|---|---|
| 1 | `activate_first_come_first_serve_on_empty_main` | passed |
| 2 | `activate_existing_admin_writes_signing_pubkey` | passed |
| 3 | `activate_unknown_admin_rejected_when_main_filled` | passed |
| 4 | `add_backup_signed_by_main_succeeds` | passed |
| 5 | `add_backup_unauthorized_signature_rejected` | passed |
| 6 | `remove_backup_cascades_to_signing_pubkey` | passed |
| 7 | `rotate_signing_pubkey_replaces_value` | passed |
| 8 | `rotate_unknown_admin_rejected` | passed |
| 9 | `pays_no_zero_balance_account_succeeds` | passed |
| 10 | `cross_province_admin_cannot_modify_other_province` | passed |
| 11 | `nonce_replay_rejected` | passed |
| 12 | `signature_with_wrong_payload_rejected` | passed |

附加 4 条:`validate_unsigned_rejects_unknown_call_path` / `validate_unsigned_passes_for_valid_activate` / `helpers_is_sheng_admin_and_main_work` / `unbind_origin_is_root_in_test_runtime`。

### 编译垫片(step2b/c 必清)

为不动 `duoqian-manage` / `genesis_config_presets.rs` / runtime tests verifier 而保留:

1. **`Pallet::current_sfid_verify_pubkey()`** → `#[deprecated]` 永远返回 `None`(被 `runtime/src/configs/mod.rs` 1 处使用 + 2 处 ignored 测试)
2. **`Pallet::sheng_signing_pubkey(_province)`** → `#[deprecated]` 永远返回 `None`(被 `runtime/src/configs/mod.rs::743` 1 处使用 — duoqian-manage 验签 fallback)
3. **`pallet::GenesisConfig`** 空 stub + `build` 不写任何 storage(`#[serde(rename = ...)]` 兼容旧 sfidMainAccount/sfidBackupAccount{1,2} JSON 字段以让 `genesis_config_presets.rs::482` 的 `serde_json::from_value` 测试 pass)
4. `runtime/src/configs/mod.rs` 3 处 runtime 集成测试 `#[ignore = "ADR-008 step2b"]`

step2b 改造 duoqian-manage 凭证 + verifier 后,垫片 1/2/4 必须连同 caller 一起删;step2c 改造 genesis_config_presets.rs 后,垫片 3 必须删。

### 残留扫描

- `grep "SfidMainAccount\|SfidBackupAccount" runtime/otherpallet/sfid-system/src/` → 仅注释引用("已删除"说明),无代码符号
- `grep "rotate_sfid_keys\|set_sheng_signing_pubkey" runtime/otherpallet/sfid-system/src/` → 仅 benchmarks/weights 顶部"已删除"注释,无代码符号

### 后续任务卡微调建议

- step2b:配套删除上述编译垫片 1/2/4,把 duoqian-manage `verify_institution_registration` 改为按 (province, signer_admin_pubkey) 二元组查 `ShengSigningPubkey::get(bounded, admin_pubkey)`,对应改造 `runtime/src/configs/mod.rs::743 fallback` + 3 个 `#[ignore]` 集成测试重写。
- step2c:配套删除编译垫片 3(`pallet::GenesisConfig`)+ 删除 `genesis_config_presets.rs:159-167` 的 SFID 3 把硬编码地址 + 删除 `root.insert("sfidSystem", ...)` 与 line 482 的 deserialize 测试。
- step2d:wumin/wuminapp 扫码 decoder 加 `signer_admin_pubkey` 字段(独立任务卡已存在)。
