#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{ensure, pallet_prelude::*, traits::Currency};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::traits::{CheckedAdd, SaturatedConversion, Zero};

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
    use frame_support::traits::OnUnbalanced;
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
        type ProtectedSourceChecker: duoqian_transaction_pow::ProtectedSourceChecker<
            Self::AccountId,
        >;

        /// 手续费分账路由（复用 PowOnchainFeeRouter）
        type FeeRouter: frame_support::traits::OnUnbalanced<
            <Self::Currency as Currency<Self::AccountId>>::NegativeImbalance,
        >;

        /// Weight 配置
        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // 活跃提案数限制已移至 voting-engine-system::active_proposal_limit 全局管控。
    // 提案业务数据和元数据已统一存储到 voting-engine-system（ProposalData / ProposalMeta）。

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
        /// 投票通过但执行失败（投票已记录，提案数据保留，可通过 execute_transfer 手动重试）
        TransferExecutionFailed {
            proposal_id: u64,
            institution: InstitutionPalletId,
        },
        /// 转账已执行（投票通过后自动触发，含手续费分账）
        TransferExecuted {
            proposal_id: u64,
            institution: InstitutionPalletId,
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
            fee: BalanceOf<T>,
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
        ProposalNotPassed,
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
            let raw_account =
                institution_pallet_address(institution).ok_or(Error::<T>::InvalidInstitution)?;
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

            // 活跃提案数由 voting-engine-system 在 create_internal_proposal 中统一检查

            // 预检余额（含手续费，与执行时检查一致，避免创建必定无法执行的提案）
            let amount_u128: u128 = amount.saturated_into();
            let fee_u128 = onchain_transaction_pow::calculate_onchain_fee(amount_u128);
            let fee: BalanceOf<T> = fee_u128.saturated_into();
            let total = amount
                .checked_add(&fee)
                .ok_or(Error::<T>::InsufficientBalance)?;
            let free = T::Currency::free_balance(&institution_account);
            let required = total
                .checked_add(&ed)
                .ok_or(Error::<T>::InsufficientBalance)?;
            ensure!(free >= required, Error::<T>::InsufficientBalance);

            // 创建内部投票提案
            let proposal_id =
                T::InternalVoteEngine::create_internal_proposal(who.clone(), org, institution)?;

            let action = TransferAction {
                institution,
                beneficiary: beneficiary.clone(),
                amount,
                remark,
                proposer: who.clone(),
            };
            let data = action.encode();
            voting_engine_system::Pallet::<T>::store_proposal_data(proposal_id, data)?;
            voting_engine_system::Pallet::<T>::store_proposal_meta(
                proposal_id,
                frame_system::Pallet::<T>::block_number(),
            );

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

            let data = voting_engine_system::Pallet::<T>::get_proposal_data(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            let action = TransferAction::<T::AccountId, BalanceOf<T>, T::MaxRemarkLen>::decode(
                &mut &data[..],
            )
            .map_err(|_| Error::<T>::ProposalActionNotFound)?;
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
                    if Self::try_execute_transfer(proposal_id).is_err() {
                        Self::deposit_event(Event::<T>::TransferExecutionFailed {
                            proposal_id,
                            institution,
                        });
                    }
                }
            }
            Ok(())
        }

        /// 手动执行已通过的转账提案。
        ///
        /// 当投票通过后自动执行失败（如余额不足），可在补充余额后通过此接口重试。
        /// 任何签名账户都可调用，避免因管理员离线导致通过的提案无法落地。
        #[pallet::call_index(2)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::execute_transfer())]
        pub fn execute_transfer(origin: OriginFor<T>, proposal_id: u64) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            Self::try_execute_transfer(proposal_id)
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

        /// 从 voting-engine-system 读取提案数据并执行转账。
        /// vote_transfer（自动执行）和 execute_transfer（手动重试）共用此逻辑。
        fn try_execute_transfer(proposal_id: u64) -> DispatchResult {
            let proposal = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            ensure!(
                proposal.status == STATUS_PASSED,
                Error::<T>::ProposalNotPassed
            );

            let data = voting_engine_system::Pallet::<T>::get_proposal_data(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            let action = TransferAction::<T::AccountId, BalanceOf<T>, T::MaxRemarkLen>::decode(
                &mut &data[..],
            )
            .map_err(|_| Error::<T>::ProposalActionNotFound)?;

            let raw_account = institution_pallet_address(action.institution)
                .ok_or(Error::<T>::InvalidInstitution)?;
            let institution_account = T::AccountId::decode(&mut &raw_account[..])
                .map_err(|_| Error::<T>::InstitutionAccountDecodeFailed)?;

            // ── 计算手续费（复用 onchain-transaction-pow 公共接口） ──
            let amount_u128: u128 = action.amount.saturated_into();
            let fee_u128 = onchain_transaction_pow::calculate_onchain_fee(amount_u128);
            let fee: BalanceOf<T> = fee_u128.saturated_into();
            let total = action
                .amount
                .checked_add(&fee)
                .ok_or(Error::<T>::InsufficientBalance)?;

            // ── 余额检查：需要 total + ED ──
            let free = T::Currency::free_balance(&institution_account);
            let ed = T::Currency::minimum_balance();
            let required = total
                .checked_add(&ed)
                .ok_or(Error::<T>::InsufficientBalance)?;
            ensure!(free >= required, Error::<T>::InsufficientBalance);

            // ── 执行转账 ──
            T::Currency::transfer(
                &institution_account,
                &action.beneficiary,
                action.amount,
                ExistenceRequirement::KeepAlive,
            )
            .map_err(|_| Error::<T>::TransferFailed)?;

            // ── 手续费：从机构账户扣取，通过 FeeRouter 按现有规则分账 ──
            let fee_imbalance = T::Currency::withdraw(
                &institution_account,
                fee,
                frame_support::traits::WithdrawReasons::FEE,
                ExistenceRequirement::KeepAlive,
            )
            .map_err(|_| Error::<T>::InsufficientBalance)?;
            T::FeeRouter::on_unbalanced(fee_imbalance);

            // ── 标记为已执行，防止双重执行 ──
            voting_engine_system::Pallet::<T>::set_status_and_emit(proposal_id, STATUS_EXECUTED)?;

            Self::deposit_event(Event::<T>::TransferExecuted {
                proposal_id,
                institution: action.institution,
                beneficiary: action.beneficiary,
                amount: action.amount,
                fee,
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

    thread_local! {
        static PROTECTED_ADDRESS: core::cell::RefCell<Option<AccountId32>> = core::cell::RefCell::new(None);
    }

    pub struct TestProtectedSourceChecker;
    impl duoqian_transaction_pow::ProtectedSourceChecker<AccountId32> for TestProtectedSourceChecker {
        fn is_protected(address: &AccountId32) -> bool {
            PROTECTED_ADDRESS.with(|pa| pa.borrow().as_ref() == Some(address))
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
        type MaxJointDecisionApprovals = ConstU32<32>;
        type SfidEligibility = TestSfidEligibility;
        type PopulationSnapshotVerifier = TestPopulationSnapshotVerifier;
        type JointVoteResultCallback = ();
        type InternalAdminProvider = TestInternalAdminProvider;
        type InternalThresholdProvider = ();
        type MaxProposalDataLen = ConstU32<1024>;
        type MaxProposalObjectLen = ConstU32<{ 10 * 1024 }>;
        type JointInstitutionDecisionVerifier = ();
        type TimeProvider = TestTimeProvider;
        type WeightInfo = ();
    }

    impl pallet::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type Currency = Balances;
        type MaxRemarkLen = ConstU32<256>;
        type InternalVoteEngine = voting_engine_system::Pallet<Test>;
        type ProtectedSourceChecker = TestProtectedSourceChecker;
        type FeeRouter = ();
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

    /// 获取最近一次 create_internal_proposal 分配的 proposal_id。
    fn last_proposal_id() -> u64 {
        voting_engine_system::Pallet::<Test>::next_proposal_id().saturating_sub(1)
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
            let pid = last_proposal_id();

            for i in 0..13 {
                assert_ok!(DuoqianTransferPow::vote_transfer(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    pid,
                    true
                ));
            }

            // 转账已执行（含手续费 10）
            assert_eq!(Balances::free_balance(&inst_account), 8_990);
            assert_eq!(Balances::free_balance(&dest), 1_000);
            // 提案数据仍保留（由 voting-engine-system 延迟清理）
            assert!(voting_engine_system::Pallet::<Test>::get_proposal_data(pid).is_some());
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
            let pid = last_proposal_id();

            for i in 0..6 {
                assert_ok!(DuoqianTransferPow::vote_transfer(
                    RuntimeOrigin::signed(prc_admin(i)),
                    pid,
                    true
                ));
            }

            assert_eq!(Balances::free_balance(&inst_account), 7_990);
            assert_eq!(Balances::free_balance(&dest), 2_000);
            assert!(voting_engine_system::Pallet::<Test>::get_proposal_data(pid).is_some());
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
            let pid = last_proposal_id();

            for i in 0..6 {
                assert_ok!(DuoqianTransferPow::vote_transfer(
                    RuntimeOrigin::signed(prb_admin(i)),
                    pid,
                    true
                ));
            }

            assert_eq!(Balances::free_balance(&inst_account), 6_990);
            assert_eq!(Balances::free_balance(&dest), 3_000);
            assert!(voting_engine_system::Pallet::<Test>::get_proposal_data(pid).is_some());
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
            let pid = last_proposal_id();

            // PRC 管理员不能给 NRC 投票
            assert_noop!(
                DuoqianTransferPow::vote_transfer(RuntimeOrigin::signed(prc_admin(0)), pid, true),
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

            // 余额 10_000，fee=10，ED=1：最多 amount=9_989（9_989+10+1=10_000）
            // amount=9_990 时 required=9_990+10+1=10_001 > 10_000 → 拒绝
            assert_noop!(
                DuoqianTransferPow::propose_transfer(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    ORG_NRC,
                    institution,
                    dest.clone(),
                    9_990,
                    BoundedVec::default(),
                ),
                Error::<Test>::InsufficientBalance
            );

            // amount=9_989 时 required=9_989+10+1=10_000 → 刚好通过
            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest,
                9_989,
                BoundedVec::default(),
            ));
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
            let pid = last_proposal_id();
            assert_ok!(DuoqianTransferPow::vote_transfer(
                RuntimeOrigin::signed(nrc_admin(1)),
                pid,
                true
            ));
            assert_noop!(
                DuoqianTransferPow::vote_transfer(RuntimeOrigin::signed(nrc_admin(1)), pid, true),
                voting_engine_system::pallet::Error::<Test>::AlreadyVoted
            );
        });
    }

    #[test]
    fn multiple_proposals_allowed_within_limit() {
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

            // 活跃提案数限制由 voting-engine-system 全局管控（上限 10），第二个提案可以成功
            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest,
                200,
                BoundedVec::default(),
            ));
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
            let pid1 = last_proposal_id();

            for i in 0..13 {
                assert_ok!(DuoqianTransferPow::vote_transfer(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    pid1,
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

            // 被拒绝后可以创建新提案
            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest,
                50,
                BoundedVec::default(),
            ));
        });
    }

    #[test]
    fn existential_deposit_is_preserved() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let inst_account = institution_account(institution);
            let dest = beneficiary();

            // 余额 10_000，ED=1，手续费=10，提案 9_989 刚好使剩余 = ED
            // required = 9_989 + 10(fee) + 1(ED) = 10_000
            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                9_989,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            for i in 0..13 {
                assert_ok!(DuoqianTransferPow::vote_transfer(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    pid,
                    true
                ));
            }

            assert_eq!(Balances::free_balance(&inst_account), 1);
            assert_eq!(Balances::free_balance(&dest), 9_989);
        });
    }

    #[test]
    fn execute_transfer_succeeds_after_failed_auto_execution() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let inst_account = institution_account(institution);
            let dest = beneficiary();

            // 余额 10_000，提案 9_990（required=9_990+10+1=10_001>10_000）
            // propose 预检也含手续费，所以 9_990 会被拒绝
            // 先用 9_989 创建提案（刚好通过预检），然后手动减余额使执行失败
            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                9_000,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            // 在投票前减少余额，使自动执行失败
            // 转走 9_000 使余额仅剩 1_000，不够 amount(9_000)+fee(10)+ED(1)=9_011
            let drain_dest = AccountId32::new([88u8; 32]);
            let _ = Balances::deposit_creating(&drain_dest, 1);
            assert_ok!(<Balances as frame_support::traits::Currency<_>>::transfer(
                &inst_account,
                &drain_dest,
                9_000,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            ));
            assert_eq!(Balances::free_balance(&inst_account), 1_000);

            // 投票通过，自动执行因余额不足失败
            for i in 0..13 {
                assert_ok!(DuoqianTransferPow::vote_transfer(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    pid,
                    true
                ));
            }

            // 提案状态为 PASSED，但转账未执行
            assert_eq!(
                voting_engine_system::Pallet::<Test>::proposals(pid)
                    .expect("proposal should exist")
                    .status,
                STATUS_PASSED
            );
            assert_eq!(Balances::free_balance(&dest), 0);
            // 提案数据仍保留
            assert!(voting_engine_system::Pallet::<Test>::get_proposal_data(pid).is_some());

            // 补充余额后手动执行
            let _ = Balances::deposit_creating(&inst_account, 9_000);
            assert_eq!(Balances::free_balance(&inst_account), 10_000);
            assert_ok!(DuoqianTransferPow::execute_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                pid
            ));
            // 转账成功：9_000 转出 + 10 手续费
            assert_eq!(Balances::free_balance(&inst_account), 990);
            assert_eq!(Balances::free_balance(&dest), 9_000);
        });
    }

    #[test]
    fn execute_transfer_rejects_non_passed_proposal() {
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
            let pid = last_proposal_id();

            // 提案仍在投票中，不能手动执行
            assert_noop!(
                DuoqianTransferPow::execute_transfer(RuntimeOrigin::signed(nrc_admin(0)), pid),
                Error::<Test>::ProposalNotPassed
            );
        });
    }

    #[test]
    fn execute_transfer_is_callable_by_non_admin() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let inst_account = institution_account(institution);
            let dest = beneficiary();
            let outsider = AccountId32::new([88u8; 32]);
            let _ = Balances::deposit_creating(&outsider, 1);

            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                100,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            // 减余额使自动执行失败
            let drain_dest = AccountId32::new([77u8; 32]);
            let _ = Balances::deposit_creating(&drain_dest, 1);
            assert_ok!(<Balances as frame_support::traits::Currency<_>>::transfer(
                &inst_account,
                &drain_dest,
                9_900,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            ));

            for i in 0..13 {
                assert_ok!(DuoqianTransferPow::vote_transfer(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    pid,
                    true
                ));
            }

            // 自动执行失败，补充余额
            assert_eq!(Balances::free_balance(&dest), 0);
            let _ = Balances::deposit_creating(&inst_account, 10_000);

            // 非管理员也能调用 execute_transfer
            assert_ok!(DuoqianTransferPow::execute_transfer(
                RuntimeOrigin::signed(outsider),
                pid
            ));
            assert_eq!(Balances::free_balance(&dest), 100);
        });
    }

    #[test]
    fn executed_transfer_cannot_be_executed_again() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let dest = beneficiary();

            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest,
                1_000,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            for i in 0..13 {
                assert_ok!(DuoqianTransferPow::vote_transfer(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    pid,
                    true
                ));
            }

            // 自动执行成功，状态变为 EXECUTED
            assert_eq!(
                voting_engine_system::Pallet::<Test>::proposals(pid)
                    .expect("proposal should exist")
                    .status,
                STATUS_EXECUTED
            );

            // 再次调用 execute_transfer 应被拒绝
            assert_noop!(
                DuoqianTransferPow::execute_transfer(RuntimeOrigin::signed(nrc_admin(0)), pid),
                Error::<Test>::ProposalNotPassed
            );
        });
    }

    #[test]
    fn protected_address_is_rejected() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let protected = AccountId32::new([77u8; 32]);

            // 标记为受保护地址
            PROTECTED_ADDRESS.with(|pa| *pa.borrow_mut() = Some(protected.clone()));

            assert_noop!(
                DuoqianTransferPow::propose_transfer(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    ORG_NRC,
                    institution,
                    protected,
                    100,
                    BoundedVec::default(),
                ),
                Error::<Test>::BeneficiaryIsProtectedAddress
            );
        });
    }

    #[test]
    fn fee_respects_minimum_on_small_amount() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let inst_account = institution_account(institution);
            let dest = beneficiary();

            // amount=1, 费率计算 1×0.1%=0.001 < 最低 10 分，手续费应为 10
            // required = 1 + 10 + 1(ED) = 12
            assert_ok!(DuoqianTransferPow::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                1,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            for i in 0..13 {
                assert_ok!(DuoqianTransferPow::vote_transfer(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    pid,
                    true
                ));
            }

            // 余额 10_000 - 1(转账) - 10(最低手续费) = 9_989
            assert_eq!(Balances::free_balance(&inst_account), 9_989);
            assert_eq!(Balances::free_balance(&dest), 1);
        });
    }
}
