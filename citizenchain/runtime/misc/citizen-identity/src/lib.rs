//! # 链上公民身份模块 (citizen-identity)
//!
//! 本模块是公民链上身份唯一真源。OnChina 只能作为注册局操作入口提交交易,
//! 投票引擎只能读取本模块的投票身份、参选身份和人口快照。

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub use pallet::*;
pub mod weights;

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::ConstU32;
use frame_support::BoundedVec;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

pub type CidNumberBound = BoundedVec<u8, ConstU32<32>>;
pub type AreaCodeBound = BoundedVec<u8, ConstU32<16>>;
pub type RoleCodeBound = BoundedVec<u8, ConstU32<64>>;
/// 公民姓、名各自的最大字节数；与管理员人员姓名字段保持一致。
pub const PERSON_NAME_MAX_BYTES: u32 = 128;
/// 姓。结构本身已经限定公民语义，字段和类型都不再重复增加 `citizen_` 前缀。
pub type FamilyName = BoundedVec<u8, ConstU32<PERSON_NAME_MAX_BYTES>>;
/// 名。与 `family_name` 分开保存，不生成或存储合并姓名。
pub type GivenName = BoundedVec<u8, ConstU32<PERSON_NAME_MAX_BYTES>>;
pub const MIN_ONCHAIN_CITIZEN_AGE_YEARS: u8 = 16;
/// 批量占号单笔上限。
pub const MAX_CID_OCCUPY_BATCH: u32 = 10_000;

/// CID 占号登记状态:吊销走墓碑,存储项永不删除、号码永不复用。
#[derive(
    Clone,
    Copy,
    Encode,
    Decode,
    DecodeWithMemTracking,
    Eq,
    PartialEq,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
#[repr(u8)]
pub enum CidRecordStatus {
    Active = 0,
    Revoked = 1,
}

/// CID 占号登记记录:链上写入时原子查重的唯一仲裁真源。
///
/// 只含号码归属与承诺哈希,不含姓名生日等隐私;居住地码用于吊销时的
/// 注册局作用域授权;承诺哈希用于建档落库失败后的幂等续用识别。
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
pub struct CidRecord<BlockNumber> {
    /// 执行登记的注册局机构 CID；管理员钱包只存在于外层签名 origin。
    pub registrar_cid_number: CidNumberBound,
    pub commitment: [u8; 32],
    pub residence_province_code: AreaCodeBound,
    pub residence_city_code: AreaCodeBound,
    pub status: CidRecordStatus,
    pub registered_at: BlockNumber,
    pub revoked_at: Option<BlockNumber>,
}

/// 批量占号单项。
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
pub struct CidOccupyItem {
    pub cid_number: CidNumberBound,
    pub commitment: [u8; 32],
}

pub type CidOccupyItemsBound = BoundedVec<CidOccupyItem, ConstU32<MAX_CID_OCCUPY_BATCH>>;

/// days since 1970-01-01 → 公历 (年, 月, 日)。
///
/// Howard Hinnant civil-from-days 整数算法,与 chrono 等价;no_std 下自带,
/// 供护照有效期(YYYYMMDD)与链上时间戳比对。
pub fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = if month <= 2 { year + 1 } else { year };
    (year, month, day)
}

#[derive(
    Clone,
    Copy,
    Encode,
    Decode,
    DecodeWithMemTracking,
    Eq,
    PartialEq,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
#[repr(u8)]
pub enum CitizenStatus {
    Normal = 0,
    Revoked = 1,
}

impl Default for CitizenStatus {
    fn default() -> Self {
        CitizenStatus::Normal
    }
}

/// 公民性别(参选身份公开档案字段)。
#[derive(
    Clone,
    Copy,
    Encode,
    Decode,
    DecodeWithMemTracking,
    Eq,
    PartialEq,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
#[repr(u8)]
pub enum CitizenSex {
    Male = 0,
    Female = 1,
}

#[derive(
    Clone,
    Copy,
    Encode,
    Decode,
    DecodeWithMemTracking,
    Eq,
    PartialEq,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
#[repr(u8)]
pub enum CitizenIdentityLevel {
    Voting = 1,
    Candidate = 2,
}

/// 公民授权主体。
///
/// 公民 CID 与钱包账户必须同时匹配 `citizen-identity` 的有效双向绑定：CID 证明
/// 公民身份和权益，钱包账户证明本次操作的签名身份，任何一项都不能单独授权。
/// 本结构只在读取时构造，不作为新的身份或权限 Storage。
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
pub struct CitizenSubject<AccountId> {
    /// 公民 CID 号；由本模块保存的有效身份提供，消费方不得自行生成或修改。
    pub cid_number: CidNumberBound,
    /// 公民钱包账户；用于验证签名，并与 `cid_number` 共同确认公民主体。
    pub wallet_account: AccountId,
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
pub struct VotingIdentityPayload<AccountId> {
    pub cid_number: CidNumberBound,
    pub wallet_account: AccountId,
    pub citizen_age_years: u8,
    pub passport_valid_from: u32,
    pub passport_valid_until: u32,
    pub citizen_status: CitizenStatus,
    pub residence_province_code: AreaCodeBound,
    pub residence_city_code: AreaCodeBound,
    pub residence_town_code: AreaCodeBound,
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
pub struct CandidateIdentityPayload<AccountId> {
    /// 公民的完整投票身份载荷；竞选身份必须建立在有效投票身份之上。
    pub voting: VotingIdentityPayload<AccountId>,
    /// 出生省级行政区代码；表示出生地，不表示当前居住地。
    pub birth_province_code: AreaCodeBound,
    /// 出生市级行政区代码；表示出生地，不表示当前居住地。
    pub birth_city_code: AreaCodeBound,
    /// 出生镇级行政区代码；表示出生地，不表示当前居住地。
    pub birth_town_code: AreaCodeBound,
    /// 姓；直接使用公民身份真源中的 `family_name`，不生成合并姓名。
    pub family_name: FamilyName,
    /// 名；直接使用公民身份真源中的 `given_name`，不生成合并姓名。
    pub given_name: GivenName,
    /// 公民性别；用于竞选资格校验和竞选信息展示。
    pub citizen_sex: CitizenSex,
    /// 出生日期(YYYYMMDD 整数)。仅竞选身份携带,写入后不可修改;
    /// 链上凭此实时计算竞选公民年龄(见 `candidate_age`)。
    pub birth_date: u32,
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
pub struct VotingIdentity<BlockNumber> {
    pub passport_valid_from: u32,
    pub passport_valid_until: u32,
    pub citizen_status: CitizenStatus,
    pub residence_province_code: AreaCodeBound,
    pub residence_city_code: AreaCodeBound,
    pub residence_town_code: AreaCodeBound,
    pub updated_at: BlockNumber,
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
pub struct CandidateIdentity<BlockNumber> {
    /// 生成该竞选身份时采用的出生省级行政区代码。
    pub birth_province_code: AreaCodeBound,
    /// 生成该竞选身份时采用的出生市级行政区代码。
    pub birth_city_code: AreaCodeBound,
    /// 生成该竞选身份时采用的出生镇级行政区代码。
    pub birth_town_code: AreaCodeBound,
    /// 生成该竞选身份时采用的姓。
    pub family_name: FamilyName,
    /// 生成该竞选身份时采用的名。
    pub given_name: GivenName,
    /// 生成该竞选身份时采用的公民性别。
    pub citizen_sex: CitizenSex,
    /// 出生日期(YYYYMMDD 整数),写一次即锁定,后续更新不得变更。
    pub birth_date: u32,
    /// 最近一次写入或更新该竞选身份的区块号；不代表现实世界时间。
    pub updated_at: BlockNumber,
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
pub enum PopulationScope {
    Country,
    Province(AreaCodeBound),
    City(AreaCodeBound, AreaCodeBound),
    Town(AreaCodeBound, AreaCodeBound, AreaCodeBound),
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
pub struct PopulationData {
    pub scope: PopulationScope,
    pub eligible_total: u64,
    /// 读取人口数据时已经提交的最后一个身份资格版本。
    pub eligibility_revision: u64,
    /// 读取人口数据时的 UTC+8 日期，投票引擎据此冻结护照判定日期。
    /// 该人口数据用于资格历史判定的 UTC+8 日期；本字段不是身份模块快照标识。
    pub eligibility_date: u32,
}

/// 单个账户的一段不可变投票资格历史。
///
/// 全局 revision 区分同一区块内的多次身份写入；`valid_until_revision` 为开区间上界。
/// 公投按 snapshot revision 二分定位版本，不依赖投票时的当前身份。
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
pub struct VotingEligibilityVersion<BlockNumber> {
    pub identity: VotingIdentity<BlockNumber>,
    pub valid_from_revision: u64,
    pub valid_until_revision: Option<u64>,
}

pub trait CitizenIdentityAuthority<AccountId, Signature> {
    fn can_manage_voting_identity(
        registrar: &AccountId,
        actor_cid_number: &[u8],
        actor_role_code: &[u8],
        residence_province_code: &[u8],
        residence_city_code: &[u8],
        level: CitizenIdentityLevel,
        action_code: u32,
    ) -> bool;

