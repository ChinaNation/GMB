//! 共用数据类型(AssetMeta / AssetClass / MonitorReason 等)。
//!
//! 全部为零业务逻辑的裸结构,仅承载字段。业务逻辑在 execution / monitor / validation 中。

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

/// 资产种类:第一期只 Plain,Pegged 协议位预留。
///
/// 中文注释:Pegged 路径当前在 `validation::ensure_class_supported` reject,
/// 启用时改为接受 + 校验 PegDeclaration。
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
    /// 有锚定声明(法币 / 资产凭证),当前 reject。
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
/// 中文注释：storage key 是 `asset_id`，发行/治理账户是机构多签账户地址。
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
    /// 发行机构多签账户地址。
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
