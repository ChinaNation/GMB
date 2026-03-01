#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{ensure, pallet_prelude::*, traits::Currency, Blake2_128Concat};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::traits::{CheckedAdd, One, Saturating, Zero};

use primitives::china::china_cb::{shenfen_id_to_fixed48 as reserve_pallet_id_to_bytes, CHINA_CB};
use primitives::china::china_ch::{
    shenfen_id_to_fixed48 as shengbank_pallet_id_to_bytes, CHINA_CH,
};
use voting_engine_system::{
    internal_vote::{ORG_NRC, ORG_PRB, ORG_PRC},
    InstitutionPalletId, STATUS_PASSED, STATUS_REJECTED,
};

pub use pallet::*;

type BalanceOf<T> =
    <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// Weight functions needed for `resolution-destro-gov`.
pub trait WeightInfo {
    fn propose_destroy() -> Weight;
    fn vote_destroy() -> Weight;
    fn execute_destroy() -> Weight;
    fn cancel_stale_destroy() -> Weight;
}

/// Default weights for `resolution-destro-gov` using runtime DbWeight.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn propose_destroy() -> Weight {
        Weight::from_parts(80_000_000, 4_096)
            .saturating_add(T::DbWeight::get().reads_writes(8_u64, 8_u64))
    }

    fn vote_destroy() -> Weight {
        // Worst-case path includes reaching PASS threshold and attempting auto execution.
        Weight::from_parts(220_000_000, 12_288)
            .saturating_add(T::DbWeight::get().reads_writes(14_u64, 12_u64))
    }

    fn execute_destroy() -> Weight {
        Weight::from_parts(140_000_000, 8_192)
            .saturating_add(T::DbWeight::get().reads_writes(9_u64, 8_u64))
    }

    fn cancel_stale_destroy() -> Weight {
        Weight::from_parts(70_000_000, 4_096)
            .saturating_add(T::DbWeight::get().reads_writes(6_u64, 6_u64))
    }
}

impl WeightInfo for () {
    fn propose_destroy() -> Weight {
        Weight::from_parts(80_000_000, 4_096).saturating_add(
            frame_support::weights::constants::RocksDbWeight::get().reads_writes(8_u64, 8_u64),
        )
    }

    fn vote_destroy() -> Weight {
        Weight::from_parts(220_000_000, 12_288).saturating_add(
            frame_support::weights::constants::RocksDbWeight::get().reads_writes(14_u64, 12_u64),
        )
    }

    fn execute_destroy() -> Weight {
        Weight::from_parts(140_000_000, 8_192).saturating_add(
            frame_support::weights::constants::RocksDbWeight::get().reads_writes(9_u64, 8_u64),
        )
    }

