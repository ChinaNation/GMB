//! # 选举投票 pallet (election-vote)
//!
//! 选举公职人员的多模式入口:普选(公民选) + 互选(机构成员内部互选)。
//!
//! 与 [`joint-vote::jointreferendum`] 不同 — jointreferendum 是联合投票被否决后的
//! 联合公投(yes/no);election-vote 用于按公民宪法选举各类公职人员。
//!
//! 公民宪法当前口径:
//! - 普选:由职位所属或对应的公权机构组织,选民集按国家/省/市/镇行政区或机构范围锁定。
//! - 互选:由机构现任成员在成员快照内互选院长、主席、参议长、众议长等职位。
//! - 同票、补选、递补、重选等细节不写死在本 pallet,后续由选举法规则接入。
//!
//! `popular.rs` 承载普选,`mutual.rs` 承载互选。两者都只做选举投票编排,
//! 当前阶段只生成当选结果快照;最终管理员写入仍必须回到 admins 权限真源。

#![cfg_attr(not(feature = "std"), no_std)]

pub mod cleanup;
pub mod mutual;
pub mod popular;
pub mod snapshot;
pub mod tally;
#[cfg(test)]
mod tests;
pub mod types;

pub use pallet::*;

use frame_support::{ensure, pallet_prelude::DispatchResult};
use frame_system::pallet_prelude::BlockNumberFor;

#[frame_support::pallet]
pub mod pallet {
    use entity_primitives::InstitutionMultisigQuery;
    use frame_support::ensure;
    use frame_support::{
        pallet_prelude::*,
        storage::{with_transaction, TransactionOutcome},
        weights::Weight,
        Blake2_128Concat,
    };
    use frame_system::pallet_prelude::*;
    use primitives::count_const::VOTING_DURATION_BLOCKS;
    use sp_runtime::{
        traits::{SaturatedConversion, Saturating},
        DispatchError,
    };
    use sp_std::vec::Vec;
    use votingengine::InternalAdminProvider;

    use crate::types::{
        ElectionMeta, ElectionMode, ElectionTally as ElectionTallyData, ElectionWinner,
    };

    pub const MODULE_TAG: &[u8] = b"election-vote";

    pub type MaxElectionOfficeCodeOf<T> = <T as Config>::MaxElectionOfficeCodeLen;
    pub type MaxElectionCandidatesOf<T> = <T as Config>::MaxElectionCandidates;
    pub type MaxElectionVotersOf<T> = <T as Config>::MaxElectionVoters;
    pub type ElectionOfficeCodeOf<T> = BoundedVec<u8, MaxElectionOfficeCodeOf<T>>;
    pub type ElectionMetaOf<T> = ElectionMeta<
        <T as frame_system::Config>::AccountId,
        BlockNumberFor<T>,
        ElectionOfficeCodeOf<T>,
    >;
    pub type ElectionWinnerOf<T> = ElectionWinner<<T as frame_system::Config>::AccountId>;

    #[pallet::config]
    pub trait Config: frame_system::Config + votingengine::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 职位编码最大长度。职位含义由业务模块/选举法解释,election-vote 只保存快照。
        #[pallet::constant]
        type MaxElectionOfficeCodeLen: Get<u32>;

        /// 单场选举最大候选人数。
        #[pallet::constant]
        type MaxElectionCandidates: Get<u32>;

        /// 当前框架可直接固化的最大选民快照人数。
        ///
        /// 大规模普选后续应接 CID 凭证/人口快照,不把几万人名单直接塞链上。
        #[pallet::constant]
        type MaxElectionVoters: Get<u32>;

        /// 机构账户 → CID 查询入口。选举提案用 CID 记录组织机构和目标机构。
        type InstitutionQuery: InstitutionMultisigQuery<Self::AccountId>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// 选举职位与规则快照。
    #[pallet::storage]
    pub type ElectionMetaStore<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, ElectionMetaOf<T>, OptionQuery>;

    /// 候选人快照。
    #[pallet::storage]
    pub type ElectionCandidates<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        BoundedVec<T::AccountId, T::MaxElectionCandidates>,
        OptionQuery,
    >;

    /// 选民快照。普选/互选均只认创建时写入的快照。
    #[pallet::storage]
    pub type ElectionVoters<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, u64, Blake2_128Concat, T::AccountId, (), OptionQuery>;

