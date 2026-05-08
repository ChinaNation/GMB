#![cfg_attr(not(feature = "std"), no_std)]

//! 个人多签管理 pallet（pallet_index = 7,MODULE_TAG = `b"per-mgmt"`）。
//!
//! 业务边界:用户自定义的多签账户(无 SFID 归属),由 `creator + account_name`
//! 派生地址 `derive_personal_duoqian_address`。承载创建/关闭两类提案的发起、
//! 投票回调执行、否决/超时清理。
//!
//! 与机构多签 (`organization-manage`) 完全独立的 storage / event / error / extrinsic 命名空间;
//! 共用基础设施仅限于 `primitives::derive` 派生函数 + `primitives::traits` 校验抽象 +
//! `votingengine::InternalVoteEngine` + `admins-change::AdminSubjectKind::PersonalDuoqian`。

/// 模块标识前缀(8 字节,与 organization-manage 的 b"org-mgmt" 长度对仗)。
/// admins-change / wumin / wuminapp 三方解码必须保持一致。
pub const MODULE_TAG: &[u8] = b"per-mgmt";

/// 提案动作类型常量,独立命名空间(从 0 起编号),与 organization-manage 的 ACTION 互不干扰。
pub const ACTION_CREATE: u8 = 0;
pub const ACTION_CLOSE: u8 = 1;

pub use pallet::*;

pub mod cleanup;
pub mod close;
pub mod create;
pub mod execute;
pub mod traits;
pub mod types;
pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;

#[cfg(test)]
mod tests;

pub use traits::PersonalMultisigQuery;
pub use types::{
    CloseDuoqianAction, CreateDuoqianAction, DuoqianAccount, DuoqianStatus, PersonalDuoqianMeta,
};

use admins_change::SubjectLifecycle;
use codec::{Decode, Encode};
use frame_support::{
    ensure,
    pallet_prelude::*,
    traits::{Currency, ReservableCurrency},
    BoundedVec,
};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::Hash;
use sp_std::prelude::*;
use votingengine::{InternalVoteResultCallback, ProposalExecutionOutcome, SubjectId};

