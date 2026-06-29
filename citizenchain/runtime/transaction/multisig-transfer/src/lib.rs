//! # 多签资金账户转账模块 (multisig-transfer)
//!
//! 本模块为治理机构(NRC/PRC/PRB)、注册机构多签账户和个人多签账户提供链上转账治理流程：
//! - 管理员发起转账提案，经内部投票通过后自动执行转账并扣取手续费。
//! - 自动执行失败时保留提案状态，可通过 `VotingEngine::retry_passed_proposal` 手动重试。
//! - 余额在提案创建和执行两个时点双重检查，含手续费和 ED 保留。
//! - 收款地址不能是转出资金账户自身，也不能是受保护地址(质押地址等)。
//! - 本模块只处理转账提案与执行；个人多签生命周期归 `personal-manage`，
//!   个人多签管理员真源归 `personal-admins`。

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{ensure, pallet_prelude::*, traits::Currency, BoundedVec};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::traits::{CheckedAdd, SaturatedConversion, Zero};

extern crate alloc;

use primitives::cid::china::china_cb::{CHINA_CB, SAFETY_FUND_ACCOUNT};
use primitives::cid::china::china_ch::CHINA_CH;
use votingengine::{
    types::{is_institution_code, InstitutionCode, NRC, PMUL, PRB, PRC},
    InternalVoteResultCallback, ProposalExecutionOutcome, STATUS_PASSED,
};

pub use pallet::*;
/// 模块标识前缀，用于在 ProposalData 中区分不同业务模块，防止跨模块误解码。
pub const MODULE_TAG: &[u8] = b"multisig-transfer";
const SAFETY_FUND_OWNER_DATA: &[u8] = b"multisig-transfer:safety";
const SWEEP_OWNER_DATA: &[u8] = b"multisig-transfer:sweep";

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod weights;

type BalanceOf<T> =
    <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// 转账动作：记录一次转账提案的完整业务参数。
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxRemarkLen))]
pub struct TransferAction<AccountId, Balance, MaxRemarkLen: Get<u32>> {
    /// 转出多签资金账户(治理机构主账户、注册机构账户或个人多签账户)。
    pub institution: AccountId,
    /// 收款地址
    pub beneficiary: AccountId,
    /// 转账金额
    pub amount: Balance,
    /// 备注
    pub remark: BoundedVec<u8, MaxRemarkLen>,
    /// 发起管理员
    pub proposer: AccountId,
}

/// 安全基金转账动作：从国储会安全基金账户向指定收款地址转账。
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxRemarkLen))]
pub struct SafetyFundAction<AccountId, Balance, MaxRemarkLen: Get<u32>> {
    /// 收款地址
    pub beneficiary: AccountId,
    /// 转账金额
    pub amount: Balance,
    /// 备注
    pub remark: BoundedVec<u8, MaxRemarkLen>,
    /// 发起管理员
    pub proposer: AccountId,
}

/// 手续费划转动作：从机构手续费账户向机构主账户划转。
///
/// `proposer` 字段与 transfer / safety_fund 两类动作对齐,便于 Executor 在
/// 投票通过 / 否决回调时统一识别提案发起人。
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct SweepAction<AccountId, Balance> {
    /// 机构资金账户。
    ///
    /// 中文注释：费用账户划转只服务治理机构费用账户，不接入个人多签。
    pub institution: AccountId,
    /// 划转金额
    pub amount: Balance,
    /// 发起管理员(Tx 1 中锁定)
    pub proposer: AccountId,
}

/// 手续费账户最低保留余额：1111.11 元（111111 分）。
const FEE_ADDRESS_MIN_RESERVE_FEN: u128 = 111_111;

/// 单次划转上限：可用余额的 80%。
const FEE_SWEEP_MAX_PERCENT: u128 = 80;

fn decode_raw_account<T: frame_system::Config>(raw: &[u8; 32]) -> Option<T::AccountId> {
    T::AccountId::decode(&mut &raw[..]).ok()
}

fn raw_account_matches<T: frame_system::Config>(raw: &[u8; 32], account: &T::AccountId) -> bool {
    decode_raw_account::<T>(raw).as_ref() == Some(account)
}

