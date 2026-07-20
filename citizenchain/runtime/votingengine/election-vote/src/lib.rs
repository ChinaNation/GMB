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
//! `popular.rs` 承载普选,`mutual.rs` 承载互选。两者只做选举投票编排和结果快照；
//! 选举业务模块必须复核结果与业务规则后，才能调用 entity 任职入口。

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod cleanup;
pub mod mutual;
pub mod popular;
pub mod snapshot;
pub mod tally;
#[cfg(test)]
mod tests;
pub mod types;
pub mod weights;

pub use pallet::*;

use frame_support::{ensure, pallet_prelude::DispatchResult};
use frame_system::pallet_prelude::BlockNumberFor;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::ensure;
    use frame_support::{
        pallet_prelude::*,
        storage::{with_transaction, TransactionOutcome},
        Blake2_128Concat,
    };
    use frame_system::pallet_prelude::*;
    use primitives::count_const::VOTING_DURATION_BLOCKS;
    use sp_runtime::{
        traits::{SaturatedConversion, Saturating},
        DispatchError,
    };
    use sp_std::vec::Vec;
    use votingengine::{CitizenIdentityReader, InstitutionRoleProvider as _};

    use crate::types::{
        ElectionMeta, ElectionMode, ElectionTally as ElectionTallyData, ElectionWinner,
    };
    use crate::weights::WeightInfo;

    pub const MODULE_TAG: &[u8] = b"election-vote";

    pub type MaxElectionOfficeCodeOf<T> = <T as Config>::MaxElectionOfficeCodeLen;
    pub type MaxElectionCandidatesOf<T> = <T as Config>::MaxElectionCandidates;
    pub type ElectionOfficeCodeOf<T> = BoundedVec<u8, MaxElectionOfficeCodeOf<T>>;
    pub type ElectionMetaOf<T> = ElectionMeta<ElectionOfficeCodeOf<T>>;
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

        /// 互选岗位任职读取器；只提供 entity 的有效任职事实，不解释业务权限。
        type InstitutionRoleProvider: votingengine::InstitutionRoleProvider<Self::AccountId>;

        /// 选举写票和候选人计票路径的实测权重。
        type WeightInfo: crate::weights::WeightInfo;
    }

    /// 重新创世直接使用含人口作用域的最终选举元数据布局。
    pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
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

    /// 当选结果快照。业务模块只读消费并复核，投票引擎不得直接写入 entity。
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
            target_cid_number: votingengine::types::CidNumber,
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
        /// 互选管理员快照为空。
        EmptyVoterSnapshot,
        /// 候选人数量超过上限。
        TooManyCandidates,
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
        /// 普选选民不具备 citizen-identity 投票资格。
        VoterNotEligible,
        /// 普选候选人不具备 citizen-identity 参选资格。
        CandidateNotEligible,
        /// 选举缺少与模式匹配的资格作用域。
        ElectionScopeMissing,
        /// 选举计划与模式、机构岗位主体不一致。
        InvalidVotePlan,
        /// 组织机构或目标机构无法解析到唯一 CID。
        InvalidInstitutionCid,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 普选投票。
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::cast_popular_vote(
            ElectionCandidates::<T>::get(*proposal_id)
                .map(|items| items.len() as u32)
                .unwrap_or_default()
        ))]
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
        #[pallet::weight(<T as Config>::WeightInfo::cast_mutual_vote(
            ElectionCandidates::<T>::get(*proposal_id)
                .map(|items| items.len() as u32)
                .unwrap_or_default()
        ))]
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

        fn resolve_subject_cid_numbers(
            actor_cid_number: &votingengine::types::CidNumber,
            target_cid_number: &votingengine::types::CidNumber,
            vote_plan: &votingengine::types::VotePlanOf<T::AccountId>,
        ) -> Result<votingengine::types::ProposalSubjectCidNumbers, DispatchError> {
            let mut raw = Vec::from([actor_cid_number.to_vec()]);
            if actor_cid_number != target_cid_number {
                raw.push(target_cid_number.to_vec());
            }
            for subject in &vote_plan.voter_subjects {
                if let votingengine::types::AuthorizationSubject::Institution(role) = subject {
                    raw.push(role.cid_number.to_vec());
                }
            }
            votingengine::Pallet::<T>::bound_subject_cid_numbers(raw)
        }

        #[allow(clippy::too_many_arguments)]
        pub(crate) fn do_create_election(
            who: T::AccountId,
            vote_plan: votingengine::types::VotePlanOf<T::AccountId>,
            mode: ElectionMode,
            actor_cid_number: votingengine::types::CidNumber,
            target_cid_number: votingengine::types::CidNumber,
            office_code: ElectionOfficeCodeOf<T>,
            rule_id: u32,
            seat_count: u16,
            term_start: u32,
            term_end: u32,
            population_scope: Option<votingengine::PopulationScope>,
            candidates: Vec<T::AccountId>,
        ) -> Result<u64, DispatchError> {
            ensure!(!office_code.is_empty(), Error::<T>::EmptyOfficeCode);
            ensure!(seat_count > 0, Error::<T>::InvalidSeatCount);
            ensure!(term_start <= term_end, Error::<T>::InvalidTerm);
            let _actor_code = votingengine::types::institution_code_from_cid_number(
                core::str::from_utf8(actor_cid_number.as_slice())
                    .map_err(|_| Error::<T>::InvalidInstitutionCid)?,
            )
            .ok_or(Error::<T>::InvalidInstitutionCid)?;
            let target_code = votingengine::types::institution_code_from_cid_number(
                core::str::from_utf8(target_cid_number.as_slice())
                    .map_err(|_| Error::<T>::InvalidInstitutionCid)?,
            )
            .ok_or(Error::<T>::InvalidInstitutionCid)?;
            ensure!(
                vote_plan.voting_engine == votingengine::types::VotingEngineKind::Election,
                Error::<T>::InvalidVotePlan
            );
            let proposer_role = match &vote_plan.proposer_subject {
                votingengine::types::AuthorizationSubject::Institution(role) => role,
                votingengine::types::AuthorizationSubject::PersonalMultisig(_) => {
                    return Err(Error::<T>::InvalidVotePlan.into())
                }
            };
            ensure!(
                proposer_role.cid_number == actor_cid_number
                    && T::InstitutionRoleProvider::is_active_assignment(
                        actor_cid_number.as_slice(),
                        &who,
                        proposer_role.role_code.as_slice(),
                    ),
                Error::<T>::NotOrganizerAdmin
            );

            let bounded_candidates = Self::bounded_candidates(candidates)?;
            ensure!(
                usize::from(seat_count) <= bounded_candidates.len(),
                Error::<T>::InvalidSeatCount
            );
            match (mode, population_scope.as_ref()) {
                (ElectionMode::Popular, Some(scope)) => {
                    ensure!(
                        vote_plan.voter_subjects.is_empty(),
                        Error::<T>::InvalidVotePlan
                    );
                    ensure!(
                        bounded_candidates.iter().all(|candidate| {
                            <T as votingengine::Config>::CitizenIdentityReader::can_be_candidate(
                                candidate, scope,
                            )
                        }),
                        Error::<T>::CandidateNotEligible
                    );
                }
                (ElectionMode::Mutual, None) => {
                    ensure!(
                        !vote_plan.voter_subjects.is_empty()
                            && vote_plan.voter_subjects.iter().all(|subject| matches!(
                                subject,
                                votingengine::types::AuthorizationSubject::Institution(role)
                                    if role.cid_number == target_cid_number
                            )),
                        Error::<T>::InvalidVotePlan
                    );
                    let mut eligible_candidates = Vec::new();
                    for subject in &vote_plan.voter_subjects {
                        let votingengine::types::AuthorizationSubject::Institution(role) = subject
                        else {
                            return Err(Error::<T>::InvalidVotePlan.into());
                        };
                        for account in T::InstitutionRoleProvider::active_accounts_for_role(
                            role.cid_number.as_slice(),
                            role.role_code.as_slice(),
                        ) {
                            if !eligible_candidates.contains(&account) {
                                eligible_candidates.push(account);
                            }
                        }
                    }
                    ensure!(
                        !eligible_candidates.is_empty(),
                        Error::<T>::EmptyVoterSnapshot
                    );
                    ensure!(
                        bounded_candidates
                            .iter()
                            .all(|candidate| eligible_candidates.contains(candidate)),
                        Error::<T>::CandidateNotEligible
                    );
                }
                _ => return Err(Error::<T>::ElectionScopeMissing.into()),
            }

            let now = frame_system::Pallet::<T>::block_number();
            let end = now.saturating_add(Self::stage_duration());
            let stage = mode.stage();
            let subject_cid_numbers = Self::resolve_subject_cid_numbers(
                &actor_cid_number,
                &target_cid_number,
                &vote_plan,
            )?;
            let proposal_owner = vote_plan.proposal_owner.to_vec();
            let proposal = votingengine::Proposal {
                kind: votingengine::PROPOSAL_KIND_ELECTION,
                stage,
                status: votingengine::STATUS_VOTING,
                internal_code: Some(target_code),
                actor_cid_number: Some(actor_cid_number.clone()),
                execution_account: None,
                subject_cid_numbers,
                start: now,
                end,
            };
            let meta = ElectionMeta {
                mode,
                population_scope: population_scope.clone(),
                actor_cid_number,
                target_cid_number: target_cid_number.clone(),
                office_code,
                rule_id,
                seat_count,
                term_start,
                term_end,
            };

            let result = with_transaction(|| {
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
                if mode == ElectionMode::Mutual {
                    for subject in &vote_plan.voter_subjects {
                        let votingengine::types::AuthorizationSubject::Institution(role) = subject
                        else {
                            return TransactionOutcome::Rollback(Err(
                                Error::<T>::InvalidVotePlan.into()
                            ));
                        };
                        let voters = T::InstitutionRoleProvider::active_accounts_for_role(
                            role.cid_number.as_slice(),
                            role.role_code.as_slice(),
                        );
                        if let Err(err) = votingengine::Pallet::<T>::snapshot_role_voters(
                            id,
                            subject.clone(),
                            voters,
                        ) {
                            return TransactionOutcome::Rollback(Err(err));
                        }
                    }
                }
                if let Some(scope) = population_scope.as_ref() {
                    match votingengine::Pallet::<T>::create_population_snapshot(id, scope) {
                        Ok(0) => {
                            return TransactionOutcome::Rollback(Err(
                                Error::<T>::EmptyVoterSnapshot.into(),
                            ))
                        }
                        Ok(_) => {}
                        Err(err) => return TransactionOutcome::Rollback(Err(err)),
                    }
                }
                // VotePlan 必须先于 ProposalOwner 绑定；随后登记业务数据时再校验 owner 一致。
                if let Err(err) = votingengine::Pallet::<T>::bind_vote_plan(id, vote_plan) {
                    return TransactionOutcome::Rollback(Err(err));
                }
                if let Err(err) = votingengine::Pallet::<T>::register_proposal_data(
                    id,
                    proposal_owner.as_slice(),
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
                    target_cid_number,
                    seat_count,
                });
                TransactionOutcome::Commit(Ok(id))
            });
            result
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
            if expected_stage == votingengine::STAGE_ELECTION_POPULAR {
                ensure!(
                    votingengine::Pallet::<T>::can_vote_at_population_snapshot(proposal_id, &who),
                    Error::<T>::VoterNotEligible
                );
            } else {
                ensure!(
                    votingengine::Pallet::<T>::is_effective_voter_in_snapshot(
                        proposal_id,
                        votingengine::types::ProposalSubject::InstitutionCid(
                            ElectionMetaStore::<T>::get(proposal_id)
                                .ok_or(Error::<T>::ElectionMetaMissing)?
                                .target_cid_number,
                        ),
                        &who,
                    ),
                    Error::<T>::VoterNotInSnapshot
                );
            }
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

            let eligible_total = if expected_stage == votingengine::STAGE_ELECTION_POPULAR {
                votingengine::Pallet::<T>::population_eligible_total_of(proposal_id)
                    .ok_or(Error::<T>::EmptyVoterSnapshot)?
            } else {
                let target = ElectionMetaStore::<T>::get(proposal_id)
                    .ok_or(Error::<T>::ElectionMetaMissing)?
                    .target_cid_number;
                u64::from(
                    votingengine::Pallet::<T>::effective_voters_len(
                        proposal_id,
                        votingengine::types::ProposalSubject::InstitutionCid(target),
                    )
                    .ok_or(Error::<T>::EmptyVoterSnapshot)?,
                )
            };
            if u64::from(tally.casted) >= eligible_total {
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
        // 这里只确认投票引擎已形成完整结果快照。候选资格、职位、席位、任期和
        // 目标机构都属于 election-campaign 的业务规则，未经业务复核不得写 entity。
        Ok(votingengine::ProposalExecutionOutcome::Executed)
    }
}