pub(crate) type BalanceOf<T> =
    <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

    #[pallet::config]
    pub trait Config: frame_system::Config + votingengine::Config + admins_change::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// 内部投票引擎
        type InternalVoteEngine: votingengine::InternalVoteEngine<Self::AccountId>;

        type AddressValidator: primitives::traits::DuoqianAddressValidator<Self::AccountId>;
        type ReservedAddressChecker: primitives::traits::DuoqianReservedAddressChecker<
            Self::AccountId,
        >;
        type ProtectedSourceChecker: primitives::traits::ProtectedSourceChecker<Self::AccountId>;
        type InstitutionAsset: institution_asset::InstitutionAsset<Self::AccountId>;

        /// 手续费分账路由(创建入金和注销转出的手续费)
        type FeeRouter: frame_support::traits::OnUnbalanced<
            <Self::Currency as Currency<Self::AccountId>>::NegativeImbalance,
        >;

        /// 个人多签账户名称最大字节数
        #[pallet::constant]
        type MaxAccountNameLength: Get<u32>;

        /// 创建时最低入金(默认 111 分 = 1.11 元)
        #[pallet::constant]
        type MinCreateAmount: Get<BalanceOf<Self>>;

        /// 注销时账户最低余额门槛(默认 111 分 = 1.11 元)
        #[pallet::constant]
        type MinCloseBalance: Get<BalanceOf<Self>>;

        type WeightInfo: crate::weights::WeightInfo;
    }

    pub type DuoqianAdminsOf<T> = BoundedVec<
        <T as frame_system::Config>::AccountId,
        <T as admins_change::Config>::MaxPersonalAccountAdmins,
    >;

    pub type DuoqianAccountOf<T> =
        DuoqianAccount<<T as frame_system::Config>::AccountId, BlockNumberFor<T>>;

    pub type AccountNameOf<T> = BoundedVec<u8, <T as Config>::MaxAccountNameLength>;

    pub type CreateDuoqianActionOf<T> =
        CreateDuoqianAction<<T as frame_system::Config>::AccountId, BalanceOf<T>>;

    pub type CloseDuoqianActionOf<T> = CloseDuoqianAction<<T as frame_system::Config>::AccountId>;

    pub type PersonalDuoqianMetaOf<T> =
        PersonalDuoqianMeta<<T as frame_system::Config>::AccountId, AccountNameOf<T>>;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// 个人多签账户配置。key 为 personal_duoqian_address。
    #[pallet::storage]
    #[pallet::getter(fn personal_duoqians)]
    pub type PersonalDuoqians<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, DuoqianAccountOf<T>, OptionQuery>;

    /// 个人多签反向索引：duoqian_address -> { creator, account_name }
    #[pallet::storage]
    #[pallet::getter(fn personal_duoqian_info)]
    pub type PersonalDuoqianInfo<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, PersonalDuoqianMetaOf<T>, OptionQuery>;

    /// 正在投票中的个人多签创建提案,用于通过/拒绝时处理 reserve 资金。
    ///
    /// 资金模型: 发起时 reserve(amount + fee), 通过后 unreserve + transfer + withdraw fee,
    /// 否决/终态失败 unreserve。
    #[pallet::storage]
    #[pallet::getter(fn pending_personal_create)]
    pub type PendingPersonalCreate<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, CreateDuoqianActionOf<T>, OptionQuery>;

    /// 个人多签账户当前进行中的关闭提案 ID(防止并发注销提案)。
    /// 发起 propose_close 时写入,execute_close 成功或执行失败后清除。
    #[pallet::storage]
    #[pallet::getter(fn pending_close_proposal)]
    pub type PendingCloseProposal<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u64, OptionQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub _phantom: core::marker::PhantomData<T>,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                _phantom: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {}
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> Weight {
            let db = T::DbWeight::get();
            let on_chain = StorageVersion::get::<Pallet<T>>();
            if on_chain >= STORAGE_VERSION {
                return db.reads(1);
            }
            STORAGE_VERSION.put::<Pallet<T>>();
            db.reads_writes(1, 1)
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 个人多签账户创建提案已发起(pending 状态预写入)。
        /// wuminapp 扫描此事件后引导其他管理员到投票引擎统一入口 `internal_vote` 投票。
        PersonalDuoqianProposed {
            proposal_id: u64,
            duoqian_address: T::AccountId,
            proposer: T::AccountId,
            account_name: AccountNameOf<T>,
            admins: DuoqianAdminsOf<T>,
            admin_count: u32,
            threshold: u32,
            amount: BalanceOf<T>,
            expires_at: BlockNumberFor<T>,
        },
        /// 个人多签账户创建成功(投票通过,入金完成,状态变为 Active)。
        DuoqianCreated {
            proposal_id: u64,
            duoqian_address: T::AccountId,
            creator: T::AccountId,
            admin_count: u32,
            threshold: u32,
            amount: BalanceOf<T>,
            fee: BalanceOf<T>,
        },
        /// 创建提案投票通过但执行失败。
        CreateExecutionFailed {
            proposal_id: u64,
            duoqian_address: T::AccountId,
        },
        /// 创建提案最终被拒绝(投票引擎返回 STATUS_REJECTED 后清理 Pending)。
        DuoqianCreateRejected {
            proposal_id: u64,
            duoqian_address: T::AccountId,
        },
        /// 关闭个人多签账户提案已发起。
        CloseDuoqianProposed {
            proposal_id: u64,
            duoqian_address: T::AccountId,
            proposer: T::AccountId,
            beneficiary: T::AccountId,
        },
        /// 个人多签账户注销成功(投票通过,余额转出,PersonalDuoqians 删除)。
        DuoqianClosed {
            proposal_id: u64,
            duoqian_address: T::AccountId,
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
            fee: BalanceOf<T>,
        },
        /// 关闭提案投票通过但执行失败。
        CloseExecutionFailed {
            proposal_id: u64,
            duoqian_address: T::AccountId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        IncompleteParameters,
        InvalidAddress,
        AddressReserved,
        DuplicateAdmin,
        InvalidThreshold,
        InsufficientAmount,
        CreateAmountBelowMinimum,
        CloseBalanceBelowMinimum,
        PermissionDenied,
        InvalidAdminCount,
        AdminCountMismatch,
        DuoqianNotFound,
        DuoqianNotActive,
        InvalidBeneficiary,
        ProtectedSource,
        DerivedAddressDecodeFailed,
        ReservedBalanceRemaining,
        InvalidRuntimeConfig,
        VoteEngineError,
        ProposalActionNotFound,
        TransferFailed,
        EmptyPersonalName,
        PersonalDuoqianAlreadyExists,
        CloseAlreadyPending,
        ProposalNotRejected,
        ReserveFailed,
        ReserveReleaseFailed,
        FeeWithdrawFailed,
        CloseTransferBelowED,
        /// propose_close 校验:仅个人多签地址可走本入口(非个人地址转 organization-manage)。
        NotPersonalDuoqian,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 发起"创建个人多签账户"提案(无需 SFID 注册)。
        ///
        /// 地址由 `creator + account_name` 派生:
        /// `Blake2b_256(DUOQIAN_DOMAIN || OP_PERSONAL || SS58_PREFIX_LE || creator.encode() || name_utf8)`
        ///
        /// 投票通过后由 `InternalVoteExecutor` 自动执行入金 + 激活。
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::propose_create())]
        pub fn propose_create(
            origin: OriginFor<T>,
            account_name: AccountNameOf<T>,
            duoqian_admins: DuoqianAdminsOf<T>,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            crate::create::do_propose_create::<T>(who, account_name, duoqian_admins, amount)
        }

        /// 发起"关闭个人多签账户"提案。
        ///
        /// 仅接受个人多签地址(`PersonalDuoqians.contains_key` 命中);机构多签走 organization-manage。
        #[pallet::call_index(1)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::propose_close())]
        pub fn propose_close(
            origin: OriginFor<T>,
            duoqian_address: T::AccountId,
            beneficiary: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            crate::close::do_propose_close::<T>(who, duoqian_address, beneficiary)
        }

        /// 清理已被拒绝或超时的创建/关闭提案残留状态。
        /// 任意签名账户可调用。用于解决投票引擎 on_initialize 超时 reject 后
        /// 本模块无法自动收到通知导致的 Pending / PendingCloseProposal 残留。
        #[pallet::call_index(2)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::cleanup_rejected_proposal())]
        pub fn cleanup_rejected_proposal(origin: OriginFor<T>, proposal_id: u64) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            crate::cleanup::do_cleanup_rejected_proposal::<T>(proposal_id)
        }
    }

    impl<T: Config> Pallet<T> {
        /// 派生个人多签地址:
        /// `Blake2b_256(DUOQIAN_DOMAIN || OP_PERSONAL || SS58_PREFIX_LE || creator.encode() || name_utf8)`
        ///
        /// 地址只依赖 creator 与 account_name,与管理员列表无关,
        /// 所以未来换管理员地址不变。
        pub fn derive_personal_duoqian_address(
            creator: &T::AccountId,
            account_name: &[u8],
        ) -> Result<T::AccountId, DispatchError> {
            let mut input = primitives::core_const::DUOQIAN_DOMAIN.to_vec();
            input.push(primitives::core_const::OP_PERSONAL);
            input.extend_from_slice(&Self::chain_domain_prefix());
            input.extend_from_slice(&creator.encode());
            input.extend_from_slice(account_name);
            let digest = sp_runtime::traits::BlakeTwo256::hash(input.as_slice());
            T::AccountId::decode(&mut digest.as_ref())
                .map_err(|_| Error::<T>::DerivedAddressDecodeFailed.into())
        }

        /// 返回链域前缀(SS58 前缀的小端 u16 字节)。
        fn chain_domain_prefix() -> [u8; 2] {
            <T as frame_system::Config>::SS58Prefix::get().to_le_bytes()
        }

        /// 校验管理员集合并返回链端派生的普通业务阈值。
        pub(crate) fn ensure_admin_config(
            who: &T::AccountId,
            duoqian_admins: &DuoqianAdminsOf<T>,
        ) -> Result<u32, DispatchError> {
            let admin_count = duoqian_admins.len() as u32;
            ensure!(
                <T as admins_change::Config>::MaxPersonalAccountAdmins::get() >= 2,
                Error::<T>::InvalidRuntimeConfig
            );
            ensure!(admin_count >= 2, Error::<T>::InvalidAdminCount);
            ensure!(
                admin_count <= <T as admins_change::Config>::MaxPersonalAccountAdmins::get(),
                Error::<T>::InvalidAdminCount
            );
            Self::ensure_unique_admins(duoqian_admins)?;
            ensure!(
                duoqian_admins.iter().any(|admin| admin == who),
                Error::<T>::PermissionDenied
            );
            admins_change::derived_threshold(
                admins_change::AdminSubjectKind::PersonalDuoqian,
                votingengine::types::ORG_REN,
                admin_count,
            )
            .ok_or(Error::<T>::InvalidThreshold.into())
        }

        pub(crate) fn ensure_unique_admins(
            admins: &DuoqianAdminsOf<T>,
        ) -> Result<(), DispatchError> {
            use sp_std::collections::btree_set::BTreeSet;
            let mut seen = BTreeSet::new();
            for admin in admins.iter() {
                ensure!(seen.insert(admin.clone()), Error::<T>::DuplicateAdmin);
            }
            Ok(())
        }

        /// 校验发起人 free 余额覆盖 amount + fee + ED,返回 (reserve_total = amount + fee, fee)。
        pub(crate) fn ensure_proposer_can_afford(
            who: &T::AccountId,
            amount: BalanceOf<T>,
        ) -> Result<(BalanceOf<T>, BalanceOf<T>), DispatchError> {
            use sp_runtime::{traits::CheckedAdd, SaturatedConversion};
            let amount_u128: u128 = amount.saturated_into();
            let fee_u128 = onchain_transaction::calculate_onchain_fee(amount_u128);
            let fee: BalanceOf<T> = fee_u128.saturated_into();
            let reserve_total = amount
                .checked_add(&fee)
                .ok_or(Error::<T>::InsufficientAmount)?;
            let ed = T::Currency::minimum_balance();
            let required = reserve_total
                .checked_add(&ed)
                .ok_or(Error::<T>::InsufficientAmount)?;
            ensure!(
                T::Currency::free_balance(who) >= required,
                Error::<T>::InsufficientAmount
            );
            Ok((reserve_total, fee))
        }

        pub(crate) fn create_pending_admin_subject_for_proposal(
            proposal_id: u64,
            institution_id: SubjectId,
            kind: admins_change::AdminSubjectKind,
            admins: &DuoqianAdminsOf<T>,
            creator: &T::AccountId,
        ) -> DispatchResult {
            admins_change::Pallet::<T>::create_pending_subject_for_proposal(
                proposal_id,
                crate::MODULE_TAG,
                institution_id,
                votingengine::types::ORG_REN,
                kind,
                admins.iter().cloned().collect(),
                creator.clone(),
            )
        }

        pub(crate) fn activate_admin_subject(
            proposal_id: u64,
            institution_id: SubjectId,
        ) -> DispatchResult {
            admins_change::Pallet::<T>::activate_subject_for_proposal(
                proposal_id,
                crate::MODULE_TAG,
                institution_id,
            )
        }

        pub(crate) fn remove_pending_admin_subject(proposal_id: u64, institution_id: SubjectId) {
            let _ = admins_change::Pallet::<T>::remove_pending_subject_for_proposal(
                proposal_id,
                crate::MODULE_TAG,
                institution_id,
            );
        }

        pub(crate) fn close_admin_subject(
            proposal_id: u64,
            institution_id: SubjectId,
        ) -> DispatchResult {
            admins_change::Pallet::<T>::close_subject_for_proposal(
                proposal_id,
                crate::MODULE_TAG,
                institution_id,
            )
        }
    }
}

