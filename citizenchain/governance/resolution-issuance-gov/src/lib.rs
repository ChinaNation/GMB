#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::DispatchResult;
pub use pallet::*;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod weights;
use voting_engine_system::JointVoteResultCallback;

/// 模块标识前缀，用于在 ProposalData 中区分不同业务模块。
pub const MODULE_TAG: &[u8] = b"res-iss";

#[frame_support::pallet]
pub mod pallet {
    use crate::weights::WeightInfo;
    use codec::{Decode, Encode};
    use frame_support::{
        pallet_prelude::*,
        storage::{with_transaction, TransactionOutcome},
        weights::Weight,
    };
    use frame_system::pallet_prelude::*;
    use primitives::china::china_cb::CHINA_CB;
    use resolution_issuance_iss::{
        weights::WeightInfo as IssuanceWeightInfoT, ResolutionAllocationsOf,
        ResolutionIssuanceExecutor, ResolutionReasonOf,
    };
    use sp_std::{collections::btree_set::BTreeSet, vec::Vec};
    use voting_engine_system::JointVoteEngine;

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(3);

    pub type ReasonOf<T> = BoundedVec<u8, <T as Config>::MaxReasonLen>;
    pub type AllocationOf<T> = BoundedVec<
        RecipientAmount<<T as frame_system::Config>::AccountId>,
        <T as Config>::MaxAllocations,
    >;
    pub type SnapshotNonceOf<T> = BoundedVec<u8, <T as Config>::MaxSnapshotNonceLength>;
    pub type SnapshotSignatureOf<T> = BoundedVec<u8, <T as Config>::MaxSnapshotSignatureLength>;

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
    pub struct RecipientAmount<AccountId> {
        pub recipient: AccountId,
        pub amount: u128,
    }

    /// 存入 voting engine ProposalData 的业务数据结构。
    #[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
    pub struct IssuanceProposalData<AccountId> {
        pub proposer: AccountId,
        pub reason: Vec<u8>,
        pub total_amount: u128,
        pub allocations: Vec<RecipientAmount<AccountId>>,
    }

    pub(crate) enum FinalizeOutcome {
        ApprovedExecutionSucceeded,
        ApprovedExecutionFailed,
        Rejected,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config + voting_engine_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 仅允许国储会管理员发起提案。
        type NrcProposeOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;
        /// 更新合法收款账户集合。
        type RecipientSetOrigin: EnsureOrigin<Self::RuntimeOrigin>;
        /// 回放联合投票结果的受限来源（生产可配置为拒绝所有外部来源）。
        type JointVoteFinalizeOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// 投票通过后，调用发行执行模块执行铸币。
        type IssuanceExecutor: ResolutionIssuanceExecutor<
            Self::AccountId,
            u128,
            Self::MaxReasonLen,
            Self::MaxAllocations,
        >;
        /// 用于估算发行执行路径的 weight。
        type IssuanceWeightInfo: IssuanceWeightInfoT;
        type JointVoteEngine: JointVoteEngine<Self::AccountId>;

        #[pallet::constant]
        type MaxReasonLen: Get<u32>;

        #[pallet::constant]
        type MaxAllocations: Get<u32>;

        #[pallet::constant]
        type MaxSnapshotNonceLength: Get<u32>;

        #[pallet::constant]
        type MaxSnapshotSignatureLength: Get<u32>;

        /// 本 pallet 的 weight 配置。
        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    // ────── 已删除的旧存储 ──────
    // NextProposalId    → 改用 voting engine 的 NextProposalId
    // Proposals         → 改用 voting engine 的 ProposalData
    // GovToJointVote    → 不再需要（统一 ID）
    // JointVoteToGov    → 不再需要（统一 ID）
    // RetryCount        → 移除重试逻辑

    /// 合法收款账户集合（链上可更新）。
    #[pallet::storage]
    #[pallet::getter(fn allowed_recipients)]
    pub type AllowedRecipients<T: Config> =
        StorageValue<_, BoundedVec<T::AccountId, T::MaxAllocations>, ValueQuery>;