    fn verify_citizen_signature(
        wallet_account: &AccountId,
        payload: &[u8],
        signature: &Signature,
    ) -> bool;
}

impl<AccountId, Signature> CitizenIdentityAuthority<AccountId, Signature> for () {
    fn can_manage_voting_identity(
        _registrar: &AccountId,
        _actor_cid_number: &[u8],
        _actor_role_code: &[u8],
        _residence_province_code: &[u8],
        _residence_city_code: &[u8],
        _level: CitizenIdentityLevel,
        _action_code: u32,
    ) -> bool {
        false
    }

    fn verify_citizen_signature(
        _wallet_account: &AccountId,
        _payload: &[u8],
        _signature: &Signature,
    ) -> bool {
        false
    }
}

pub trait OnVotingIdentityRegistered<AccountId> {
    fn on_voting_identity_registered(_who: &AccountId, _cid_number: &[u8]) {}
}

impl<AccountId> OnVotingIdentityRegistered<AccountId> for () {}

pub trait OnVotingIdentityRegisteredWeight {
    fn on_voting_identity_registered_weight() -> frame_support::weights::Weight {
        frame_support::weights::Weight::zero()
    }
}

impl OnVotingIdentityRegisteredWeight for () {}

pub trait CitizenIdentityProvider<AccountId> {
    /// 读取经过 CID 状态和 CID↔钱包双向绑定校验的完整公民主体。
    fn citizen_subject(who: &AccountId) -> Option<CitizenSubject<AccountId>>;
    fn can_vote(who: &AccountId, scope: &PopulationScope) -> bool;
    fn can_be_candidate(who: &AccountId, scope: &PopulationScope) -> bool;
    fn population_count(scope: &PopulationScope) -> u64;
    fn population_data(scope: &PopulationScope) -> PopulationData;
    fn can_vote_at(who: &AccountId, population_data: &PopulationData) -> bool;
}

impl<AccountId> CitizenIdentityProvider<AccountId> for () {
    fn citizen_subject(_who: &AccountId) -> Option<CitizenSubject<AccountId>> {
        None
    }

    fn can_vote(_who: &AccountId, _scope: &PopulationScope) -> bool {
        false
    }

    fn can_be_candidate(_who: &AccountId, _scope: &PopulationScope) -> bool {
        false
    }

    fn population_count(_scope: &PopulationScope) -> u64 {
        0
    }

    fn population_data(scope: &PopulationScope) -> PopulationData {
        PopulationData {
            scope: scope.clone(),
            eligible_total: 0,
            eligibility_revision: 0,
            eligibility_date: 0,
        }
    }

    fn can_vote_at(_who: &AccountId, _population_data: &PopulationData) -> bool {
        false
    }
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;
    use frame_support::{pallet_prelude::*, Blake2_128Concat};
    use frame_system::pallet_prelude::*;

    /// 创世链直接采用当前存储结构，不保留历史迁移或兼容分支。
    pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

    pub type SignatureOf<T> = BoundedVec<u8, <T as Config>::MaxCitizenSignatureLength>;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        #[pallet::constant]
        type MaxCitizenSignatureLength: Get<u32>;

        type CitizenIdentityAuthority: CitizenIdentityAuthority<Self::AccountId, SignatureOf<Self>>;

        type OnVotingIdentityRegistered: OnVotingIdentityRegistered<Self::AccountId>
            + OnVotingIdentityRegisteredWeight;

        /// 链上时间源(pallet-timestamp),用于投票时校验护照有效期窗口。
        type TimeProvider: frame_support::traits::UnixTime;

        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    /// 永久公民 CID 到投票身份。CID 是身份主键，钱包不参与身份寻址。
    pub type VotingIdentityByCid<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        CidNumberBound,
        VotingIdentity<BlockNumberFor<T>>,
        OptionQuery,
    >;

    /// 永久公民 CID 到竞选身份。更换签名钱包不会搬迁竞选资料。
    #[pallet::storage]
    pub type CandidateIdentityByCid<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        CidNumberBound,
        CandidateIdentity<BlockNumberFor<T>>,
        OptionQuery,
    >;