// ──── 投票终态回调:个人多签创建/关闭提案的执行落地 ────
//
// 投票统一由投票引擎承担,提案通过(或否决)经
// `votingengine::InternalVoteResultCallback` tuple 广播回来。
// 本 Executor 按 `MODULE_TAG + ACTION_*` 前缀认领本模块提案,
// approved=true 分派 execute_create / execute_close;approved=false 清理 Pending 存储。
// ──── PersonalMultisigQuery 实现:对 duoqian-transfer / runtime config 暴露查询 ────

impl<T: pallet::Config> traits::PersonalMultisigQuery<T::AccountId> for pallet::Pallet<T> {
    fn lookup_admin_config(
        addr: &T::AccountId,
    ) -> Option<primitives::types::MultisigConfigSnapshot<T::AccountId>> {
        let account = pallet::PersonalDuoqians::<T>::get(addr)?;
        if account.status != types::DuoqianStatus::Active {
            return None;
        }
        let subject = primitives::derive::subject_id_from_account(addr);
        let org = votingengine::types::ORG_REN;
        let admins = admins_change::Pallet::<T>::active_subject_admins(org, subject)?;
        let admin_count = admins_change::Pallet::<T>::active_subject_admin_count(org, subject)?;
        let threshold = admins_change::Pallet::<T>::active_subject_threshold(org, subject)?;
        Some(primitives::types::MultisigConfigSnapshot {
            admins,
            admin_count,
            threshold,
        })
    }

