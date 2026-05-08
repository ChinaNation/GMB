//! 共用数据类型(AssetMeta / AssetClass / MonitorReason 等)。
//!
//! 全部为零业务逻辑的裸结构,仅承载字段。业务逻辑在 execution / monitor / validation 中。

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

/// 资产种类:第一期只 Plain,Pegged 协议位预留。
///
/// 中文注释:Pegged 路径在 `validation::ensure_class_supported` reject,
/// Phase 2 启用时把 reject 改为接受 + 校验 PegDeclaration。
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    Copy,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
)]
pub enum AssetClass {
    /// 无锚定声明(第一期唯一允许值)。
    Plain,
    /// 有锚定声明(法币 / 资产凭证),Phase 2 启用,当前 reject。
    Pegged,
}

/// 资产生命周期状态。
///
/// 中文注释:Active 是默认态;Closed 由发行方 `propose_close` 终态化;
/// ForceClosed 由 NRC 监管 `monitor_force_close` 进入,30 天后自动销毁余额。
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    Copy,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
)]
pub enum AssetState {
    Active,
    Closed,
    ForceClosed { close_block: u32 },
}

/// 用户代币元数据(链端权威 storage)。
///
/// 中文注释:每条记录对应一个 SubjectId(0x04)。
/// ADR-011 v2 修订:
/// - **去掉 `asset_id` 字段**:asset_id 已编入 SubjectId byte[1..5],storage key 即可反推
/// - **去掉 `monitor_subject_id` 字段**:NRC monitor 全局强制,走 NrcMainAccountProvider
///   trait 解析,每条资产存 48B NRC SubjectId 是冗余
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
)]
#[scale_info(skip_type_params(MaxName, MaxSymbol, MaxDescription))]
pub struct OnchainAssetMeta<AccountId> {
    /// 发行人主体 SubjectId(0x02 SfidInstitution 或 0x03 PersonalDuoqian)。
    pub issuer_subject_id: [u8; 48],
    /// 发行人代理账户(用于 mint 受益人 / 关闭余额清算等内核操作)。
    pub issuer_account: AccountId,
    /// 资产种类(第一期只 Plain)。
    pub class: AssetClass,
    /// 小数位(0..=18,链端校验)。
    pub decimals: u8,
    /// 资产生命周期状态。
    pub state: AssetState,
}

/// NRC 监管动作的 reason hash(链下文书 sha256)。
///
/// 中文注释:链端只存 hash 不存文书原文,文书原文走链下司法/听证流程。
pub type MonitorReasonHash = [u8; 32];