    /// 永久公民 CID 当前绑定的唯一签名钱包。
    #[pallet::storage]
    pub type WalletAccountByCid<T: Config> =
        StorageMap<_, Blake2_128Concat, CidNumberBound, T::AccountId, OptionQuery>;

    /// 当前签名钱包反向绑定的永久公民 CID；与 `WalletAccountByCid` 必须闭环。
    #[pallet::storage]
    pub type CidByWalletAccount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, CidNumberBound, OptionQuery>;

    /// CID 占号登记表:发号全局唯一的链上真源(占号先行,墓碑不删除)。
    #[pallet::storage]
    pub type CidRegistry<T: Config> =
        StorageMap<_, Blake2_128Concat, CidNumberBound, CidRecord<BlockNumberFor<T>>, OptionQuery>;

    #[pallet::storage]
    pub type CountryVotingCount<T> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    pub type ProvinceVotingCount<T: Config> =
        StorageMap<_, Blake2_128Concat, AreaCodeBound, u64, ValueQuery>;

    #[pallet::storage]
    pub type CityVotingCount<T: Config> =
        StorageMap<_, Blake2_128Concat, (AreaCodeBound, AreaCodeBound), u64, ValueQuery>;

    #[pallet::storage]
    pub type TownVotingCount<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        (AreaCodeBound, AreaCodeBound, AreaCodeBound),
        u64,
        ValueQuery,
    >;

    /// 全局身份资格修订号。每次投票身份写入严格递增，用于冻结同区块交易顺序。
    #[pallet::storage]
    pub type NextEligibilityRevision<T> = StorageValue<_, u64, ValueQuery>;

    /// 单个永久 CID 的历史版本数量；版本索引为 0..count，支持按 revision 有界二分。
    #[pallet::storage]
    pub type VotingEligibilityVersionCount<T: Config> =
        StorageMap<_, Blake2_128Concat, CidNumberBound, u64, ValueQuery>;

