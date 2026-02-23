#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{ensure, pallet_prelude::*, traits::Currency, Blake2_128Concat};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::traits::Zero;

use primitives::china::china_cb::{
    shenfen_id_to_fixed48 as reserve_pallet_id_to_bytes, CHINA_CB,
};
use primitives::china::china_ch::{
    shenfen_id_to_fixed48 as shengbank_pallet_id_to_bytes, CHINA_CH,
};
use voting_engine_system::{
    internal_vote::{ORG_NRC, ORG_PRB, ORG_PRC},
    InstitutionPalletId, STATUS_PASSED,
};

pub use pallet::*;

type BalanceOf<T> =
    <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct DestroyAction<Balance> {
    /// 目标机构（机构标识 pallet_id）
    pub institution: InstitutionPalletId,
    /// 销毁数量
    pub amount: Balance,
    /// 是否已执行
    pub executed: bool,
}

fn str_to_pallet_id(s: &str) -> Option<InstitutionPalletId> {
    reserve_pallet_id_to_bytes(s)
}

fn str_to_shengbank_pallet_id(s: &str) -> Option<InstitutionPalletId> {
    shengbank_pallet_id_to_bytes(s)
}

fn nrc_pallet_id_bytes() -> InstitutionPalletId {
    CHINA_CB
        .first()
        .and_then(|n| reserve_pallet_id_to_bytes(n.shenfen_id))
        .expect("NRC shenfen_id must be valid")
}

fn institution_org(institution: InstitutionPalletId) -> Option<u8> {
    if institution == nrc_pallet_id_bytes() {
        return Some(ORG_NRC);
    }

    if CHINA_CB
        .iter()
        .skip(1)
        .filter_map(|n| str_to_pallet_id(n.shenfen_id))
        .any(|pid| pid == institution)
    {
        return Some(ORG_PRC);
    }

    if CHINA_CH
        .iter()
        .filter_map(|n| str_to_shengbank_pallet_id(n.shenfen_id))
        .any(|pid| pid == institution)
    {
        return Some(ORG_PRB);
    }

    None
}