    /// 投票记录:proposal_id + voter → candidate。
    #[pallet::storage]
    pub type ElectionVotesByVoter<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        T::AccountId,
        T::AccountId,
        OptionQuery,
    >;

    /// 候选人票数。
    #[pallet::storage]
    pub type ElectionCandidateTallies<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, u64, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

    /// 本场选举已投票人数。
    #[pallet::storage]
    pub type ElectionTallyStore<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, ElectionTallyData, ValueQuery>;

    /// 当选结果快照。后续 admins/法定代表人模块只能消费该结果,不得反向改票。
    #[pallet::storage]
    pub type ElectionResults<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        BoundedVec<ElectionWinnerOf<T>, T::MaxElectionCandidates>,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 选举提案已创建。
        ElectionCreated {
            proposal_id: u64,
            mode: ElectionMode,
            target: T::AccountId,
            seat_count: u16,
        },
        /// 选民已投票。候选人明文保留,便于链上审计当前最小框架。
        ElectionVoteCast {
            proposal_id: u64,
            voter: T::AccountId,
            candidate: T::AccountId,
        },
        /// 当选结果已生成。
        ElectionResultReady { proposal_id: u64 },
        /// 因无票或席位边界同票,当前框架拒绝本次结果,等待选举法细化规则。
        ElectionRejectedByTieOrNoVotes { proposal_id: u64 },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 职位编码为空。
        EmptyOfficeCode,
        /// 候选人快照为空。
        EmptyCandidateSnapshot,
        /// 选民快照为空。
        EmptyVoterSnapshot,
        /// 候选人数量超过上限。
        TooManyCandidates,
        /// 选民数量超过上限。
        TooManyVoters,
        /// 候选人/选民快照内有重复账户。
        DuplicateAccount,
        /// 席位数非法。
        InvalidSeatCount,
        /// 任期快照非法。
        InvalidTerm,
        /// 选举元数据缺失。
        ElectionMetaMissing,
        /// 调用者不是组织机构管理员。
        NotOrganizerAdmin,
        /// 投票人不在选民快照内。
        VoterNotInSnapshot,
        /// 候选人不在候选快照内。
        CandidateNotInSnapshot,
        /// 组织机构或目标机构无法解析到唯一 CID。
        InvalidInstitutionCid,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 创建普选提案。
        ///
        /// 职位、任期、候选/选民范围由调用方解释并传入快照；
        /// election-vote 不在这里硬编码总统、议员、任期等业务规则。
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        #[allow(clippy::too_many_arguments)]
        pub fn create_popular_election(
            origin: OriginFor<T>,
            organizer_code: votingengine::InstitutionCode,
            organizer: T::AccountId,
            target_code: votingengine::InstitutionCode,
            target: T::AccountId,
            office_code: ElectionOfficeCodeOf<T>,
            rule_id: u32,
            seat_count: u16,
            term_start: BlockNumberFor<T>,
            term_end: BlockNumberFor<T>,
            candidates: Vec<T::AccountId>,
            voters: Vec<T::AccountId>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_create_popular_election(
                who,
                organizer_code,
                organizer,
                target_code,
                target,
                office_code,
                rule_id,
                seat_count,
                term_start,
                term_end,
                candidates,
                voters,
            )?;
            Ok(())
        }

        /// 创建机构内部互选提案。
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        #[allow(clippy::too_many_arguments)]
        pub fn create_mutual_election(
            origin: OriginFor<T>,
            organizer_code: votingengine::InstitutionCode,
            organizer: T::AccountId,
            target_code: votingengine::InstitutionCode,
            target: T::AccountId,
            office_code: ElectionOfficeCodeOf<T>,
            rule_id: u32,
            seat_count: u16,
            term_start: BlockNumberFor<T>,
            term_end: BlockNumberFor<T>,
            candidates: Vec<T::AccountId>,
            voters: Vec<T::AccountId>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_create_mutual_election(
                who,
                organizer_code,
                organizer,
                target_code,
                target,
                office_code,
                rule_id,
                seat_count,
                term_start,
                term_end,
                candidates,
                voters,
            )?;
            Ok(())
        }

        /// 普选投票。
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn cast_popular_vote(
            origin: OriginFor<T>,
            proposal_id: u64,
            candidate: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_cast_popular_vote(who, proposal_id, candidate)
        }