    /// 永久 CID 的不可变投票资格历史：(CID, 版本序号) → 资格区间。
    #[pallet::storage]
    pub type VotingEligibilityVersions<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        CidNumberBound,
        Blake2_128Concat,
        u64,
        VotingEligibilityVersion<BlockNumberFor<T>>,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        VotingIdentityRegistered {
            wallet_account: T::AccountId,
            cid_number: CidNumberBound,
        },
        VotingIdentityUpdated {
            wallet_account: T::AccountId,
            cid_number: CidNumberBound,
        },
        CandidateIdentityUpgraded {
            wallet_account: T::AccountId,
            cid_number: CidNumberBound,
        },
        CandidateIdentityUpdated {
            wallet_account: T::AccountId,
            cid_number: CidNumberBound,
        },
        CitizenIdentityRevoked {
            wallet_account: T::AccountId,
            cid_number: CidNumberBound,
        },
        CidOccupied {
            cid_number: CidNumberBound,
            registrar_cid_number: CidNumberBound,
        },
        CidRevoked {
            cid_number: CidNumberBound,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        EmptyCidNumber,
        EmptyResidenceScope,
        EmptyBirthScope,
        EmptyFamilyName,
        EmptyGivenName,
        /// 出生日期非法(非 YYYYMMDD 或无法计算年龄)。
        InvalidBirthDate,
        /// 出生日期写入后不可修改,更新竞选身份时不得变更。
        BirthDateImmutable,
        InvalidDateRange,
        InvalidCitizenCode,
        UnauthorizedRegistrar,
        InvalidCitizenSignature,
        UnderVotingAge,
        /// 该永久 CID 已经建立投票身份；登记入口不得兼作更新入口。
        VotingIdentityAlreadyExists,
        /// CID 与入参钱包不符合当前双向绑定。
        CidWalletBindingMismatch,
        /// 入参钱包已经绑定另一个永久 CID。
        WalletAlreadyBoundToAnotherCid,
        CidNotFound,
        VotingIdentityNotFound,
        CidAlreadyOccupied,
        CidNotOccupied,
        CidAlreadyRevoked,
        /// 身份资格修订号达到 u64 上限。
        EligibilityRevisionOverflow,
        /// 单个永久 CID 的身份历史版本数达到 u64 上限。
        EligibilityVersionOverflow,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(
            T::WeightInfo::register_voting_identity()
                .saturating_add(T::OnVotingIdentityRegistered::on_voting_identity_registered_weight())
        )]
        pub fn register_voting_identity(
            origin: OriginFor<T>,
            actor_cid_number: CidNumberBound,
            actor_role_code: RoleCodeBound,
            payload: VotingIdentityPayload<T::AccountId>,
            citizen_signature: SignatureOf<T>,
        ) -> DispatchResult {
            let registrar = ensure_signed(origin)?;
            Self::ensure_valid_voting_payload(&payload)?;
            Self::ensure_authorized(
                &registrar,
                actor_cid_number.as_slice(),
                actor_role_code.as_slice(),
                &payload,
                CitizenIdentityLevel::Voting,
                0,
            )?;
            Self::ensure_citizen_signature(
                &payload.wallet_account,
                &payload.encode(),
                &citizen_signature,
            )?;
            Self::ensure_cid_occupied_active(&payload.cid_number)?;
            ensure!(
                !VotingIdentityByCid::<T>::contains_key(&payload.cid_number),
                Error::<T>::VotingIdentityAlreadyExists
            );
            Self::ensure_wallet_binding_available(&payload.cid_number, &payload.wallet_account)?;

            let identity = Self::identity_from_payload(&payload);
            Self::replace_voting_identity(payload.cid_number.clone(), identity, None)?;
            Self::bind_wallet_account(&payload.cid_number, &payload.wallet_account);
            T::OnVotingIdentityRegistered::on_voting_identity_registered(
                &payload.wallet_account,
                payload.cid_number.as_slice(),
            );
            Self::deposit_event(Event::<T>::VotingIdentityRegistered {
                wallet_account: payload.wallet_account,
                cid_number: payload.cid_number,
            });
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::upgrade_to_candidate_identity())]
        pub fn upgrade_to_candidate_identity(
            origin: OriginFor<T>,
            actor_cid_number: CidNumberBound,
            actor_role_code: RoleCodeBound,
            payload: CandidateIdentityPayload<T::AccountId>,
            citizen_signature: SignatureOf<T>,
        ) -> DispatchResult {
            let registrar = ensure_signed(origin)?;
            Self::ensure_valid_candidate_payload(&payload)?;
            Self::ensure_authorized(
                &registrar,
                actor_cid_number.as_slice(),
                actor_role_code.as_slice(),
                &payload.voting,
                CitizenIdentityLevel::Candidate,
                1,
            )?;
            Self::ensure_citizen_signature(
                &payload.voting.wallet_account,
                &payload.encode(),
                &citizen_signature,
            )?;
            Self::ensure_cid_occupied_active(&payload.voting.cid_number)?;
            Self::ensure_wallet_binding_available(
                &payload.voting.cid_number,
                &payload.voting.wallet_account,
            )?;

            Self::ensure_birth_date_immutable(&payload.voting.cid_number, payload.birth_date)?;

            let old = VotingIdentityByCid::<T>::get(&payload.voting.cid_number);
            let identity = Self::identity_from_payload(&payload.voting);
            Self::replace_voting_identity(payload.voting.cid_number.clone(), identity, old)?;
            Self::bind_wallet_account(&payload.voting.cid_number, &payload.voting.wallet_account);
            CandidateIdentityByCid::<T>::insert(
                &payload.voting.cid_number,
                CandidateIdentity {
                    birth_province_code: payload.birth_province_code,
                    birth_city_code: payload.birth_city_code,
                    birth_town_code: payload.birth_town_code,
                    family_name: payload.family_name,
                    given_name: payload.given_name,
                    citizen_sex: payload.citizen_sex,
                    birth_date: payload.birth_date,
                    updated_at: frame_system::Pallet::<T>::block_number(),
                },
            );
            Self::deposit_event(Event::<T>::CandidateIdentityUpgraded {
                wallet_account: payload.voting.wallet_account,
                cid_number: payload.voting.cid_number,
            });
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::update_voting_identity())]
        pub fn update_voting_identity(
            origin: OriginFor<T>,
            actor_cid_number: CidNumberBound,
            actor_role_code: RoleCodeBound,
            payload: VotingIdentityPayload<T::AccountId>,
            citizen_signature: SignatureOf<T>,
        ) -> DispatchResult {
            let registrar = ensure_signed(origin)?;
            Self::ensure_valid_voting_payload(&payload)?;
            Self::ensure_authorized(
                &registrar,
                actor_cid_number.as_slice(),
                actor_role_code.as_slice(),
                &payload,
                CitizenIdentityLevel::Voting,
                2,
            )?;
            Self::ensure_citizen_signature(
                &payload.wallet_account,
                &payload.encode(),
                &citizen_signature,
            )?;
            Self::ensure_cid_occupied_active(&payload.cid_number)?;
            Self::ensure_current_wallet_binding(&payload.cid_number, &payload.wallet_account)?;

            let old = VotingIdentityByCid::<T>::get(&payload.cid_number)
                .ok_or(Error::<T>::VotingIdentityNotFound)?;
            let identity = Self::identity_from_payload(&payload);
            Self::replace_voting_identity(payload.cid_number.clone(), identity, Some(old))?;
            Self::deposit_event(Event::<T>::VotingIdentityUpdated {
                wallet_account: payload.wallet_account,
                cid_number: payload.cid_number,
            });
            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::update_candidate_identity())]
        pub fn update_candidate_identity(
            origin: OriginFor<T>,
            actor_cid_number: CidNumberBound,
            actor_role_code: RoleCodeBound,
            payload: CandidateIdentityPayload<T::AccountId>,
            citizen_signature: SignatureOf<T>,
        ) -> DispatchResult {
            let registrar = ensure_signed(origin)?;
            Self::ensure_valid_candidate_payload(&payload)?;
            Self::ensure_authorized(
                &registrar,
                actor_cid_number.as_slice(),
                actor_role_code.as_slice(),
                &payload.voting,
                CitizenIdentityLevel::Candidate,
                3,
            )?;
            Self::ensure_citizen_signature(
                &payload.voting.wallet_account,
                &payload.encode(),
                &citizen_signature,
            )?;
            Self::ensure_cid_occupied_active(&payload.voting.cid_number)?;
            Self::ensure_current_wallet_binding(
                &payload.voting.cid_number,
                &payload.voting.wallet_account,
            )?;

            Self::ensure_birth_date_immutable(&payload.voting.cid_number, payload.birth_date)?;

            let old = VotingIdentityByCid::<T>::get(&payload.voting.cid_number)
                .ok_or(Error::<T>::VotingIdentityNotFound)?;
            let identity = Self::identity_from_payload(&payload.voting);
            Self::replace_voting_identity(payload.voting.cid_number.clone(), identity, Some(old))?;
            CandidateIdentityByCid::<T>::insert(
                &payload.voting.cid_number,
                CandidateIdentity {
                    birth_province_code: payload.birth_province_code,
                    birth_city_code: payload.birth_city_code,
                    birth_town_code: payload.birth_town_code,
                    family_name: payload.family_name,
                    given_name: payload.given_name,
                    citizen_sex: payload.citizen_sex,
                    birth_date: payload.birth_date,
                    updated_at: frame_system::Pallet::<T>::block_number(),
                },
            );
            Self::deposit_event(Event::<T>::CandidateIdentityUpdated {
                wallet_account: payload.voting.wallet_account,
                cid_number: payload.voting.cid_number,
            });
            Ok(())
        }

        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::revoke_identity())]
        pub fn revoke_identity(
            origin: OriginFor<T>,
            actor_cid_number: CidNumberBound,
            actor_role_code: RoleCodeBound,
            cid_number: CidNumberBound,
        ) -> DispatchResult {
            let registrar = ensure_signed(origin)?;
            ensure!(!cid_number.is_empty(), Error::<T>::EmptyCidNumber);
            let account =
                WalletAccountByCid::<T>::get(&cid_number).ok_or(Error::<T>::CidNotFound)?;
            let old = VotingIdentityByCid::<T>::get(&cid_number)
                .ok_or(Error::<T>::VotingIdentityNotFound)?;
            Self::ensure_authorized(
                &registrar,
                actor_cid_number.as_slice(),
                actor_role_code.as_slice(),
                &VotingIdentityPayload {
                    cid_number: cid_number.clone(),
                    wallet_account: account.clone(),
                    citizen_age_years: MIN_ONCHAIN_CITIZEN_AGE_YEARS,
                    passport_valid_from: old.passport_valid_from,
                    passport_valid_until: old.passport_valid_until,
                    citizen_status: old.citizen_status,
                    residence_province_code: old.residence_province_code.clone(),
                    residence_city_code: old.residence_city_code.clone(),
                    residence_town_code: old.residence_town_code.clone(),
                },
                CitizenIdentityLevel::Voting,
                4,
            )?;

            let mut revoked = old.clone();
            revoked.citizen_status = CitizenStatus::Revoked;
            revoked.updated_at = frame_system::Pallet::<T>::block_number();
            Self::replace_voting_identity(cid_number.clone(), revoked, Some(old))?;
            CandidateIdentityByCid::<T>::remove(&cid_number);
            // 身份吊销联动登记表墓碑,保证发号真源与身份状态一致。
            Self::tombstone_cid_record(&cid_number);
            Self::deposit_event(Event::<T>::CitizenIdentityRevoked {
                wallet_account: account,
                cid_number,
            });
            Ok(())
        }