fn institution_pallet_address(institution: InstitutionPalletId) -> Option<[u8; 32]> {
    if let Some(node) = CHINA_CB
        .iter()
        .find(|n| str_to_pallet_id(n.shenfen_id) == Some(institution))
    {
        return Some(node.duoqian_address);
    }

    CHINA_CH
        .iter()
        .find(|n| str_to_shengbank_pallet_id(n.shenfen_id) == Some(institution))
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

        /// 中文注释：通过统一内部投票引擎创建提案，返回真实 proposal_id。
        type InternalVoteEngine: voting_engine_system::InternalVoteEngine<Self::AccountId>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn proposal_action)]
    pub type ProposalActions<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, DestroyAction<BalanceOf<T>>, OptionQuery>;

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
        ProposalAlreadyExecuted,
        InstitutionAccountDecodeFailed,
        InsufficientBalance,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 发起“决议销毁”内部投票提案。
        #[pallet::call_index(0)]
        #[pallet::weight(T::DbWeight::get().reads_writes(4, 2))]
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
            ensure!(
                Self::is_internal_admin(org, institution, &who),
                Error::<T>::UnauthorizedAdmin
            );

            let proposal_id =
                T::InternalVoteEngine::create_internal_proposal(who.clone(), org, institution)?;

            ProposalActions::<T>::insert(
                proposal_id,
                DestroyAction {
                    institution,
                    amount,
                    executed: false,
                },
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
        #[pallet::weight(T::DbWeight::get().reads_writes(6, 5))]
        pub fn vote_destroy(
            origin: OriginFor<T>,
            proposal_id: u64,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let action =
                ProposalActions::<T>::get(proposal_id).ok_or(Error::<T>::ProposalActionNotFound)?;
            ensure!(!action.executed, Error::<T>::ProposalAlreadyExecuted);
            let org = institution_org(action.institution).ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                Self::is_internal_admin(org, action.institution, &who),
                Error::<T>::UnauthorizedAdmin
            );

            voting_engine_system::Pallet::<T>::internal_vote(
                frame_system::RawOrigin::Signed(who.clone()).into(),
                proposal_id,
                approve,
            )?;

            Self::deposit_event(Event::<T>::DestroyVoteSubmitted {
                proposal_id,
                who,
                approve,
            });

            if approve {
                if let Some(proposal) = voting_engine_system::Pallet::<T>::proposals(proposal_id) {
                    if proposal.status == STATUS_PASSED {
                        Self::try_execute_destroy(proposal_id)?;
                    }
                }
            }
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn is_internal_admin(
            org: u8,
            institution: InstitutionPalletId,
            who: &T::AccountId,
        ) -> bool {
            // 中文注释：生产环境仅信任动态管理员来源（链上治理替换后的最终状态）。
            #[cfg(not(test))]
            {
                <T as voting_engine_system::Config>::InternalAdminProvider::is_internal_admin(
                    org,
                    institution,
                    who,
                )
            }
            // 中文注释：单测环境允许回退到常量管理员，便于独立测试本 pallet。
            #[cfg(test)]
            {
                if <T as voting_engine_system::Config>::InternalAdminProvider::is_internal_admin(
                    org,
                    institution,
                    who,
                ) {
                    return true;
                }

                let who_bytes = who.encode();
                if who_bytes.len() != 32 {
                    return false;
                }
                let mut who_arr = [0u8; 32];
                who_arr.copy_from_slice(&who_bytes);

                match org {
                    ORG_NRC | ORG_PRC => CHINA_CB
                        .iter()
                        .find(|n| str_to_pallet_id(n.shenfen_id) == Some(institution))
                        .map(|n| n.admins.iter().any(|admin| *admin == who_arr))
                        .unwrap_or(false),
                    ORG_PRB => CHINA_CH
                        .iter()
                        .find(|n| str_to_shengbank_pallet_id(n.shenfen_id) == Some(institution))
                        .map(|n| n.admins.iter().any(|admin| *admin == who_arr))
                        .unwrap_or(false),
                    _ => false,
                }
            }
        }

        fn try_execute_destroy(proposal_id: u64) -> DispatchResult {
            let action =
                ProposalActions::<T>::get(proposal_id).ok_or(Error::<T>::ProposalActionNotFound)?;
            ensure!(!action.executed, Error::<T>::ProposalAlreadyExecuted);

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
            ensure!(free >= action.amount, Error::<T>::InsufficientBalance);

            // 中文注释：slash 会同步减少总发行量，实现链上“销毁”。
            let (_imbalance, remaining) = T::Currency::slash(&institution_account, action.amount);
            ensure!(remaining.is_zero(), Error::<T>::InsufficientBalance);

            ProposalActions::<T>::mutate(proposal_id, |maybe| {
                if let Some(inner) = maybe {
                    inner.executed = true;
                }
            });

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
        fn is_internal_admin(
            org: u8,
            institution: InstitutionPalletId,
            who: &AccountId32,
        ) -> bool {
            let who_bytes = who.encode();
            if who_bytes.len() != 32 {
                return false;
            }
            let mut who_arr = [0u8; 32];
            who_arr.copy_from_slice(&who_bytes);
            match org {
                ORG_NRC | ORG_PRC => CHINA_CB
                    .iter()
                    .find(|n| str_to_pallet_id(n.shenfen_id) == Some(institution))
                    .map(|n| n.admins.iter().any(|admin| *admin == who_arr))
                    .unwrap_or(false),
                ORG_PRB => CHINA_CH
                    .iter()
                    .find(|n| str_to_shengbank_pallet_id(n.shenfen_id) == Some(institution))
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
        type InternalVoteEngine = voting_engine_system::Pallet<Test>;
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

            assert_eq!(Balances::free_balance(account), 900);
            let action = ResolutionDestroGov::proposal_action(0).expect("action should exist");
            assert!(action.executed);
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

            assert_eq!(Balances::free_balance(account), 800);
            let action = ResolutionDestroGov::proposal_action(0).expect("action should exist");
            assert!(action.executed);
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

            assert_eq!(Balances::free_balance(account), 700);
            let action = ResolutionDestroGov::proposal_action(0).expect("action should exist");
            assert!(action.executed);
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

            let err =
                ResolutionDestroGov::vote_destroy(RuntimeOrigin::signed(nrc_admin(12)), 0, true)
                    .expect_err("insufficient balance should fail");
            assert_eq!(err, Error::<Test>::InsufficientBalance.into());
        });
    }
}