    fn is_active(addr: &T::AccountId) -> bool {
        matches!(
            pallet::PersonalDuoqians::<T>::get(addr).map(|a| a.status),
            Some(types::DuoqianStatus::Active)
        )
    }
}

pub struct InternalVoteExecutor<T>(core::marker::PhantomData<T>);

impl<T: pallet::Config> InternalVoteResultCallback for InternalVoteExecutor<T> {
    fn on_internal_vote_finalized(
        proposal_id: u64,
        approved: bool,
    ) -> Result<ProposalExecutionOutcome, sp_runtime::DispatchError> {
        use frame_support::storage::{with_transaction, TransactionOutcome};
        let raw = match votingengine::Pallet::<T>::get_proposal_data(proposal_id) {
            Some(raw) if raw.starts_with(crate::MODULE_TAG) => raw,
            _ => return Ok(ProposalExecutionOutcome::Ignored),
        };
        let tag = crate::MODULE_TAG;
        if raw.len() <= tag.len() {
            return Ok(ProposalExecutionOutcome::Ignored);
        }
        let action_byte = raw[tag.len()];

        if approved {
            match action_byte {
                ACTION_CREATE => {
                    let action =
                        pallet::CreateDuoqianActionOf::<T>::decode(&mut &raw[tag.len() + 1..])
                            .map_err(|_| pallet::Error::<T>::ProposalActionNotFound)?;
                    let outcome = with_transaction(
                        || -> TransactionOutcome<Result<ProposalExecutionOutcome, sp_runtime::DispatchError>> {
                            match crate::execute::execute_create_with_finalizer::<T>(
                                proposal_id,
                                &action,
                                approved,
                            ) {
                                Ok(()) => TransactionOutcome::Commit(Ok(ProposalExecutionOutcome::Executed)),
                                Err(_) => {
                                    pallet::Pallet::<T>::deposit_event(
                                        pallet::Event::<T>::CreateExecutionFailed {
                                            proposal_id,
                                            duoqian_address: action.duoqian_address.clone(),
                                        },
                                    );
                                    TransactionOutcome::Rollback(Ok(
                                        ProposalExecutionOutcome::FatalFailed,
                                    ))
                                }
                            }
                        },
                    )?;
                    Ok(outcome)
                }
                ACTION_CLOSE => {
                    let action =
                        pallet::CloseDuoqianActionOf::<T>::decode(&mut &raw[tag.len() + 1..])
                            .map_err(|_| pallet::Error::<T>::ProposalActionNotFound)?;
                    let outcome = with_transaction(
                        || -> TransactionOutcome<Result<ProposalExecutionOutcome, sp_runtime::DispatchError>> {
                            match crate::execute::execute_close_with_finalizer::<T>(
                                proposal_id,
                                &action,
                                approved,
                            ) {
                                Ok(()) => TransactionOutcome::Commit(Ok(ProposalExecutionOutcome::Executed)),
                                Err(_) => {
                                    pallet::Pallet::<T>::deposit_event(
                                        pallet::Event::<T>::CloseExecutionFailed {
                                            proposal_id,
                                            duoqian_address: action.duoqian_address.clone(),
                                        },
                                    );
                                    TransactionOutcome::Rollback(Ok(
                                        ProposalExecutionOutcome::FatalFailed,
                                    ))
                                }
                            }
                        },
                    )?;
                    Ok(outcome)
                }
                _ => Ok(ProposalExecutionOutcome::Ignored),
            }
        } else {
            // 否决路径:清理 Pending 存储 + unreserve 资金。
            match action_byte {
                ACTION_CREATE => {
                    let action =
                        pallet::CreateDuoqianActionOf::<T>::decode(&mut &raw[tag.len() + 1..])
                            .map_err(|_| pallet::Error::<T>::ProposalActionNotFound)?;
                    crate::execute::cleanup_pending_create::<T>(proposal_id, &action, true);
                    Ok(ProposalExecutionOutcome::Executed)
                }
                ACTION_CLOSE => {
                    let action =
                        pallet::CloseDuoqianActionOf::<T>::decode(&mut &raw[tag.len() + 1..])
                            .map_err(|_| pallet::Error::<T>::ProposalActionNotFound)?;
                    pallet::PendingCloseProposal::<T>::remove(&action.duoqian_address);
                    Ok(ProposalExecutionOutcome::Executed)
                }
                _ => Ok(ProposalExecutionOutcome::Ignored),
            }
        }
    }
}