        // call_index(5) 已永久废弃：人口快照只能由 votingengine 的内部 provider 调用生成。

        /// 占号:公民建档先行登记 CID 号,链上原子「验格式+查重+登记」是
        /// 全局唯一的唯一仲裁;成功后注册局才落本地档案。
        #[pallet::call_index(6)]
        #[pallet::weight(<T as Config>::WeightInfo::occupy_cid())]
        pub fn occupy_cid(
            origin: OriginFor<T>,
            actor_cid_number: CidNumberBound,
            actor_role_code: RoleCodeBound,
            cid_number: CidNumberBound,
            commitment: [u8; 32],
            residence_province_code: AreaCodeBound,
            residence_city_code: AreaCodeBound,
        ) -> DispatchResult {
            let registrar = ensure_signed(origin)?;
            ensure!(
                !residence_province_code.is_empty() && !residence_city_code.is_empty(),
                Error::<T>::EmptyResidenceScope
            );
            ensure!(
                T::CitizenIdentityAuthority::can_manage_voting_identity(
                    &registrar,
                    actor_cid_number.as_slice(),
                    actor_role_code.as_slice(),
                    residence_province_code.as_slice(),
                    residence_city_code.as_slice(),
                    CitizenIdentityLevel::Voting,
                    6,
                ),
                Error::<T>::UnauthorizedRegistrar
            );
            Self::do_occupy_cid(
                &actor_cid_number,
                &cid_number,
                &commitment,
                &residence_province_code,
                &residence_city_code,
            )
        }

        /// 批量占号:同一注册局同一作用域一次占 N 号(批量建档摊薄冷签);
        /// 任一项失败整笔回滚。
        #[pallet::call_index(7)]
        #[pallet::weight(<T as Config>::WeightInfo::occupy_cids_batch(items.len() as u32))]
        pub fn occupy_cids_batch(
            origin: OriginFor<T>,
            actor_cid_number: CidNumberBound,
            actor_role_code: RoleCodeBound,
            items: CidOccupyItemsBound,
            residence_province_code: AreaCodeBound,
            residence_city_code: AreaCodeBound,
        ) -> DispatchResult {
            let registrar = ensure_signed(origin)?;
            ensure!(!items.is_empty(), Error::<T>::EmptyCidNumber);
            ensure!(
                !residence_province_code.is_empty() && !residence_city_code.is_empty(),
                Error::<T>::EmptyResidenceScope
            );
            ensure!(
                T::CitizenIdentityAuthority::can_manage_voting_identity(
                    &registrar,
                    actor_cid_number.as_slice(),
                    actor_role_code.as_slice(),
                    residence_province_code.as_slice(),
                    residence_city_code.as_slice(),
                    CitizenIdentityLevel::Voting,
                    7,
                ),
                Error::<T>::UnauthorizedRegistrar
            );
            for item in items.iter() {
                Self::do_occupy_cid(
                    &actor_cid_number,
                    &item.cid_number,
                    &item.commitment,
                    &residence_province_code,
                    &residence_city_code,
                )?;
            }
            Ok(())
        }

