#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{ensure, pallet_prelude::*, traits::Currency, Blake2_128Concat};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::traits::{CheckedAdd, Zero};

use primitives::china::china_cb::{shenfen_id_to_fixed48 as reserve_pallet_id_to_bytes, CHINA_CB};
use primitives::china::china_ch::{
    shenfen_id_to_fixed48 as shengbank_pallet_id_to_bytes, CHINA_CH,
};
use voting_engine_system::{
    internal_vote::{ORG_NRC, ORG_PRB, ORG_PRC},
    InstitutionPalletId, STATUS_PASSED, STATUS_REJECTED,
};

pub use pallet::*;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod weights;

type BalanceOf<T> =
    <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// 转账动作：记录一次转账提案的完整业务参数。
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxRemarkLen))]
pub struct TransferAction<AccountId, Balance, MaxRemarkLen: Get<u32>> {
    /// 转出机构
    pub institution: InstitutionPalletId,
    /// 收款地址
    pub beneficiary: AccountId,
    /// 转账金额
    pub amount: Balance,
    /// 备注
    pub remark: BoundedVec<u8, MaxRemarkLen>,
    /// 发起管理员
    pub proposer: AccountId,
}

fn institution_org(institution: InstitutionPalletId) -> Option<u8> {
    if CHINA_CB
        .first()
        .and_then(|n| reserve_pallet_id_to_bytes(n.shenfen_id))
        == Some(institution)
    {
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
    use duoqian_transaction_pow::ProtectedSourceChecker;
    use frame_support::traits::ExistenceRequirement;
    use voting_engine_system::InternalAdminProvider;
    use voting_engine_system::InternalVoteEngine;

    #[pallet::config]
    pub trait Config: frame_system::Config + voting_engine_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Currency: Currency<Self::AccountId>;

        /// 备注最大长度
        #[pallet::constant]
        type MaxRemarkLen: Get<u32>;

        /// 内部投票引擎
        type InternalVoteEngine: voting_engine_system::InternalVoteEngine<Self::AccountId>;

        /// 受保护地址检查器（复用 duoqian-transaction-pow 的 trait）
        type ProtectedSourceChecker: duoqian_transaction_pow::ProtectedSourceChecker<Self::AccountId>;

        /// Weight 配置
        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn proposal_action)]
    pub type ProposalActions<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        TransferAction<T::AccountId, BalanceOf<T>, T::MaxRemarkLen>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn proposal_created_at)]
    pub type ProposalCreatedAt<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, BlockNumberFor<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn active_proposal_by_institution)]
    pub type ActiveProposalByInstitution<T: Config> =
        StorageMap<_, Blake2_128Concat, InstitutionPalletId, u64, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 转账提案已创建
        TransferProposed {
            proposal_id: u64,
            org: u8,
            institution: InstitutionPalletId,
            proposer: T::AccountId,
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// 投票已提交
        TransferVoteSubmitted {
            proposal_id: u64,
            who: T::AccountId,
            approve: bool,
        },
        /// 投票通过但执行失败（投票已记录，提案已清理，需重新发起提案）
        TransferExecutionFailed {
            proposal_id: u64,
            institution: InstitutionPalletId,
        },
        /// 转账已执行（投票通过后自动触发）
        TransferExecuted {
            proposal_id: u64,
            institution: InstitutionPalletId,
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        InvalidInstitution,
        InstitutionOrgMismatch,
        UnauthorizedAdmin,
        ZeroAmount,
        AmountBelowExistentialDeposit,
        SelfTransferNotAllowed,
        BeneficiaryIsProtectedAddress,
        ProposalActionNotFound,
        InstitutionAccountDecodeFailed,
        InsufficientBalance,
        ActiveProposalExists,
        TransferFailed,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 发起机构多签名地址转账提案。
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::propose_transfer())]
        pub fn propose_transfer(
            origin: OriginFor<T>,
            org: u8,
            institution: InstitutionPalletId,
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
            remark: BoundedVec<u8, T::MaxRemarkLen>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(amount > Zero::zero(), Error::<T>::ZeroAmount);
            let actual_org = institution_org(institution).ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(actual_org == org, Error::<T>::InstitutionOrgMismatch);
            ensure!(
                Self::is_internal_admin(org, institution, &who),
                Error::<T>::UnauthorizedAdmin
            );

            // 获取机构 duoqian_address
            let raw_account = institution_pallet_address(institution)
                .ok_or(Error::<T>::InvalidInstitution)?;
            let institution_account = T::AccountId::decode(&mut &raw_account[..])
                .map_err(|_| Error::<T>::InstitutionAccountDecodeFailed)?;

            // 转账金额必须 >= ED，防止收款地址不存在时创建失败
            let ed = T::Currency::minimum_balance();
            ensure!(amount >= ed, Error::<T>::AmountBelowExistentialDeposit);

            // 不允许自转账
            ensure!(
                beneficiary != institution_account,
                Error::<T>::SelfTransferNotAllowed
            );

            // 不允许转到受保护地址（质押地址）
            ensure!(
                !T::ProtectedSourceChecker::is_protected(&beneficiary),
                Error::<T>::BeneficiaryIsProtectedAddress
            );

            // 一机构一提案
            Self::ensure_no_active_proposal(institution)?;

            // 预检余额
            let free = T::Currency::free_balance(&institution_account);
            let required = amount
                .checked_add(&ed)
                .ok_or(Error::<T>::InsufficientBalance)?;
            ensure!(free >= required, Error::<T>::InsufficientBalance);

            // 创建内部投票提案
            let proposal_id =
                T::InternalVoteEngine::create_internal_proposal(who.clone(), org, institution)?;

            ProposalActions::<T>::insert(
                proposal_id,
                TransferAction {
                    institution,
                    beneficiary: beneficiary.clone(),
                    amount,
                    remark,
                    proposer: who.clone(),
                },
            );
            ProposalCreatedAt::<T>::insert(proposal_id, frame_system::Pallet::<T>::block_number());
            ActiveProposalByInstitution::<T>::insert(institution, proposal_id);

            Self::deposit_event(Event::<T>::TransferProposed {
                proposal_id,
                org,
                institution,
                proposer: who,
                beneficiary,
                amount,
            });
            Ok(())
        }

        /// 对转账提案投票，达到阈值后自动执行转账。
        #[pallet::call_index(1)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::vote_transfer())]
        pub fn vote_transfer(
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

            Self::deposit_event(Event::<T>::TransferVoteSubmitted {
                proposal_id,
                who,
                approve,
            });

            // 检查投票结果
            if let Some(proposal) = voting_engine_system::Pallet::<T>::proposals(proposal_id) {
                if proposal.status == STATUS_PASSED {
                    // 投票通过，尝试自动执行转账
                    let institution = action.institution;
                    if Self::execute_transfer_from_action(proposal_id, action).is_err() {
                        // 执行失败：投票已记录不回滚，清理提案让管理员重新发起
                        Self::cleanup_proposal(institution, proposal_id);
                        Self::deposit_event(Event::<T>::TransferExecutionFailed {
                            proposal_id,
                            institution,
                        });
                    }
                } else if proposal.status == STATUS_REJECTED {
                    Self::cleanup_proposal(action.institution, proposal_id);
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
            <T as voting_engine_system::Config>::InternalAdminProvider::is_internal_admin(
                org,
                institution,
                who,
            )
        }

        fn ensure_no_active_proposal(institution: InstitutionPalletId) -> DispatchResult {
            if let Some(existing_id) = ActiveProposalByInstitution::<T>::get(institution) {
                if ProposalActions::<T>::contains_key(existing_id) {
                    if let Some(proposal) =
                        voting_engine_system::Pallet::<T>::proposals(existing_id)
                    {
                        if proposal.status == STATUS_REJECTED {
                            // 已拒绝的提案可以被覆盖
                            Self::cleanup_proposal(institution, existing_id);
                            return Ok(());
                        }
                        // 仍在投票中或其他状态，不允许创建新提案
                        return Err(Error::<T>::ActiveProposalExists.into());
                    }
                }
                // 投票引擎中已不存在，清理孤儿数据
                Self::cleanup_proposal(institution, existing_id);
            }
            Ok(())
        }

        fn execute_transfer_from_action(
            proposal_id: u64,
            action: TransferAction<T::AccountId, BalanceOf<T>, T::MaxRemarkLen>,
        ) -> DispatchResult {
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

            T::Currency::transfer(
                &institution_account,
                &action.beneficiary,
                action.amount,
                ExistenceRequirement::KeepAlive,
            )
            .map_err(|_| Error::<T>::TransferFailed)?;

            Self::cleanup_proposal(action.institution, proposal_id);

            Self::deposit_event(Event::<T>::TransferExecuted {
                proposal_id,
                institution: action.institution,
                beneficiary: action.beneficiary,
                amount: action.amount,
            });
            Ok(())
        }

        fn remove_active_proposal_if_matches(institution: InstitutionPalletId, proposal_id: u64) {
            if ActiveProposalByInstitution::<T>::get(institution) == Some(proposal_id) {
                ActiveProposalByInstitution::<T>::remove(institution);
            }
        }

        fn cleanup_proposal(institution: InstitutionPalletId, proposal_id: u64) {
            ProposalActions::<T>::remove(proposal_id);
            ProposalCreatedAt::<T>::remove(proposal_id);
            Self::remove_active_proposal_if_matches(institution, proposal_id);
            T::InternalVoteEngine::cleanup_internal_proposal(proposal_id);
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
    use voting_engine_system::STATUS_REJECTED;

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
        pub type DuoqianTransferPow = super;
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

    pub struct TestProtectedSourceChecker;
    impl duoqian_transaction_pow::ProtectedSourceChecker<AccountId32>
        for TestProtectedSourceChecker
    {
        fn is_protected(_address: &AccountId32) -> bool {
            false
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
        type MaxJointDecisionApprovals = ConstU32<32>;
        type SfidEligibility = TestSfidEligibility;
        type PopulationSnapshotVerifier = TestPopulationSnapshotVerifier;
        type JointVoteResultCallback = ();
        type InternalAdminProvider = TestInternalAdminProvider;
        type JointInstitutionDecisionVerifier = ();
        type WeightInfo = ();
    }

    impl pallet::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type Currency = Balances;
        type MaxRemarkLen = ConstU32<256>;
        type InternalVoteEngine = voting_engine_system::Pallet<Test>;
        type ProtectedSourceChecker = TestProtectedSourceChecker;
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

    /// 收款人：使用一个不是管理员也不是机构的普通地址
    fn beneficiary() -> AccountId32 {
        AccountId32::new([99u8; 32])
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let mut storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("test storage should build");

        let balances = vec![
            (institution_account(nrc_pallet_id()), 10_000),
            (institution_account(prc_pallet_id()), 10_000),
            (institution_account(prb_pallet_id()), 10_000),
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
    fn nrc_transfer_executes_when_yes_votes_reach_threshold() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let inst_account = institution_account(institution);
            let dest = beneficiary();

            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                1_000,
                BoundedVec::default(),
            ));

            for i in 0..13 {
                assert_ok!(DuoqianTransferPow::vote_transfer(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    0,
                    true
                ));
            }

            // 转账已执行
            assert_eq!(Balances::free_balance(&inst_account), 9_000);
            assert_eq!(Balances::free_balance(&dest), 1_000);
            // 提案已清理
            assert!(DuoqianTransferPow::proposal_action(0).is_none());
            assert!(voting_engine_system::Pallet::<Test>::proposals(0).is_none());
            assert!(ActiveProposalByInstitution::<Test>::get(institution).is_none());
        });
    }

    #[test]
    fn prc_transfer_executes_when_yes_votes_reach_threshold() {
        new_test_ext().execute_with(|| {
            let institution = prc_pallet_id();
            let inst_account = institution_account(institution);
            let dest = beneficiary();

            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(prc_admin(0)),
                ORG_PRC,
                institution,
                dest.clone(),
                2_000,
                BoundedVec::default(),
            ));

            for i in 0..6 {
                assert_ok!(DuoqianTransferPow::vote_transfer(
                    RuntimeOrigin::signed(prc_admin(i)),
                    0,
                    true
                ));
            }

            assert_eq!(Balances::free_balance(&inst_account), 8_000);
            assert_eq!(Balances::free_balance(&dest), 2_000);
            assert!(DuoqianTransferPow::proposal_action(0).is_none());
        });
    }

    #[test]
    fn prb_transfer_executes_when_yes_votes_reach_threshold() {
        new_test_ext().execute_with(|| {
            let institution = prb_pallet_id();
            let inst_account = institution_account(institution);
            let dest = beneficiary();

            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(prb_admin(0)),
                ORG_PRB,
                institution,
                dest.clone(),
                3_000,
                BoundedVec::default(),
            ));

            for i in 0..6 {
                assert_ok!(DuoqianTransferPow::vote_transfer(
                    RuntimeOrigin::signed(prb_admin(i)),
                    0,
                    true
                ));
            }

            assert_eq!(Balances::free_balance(&inst_account), 7_000);
            assert_eq!(Balances::free_balance(&dest), 3_000);
            assert!(DuoqianTransferPow::proposal_action(0).is_none());
        });
    }

    #[test]
    fn non_admin_cannot_propose_or_vote() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let dest = beneficiary();

            // PRC 管理员不能给 NRC 提案
            assert_noop!(
                DuoqianTransferPow::propose_transfer(
                    RuntimeOrigin::signed(prc_admin(0)),
                    ORG_NRC,
                    institution,
                    dest.clone(),
                    100,
                    BoundedVec::default(),
                ),
                Error::<Test>::UnauthorizedAdmin
            );

            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest,
                100,
                BoundedVec::default(),
            ));

            // PRC 管理员不能给 NRC 投票
            assert_noop!(
                DuoqianTransferPow::vote_transfer(RuntimeOrigin::signed(prc_admin(0)), 0, true),
                Error::<Test>::UnauthorizedAdmin
            );
        });
    }

    #[test]
    fn zero_amount_is_rejected() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let dest = beneficiary();

            assert_noop!(
                DuoqianTransferPow::propose_transfer(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    ORG_NRC,
                    institution,
                    dest,
                    0,
                    BoundedVec::default(),
                ),
                Error::<Test>::ZeroAmount
            );
        });
    }

    #[test]
    fn self_transfer_is_rejected() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let inst_account = institution_account(institution);

            assert_noop!(
                DuoqianTransferPow::propose_transfer(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    ORG_NRC,
                    institution,
                    inst_account,
                    100,
                    BoundedVec::default(),
                ),
                Error::<Test>::SelfTransferNotAllowed
            );
        });
    }

    #[test]
    fn insufficient_balance_is_rejected_on_propose() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let dest = beneficiary();

            // 余额 10_000，ED=1，最多只能提案 9_999
            assert_noop!(
                DuoqianTransferPow::propose_transfer(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    ORG_NRC,
                    institution,
                    dest,
                    10_000,
                    BoundedVec::default(),
                ),
                Error::<Test>::InsufficientBalance
            );
        });
    }

    #[test]
    fn duplicate_vote_is_rejected() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let dest = beneficiary();

            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest,
                100,
                BoundedVec::default(),
            ));
            assert_ok!(DuoqianTransferPow::vote_transfer(
                RuntimeOrigin::signed(nrc_admin(1)),
                0,
                true
            ));
            assert_noop!(
                DuoqianTransferPow::vote_transfer(RuntimeOrigin::signed(nrc_admin(1)), 0, true),
                voting_engine_system::pallet::Error::<Test>::AlreadyVoted
            );
        });
    }

    #[test]
    fn active_proposal_blocks_new_proposal() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let dest = beneficiary();

            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                100,
                BoundedVec::default(),
            ));

            assert_noop!(
                DuoqianTransferPow::propose_transfer(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    ORG_NRC,
                    institution,
                    dest,
                    200,
                    BoundedVec::default(),
                ),
                Error::<Test>::ActiveProposalExists
            );
        });
    }

    #[test]
    fn executed_transfer_does_not_block_new_proposal() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let dest = beneficiary();

            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                100,
                BoundedVec::default(),
            ));

            for i in 0..13 {
                assert_ok!(DuoqianTransferPow::vote_transfer(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    0,
                    true
                ));
            }

            // 转账已执行，可以创建新提案
            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest,
                200,
                BoundedVec::default(),
            ));
            assert_eq!(
                ActiveProposalByInstitution::<Test>::get(institution),
                Some(1)
            );
        });
    }

    #[test]
    fn rejected_proposal_does_not_block_new_proposal() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let dest = beneficiary();

            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                100,
                BoundedVec::default(),
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

            // 被拒绝后可以创建新提案
            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest,
                50,
                BoundedVec::default(),
            ));
            assert_eq!(
                ActiveProposalByInstitution::<Test>::get(institution),
                Some(1)
            );
        });
    }

    #[test]
    fn existential_deposit_is_preserved() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let inst_account = institution_account(institution);
            let dest = beneficiary();

            // 余额 10_000，ED=1，提案 9_999 应该成功
            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                9_999,
                BoundedVec::default(),
            ));

            for i in 0..13 {
                assert_ok!(DuoqianTransferPow::vote_transfer(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    0,
                    true
                ));
            }

            assert_eq!(Balances::free_balance(&inst_account), 1);
            assert_eq!(Balances::free_balance(&dest), 9_999);
        });
    }
}
