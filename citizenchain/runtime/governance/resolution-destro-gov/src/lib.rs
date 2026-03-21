#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{ensure, pallet_prelude::*, traits::Currency};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::traits::{CheckedAdd, Zero};

use primitives::china::china_cb::{shenfen_id_to_fixed48 as reserve_pallet_id_to_bytes, CHINA_CB};
use primitives::china::china_ch::{
    shenfen_id_to_fixed48 as shengbank_pallet_id_to_bytes, CHINA_CH,
};
use voting_engine_system::{
    internal_vote::{ORG_NRC, ORG_PRB, ORG_PRC},
    InstitutionPalletId, STATUS_EXECUTED, STATUS_PASSED,
};

pub use pallet::*;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod weights;

type BalanceOf<T> =
    <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

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
    use crate::weights::WeightInfo;
    use voting_engine_system::InternalAdminProvider;
    use voting_engine_system::InternalVoteEngine;

    #[pallet::config]
    pub trait Config: frame_system::Config + voting_engine_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Currency: Currency<Self::AccountId>;

        /// 中文注释：通过统一内部投票引擎创建提案，返回真实 proposal_id。
        type InternalVoteEngine: voting_engine_system::InternalVoteEngine<Self::AccountId>;

        /// 该 pallet 的可配置权重实现。
        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // 提案数据、元数据、活跃提案列表均已移至 voting-engine-system 统一管控。

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
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 发起“决议销毁”内部投票提案。
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::propose_destroy())]
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
            // 活跃提案数由 voting-engine-system 在 create_internal_proposal 中统一检查
            ensure!(
                Self::is_internal_admin(org, institution, &who),
                Error::<T>::UnauthorizedAdmin
            );

            let proposal_id =
                T::InternalVoteEngine::create_internal_proposal(who.clone(), org, institution)?;

            let action = DestroyAction {
                institution,
                amount,
            };
            let data = action.encode();
            voting_engine_system::Pallet::<T>::store_proposal_data(proposal_id, data)?;
            voting_engine_system::Pallet::<T>::store_proposal_meta(
                proposal_id,
                frame_system::Pallet::<T>::block_number(),
            );

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
        #[pallet::weight(<T as pallet::Config>::WeightInfo::vote_destroy())]
        pub fn vote_destroy(
            origin: OriginFor<T>,
            proposal_id: u64,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let data = voting_engine_system::Pallet::<T>::get_proposal_data(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            let action = DestroyAction::decode(&mut &data[..])
                .map_err(|_| Error::<T>::ProposalActionNotFound)?;
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
                    if voting_engine_system::Pallet::<T>::get_proposal_meta(proposal_id)
                        .and_then(|m| m.passed_at)
                        .is_none()
                    {
                        voting_engine_system::Pallet::<T>::set_proposal_passed(
                            proposal_id,
                            frame_system::Pallet::<T>::block_number(),
                        );
                    }
                    if approve
                        && Self::try_execute_destroy_from_action(proposal_id, action).is_err()
                    {
                        Self::deposit_event(Event::<T>::DestroyExecutionFailed { proposal_id });
                    }
                }
            }
            Ok(())
        }

        /// 手动执行已通过的销毁提案。
        #[pallet::call_index(2)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::execute_destroy())]
        pub fn execute_destroy(origin: OriginFor<T>, proposal_id: u64) -> DispatchResult {
            // 中文注释：这里保留公开触发入口，任何签名账户都可以推动“已获批准”的销毁落地，
            // 不要求再次拿到管理员身份，避免因管理员离线导致通过提案迟迟无法执行。
            let _ = ensure_signed(origin)?;
            Self::try_execute_destroy(proposal_id)
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

        fn try_execute_destroy(proposal_id: u64) -> DispatchResult {
            let data = voting_engine_system::Pallet::<T>::get_proposal_data(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            let action = DestroyAction::decode(&mut &data[..])
                .map_err(|_| Error::<T>::ProposalActionNotFound)?;
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
            // 中文注释：销毁前必须预留 ED，确保机构账户不会因一次销毁被直接 reap。
            let required = action
                .amount
                .checked_add(&ed)
                .ok_or(Error::<T>::InsufficientBalance)?;
            ensure!(free >= required, Error::<T>::InsufficientBalance);

            // 中文注释：slash 会同步减少总发行量，实现链上”销毁”。
            let (_negative_imbalance, remaining) =
                T::Currency::slash(&institution_account, action.amount);
            ensure!(remaining.is_zero(), Error::<T>::InsufficientBalance);

            // 标记为已执行，防止双重执行
            voting_engine_system::Pallet::<T>::set_status_and_emit(proposal_id, STATUS_EXECUTED)?;

            Self::deposit_event(Event::<T>::DestroyExecuted {
                proposal_id,
                institution: action.institution,
                amount: action.amount,
            });
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codec::Encode;
    use frame_support::{
        assert_noop, assert_ok, derive_impl,
        traits::{ConstU128, ConstU32},
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
    impl voting_engine_system::SfidEligibility<AccountId32, <Test as frame_system::Config>::Hash>
        for TestSfidEligibility
    {
        fn is_eligible(
            _binding_id: &<Test as frame_system::Config>::Hash,
            _who: &AccountId32,
        ) -> bool {
            true
        }

        fn verify_and_consume_vote_credential(
            _binding_id: &<Test as frame_system::Config>::Hash,
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
        type MaxProposalDataLen = ConstU32<256>;
        type MaxProposalObjectLen = ConstU32<{ 10 * 1024 }>;
        type SfidEligibility = TestSfidEligibility;
        type PopulationSnapshotVerifier = TestPopulationSnapshotVerifier;
        type JointVoteResultCallback = ();
        type InternalAdminProvider = TestInternalAdminProvider;
        type InternalThresholdProvider = ();
        type InternalAdminCountProvider = ();
        type TimeProvider = TestTimeProvider;
        type WeightInfo = ();
    }

    impl pallet::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type Currency = Balances;
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

    /// 获取最近一次 create_internal_proposal 分配的 proposal_id。
    fn last_proposal_id() -> u64 {
        voting_engine_system::Pallet::<Test>::next_proposal_id().saturating_sub(1)
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
            let pid = last_proposal_id();

            for i in 0..13 {
                assert_ok!(ResolutionDestroGov::vote_destroy(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    pid,
                    true
                ));
            }

            assert_eq!(Balances::free_balance(&account), 900);
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
            let pid = last_proposal_id();

            for i in 0..6 {
                assert_ok!(ResolutionDestroGov::vote_destroy(
                    RuntimeOrigin::signed(prc_admin(i)),
                    pid,
                    true
                ));
            }

            assert_eq!(Balances::free_balance(&account), 800);
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
            let pid = last_proposal_id();

            for i in 0..6 {
                assert_ok!(ResolutionDestroGov::vote_destroy(
                    RuntimeOrigin::signed(prb_admin(i)),
                    pid,
                    true
                ));
            }

            assert_eq!(Balances::free_balance(&account), 700);
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
            let pid = last_proposal_id();

            assert_noop!(
                ResolutionDestroGov::vote_destroy(RuntimeOrigin::signed(prc_admin(0)), pid, true),
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
            let pid = last_proposal_id();

            for i in 0..12 {
                assert_ok!(ResolutionDestroGov::vote_destroy(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    pid,
                    true
                ));
            }

            // 第 13 票应被记录，自动执行失败不回滚投票。
            assert_ok!(ResolutionDestroGov::vote_destroy(
                RuntimeOrigin::signed(nrc_admin(12)),
                pid,
                true
            ));
            assert_eq!(
                voting_engine_system::Pallet::<Test>::proposals(pid)
                    .expect("proposal should exist")
                    .status,
                STATUS_PASSED
            );
            assert_eq!(
                Balances::free_balance(institution_account(institution)),
                1_000
            );
            assert!(voting_engine_system::Pallet::<Test>::get_proposal_data(pid).is_some());
            assert_noop!(
                ResolutionDestroGov::execute_destroy(RuntimeOrigin::signed(nrc_admin(0)), pid),
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
            let pid = last_proposal_id();

            for i in 0..13 {
                assert_ok!(ResolutionDestroGov::vote_destroy(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    pid,
                    true
                ));
            }

            // 如果不校验 ED，这里会被销毁到 0 并触发账户 reap。
            assert_eq!(Balances::free_balance(&account), 1_000);
            assert_noop!(
                ResolutionDestroGov::execute_destroy(RuntimeOrigin::signed(nrc_admin(0)), pid),
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
            let pid1 = last_proposal_id();

            let end = voting_engine_system::Pallet::<Test>::proposals(pid1)
                .expect("proposal should exist")
                .end;
            System::set_block_number(end + 1);
            assert_ok!(voting_engine_system::Pallet::<Test>::finalize_proposal(
                RuntimeOrigin::signed(nrc_admin(0)),
                pid1
            ));
            assert_eq!(
                voting_engine_system::Pallet::<Test>::proposals(pid1)
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
            let pid2 = last_proposal_id();
            // 提案 2 应该已创建
            assert!(voting_engine_system::Pallet::<Test>::get_proposal_data(pid2).is_some());
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
            let pid = last_proposal_id();

            for i in 0..13 {
                assert_ok!(ResolutionDestroGov::vote_destroy(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    pid,
                    true
                ));
            }

            assert_eq!(
                voting_engine_system::Pallet::<Test>::proposals(pid)
                    .expect("proposal should exist")
                    .status,
                STATUS_PASSED
            );
            assert_eq!(Balances::free_balance(&account), 1_000);
            assert!(voting_engine_system::Pallet::<Test>::get_proposal_data(pid).is_some());

            let _ = Balances::deposit_creating(&account, 200);
            assert_ok!(ResolutionDestroGov::execute_destroy(
                RuntimeOrigin::signed(nrc_admin(0)),
                pid
            ));
            assert_eq!(Balances::free_balance(&account), 100);
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
            let pid1 = last_proposal_id();

            for i in 0..13 {
                assert_ok!(ResolutionDestroGov::vote_destroy(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    pid1,
                    true
                ));
            }

            assert_ok!(ResolutionDestroGov::propose_destroy(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                50
            ));
            let pid2 = last_proposal_id();
            assert!(voting_engine_system::Pallet::<Test>::get_proposal_data(pid2).is_some());
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
            let pid = last_proposal_id();
            assert_ok!(ResolutionDestroGov::vote_destroy(
                RuntimeOrigin::signed(nrc_admin(1)),
                pid,
                true
            ));
            assert_noop!(
                ResolutionDestroGov::vote_destroy(RuntimeOrigin::signed(nrc_admin(1)), pid, true),
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
            let pid = last_proposal_id();
            for i in 0..13 {
                assert_ok!(ResolutionDestroGov::vote_destroy(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    pid,
                    true
                ));
            }
            let _ = Balances::deposit_creating(&account, 200);
            assert_ok!(ResolutionDestroGov::execute_destroy(
                RuntimeOrigin::signed(outsider),
                pid
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
