//! 中文注释:链上省管理员名册的共享类型。

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

/// 中文注释:省管理员 3-tier 槽位枚举。
/// - Main:首激活方占位;后续所有 backup 写入须由 Main 私钥签名。
/// - Backup1 / Backup2:由 Main 通过 `add_sheng_admin_backup` 动态添加。
#[derive(
    Clone,
    Copy,
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
    RuntimeDebug,
)]
pub enum Slot {
    Main,
    Backup1,
    Backup2,
}