    fn cancel_stale_destroy() -> Weight {
        Weight::from_parts(70_000_000, 4_096).saturating_add(
            frame_support::weights::constants::RocksDbWeight::get().reads_writes(6_u64, 6_u64),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct DestroyAction<Balance> {
    /// 目标机构（机构标识 pallet_id）
    pub institution: InstitutionPalletId,
    /// 销毁数量
    pub amount: Balance,
}

fn nrc_pallet_id_bytes() -> Option<InstitutionPalletId> {
    CHINA_CB
        .first()
        .and_then(|n| reserve_pallet_id_to_bytes(n.shenfen_id))
}

fn institution_org(institution: InstitutionPalletId) -> Option<u8> {
    if Some(institution) == nrc_pallet_id_bytes() {
        return Some(ORG_NRC);
    }

    if CHINA_CB
        .iter()
        .skip(1)
        .filter_map(|n| reserve_pallet_id_to_bytes(n.shenfen_id))
        .any(|pid| pid == institution)
    {
        return Some(ORG_PRC);
    }

    if CHINA_CH
        .iter()
        .filter_map(|n| shengbank_pallet_id_to_bytes(n.shenfen_id))
        .any(|pid| pid == institution)
    {
        return Some(ORG_PRB);
    }

    None
}

fn institution_pallet_address(institution: InstitutionPalletId) -> Option<[u8; 32]> {
    if let Some(node) = CHINA_CB
        .iter()
        .find(|n| reserve_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
    {
        return Some(node.duoqian_address);
    }

    CHINA_CH
        .iter()
        .find(|n| shengbank_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
        .map(|n| n.duoqian_address)
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use voting_engine_system::InternalAdminProvider;
    use voting_engine_system::InternalVoteEngine;

    #[pallet::config]
    pub trait Config: frame_system::Config + voting_engine_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Currency: Currency<Self::AccountId>;

        #[pallet::constant]
        /// 超过该时长仍未执行的提案可被清理。
        type StaleProposalLifetime: Get<BlockNumberFor<Self>>;

        /// 中文注释：通过统一内部投票引擎创建提案，返回真实 proposal_id。
        type InternalVoteEngine: voting_engine_system::InternalVoteEngine<Self::AccountId>;

        /// 该 pallet 的可配置权重实现。
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        #[cfg(feature = "std")]
        fn integrity_test() {
            assert!(
                !T::StaleProposalLifetime::get().is_zero(),
                "StaleProposalLifetime must be > 0"
            );
        }
    }

    #[pallet::storage]
    #[pallet::getter(fn proposal_action)]
    pub type ProposalActions<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, DestroyAction<BalanceOf<T>>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn proposal_created_at)]
    pub type ProposalCreatedAt<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, BlockNumberFor<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn proposal_passed_at)]
    pub type ProposalPassedAt<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, BlockNumberFor<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn active_proposal_by_institution)]
    pub type ActiveProposalByInstitution<T: Config> =
        StorageMap<_, Blake2_128Concat, InstitutionPalletId, u64, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 已发起销毁提案（并已在投票引擎创建内部提案）
        DestroyProposed {
            proposal_id: u64,
            org: u8,
            institution: InstitutionPalletId,
            proposer: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// 提交销毁投票
        DestroyVoteSubmitted {
            proposal_id: u64,
            who: T::AccountId,
            approve: bool,
        },
        /// 提案达到通过状态但自动执行失败（投票不回滚）
        DestroyExecutionFailed { proposal_id: u64 },
        /// 销毁执行完成
        DestroyExecuted {
            proposal_id: u64,
            institution: InstitutionPalletId,
            amount: BalanceOf<T>,
        },
        /// 过期且未执行的销毁提案被清理
        StaleDestroyCancelled {
            proposal_id: u64,
            institution: InstitutionPalletId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        InvalidInstitution,
        InstitutionOrgMismatch,
        UnauthorizedAdmin,
        ZeroAmount,
        ProposalActionNotFound,
        ProposalNotPassed,
        InstitutionAccountDecodeFailed,
        InsufficientBalance,
        ActiveProposalExists,
        ProposalNotStale,
        PassedProposalCannotBeCancelled,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 发起“决议销毁”内部投票提案。
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::propose_destroy())]
        pub fn propose_destroy(
            origin: OriginFor<T>,
            org: u8,
            institution: InstitutionPalletId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(amount > Zero::zero(), Error::<T>::ZeroAmount);
            let actual_org = institution_org(institution).ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(actual_org == org, Error::<T>::InstitutionOrgMismatch);
            let stale_proposal = Self::check_no_active_proposal(institution)?;
            ensure!(
                Self::is_internal_admin(org, institution, &who),
                Error::<T>::UnauthorizedAdmin
            );

            let proposal_id =
                T::InternalVoteEngine::create_internal_proposal(who.clone(), org, institution)?;
            if let Some((stale_id, emit_stale_event)) = stale_proposal {
                // 中文注释：防御性保护，避免极端 proposal_id 回绕时误删新提案。
                if stale_id != proposal_id {
                    Self::cleanup_inactive_proposal(institution, stale_id);
                    if emit_stale_event {
                        Self::deposit_event(Event::<T>::StaleDestroyCancelled {
                            proposal_id: stale_id,
                            institution,
                        });
                    }
                }
            }

            ProposalActions::<T>::insert(
                proposal_id,
                DestroyAction {
                    institution,
                    amount,
                },
            );
            ProposalCreatedAt::<T>::insert(proposal_id, frame_system::Pallet::<T>::block_number());
            ActiveProposalByInstitution::<T>::insert(institution, proposal_id);

            Self::deposit_event(Event::<T>::DestroyProposed {
                proposal_id,
                org,
                institution,
                proposer: who,
                amount,
            });
            Ok(())
        }

        /// 对“决议销毁”提案投票，达到阈值通过后自动执行销毁。
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::vote_destroy())]
        pub fn vote_destroy(
            origin: OriginFor<T>,
            proposal_id: u64,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let action =
                ProposalActions::<T>::get(proposal_id).ok_or(Error::<T>::ProposalActionNotFound)?;
            let org = institution_org(action.institution).ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                Self::is_internal_admin(org, action.institution, &who),
                Error::<T>::UnauthorizedAdmin
            );

            T::InternalVoteEngine::cast_internal_vote(who.clone(), proposal_id, approve)?;

            Self::deposit_event(Event::<T>::DestroyVoteSubmitted {
                proposal_id,
                who,
                approve,
            });

            if let Some(proposal) = voting_engine_system::Pallet::<T>::proposals(proposal_id) {
                if proposal.status == STATUS_PASSED {
                    if ProposalPassedAt::<T>::get(proposal_id).is_none() {
                        ProposalPassedAt::<T>::insert(
                            proposal_id,
                            frame_system::Pallet::<T>::block_number(),
                        );
                    }
                    if approve
                        && Self::try_execute_destroy_from_action(proposal_id, action).is_err()
                    {
                        Self::deposit_event(Event::<T>::DestroyExecutionFailed { proposal_id });
                    }
                } else if proposal.status == STATUS_REJECTED {
                    Self::cleanup_inactive_proposal(action.institution, proposal_id);
                }
            }
            Ok(())
        }

        /// 手动执行已通过的销毁提案。
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::execute_destroy())]
        pub fn execute_destroy(origin: OriginFor<T>, proposal_id: u64) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            Self::try_execute_destroy(proposal_id)
        }

        /// 清理已过期且未执行的销毁提案。
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::cancel_stale_destroy())]
        pub fn cancel_stale_destroy(origin: OriginFor<T>, proposal_id: u64) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            let action =
                ProposalActions::<T>::get(proposal_id).ok_or(Error::<T>::ProposalActionNotFound)?;
            let created_at = ProposalCreatedAt::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            let is_passed = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .map(|proposal| proposal.status == STATUS_PASSED)
                .unwrap_or(false);
            ensure!(!is_passed, Error::<T>::PassedProposalCannotBeCancelled);
            let now = frame_system::Pallet::<T>::block_number();
            let stale_at = created_at.saturating_add(Self::effective_stale_lifetime());
            ensure!(now >= stale_at, Error::<T>::ProposalNotStale);

            Self::cleanup_inactive_proposal(action.institution, proposal_id);

            Self::deposit_event(Event::<T>::StaleDestroyCancelled {
                proposal_id,
                institution: action.institution,
            });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn is_internal_admin(
            org: u8,
            institution: InstitutionPalletId,
            who: &T::AccountId,
        ) -> bool {
            <T as voting_engine_system::Config>::InternalAdminProvider::is_internal_admin(
                org,
                institution,
                who,
            )
        }

        fn effective_stale_lifetime() -> BlockNumberFor<T> {
            let configured = T::StaleProposalLifetime::get();
            if configured.is_zero() {
                One::one()
            } else {
                configured
            }
        }

        fn check_no_active_proposal(
            institution: InstitutionPalletId,
        ) -> Result<Option<(u64, bool)>, DispatchError> {
            if let Some(existing_id) = ActiveProposalByInstitution::<T>::get(institution) {
                if ProposalActions::<T>::contains_key(existing_id) {
                    if let Some(proposal) =
                        voting_engine_system::Pallet::<T>::proposals(existing_id)
                    {
                        if proposal.status == STATUS_REJECTED {
                            return Ok(Some((existing_id, false)));
                        }
                        if proposal.status == STATUS_PASSED {
                            let now = frame_system::Pallet::<T>::block_number();
                            let anchor = ProposalPassedAt::<T>::get(existing_id)
                                .or_else(|| ProposalCreatedAt::<T>::get(existing_id));
                            let still_active = anchor
                                .map(|at| {
                                    let stale_at =
                                        at.saturating_add(Self::effective_stale_lifetime());
                                    now < stale_at
                                })
                                .unwrap_or(false);
                            if still_active {
                                return Err(Error::<T>::ActiveProposalExists.into());
                            }
                            return Ok(Some((existing_id, anchor.is_some())));
                        }
                        return Err(Error::<T>::ActiveProposalExists.into());
                    }
                }
                return Ok(Some((existing_id, false)));
            }
            Ok(None)
        }

        fn remove_active_proposal_if_matches(institution: InstitutionPalletId, proposal_id: u64) {
            if ActiveProposalByInstitution::<T>::get(institution) == Some(proposal_id) {
                ActiveProposalByInstitution::<T>::remove(institution);
            }
        }

        fn cleanup_inactive_proposal(institution: InstitutionPalletId, proposal_id: u64) {
            ProposalActions::<T>::remove(proposal_id);
            ProposalCreatedAt::<T>::remove(proposal_id);
            ProposalPassedAt::<T>::remove(proposal_id);
            Self::remove_active_proposal_if_matches(institution, proposal_id);
            T::InternalVoteEngine::cleanup_internal_proposal(proposal_id);
        }

        fn try_execute_destroy(proposal_id: u64) -> DispatchResult {
            let action =
                ProposalActions::<T>::get(proposal_id).ok_or(Error::<T>::ProposalActionNotFound)?;
            Self::try_execute_destroy_from_action(proposal_id, action)
        }

        fn try_execute_destroy_from_action(
            proposal_id: u64,
            action: DestroyAction<BalanceOf<T>>,
        ) -> DispatchResult {
            let proposal = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            ensure!(
                proposal.status == STATUS_PASSED,
                Error::<T>::ProposalNotPassed
            );

            let raw_account = institution_pallet_address(action.institution)
                .ok_or(Error::<T>::InvalidInstitution)?;
            let institution_account = T::AccountId::decode(&mut &raw_account[..])
                .map_err(|_| Error::<T>::InstitutionAccountDecodeFailed)?;

            let free = T::Currency::free_balance(&institution_account);
            let ed = T::Currency::minimum_balance();
            let required = action
                .amount
                .checked_add(&ed)
                .ok_or(Error::<T>::InsufficientBalance)?;
            ensure!(free >= required, Error::<T>::InsufficientBalance);

            // 中文注释：slash 会同步减少总发行量，实现链上“销毁”。
            let (_negative_imbalance, remaining) =
                T::Currency::slash(&institution_account, action.amount);
            ensure!(remaining.is_zero(), Error::<T>::InsufficientBalance);

            Self::cleanup_inactive_proposal(action.institution, proposal_id);

            Self::deposit_event(Event::<T>::DestroyExecuted {
                proposal_id,
                institution: action.institution,
                amount: action.amount,
            });
            Ok(())
        }
    }
}

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking {
    use super::*;

    use codec::Decode;
    use frame_benchmarking::v2::*;
    use frame_support::traits::Currency;
    use frame_system::RawOrigin;
    use sp_runtime::traits::{SaturatedConversion, Saturating};
    use voting_engine_system::InternalVoteEngine;

    use crate::Pallet as ResolutionDestroGov;

    fn decode_account<T: pallet::Config>(raw: [u8; 32]) -> T::AccountId {
        T::AccountId::decode(&mut &raw[..]).expect("benchmark account must decode")
    }

    fn prc_institution() -> InstitutionPalletId {
        reserve_pallet_id_to_bytes(CHINA_CB[1].shenfen_id).expect("PRC institution should be valid")
    }

    fn prc_admin<T: pallet::Config>(index: usize) -> T::AccountId {
        decode_account::<T>(CHINA_CB[1].admins[index])
    }

    fn institution_account<T: pallet::Config>(institution: InstitutionPalletId) -> T::AccountId {
        let raw =
            institution_pallet_address(institution).expect("institution account should exist");
        decode_account::<T>(raw)
    }

    #[benchmarks]
    mod benchmarks {
        use super::*;

        #[benchmark]
        fn propose_destroy() {
            let institution = prc_institution();
            let proposer = prc_admin::<T>(0);
            let amount: BalanceOf<T> = 100u128.saturated_into();

            #[extrinsic_call]
            propose_destroy(
                RawOrigin::Signed(proposer.clone()),
                ORG_PRC,
                institution,
                amount,
            );

            assert_eq!(ActiveProposalByInstitution::<T>::get(institution), Some(0));
            assert!(ProposalActions::<T>::contains_key(0));
        }

        #[benchmark]
        fn vote_destroy() {
            let institution = prc_institution();
            let proposer = prc_admin::<T>(0);
            let final_voter = prc_admin::<T>(5);
            let amount: BalanceOf<T> = 100u128.saturated_into();
            let top_up: BalanceOf<T> = 1_000_000u128.saturated_into();

            assert!(ResolutionDestroGov::<T>::propose_destroy(
                RawOrigin::Signed(proposer).into(),
                ORG_PRC,
                institution,
                amount,
            )
            .is_ok());

            let institution_account = institution_account::<T>(institution);
            let _ = T::Currency::deposit_creating(&institution_account, top_up);

            for i in 0..5 {
                let voter = prc_admin::<T>(i);
                assert!(T::InternalVoteEngine::cast_internal_vote(voter, 0, true).is_ok());
            }

            #[extrinsic_call]
            vote_destroy(RawOrigin::Signed(final_voter), 0, true);

            assert!(!ProposalActions::<T>::contains_key(0));
        }

        #[benchmark]
        fn execute_destroy() {
            let institution = prc_institution();
            let proposer = prc_admin::<T>(0);
            let caller = prc_admin::<T>(6);
            let amount: BalanceOf<T> = 100u128.saturated_into();
            let top_up: BalanceOf<T> = 1_000_000u128.saturated_into();

            assert!(ResolutionDestroGov::<T>::propose_destroy(
                RawOrigin::Signed(proposer).into(),
                ORG_PRC,
                institution,
                amount,
            )
            .is_ok());

            let institution_account = institution_account::<T>(institution);
            let _ = T::Currency::deposit_creating(&institution_account, top_up);

            for i in 0..6 {
                let voter = prc_admin::<T>(i);
                assert!(T::InternalVoteEngine::cast_internal_vote(voter, 0, true).is_ok());
            }

            #[extrinsic_call]
            execute_destroy(RawOrigin::Signed(caller), 0);

            assert!(!ProposalActions::<T>::contains_key(0));
        }

        #[benchmark]
        fn cancel_stale_destroy() {
            let institution = prc_institution();
            let proposer = prc_admin::<T>(0);
            let caller = prc_admin::<T>(1);
            let amount: BalanceOf<T> = 100u128.saturated_into();

            assert!(ResolutionDestroGov::<T>::propose_destroy(
                RawOrigin::Signed(proposer).into(),
                ORG_PRC,
                institution,
                amount,
            )
            .is_ok());

            let one: BlockNumberFor<T> = 1u32.saturated_into();
            let stale_block = T::StaleProposalLifetime::get().saturating_add(one);
            frame_system::Pallet::<T>::set_block_number(stale_block);

            #[extrinsic_call]
            cancel_stale_destroy(RawOrigin::Signed(caller), 0);

            assert!(!ProposalActions::<T>::contains_key(0));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codec::Encode;
    use frame_support::{
        assert_noop, assert_ok, derive_impl,
        traits::{ConstU128, ConstU32, ConstU64},
    };
    use frame_system as system;
    use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};
    use voting_engine_system::{STATUS_PASSED, STATUS_REJECTED};

    type Balance = u128;
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
        pub type Balances = pallet_balances;

        #[runtime::pallet_index(2)]
        pub type VotingEngineSystem = voting_engine_system;

        #[runtime::pallet_index(3)]
        pub type ResolutionDestroGov = super;
    }

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl system::Config for Test {
        type Block = Block;
        type AccountId = AccountId32;
        type Lookup = IdentityLookup<Self::AccountId>;
        type AccountData = pallet_balances::AccountData<Balance>;
    }

    impl pallet_balances::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type Balance = Balance;
        type DustRemoval = ();
        type ExistentialDeposit = ConstU128<1>;
        type AccountStore = System;
        type MaxLocks = ConstU32<0>;
        type MaxReserves = ();
        type ReserveIdentifier = [u8; 8];
        type FreezeIdentifier = RuntimeFreezeReason;
        type MaxFreezes = ConstU32<0>;
        type RuntimeHoldReason = RuntimeHoldReason;
        type RuntimeFreezeReason = RuntimeFreezeReason;
        type DoneSlashHandler = ();
        type WeightInfo = ();
    }

    pub struct TestSfidEligibility;
    impl voting_engine_system::SfidEligibility<AccountId32> for TestSfidEligibility {
        fn is_eligible(_sfid: &[u8], _who: &AccountId32) -> bool {
            true
        }

        fn verify_and_consume_vote_credential(
            _sfid: &[u8],
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

    pub struct TestInternalAdminProvider;
    impl voting_engine_system::InternalAdminProvider<AccountId32> for TestInternalAdminProvider {
        fn is_internal_admin(org: u8, institution: InstitutionPalletId, who: &AccountId32) -> bool {
            let who_bytes = who.encode();
            if who_bytes.len() != 32 {
                return false;
            }
            let mut who_arr = [0u8; 32];
            who_arr.copy_from_slice(&who_bytes);
            match org {
                ORG_NRC | ORG_PRC => CHINA_CB
                    .iter()
                    .find(|n| reserve_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                    .map(|n| n.admins.iter().any(|admin| *admin == who_arr))
                    .unwrap_or(false),
                ORG_PRB => CHINA_CH
                    .iter()
                    .find(|n| shengbank_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                    .map(|n| n.admins.iter().any(|admin| *admin == who_arr))
                    .unwrap_or(false),
                _ => false,
            }
        }
    }

    impl voting_engine_system::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type MaxSfidLength = ConstU32<64>;
        type MaxVoteNonceLength = ConstU32<64>;
        type MaxVoteSignatureLength = ConstU32<64>;
        type SfidEligibility = TestSfidEligibility;
        type PopulationSnapshotVerifier = TestPopulationSnapshotVerifier;
        type JointVoteResultCallback = ();
        type InternalAdminProvider = TestInternalAdminProvider;
    }

    impl pallet::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type Currency = Balances;
        type StaleProposalLifetime = ConstU64<100>;
        type InternalVoteEngine = voting_engine_system::Pallet<Test>;
        type WeightInfo = ();
    }

    fn nrc_admin(index: usize) -> AccountId32 {
        AccountId32::new(CHINA_CB[0].admins[index])
    }

    fn prc_admin(index: usize) -> AccountId32 {
        AccountId32::new(CHINA_CB[1].admins[index])
    }

    fn prb_admin(index: usize) -> AccountId32 {
        AccountId32::new(CHINA_CH[0].admins[index])
    }

    fn nrc_pallet_id() -> InstitutionPalletId {
        reserve_pallet_id_to_bytes(CHINA_CB[0].shenfen_id).expect("nrc id should be valid")
    }

    fn prc_pallet_id() -> InstitutionPalletId {
        reserve_pallet_id_to_bytes(CHINA_CB[1].shenfen_id).expect("prc id should be valid")
    }

    fn prb_pallet_id() -> InstitutionPalletId {
        shengbank_pallet_id_to_bytes(CHINA_CH[0].shenfen_id).expect("prb id should be valid")
    }

    fn institution_account(institution: InstitutionPalletId) -> AccountId32 {
        let raw =
            institution_pallet_address(institution).expect("institution pallet address must exist");
        AccountId32::new(raw)
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let mut storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("test storage should build");

        let balances = vec![
            (institution_account(nrc_pallet_id()), 1_000),
            (institution_account(prc_pallet_id()), 1_000),
            (institution_account(prb_pallet_id()), 1_000),
        ];
        pallet_balances::GenesisConfig::<Test> {
            balances,
            ..Default::default()
        }
        .assimilate_storage(&mut storage)
        .expect("balances should assimilate");

        storage.into()
    }

    #[test]
    fn nrc_destroy_executes_when_yes_votes_reach_threshold() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let account = institution_account(institution);

            assert_ok!(ResolutionDestroGov::propose_destroy(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                100
            ));

            for i in 0..13 {
                assert_ok!(ResolutionDestroGov::vote_destroy(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    0,
                    true
                ));
            }

            assert_eq!(Balances::free_balance(&account), 900);
            assert!(ResolutionDestroGov::proposal_action(0).is_none());
            assert!(voting_engine_system::Pallet::<Test>::proposals(0).is_none());
        });
    }

    #[test]
    fn prc_destroy_executes_when_yes_votes_reach_threshold() {
        new_test_ext().execute_with(|| {
            let institution = prc_pallet_id();
            let account = institution_account(institution);

            assert_ok!(ResolutionDestroGov::propose_destroy(
                RuntimeOrigin::signed(prc_admin(0)),
                ORG_PRC,
                institution,
                200
            ));

            for i in 0..6 {
                assert_ok!(ResolutionDestroGov::vote_destroy(
                    RuntimeOrigin::signed(prc_admin(i)),
                    0,
                    true
                ));
            }

            assert_eq!(Balances::free_balance(&account), 800);
            assert!(ResolutionDestroGov::proposal_action(0).is_none());
        });
    }

    #[test]
    fn prb_destroy_executes_when_yes_votes_reach_threshold() {
        new_test_ext().execute_with(|| {
            let institution = prb_pallet_id();
            let account = institution_account(institution);

            assert_ok!(ResolutionDestroGov::propose_destroy(
                RuntimeOrigin::signed(prb_admin(0)),
                ORG_PRB,
                institution,
                300
            ));

            for i in 0..6 {
                assert_ok!(ResolutionDestroGov::vote_destroy(
                    RuntimeOrigin::signed(prb_admin(i)),
                    0,
                    true
                ));
            }

            assert_eq!(Balances::free_balance(&account), 700);
            assert!(ResolutionDestroGov::proposal_action(0).is_none());
        });
    }

    #[test]
    fn non_admin_cannot_propose_or_vote() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();

            assert_noop!(
                ResolutionDestroGov::propose_destroy(
                    RuntimeOrigin::signed(prc_admin(0)),
                    ORG_NRC,
                    institution,
                    100
                ),
                Error::<Test>::UnauthorizedAdmin
            );

            assert_ok!(ResolutionDestroGov::propose_destroy(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                100
            ));

            assert_noop!(
                ResolutionDestroGov::vote_destroy(RuntimeOrigin::signed(prc_admin(0)), 0, true),
                Error::<Test>::UnauthorizedAdmin
            );
        });
    }

    #[test]
    fn zero_amount_and_insufficient_balance_are_rejected() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();

            assert_noop!(
                ResolutionDestroGov::propose_destroy(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    ORG_NRC,
                    institution,
                    0
                ),
                Error::<Test>::ZeroAmount
            );

            assert_ok!(ResolutionDestroGov::propose_destroy(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                2_000
            ));

            for i in 0..12 {
                assert_ok!(ResolutionDestroGov::vote_destroy(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    0,
                    true
                ));
            }

            // 第 13 票应被记录，自动执行失败不回滚投票。
            assert_ok!(ResolutionDestroGov::vote_destroy(
                RuntimeOrigin::signed(nrc_admin(12)),
                0,
                true
            ));
            assert_eq!(
                voting_engine_system::Pallet::<Test>::proposals(0)
                    .expect("proposal should exist")
                    .status,
                STATUS_PASSED
            );
            assert_eq!(
                Balances::free_balance(institution_account(institution)),
                1_000
            );
            assert!(ResolutionDestroGov::proposal_action(0).is_some());
            assert_noop!(
                ResolutionDestroGov::execute_destroy(RuntimeOrigin::signed(nrc_admin(0)), 0),
                Error::<Test>::InsufficientBalance
            );
        });
    }

    #[test]
    fn existential_deposit_is_preserved() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let account = institution_account(institution);

            assert_ok!(ResolutionDestroGov::propose_destroy(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                1_000
            ));

            for i in 0..13 {
                assert_ok!(ResolutionDestroGov::vote_destroy(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    0,
                    true
                ));
            }

            // 如果不校验 ED，这里会被销毁到 0 并触发账户 reap。
            assert_eq!(Balances::free_balance(&account), 1_000);
            assert_noop!(
                ResolutionDestroGov::execute_destroy(RuntimeOrigin::signed(nrc_admin(0)), 0),
                Error::<Test>::InsufficientBalance
            );
        });
    }

    #[test]
    fn rejected_proposal_does_not_block_new_proposal() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            assert_ok!(ResolutionDestroGov::propose_destroy(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                100
            ));

            let end = voting_engine_system::Pallet::<Test>::proposals(0)
                .expect("proposal should exist")
                .end;
            System::set_block_number(end + 1);
            assert_ok!(voting_engine_system::Pallet::<Test>::finalize_proposal(
                RuntimeOrigin::signed(nrc_admin(0)),
                0
            ));
            assert_eq!(
                voting_engine_system::Pallet::<Test>::proposals(0)
                    .expect("proposal should exist")
                    .status,
                STATUS_REJECTED
            );

            assert_ok!(ResolutionDestroGov::propose_destroy(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                50
            ));
            assert!(ResolutionDestroGov::proposal_action(0).is_none());
            assert!(ProposalCreatedAt::<Test>::get(0).is_none());
            assert_eq!(
                ActiveProposalByInstitution::<Test>::get(institution),
                Some(1)
            );
        });
    }

    #[test]
    fn execute_destroy_succeeds_after_failed_auto_execution() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let account = institution_account(institution);

            assert_ok!(ResolutionDestroGov::propose_destroy(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                1_100
            ));

            for i in 0..13 {
                assert_ok!(ResolutionDestroGov::vote_destroy(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    0,
                    true
                ));
            }

            assert_eq!(
                voting_engine_system::Pallet::<Test>::proposals(0)
                    .expect("proposal should exist")
                    .status,
                STATUS_PASSED
            );
            assert_eq!(Balances::free_balance(&account), 1_000);
            assert!(ResolutionDestroGov::proposal_action(0).is_some());

            let _ = Balances::deposit_creating(&account, 200);
            assert_ok!(ResolutionDestroGov::execute_destroy(
                RuntimeOrigin::signed(nrc_admin(0)),
                0
            ));
            assert_eq!(Balances::free_balance(&account), 100);
            assert!(ResolutionDestroGov::proposal_action(0).is_none());
            assert!(ProposalCreatedAt::<Test>::get(0).is_none());
            assert!(ActiveProposalByInstitution::<Test>::get(institution).is_none());
        });
    }

    #[test]
    fn stale_proposal_can_be_cancelled() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            assert_ok!(ResolutionDestroGov::propose_destroy(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                100
            ));

            System::set_block_number(99);
            assert_noop!(
                ResolutionDestroGov::cancel_stale_destroy(RuntimeOrigin::signed(nrc_admin(0)), 0),
                Error::<Test>::ProposalNotStale
            );

            System::set_block_number(100);
            assert_ok!(ResolutionDestroGov::cancel_stale_destroy(
                RuntimeOrigin::signed(nrc_admin(0)),
                0
            ));
            assert!(ResolutionDestroGov::proposal_action(0).is_none());
            assert!(ProposalCreatedAt::<Test>::get(0).is_none());
            assert!(ActiveProposalByInstitution::<Test>::get(institution).is_none());
            assert!(voting_engine_system::Pallet::<Test>::proposals(0).is_none());
        });
    }

    #[test]
    fn passed_proposal_cannot_be_cancelled_but_stale_can_be_overridden() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            assert_ok!(ResolutionDestroGov::propose_destroy(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                2_000
            ));

            System::set_block_number(90);
            for i in 0..13 {
                assert_ok!(ResolutionDestroGov::vote_destroy(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    0,
                    true
                ));
            }
            assert_eq!(
                ProposalPassedAt::<Test>::get(0),
                Some(System::block_number())
            );

            System::set_block_number(190);
            assert_noop!(
                ResolutionDestroGov::cancel_stale_destroy(RuntimeOrigin::signed(nrc_admin(0)), 0),
                Error::<Test>::PassedProposalCannotBeCancelled
            );

            assert_ok!(ResolutionDestroGov::propose_destroy(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                100
            ));
            assert!(System::events().iter().any(|record| {
                matches!(
                    &record.event,
                    RuntimeEvent::ResolutionDestroGov(
                        Event::<Test>::StaleDestroyCancelled {
                            proposal_id,
                            institution: inst,
                        }
                    ) if *proposal_id == 0 && *inst == institution
                )
            }));
            assert!(ResolutionDestroGov::proposal_action(0).is_none());
            assert!(voting_engine_system::Pallet::<Test>::proposals(0).is_none());
            assert_eq!(
                ActiveProposalByInstitution::<Test>::get(institution),
                Some(1)
            );
        });
    }

    #[test]
    fn passed_proposal_without_timestamps_can_be_overridden_for_recovery() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            assert_ok!(ResolutionDestroGov::propose_destroy(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                2_000
            ));

            System::set_block_number(90);
            for i in 0..13 {
                assert_ok!(ResolutionDestroGov::vote_destroy(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    0,
                    true
                ));
            }
            assert_eq!(
                voting_engine_system::Pallet::<Test>::proposals(0)
                    .expect("proposal should exist")
                    .status,
                STATUS_PASSED
            );
            ProposalPassedAt::<Test>::remove(0);
            ProposalCreatedAt::<Test>::remove(0);

            assert_ok!(ResolutionDestroGov::propose_destroy(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                100
            ));
            assert!(ResolutionDestroGov::proposal_action(0).is_none());
            assert!(voting_engine_system::Pallet::<Test>::proposals(0).is_none());
            assert_eq!(
                ActiveProposalByInstitution::<Test>::get(institution),
                Some(1)
            );
        });
    }

    #[test]
    fn cancel_stale_destroy_is_allowed_for_non_admin() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let outsider = AccountId32::new([99u8; 32]);
            assert_ok!(ResolutionDestroGov::propose_destroy(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                100
            ));

            System::set_block_number(100);
            assert_ok!(ResolutionDestroGov::cancel_stale_destroy(
                RuntimeOrigin::signed(outsider),
                0
            ));
            assert!(ResolutionDestroGov::proposal_action(0).is_none());
            assert!(ActiveProposalByInstitution::<Test>::get(institution).is_none());
        });
    }

    #[test]
    fn executed_proposal_does_not_block_new_proposal() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            assert_ok!(ResolutionDestroGov::propose_destroy(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                100
            ));

            for i in 0..13 {
                assert_ok!(ResolutionDestroGov::vote_destroy(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    0,
                    true
                ));
            }

            assert!(ResolutionDestroGov::proposal_action(0).is_none());
            assert!(ActiveProposalByInstitution::<Test>::get(institution).is_none());

            assert_ok!(ResolutionDestroGov::propose_destroy(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                50
            ));
            assert_eq!(
                ActiveProposalByInstitution::<Test>::get(institution),
                Some(1)
            );
        });
    }

    #[test]
    fn duplicate_vote_is_rejected_by_voting_engine() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            assert_ok!(ResolutionDestroGov::propose_destroy(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                100
            ));
            assert_ok!(ResolutionDestroGov::vote_destroy(
                RuntimeOrigin::signed(nrc_admin(1)),
                0,
                true
            ));
            assert_noop!(
                ResolutionDestroGov::vote_destroy(RuntimeOrigin::signed(nrc_admin(1)), 0, true),
                voting_engine_system::pallet::Error::<Test>::AlreadyVoted
            );
        });
    }

    #[test]
    fn execute_destroy_is_allowed_for_non_admin() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let account = institution_account(institution);
            let outsider = AccountId32::new([99u8; 32]);

            assert_ok!(ResolutionDestroGov::propose_destroy(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                1_100
            ));
            for i in 0..13 {
                assert_ok!(ResolutionDestroGov::vote_destroy(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    0,
                    true
                ));
            }
            let _ = Balances::deposit_creating(&account, 200);
            assert_ok!(ResolutionDestroGov::execute_destroy(
                RuntimeOrigin::signed(outsider),
                0
            ));
            assert_eq!(Balances::free_balance(&account), 100);
        });
    }

    #[test]
    fn institution_org_returns_none_for_invalid_institution() {
        new_test_ext().execute_with(|| {
            assert_eq!(institution_org([0u8; 48]), None);
        });
    }
}
