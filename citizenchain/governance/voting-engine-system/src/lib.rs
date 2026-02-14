#![cfg_attr(not(feature = "std"), no_std)]

pub mod citizen_vote;
pub mod internal_vote;
pub mod joint_vote;

pub use citizen_vote::CiicEligibility;
pub use pallet::*;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::dispatch::DispatchResult;
use scale_info::TypeInfo;
use sp_runtime::DispatchError;
use sp_runtime::traits::{SaturatedConversion, Saturating};

pub type InstitutionPalletId = [u8; 8];

pub const PROPOSAL_KIND_INTERNAL: u8 = 0;
pub const PROPOSAL_KIND_JOINT: u8 = 1;

pub const STAGE_INTERNAL: u8 = 0;
pub const STAGE_JOINT: u8 = 1;
pub const STAGE_CITIZEN: u8 = 2;

pub const STATUS_VOTING: u8 = 0;
pub const STATUS_PASSED: u8 = 1;
pub const STATUS_REJECTED: u8 = 2;

pub trait JointVoteEngine {
    fn create_joint_proposal() -> Result<u64, DispatchError>;
}

pub trait JointVoteResultCallback {
    fn on_joint_vote_finalized(vote_proposal_id: u64, approved: bool) -> DispatchResult;
}

impl JointVoteResultCallback for () {
    fn on_joint_vote_finalized(_vote_proposal_id: u64, _approved: bool) -> DispatchResult {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct Proposal<BlockNumber> {
    /// 提案类型：内部投票/联合投票
    pub kind: u8,
    /// 当前所处投票阶段：内部/联合/公民
    pub stage: u8,
    /// 当前提案状态：投票中/通过/否决
    pub status: u8,
    /// 仅内部投票使用：机构类型（国储会/省储会/省储行）
    pub internal_org: Option<u8>,
    /// 本阶段起始区块
    pub start: BlockNumber,
    /// 本阶段截止区块（超过则超时）
    pub end: BlockNumber,
    /// 公民投票阶段的可投票总人数（由外部资格系统给出）
    pub citizen_eligible_total: u64,
}

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    Encode,
    Decode,
    TypeInfo,
    MaxEncodedLen,
)]
pub struct VoteCountU32 {
    /// 赞成票
    pub yes: u32,
    /// 反对票
    pub no: u32,
}

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    Encode,
    Decode,
    TypeInfo,
    MaxEncodedLen,
)]
pub struct VoteCountU64 {
    /// 赞成票
    pub yes: u64,
    /// 反对票
    pub no: u64,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{pallet_prelude::*, Blake2_128Concat};
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>>
            + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        #[pallet::constant]
        type MaxCiicLength: Get<u32>;

        type CiicEligibility: CiicEligibility<Self::AccountId>;