        /// 吊销:登记表墓碑(Active→Revoked,永不复用);号已绑定链上身份
        /// 则联动置 Revoked。作用域授权用占号时登记的居住地,防跨域吊销。
        #[pallet::call_index(8)]
        #[pallet::weight(<T as Config>::WeightInfo::revoke_cid())]
        pub fn revoke_cid(
            origin: OriginFor<T>,
            actor_cid_number: CidNumberBound,
            actor_role_code: RoleCodeBound,
            cid_number: CidNumberBound,
        ) -> DispatchResult {
            let registrar = ensure_signed(origin)?;
            let rec = CidRegistry::<T>::get(&cid_number).ok_or(Error::<T>::CidNotOccupied)?;
            ensure!(
                rec.status == CidRecordStatus::Active,
                Error::<T>::CidAlreadyRevoked
            );
            ensure!(
                T::CitizenIdentityAuthority::can_manage_voting_identity(
                    &registrar,
                    actor_cid_number.as_slice(),
                    actor_role_code.as_slice(),
                    rec.residence_province_code.as_slice(),
                    rec.residence_city_code.as_slice(),
                    CitizenIdentityLevel::Voting,
                    8,
                ),
                Error::<T>::UnauthorizedRegistrar
            );
            Self::tombstone_cid_record(&cid_number);
            if WalletAccountByCid::<T>::contains_key(&cid_number) {
                Self::revoke_bound_identity(&cid_number)?;
            }
            Self::deposit_event(Event::<T>::CidRevoked { cid_number });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// 公民 CID 号全量校验(段结构+机构码+盈利位+校验和)单源
        /// primitives::cid,且机构码必须是公民人 CTZN。
        fn ensure_valid_citizen_cid(cid_number: &CidNumberBound) -> DispatchResult {
            ensure!(!cid_number.is_empty(), Error::<T>::EmptyCidNumber);
            let parts =
                primitives::cid::number::parse_cid_number_parts_bytes(cid_number.as_slice())
                    .map_err(|_| Error::<T>::InvalidCitizenCode)?;
            ensure!(
                parts.institution == *b"CTZN",
                Error::<T>::InvalidCitizenCode
            );
            Ok(())
        }

        /// 身份写入前置:CID 必须已占号且未吊销(占号先行铁律)。
        fn ensure_cid_occupied_active(cid_number: &CidNumberBound) -> DispatchResult {
            match CidRegistry::<T>::get(cid_number) {
                Some(rec) if rec.status == CidRecordStatus::Active => Ok(()),
                Some(_) => Err(Error::<T>::CidAlreadyRevoked.into()),
                None => Err(Error::<T>::CidNotOccupied.into()),
            }
        }

        /// 登记表墓碑:Active → Revoked;不存在或已吊销则不动(幂等)。
        fn tombstone_cid_record(cid_number: &CidNumberBound) {
            CidRegistry::<T>::mutate(cid_number, |rec| {
                if let Some(rec) = rec {
                    if rec.status == CidRecordStatus::Active {
                        rec.status = CidRecordStatus::Revoked;
                        rec.revoked_at = Some(frame_system::Pallet::<T>::block_number());
                    }
                }
            });
        }

        /// 占号核心:链上原子「验格式+查重+登记」。
        /// 同注册局+同承诺哈希的重复提交幂等放行(建档落库失败恢复路径)。
        fn do_occupy_cid(
            registrar_cid_number: &CidNumberBound,
            cid_number: &CidNumberBound,
            commitment: &[u8; 32],
            residence_province_code: &AreaCodeBound,
            residence_city_code: &AreaCodeBound,
        ) -> DispatchResult {
            Self::ensure_valid_citizen_cid(cid_number)?;
            match CidRegistry::<T>::get(cid_number) {
                None => {
                    CidRegistry::<T>::insert(
                        cid_number,
                        CidRecord {
                            registrar_cid_number: registrar_cid_number.clone(),
                            commitment: *commitment,
                            residence_province_code: residence_province_code.clone(),
                            residence_city_code: residence_city_code.clone(),
                            status: CidRecordStatus::Active,
                            registered_at: frame_system::Pallet::<T>::block_number(),
                            revoked_at: None,
                        },
                    );
                    Self::deposit_event(Event::<T>::CidOccupied {
                        cid_number: cid_number.clone(),
                        registrar_cid_number: registrar_cid_number.clone(),
                    });
                    Ok(())
                }
                Some(rec)
                    if rec.status == CidRecordStatus::Active
                        && rec.registrar_cid_number == *registrar_cid_number
                        && rec.commitment == *commitment =>
                {
                    Ok(())
                }
                Some(_) => Err(Error::<T>::CidAlreadyOccupied.into()),
            }
        }

        /// 吊销已绑定的链上身份:状态置 Revoked、退出人口分母、移除参选档案。
        fn revoke_bound_identity(cid_number: &CidNumberBound) -> DispatchResult {
            if let Some(old) = VotingIdentityByCid::<T>::get(cid_number) {
                if old.citizen_status != CitizenStatus::Revoked {
                    let mut revoked = old.clone();
                    revoked.citizen_status = CitizenStatus::Revoked;
                    revoked.updated_at = frame_system::Pallet::<T>::block_number();
                    Self::replace_voting_identity(cid_number.clone(), revoked, Some(old))?;
                    CandidateIdentityByCid::<T>::remove(cid_number);
                }
            }
            Ok(())
        }

        fn ensure_valid_voting_payload(
            payload: &VotingIdentityPayload<T::AccountId>,
        ) -> DispatchResult {
            Self::ensure_valid_citizen_cid(&payload.cid_number)?;
            ensure!(
                !payload.residence_province_code.is_empty()
                    && !payload.residence_city_code.is_empty()
                    && !payload.residence_town_code.is_empty(),
                Error::<T>::EmptyResidenceScope
            );
            ensure!(
                payload.passport_valid_from <= payload.passport_valid_until,
                Error::<T>::InvalidDateRange
            );
            ensure!(
                payload.citizen_age_years >= MIN_ONCHAIN_CITIZEN_AGE_YEARS,
                Error::<T>::UnderVotingAge
            );
            Ok(())
        }

        fn ensure_valid_candidate_payload(
            payload: &CandidateIdentityPayload<T::AccountId>,
        ) -> DispatchResult {
            Self::ensure_valid_voting_payload(&payload.voting)?;
            ensure!(
                !payload.birth_province_code.is_empty()
                    && !payload.birth_city_code.is_empty()
                    && !payload.birth_town_code.is_empty(),
                Error::<T>::EmptyBirthScope
            );
            ensure!(!payload.family_name.is_empty(), Error::<T>::EmptyFamilyName);
            ensure!(!payload.given_name.is_empty(), Error::<T>::EmptyGivenName);
            ensure!(
                Self::is_plausible_yyyymmdd(payload.birth_date),
                Error::<T>::InvalidBirthDate
            );
            // 出生日期决定竞选公民年龄:必须能算出年龄且不低于法定最小年龄。
            let age = Self::age_from_birth_date(payload.birth_date)
                .ok_or(Error::<T>::InvalidBirthDate)?;
            ensure!(
                age >= MIN_ONCHAIN_CITIZEN_AGE_YEARS as u32,
                Error::<T>::UnderVotingAge
            );
            Ok(())
        }

        fn ensure_authorized(
            registrar: &T::AccountId,
            actor_cid_number: &[u8],
            actor_role_code: &[u8],
            payload: &VotingIdentityPayload<T::AccountId>,
            level: CitizenIdentityLevel,
            action_code: u32,
        ) -> DispatchResult {
            ensure!(
                T::CitizenIdentityAuthority::can_manage_voting_identity(
                    registrar,
                    actor_cid_number,
                    actor_role_code,
                    payload.residence_province_code.as_slice(),
                    payload.residence_city_code.as_slice(),
                    level,
                    action_code,
                ),
                Error::<T>::UnauthorizedRegistrar
            );
            Ok(())
        }

        fn ensure_citizen_signature(
            wallet_account: &T::AccountId,
            payload: &[u8],
            signature: &SignatureOf<T>,
        ) -> DispatchResult {
            ensure!(
                T::CitizenIdentityAuthority::verify_citizen_signature(
                    wallet_account,
                    payload,
                    signature,
                ),
                Error::<T>::InvalidCitizenSignature
            );
            Ok(())
        }

        /// 初次登记或候选升级时校验 CID↔钱包双向绑定没有指向另一主体。
        fn ensure_wallet_binding_available(
            cid_number: &CidNumberBound,
            account: &T::AccountId,
        ) -> DispatchResult {
            if let Some(existing) = WalletAccountByCid::<T>::get(cid_number) {
                ensure!(existing == *account, Error::<T>::CidWalletBindingMismatch);
            }
            if let Some(existing) = CidByWalletAccount::<T>::get(account) {
                ensure!(
                    existing == *cid_number,
                    Error::<T>::WalletAlreadyBoundToAnotherCid
                );
            }
            Ok(())
        }

        /// 身份资料更新只能使用该永久 CID 当前绑定的钱包；CID 主键和钱包绑定都不属于本入口可变字段。
        fn ensure_current_wallet_binding(
            cid_number: &CidNumberBound,
            account: &T::AccountId,
        ) -> DispatchResult {
            ensure!(
                WalletAccountByCid::<T>::get(cid_number).as_ref() == Some(account)
                    && CidByWalletAccount::<T>::get(account).as_ref() == Some(cid_number),
                Error::<T>::CidWalletBindingMismatch
            );
            Ok(())
        }

        fn bind_wallet_account(cid_number: &CidNumberBound, account: &T::AccountId) {
            WalletAccountByCid::<T>::insert(cid_number, account);
            CidByWalletAccount::<T>::insert(account, cid_number);
        }

        fn identity_from_payload(
            payload: &VotingIdentityPayload<T::AccountId>,
        ) -> VotingIdentity<BlockNumberFor<T>> {
            VotingIdentity {
                passport_valid_from: payload.passport_valid_from,
                passport_valid_until: payload.passport_valid_until,
                citizen_status: payload.citizen_status,
                residence_province_code: payload.residence_province_code.clone(),
                residence_city_code: payload.residence_city_code.clone(),
                residence_town_code: payload.residence_town_code.clone(),
                updated_at: frame_system::Pallet::<T>::block_number(),
            }
        }

        // 人口计数器只按状态增量维护(链上没有"护照到期"事件,无法按时间自动
        // 减计数),护照过期公民在注册局更新状态前仍计入分母;投票资格由
        // `can_vote` 的护照有效期窗口实时拦截。
        fn identity_counts_as_voter(identity: &VotingIdentity<BlockNumberFor<T>>) -> bool {
            identity.citizen_status == CitizenStatus::Normal
        }

        /// 链上当前日期(UTC+8,YYYYMMDD 整数;时间戳未初始化时返回 0,fail-closed)。
        pub fn current_date_int() -> u32 {
            let secs = <T::TimeProvider as frame_support::traits::UnixTime>::now().as_secs();
            if secs == 0 {
                return 0;
            }
            let days = (secs as i64 + 8 * 3600) / 86_400;
            let (year, month, day) = crate::civil_from_days(days);
            if !(1900..=9999).contains(&year) {
                return 0;
            }
            (year as u32) * 10_000 + month * 100 + day
        }

        /// 校验 YYYYMMDD 整数的基本合法性(年 1900–9999、月 1–12、日 1–31)。
        /// 只做粗校验(不判每月天数),精确到期由业务与前端展示层负责。
        pub fn is_plausible_yyyymmdd(date: u32) -> bool {
            let year = date / 10_000;
            let month = (date / 100) % 100;
            let day = date % 100;
            (1900..=9999).contains(&year) && (1..=12).contains(&month) && (1..=31).contains(&day)
        }

        /// 由出生日期(YYYYMMDD)与链上当前日期(UTC+8)计算周岁。
        /// 整数除法自动判断今年生日是否已过;当前日期未初始化(时间戳=0)、
        /// 出生日期为 0 或落在未来一律返回 `None`(fail-closed)。
        pub fn age_from_birth_date(birth_date: u32) -> Option<u32> {
            let today = Self::current_date_int();
            if today == 0 || birth_date == 0 || birth_date > today {
                return None;
            }
            Some((today - birth_date) / 10_000)
        }

        /// 读取某当前钱包所绑定永久 CID 的竞选身份并计算周岁；无有效主体返回 `None`。
        /// 出生日期是链上公开信息,任何调用方可据此实时计算竞选公民年龄。
        pub fn candidate_age(account: &T::AccountId) -> Option<u32> {
            let subject = Self::citizen_subject(account)?;
            let identity = CandidateIdentityByCid::<T>::get(subject.cid_number)?;
            Self::age_from_birth_date(identity.birth_date)
        }

        /// 出生日期写一次即锁定:已存在竞选身份时,入参出生日期必须与链上一致,
        /// 否则拒绝(防止升级/更新竞选身份时篡改出生日期)。
        fn ensure_birth_date_immutable(
            cid_number: &CidNumberBound,
            incoming: u32,
        ) -> DispatchResult {
            if let Some(existing) = CandidateIdentityByCid::<T>::get(cid_number) {
                ensure!(
                    existing.birth_date == incoming,
                    Error::<T>::BirthDateImmutable
                );
            }
            Ok(())
        }

        /// 护照有效期窗口校验:valid_from ≤ 今日 ≤ valid_until。
        /// 过期或未生效的护照不能投票;时间戳缺失时按不可投票处理。
        fn passport_window_valid(identity: &VotingIdentity<BlockNumberFor<T>>) -> bool {
            let today = Self::current_date_int();
            Self::passport_window_valid_on(identity, today)
        }

        fn passport_window_valid_on(
            identity: &VotingIdentity<BlockNumberFor<T>>,
            date: u32,
        ) -> bool {
            date != 0
                && identity.passport_valid_from <= date
                && date <= identity.passport_valid_until
        }

        fn replace_voting_identity(
            cid_number: CidNumberBound,
            next: VotingIdentity<BlockNumberFor<T>>,
            old: Option<VotingIdentity<BlockNumberFor<T>>>,
        ) -> DispatchResult {
            let revision = NextEligibilityRevision::<T>::get()
                .checked_add(1)
                .ok_or(Error::<T>::EligibilityRevisionOverflow)?;
            let version_count = VotingEligibilityVersionCount::<T>::get(&cid_number);
            if let Some(old_identity) = old {
                if Self::identity_counts_as_voter(&old_identity) {
                    Self::decrement_scope_counts(&old_identity);
                }
                if version_count > 0 {
                    VotingEligibilityVersions::<T>::mutate(
                        &cid_number,
                        version_count.saturating_sub(1),
                        |version| {
                            if let Some(version) = version {
                                version.valid_until_revision = Some(revision);
                            }
                        },
                    );
                }
            }
            if Self::identity_counts_as_voter(&next) {
                Self::increment_scope_counts(&next);
            }
            let next_version_count = version_count
                .checked_add(1)
                .ok_or(Error::<T>::EligibilityVersionOverflow)?;
            VotingEligibilityVersions::<T>::insert(
                &cid_number,
                version_count,
                VotingEligibilityVersion {
                    identity: next.clone(),
                    valid_from_revision: revision,
                    valid_until_revision: None,
                },
            );
            VotingEligibilityVersionCount::<T>::insert(&cid_number, next_version_count);
            NextEligibilityRevision::<T>::put(revision);
            VotingIdentityByCid::<T>::insert(cid_number, next);
            Ok(())
        }

        fn increment_scope_counts(identity: &VotingIdentity<BlockNumberFor<T>>) {
            CountryVotingCount::<T>::mutate(|v| *v = v.saturating_add(1));
            ProvinceVotingCount::<T>::mutate(identity.residence_province_code.clone(), |v| {
                *v = v.saturating_add(1)
            });
            CityVotingCount::<T>::mutate(
                (
                    identity.residence_province_code.clone(),
                    identity.residence_city_code.clone(),
                ),
                |v| *v = v.saturating_add(1),
            );
            TownVotingCount::<T>::mutate(
                (
                    identity.residence_province_code.clone(),
                    identity.residence_city_code.clone(),
                    identity.residence_town_code.clone(),
                ),
                |v| *v = v.saturating_add(1),
            );
        }

        fn decrement_scope_counts(identity: &VotingIdentity<BlockNumberFor<T>>) {
            CountryVotingCount::<T>::mutate(|v| *v = v.saturating_sub(1));
            ProvinceVotingCount::<T>::mutate(identity.residence_province_code.clone(), |v| {
                *v = v.saturating_sub(1)
            });
            CityVotingCount::<T>::mutate(
                (
                    identity.residence_province_code.clone(),
                    identity.residence_city_code.clone(),
                ),
                |v| *v = v.saturating_sub(1),
            );
            TownVotingCount::<T>::mutate(
                (
                    identity.residence_province_code.clone(),
                    identity.residence_city_code.clone(),
                    identity.residence_town_code.clone(),
                ),
                |v| *v = v.saturating_sub(1),
            );
        }

        fn scope_matches(
            identity: &VotingIdentity<BlockNumberFor<T>>,
            scope: &PopulationScope,
        ) -> bool {
            match scope {
                PopulationScope::Country => true,
                PopulationScope::Province(p) => &identity.residence_province_code == p,
                PopulationScope::City(p, c) => {
                    &identity.residence_province_code == p && &identity.residence_city_code == c
                }
                PopulationScope::Town(p, c, t) => {
                    &identity.residence_province_code == p
                        && &identity.residence_city_code == c
                        && &identity.residence_town_code == t
                }
            }
        }

        pub fn population_count_for_scope(scope: &PopulationScope) -> u64 {
            match scope {
                PopulationScope::Country => CountryVotingCount::<T>::get(),
                PopulationScope::Province(p) => ProvinceVotingCount::<T>::get(p.clone()),
                PopulationScope::City(p, c) => CityVotingCount::<T>::get((p.clone(), c.clone())),
                PopulationScope::Town(p, c, t) => {
                    TownVotingCount::<T>::get((p.clone(), c.clone(), t.clone()))
                }
            }
        }

        /// 从身份 Storage 构造完整公民主体。
        ///
        /// 钱包反向 CID、CID 正向钱包、CID 身份和 CID 登记状态必须同时一致；身份或
        /// CID 已吊销、任一方向绑定缺失或错配都返回 `None`，不得退化为裸钱包授权。
        pub fn citizen_subject(who: &T::AccountId) -> Option<CitizenSubject<T::AccountId>> {
            let cid_number = CidByWalletAccount::<T>::get(who)?;
            if WalletAccountByCid::<T>::get(&cid_number).as_ref() != Some(who) {
                return None;
            }
            let identity = VotingIdentityByCid::<T>::get(&cid_number)?;
            if identity.citizen_status != CitizenStatus::Normal {
                return None;
            }
            let record = CidRegistry::<T>::get(&cid_number)?;
            if record.status != CidRecordStatus::Active {
                return None;
            }
            Some(CitizenSubject {
                cid_number,
                wallet_account: who.clone(),
            })
        }

        /// 返回投票引擎生成提案快照所需的同源人口数据。
        ///
        /// 本函数只读取 citizen-identity 自有的四级人口计数、资格 revision 和日期，
        /// 不创建、保存或释放任何投票快照。
        pub fn governance_population_data(scope: &PopulationScope) -> PopulationData {
            PopulationData {
                scope: scope.clone(),
                eligible_total: Self::population_count_for_scope(scope),
                eligibility_revision: NextEligibilityRevision::<T>::get(),
                eligibility_date: Self::current_date_int(),
            }
        }

        /// 按快照 revision 二分定位永久 CID 当时的身份版本。
        fn identity_at_revision(
            cid_number: &CidNumberBound,
            revision: u64,
        ) -> Option<VotingIdentity<BlockNumberFor<T>>> {
            let count = VotingEligibilityVersionCount::<T>::get(cid_number);
            if count == 0 {
                return None;
            }
            let mut low = 0u64;
            let mut high = count;
            while low < high {
                let mid = low.saturating_add(high.saturating_sub(low) / 2);
                let version = VotingEligibilityVersions::<T>::get(cid_number, mid)?;
                if version.valid_from_revision <= revision {
                    low = mid.saturating_add(1);
                } else {
                    high = mid;
                }
            }
            if low == 0 {
                return None;
            }
            let version = VotingEligibilityVersions::<T>::get(cid_number, low.saturating_sub(1))?;
            if version
                .valid_until_revision
                .map(|until| revision >= until)
                .unwrap_or(false)
            {
                return None;
            }
            Some(version.identity)
        }

        /// 使用 citizen-identity 自有历史验证账户在投票引擎快照时点是否具备资格。
        pub fn can_vote_at_population_data(
            who: &T::AccountId,
            population_data: &PopulationData,
        ) -> bool {
            // 历史资格跟随永久 CID；钱包只负责当前交易签名和 CID 反向解析。
            let Some(cid_number) = CidByWalletAccount::<T>::get(who) else {
                return false;
            };
            if WalletAccountByCid::<T>::get(&cid_number).as_ref() != Some(who) {
                return false;
            }
            let Some(identity) =
                Self::identity_at_revision(&cid_number, population_data.eligibility_revision)
            else {
                return false;
            };
            Self::identity_counts_as_voter(&identity)
                && Self::passport_window_valid_on(&identity, population_data.eligibility_date)
                && Self::scope_matches(&identity, &population_data.scope)
        }
    }