    /// 当前处于 Voting 状态的提案数量，用于阻止治理中途切换收款集合。
    #[pallet::storage]
    #[pallet::getter(fn voting_proposal_count)]
    pub type VotingProposalCount<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub allowed_recipients: Vec<T::AccountId>,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            let allowed_recipients = CHINA_CB
                .iter()
                .skip(1)
                .map(|node| {
                    T::AccountId::decode(&mut &node.duoqian_address[..])
                        .expect("CHINA_CB duoqian_address must decode to AccountId")
                })
                .collect();
            Self { allowed_recipients }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            let bounded: BoundedVec<T::AccountId, T::MaxAllocations> = self
                .allowed_recipients
                .clone()
                .try_into()
                .expect("allowed_recipients must fit MaxAllocations");
            Pallet::<T>::ensure_unique_recipients(bounded.as_slice())
                .expect("allowed_recipients must not contain duplicates");
            AllowedRecipients::<T>::put(bounded);
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> Weight {
            let db = T::DbWeight::get();
            let on_chain = StorageVersion::get::<Pallet<T>>();
            if on_chain >= STORAGE_VERSION {
                return db.reads(1);
            }

            let mut reads = 1u64;
            let mut writes = 0u64;

            if on_chain < StorageVersion::new(1) {
                reads = reads.saturating_add(1);
                if AllowedRecipients::<T>::get().is_empty() {
                    if let Some(defaults) = Self::decode_default_allowed_recipients() {
                        AllowedRecipients::<T>::put(defaults);
                        writes = writes.saturating_add(1);
                    }
                }
            }

            if on_chain < StorageVersion::new(2) {
                reads = reads.saturating_add(1);
                let current_allowed = AllowedRecipients::<T>::get();
                if Self::ensure_unique_recipients(current_allowed.as_slice()).is_err() {
                    if let Some(defaults) = Self::decode_default_allowed_recipients() {
                        AllowedRecipients::<T>::put(defaults);
                        writes = writes.saturating_add(1);
                    }
                }
            }

            // v3: 旧存储（NextProposalId / Proposals / GovToJointVote / JointVoteToGov / RetryCount）
            // 已在链上无数据（预启动链），无需迁移。

            STORAGE_VERSION.put::<Pallet<T>>();
            writes = writes.saturating_add(1);
            db.reads_writes(reads, writes)
        }

        #[cfg(feature = "std")]
        fn integrity_test() {
            assert!(
                (CHINA_CB.len() as u32).saturating_sub(1) <= T::MaxAllocations::get(),
                "MaxAllocations must cover CHINA_CB recipients"
            );
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ResolutionIssuanceProposed {
            proposal_id: u64,
            proposer: T::AccountId,
            total_amount: u128,
            allocation_count: u32,
        },
        JointVoteFinalized {
            proposal_id: u64,
            approved: bool,
        },
        IssuanceExecutionTriggered {
            proposal_id: u64,
            total_amount: u128,
        },
        IssuanceExecutionFailed {
            proposal_id: u64,
        },
        AllowedRecipientsUpdated {
            count: u32,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        EmptyReason,
        EmptyAllocations,
        InvalidAllocationCount,
        DuplicateRecipient,
        InvalidRecipientSet,
        ZeroAmount,
        AllocationOverflow,
        TotalMismatch,
        ProposalNotFound,
        ProposalNotVoting,
        JointVoteCreateFailed,
        RecipientsNotConfigured,
        DuplicateAllowedRecipient,
        ActiveVotingProposalsExist,
        VotingProposalCountOverflow,
        VotingProposalCountUnderflow,
        ProposalDataStoreFailed,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 国储会提案：创建"决议发行"联合投票提案。
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::propose_resolution_issuance())]
        pub fn propose_resolution_issuance(
            origin: OriginFor<T>,
            reason: ReasonOf<T>,
            total_amount: u128,
            allocations: AllocationOf<T>,
            eligible_total: u64,
            snapshot_nonce: SnapshotNonceOf<T>,
            snapshot_signature: SnapshotSignatureOf<T>,
        ) -> DispatchResult {
            let proposer = T::NrcProposeOrigin::ensure_origin(origin)?;

            ensure!(!reason.is_empty(), Error::<T>::EmptyReason);
            Self::validate_allocations(total_amount, allocations.as_slice())?;

            // 中文注释：联合投票提案创建与业务数据写入必须原子执行；
            // 否则一旦后续写入失败，就会留下孤儿 proposal。
            with_transaction(|| {
                let proposal_id = match T::JointVoteEngine::create_joint_proposal(
                    proposer.clone(),
                    eligible_total,
                    snapshot_nonce.as_slice(),
                    snapshot_signature.as_slice(),
                ) {
                    Ok(id) => id,
                    Err(_) => {
                        return TransactionOutcome::Rollback(Err(
                            Error::<T>::JointVoteCreateFailed.into()
                        ))
                    }
                };

                // 将业务数据编码后存入投票引擎统一存储
                let data = IssuanceProposalData {
                    proposer: proposer.clone(),
                    reason: reason.to_vec(),
                    total_amount,
                    allocations: allocations.to_vec(),
                };
                let mut encoded = Vec::from(crate::MODULE_TAG);
                encoded.extend_from_slice(&data.encode());
                if voting_engine_system::Pallet::<T>::store_proposal_data(proposal_id, encoded)
                    .is_err()
                {
                    return TransactionOutcome::Rollback(Err(
                        Error::<T>::ProposalDataStoreFailed.into()
                    ));
                }
                let now = frame_system::Pallet::<T>::block_number();
                voting_engine_system::Pallet::<T>::store_proposal_meta(proposal_id, now);

                if let Err(err) = Self::increment_voting_proposal_count() {
                    return TransactionOutcome::Rollback(Err(err));
                }

                Self::deposit_event(Event::<T>::ResolutionIssuanceProposed {
                    proposal_id,
                    proposer,
                    total_amount,
                    allocation_count: allocations.len() as u32,
                });
                TransactionOutcome::Commit(Ok(()))
            })
        }