        /// 互选投票。
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn cast_mutual_vote(
            origin: OriginFor<T>,
            proposal_id: u64,
            candidate: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_cast_mutual_vote(who, proposal_id, candidate)
        }
    }

    impl<T: Config> Pallet<T> {
        fn stage_duration() -> BlockNumberFor<T> {
            (VOTING_DURATION_BLOCKS as u64).saturated_into()
        }

        fn push_subject_cid(
            raw: &mut Vec<Vec<u8>>,
            account: &T::AccountId,
        ) -> Result<(), DispatchError> {
            let cid = T::InstitutionQuery::lookup_cid(account)
                .ok_or(Error::<T>::InvalidInstitutionCid)?;
            if !raw.iter().any(|existing| existing == &cid) {
                raw.push(cid);
            }
            Ok(())
        }

        fn resolve_subject_cid_numbers(
            organizer: &T::AccountId,
            target: &T::AccountId,
        ) -> Result<votingengine::types::ProposalSubjectCidNumbers, DispatchError> {
            let mut raw = Vec::new();
            Self::push_subject_cid(&mut raw, organizer)?;
            Self::push_subject_cid(&mut raw, target)?;
            votingengine::Pallet::<T>::bound_subject_cid_numbers(raw)
        }

        #[allow(clippy::too_many_arguments)]
        pub(crate) fn do_create_election(
            who: T::AccountId,
            mode: ElectionMode,
            organizer_code: votingengine::InstitutionCode,
            organizer: T::AccountId,
            target_code: votingengine::InstitutionCode,
            target: T::AccountId,
            office_code: ElectionOfficeCodeOf<T>,
            rule_id: u32,
            seat_count: u16,
            term_start: BlockNumberFor<T>,
            term_end: BlockNumberFor<T>,
            candidates: Vec<T::AccountId>,
            voters: Vec<T::AccountId>,
        ) -> Result<u64, DispatchError> {
            ensure!(!office_code.is_empty(), Error::<T>::EmptyOfficeCode);
            ensure!(seat_count > 0, Error::<T>::InvalidSeatCount);
            ensure!(term_start <= term_end, Error::<T>::InvalidTerm);
            ensure!(
                <T as votingengine::Config>::InternalAdminProvider::is_internal_admin(
                    organizer_code,
                    organizer.clone(),
                    &who,
                ),
                Error::<T>::NotOrganizerAdmin
            );

            let bounded_candidates = Self::bounded_candidates(candidates)?;
            ensure!(
                usize::from(seat_count) <= bounded_candidates.len(),
                Error::<T>::InvalidSeatCount
            );
            let bounded_voters = Self::bounded_voters(voters)?;

            let now = frame_system::Pallet::<T>::block_number();
            let end = now.saturating_add(Self::stage_duration());
            let stage = mode.stage();
            let subject_cid_numbers = Self::resolve_subject_cid_numbers(&organizer, &target)?;
            let proposal = votingengine::Proposal {
                kind: votingengine::PROPOSAL_KIND_ELECTION,
                stage,
                status: votingengine::STATUS_VOTING,
                internal_code: Some(target_code),
                account_context: Some(target.clone()),
                subject_cid_numbers,
                start: now,
                end,
                citizen_eligible_total: bounded_voters.len() as u64,
            };
            let meta = ElectionMeta {
                mode,
                organizer_code,
                organizer,
                target_code,
                target: target.clone(),
                office_code,
                rule_id,
                seat_count,
                term_start,
                term_end,
            };

            with_transaction(|| {
                let id = match votingengine::Pallet::<T>::allocate_proposal_id() {
                    Ok(id) => id,
                    Err(err) => return TransactionOutcome::Rollback(Err(err)),
                };
                if let Err(err) =
                    votingengine::limit::try_add_active_proposals::<T>(proposal.subject_keys(), id)
                {
                    return TransactionOutcome::Rollback(Err(err));
                }
                votingengine::pallet::Proposals::<T>::insert(id, proposal);
                ElectionMetaStore::<T>::insert(id, meta);
                ElectionCandidates::<T>::insert(id, bounded_candidates.clone());
                ElectionTallyStore::<T>::insert(id, crate::types::ElectionTally::default());
                Self::write_voter_snapshot(id, &bounded_voters);
                if let Err(err) = votingengine::Pallet::<T>::register_proposal_data(
                    id,
                    MODULE_TAG,
                    Vec::new(),
                    now,
                ) {
                    return TransactionOutcome::Rollback(Err(err));
                }
                if let Err(err) = votingengine::Pallet::<T>::schedule_proposal_expiry(id, end) {
                    return TransactionOutcome::Rollback(Err(err));
                }
                votingengine::Pallet::<T>::emit_proposal_created(
                    id,
                    votingengine::PROPOSAL_KIND_ELECTION,
                    stage,
                    end,
                );
                Self::deposit_event(Event::<T>::ElectionCreated {
                    proposal_id: id,
                    mode,
                    target,
                    seat_count,
                });
                TransactionOutcome::Commit(Ok(id))
            })
        }

        pub(crate) fn do_cast_election_vote(
            who: T::AccountId,
            proposal_id: u64,
            expected_stage: u8,
            candidate: T::AccountId,
        ) -> DispatchResult {
            let proposal = votingengine::Pallet::<T>::ensure_open_proposal(proposal_id)?;
            ensure!(
                proposal.kind == votingengine::PROPOSAL_KIND_ELECTION,
                votingengine::Error::<T>::InvalidProposalKind
            );
            ensure!(
                proposal.stage == expected_stage,
                votingengine::Error::<T>::InvalidProposalStage
            );
            ensure!(
                !ElectionVotesByVoter::<T>::contains_key(proposal_id, &who),
                votingengine::Error::<T>::AlreadyVoted
            );
            ensure!(
                Self::voter_exists(proposal_id, &who),
                Error::<T>::VoterNotInSnapshot
            );
            ensure!(
                Self::candidate_exists(proposal_id, &candidate),
                Error::<T>::CandidateNotInSnapshot
            );

            ElectionVotesByVoter::<T>::insert(proposal_id, &who, &candidate);
            ElectionCandidateTallies::<T>::mutate(proposal_id, &candidate, |votes| {
                *votes = votes.saturating_add(1);
            });
            let tally = ElectionTallyStore::<T>::mutate(proposal_id, |t| {
                t.casted = t.casted.saturating_add(1);
                *t
            });
            Self::deposit_event(Event::<T>::ElectionVoteCast {
                proposal_id,
                voter: who,
                candidate,
            });

            if tally.casted >= proposal.citizen_eligible_total as u32 {
                Self::finalize_election_result(proposal_id)?;
            }
            Ok(())
        }
    }
}

