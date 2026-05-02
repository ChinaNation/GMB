//! 中文注释:省管理员历史 storage 迁移提示。
//!
//! 当前开发链采用 fresh-genesis 路径,老的 KEY_ADMIN / 单签名公钥 storage 已删除。
//! 保留独立函数是为了让 `lib.rs` 的 hook 只负责挂载,具体提示文本归本目录维护。

use frame_support::weights::Weight;

pub fn log_legacy_storage_cleanup() -> Weight {
    log::info!(
        "ADR-008 Step 2c on_runtime_upgrade: legacy SFID storage (SfidMainAccount/Backup{{1,2}}, single-value ShengSigningPubkey, ProvinceBySigningPubkey, GenesisConfig) removed; fresh-genesis path expected"
    );
    Weight::zero()
}