        /// 联合投票回调：仅接受联合投票引擎/治理权限来源。
        /// approved=true 时，触发 execution pallet 执行发行。
        #[pallet::call_index(1)]
        #[pallet::weight(if *approved {
            <T as Config>::WeightInfo::finalize_joint_vote_approved()
        } else {
            <T as Config>::WeightInfo::finalize_joint_vote_rejected()
        })]
        pub fn finalize_joint_vote(
            origin: OriginFor<T>,
            proposal_id: u64,
            approved: bool,
        ) -> DispatchResultWithPostInfo {
            T::JointVoteFinalizeOrigin::ensure_origin(origin)?;
            let outcome = Self::apply_joint_vote_result(proposal_id, approved)?;
            let actual = match outcome {
                FinalizeOutcome::ApprovedExecutionSucceeded => None,
                FinalizeOutcome::ApprovedExecutionFailed => {
                    Some(T::DbWeight::get().reads_writes(3, 5))
                }
                FinalizeOutcome::Rejected => Some(T::DbWeight::get().reads_writes(3, 4)),
            };
            Ok(actual.into())
        }

        /// 更新链上合法收款账户集合。
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::set_allowed_recipients())]
        pub fn set_allowed_recipients(
            origin: OriginFor<T>,
            recipients: BoundedVec<T::AccountId, T::MaxAllocations>,
        ) -> DispatchResult {
            T::RecipientSetOrigin::ensure_origin(origin)?;
            ensure!(!recipients.is_empty(), Error::<T>::RecipientsNotConfigured);
            // 中文注释：只要还有 Voting 中的提案，就禁止切换合法收款集合，
            // 否则同一提案在投票前后的收款口径可能不一致。
            ensure!(
                VotingProposalCount::<T>::get() == 0,
                Error::<T>::ActiveVotingProposalsExist
            );
            Self::ensure_unique_recipients(recipients.as_slice())?;
            AllowedRecipients::<T>::put(recipients.clone());
            Self::deposit_event(Event::<T>::AllowedRecipientsUpdated {
                count: recipients.len() as u32,
            });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// 从投票引擎的 ProposalData 中读取并解码本模块的业务数据。
        pub fn load_proposal_data(
            proposal_id: u64,
        ) -> Option<IssuanceProposalData<T::AccountId>> {
            let raw = voting_engine_system::Pallet::<T>::get_proposal_data(proposal_id)?;
            Self::decode_tagged_data(&raw)
        }

        /// 判断指定提案是否属于本模块（检查 MODULE_TAG 前缀）。
        pub fn owns_proposal(proposal_id: u64) -> bool {
            voting_engine_system::Pallet::<T>::get_proposal_data(proposal_id)
                .map(|raw| raw.starts_with(crate::MODULE_TAG))
                .unwrap_or(false)
        }

        fn decode_tagged_data(raw: &[u8]) -> Option<IssuanceProposalData<T::AccountId>> {
            let tag = crate::MODULE_TAG;
            if raw.len() < tag.len() || &raw[..tag.len()] != tag {
                return None;
            }
            IssuanceProposalData::decode(&mut &raw[tag.len()..]).ok()
        }

        pub(crate) fn apply_joint_vote_result(
            proposal_id: u64,
            approved: bool,
        ) -> Result<FinalizeOutcome, DispatchError> {
            // 中文注释：联合投票终结、发行执行和计数变更必须在同一事务里提交。
            with_transaction(|| {
                let data = match Self::load_proposal_data(proposal_id) {
                    Some(data) => data,
                    None => {
                        return TransactionOutcome::Rollback(Err(
                            Error::<T>::ProposalNotFound.into()
                        ))
                    }
                };

                if approved {
                    let execute_reason: ReasonOf<T> = match data
                        .reason
                        .clone()
                        .try_into()
                    {
                        Ok(v) => v,
                        Err(_) => {
                            return TransactionOutcome::Rollback(Err(
                                Error::<T>::ProposalNotFound.into()
                            ))
                        }
                    };
                    let execute_allocations_raw: AllocationOf<T> = match data
                        .allocations
                        .clone()
                        .try_into()
                    {
                        Ok(v) => v,
                        Err(_) => {
                            return TransactionOutcome::Rollback(Err(
                                Error::<T>::ProposalNotFound.into()
                            ))
                        }
                    };
                    let (execute_reason, execute_allocations) =
                        match Self::issuance_payload(&execute_reason, &execute_allocations_raw) {
                            Ok(v) => v,
                            Err(e) => return TransactionOutcome::Rollback(Err(e)),
                        };

                    if T::IssuanceExecutor::execute_resolution_issuance(
                        proposal_id,
                        execute_reason,
                        data.total_amount,
                        execute_allocations,
                    )
                    .is_ok()
                    {
                        T::JointVoteEngine::cleanup_joint_proposal(proposal_id);
                        if let Err(err) = Self::decrement_voting_proposal_count() {
                            return TransactionOutcome::Rollback(Err(err));
                        }
                        Self::deposit_event(Event::<T>::JointVoteFinalized {
                            proposal_id,
                            approved: true,
                        });
                        Self::deposit_event(Event::<T>::IssuanceExecutionTriggered {
                            proposal_id,
                            total_amount: data.total_amount,
                        });
                        return TransactionOutcome::Commit(Ok(
                            FinalizeOutcome::ApprovedExecutionSucceeded,
                        ));
                    }

                    // 执行失败：不再有重试逻辑，仅发出失败事件
                    T::JointVoteEngine::cleanup_joint_proposal(proposal_id);
                    if let Err(err) = Self::decrement_voting_proposal_count() {
                        return TransactionOutcome::Rollback(Err(err));
                    }
                    Self::deposit_event(Event::<T>::JointVoteFinalized {
                        proposal_id,
                        approved: true,
                    });
                    Self::deposit_event(Event::<T>::IssuanceExecutionFailed { proposal_id });
                    return TransactionOutcome::Commit(Ok(
                        FinalizeOutcome::ApprovedExecutionFailed,
                    ));
                }

                // 否决
                T::JointVoteEngine::cleanup_joint_proposal(proposal_id);
                if let Err(err) = Self::decrement_voting_proposal_count() {
                    return TransactionOutcome::Rollback(Err(err));
                }
                Self::deposit_event(Event::<T>::JointVoteFinalized {
                    proposal_id,
                    approved: false,
                });
                TransactionOutcome::Commit(Ok(FinalizeOutcome::Rejected))
            })
        }

        fn validate_allocations(
            total_amount: u128,
            allocations: &[RecipientAmount<T::AccountId>],
        ) -> DispatchResult {
            ensure!(!allocations.is_empty(), Error::<T>::EmptyAllocations);
            ensure!(total_amount > 0, Error::<T>::ZeroAmount);
            let expected = AllowedRecipients::<T>::get();
            ensure!(!expected.is_empty(), Error::<T>::RecipientsNotConfigured);
            // 中文注释：治理提案里的收款人集合必须与链上白名单完全一致，
            // 既不能缺人，也不能额外塞入其他账户。
            let expected_set: BTreeSet<&T::AccountId> = expected.iter().collect();
            ensure!(
                expected_set.len() == expected.len(),
                Error::<T>::DuplicateAllowedRecipient
            );
            ensure!(
                allocations.len() == expected_set.len(),
                Error::<T>::InvalidAllocationCount
            );
            let mut seen: BTreeSet<&T::AccountId> = BTreeSet::new();

            let mut sum = 0u128;
            for item in allocations {
                ensure!(item.amount > 0, Error::<T>::ZeroAmount);
                ensure!(seen.insert(&item.recipient), Error::<T>::DuplicateRecipient);
                ensure!(
                    expected_set.contains(&item.recipient),
                    Error::<T>::InvalidRecipientSet
                );
                sum = sum
                    .checked_add(item.amount)
                    .ok_or(Error::<T>::AllocationOverflow)?;
            }

            // 防御性校验：正常流程在上面的长度/成员约束下已可推出相等，这里保留用于防回归。
            ensure!(seen == expected_set, Error::<T>::InvalidRecipientSet);
            ensure!(sum == total_amount, Error::<T>::TotalMismatch);
            Ok(())
        }

        fn issuance_payload(
            reason: &ReasonOf<T>,
            allocations: &AllocationOf<T>,
        ) -> Result<
            (
                ResolutionReasonOf<T::MaxReasonLen>,
                ResolutionAllocationsOf<T::AccountId, u128, T::MaxAllocations>,
            ),
            DispatchError,
        > {
            let execute_reason: ResolutionReasonOf<T::MaxReasonLen> = reason.clone();
            let execute_allocations: ResolutionAllocationsOf<
                T::AccountId,
                u128,
                T::MaxAllocations,
            > = allocations
                .iter()
                .map(|x| (x.recipient.clone(), x.amount))
                .collect::<Vec<_>>()
                .try_into()
                .map_err(|_| Error::<T>::InvalidAllocationCount)?;
            Ok((execute_reason, execute_allocations))
        }

        fn ensure_unique_recipients(recipients: &[T::AccountId]) -> DispatchResult {
            let mut seen: BTreeSet<&T::AccountId> = BTreeSet::new();
            for recipient in recipients {
                ensure!(seen.insert(recipient), Error::<T>::DuplicateAllowedRecipient);
            }
            Ok(())
        }

        fn decode_default_allowed_recipients() -> Option<BoundedVec<T::AccountId, T::MaxAllocations>>
        {
            let recipients: Vec<T::AccountId> = CHINA_CB
                .iter()
                .skip(1)
                .filter_map(|node| T::AccountId::decode(&mut &node.duoqian_address[..]).ok())
                .collect();
            if recipients.is_empty() {
                return None;
            }
            let bounded: BoundedVec<T::AccountId, T::MaxAllocations> =
                recipients.try_into().ok()?;
            if Self::ensure_unique_recipients(bounded.as_slice()).is_err() {
                return None;
            }
            Some(bounded)
        }

        fn increment_voting_proposal_count() -> DispatchResult {
            VotingProposalCount::<T>::try_mutate(|count| -> DispatchResult {
                *count = count
                    .checked_add(1)
                    .ok_or(Error::<T>::VotingProposalCountOverflow)?;
                Ok(())
            })
        }

        fn decrement_voting_proposal_count() -> DispatchResult {
            VotingProposalCount::<T>::try_mutate(|count| -> DispatchResult {
                *count = count
                    .checked_sub(1)
                    .ok_or(Error::<T>::VotingProposalCountUnderflow)?;
                Ok(())
            })
        }
    }
}

