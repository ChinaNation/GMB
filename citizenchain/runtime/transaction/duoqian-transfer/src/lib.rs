//! # 机构多签名地址转账模块 (duoqian-transfer)
//!
//! 本模块为治理机构（NRC/PRC/PRB）和注册多签机构提供链上转账治理流程：
//! - 管理员发起转账提案，经内部投票通过后自动执行转账并扣取手续费。
//! - 自动执行失败时保留提案状态，可通过 `execute_transfer` 手动重试。
//! - 余额在提案创建和执行两个时点双重检查，含手续费和 ED 保留。
//! - 收款地址不能是机构自身，也不能是受保护地址（质押地址）。

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{ensure, pallet_prelude::*, traits::Currency, BoundedVec};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::traits::{CheckedAdd, SaturatedConversion, Zero};

// 统一状态机整改:原业务自有投票/finalize 路径所依赖的
// `sr25519::{Public, Signature}` / `Vec` / `BTreeSet` 已全部随投票统一入口
// 改造下线,不再从此处导入。
extern crate alloc;

use primitives::china::china_cb::{
    shenfen_id_to_fixed48 as reserve_pallet_id_to_bytes, CHINA_CB, NRC_ANQUAN_ADDRESS,
};
use primitives::china::china_ch::{
    shenfen_id_to_fixed48 as shengbank_pallet_id_to_bytes, CHINA_CH,
};
use voting_engine::{
    internal_vote::{ORG_DUOQIAN, ORG_NRC, ORG_PRB, ORG_PRC},
    InstitutionPalletId, InternalVoteResultCallback, ProposalExecutionOutcome, STATUS_PASSED,
};

pub use pallet::*;
/// 模块标识前缀，用于在 ProposalData 中区分不同业务模块，防止跨模块误解码。
pub const MODULE_TAG: &[u8] = b"dq-xfer";
const SAFETY_FUND_OWNER_DATA: &[u8] = b"dq-xfer:safety";
const SWEEP_OWNER_DATA: &[u8] = b"dq-xfer:sweep";

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod weights;

type BalanceOf<T> = <<T as duoqian_manage::Config>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::Balance;

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
    /// 机构标识
    pub institution: InstitutionPalletId,
    /// 划转金额
    pub amount: Balance,
    /// 发起管理员(Tx 1 中锁定)
    pub proposer: AccountId,
}

/// 手续费账户最低保留余额：1111.11 元（111111 分）。
const FEE_ADDRESS_MIN_RESERVE_FEN: u128 = 111_111;

/// 单次划转上限：可用余额的 80%。
const FEE_SWEEP_MAX_PERCENT: u128 = 80;

/// 中文注释：判断机构属于 NRC/PRC/PRB（不含注册多签，注册多签由链上存储判断）。
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