impl<T: pallet::Config> votingengine::ElectionProposalFinalizer<BlockNumberFor<T>, T::AccountId>
    for pallet::Pallet<T>
{
    fn finalize_election_popular_timeout(
        proposal: &votingengine::Proposal<BlockNumberFor<T>, T::AccountId>,
        proposal_id: u64,
    ) -> DispatchResult {
        ensure!(
            frame_system::Pallet::<T>::block_number() > proposal.end,
            votingengine::Error::<T>::VoteNotExpired
        );
        pallet::Pallet::<T>::finalize_election_result(proposal_id)
    }

    fn finalize_election_mutual_timeout(
        proposal: &votingengine::Proposal<BlockNumberFor<T>, T::AccountId>,
        proposal_id: u64,
    ) -> DispatchResult {
        ensure!(
            frame_system::Pallet::<T>::block_number() > proposal.end,
            votingengine::Error::<T>::VoteNotExpired
        );
        pallet::Pallet::<T>::finalize_election_result(proposal_id)
    }
}

impl<T: pallet::Config> votingengine::ElectionVoteResultCallback for pallet::Pallet<T> {
    fn on_election_vote_finalized(
        vote_proposal_id: u64,
        approved: bool,
    ) -> Result<votingengine::ProposalExecutionOutcome, sp_runtime::DispatchError> {
        if !pallet::ElectionMetaStore::<T>::contains_key(vote_proposal_id) {
            return Ok(votingengine::ProposalExecutionOutcome::Ignored);
        }
        if approved && !pallet::ElectionResults::<T>::contains_key(vote_proposal_id) {
            return Ok(votingengine::ProposalExecutionOutcome::FatalFailed);
        }
        Ok(votingengine::ProposalExecutionOutcome::Executed)
    }
}