impl<T: pallet::Config> JointVoteResultCallback for pallet::Pallet<T> {
    fn on_joint_vote_finalized(vote_proposal_id: u64, approved: bool) -> DispatchResult {
        // 统一 ID：vote_proposal_id 就是唯一的提案 ID，无需转换。
        pallet::Pallet::<T>::apply_joint_vote_result(vote_proposal_id, approved).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::RefCell;
    use frame_support::{assert_noop, assert_ok, derive_impl, traits::ConstU32, BoundedVec};
    use frame_system as system;
    use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage, DispatchError};

    type Block = frame_system::mocking::MockBlock<Test>;

    #[frame_support::runtime]
    mod runtime {
        #[runtime::runtime]
        #[runtime::derive(
            RuntimeCall,
            RuntimeEvent,
            RuntimeError,
            RuntimeOrigin,
            RuntimeFreezeReason,
            RuntimeHoldReason,
            RuntimeSlashReason,
            RuntimeLockId,
            RuntimeTask,
            RuntimeViewFunction
        )]
        pub struct Test;

        #[runtime::pallet_index(0)]
        pub type System = frame_system;

        #[runtime::pallet_index(1)]
        pub type VotingEngine = voting_engine_system;

        #[runtime::pallet_index(2)]
        pub type ResolutionIssuanceGov = super;
    }

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl system::Config for Test {
        type Block = Block;
        type AccountId = AccountId32;
        type Lookup = IdentityLookup<Self::AccountId>;
    }

    pub struct EnsureNrcAdminForTest;
    impl frame_support::traits::EnsureOrigin<RuntimeOrigin> for EnsureNrcAdminForTest {
        type Success = AccountId32;

        fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
            let who = frame_system::EnsureSigned::<AccountId32>::try_origin(o)?;
            if who == AccountId32::new([1u8; 32]) {
                Ok(who)
            } else {
                Err(RuntimeOrigin::from(frame_system::RawOrigin::Signed(who)))
            }
        }

        #[cfg(feature = "runtime-benchmarks")]
        fn try_successful_origin() -> Result<RuntimeOrigin, ()> {
            Ok(RuntimeOrigin::signed(AccountId32::new([1u8; 32])))
        }
    }

    thread_local! {
        static NEXT_JOINT_ID: RefCell<u64> = const { RefCell::new(100) };
        static EXEC_CALLS: RefCell<Vec<(u64, u128, usize)>> = const { RefCell::new(Vec::new()) };
        static EXEC_SHOULD_FAIL: RefCell<bool> = const { RefCell::new(false) };
    }

    pub struct TestJointVoteEngine;
    impl voting_engine_system::JointVoteEngine<AccountId32> for TestJointVoteEngine {
        fn create_joint_proposal(
            _who: AccountId32,
            eligible_total: u64,
            snapshot_nonce: &[u8],
            snapshot_signature: &[u8],
        ) -> Result<u64, DispatchError> {
            if eligible_total == 0 || snapshot_nonce.is_empty() || snapshot_signature.is_empty() {
                return Err(DispatchError::Other("bad snapshot"));
            }
            NEXT_JOINT_ID.with(|id| {
                let mut id = id.borrow_mut();
                let v = *id;
                *id = id.saturating_add(1);
                Ok(v)
            })
        }
    }

    pub struct TestIssuanceExecutor;
    impl
        resolution_issuance_iss::ResolutionIssuanceExecutor<
            AccountId32,
            u128,
            ConstU32<128>,
            ConstU32<64>,
        > for TestIssuanceExecutor
    {
        fn execute_resolution_issuance(
            proposal_id: u64,
            _reason: resolution_issuance_iss::ResolutionReasonOf<ConstU32<128>>,
            total_amount: u128,
            allocations: resolution_issuance_iss::ResolutionAllocationsOf<
                AccountId32,
                u128,
                ConstU32<64>,
            >,
        ) -> DispatchResult {
            let should_fail = EXEC_SHOULD_FAIL.with(|v| *v.borrow());
            if should_fail {
                return Err(DispatchError::Other("exec failed"));
            }
            EXEC_CALLS.with(|calls| {
                calls
                    .borrow_mut()
                    .push((proposal_id, total_amount, allocations.len()));
            });
            Ok(())
        }
    }

    // Minimal voting engine config for tests
    pub struct TestSfidEligibility;
    impl voting_engine_system::SfidEligibility<AccountId32, <Test as frame_system::Config>::Hash>
        for TestSfidEligibility
    {
        fn is_eligible(
            _sfid_hash: &<Test as frame_system::Config>::Hash,
            _who: &AccountId32,
        ) -> bool {
            true
        }

        fn verify_and_consume_vote_credential(
            _sfid_hash: &<Test as frame_system::Config>::Hash,
            _who: &AccountId32,
            _proposal_id: u64,
            _nonce: &[u8],
            _signature: &[u8],
        ) -> bool {
            true
        }
    }

    pub struct TestPopulationSnapshotVerifier;
    impl
        voting_engine_system::PopulationSnapshotVerifier<
            AccountId32,
            voting_engine_system::pallet::VoteNonceOf<Test>,
            voting_engine_system::pallet::VoteSignatureOf<Test>,
        > for TestPopulationSnapshotVerifier
    {
        fn verify_population_snapshot(
            _who: &AccountId32,
            _eligible_total: u64,
            _nonce: &voting_engine_system::pallet::VoteNonceOf<Test>,
            _signature: &voting_engine_system::pallet::VoteSignatureOf<Test>,
        ) -> bool {
            true
        }
    }

    pub struct TestTimeProvider;
    impl frame_support::traits::UnixTime for TestTimeProvider {
        fn now() -> core::time::Duration {
            core::time::Duration::from_secs(1_782_864_000) // 2026-07-01
        }
    }

    impl voting_engine_system::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type MaxVoteNonceLength = ConstU32<64>;
        type MaxVoteSignatureLength = ConstU32<64>;
        type MaxAutoFinalizePerBlock = ConstU32<64>;
        type MaxProposalsPerExpiry = ConstU32<128>;
        type MaxCleanupStepsPerBlock = ConstU32<8>;
        type CleanupKeysPerStep = ConstU32<64>;
        type MaxProposalDataLen = ConstU32<8192>;
        type MaxJointDecisionApprovals = ConstU32<32>;
        type SfidEligibility = TestSfidEligibility;
        type PopulationSnapshotVerifier = TestPopulationSnapshotVerifier;
        type JointVoteResultCallback = ();
        type InternalAdminProvider = ();
        type InternalThresholdProvider = ();
        type JointInstitutionDecisionVerifier = ();
        type TimeProvider = TestTimeProvider;
        type WeightInfo = ();
    }

    impl pallet::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type NrcProposeOrigin = EnsureNrcAdminForTest;
        type RecipientSetOrigin = frame_system::EnsureRoot<AccountId32>;
        type JointVoteFinalizeOrigin = frame_system::EnsureRoot<AccountId32>;
        type IssuanceExecutor = TestIssuanceExecutor;
        type IssuanceWeightInfo = ();
        type WeightInfo = crate::weights::SubstrateWeight<Test>;
        type JointVoteEngine = TestJointVoteEngine;
        type MaxReasonLen = ConstU32<128>;
        type MaxAllocations = ConstU32<64>;
        type MaxSnapshotNonceLength = ConstU32<64>;
        type MaxSnapshotSignatureLength = ConstU32<64>;
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("test storage should build");
        let mut ext: sp_io::TestExternalities = storage.into();
        ext.execute_with(|| {
            EXEC_CALLS.with(|c| c.borrow_mut().clear());
            EXEC_SHOULD_FAIL.with(|v| *v.borrow_mut() = false);
            NEXT_JOINT_ID.with(|id| *id.borrow_mut() = 100);
            let recipients = reserve_council_accounts();
            let bounded: BoundedVec<AccountId32, ConstU32<64>> =
                recipients.try_into().expect("recipients should fit");
            pallet::AllowedRecipients::<Test>::put(bounded);
        });
        ext
    }

    fn reason_ok() -> pallet::ReasonOf<Test> {
        b"issuance".to_vec().try_into().expect("reason should fit")
    }

    fn nonce_ok() -> pallet::SnapshotNonceOf<Test> {
        b"snap-nonce".to_vec().try_into().expect("nonce should fit")
    }

    fn sig_ok() -> pallet::SnapshotSignatureOf<Test> {
        b"snap-signature"
            .to_vec()
            .try_into()
            .expect("signature should fit")
    }

    fn reserve_council_accounts() -> Vec<AccountId32> {
        primitives::china::china_cb::CHINA_CB
            .iter()
            .skip(1)
            .map(|n| AccountId32::new(n.duoqian_address))
            .collect()
    }

    fn allocations_ok(total: u128) -> pallet::AllocationOf<Test> {
        let recipients = reserve_council_accounts();
        let count = recipients.len() as u128;
        let per = total / count;
        let mut left = total;
        let mut v = Vec::new();
        for (i, recipient) in recipients.into_iter().enumerate() {
            let amount = if i + 1 == count as usize { left } else { per };
            left = left.saturating_sub(amount);
            v.push(pallet::RecipientAmount { recipient, amount });
        }
        v.try_into().expect("allocations should fit")
    }

    #[test]
    fn only_nrc_admin_can_propose() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                ResolutionIssuanceGov::propose_resolution_issuance(
                    RuntimeOrigin::signed(AccountId32::new([2u8; 32])),
                    reason_ok(),
                    1000,
                    allocations_ok(1000),
                    10,
                    nonce_ok(),
                    sig_ok()
                ),
                sp_runtime::DispatchError::BadOrigin
            );
        });
    }

    #[test]
    fn reject_invalid_allocation_count() {
        new_test_ext().execute_with(|| {
            let one = vec![pallet::RecipientAmount {
                recipient: reserve_council_accounts()[0].clone(),
                amount: 1000,
            }];
            let alloc: pallet::AllocationOf<Test> = one.try_into().expect("should fit");
            assert_noop!(
                ResolutionIssuanceGov::propose_resolution_issuance(
                    RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                    reason_ok(),
                    1000,
                    alloc,
                    10,
                    nonce_ok(),
                    sig_ok()
                ),
                pallet::Error::<Test>::InvalidAllocationCount
            );
        });
    }

    #[test]
    fn reject_empty_reason() {
        new_test_ext().execute_with(|| {
            let reason: pallet::ReasonOf<Test> = Vec::<u8>::new().try_into().expect("should fit");
            assert_noop!(
                ResolutionIssuanceGov::propose_resolution_issuance(
                    RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                    reason,
                    1000,
                    allocations_ok(1000),
                    10,
                    nonce_ok(),
                    sig_ok()
                ),
                pallet::Error::<Test>::EmptyReason
            );
        });
    }

    #[test]
    fn reject_zero_amount_allocation() {
        new_test_ext().execute_with(|| {
            let mut raw = allocations_ok(1000).into_inner();
            raw[0].amount = 0;
            let alloc: pallet::AllocationOf<Test> = raw.try_into().expect("should fit");
            assert_noop!(
                ResolutionIssuanceGov::propose_resolution_issuance(
                    RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                    reason_ok(),
                    1000,
                    alloc,
                    10,
                    nonce_ok(),
                    sig_ok()
                ),
                pallet::Error::<Test>::ZeroAmount
            );
        });
    }

    #[test]
    fn reject_duplicate_recipient_allocation() {
        new_test_ext().execute_with(|| {
            let recipients = reserve_council_accounts();
            let mut raw: Vec<pallet::RecipientAmount<AccountId32>> = recipients
                .iter()
                .cloned()
                .map(|recipient| pallet::RecipientAmount {
                    recipient,
                    amount: 1u128,
                })
                .collect();
            let last = raw.len().saturating_sub(1);
            raw[last].recipient = raw[0].recipient.clone();
            let alloc: pallet::AllocationOf<Test> = raw.try_into().expect("should fit");
            assert_noop!(
                ResolutionIssuanceGov::propose_resolution_issuance(
                    RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                    reason_ok(),
                    recipients.len() as u128,
                    alloc,
                    10,
                    nonce_ok(),
                    sig_ok()
                ),
                pallet::Error::<Test>::DuplicateRecipient
            );
        });
    }

    #[test]
    fn reject_total_mismatch() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                ResolutionIssuanceGov::propose_resolution_issuance(
                    RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                    reason_ok(),
                    999,
                    allocations_ok(1000),
                    10,
                    nonce_ok(),
                    sig_ok()
                ),
                pallet::Error::<Test>::TotalMismatch
            );
        });
    }

    #[test]
    fn approved_callback_executes_issuance() {
        new_test_ext().execute_with(|| {
            assert_ok!(ResolutionIssuanceGov::propose_resolution_issuance(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                reason_ok(),
                1000,
                allocations_ok(1000),
                10,
                nonce_ok(),
                sig_ok()
            ));

            // proposal_id == vote_proposal_id == 100 (from TestJointVoteEngine)
            assert_ok!(ResolutionIssuanceGov::on_joint_vote_finalized(100, true));

            let calls = EXEC_CALLS.with(|c| c.borrow().clone());
            assert_eq!(calls.len(), 1);
            assert_eq!(calls[0].0, 100);
            assert_eq!(calls[0].1, 1000);
            assert_eq!(calls[0].2, reserve_council_accounts().len());
        });
    }

    #[test]
    fn propose_rolls_back_when_voting_count_overflows() {
        new_test_ext().execute_with(|| {
            pallet::VotingProposalCount::<Test>::put(u32::MAX);

            assert_noop!(
                ResolutionIssuanceGov::propose_resolution_issuance(
                    RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                    reason_ok(),
                    1000,
                    allocations_ok(1000),
                    10,
                    nonce_ok(),
                    sig_ok()
                ),
                pallet::Error::<Test>::VotingProposalCountOverflow
            );
        });
    }

    #[test]
    fn finalize_rejects_missing_proposal() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                ResolutionIssuanceGov::finalize_joint_vote(RuntimeOrigin::root(), 99, true),
                pallet::Error::<Test>::ProposalNotFound
            );
        });
    }

    #[test]
    fn rejected_callback_marks_rejected() {
        new_test_ext().execute_with(|| {
            assert_ok!(ResolutionIssuanceGov::propose_resolution_issuance(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                reason_ok(),
                1000,
                allocations_ok(1000),
                10,
                nonce_ok(),
                sig_ok()
            ));

            // proposal_id == 100
            assert_ok!(ResolutionIssuanceGov::on_joint_vote_finalized(100, false));
            // VotingProposalCount should be decremented
            assert_eq!(pallet::VotingProposalCount::<Test>::get(), 0);
        });
    }

    #[test]
    fn finalize_rolls_back_when_post_execution_accounting_fails() {
        new_test_ext().execute_with(|| {
            assert_ok!(ResolutionIssuanceGov::propose_resolution_issuance(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                reason_ok(),
                1000,
                allocations_ok(1000),
                10,
                nonce_ok(),
                sig_ok()
            ));
            pallet::VotingProposalCount::<Test>::put(0);

            assert_noop!(
                ResolutionIssuanceGov::finalize_joint_vote(RuntimeOrigin::root(), 100, true),
                pallet::Error::<Test>::VotingProposalCountUnderflow
            );

            // ProposalData should still exist (transaction rolled back)
            assert!(pallet::Pallet::<Test>::load_proposal_data(100).is_some());
        });
    }

    #[test]
    fn approved_callback_execution_failure_emits_failure_event() {
        new_test_ext().execute_with(|| {
            assert_ok!(ResolutionIssuanceGov::propose_resolution_issuance(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                reason_ok(),
                1000,
                allocations_ok(1000),
                10,
                nonce_ok(),
                sig_ok()
            ));

            EXEC_SHOULD_FAIL.with(|v| *v.borrow_mut() = true);
            assert_ok!(ResolutionIssuanceGov::on_joint_vote_finalized(100, true));

            let calls = EXEC_CALLS.with(|c| c.borrow().clone());
            assert_eq!(calls.len(), 0);
            // VotingProposalCount should be decremented even on execution failure
            assert_eq!(pallet::VotingProposalCount::<Test>::get(), 0);
        });
    }

    #[test]
    fn owns_proposal_returns_true_for_issuance_proposals() {
        new_test_ext().execute_with(|| {
            assert_ok!(ResolutionIssuanceGov::propose_resolution_issuance(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                reason_ok(),
                1000,
                allocations_ok(1000),
                10,
                nonce_ok(),
                sig_ok()
            ));
            assert!(pallet::Pallet::<Test>::owns_proposal(100));
            assert!(!pallet::Pallet::<Test>::owns_proposal(999));
        });
    }

    #[test]
    fn set_allowed_recipients_rejected_when_voting_exists() {
        new_test_ext().execute_with(|| {
            assert_ok!(ResolutionIssuanceGov::propose_resolution_issuance(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                reason_ok(),
                1000,
                allocations_ok(1000),
                10,
                nonce_ok(),
                sig_ok()
            ));
            let recipients: BoundedVec<AccountId32, ConstU32<64>> = reserve_council_accounts()
                .try_into()
                .expect("recipients should fit");
            assert_noop!(
                ResolutionIssuanceGov::set_allowed_recipients(RuntimeOrigin::root(), recipients),
                pallet::Error::<Test>::ActiveVotingProposalsExist
            );
        });
    }

    #[test]
    fn set_allowed_recipients_rejects_empty_list() {
        new_test_ext().execute_with(|| {
            let recipients: BoundedVec<AccountId32, ConstU32<64>> =
                Vec::new().try_into().expect("empty should fit");
            assert_noop!(
                ResolutionIssuanceGov::set_allowed_recipients(RuntimeOrigin::root(), recipients),
                pallet::Error::<Test>::RecipientsNotConfigured
            );
        });
    }

    #[test]
    fn set_allowed_recipients_rejects_duplicates() {
        new_test_ext().execute_with(|| {
            let first = reserve_council_accounts()[0].clone();
            let recipients: BoundedVec<AccountId32, ConstU32<64>> = vec![first.clone(), first]
                .try_into()
                .expect("recipients should fit");
            assert_noop!(
                ResolutionIssuanceGov::set_allowed_recipients(RuntimeOrigin::root(), recipients),
                pallet::Error::<Test>::DuplicateAllowedRecipient
            );
        });
    }
}