/// 中文注释：从 CHINA_CB/CHINA_CH 中查找机构的多签账户地址（main_address）。
fn institution_pallet_address(institution: InstitutionPalletId) -> Option<[u8; 32]> {
    if let Some(node) = CHINA_CB
        .iter()
        .find(|n| reserve_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
    {
        return Some(node.main_address);
    }

    CHINA_CH
        .iter()
        .find(|n| shengbank_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
        .map(|n| n.main_address)
}

/// 中文注释：检查机构 ID 后 16 字节是否全零（注册多签机构的 ID 格式要求）。
fn institution_id_has_zero_suffix(institution: InstitutionPalletId) -> bool {
    institution[32..].iter().all(|b| *b == 0)
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;
    use duoqian_manage::ProtectedSourceChecker;
    use frame_support::traits::ExistenceRequirement;
    use frame_support::traits::OnUnbalanced;
    use institution_asset::{InstitutionAsset, InstitutionAssetAction};
    use voting_engine::InternalAdminProvider;
    use voting_engine::InternalVoteEngine;

    #[pallet::config]
    pub trait Config:
        frame_system::Config + voting_engine::Config + duoqian_manage::Config
    {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 备注最大长度
        #[pallet::constant]
        type MaxRemarkLen: Get<u32>;

        /// 手续费分账路由（复用 OnchainFeeRouter）
        type FeeRouter: frame_support::traits::OnUnbalanced<
            <<Self as duoqian_manage::Config>::Currency as Currency<
                Self::AccountId,
            >>::NegativeImbalance,
        >;

        /// Weight 配置
        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // 活跃提案数限制已移至 voting-engine::active_proposal_limit 全局管控。
    // 提案业务数据和元数据已统一存储到 voting-engine（ProposalData / ProposalMeta）。

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
        /// 转账提案已创建。wuminapp 可扫描此事件展示投票详情。
        TransferProposed {
            proposal_id: u64,
            org: u8,
            institution: InstitutionPalletId,
            proposer: T::AccountId,
            /// 资金源(= 机构主账户)。
            from: T::AccountId,
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
            /// 原文 remark,供管理员投票前核对。
            remark: BoundedVec<u8, T::MaxRemarkLen>,
            /// 投票引擎分配的超时区块,供 wuminapp 倒计时
            expires_at: BlockNumberFor<T>,
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
        /// 安全基金转账提案已创建。
        SafetyFundTransferProposed {
            proposal_id: u64,
            proposer: T::AccountId,
            /// 资金源(= NRC_ANQUAN_ADDRESS 常量)
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
            institution: InstitutionPalletId,
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
            institution: InstitutionPalletId,
            amount: BalanceOf<T>,
            fee: BalanceOf<T>,
            reserve_left: BalanceOf<T>,
        },
        /// 手续费划转投票通过但执行失败
        SweepExecutionFailed { proposal_id: u64 },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 中文注释：机构不属于 NRC/PRC/PRB 且非注册多签机构。
        InvalidInstitution,
        /// 中文注释：调用者声明的 org 类型与机构实际类型不一致。
        InstitutionOrgMismatch,
        /// 中文注释：调用者不是该机构的管理员。
        UnauthorizedAdmin,
        /// 中文注释：机构资产保护检查未通过（如冻结期间禁止支出）。
        InstitutionSpendNotAllowed,
        /// 中文注释：转账金额不能为零。
        ZeroAmount,
        /// 中文注释：转账金额低于 ED（存在性保证金），收款地址可能无法创建。
        AmountBelowExistentialDeposit,
        /// 中文注释：不允许转账给机构自身。
        SelfTransferNotAllowed,
        /// 中文注释：收款地址是受保护地址（如质押地址），不允许作为收款方。
        BeneficiaryIsProtectedAddress,
        /// 中文注释：提案动作数据未找到或解码失败。
        ProposalActionNotFound,
        /// 中文注释：机构账户地址解码失败。
        InstitutionAccountDecodeFailed,
        /// 中文注释：机构余额不足（需 amount + fee + ED）。
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
            let (actual_org, institution_account) = Self::resolve_institution_account(institution)?;
            ensure!(actual_org == org, Error::<T>::InstitutionOrgMismatch);
            ensure!(
                Self::is_internal_admin(org, institution, &who),
                Error::<T>::UnauthorizedAdmin
            );
            ensure!(
                <T as duoqian_manage::Config>::InstitutionAsset::can_spend(
                    &institution_account,
                    InstitutionAssetAction::DuoqianTransferExecute,
                ),
                Error::<T>::InstitutionSpendNotAllowed
            );

            // 转账金额必须 >= ED，防止收款地址不存在时创建失败
            let ed = <T as duoqian_manage::Config>::Currency::minimum_balance();
            ensure!(amount >= ed, Error::<T>::AmountBelowExistentialDeposit);

            // 不允许自转账
            ensure!(
                beneficiary != institution_account,
                Error::<T>::SelfTransferNotAllowed
            );

            // 不允许转到受保护地址（质押地址）
            ensure!(
                !<T as duoqian_manage::Config>::ProtectedSourceChecker::is_protected(&beneficiary,),
                Error::<T>::BeneficiaryIsProtectedAddress
            );

            // 活跃提案数由 voting-engine 在 create_internal_proposal 中统一检查

            // 预检余额（含手续费，与执行时检查一致，避免创建必定无法执行的提案）
            let amount_u128: u128 = amount.saturated_into();
            let fee_u128 = onchain_transaction::calculate_onchain_fee(amount_u128);
            let fee: BalanceOf<T> = fee_u128.saturated_into();
            let total = amount
                .checked_add(&fee)
                .ok_or(Error::<T>::InsufficientBalance)?;
            let free = <T as duoqian_manage::Config>::Currency::free_balance(&institution_account);
            let required = total
                .checked_add(&ed)
                .ok_or(Error::<T>::InsufficientBalance)?;
            ensure!(free >= required, Error::<T>::InsufficientBalance);

            let action = TransferAction {
                institution,
                beneficiary: beneficiary.clone(),
                amount,
                remark: remark.clone(),
                proposer: who.clone(),
            };
            let mut encoded = sp_runtime::Vec::from(crate::MODULE_TAG);
            encoded.extend_from_slice(&action.encode());
            // 中文注释：创建提案时同步写入 owner/data/meta，禁止后续跨模块覆写业务数据。
            let proposal_id =
                <T as duoqian_manage::Config>::InternalVoteEngine::create_internal_proposal_with_data(
                    who.clone(),
                    org,
                    institution,
                    crate::MODULE_TAG,
                    encoded,
                )?;

            // 从投票引擎回读 proposal.end 作为 expires_at,供 wuminapp 倒计时。
            let expires_at = voting_engine::Pallet::<T>::proposals(proposal_id)
                .map(|p| p.end)
                .ok_or(Error::<T>::ProposalActionNotFound)?;

            Self::deposit_event(Event::<T>::TransferProposed {
                proposal_id,
                org,
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
        /// 从安全基金账户（`NRC_ANQUAN_ADDRESS`）向指定收款地址转账。
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
            let nrc_institution = reserve_pallet_id_to_bytes(CHINA_CB[0].shenfen_id)
                .ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                <T as voting_engine::Config>::InternalAdminProvider::is_internal_admin(
                    ORG_NRC,
                    nrc_institution,
                    &who,
                ),
                Error::<T>::UnauthorizedAdmin
            );

            // 验证安全基金账户余额
            let safety_fund_account = T::AccountId::decode(&mut &NRC_ANQUAN_ADDRESS[..])
                .map_err(|_| Error::<T>::InstitutionAccountDecodeFailed)?;
            ensure!(
                <T as duoqian_manage::Config>::InstitutionAsset::can_spend(
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
            let ed: BalanceOf<T> = <T as duoqian_manage::Config>::Currency::minimum_balance();
            let free = <T as duoqian_manage::Config>::Currency::free_balance(&safety_fund_account);
            let required = total
                .checked_add(&ed)
                .ok_or(Error::<T>::SafetyFundInsufficientBalance)?;
            ensure!(free >= required, Error::<T>::SafetyFundInsufficientBalance);

            let proposal_id =
                <T as duoqian_manage::Config>::InternalVoteEngine::create_internal_proposal_with_data(
                    who.clone(),
                    ORG_NRC,
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
            let expires_at = voting_engine::Pallet::<T>::proposals(proposal_id)
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
            institution: InstitutionPalletId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let amount_u128: u128 = amount.saturated_into();
            ensure!(amount_u128 > 0, Error::<T>::InvalidSweepAmount);

            // 动态判断 org 类型
            let org = Self::resolve_sweep_org(institution)?;
            ensure!(
                <T as voting_engine::Config>::InternalAdminProvider::is_internal_admin(
                    org,
                    institution,
                    &who,
                ),
                Error::<T>::UnauthorizedAdmin
            );

            let proposal_id =
                <T as duoqian_manage::Config>::InternalVoteEngine::create_internal_proposal_with_data(
                    who.clone(),
                    org,
                    institution,
                    crate::MODULE_TAG,
                    sp_runtime::Vec::from(SWEEP_OWNER_DATA),
                )?;

            SweepProposalActions::<T>::insert(
                proposal_id,
                SweepAction {
                    institution,
                    amount,
                    proposer: who.clone(),
                },
            );

            let fee_account = Self::resolve_fee_account(institution)?;
            let main_account = Self::resolve_main_account(institution)?;
            let expires_at = voting_engine::Pallet::<T>::proposals(proposal_id)
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

        // call_index = 3, 4, 5 已废弃: execute_transfer /
        // execute_safety_fund_transfer / execute_sweep_to_main 已统一到
        // VotingEngine::retry_passed_proposal —— 前端必须直接调用投票引擎入口。
    }

    impl<T: Config> Pallet<T> {
        fn registered_duoqian_account(
            institution: InstitutionPalletId,
        ) -> Result<T::AccountId, Error<T>> {
            ensure!(
                institution_id_has_zero_suffix(institution),
                Error::<T>::InvalidInstitution
            );
            let account = T::AccountId::decode(&mut &institution[..32])
                .map_err(|_| Error::<T>::InstitutionAccountDecodeFailed)?;
            let duoqian = duoqian_manage::DuoqianAccounts::<T>::get(&account)
                .ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                duoqian.status == duoqian_manage::DuoqianStatus::Active,
                Error::<T>::InvalidInstitution
            );
            Ok(account)
        }

        fn resolve_institution_account(
            institution: InstitutionPalletId,
        ) -> Result<(u8, T::AccountId), Error<T>> {
            if let Some(actual_org) = institution_org(institution) {
                let raw_account = institution_pallet_address(institution)
                    .ok_or(Error::<T>::InvalidInstitution)?;
                let institution_account = T::AccountId::decode(&mut &raw_account[..])
                    .map_err(|_| Error::<T>::InstitutionAccountDecodeFailed)?;
                return Ok((actual_org, institution_account));
            }

            let institution_account = Self::registered_duoqian_account(institution)?;
            Ok((ORG_DUOQIAN, institution_account))
        }

        fn is_internal_admin(
            org: u8,
            institution: InstitutionPalletId,
            who: &T::AccountId,
        ) -> bool {
            <T as voting_engine::Config>::InternalAdminProvider::is_internal_admin(
                org,
                institution,
                who,
            )
        }

        /// 判断机构的 org 类型用于 sweep 提案。
        fn resolve_sweep_org(institution: InstitutionPalletId) -> Result<u8, Error<T>> {
            // 国储会
            if CHINA_CB
                .first()
                .and_then(|n| reserve_pallet_id_to_bytes(n.shenfen_id))
                == Some(institution)
            {
                return Ok(ORG_NRC);
            }
            // 省储行
            if CHINA_CH
                .iter()
                .filter_map(|n| shengbank_pallet_id_to_bytes(n.shenfen_id))
                .any(|pid| pid == institution)
            {
                return Ok(ORG_PRB);
            }
            Err(Error::<T>::InvalidInstitution)
        }

        /// 解析机构手续费账户。
        fn resolve_fee_account(
            institution: InstitutionPalletId,
        ) -> Result<T::AccountId, DispatchError> {
            // 国储会：使用常量地址
            if CHINA_CB
                .first()
                .and_then(|n| reserve_pallet_id_to_bytes(n.shenfen_id))
                == Some(institution)
            {
                return T::AccountId::decode(&mut &CHINA_CB[0].fee_address[..])
                    .map_err(|_| Error::<T>::InstitutionAccountDecodeFailed.into());
            }
            // 省储行：使用 fee_address（BLAKE2-256 派生）
            let node = CHINA_CH
                .iter()
                .find(|n| shengbank_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                .ok_or(Error::<T>::InvalidInstitution)?;
            T::AccountId::decode(&mut &node.fee_address[..])
                .map_err(|_| Error::<T>::InstitutionAccountDecodeFailed.into())
        }

        /// 解析机构主账户。
        fn resolve_main_account(
            institution: InstitutionPalletId,
        ) -> Result<T::AccountId, DispatchError> {
            let raw =
                institution_pallet_address(institution).ok_or(Error::<T>::InvalidInstitution)?;
            T::AccountId::decode(&mut &raw[..])
                .map_err(|_| Error::<T>::InstitutionAccountDecodeFailed.into())
        }

        pub(crate) fn try_execute_sweep_from_callback(
            proposal_id: u64,
            _callback_context: bool,
        ) -> DispatchResult {
            let action = SweepProposalActions::<T>::get(proposal_id)
                .ok_or(Error::<T>::SweepProposalNotFound)?;

            let proposal = voting_engine::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::SweepProposalNotFound)?;
            ensure!(
                proposal.status == STATUS_PASSED,
                Error::<T>::SweepProposalNotPassed
            );

            let fee_account = Self::resolve_fee_account(action.institution)?;
            let main_account = Self::resolve_main_account(action.institution)?;

            ensure!(
                <T as duoqian_manage::Config>::InstitutionAsset::can_spend(
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
                <T as duoqian_manage::Config>::Currency::free_balance(&fee_account)
                    .saturated_into();
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
            <T as duoqian_manage::Config>::Currency::transfer(
                &fee_account,
                &main_account,
                action.amount,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )?;

            // ── 手续费：从费用账户扣取，通过 FeeRouter 按 80/10/10 分账 ──
            let fee_imbalance = <T as duoqian_manage::Config>::Currency::withdraw(
                &fee_account,
                tx_fee,
                frame_support::traits::WithdrawReasons::FEE,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )
            .map_err(|_| Error::<T>::InsufficientFeeReserve)?;
            <T as pallet::Config>::FeeRouter::on_unbalanced(fee_imbalance);

            let reserve_left = <T as duoqian_manage::Config>::Currency::free_balance(&fee_account);

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

            let proposal = voting_engine::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::SafetyFundProposalNotFound)?;
            ensure!(
                proposal.status == STATUS_PASSED,
                Error::<T>::SafetyFundProposalNotPassed
            );

            let safety_fund_account = T::AccountId::decode(&mut &NRC_ANQUAN_ADDRESS[..])
                .map_err(|_| Error::<T>::InstitutionAccountDecodeFailed)?;

            ensure!(
                <T as duoqian_manage::Config>::InstitutionAsset::can_spend(
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
            let free = <T as duoqian_manage::Config>::Currency::free_balance(&safety_fund_account);
            let ed = <T as duoqian_manage::Config>::Currency::minimum_balance();
            let required = total
                .checked_add(&ed)
                .ok_or(Error::<T>::SafetyFundInsufficientBalance)?;
            ensure!(free >= required, Error::<T>::SafetyFundInsufficientBalance);

            // ── 执行转账 ──
            <T as duoqian_manage::Config>::Currency::transfer(
                &safety_fund_account,
                &action.beneficiary,
                action.amount,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )
            .map_err(|_| Error::<T>::SafetyFundInsufficientBalance)?;

            // ── 手续费：从安全基金扣取，通过 FeeRouter 按 80/10/10 分账 ──
            let fee_imbalance = <T as duoqian_manage::Config>::Currency::withdraw(
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
            let proposal = voting_engine::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            ensure!(
                proposal.status == STATUS_PASSED,
                Error::<T>::ProposalNotPassed
            );

            let raw = voting_engine::Pallet::<T>::get_proposal_data(proposal_id)
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
            let (_, institution_account) = Self::resolve_institution_account(action.institution)?;
            ensure!(
                <T as duoqian_manage::Config>::InstitutionAsset::can_spend(
                    &institution_account,
                    InstitutionAssetAction::DuoqianTransferExecute,
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
            let free = <T as duoqian_manage::Config>::Currency::free_balance(&institution_account);
            let ed = <T as duoqian_manage::Config>::Currency::minimum_balance();
            let required = total
                .checked_add(&ed)
                .ok_or(Error::<T>::InsufficientBalance)?;
            ensure!(free >= required, Error::<T>::InsufficientBalance);

            // ── 原子执行：手续费扣取 + 转账，任一失败整体回滚 ──
            let exec_result = frame_support::storage::with_transaction(|| {
                // 先扣手续费
                let fee_imbalance = match <T as duoqian_manage::Config>::Currency::withdraw(
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
                match <T as duoqian_manage::Config>::Currency::transfer(
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
// 由投票引擎通过 [`voting_engine::InternalVoteResultCallback`] 广播回来。
// 本 Executor 按 `MODULE_TAG` 前缀 + 独立存储键认领对应业务:
// - `MODULE_TAG` 前缀 `dq-xfer` → transfer
// - `SafetyFundProposalActions[id]` 存在 → safety_fund
// - `SweepProposalActions[id]` 存在 → sweep
//
// 失败语义:执行失败发 ExecutionFailed 事件,提案保留 PASSED 状态,快照管理员
// 可通过 execute_X 手动重试(call_index 3/4/5),实际权限由 voting-engine 统一校验。
pub struct InternalVoteExecutor<T>(core::marker::PhantomData<T>);

impl<T: pallet::Config> InternalVoteResultCallback for InternalVoteExecutor<T> {
    fn on_internal_vote_finalized(
        proposal_id: u64,
        approved: bool,
    ) -> Result<ProposalExecutionOutcome, sp_runtime::DispatchError> {
        let is_transfer = voting_engine::Pallet::<T>::get_proposal_data(proposal_id)
            .map(|raw| raw.starts_with(crate::MODULE_TAG))
            .unwrap_or(false);
        let is_safety_fund = SafetyFundProposalActions::<T>::contains_key(proposal_id);
        let is_sweep = SweepProposalActions::<T>::contains_key(proposal_id);

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
                // 执行失败:发事件,提案保留 PASSED,供 execute_X 重试。
                if is_transfer {
                    if let Some(raw) = voting_engine::Pallet::<T>::get_proposal_data(proposal_id) {
                        if let Ok(action) =
                            TransferAction::<T::AccountId, BalanceOf<T>, T::MaxRemarkLen>::decode(
                                &mut &raw[crate::MODULE_TAG.len()..],
                            )
                        {
                            pallet::Pallet::<T>::deposit_event(
                                pallet::Event::<T>::TransferExecutionFailed {
                                    proposal_id,
                                    institution: action.institution,
                                },
                            );
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
mod tests {
    use super::*;
    use codec::Encode;
    use frame_support::{
        assert_noop, assert_ok, derive_impl,
        traits::{ConstU128, ConstU32},
    };
    use frame_system as system;
    use sp_core::{sr25519, Pair as PairT};
    use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};
    use voting_engine::{STATUS_EXECUTED, STATUS_REJECTED, STATUS_VOTING};

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
        pub type VotingEngine = voting_engine;

        #[runtime::pallet_index(3)]
        pub type DuoqianManage = duoqian_manage;

        #[runtime::pallet_index(4)]
        pub type DuoqianTransfer = super;

        #[runtime::pallet_index(5)]
        pub type AdminsChange = admins_change;
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

    pub struct TestAddressValidator;
    impl duoqian_manage::DuoqianAddressValidator<AccountId32> for TestAddressValidator {
        fn is_valid(address: &AccountId32) -> bool {
            address != &AccountId32::new([0u8; 32])
        }
    }

    pub struct TestReservedAddressChecker;
    impl duoqian_manage::DuoqianReservedAddressChecker<AccountId32> for TestReservedAddressChecker {
        fn is_reserved(address: &AccountId32) -> bool {
            *address == AccountId32::new([0xAA; 32])
        }
    }

    pub struct TestSfidInstitutionVerifier;
    impl
        duoqian_manage::SfidInstitutionVerifier<
            duoqian_manage::pallet::AccountNameOf<Test>,
            duoqian_manage::pallet::RegisterNonceOf<Test>,
            duoqian_manage::pallet::RegisterSignatureOf<Test>,
        > for TestSfidInstitutionVerifier
    {
        fn verify_institution_registration(
            _sfid_id: &[u8],
            institution_name: &duoqian_manage::pallet::AccountNameOf<Test>,
            account_names: &[alloc::vec::Vec<u8>],
            nonce: &duoqian_manage::pallet::RegisterNonceOf<Test>,
            signature: &duoqian_manage::pallet::RegisterSignatureOf<Test>,
            province: &[u8],
            signer_admin_pubkey: &[u8; 32],
        ) -> bool {
            !institution_name.is_empty()
                && !account_names.is_empty()
                && !nonce.is_empty()
                && !province.is_empty()
                && signer_admin_pubkey != &[0u8; 32]
                && signature.as_slice() == b"register-ok"
        }
    }

    pub struct TestSfidEligibility;
    impl voting_engine::SfidEligibility<AccountId32, <Test as frame_system::Config>::Hash>
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
            _province: &[u8],
            _signer_admin_pubkey: &[u8; 32],
        ) -> bool {
            true
        }
    }

    pub struct TestPopulationSnapshotVerifier;
    impl
        voting_engine::PopulationSnapshotVerifier<
            AccountId32,
            voting_engine::pallet::VoteNonceOf<Test>,
            voting_engine::pallet::VoteSignatureOf<Test>,
        > for TestPopulationSnapshotVerifier
    {
        fn verify_population_snapshot(
            _who: &AccountId32,
            _eligible_total: u64,
            _nonce: &voting_engine::pallet::VoteNonceOf<Test>,
            _signature: &voting_engine::pallet::VoteSignatureOf<Test>,
            _province: &[u8],
            _signer_admin_pubkey: &[u8; 32],
        ) -> bool {
            true
        }
    }

    // 测试扩展:
    // 原 TestInternalAdminProvider 只读 CHINA_CB/CHINA_CH 硬编码 admin(非真实 sr25519 公钥,无法签名)。
    // 为支持 `internal_vote` 的可签名测试账户,新增 thread_local 覆盖层:
    //   - EXTRA_ADMINS 按 (org, institution) 注入 sr25519 派生 admin 集合。
    // NRC/PRC/PRB 的内部阈值是 voting-engine 固定制度常量,测试必须注入足够管理员并投满该阈值。
    // 若某 (org, institution) 在 thread_local 有注入,优先用;否则 fallback 到原硬编码逻辑。
    thread_local! {
        static EXTRA_ADMINS: core::cell::RefCell<
            alloc::collections::BTreeMap<(u8, InstitutionPalletId), alloc::vec::Vec<AccountId32>>,
        > = core::cell::RefCell::new(alloc::collections::BTreeMap::new());
    }

    fn set_extra_admins(org: u8, institution: InstitutionPalletId, admins: Vec<AccountId32>) {
        EXTRA_ADMINS.with(|m| {
            m.borrow_mut().insert((org, institution), admins);
        });
    }

    fn get_extra_admins(org: u8, institution: InstitutionPalletId) -> Option<Vec<AccountId32>> {
        EXTRA_ADMINS.with(|m| m.borrow().get(&(org, institution)).cloned())
    }

    pub struct TestInternalAdminProvider;
    impl voting_engine::InternalAdminProvider<AccountId32> for TestInternalAdminProvider {
        fn is_internal_admin(org: u8, institution: InstitutionPalletId, who: &AccountId32) -> bool {
            // 优先:测试注入的 sr25519 派生 admin
            if let Some(admins) = get_extra_admins(org, institution) {
                return admins.iter().any(|a| a == who);
            }
            // Fallback:原硬编码 admin
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
                    .map(|n| n.duoqian_admins.iter().any(|admin| *admin == who_arr))
                    .unwrap_or(false),
                ORG_PRB => CHINA_CH
                    .iter()
                    .find(|n| shengbank_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                    .map(|n| n.duoqian_admins.iter().any(|admin| *admin == who_arr))
                    .unwrap_or(false),
                ORG_DUOQIAN => {
                    let Ok(account) = AccountId32::decode(&mut &institution[..32]) else {
                        return false;
                    };
                    if let Some(duoqian) = duoqian_manage::DuoqianAccounts::<Test>::get(&account) {
                        duoqian.duoqian_admins.iter().any(|admin| admin == who)
                    } else {
                        false
                    }
                }
                _ => false,
            }
        }

        fn get_admin_list(org: u8, institution: InstitutionPalletId) -> Option<Vec<AccountId32>> {
            if let Some(admins) = get_extra_admins(org, institution) {
                return Some(admins);
            }
            match org {
                ORG_NRC | ORG_PRC => CHINA_CB
                    .iter()
                    .find(|n| reserve_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                    .map(|n| {
                        n.duoqian_admins
                            .iter()
                            .copied()
                            .map(AccountId32::new)
                            .collect()
                    }),
                ORG_PRB => CHINA_CH
                    .iter()
                    .find(|n| shengbank_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                    .map(|n| {
                        n.duoqian_admins
                            .iter()
                            .copied()
                            .map(AccountId32::new)
                            .collect()
                    }),
                ORG_DUOQIAN => {
                    let account = AccountId32::decode(&mut &institution[..32]).ok()?;
                    let duoqian = duoqian_manage::DuoqianAccounts::<Test>::get(&account)?;
                    Some(duoqian.duoqian_admins.into_inner())
                }
                _ => None,
            }
        }
    }

    pub struct TestInternalAdminCountProvider;
    impl voting_engine::InternalAdminCountProvider for TestInternalAdminCountProvider {
        fn admin_count(org: u8, institution: InstitutionPalletId) -> Option<u32> {
            match org {
                ORG_NRC | ORG_PRC => CHINA_CB
                    .iter()
                    .find(|n| reserve_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                    .and_then(|n| u32::try_from(n.duoqian_admins.len()).ok()),
                ORG_PRB => CHINA_CH
                    .iter()
                    .find(|n| shengbank_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                    .and_then(|n| u32::try_from(n.duoqian_admins.len()).ok()),
                ORG_DUOQIAN => {
                    let account = AccountId32::decode(&mut &institution[..32]).ok()?;
                    let duoqian = duoqian_manage::DuoqianAccounts::<Test>::get(&account)?;
                    u32::try_from(duoqian.duoqian_admins.len()).ok()
                }
                _ => None,
            }
        }
    }

    pub struct TestInternalThresholdProvider;
    impl voting_engine::InternalThresholdProvider for TestInternalThresholdProvider {
        fn is_known_subject(org: u8, institution: InstitutionPalletId) -> bool {
            match org {
                ORG_DUOQIAN => AccountId32::decode(&mut &institution[..32])
                    .ok()
                    .and_then(|account| duoqian_manage::DuoqianAccounts::<Test>::get(&account))
                    .is_some(),
                _ => false,
            }
        }

        fn pass_threshold(org: u8, institution: InstitutionPalletId) -> Option<u32> {
            match org {
                ORG_NRC | ORG_PRC | ORG_PRB => {
                    voting_engine::internal_vote::fixed_governance_pass_threshold(org)
                }
                ORG_DUOQIAN => {
                    let account = AccountId32::decode(&mut &institution[..32]).ok()?;
                    let duoqian = duoqian_manage::DuoqianAccounts::<Test>::get(&account)?;
                    Some(duoqian.threshold)
                }
                _ => None,
            }
        }
    }

    thread_local! {
        static PROTECTED_ADDRESS: core::cell::RefCell<Option<AccountId32>> = core::cell::RefCell::new(None);
        static DENIED_SPEND_SOURCE: core::cell::RefCell<Option<AccountId32>> = core::cell::RefCell::new(None);
    }

    pub struct TestProtectedSourceChecker;
    impl duoqian_manage::ProtectedSourceChecker<AccountId32> for TestProtectedSourceChecker {
        fn is_protected(address: &AccountId32) -> bool {
            PROTECTED_ADDRESS.with(|pa| pa.borrow().as_ref() == Some(address))
        }
    }

    pub struct TestInstitutionAsset;
    impl institution_asset::InstitutionAsset<AccountId32> for TestInstitutionAsset {
        fn can_spend(
            source: &AccountId32,
            _action: institution_asset::InstitutionAssetAction,
        ) -> bool {
            DENIED_SPEND_SOURCE.with(|blocked| blocked.borrow().as_ref() != Some(source))
        }
    }

    pub struct TestTimeProvider;
    impl frame_support::traits::UnixTime for TestTimeProvider {
        fn now() -> core::time::Duration {
            core::time::Duration::from_secs(1_782_864_000) // 2026-07-01
        }
    }

    impl voting_engine::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type MaxVoteNonceLength = ConstU32<64>;
        type MaxVoteSignatureLength = ConstU32<64>;
        type MaxAutoFinalizePerBlock = ConstU32<64>;
        type MaxProposalsPerExpiry = ConstU32<128>;
        type MaxInternalProposalMutexBindings = ConstU32<256>;
        type MaxActiveProposals = ConstU32<10>;
        type MaxCleanupStepsPerBlock = ConstU32<8>;
        type CleanupKeysPerStep = ConstU32<64>;
        type SfidEligibility = TestSfidEligibility;
        type PopulationSnapshotVerifier = TestPopulationSnapshotVerifier;
        type JointVoteResultCallback = ();
        // Phase 2:挂上本模块 Executor,3 组业务提案通过后自动 try_execute_X。
        type InternalVoteResultCallback = crate::InternalVoteExecutor<Test>;
        type InternalAdminProvider = TestInternalAdminProvider;
        type InternalAdminCountProvider = TestInternalAdminCountProvider;
        type InternalThresholdProvider = TestInternalThresholdProvider;
        type MaxAdminsPerInstitution = ConstU32<64>;
        type MaxProposalDataLen = ConstU32<1024>;
        type MaxProposalObjectLen = ConstU32<{ 10 * 1024 }>;
        type MaxModuleTagLen = ConstU32<32>;
        type MaxManualExecutionAttempts = ConstU32<3>;
        type ExecutionRetryGraceBlocks = frame_support::traits::ConstU64<216>;
        type MaxExecutionRetryDeadlinesPerBlock = ConstU32<128>;
        type MaxCleanupQueueBucketLimit = ConstU32<50>;
        type MaxCleanupScheduleOffset = ConstU32<100>;
        type MaxPendingRetryExpirationsPerBlock = ConstU32<16>;
        type TimeProvider = TestTimeProvider;
        type WeightInfo = ();
    }

    impl duoqian_manage::pallet::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type Currency = Balances;
        type InternalVoteEngine = voting_engine::Pallet<Test>;
        type AddressValidator = TestAddressValidator;
        type ReservedAddressChecker = TestReservedAddressChecker;
        type ProtectedSourceChecker = TestProtectedSourceChecker;
        type InstitutionAsset = TestInstitutionAsset;
        type SfidInstitutionVerifier = TestSfidInstitutionVerifier;
        type FeeRouter = ();
        type MaxAdmins = ConstU32<10>;
        type MaxSfidIdLength = ConstU32<96>;
        type MaxAccountNameLength = ConstU32<128>;
        type MaxRegisterNonceLength = ConstU32<64>;
        type MaxRegisterSignatureLength = ConstU32<64>;
        type MaxAdminSignatureLength = ConstU32<64>;
        type MaxInstitutionAccounts = ConstU32<8>;
        type MinCreateAmount = ConstU128<111>;
        type MinCloseBalance = ConstU128<111>;
        type WeightInfo = ();
    }

    impl admins_change::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type MaxAdminsPerInstitution = ConstU32<64>;
        type InternalVoteEngine = voting_engine::Pallet<Test>;
        type WeightInfo = ();
    }

    impl pallet::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type MaxRemarkLen = ConstU32<256>;
        type FeeRouter = ();
        type WeightInfo = ();
    }

    /// 测试 helper:从 (org, institution, index) 派生 sr25519 keypair。
    ///
    /// 同 (org, institution, index) 每次调用返回相同 keypair,保证测试确定性。
    /// 公钥的 32 字节直接作为 AccountId32,满足 `pubkey_from_accountid` 的铁律。
    fn derive_admin_pair(
        org: u8,
        institution: &InstitutionPalletId,
        index: u8,
    ) -> (AccountId32, sr25519::Pair) {
        let mut seed_bytes = [0u8; 32];
        seed_bytes[0] = org;
        seed_bytes[1] = index;
        // 后 30 字节由 institution_pallet_id 前 30 字节填充,保证不同机构的 seed 不同
        seed_bytes[2..32].copy_from_slice(&institution[..30]);
        let pair = sr25519::Pair::from_seed(&seed_bytes);
        let account = AccountId32::new(pair.public().0);
        (account, pair)
    }

    fn nrc_admin(index: usize) -> AccountId32 {
        derive_admin_pair(ORG_NRC, &nrc_pallet_id(), index as u8).0
    }

    fn prc_admin(index: usize) -> AccountId32 {
        derive_admin_pair(ORG_PRC, &prc_pallet_id(), index as u8).0
    }

    fn prb_admin(index: usize) -> AccountId32 {
        derive_admin_pair(ORG_PRB, &prb_pallet_id(), index as u8).0
    }

    // 统一状态机整改:业务模块不再持有独立 vote/finalize call,投票统一走
    // `VotingEngine::internal_vote`;`cast_transfer_votes_n` 直接用 admin 账户逐个投票。

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

    fn registered_duoqian_account() -> AccountId32 {
        AccountId32::new([0x55; 32])
    }

    fn registered_duoqian_institution() -> InstitutionPalletId {
        duoqian_manage::account_to_institution_id(&registered_duoqian_account())
    }

    fn registered_duoqian_admin(index: usize) -> AccountId32 {
        registered_duoqian_pair(index).0
    }

    /// 注册多签(ORG_DUOQIAN)的 admin sr25519 keypair helper。
    /// seed 按 (ORG_DUOQIAN, registered_duoqian_institution, index) 派生,保证确定性。
    fn registered_duoqian_pair(index: usize) -> (AccountId32, sr25519::Pair) {
        derive_admin_pair(ORG_DUOQIAN, &registered_duoqian_institution(), index as u8)
    }

    fn registered_duoqian_pairs(count: u8) -> Vec<(AccountId32, sr25519::Pair)> {
        (0..count)
            .map(|i| registered_duoqian_pair(i as usize))
            .collect()
    }

    /// 收款人：使用一个不是管理员也不是机构的普通地址
    fn beneficiary() -> AccountId32 {
        AccountId32::new([99u8; 32])
    }

    /// 获取最近一次 create_internal_proposal 分配的 proposal_id。
    fn last_proposal_id() -> u64 {
        voting_engine::Pallet::<Test>::next_proposal_id().saturating_sub(1)
    }

    /// 返回 (org, institution) 对应的前 `count` 个 sr25519 admin keypair。
    fn admin_pairs(
        org: u8,
        institution: InstitutionPalletId,
        count: u8,
    ) -> Vec<(AccountId32, sr25519::Pair)> {
        (0..count)
            .map(|i| derive_admin_pair(org, &institution, i))
            .collect()
    }

    fn nrc_pairs(count: u8) -> Vec<(AccountId32, sr25519::Pair)> {
        admin_pairs(ORG_NRC, nrc_pallet_id(), count)
    }

    fn prc_pairs(count: u8) -> Vec<(AccountId32, sr25519::Pair)> {
        admin_pairs(ORG_PRC, prc_pallet_id(), count)
    }

    fn prb_pairs(count: u8) -> Vec<(AccountId32, sr25519::Pair)> {
        admin_pairs(ORG_PRB, prb_pallet_id(), count)
    }

    fn nrc_pass_count() -> usize {
        primitives::count_const::NRC_INTERNAL_THRESHOLD as usize
    }

    fn prc_pass_count() -> usize {
        primitives::count_const::PRC_INTERNAL_THRESHOLD as usize
    }

    fn prb_pass_count() -> usize {
        primitives::count_const::PRB_INTERNAL_THRESHOLD as usize
    }

    fn nrc_pass_pairs() -> Vec<(AccountId32, sr25519::Pair)> {
        nrc_pairs(primitives::count_const::NRC_INTERNAL_THRESHOLD as u8)
    }

    fn prc_pass_pairs() -> Vec<(AccountId32, sr25519::Pair)> {
        prc_pairs(primitives::count_const::PRC_INTERNAL_THRESHOLD as u8)
    }

    fn prb_pass_pairs() -> Vec<(AccountId32, sr25519::Pair)> {
        prb_pairs(primitives::count_const::PRB_INTERNAL_THRESHOLD as u8)
    }

    /// Phase 2 测试辅助:走投票引擎公开 `internal_vote` extrinsic,
    /// 让 `pairs` 前 `n` 个成员各投一张赞成票。
    ///
    /// 替代旧的聚合签名 helper——业务模块不再持有独立 finalize call,
    /// 通过后由 [`InternalVoteExecutor`] 自动触发 `try_execute_transfer`。
    /// 多余参数(`_org` / `_institution` / `_from` / `_to` /
    /// `_amount` / `_remark` / `_proposer`)保留占位,让调用端旧语义透明迁移。
    fn cast_transfer_votes_n(
        pairs: &[(AccountId32, sr25519::Pair)],
        n: usize,
        pid: u64,
        _org: u8,
        _institution: InstitutionPalletId,
        _from: AccountId32,
        _to: AccountId32,
        _amount: Balance,
        _remark: &[u8],
        _proposer: AccountId32,
    ) -> frame_support::dispatch::DispatchResult {
        for (admin, _pair) in pairs.iter().take(n) {
            VotingEngine::internal_vote(RuntimeOrigin::signed(admin.clone()), pid, true)?;
            if VotingEngine::proposals(pid)
                .map(|proposal| proposal.status != STATUS_VOTING)
                .unwrap_or(true)
            {
                break;
            }
        }
        Ok(())
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
        admins_change::GenesisConfig::<Test>::default()
            .assimilate_storage(&mut storage)
            .expect("admins-change genesis should assimilate");

        let mut ext: sp_io::TestExternalities = storage.into();
        ext.execute_with(|| {
            // 为 3 种固定治理 org 注入 sr25519 派生 admin。
            // 注入数量必须覆盖 voting-engine 的固定制度阈值,保证投票测试走真实状态机。
            // Provider 的 is_internal_admin / get_admin_list 会优先读 thread_local 注入,
            // 未注入时 fallback 到 CHINA_CB / CHINA_CH 硬编码。
            let nrc = nrc_pallet_id();
            let prc = prc_pallet_id();
            let prb = prb_pallet_id();
            let dq = registered_duoqian_institution();
            let nrc_accts: Vec<AccountId32> =
                nrc_pass_pairs().into_iter().map(|(a, _)| a).collect();
            let prc_accts: Vec<AccountId32> =
                prc_pass_pairs().into_iter().map(|(a, _)| a).collect();
            let prb_accts: Vec<AccountId32> =
                prb_pass_pairs().into_iter().map(|(a, _)| a).collect();
            set_extra_admins(ORG_NRC, nrc, nrc_accts);
            set_extra_admins(ORG_PRC, prc, prc_accts);
            set_extra_admins(ORG_PRB, prb, prb_accts);
            // ORG_DUOQIAN 的 admin / threshold 直接从 DuoqianAccounts 读,测试需要时
            // 显式写入 DuoqianAccounts(见 `registered_duoqian_admin` 路径)。
            let _ = dq;
        });
        ext
    }

    #[test]
    fn nrc_transfer_executes_when_internal_vote_reaches_threshold() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let inst_account = institution_account(institution);
            let dest = beneficiary();

            assert_ok!(DuoqianTransfer::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                1_000,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            assert_ok!(cast_transfer_votes_n(
                &nrc_pass_pairs(),
                nrc_pass_count(),
                pid,
                ORG_NRC,
                institution,
                inst_account.clone(),
                dest.clone(),
                1_000,
                &[],
                nrc_admin(0),
            ));

            // 转账已执行（含手续费 10）
            assert_eq!(Balances::free_balance(&inst_account), 8_990);
            assert_eq!(Balances::free_balance(&dest), 1_000);
            // 提案数据仍保留（由 voting-engine 延迟清理）
            assert!(voting_engine::Pallet::<Test>::get_proposal_data(pid).is_some());
        });
    }

    #[test]
    fn prc_transfer_executes_when_internal_vote_reaches_threshold() {
        new_test_ext().execute_with(|| {
            let institution = prc_pallet_id();
            let inst_account = institution_account(institution);
            let dest = beneficiary();

            assert_ok!(DuoqianTransfer::propose_transfer(
                RuntimeOrigin::signed(prc_admin(0)),
                ORG_PRC,
                institution,
                dest.clone(),
                2_000,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            assert_ok!(cast_transfer_votes_n(
                &prc_pass_pairs(),
                prc_pass_count(),
                pid,
                ORG_PRC,
                institution,
                inst_account.clone(),
                dest.clone(),
                2_000,
                &[],
                prc_admin(0),
            ));

            assert_eq!(Balances::free_balance(&inst_account), 7_990);
            assert_eq!(Balances::free_balance(&dest), 2_000);
            assert!(voting_engine::Pallet::<Test>::get_proposal_data(pid).is_some());
        });
    }

    #[test]
    fn prb_transfer_executes_when_internal_vote_reaches_threshold() {
        new_test_ext().execute_with(|| {
            let institution = prb_pallet_id();
            let inst_account = institution_account(institution);
            let dest = beneficiary();

            assert_ok!(DuoqianTransfer::propose_transfer(
                RuntimeOrigin::signed(prb_admin(0)),
                ORG_PRB,
                institution,
                dest.clone(),
                3_000,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            assert_ok!(cast_transfer_votes_n(
                &prb_pass_pairs(),
                prb_pass_count(),
                pid,
                ORG_PRB,
                institution,
                inst_account.clone(),
                dest.clone(),
                3_000,
                &[],
                prb_admin(0),
            ));

            assert_eq!(Balances::free_balance(&inst_account), 6_990);
            assert_eq!(Balances::free_balance(&dest), 3_000);
            assert!(voting_engine::Pallet::<Test>::get_proposal_data(pid).is_some());
        });
    }

    #[test]
    fn registered_duoqian_transfer_executes_when_internal_vote_reaches_threshold() {
        new_test_ext().execute_with(|| {
            let institution = registered_duoqian_institution();
            let inst_account = registered_duoqian_account();
            let dest = beneficiary();
            let admins = BoundedVec::try_from(vec![
                registered_duoqian_admin(0),
                registered_duoqian_admin(1),
                registered_duoqian_admin(2),
            ])
            .expect("admins should fit");

            duoqian_manage::DuoqianAccounts::<Test>::insert(
                &inst_account,
                duoqian_manage::DuoqianAccount {
                    admin_count: 3,
                    threshold: 2,
                    duoqian_admins: admins,
                    creator: registered_duoqian_admin(0),
                    created_at: 1,
                    status: duoqian_manage::DuoqianStatus::Active,
                },
            );
            let _ = Balances::deposit_creating(&inst_account, 10_000);

            assert_ok!(DuoqianTransfer::propose_transfer(
                RuntimeOrigin::signed(registered_duoqian_admin(0)),
                ORG_DUOQIAN,
                institution,
                dest.clone(),
                1_500,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            assert_ok!(cast_transfer_votes_n(
                &registered_duoqian_pairs(2),
                2,
                pid,
                ORG_DUOQIAN,
                institution,
                inst_account.clone(),
                dest.clone(),
                1_500,
                &[],
                registered_duoqian_admin(0),
            ));

            assert_eq!(Balances::free_balance(&inst_account), 8_490);
            assert_eq!(Balances::free_balance(&dest), 1_500);
            assert_eq!(
                voting_engine::Pallet::<Test>::proposals(pid)
                    .expect("proposal should exist")
                    .status,
                STATUS_EXECUTED
            );
        });
    }

    #[test]
    fn zero_amount_is_rejected() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let dest = beneficiary();

            assert_noop!(
                DuoqianTransfer::propose_transfer(
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
                DuoqianTransfer::propose_transfer(
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
                DuoqianTransfer::propose_transfer(
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
            assert_ok!(DuoqianTransfer::propose_transfer(
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
    fn multiple_proposals_allowed_within_limit() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let dest = beneficiary();

            assert_ok!(DuoqianTransfer::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                100,
                BoundedVec::default(),
            ));

            // 活跃提案数限制由 voting-engine 全局管控（上限 10），第二个提案可以成功
            assert_ok!(DuoqianTransfer::propose_transfer(
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
            let inst_account = institution_account(institution);
            let dest = beneficiary();

            assert_ok!(DuoqianTransfer::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                100,
                BoundedVec::default(),
            ));
            let pid1 = last_proposal_id();

            assert_ok!(cast_transfer_votes_n(
                &nrc_pass_pairs(),
                nrc_pass_count(),
                pid1,
                ORG_NRC,
                institution,
                inst_account,
                dest.clone(),
                100,
                &[],
                nrc_admin(0),
            ));

            // 转账已执行，可以创建新提案
            assert_ok!(DuoqianTransfer::propose_transfer(
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

            assert_ok!(DuoqianTransfer::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                100,
                BoundedVec::default(),
            ));
            let pid1 = last_proposal_id();

            let end = voting_engine::Pallet::<Test>::proposals(pid1)
                .expect("proposal should exist")
                .end;
            System::set_block_number(end + 1);
            assert_ok!(voting_engine::Pallet::<Test>::finalize_proposal(
                RuntimeOrigin::signed(nrc_admin(0)),
                pid1
            ));
            assert_eq!(
                voting_engine::Pallet::<Test>::proposals(pid1)
                    .expect("proposal should exist")
                    .status,
                STATUS_REJECTED
            );

            // 被拒绝后可以创建新提案
            assert_ok!(DuoqianTransfer::propose_transfer(
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
            assert_ok!(DuoqianTransfer::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                9_989,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            assert_ok!(cast_transfer_votes_n(
                &nrc_pass_pairs(),
                nrc_pass_count(),
                pid,
                ORG_NRC,
                institution,
                inst_account.clone(),
                dest.clone(),
                9_989,
                &[],
                nrc_admin(0),
            ));

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

            // 余额 10_000,提案 9_000(预检通过),然后在投票通过前转走 9_000。
            // 使余额仅 1_000,自动执行因余额不足失败,但提案保留,可 execute_transfer 重试。
            assert_ok!(DuoqianTransfer::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                9_000,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            // 投票通过前转走余额,使自动执行失败。
            let drain_dest = AccountId32::new([88u8; 32]);
            let _ = Balances::deposit_creating(&drain_dest, 1);
            assert_ok!(<Balances as frame_support::traits::Currency<_>>::transfer(
                &inst_account,
                &drain_dest,
                9_000,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            ));
            assert_eq!(Balances::free_balance(&inst_account), 1_000);

            // 投票达阈值后自动执行,但 try_execute_transfer 因余额不足失败。
            // 提案仍为 PASSED,转账未执行。
            assert_ok!(cast_transfer_votes_n(
                &nrc_pass_pairs(),
                nrc_pass_count(),
                pid,
                ORG_NRC,
                institution,
                inst_account.clone(),
                dest.clone(),
                9_000,
                &[],
                nrc_admin(0),
            ));
            assert_eq!(
                voting_engine::Pallet::<Test>::proposals(pid)
                    .expect("proposal should exist")
                    .status,
                STATUS_PASSED
            );
            assert_eq!(Balances::free_balance(&dest), 0);
            assert!(voting_engine::Pallet::<Test>::get_proposal_data(pid).is_some());

            // 补充余额后手动执行
            let _ = Balances::deposit_creating(&inst_account, 9_000);
            assert_eq!(Balances::free_balance(&inst_account), 10_000);
            assert_ok!(VotingEngine::retry_passed_proposal(
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

            assert_ok!(DuoqianTransfer::propose_transfer(
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
                VotingEngine::retry_passed_proposal(RuntimeOrigin::signed(nrc_admin(0)), pid),
                voting_engine::Error::<Test>::ProposalNotRetryable
            );
        });
    }

    #[test]
    fn execute_transfer_rejects_non_admin_retry() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let inst_account = institution_account(institution);
            let dest = beneficiary();
            let outsider = AccountId32::new([88u8; 32]);
            let _ = Balances::deposit_creating(&outsider, 1);

            assert_ok!(DuoqianTransfer::propose_transfer(
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

            assert_ok!(cast_transfer_votes_n(
                &nrc_pass_pairs(),
                nrc_pass_count(),
                pid,
                ORG_NRC,
                institution,
                inst_account.clone(),
                dest.clone(),
                100,
                &[],
                nrc_admin(0),
            ));

            // 自动执行失败，补充余额
            assert_eq!(Balances::free_balance(&dest), 0);
            let _ = Balances::deposit_creating(&inst_account, 10_000);

            // 统一重试入口只允许快照管理员手动重试。
            assert_noop!(
                VotingEngine::retry_passed_proposal(RuntimeOrigin::signed(outsider), pid),
                voting_engine::Error::<Test>::NoPermission
            );
            assert_eq!(Balances::free_balance(&dest), 0);
        });
    }

    #[test]
    fn executed_transfer_cannot_be_executed_again() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let inst_account = institution_account(institution);
            let dest = beneficiary();

            assert_ok!(DuoqianTransfer::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                1_000,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            assert_ok!(cast_transfer_votes_n(
                &nrc_pass_pairs(),
                nrc_pass_count(),
                pid,
                ORG_NRC,
                institution,
                inst_account,
                dest,
                1_000,
                &[],
                nrc_admin(0),
            ));

            // 自动执行成功，状态变为 EXECUTED
            assert_eq!(
                voting_engine::Pallet::<Test>::proposals(pid)
                    .expect("proposal should exist")
                    .status,
                STATUS_EXECUTED
            );

            // 再次调用 execute_transfer 应被拒绝
            assert_noop!(
                VotingEngine::retry_passed_proposal(RuntimeOrigin::signed(nrc_admin(0)), pid),
                voting_engine::Error::<Test>::ProposalNotRetryable
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
                DuoqianTransfer::propose_transfer(
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
    fn institution_spend_guard_blocks_transfer_proposal() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let source = institution_account(institution);
            let dest = beneficiary();
            DENIED_SPEND_SOURCE.with(|blocked| *blocked.borrow_mut() = Some(source.clone()));

            assert_noop!(
                DuoqianTransfer::propose_transfer(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    ORG_NRC,
                    institution,
                    dest,
                    100,
                    BoundedVec::default(),
                ),
                Error::<Test>::InstitutionSpendNotAllowed
            );

            DENIED_SPEND_SOURCE.with(|blocked| *blocked.borrow_mut() = None);
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
            assert_ok!(DuoqianTransfer::propose_transfer(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                dest.clone(),
                1,
                BoundedVec::default(),
            ));
            let pid = last_proposal_id();

            assert_ok!(cast_transfer_votes_n(
                &nrc_pass_pairs(),
                nrc_pass_count(),
                pid,
                ORG_NRC,
                institution,
                inst_account.clone(),
                dest.clone(),
                1,
                &[],
                nrc_admin(0),
            ));

            // 余额 10_000 - 1(转账) - 10(最低手续费) = 9_989
            assert_eq!(Balances::free_balance(&inst_account), 9_989);
            assert_eq!(Balances::free_balance(&dest), 1);
        });
    }
}
