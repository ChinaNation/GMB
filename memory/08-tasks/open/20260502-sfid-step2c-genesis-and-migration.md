# SFID Step 2c:genesis_config_presets 删 SFID 3 账户 + on_runtime_upgrade(spec_version 不升)

- 状态:open
- 创建日期:2026-05-02
- 模块:`citizenchain/runtime/genesis/` + `citizenchain/runtime/otherpallet/sfid-system/`
- 关联 ADR:`memory/04-decisions/ADR-008-sheng-admin-3tier.md`
- 前置依赖:step2a + step2b
- 阻塞下游:SFID phase7(链端就绪后切真)

## 任务需求

- `genesis_config_presets.rs` 删除 `sfid_main / backup_1 / backup_2` 硬编码 3 账户(链上 0 prior knowledge of SFID)
- ShengAdmins / ShengSigningPubkey 创世空,first-come-first-serve activation
- `pallet::on_runtime_upgrade`:开发期清空旧 storage(若有)
- **`spec_version` 不动**(本期裸升级开发期,等 chain 上线后再走 setCode)

## 影响范围

### genesis_config_presets.rs

```rust
// 删除(原 159-167 行附近):
// let sfid_main = AccountId::new(hex!("14e4f684..."));
// let sfid_backup_1 = AccountId::new(hex!("9084bbff..."));
// let sfid_backup_2 = AccountId::new(hex!("502a1021..."));
// genesis 中 SfidSystem 配置删除

// 新形态:不注入任何 SFID 账户;创世后由 SFID 后端走 first-come-first-serve activation
```

### sfid-system pallet on_runtime_upgrade

```rust
#[pallet::hooks]
impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    fn on_runtime_upgrade() -> Weight {
        // 中文注释:ADR-008 Step 2c。开发期清空旧 storage(若有数据)。
        // 由于开发期数据可丢,本 migration 不做版本号判断,每次升级都执行(idempotent)。
        // SfidMainAccount/Backup1/Backup2 已在 step2a 移除字段定义,这里只清残留实例。

        let mut weight = Weight::zero();
        // 旧 ShengSigningPubkey 单值表(若 storage 仍有 entry,移到 trash;但因字段已删,只能用 raw kill_storage)
        let _ = frame_support::storage::unhashed::clear_prefix(
            &Self::sheng_signing_pubkey_legacy_prefix(),
            None,
            None,
        );
        weight
    }
}
```

实际操作:开发期 chain 数据可丢,migration 内可只输出 log(`info!("ADR-008 Step 2c: legacy storage cleanup")`),配合 chain 重启从空 storage 走全新流程。

### genesis_config GenesisConfig 删除

`#[pallet::genesis_config]` + `BuildGenesisConfig::build()` 已在 step2a 删除。本卡只确认 chainspec 配置中 sfid-system section 被删干净。

### configs/mod.rs


### 文件级

- `citizenchain/runtime/src/genesis_config_presets.rs`(删 SFID 3 账户硬编码)
- `citizenchain/runtime/otherpallet/sfid-system/src/lib.rs`(加 on_runtime_upgrade)
- `citizenchain/runtime/src/lib.rs`:**spec_version 不动**(本卡明确)

## 主要风险点

- **chainspec 已固定**:`feedback_chainspec_frozen.md` 铁律。本期开发期裸升级符合 `feedback_chain_in_dev.md`,允许重启;**生产环境不允许此路径**
- **on_runtime_upgrade 字段已删但 storage 还有数据**:用 `frame_support::storage::unhashed::clear_prefix` 直接按原 prefix kill
- **genesis_build 已删但 chainspec.json 中可能仍有 sfid_system section**:本卡同步检查 chain spec/preset 文件

## 是否需要先沟通

- 否(开发期裸升级符合既定铁律)

## 验收清单

- `cargo check` + `cargo test` runtime 全绿
- `runtime/src/lib.rs` 中 `spec_version` 字段值未变化
- chain 启动(`./run.sh` 或 `cargo run`):创世无 SFID 3 账户,链空起步
- 集成测试:`SfidSystem::sheng_admins_iter()` 创世后空(等 first activation)
- 任务卡 progress 章节回写

## 不要做的事

- 不要升 `spec_version`(用户明确要求)
- 不要碰 sfid-system 业务逻辑(step2a 范围)
- 不要碰 duoqian-manage(step2b 范围)
- 不要碰 wumin / wuminapp(step2d)
- 不要 commit

## 工作量

~80 行 + 2 测试,~0.5 agent round。

## Progress

### 2026-05-02 完工(blockchain-agent / step2c)

**baseline**:commit fd5273b(step2b 完工)。spec_version=0 不动。

**实际改动**(共 5 文件):