/// 中文注释：判断内置机构属于 NRC/PRC/PRB；注册多签由链上存储判断。
fn builtin_org<T: frame_system::Config>(institution: &T::AccountId) -> Option<InstitutionCode> {
    if CHINA_CB
        .first()
        .map(|n| raw_account_matches::<T>(&n.main_account, institution))
        .unwrap_or(false)
    {
        return Some(NRC);
    }

    if CHINA_CB
        .iter()
        .skip(1)
        .any(|n| raw_account_matches::<T>(&n.main_account, institution))
    {
        return Some(PRC);
    }

    if CHINA_CH
        .iter()
        .any(|n| raw_account_matches::<T>(&n.main_account, institution))
    {
        return Some(PRB);
    }

    None
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;
    use entity_primitives::ProtectedSourceChecker;
    use frame_support::traits::ExistenceRequirement;
    use frame_support::traits::OnUnbalanced;
    use institution_asset::{InstitutionAsset, InstitutionAssetAction};
    use votingengine::InternalAdminProvider;
    use votingengine::InternalVoteEngine;

    #[pallet::config]
    pub trait Config: frame_system::Config + votingengine::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 链上基础货币。
        type Currency: Currency<Self::AccountId>;

        /// 内部投票引擎。
        type InternalVoteEngine: votingengine::InternalVoteEngine<Self::AccountId>;

        /// 资金白名单检查器。
        type InstitutionAsset: institution_asset::InstitutionAsset<Self::AccountId>;

        /// 资金源保护检查器。
        type ProtectedSourceChecker: ProtectedSourceChecker<Self::AccountId>;

        /// 备注最大长度
        #[pallet::constant]
        type MaxRemarkLen: Get<u32>;

        /// 手续费分账路由（复用 OnchainFeeRouter）
        type FeeRouter: frame_support::traits::OnUnbalanced<
            <<Self as Config>::Currency as Currency<Self::AccountId>>::NegativeImbalance,
        >;

        /// 个人多签账户状态与管理员配置查询,由 personal-manage 聚合 personal-admins 提供。
        type PersonalQuery: personal_manage::traits::PersonalMultisigQuery<Self::AccountId>;

        /// 注册机构账户状态与管理员配置查询,由 runtime 聚合 public/private 生命周期模块提供。
        type InstitutionQuery: entity_primitives::InstitutionMultisigQuery<Self::AccountId>;

        /// Weight 配置
        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // 活跃提案数限制已移至 votingengine::active_proposal_limit 全局管控。
    // 提案业务数据和元数据已统一存储到 votingengine（ProposalData / ProposalMeta）。

    /// 安全基金转账提案动作存储。
    #[pallet::storage]
    pub type SafetyFundProposalActions<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        SafetyFundAction<T::AccountId, BalanceOf<T>, T::MaxRemarkLen>,
        OptionQuery,
    >;

    /// 手续费划转提案动作存储（省储行 + 国储会共用）。
    #[pallet::storage]
    pub type SweepProposalActions<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, SweepAction<T::AccountId, BalanceOf<T>>, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 转账提案已创建。citizenapp 可扫描此事件展示投票详情。
        TransferProposed {
            proposal_id: u64,
            institution_code: InstitutionCode,
            institution: T::AccountId,
            proposer: T::AccountId,
            /// 资金源(= 机构主账户)。
            from: T::AccountId,
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
            /// 原文 remark,供管理员投票前核对。
            remark: BoundedVec<u8, T::MaxRemarkLen>,
            /// 投票引擎分配的超时区块,供 citizenapp 倒计时
            expires_at: BlockNumberFor<T>,
        },
        /// 投票通过但执行失败（投票已记录，提案数据保留，可通过 VotingEngine 统一入口手动重试）
        TransferExecutionFailed {
            proposal_id: u64,
            institution: T::AccountId,
        },
        /// 转账已执行（投票通过后自动触发，含手续费分账）
        TransferExecuted {
            proposal_id: u64,
            institution: T::AccountId,
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
            fee: BalanceOf<T>,
        },
        /// 安全基金转账提案已创建。
        SafetyFundTransferProposed {
            proposal_id: u64,
            proposer: T::AccountId,
            /// 资金源(= SAFETY_FUND_ACCOUNT 常量)
            from: T::AccountId,
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
            /// 原文 remark,供管理员投票前核对。
            remark: BoundedVec<u8, T::MaxRemarkLen>,
            expires_at: BlockNumberFor<T>,
        },
        /// 安全基金转账已执行
        SafetyFundTransferExecuted {
            proposal_id: u64,
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
            fee: BalanceOf<T>,
        },
        /// 安全基金投票通过但执行失败
        SafetyFundExecutionFailed { proposal_id: u64 },
        /// 手续费划转提案已创建。
        SweepToMainProposed {
            proposal_id: u64,
            institution: T::AccountId,
            proposer: T::AccountId,
            /// 资金源(= 机构费用账户)
            from: T::AccountId,
            /// 资金目标(= 机构主账户)
            to: T::AccountId,
            amount: BalanceOf<T>,
            expires_at: BlockNumberFor<T>,
        },
        /// 手续费划转已执行
        SweepToMainExecuted {
            proposal_id: u64,
            institution: T::AccountId,
            amount: BalanceOf<T>,
            fee: BalanceOf<T>,
            reserve_left: BalanceOf<T>,
        },
        /// 手续费划转投票通过但执行失败
        SweepExecutionFailed { proposal_id: u64 },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 中文注释：资金账户不属于 NRC/PRC/PRB、个人多签或注册机构账户。
        InvalidInstitution,
        /// 中文注释：调用者声明的机构码与资金账户实际分类不一致。
        InstitutionCodeMismatch,
        /// 中文注释：调用者不是该多签资金账户的管理员。
        UnauthorizedAdmin,
        /// 中文注释：资金账户保护检查未通过（如冻结期间禁止支出）。
        InstitutionSpendNotAllowed,
        /// 中文注释：转账金额不能为零。
        ZeroAmount,
        /// 中文注释：转账金额低于 ED（存在性保证金），收款地址可能无法创建。
        AmountBelowExistentialDeposit,
        /// 中文注释：不允许转账给转出资金账户自身。
        SelfTransferNotAllowed,
        /// 中文注释：收款地址是受保护地址（如质押地址），不允许作为收款方。
        BeneficiaryIsProtectedAddress,
        /// 中文注释：提案动作数据未找到或解码失败。
        ProposalActionNotFound,
        /// 中文注释：机构账户地址解码失败。
        InstitutionAccountDecodeFailed,
        /// 中文注释：资金账户余额不足（需 amount + fee + ED）。
        InsufficientBalance,
        /// 中文注释：提案未达到通过状态，不可执行。
        ProposalNotPassed,
        /// 中文注释：链上转账操作失败。
        TransferFailed,
        /// 中文注释：安全基金提案未找到。
        SafetyFundProposalNotFound,
        /// 中文注释：安全基金余额不足。
        SafetyFundInsufficientBalance,
        /// 中文注释：安全基金提案未通过。
        SafetyFundProposalNotPassed,
        /// 中文注释：手续费划转提案未找到。
        SweepProposalNotFound,
        /// 中文注释：手续费划转金额无效。
        InvalidSweepAmount,
        /// 中文注释：手续费账户余额不足（需保留最低余额）。
        InsufficientFeeReserve,
        /// 中文注释：手续费划转金额超过上限（可用余额的 80%）。
        SweepAmountExceedsCap,
        /// 中文注释：手续费划转提案未通过。
        SweepProposalNotPassed,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 发起多签资金账户转账提案。
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::propose_transfer())]
        pub fn propose_transfer(
            origin: OriginFor<T>,
            institution_code: InstitutionCode,
            institution: T::AccountId,
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
            remark: BoundedVec<u8, T::MaxRemarkLen>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(amount > Zero::zero(), Error::<T>::ZeroAmount);
            let (actual_org, institution_account) =
                Self::resolve_institution_account(institution.clone())?;
            ensure!(
                actual_org == institution_code,
                Error::<T>::InstitutionCodeMismatch
            );
            ensure!(
                Self::is_internal_admin(institution_code, institution.clone(), &who),
                Error::<T>::UnauthorizedAdmin
            );
            ensure!(
                <T as Config>::InstitutionAsset::can_spend(
                    &institution_account,
                    InstitutionAssetAction::MultisigTransferExecute,
                ),
                Error::<T>::InstitutionSpendNotAllowed
            );

            // 转账金额必须 >= ED，防止收款地址不存在时创建失败
            let ed = <T as Config>::Currency::minimum_balance();
            ensure!(amount >= ed, Error::<T>::AmountBelowExistentialDeposit);

            // 不允许自转账
            ensure!(
                beneficiary != institution_account,
                Error::<T>::SelfTransferNotAllowed
            );

            // 不允许转到受保护地址（质押地址）
            ensure!(
                !<T as Config>::ProtectedSourceChecker::is_protected(&beneficiary,),
                Error::<T>::BeneficiaryIsProtectedAddress
            );

            // 活跃提案数由 votingengine 在 create_internal_proposal 中统一检查

            // 预检余额（含手续费，与执行时检查一致，避免创建必定无法执行的提案）
            let amount_u128: u128 = amount.saturated_into();
            let fee_u128 = onchain_transaction::calculate_onchain_fee(amount_u128);
            let fee: BalanceOf<T> = fee_u128.saturated_into();
            let total = amount
                .checked_add(&fee)
                .ok_or(Error::<T>::InsufficientBalance)?;
            let free = <T as Config>::Currency::free_balance(&institution_account);
            let required = total
                .checked_add(&ed)
                .ok_or(Error::<T>::InsufficientBalance)?;
            ensure!(free >= required, Error::<T>::InsufficientBalance);

            let action = TransferAction {
                institution: institution.clone(),
                beneficiary: beneficiary.clone(),
                amount,
                remark: remark.clone(),
                proposer: who.clone(),
            };
            let mut encoded = sp_runtime::Vec::from(crate::MODULE_TAG);
            encoded.extend_from_slice(&action.encode());
            // 中文注释：创建提案时同步写入 owner/data/meta，禁止后续跨模块覆写业务数据。
            let proposal_id =
                <T as Config>::InternalVoteEngine::create_general_internal_proposal_with_data(
                    who.clone(),
                    institution_code,
                    institution.clone(),
                    crate::MODULE_TAG,
                    encoded,
                )?;

            // 从投票引擎回读 proposal.end 作为 expires_at,供 citizenapp 倒计时。
            let expires_at = votingengine::Pallet::<T>::proposals(proposal_id)
                .map(|p| p.end)
                .ok_or(Error::<T>::ProposalActionNotFound)?;

            Self::deposit_event(Event::<T>::TransferProposed {
                proposal_id,
                institution_code,
                institution,
                proposer: who,
                from: institution_account,
                beneficiary,
                amount,
                remark,
                expires_at,
            });
            Ok(())
        }

        /// 发起国储会安全基金转账提案（内部投票）。
        ///
        /// 从安全基金账户（`SAFETY_FUND_ACCOUNT`）向指定收款地址转账。
        /// 仅国储会管理员可发起。
        #[pallet::call_index(1)]
        #[pallet::weight(T::DbWeight::get().reads_writes(4, 2))]
        pub fn propose_safety_fund_transfer(
            origin: OriginFor<T>,
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
            remark: BoundedVec<u8, T::MaxRemarkLen>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(amount > Zero::zero(), Error::<T>::ZeroAmount);

            // 验证国储会管理员
            let nrc_institution = Self::decode_institution_account(&CHINA_CB[0].main_account)?;
            ensure!(
                <T as votingengine::Config>::InternalAdminProvider::is_internal_admin(
                    NRC,
                    nrc_institution.clone(),
                    &who,
                ),
                Error::<T>::UnauthorizedAdmin
            );

            // 验证安全基金账户余额
            let safety_fund_account = T::AccountId::decode(&mut &SAFETY_FUND_ACCOUNT[..])
                .map_err(|_| Error::<T>::InstitutionAccountDecodeFailed)?;
            ensure!(
                <T as Config>::InstitutionAsset::can_spend(
                    &safety_fund_account,
                    InstitutionAssetAction::NrcSafetyFundTransfer,
                ),
                Error::<T>::InstitutionSpendNotAllowed
            );

            // 预检余额（含手续费，避免创建必定无法执行的提案）
            let amount_u128: u128 = amount.saturated_into();
            let fee_u128 = onchain_transaction::calculate_onchain_fee(amount_u128);
            let fee: BalanceOf<T> = fee_u128.saturated_into();
            let total = amount
                .checked_add(&fee)
                .ok_or(Error::<T>::SafetyFundInsufficientBalance)?;
            let ed: BalanceOf<T> = <T as Config>::Currency::minimum_balance();
            let free = <T as Config>::Currency::free_balance(&safety_fund_account);
            let required = total
                .checked_add(&ed)
                .ok_or(Error::<T>::SafetyFundInsufficientBalance)?;
            ensure!(free >= required, Error::<T>::SafetyFundInsufficientBalance);

            let proposal_id =
                <T as Config>::InternalVoteEngine::create_general_internal_proposal_with_data(
                    who.clone(),
                    NRC,
                    nrc_institution,
                    crate::MODULE_TAG,
                    sp_runtime::Vec::from(SAFETY_FUND_OWNER_DATA),
                )?;

            SafetyFundProposalActions::<T>::insert(
                proposal_id,
                SafetyFundAction {
                    beneficiary: beneficiary.clone(),
                    amount,
                    remark: remark.clone(),
                    proposer: who.clone(),
                },
            );

            // 从投票引擎回读 proposal.end 作为 expires_at。
            let expires_at = votingengine::Pallet::<T>::proposals(proposal_id)
                .map(|p| p.end)
                .ok_or(Error::<T>::SafetyFundProposalNotFound)?;

            Self::deposit_event(Event::SafetyFundTransferProposed {
                proposal_id,
                proposer: who,
                from: safety_fund_account,
                beneficiary,
                amount,
                remark,
                expires_at,
            });
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(T::DbWeight::get().reads_writes(4, 2))]
        pub fn propose_sweep_to_main(
            origin: OriginFor<T>,
            institution: T::AccountId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let amount_u128: u128 = amount.saturated_into();
            ensure!(amount_u128 > 0, Error::<T>::InvalidSweepAmount);

            // 动态判断治理机构码类型。
            let institution_code = Self::resolve_sweep_org(&institution)?;
            ensure!(
                <T as votingengine::Config>::InternalAdminProvider::is_internal_admin(
                    institution_code,
                    institution.clone(),
                    &who,
                ),
                Error::<T>::UnauthorizedAdmin
            );

            let proposal_id =
                <T as Config>::InternalVoteEngine::create_general_internal_proposal_with_data(
                    who.clone(),
                    institution_code,
                    institution.clone(),
                    crate::MODULE_TAG,
                    sp_runtime::Vec::from(SWEEP_OWNER_DATA),
                )?;

            SweepProposalActions::<T>::insert(
                proposal_id,
                SweepAction {
                    institution: institution.clone(),
                    amount,
                    proposer: who.clone(),
                },
            );

            let fee_account = Self::resolve_fee_account(&institution)?;
            let main_account = Self::resolve_main_account(institution.clone())?;
            let expires_at = votingengine::Pallet::<T>::proposals(proposal_id)
                .map(|p| p.end)
                .ok_or(Error::<T>::SweepProposalNotFound)?;

            Self::deposit_event(Event::SweepToMainProposed {
                proposal_id,
                institution,
                proposer: who,
                from: fee_account,
                to: main_account,
                amount,
                expires_at,
            });
            Ok(())
        }

        // call_index 3/4/5 永久保留空位,不复用。
    }

    impl<T: Config> Pallet<T> {
        fn decode_institution_account(raw: &[u8; 32]) -> Result<T::AccountId, Error<T>> {
            decode_raw_account::<T>(raw).ok_or(Error::<T>::InstitutionAccountDecodeFailed)
        }

        fn resolve_institution_account(
            institution: T::AccountId,
        ) -> Result<(InstitutionCode, T::AccountId), Error<T>> {
            use entity_primitives::InstitutionMultisigQuery;
            use personal_manage::traits::PersonalMultisigQuery;

            if let Some(actual_org) = builtin_org::<T>(&institution) {
                return Ok((actual_org, institution));
            }

            if <T as Config>::PersonalQuery::is_active(&institution) {
                return Ok((PMUL, institution));
            }

            ensure!(
                <T as Config>::InstitutionQuery::is_active(&institution),
                Error::<T>::InvalidInstitution
            );
            let institution_code = <T as Config>::InstitutionQuery::lookup_org(&institution)
                .ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                is_institution_code(&institution_code),
                Error::<T>::InvalidInstitution
            );
            Ok((institution_code, institution))
        }

        fn is_internal_admin(
            institution_code: InstitutionCode,
            institution: T::AccountId,
            who: &T::AccountId,
        ) -> bool {
            <T as votingengine::Config>::InternalAdminProvider::is_internal_admin(
                institution_code,
                institution,
                who,
            )
        }

        /// 判断治理机构码类型用于 sweep 提案。
        ///
        /// 中文注释：sweep 只服务治理机构费用账户，不接入个人多签或注册机构账户。
        fn resolve_sweep_org(institution: &T::AccountId) -> Result<InstitutionCode, Error<T>> {
            if CHINA_CB
                .first()
                .map(|n| raw_account_matches::<T>(&n.main_account, institution))
                .unwrap_or(false)
            {
                return Ok(NRC);
            }
            if CHINA_CH
                .iter()
                .any(|n| raw_account_matches::<T>(&n.main_account, institution))
            {
                return Ok(PRB);
            }
            Err(Error::<T>::InvalidInstitution)
        }

        /// 解析治理机构手续费账户。
        fn resolve_fee_account(institution: &T::AccountId) -> Result<T::AccountId, DispatchError> {
            if CHINA_CB
                .first()
                .map(|n| raw_account_matches::<T>(&n.main_account, institution))
                .unwrap_or(false)
            {
                return T::AccountId::decode(&mut &CHINA_CB[0].fee_account[..])
                    .map_err(|_| Error::<T>::InstitutionAccountDecodeFailed.into());
            }
            let node = CHINA_CH
                .iter()
                .find(|n| raw_account_matches::<T>(&n.main_account, institution))
                .ok_or(Error::<T>::InvalidInstitution)?;
            T::AccountId::decode(&mut &node.fee_account[..])
                .map_err(|_| Error::<T>::InstitutionAccountDecodeFailed.into())
        }

        /// 解析治理机构主账户。
        fn resolve_main_account(institution: T::AccountId) -> Result<T::AccountId, DispatchError> {
            Ok(institution)
        }

        pub(crate) fn try_execute_sweep_from_callback(
            proposal_id: u64,
            _callback_context: bool,
        ) -> DispatchResult {
            let action = SweepProposalActions::<T>::get(proposal_id)
                .ok_or(Error::<T>::SweepProposalNotFound)?;

            let proposal = votingengine::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::SweepProposalNotFound)?;
            ensure!(
                proposal.status == STATUS_PASSED,
                Error::<T>::SweepProposalNotPassed
            );

            let fee_account = Self::resolve_fee_account(&action.institution)?;
            let main_account = Self::resolve_main_account(action.institution.clone())?;

            ensure!(
                <T as Config>::InstitutionAsset::can_spend(
                    &fee_account,
                    InstitutionAssetAction::OffchainFeeSweepExecute,
                ),
                Error::<T>::InstitutionSpendNotAllowed
            );

            // ── 计算手续费 ──
            let amount_u128: u128 = action.amount.saturated_into();
            let tx_fee_u128 = onchain_transaction::calculate_onchain_fee(amount_u128);
            let tx_fee: BalanceOf<T> = tx_fee_u128.saturated_into();

            let fee_balance_u128: u128 =
                <T as Config>::Currency::free_balance(&fee_account).saturated_into();
            let reserve_u128 = FEE_ADDRESS_MIN_RESERVE_FEN;

            // ── 余额检查：amount + tx_fee + reserve ──
            let total_deduct_u128 = amount_u128.saturating_add(tx_fee_u128);
            ensure!(
                fee_balance_u128 >= total_deduct_u128
                    && fee_balance_u128.saturating_sub(total_deduct_u128) >= reserve_u128,
                Error::<T>::InsufficientFeeReserve
            );
            // ── cap 检查：划转金额不超过可用余额的 80%（可用 = 余额 - reserve） ──
            let available_u128 = fee_balance_u128.saturating_sub(reserve_u128);
            let cap_u128 = available_u128
                .saturating_mul(FEE_SWEEP_MAX_PERCENT)
                .saturating_div(100);
            ensure!(amount_u128 <= cap_u128, Error::<T>::SweepAmountExceedsCap);

            // ── 执行划转 ──
            <T as Config>::Currency::transfer(
                &fee_account,
                &main_account,
                action.amount,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )?;

            // ── 手续费：从费用账户扣取，通过 FeeRouter 按 80/10/10 分账 ──
            let fee_imbalance = <T as Config>::Currency::withdraw(
                &fee_account,
                tx_fee,
                frame_support::traits::WithdrawReasons::FEE,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )
            .map_err(|_| Error::<T>::InsufficientFeeReserve)?;
            <T as pallet::Config>::FeeRouter::on_unbalanced(fee_imbalance);

            let reserve_left = <T as Config>::Currency::free_balance(&fee_account);

            Self::deposit_event(Event::SweepToMainExecuted {
                proposal_id,
                institution: action.institution,
                amount: action.amount,
                fee: tx_fee,
                reserve_left,
            });
            Ok(())
        }

        pub(crate) fn try_execute_safety_fund_from_callback(
            proposal_id: u64,
            _callback_context: bool,
        ) -> DispatchResult {
            let action = SafetyFundProposalActions::<T>::get(proposal_id)
                .ok_or(Error::<T>::SafetyFundProposalNotFound)?;

            let proposal = votingengine::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::SafetyFundProposalNotFound)?;
            ensure!(
                proposal.status == STATUS_PASSED,
                Error::<T>::SafetyFundProposalNotPassed
            );

            let safety_fund_account = T::AccountId::decode(&mut &SAFETY_FUND_ACCOUNT[..])
                .map_err(|_| Error::<T>::InstitutionAccountDecodeFailed)?;

            ensure!(
                <T as Config>::InstitutionAsset::can_spend(
                    &safety_fund_account,
                    InstitutionAssetAction::NrcSafetyFundTransfer,
                ),
                Error::<T>::InstitutionSpendNotAllowed
            );

            // ── 计算手续费 ──
            let amount_u128: u128 = action.amount.saturated_into();
            let fee_u128 = onchain_transaction::calculate_onchain_fee(amount_u128);
            let fee: BalanceOf<T> = fee_u128.saturated_into();
            let total = action
                .amount
                .checked_add(&fee)
                .ok_or(Error::<T>::SafetyFundInsufficientBalance)?;

            // ── 余额检查：amount + fee + ED ──
            let free = <T as Config>::Currency::free_balance(&safety_fund_account);
            let ed = <T as Config>::Currency::minimum_balance();
            let required = total
                .checked_add(&ed)
                .ok_or(Error::<T>::SafetyFundInsufficientBalance)?;
            ensure!(free >= required, Error::<T>::SafetyFundInsufficientBalance);

            // ── 执行转账 ──
            <T as Config>::Currency::transfer(
                &safety_fund_account,
                &action.beneficiary,
                action.amount,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )
            .map_err(|_| Error::<T>::SafetyFundInsufficientBalance)?;

            // ── 手续费：从安全基金扣取，通过 FeeRouter 按 80/10/10 分账 ──
            let fee_imbalance = <T as Config>::Currency::withdraw(
                &safety_fund_account,
                fee,
                frame_support::traits::WithdrawReasons::FEE,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )
            .map_err(|_| Error::<T>::SafetyFundInsufficientBalance)?;
            <T as pallet::Config>::FeeRouter::on_unbalanced(fee_imbalance);

            Self::deposit_event(Event::SafetyFundTransferExecuted {
                proposal_id,
                beneficiary: action.beneficiary,
                amount: action.amount,
                fee,
            });

            Ok(())
        }

        pub(crate) fn try_execute_transfer_from_callback(
            proposal_id: u64,
            _callback_context: bool,
        ) -> DispatchResult {
            let proposal = votingengine::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            ensure!(
                proposal.status == STATUS_PASSED,
                Error::<T>::ProposalNotPassed
            );

            let raw = votingengine::Pallet::<T>::get_proposal_data(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            let tag = crate::MODULE_TAG;
            ensure!(
                raw.len() >= tag.len() && &raw[..tag.len()] == tag,
                Error::<T>::ProposalActionNotFound
            );
            let action = TransferAction::<T::AccountId, BalanceOf<T>, T::MaxRemarkLen>::decode(
                &mut &raw[tag.len()..],
            )
            .map_err(|_| Error::<T>::ProposalActionNotFound)?;
            let (_, institution_account) =
                Self::resolve_institution_account(action.institution.clone())?;
            ensure!(
                <T as Config>::InstitutionAsset::can_spend(
                    &institution_account,
                    InstitutionAssetAction::MultisigTransferExecute,
                ),
                Error::<T>::InstitutionSpendNotAllowed
            );

            // ── 计算手续费（复用 onchain-transaction 公共接口） ──
            let amount_u128: u128 = action.amount.saturated_into();
            let fee_u128 = onchain_transaction::calculate_onchain_fee(amount_u128);
            let fee: BalanceOf<T> = fee_u128.saturated_into();
            let total = action
                .amount
                .checked_add(&fee)
                .ok_or(Error::<T>::InsufficientBalance)?;

            // ── 余额检查：需要 total + ED ──
            let free = <T as Config>::Currency::free_balance(&institution_account);
            let ed = <T as Config>::Currency::minimum_balance();
            let required = total
                .checked_add(&ed)
                .ok_or(Error::<T>::InsufficientBalance)?;
            ensure!(free >= required, Error::<T>::InsufficientBalance);

            // ── 原子执行：手续费扣取 + 转账，任一失败整体回滚 ──
            let exec_result = frame_support::storage::with_transaction(|| {
                // 先扣手续费
                let fee_imbalance = match <T as Config>::Currency::withdraw(
                    &institution_account,
                    fee,
                    frame_support::traits::WithdrawReasons::FEE,
                    ExistenceRequirement::KeepAlive,
                ) {
                    Ok(imbalance) => imbalance,
                    Err(_) => {
                        return frame_support::storage::TransactionOutcome::Rollback(Err(
                            Error::<T>::InsufficientBalance.into(),
                        ))
                    }
                };
                <T as pallet::Config>::FeeRouter::on_unbalanced(fee_imbalance);

                // 再转账
                match <T as Config>::Currency::transfer(
                    &institution_account,
                    &action.beneficiary,
                    action.amount,
                    ExistenceRequirement::KeepAlive,
                ) {
                    Ok(()) => frame_support::storage::TransactionOutcome::Commit(Ok(())),
                    Err(e) => frame_support::storage::TransactionOutcome::Rollback(Err(e)),
                }
            });
            exec_result?;

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

// ──── 投票终态回调:把已通过的 3 组业务提案(转账/安全基金/手续费划转)落地到链上 ────
//
// 统一状态机整改后业务模块不再持有独立 vote/finalize call,提案通过(或否决)
// 由投票引擎通过 [`votingengine::InternalVoteResultCallback`] 广播回来。
// 本 Executor 按 `MODULE_TAG` 前缀 + 独立存储键认领对应业务:
// - `MODULE_TAG` 前缀 `multisig-transfer` → transfer
// - `SafetyFundProposalActions[id]` 存在 → safety_fund
// - `SweepProposalActions[id]` 存在 → sweep
//
// 失败语义:执行失败发 ExecutionFailed 事件,提案保留 PASSED 状态,快照管理员
// 可通过 VotingEngine::retry_passed_proposal 手动重试,实际权限由 votingengine 统一校验。
pub struct InternalVoteExecutor<T>(core::marker::PhantomData<T>);

impl<T: pallet::Config> InternalVoteResultCallback for InternalVoteExecutor<T> {
    fn on_internal_vote_finalized(
        proposal_id: u64,
        approved: bool,
    ) -> Result<ProposalExecutionOutcome, sp_runtime::DispatchError> {
        let is_safety_fund = SafetyFundProposalActions::<T>::contains_key(proposal_id);
        let is_sweep = SweepProposalActions::<T>::contains_key(proposal_id);
        let is_transfer = !is_safety_fund
            && !is_sweep
            && votingengine::Pallet::<T>::get_proposal_data(proposal_id)
                .map(|raw| raw.starts_with(crate::MODULE_TAG))
                .unwrap_or(false);

        if !is_transfer && !is_safety_fund && !is_sweep {
            return Ok(ProposalExecutionOutcome::Ignored); // 非本模块提案
        }

        if approved {
            let exec_result = if is_transfer {
                pallet::Pallet::<T>::try_execute_transfer_from_callback(proposal_id, true)
            } else if is_safety_fund {
                pallet::Pallet::<T>::try_execute_safety_fund_from_callback(proposal_id, true)
            } else {
                pallet::Pallet::<T>::try_execute_sweep_from_callback(proposal_id, true)
            };
            if let Err(_e) = exec_result {
                // 执行失败:发事件,提案保留 PASSED,供 VotingEngine 统一重试入口处理。
                if is_transfer {
                    if let Some(raw) = votingengine::Pallet::<T>::get_proposal_data(proposal_id) {
                        if raw.len() >= crate::MODULE_TAG.len()
                            && raw.starts_with(crate::MODULE_TAG)
                        {
                            if let Ok(action) = TransferAction::<
                                T::AccountId,
                                BalanceOf<T>,
                                T::MaxRemarkLen,
                            >::decode(
                                &mut &raw[crate::MODULE_TAG.len()..]
                            ) {
                                pallet::Pallet::<T>::deposit_event(
                                    pallet::Event::<T>::TransferExecutionFailed {
                                        proposal_id,
                                        institution: action.institution,
                                    },
                                );
                            }
                        }
                    }
                } else if is_safety_fund {
                    pallet::Pallet::<T>::deposit_event(
                        pallet::Event::<T>::SafetyFundExecutionFailed { proposal_id },
                    );
                } else if is_sweep {
                    pallet::Pallet::<T>::deposit_event(pallet::Event::<T>::SweepExecutionFailed {
                        proposal_id,
                    });
                }
                return Ok(ProposalExecutionOutcome::RetryableFailed);
            }
            return Ok(ProposalExecutionOutcome::Executed);
        } else {
            // 否决:清理独立存储,避免僵尸数据。
            SafetyFundProposalActions::<T>::remove(proposal_id);
            SweepProposalActions::<T>::remove(proposal_id);
        }
        Ok(ProposalExecutionOutcome::Executed)
    }

    fn on_execution_failed_terminal(proposal_id: u64) -> DispatchResult {
        // 中文注释：普通转账仅依赖 ProposalData；安全基金和 sweep 还有独立动作存储，需要终态清理。
        SafetyFundProposalActions::<T>::remove(proposal_id);
        SweepProposalActions::<T>::remove(proposal_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
