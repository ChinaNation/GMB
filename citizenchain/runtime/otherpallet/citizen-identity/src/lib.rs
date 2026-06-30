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
pub type CitizenNameBound = BoundedVec<u8, ConstU32<128>>;

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
    pub voting: VotingIdentityPayload<AccountId>,
    pub birth_province_code: AreaCodeBound,
    pub birth_city_code: AreaCodeBound,
    pub birth_town_code: AreaCodeBound,
    pub citizen_full_name: CitizenNameBound,
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
    pub cid_number: CidNumberBound,
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
    pub birth_province_code: AreaCodeBound,
    pub birth_city_code: AreaCodeBound,
    pub birth_town_code: AreaCodeBound,
    pub citizen_full_name: CitizenNameBound,
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
pub struct PopulationSnapshot<BlockNumber> {
    pub scope: PopulationScope,
    pub eligible_total: u64,
    pub created_at: BlockNumber,
}

pub trait CitizenIdentityAuthority<AccountId, Signature> {
    fn can_manage_voting_identity(
        registrar: &AccountId,
        registrar_account: &AccountId,
        residence_province_code: &[u8],
        residence_city_code: &[u8],
        level: CitizenIdentityLevel,
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
        _registrar_account: &AccountId,
        _residence_province_code: &[u8],
        _residence_city_code: &[u8],
        _level: CitizenIdentityLevel,
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
    fn can_vote(who: &AccountId, scope: &PopulationScope) -> bool;
    fn can_be_candidate(who: &AccountId, scope: &PopulationScope) -> bool;
    fn population_count(scope: &PopulationScope) -> u64;
}

impl<AccountId> CitizenIdentityProvider<AccountId> for () {
    fn can_vote(_who: &AccountId, _scope: &PopulationScope) -> bool {
        false
    }

    fn can_be_candidate(_who: &AccountId, _scope: &PopulationScope) -> bool {
        false
    }

    fn population_count(_scope: &PopulationScope) -> u64 {
        0
    }
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;
    use frame_support::{pallet_prelude::*, Blake2_128Concat};
    use frame_system::pallet_prelude::*;

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

        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub type VotingIdentityByAccount<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        VotingIdentity<BlockNumberFor<T>>,
        OptionQuery,
    >;

    #[pallet::storage]
    pub type CandidateIdentityByAccount<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        CandidateIdentity<BlockNumberFor<T>>,
        OptionQuery,
    >;

    #[pallet::storage]
    pub type AccountByCid<T: Config> =
        StorageMap<_, Blake2_128Concat, CidNumberBound, T::AccountId, OptionQuery>;

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