1. `citizenchain/runtime/otherpallet/sfid-system/src/lib.rs`
   - 删 `#[pallet::genesis_config]` + `GenesisConfig<T>` struct(3 个 sfid_*_account Option 字段)
   - 删 `#[pallet::genesis_build]` + `BuildGenesisConfig for GenesisConfig<T>` 空 impl
   - 在 `#[pallet::hooks]` 内加 `on_runtime_upgrade()`(开发期 log only,fresh genesis 兜底)
   - 净 -25 行

2. `citizenchain/runtime/otherpallet/sfid-system/Cargo.toml`
   - 加 `log = { version = "0.4", default-features = false }` dep + `"log/std"` 到 std features
   - 净 +2 行

3. `citizenchain/runtime/src/genesis_config_presets.rs`
   - 删 3 把 `hex!("14e4f684...")` / `hex!("9084bbff...")` / `hex!("502a1021...")` 硬编码 AccountId
   - 删 `root.insert("sfidSystem", ...)` 整段
   - 删 `use hex_literal::hex;`(无其他用法)
   - 删两个 sfid 测试断言(`sfidMainAccount/sfidBackupAccount1/2` 字段 deserialize + `sfid_system::GenesisConfig` deserialize)
   - 净 -38 行

4. `citizenchain/runtime/issuance/citizen-issuance/tests/integration_bind_sfid.rs`
   - `new_test_ext` 删 `sfid_system::GenesisConfig::<Test>::{...}.assimilate_storage(...)` 块
   - mock `Config for Test` 补 `type UnbindOrigin = EnsureRoot<u64>;`(step2a 加的 trait item baseline 缺失修复)
   - 加 `use frame_system::{self as system, EnsureRoot};`
   - 净 -10 行

5. 任务卡 progress 章节(本次提交)

**chainspec.json**:`citizenchain/` 子树**无** `chainspec.json`(创世由 `genesis_config_presets.rs::genesis_config()` 运行时生成),`wuminapp/assets/chainspec.json` 不含 `sfidSystem` section,**无需手动改任何 chainspec 文件**。下次 `clean-run.sh` 走 fresh genesis 时新创世自动不含 SFID 3 账户(`feedback_chainspec_frozen.md` 铁律不影响本卡:本卡只改 Rust 代码,不动既有创世文件)。

**验收数字**:
- `cargo check -p sfid-system` 全绿
- `cargo check -p citizenchain`(WASM_FILE 设)全绿
- `cargo test -p sfid-system` **31/31 passed**
- `cargo test -p duoqian-manage` **26/26 passed**
- `cargo test -p citizenchain --tests`(WASM_FILE 设)**31/31 passed**(含 4 个 genesis_config_presets 测试)
- `cargo test -p citizen-issuance`(含集成测试 integration_bind_sfid)**全绿,7/7 集成测试 + 库测试通过**
- `cargo clippy -p sfid-system -p citizenchain` baseline 持平(citizenchain 10 个 stylistic warnings 跟本卡无关,均预先存在)

**残留 grep**:
- `GenesisConfig|genesis_config|BuildGenesisConfig` in `sfid-system/src/`:仅 2 行注释 + tests.rs 中 `frame_system::GenesisConfig`(必要)= **0 个 sfid-system 自定义 GenesisConfig 残留**
- `sfid_main_account|sfid_backup_account|SfidMainAccount|SfidBackupAccount` 在 runtime/:**0 个真实代码使用**(所有匹配项均在历史/解释性注释内)
- `0x14e4f684|0x9084bbff|0x502a1021` 等 SFID 3 把 hex 地址在 runtime/ = **0**
- `runtime/src/lib.rs::VERSION.spec_version = 0` **未变**(测试 `assert_eq!(VERSION.spec_version, 0)` 仍通过)

**chainspec.json 后续**:本卡不动既有创世文件;下次 fresh genesis(`feedback_chain_in_dev.md` 允许)由新代码自动生成不含 SFID 创世段的 chainspec,无需人工干预。

**后续任务卡**:
- step3(独立任务卡待开):`BindCredential` / `VoteCredential` / `PopSnapshotCredential` 加 `(province, signer_admin_pubkey)` 字段 + `runtime/src/configs/mod.rs` 内 `RuntimeSfidVerifier` / `RuntimeSfidVoteVerifier` / `RuntimePopulationSnapshotVerifier` 的 stub 接通真实双层匹配验签(当前都返回 false,占位待 step3)。
- step2d(`wumin/wuminapp` 扫码签名 decoder 同步加 signer_admin_pubkey):**未启动**,留给 mobile-agent。

**status**:done。Step 2 链端清理收尾完成,SFID phase7 链端 mock 切真前置条件齐备(待 step3 接通 verifier)。
