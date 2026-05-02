//! SFID 机构备案链上类型。

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::{pallet_prelude::ConstU32, BoundedVec};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

/// 中文注释:机构 SFID 号上限。这里按字符串字节存储,不解析 SFID 各段业务含义。
pub type FilingSfidId = BoundedVec<u8, ConstU32<128>>;
/// 中文注释:机构名称上限。照片、章程等 SFID 内部资料不进入链上备案。
pub type FilingInstitutionName = BoundedVec<u8, ConstU32<128>>;
/// 中文注释:机构账户名称上限,例如"主账户"、"费用账户"或其他命名账户。
pub type FilingAccountName = BoundedVec<u8, ConstU32<64>>;

#[derive(
    Clone,
    Encode,
    Decode,
    DecodeWithMemTracking,
    Eq,
    PartialEq,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
pub struct InstitutionFilingPayload {
    pub sfid_id: FilingSfidId,
    pub institution_name: FilingInstitutionName,
    pub account_name: FilingAccountName,
}

#[derive(
    Clone,
    Encode,
    Decode,
    DecodeWithMemTracking,
    Eq,
    PartialEq,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
pub struct InstitutionFilingRecord<BlockNumber> {
    pub sfid_id: FilingSfidId,
    pub institution_name: FilingInstitutionName,
    pub account_name: FilingAccountName,
    pub filed_at_block: BlockNumber,
}