    impl<T: Config> crate::CitizenIdentityProvider<T::AccountId> for Pallet<T> {
        fn citizen_subject(who: &T::AccountId) -> Option<CitizenSubject<T::AccountId>> {
            Pallet::<T>::citizen_subject(who)
        }

        // 消费端全量校验:身份存在(注册时已锁定 CID↔钱包一对一并验公民签名)、
        // 状态 NORMAL、护照有效期窗口内、居住地在作用域内。
        fn can_vote(who: &T::AccountId, scope: &PopulationScope) -> bool {
            let Some(subject) = Pallet::<T>::citizen_subject(who) else {
                return false;
            };
            VotingIdentityByCid::<T>::get(subject.cid_number)
                .map(|identity| {
                    Self::identity_counts_as_voter(&identity)
                        && Self::passport_window_valid(&identity)
                        && Self::scope_matches(&identity, scope)
                })
                .unwrap_or(false)
        }

        fn can_be_candidate(who: &T::AccountId, scope: &PopulationScope) -> bool {
            if !Self::can_vote(who, scope) {
                return false;
            }
            CidByWalletAccount::<T>::get(who).is_some_and(CandidateIdentityByCid::<T>::contains_key)
        }

        fn population_count(scope: &PopulationScope) -> u64 {
            Self::population_count_for_scope(scope)
        }

        fn population_data(scope: &PopulationScope) -> PopulationData {
            Self::governance_population_data(scope)
        }

        fn can_vote_at(who: &T::AccountId, population_data: &PopulationData) -> bool {
            Self::can_vote_at_population_data(who, population_data)
        }
    }
}

#[cfg(test)]
mod tests;