        type JointVoteResultCallback: JointVoteResultCallback;
    }

    pub type CiicOf<T> = BoundedVec<u8, <T as Config>::MaxCiicLength>;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn next_proposal_id)]
    pub type NextProposalId<T> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn proposals)]
    pub type Proposals<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, Proposal<BlockNumberFor<T>>, OptionQuery>;

    #[pallet::storage]
    pub type InternalVotesByAccount<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        T::AccountId,
        bool,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn internal_tally)]
    pub type InternalTallies<T> =
        StorageMap<_, Blake2_128Concat, u64, VoteCountU32, ValueQuery>;

    #[pallet::storage]
    pub type JointVotesByInstitution<T> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        InstitutionPalletId,
        bool,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn joint_tally)]
    pub type JointTallies<T> =
        StorageMap<_, Blake2_128Concat, u64, VoteCountU32, ValueQuery>;

    #[pallet::storage]
    pub type CitizenVotesByCiic<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        T::Hash,
        bool,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn citizen_tally)]
    pub type CitizenTallies<T> =
        StorageMap<_, Blake2_128Concat, u64, VoteCountU64, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ProposalCreated {
            proposal_id: u64,
            kind: u8,
            stage: u8,
            end: BlockNumberFor<T>,
        },
        ProposalAdvancedToCitizen {
            proposal_id: u64,
            citizen_end: BlockNumberFor<T>,
            eligible_total: u64,
        },
        ProposalFinalized {
            proposal_id: u64,
            status: u8,
        },
        InternalVoteCast {
            proposal_id: u64,
            who: T::AccountId,
            approve: bool,
        },
        JointInstitutionVoteCast {
            proposal_id: u64,
            institution: InstitutionPalletId,
            internal_passed: bool,
        },
        CitizenVoteCast {
            proposal_id: u64,
            who: T::AccountId,
            ciic_hash: T::Hash,
            approve: bool,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        ProposalNotFound,
        InvalidProposalKind,
        InvalidProposalStage,
        InvalidProposalStatus,
        InvalidInternalOrg,
        InvalidInstitution,
        VoteClosed,
        VoteNotExpired,
        AlreadyVoted,
        CiicNotEligible,
        EmptyCiic,
        ProposalAlreadyFinalized,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(T::DbWeight::get().reads_writes(2, 2))]
        pub fn create_internal_proposal(
            origin: OriginFor<T>,
            org: u8,
        ) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            Self::do_create_internal_proposal(org)
        }

        #[pallet::call_index(1)]
        #[pallet::weight(T::DbWeight::get().reads_writes(2, 2))]
        pub fn create_joint_proposal(origin: OriginFor<T>) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            Self::do_create_joint_proposal()
        }

        #[pallet::call_index(2)]
        #[pallet::weight(T::DbWeight::get().reads_writes(4, 4))]
        pub fn internal_vote(
            origin: OriginFor<T>,
            proposal_id: u64,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_internal_vote(who, proposal_id, approve)
        }

        #[pallet::call_index(3)]
        #[pallet::weight(T::DbWeight::get().reads_writes(5, 5))]
        pub fn submit_joint_institution_vote(
            origin: OriginFor<T>,
            proposal_id: u64,
            institution: InstitutionPalletId,
            internal_passed: bool,
        ) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            Self::do_submit_joint_institution_vote(proposal_id, institution, internal_passed)
        }

        #[pallet::call_index(4)]
        #[pallet::weight(T::DbWeight::get().reads_writes(4, 4))]
        pub fn citizen_vote(
            origin: OriginFor<T>,
            proposal_id: u64,
            ciic: CiicOf<T>,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_citizen_vote(who, proposal_id, ciic, approve)
        }

        #[pallet::call_index(5)]
        #[pallet::weight(T::DbWeight::get().reads_writes(2, 2))]
        pub fn finalize_proposal(
            origin: OriginFor<T>,
            proposal_id: u64,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            let proposal = Proposals::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;

            match proposal.stage {
                STAGE_INTERNAL => {
                    Self::do_finalize_internal_timeout(proposal_id)?;
                }
                STAGE_JOINT => {
                    Self::do_finalize_joint_timeout(proposal_id)?;
                }
                STAGE_CITIZEN => {
                    Self::do_finalize_citizen_timeout(proposal_id)?;
                }
                _ => return Err(Error::<T>::InvalidProposalStage.into()),
            }

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        pub(crate) fn allocate_proposal_id() -> u64 {
            let id = NextProposalId::<T>::get();
            NextProposalId::<T>::put(id.saturating_add(1));
            id
        }

        pub(crate) fn ensure_open_proposal(
            proposal_id: u64,
        ) -> Result<Proposal<BlockNumberFor<T>>, DispatchError> {
            let proposal = Proposals::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;

            ensure!(proposal.status == STATUS_VOTING, Error::<T>::InvalidProposalStatus);
            ensure!(<frame_system::Pallet<T>>::block_number() <= proposal.end, Error::<T>::VoteClosed);

            Ok(proposal)
        }

        pub(crate) fn set_status_and_emit(proposal_id: u64, status: u8) -> DispatchResult {
            let proposal_before = Proposals::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;
            Proposals::<T>::try_mutate(proposal_id, |maybe| -> DispatchResult {
                let proposal = maybe.as_mut().ok_or(Error::<T>::ProposalNotFound)?;
                proposal.status = status;
                Ok(())
            })?;

            Self::deposit_event(Event::<T>::ProposalFinalized {
                proposal_id,
                status,
            });

            if proposal_before.kind == PROPOSAL_KIND_JOINT && status != STATUS_VOTING {
                T::JointVoteResultCallback::on_joint_vote_finalized(
                    proposal_id,
                    status == STATUS_PASSED,
                )?;
            }
            Ok(())
        }
    }
}

impl<T: pallet::Config> JointVoteEngine for pallet::Pallet<T> {
    fn create_joint_proposal() -> Result<u64, DispatchError> {
        let id = pallet::Pallet::<T>::allocate_proposal_id();
        let now = <frame_system::Pallet<T>>::block_number();
        let duration: frame_system::pallet_prelude::BlockNumberFor<T> =
            (primitives::count_const::VOTING_DURATION_BLOCKS as u64).saturated_into();
        let end = now.saturating_add(duration);

        let proposal = Proposal {
            kind: PROPOSAL_KIND_JOINT,
            stage: STAGE_JOINT,
            status: STATUS_VOTING,
            internal_org: None,
            start: now,
            end,
            citizen_eligible_total: 0,
        };

        pallet::Proposals::<T>::insert(id, proposal);
        pallet::Pallet::<T>::deposit_event(pallet::Event::<T>::ProposalCreated {
            proposal_id: id,
            kind: PROPOSAL_KIND_JOINT,
            stage: STAGE_JOINT,
            end,
        });
        Ok(id)
    }
}
