# SFID Step 2c:genesis_config_presets 删 SFID 3 账户 + on_runtime_upgrade(spec_version 不升)

- 状态:open
- 创建日期:2026-05-02
- 模块:`citizenchain/runtime/genesis/` + `citizenchain/runtime/otherpallet/sfid-system/`
- 关联 ADR:`memory/04-decisions/ADR-008-sheng-admin-3tier-and-key-admin-removal.md`
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

如果 `runtime/src/configs/mod.rs` 有 `pallet_sfid_system::Config` 注入项需要清理 KEY_ADMIN 相关 const,本卡处理。

### 文件级

- `citizenchain/runtime/src/genesis_config_presets.rs`(删 SFID 3 账户硬编码)
- `citizenchain/runtime/otherpallet/sfid-system/src/lib.rs`(加 on_runtime_upgrade)
- `citizenchain/runtime/src/configs/mod.rs`(若需,删 KEY_ADMIN 相关 const)
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