    #[pallet::storage]
    pub type NextSnapshotId<T> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    pub type PopulationSnapshots<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, PopulationSnapshot<BlockNumberFor<T>>, OptionQuery>;

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
        PopulationSnapshotCreated {
            snapshot_id: u64,
            scope: PopulationScope,
            eligible_total: u64,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        EmptyCidNumber,
        EmptyResidenceScope,
        EmptyBirthScope,
        EmptyCitizenName,
        InvalidDateRange,
        InvalidCitizenCode,
        UnauthorizedRegistrar,
        InvalidCitizenSignature,
        CidAlreadyRegisteredToAnotherAccount,
        CidNotFound,
        VotingIdentityNotFound,
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
            registrar_account: T::AccountId,
            payload: VotingIdentityPayload<T::AccountId>,
            citizen_signature: SignatureOf<T>,
        ) -> DispatchResult {
            let registrar = ensure_signed(origin)?;
            Self::ensure_valid_voting_payload(&payload)?;
            Self::ensure_authorized(
                &registrar,
                &registrar_account,
                &payload,
                CitizenIdentityLevel::Voting,
            )?;
            Self::ensure_citizen_signature(
                &payload.wallet_account,
                &payload.encode(),
                &citizen_signature,
            )?;
            Self::ensure_cid_available(&payload.cid_number, &payload.wallet_account)?;

            let old = VotingIdentityByAccount::<T>::get(&payload.wallet_account);
            let first_identity = old.is_none();
            let identity = Self::identity_from_payload(&payload);
            Self::replace_voting_identity(payload.wallet_account.clone(), identity, old);
            AccountByCid::<T>::insert(&payload.cid_number, &payload.wallet_account);

            if first_identity {
                T::OnVotingIdentityRegistered::on_voting_identity_registered(
                    &payload.wallet_account,
                    payload.cid_number.as_slice(),
                );
                Self::deposit_event(Event::<T>::VotingIdentityRegistered {
                    wallet_account: payload.wallet_account,
                    cid_number: payload.cid_number,
                });
            } else {
                Self::deposit_event(Event::<T>::VotingIdentityUpdated {
                    wallet_account: payload.wallet_account,
                    cid_number: payload.cid_number,
                });
            }
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::upgrade_to_candidate_identity())]
        pub fn upgrade_to_candidate_identity(
            origin: OriginFor<T>,
            registrar_account: T::AccountId,
            payload: CandidateIdentityPayload<T::AccountId>,
            citizen_signature: SignatureOf<T>,
        ) -> DispatchResult {
            let registrar = ensure_signed(origin)?;
            Self::ensure_valid_candidate_payload(&payload)?;
            Self::ensure_authorized(
                &registrar,
                &registrar_account,
                &payload.voting,
                CitizenIdentityLevel::Candidate,
            )?;
            Self::ensure_citizen_signature(
                &payload.voting.wallet_account,
                &payload.encode(),
                &citizen_signature,
            )?;
            Self::ensure_cid_available(&payload.voting.cid_number, &payload.voting.wallet_account)?;

            let old = VotingIdentityByAccount::<T>::get(&payload.voting.wallet_account);
            let identity = Self::identity_from_payload(&payload.voting);
            Self::replace_voting_identity(payload.voting.wallet_account.clone(), identity, old);
            AccountByCid::<T>::insert(&payload.voting.cid_number, &payload.voting.wallet_account);
            CandidateIdentityByAccount::<T>::insert(
                &payload.voting.wallet_account,
                CandidateIdentity {
                    birth_province_code: payload.birth_province_code,
                    birth_city_code: payload.birth_city_code,
                    birth_town_code: payload.birth_town_code,
                    citizen_full_name: payload.citizen_full_name,
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
            registrar_account: T::AccountId,
            payload: VotingIdentityPayload<T::AccountId>,
            citizen_signature: SignatureOf<T>,
        ) -> DispatchResult {
            let registrar = ensure_signed(origin)?;
            Self::ensure_valid_voting_payload(&payload)?;
            Self::ensure_authorized(
                &registrar,
                &registrar_account,
                &payload,
                CitizenIdentityLevel::Voting,
            )?;
            Self::ensure_citizen_signature(
                &payload.wallet_account,
                &payload.encode(),
                &citizen_signature,
            )?;
            Self::ensure_cid_available(&payload.cid_number, &payload.wallet_account)?;

            let old = VotingIdentityByAccount::<T>::get(&payload.wallet_account)
                .ok_or(Error::<T>::VotingIdentityNotFound)?;
            let identity = Self::identity_from_payload(&payload);
            Self::replace_voting_identity(payload.wallet_account.clone(), identity, Some(old));
            AccountByCid::<T>::insert(&payload.cid_number, &payload.wallet_account);
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
            registrar_account: T::AccountId,
            payload: CandidateIdentityPayload<T::AccountId>,
            citizen_signature: SignatureOf<T>,
        ) -> DispatchResult {
            let registrar = ensure_signed(origin)?;
            Self::ensure_valid_candidate_payload(&payload)?;
            Self::ensure_authorized(
                &registrar,
                &registrar_account,
                &payload.voting,
                CitizenIdentityLevel::Candidate,
            )?;
            Self::ensure_citizen_signature(
                &payload.voting.wallet_account,
                &payload.encode(),
                &citizen_signature,
            )?;
            Self::ensure_cid_available(&payload.voting.cid_number, &payload.voting.wallet_account)?;

            let old = VotingIdentityByAccount::<T>::get(&payload.voting.wallet_account)
                .ok_or(Error::<T>::VotingIdentityNotFound)?;
            let identity = Self::identity_from_payload(&payload.voting);
            Self::replace_voting_identity(
                payload.voting.wallet_account.clone(),
                identity,
                Some(old),
            );
            CandidateIdentityByAccount::<T>::insert(
                &payload.voting.wallet_account,
                CandidateIdentity {
                    birth_province_code: payload.birth_province_code,
                    birth_city_code: payload.birth_city_code,
                    birth_town_code: payload.birth_town_code,
                    citizen_full_name: payload.citizen_full_name,
                    updated_at: frame_system::Pallet::<T>::block_number(),
                },
            );
            AccountByCid::<T>::insert(&payload.voting.cid_number, &payload.voting.wallet_account);
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
            registrar_account: T::AccountId,
            cid_number: CidNumberBound,
        ) -> DispatchResult {
            let registrar = ensure_signed(origin)?;
            ensure!(!cid_number.is_empty(), Error::<T>::EmptyCidNumber);
            let account = AccountByCid::<T>::get(&cid_number).ok_or(Error::<T>::CidNotFound)?;
            let old = VotingIdentityByAccount::<T>::get(&account)
                .ok_or(Error::<T>::VotingIdentityNotFound)?;
            Self::ensure_authorized(
                &registrar,
                &registrar_account,
                &VotingIdentityPayload {
                    cid_number: old.cid_number.clone(),
                    wallet_account: account.clone(),
                    passport_valid_from: old.passport_valid_from,
                    passport_valid_until: old.passport_valid_until,
                    citizen_status: old.citizen_status,
                    residence_province_code: old.residence_province_code.clone(),
                    residence_city_code: old.residence_city_code.clone(),
                    residence_town_code: old.residence_town_code.clone(),
                },
                CitizenIdentityLevel::Voting,
            )?;

            let mut revoked = old.clone();
            revoked.citizen_status = CitizenStatus::Revoked;
            revoked.updated_at = frame_system::Pallet::<T>::block_number();
            Self::replace_voting_identity(account.clone(), revoked, Some(old));
            CandidateIdentityByAccount::<T>::remove(&account);
            Self::deposit_event(Event::<T>::CitizenIdentityRevoked {
                wallet_account: account,
                cid_number,
            });
            Ok(())
        }

        #[pallet::call_index(5)]
        #[pallet::weight(<T as Config>::WeightInfo::start_population_snapshot())]
        pub fn start_population_snapshot(
            origin: OriginFor<T>,
            scope: PopulationScope,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            let snapshot_id = NextSnapshotId::<T>::get();
            let eligible_total = Self::population_count_for_scope(&scope);
            let snapshot = PopulationSnapshot {
                scope: scope.clone(),
                eligible_total,
                created_at: frame_system::Pallet::<T>::block_number(),
            };
            PopulationSnapshots::<T>::insert(snapshot_id, snapshot);
            NextSnapshotId::<T>::put(snapshot_id.saturating_add(1));
            Self::deposit_event(Event::<T>::PopulationSnapshotCreated {
                snapshot_id,
                scope,
                eligible_total,
            });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn ensure_valid_voting_payload(
            payload: &VotingIdentityPayload<T::AccountId>,
        ) -> DispatchResult {
            ensure!(!payload.cid_number.is_empty(), Error::<T>::EmptyCidNumber);
            ensure!(
                payload.cid_number.as_slice().starts_with(b"CTZN"),
                Error::<T>::InvalidCitizenCode
            );
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
            ensure!(
                !payload.citizen_full_name.is_empty(),
                Error::<T>::EmptyCitizenName
            );
            Ok(())
        }

        fn ensure_authorized(
            registrar: &T::AccountId,
            registrar_account: &T::AccountId,
            payload: &VotingIdentityPayload<T::AccountId>,
            level: CitizenIdentityLevel,
        ) -> DispatchResult {
            ensure!(
                T::CitizenIdentityAuthority::can_manage_voting_identity(
                    registrar,
                    registrar_account,
                    payload.residence_province_code.as_slice(),
                    payload.residence_city_code.as_slice(),
                    level,
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

        fn ensure_cid_available(
            cid_number: &CidNumberBound,
            account: &T::AccountId,
        ) -> DispatchResult {
            if let Some(existing) = AccountByCid::<T>::get(cid_number) {
                ensure!(
                    existing == *account,
                    Error::<T>::CidAlreadyRegisteredToAnotherAccount
                );
            }
            Ok(())
        }

        fn identity_from_payload(
            payload: &VotingIdentityPayload<T::AccountId>,
        ) -> VotingIdentity<BlockNumberFor<T>> {
            VotingIdentity {
                cid_number: payload.cid_number.clone(),
                passport_valid_from: payload.passport_valid_from,
                passport_valid_until: payload.passport_valid_until,
                citizen_status: payload.citizen_status,
                residence_province_code: payload.residence_province_code.clone(),
                residence_city_code: payload.residence_city_code.clone(),
                residence_town_code: payload.residence_town_code.clone(),
                updated_at: frame_system::Pallet::<T>::block_number(),
            }
        }

        fn identity_counts_as_voter(identity: &VotingIdentity<BlockNumberFor<T>>) -> bool {
            identity.citizen_status == CitizenStatus::Normal
        }

        fn replace_voting_identity(
            account: T::AccountId,
            next: VotingIdentity<BlockNumberFor<T>>,
            old: Option<VotingIdentity<BlockNumberFor<T>>>,
        ) {
            if let Some(old_identity) = old {
                if Self::identity_counts_as_voter(&old_identity) {
                    Self::decrement_scope_counts(&old_identity);
                }
                if old_identity.cid_number != next.cid_number {
                    AccountByCid::<T>::remove(&old_identity.cid_number);
                }
            }
            if Self::identity_counts_as_voter(&next) {
                Self::increment_scope_counts(&next);
            }
            VotingIdentityByAccount::<T>::insert(account, next);
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
    }

    impl<T: Config> crate::CitizenIdentityProvider<T::AccountId> for Pallet<T> {
        fn can_vote(who: &T::AccountId, scope: &PopulationScope) -> bool {
            VotingIdentityByAccount::<T>::get(who)
                .map(|identity| {
                    Self::identity_counts_as_voter(&identity)
                        && Self::scope_matches(&identity, scope)
                })
                .unwrap_or(false)
        }

        fn can_be_candidate(who: &T::AccountId, scope: &PopulationScope) -> bool {
            if !Self::can_vote(who, scope) {
                return false;
            }
            CandidateIdentityByAccount::<T>::contains_key(who)
        }

        fn population_count(scope: &PopulationScope) -> u64 {
            Self::population_count_for_scope(scope)
        }
    }
}

#[cfg(test)]
mod tests;
