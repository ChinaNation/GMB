#![cfg_attr(not(feature = "std"), no_std)]

//! 个人多签管理 pallet（pallet_index = 7,MODULE_TAG = `b"per-mgmt"`）。
//!
//! 业务边界:用户自定义的多签账户(无 CID 归属),由 `creator + account_name`
//! 派生地址 `derive_personal_account`。承载创建/关闭/管理员更换三类提案的发起、
//! 投票回调执行、否决/超时清理。
//!
//! 与机构多签 (`organization-manage`) 完全独立的 storage / event / error / extrinsic 命名空间;
//! 共用基础设施仅限于 `primitives::core_const` 派生函数、
//! `primitives::multisig` 校验抽象、`votingengine::InternalVoteEngine` 和
//! `admin-primitives` 共用管理员类型。

/// 模块标识前缀(8 字节,与 organization-manage 的 b"org-mgmt" 长度对仗)。
/// personal-admins / citizenwallet / citizenapp 三方解码必须保持一致。
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
pub use types::{PersonalAccount, PersonalCloseAction, PersonalCreateAction, PersonalStatus};

use admin_primitives::{
    AdminAccount, AdminAccountKind, AdminAccountLifecycle, AdminAccountStatus, AdminSetChangeAction,
};
use codec::{Decode, Encode};
use frame_support::{
    ensure,
    pallet_prelude::*,
    storage::{with_transaction, TransactionOutcome},
    traits::{Currency, ReservableCurrency},
    BoundedVec,
};
use frame_system::pallet_prelude::*;
use sp_std::prelude::*;
use votingengine::{
    InternalVoteEngine, InternalVoteResultCallback, ProposalExecutionOutcome,
    PROPOSAL_KIND_INTERNAL, STAGE_INTERNAL, STATUS_EXECUTION_FAILED, STATUS_PASSED,
    STATUS_REJECTED, STATUS_VOTING,
};

pub(crate) type BalanceOf<T> =
    <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::config]
    pub trait Config: frame_system::Config + votingengine::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// 内部投票引擎
        type InternalVoteEngine: votingengine::InternalVoteEngine<Self::AccountId>;

        type AccountValidator: primitives::multisig::AccountValidator<Self::AccountId>;
        type ReservedAccountChecker: primitives::multisig::ReservedAccountGuard<Self::AccountId>;
        type ProtectedSourceChecker: primitives::multisig::ProtectedSourceChecker<Self::AccountId>;
        type InstitutionAsset: institution_asset::InstitutionAsset<Self::AccountId>;

        /// 手续费分账路由(创建入金和注销转出的手续费)
        type FeeRouter: frame_support::traits::OnUnbalanced<
            <Self::Currency as Currency<Self::AccountId>>::NegativeImbalance,
        >;

        /// 个人多签账户名称最大字节数
        #[pallet::constant]
        type MaxAccountNameLength: Get<u32>;

        /// 单个个人多签账户管理员最大数量上限。
        #[pallet::constant]
        type MaxPersonalAccountAdmins: Get<u32>;

        /// 创建时最低入金(默认 111 分 = 1.11 元)
        #[pallet::constant]
        type MinCreateAmount: Get<BalanceOf<Self>>;

        /// 注销时账户最低余额门槛(默认 111 分 = 1.11 元)
        #[pallet::constant]
        type MinCloseBalance: Get<BalanceOf<Self>>;

        type WeightInfo: crate::weights::WeightInfo;
    }

    pub type AdminsOf<T> =
        BoundedVec<<T as frame_system::Config>::AccountId, <T as Config>::MaxPersonalAccountAdmins>;

    pub type AdminAccountOf<T> =
        AdminAccount<AdminsOf<T>, <T as frame_system::Config>::AccountId, BlockNumberFor<T>>;

    pub type PersonalAccountOf<T> = PersonalAccount<
        <T as frame_system::Config>::AccountId,
        AccountNameOf<T>,
        BlockNumberFor<T>,
    >;

    pub type AccountNameOf<T> = BoundedVec<u8, <T as Config>::MaxAccountNameLength>;

    pub type PersonalCreateActionOf<T> =
        PersonalCreateAction<<T as frame_system::Config>::AccountId, BalanceOf<T>>;

    pub type PersonalCloseActionOf<T> = PersonalCloseAction<<T as frame_system::Config>::AccountId>;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// 个人多签账户配置。key 为 personal_account。
    ///
    /// 中文注释:本表统一保存 creator/account_name/created_at/status。
    /// 管理员集合、管理员数量只允许从 personal-admins 读取；
    /// 普通动态阈值只允许从投票引擎 internal-vote 读取。
    #[pallet::storage]
    #[pallet::getter(fn personal_accounts)]
    pub type PersonalAccounts<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, PersonalAccountOf<T>, OptionQuery>;

    /// 个人多签管理员集合。key 为 personal_account。
    ///
    /// 中文注释：个人多签不依赖 CID 资料，链上只保存管理员 AccountId 集合；
    /// 账户名、创建者和生命周期展示资料继续由 `PersonalAccounts` 保存。
    #[pallet::storage]
    #[pallet::getter(fn admin_account_of)]
    pub type AdminAccounts<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, AdminAccountOf<T>, OptionQuery>;

    /// 正在投票中的个人多签创建提案,用于通过/拒绝时处理 reserve 资金。
    ///
    /// 资金模型: 发起时 reserve(amount + fee), 通过后 unreserve + transfer + withdraw fee,
    /// 否决/终态失败 unreserve。
    #[pallet::storage]
    #[pallet::getter(fn pending_personal_create)]
    pub type PendingPersonalCreate<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, PersonalCreateActionOf<T>, OptionQuery>;

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
        /// citizenapp 扫描此事件后引导其他管理员到投票引擎统一入口 `internal_vote` 投票。
        PersonalCreateProposed {
            proposal_id: u64,
            account: T::AccountId,
            proposer: T::AccountId,
            account_name: AccountNameOf<T>,
            admins: AdminsOf<T>,
            admins_len: u32,
            threshold: u32,
            amount: BalanceOf<T>,
            fee: BalanceOf<T>,
            expires_at: BlockNumberFor<T>,
        },
        /// 个人多签账户创建成功(投票通过,入金完成,状态变为 Active)。
        PersonalCreated {
            proposal_id: u64,
            account: T::AccountId,
            creator: T::AccountId,
            admins_len: u32,
            threshold: u32,
            amount: BalanceOf<T>,
            fee: BalanceOf<T>,
        },
        /// 创建提案投票通过但执行失败。
        CreateExecutionFailed {
            proposal_id: u64,
            account: T::AccountId,
        },
        /// 创建提案最终被拒绝(投票引擎返回 STATUS_REJECTED 后清理 Pending)。
        PersonalCreateRejected {
            proposal_id: u64,
            account: T::AccountId,
        },
        /// 关闭个人多签账户提案已发起。
        PersonalCloseProposed {
            proposal_id: u64,
            account: T::AccountId,
            proposer: T::AccountId,
            beneficiary: T::AccountId,
        },
        /// 个人多签账户注销成功(投票通过,余额转出,PersonalAccounts 删除)。
        PersonalClosed {
            proposal_id: u64,
            account: T::AccountId,
            beneficiary: T::AccountId,
            admins_len: u32,
            threshold: u32,
            amount: BalanceOf<T>,
            fee: BalanceOf<T>,
        },
        /// 关闭提案投票通过但执行失败。
        CloseExecutionFailed {
            proposal_id: u64,
            account: T::AccountId,
        },
        /// 已发起个人多签管理员集合变更提案。
        AdminSetChangeProposed {
            proposal_id: u64,
            account: T::AccountId,
            proposer: T::AccountId,
            old_admins_len: u32,
            new_admins_len: u32,
            new_threshold: u32,
        },
        /// 个人多签管理员集合已完成执行。
        AdminSetChanged {
            proposal_id: u64,
            account: T::AccountId,
            admins_len: u32,
            threshold: u32,
        },
        /// 个人多签管理员集合提案通过后执行失败。
        AdminSetChangeExecutionFailed { proposal_id: u64 },
    }

    #[pallet::error]
    pub enum Error<T> {
        IncompleteParameters,
        InvalidAccount,
        AccountReserved,
        DuplicateAdmin,
        InvalidThreshold,
        InsufficientAmount,
        CreateAmountBelowMinimum,
        CloseBalanceBelowMinimum,
        PermissionDenied,
        InvalidAdminsLen,
        AdminsLenMismatch,
        PersonalNotFound,
        PersonalNotActive,
        InvalidBeneficiary,
        ProtectedSource,
        DerivedAccountDecodeFailed,
        ReservedBalanceRemaining,
        VoteEngineError,
        ProposalActionNotFound,
        TransferFailed,
        EmptyPersonalName,
        PersonalAlreadyExists,
        CloseAlreadyPending,
        ProposalNotRejected,
        ReserveFailed,
        ReserveReleaseFailed,
        FeeWithdrawFailed,
        CloseTransferBelowED,
        /// propose_close 校验:仅个人多签账户可走本入口(非个人地址转 organization-manage)。
        NotPersonalAccount,
        /// 管理员集合没有发生变化。
        AdminSetUnchanged,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 发起"创建个人多签账户"提案(无需 CID 注册)。
        ///
        /// 地址由 `creator + account_name` 派生，统一调用
        /// `primitives::account_derive::AccountKind::Personal { creator, account_name }.derive(ss58)`。
        ///
        /// 投票通过后由 `InternalVoteExecutor` 自动执行入金 + 激活。
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::propose_create())]
        pub fn propose_create(
            origin: OriginFor<T>,
            account_name: AccountNameOf<T>,
            admins: AdminsOf<T>,
            regular_threshold: u32,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            crate::create::do_propose_create::<T>(
                who,
                account_name,
                admins,
                regular_threshold,
                amount,
            )
        }

        /// 发起"关闭个人多签账户"提案。
        ///
        /// 仅接受个人多签账户(`PersonalAccounts.contains_key` 命中);机构多签走 organization-manage。
        #[pallet::call_index(1)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::propose_close())]
        pub fn propose_close(
            origin: OriginFor<T>,
            account: T::AccountId,
            beneficiary: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            crate::close::do_propose_close::<T>(who, account, beneficiary)
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

        /// 发起个人多签管理员集合变更提案。
        ///
        /// 中文注释：个人多签管理员更换必须完整留在 personal-admins 内，
        /// 不走 genesis/public/private 管理员模块。
        #[pallet::call_index(3)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::propose_admin_set_change())]
        pub fn propose_admin_set_change(
            origin: OriginFor<T>,
            institution_code: votingengine::types::InstitutionCode,
            account: T::AccountId,
            admins: AdminsOf<T>,
            new_threshold: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                institution_code == votingengine::types::PMUL,
                Error::<T>::NotPersonalAccount
            );
            let personal =
                PersonalAccounts::<T>::get(&account).ok_or(Error::<T>::PersonalNotFound)?;
            ensure!(
                personal.status == PersonalStatus::Active,
                Error::<T>::PersonalNotActive
            );
            let current =
                AdminAccounts::<T>::get(account.clone()).ok_or(Error::<T>::PersonalNotFound)?;
            ensure!(
                current.status == AdminAccountStatus::Active
                    && current.kind == AdminAccountKind::PersonalMultisig
                    && current.institution_code == votingengine::types::PMUL,
                Error::<T>::NotPersonalAccount
            );
            let current_admins = current.admins.clone().into_inner();
            ensure!(current_admins.contains(&who), Error::<T>::PermissionDenied);
            Self::validate_admin_set_for_change(&admins, new_threshold)?;
            ensure!(
                !Self::same_admin_set(current_admins.as_slice(), admins.as_slice()),
                Error::<T>::AdminSetUnchanged
            );

            with_transaction(|| {
                let action = AdminSetChangeAction {
                    admin_root_account_id: account.clone(),
                    admins: admins.clone(),
                    new_threshold,
                };
                let proposal_id =
                    match T::InternalVoteEngine::create_admin_change_internal_proposal_with_data(
                        who.clone(),
                        votingengine::types::PMUL,
                        account.clone(),
                        admins.len() as u32,
                        new_threshold,
                        crate::MODULE_TAG,
                        action.encode(),
                    ) {
                        Ok(proposal_id) => proposal_id,
                        Err(err) => return TransactionOutcome::Rollback(Err(err)),
                    };
                Self::deposit_event(Event::<T>::AdminSetChangeProposed {
                    proposal_id,
                    account,
                    proposer: who,
                    old_admins_len: current_admins.len() as u32,
                    new_admins_len: admins.len() as u32,
                    new_threshold,
                });
                TransactionOutcome::Commit(Ok(()))
            })
        }
    }

    impl<T: Config> Pallet<T> {
        /// 派生个人多签账户。
        ///
        /// 地址只依赖 creator 与 account_name,与管理员列表无关,
        /// 所以未来换管理员地址不变。
        pub fn derive_personal_account(
            creator: &T::AccountId,
            account_name: &[u8],
        ) -> Result<T::AccountId, DispatchError> {
            // creator (AccountId32) 编码即 32 字节原始 pubkey;account_derive 的
            // Personal payload = creator(32B) || account_name,与历史拼装逐字节一致。
            let encoded = creator.encode();
            ensure!(encoded.len() >= 32, Error::<T>::DerivedAccountDecodeFailed);
            let mut creator_32 = [0u8; 32];
            creator_32.copy_from_slice(&encoded[..32]);
            let digest = primitives::account_derive::AccountKind::Personal {
                creator: &creator_32,
                account_name,
            }
            .derive(<T as frame_system::Config>::SS58Prefix::get());
            T::AccountId::decode(&mut &digest[..])
                .map_err(|_| Error::<T>::DerivedAccountDecodeFailed.into())
        }

        /// 校验管理员集合和用户输入的普通业务动态阈值。
        pub(crate) fn ensure_admin_config(
            who: &T::AccountId,
            admins: &AdminsOf<T>,
            regular_threshold: u32,
        ) -> Result<u32, DispatchError> {
            let admins_len = admins.len() as u32;
            ensure!(admins_len >= 2, Error::<T>::InvalidAdminsLen);
            ensure!(
                admins_len <= <T as Config>::MaxPersonalAccountAdmins::get(),
                Error::<T>::InvalidAdminsLen
            );
            ensure!(
                regular_threshold > 0
                    && regular_threshold <= admins_len
                    && u64::from(regular_threshold).saturating_mul(2) > u64::from(admins_len),
                Error::<T>::InvalidThreshold
            );
            Self::ensure_unique_admins(admins)?;
            ensure!(
                admins.iter().any(|admin| admin == who),
                Error::<T>::PermissionDenied
            );
            Ok(regular_threshold)
        }

        pub(crate) fn ensure_unique_admins(admins: &AdminsOf<T>) -> Result<(), DispatchError> {
            use sp_std::collections::btree_set::BTreeSet;
            let mut seen = BTreeSet::new();
            for admin in admins.iter() {
                ensure!(seen.insert(admin.clone()), Error::<T>::DuplicateAdmin);
            }
            Ok(())
        }

        fn validate_admin_set_for_change(
            admins: &AdminsOf<T>,
            new_threshold: u32,
        ) -> DispatchResult {
            let admins_len = admins.len() as u32;
            ensure!(admins_len >= 2, Error::<T>::InvalidAdminsLen);
            ensure!(
                admins_len <= <T as Config>::MaxPersonalAccountAdmins::get(),
                Error::<T>::InvalidAdminsLen
            );
            ensure!(
                new_threshold > 0
                    && new_threshold <= admins_len
                    && u64::from(new_threshold).saturating_mul(2) > u64::from(admins_len),
                Error::<T>::InvalidThreshold
            );
            Self::ensure_unique_admins(admins)?;
            Ok(())
        }

        fn same_admin_set(left: &[T::AccountId], right: &[T::AccountId]) -> bool {
            use sp_std::collections::btree_set::BTreeSet;
            if left.len() != right.len() {
                return false;
            }
            let left_set: BTreeSet<T::AccountId> = left.iter().cloned().collect();
            let right_set: BTreeSet<T::AccountId> = right.iter().cloned().collect();
            left_set == right_set
        }

        pub(crate) fn try_execute_set_change_from_action(
            proposal_id: u64,
            action: AdminSetChangeAction<T::AccountId, AdminsOf<T>>,
        ) -> DispatchResult {
            let proposal = votingengine::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            ensure!(
                proposal.kind == PROPOSAL_KIND_INTERNAL && proposal.stage == STAGE_INTERNAL,
                Error::<T>::ProposalActionNotFound
            );
            ensure!(
                proposal.status == STATUS_PASSED,
                Error::<T>::ProposalActionNotFound
            );
            ensure!(
                proposal.internal_institution == Some(action.admin_root_account_id.clone()),
                Error::<T>::ProposalActionNotFound
            );
            ensure!(
                proposal.internal_code == Some(votingengine::types::PMUL),
                Error::<T>::ProposalActionNotFound
            );
            votingengine::Pallet::<T>::ensure_admin_set_mutation_lock_owner(
                votingengine::types::PMUL,
                action.admin_root_account_id.clone(),
                proposal_id,
            )?;

            let current = AdminAccounts::<T>::get(action.admin_root_account_id.clone())
                .ok_or(Error::<T>::PersonalNotFound)?;
            ensure!(
                current.status == AdminAccountStatus::Active
                    && current.kind == AdminAccountKind::PersonalMultisig
                    && current.institution_code == votingengine::types::PMUL,
                Error::<T>::NotPersonalAccount
            );
            let current_admins = current.admins.clone().into_inner();
            Self::validate_admin_set_for_change(&action.admins, action.new_threshold)?;
            ensure!(
                !Self::same_admin_set(current_admins.as_slice(), action.admins.as_slice()),
                Error::<T>::AdminSetUnchanged
            );

            AdminAccounts::<T>::mutate(action.admin_root_account_id.clone(), |maybe| {
                if let Some(account) = maybe {
                    account.admins = action.admins.clone();
                    account.updated_at = frame_system::Pallet::<T>::block_number();
                }
            });
            Self::deposit_event(Event::<T>::AdminSetChanged {
                proposal_id,
                account: action.admin_root_account_id,
                admins_len: action.admins.len() as u32,
                threshold: action.new_threshold,
            });
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

        pub(crate) fn ensure_lifecycle_proposal(
            proposal_id: u64,
            module_tag: &[u8],
            account: T::AccountId,
            expected_status: u8,
            require_callback_scope: bool,
        ) -> DispatchResult {
            ensure!(
                votingengine::Pallet::<T>::is_proposal_owner(proposal_id, module_tag),
                Error::<T>::ProposalActionNotFound
            );
            let proposal = votingengine::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            ensure!(
                proposal.kind == PROPOSAL_KIND_INTERNAL,
                Error::<T>::ProposalActionNotFound
            );
            ensure!(
                proposal.stage == STAGE_INTERNAL,
                Error::<T>::ProposalActionNotFound
            );
            ensure!(
                proposal.internal_institution == Some(account),
                Error::<T>::ProposalActionNotFound
            );
            ensure!(
                proposal.internal_code == Some(votingengine::types::PMUL),
                Error::<T>::ProposalActionNotFound
            );
            ensure!(
                proposal.status == expected_status,
                Error::<T>::ProposalActionNotFound
            );
            if require_callback_scope {
                ensure!(
                    votingengine::Pallet::<T>::is_callback_execution_scope(proposal_id),
                    Error::<T>::ProposalActionNotFound
                );
            }
            Ok(())
        }

        pub(crate) fn do_create_pending_admin_account(
            account: T::AccountId,
            kind: AdminAccountKind,
            admins: Vec<T::AccountId>,
            creator: T::AccountId,
        ) -> DispatchResult {
            ensure!(
                kind == AdminAccountKind::PersonalMultisig,
                Error::<T>::NotPersonalAccount
            );
            ensure!(
                !AdminAccounts::<T>::contains_key(account.clone()),
                Error::<T>::PersonalAlreadyExists
            );
            let bounded: AdminsOf<T> = admins
                .try_into()
                .map_err(|_| Error::<T>::InvalidAdminsLen)?;
            Self::ensure_unique_admins(&bounded)?;
            let now = frame_system::Pallet::<T>::block_number();
            AdminAccounts::<T>::insert(
                account,
                AdminAccount {
                    institution_code: votingengine::types::PMUL,
                    kind,
                    admins: bounded,
                    creator,
                    created_at: now,
                    updated_at: now,
                    status: AdminAccountStatus::Pending,
                },
            );
            Ok(())
        }

        pub(crate) fn do_activate_admin_account(account: T::AccountId) -> DispatchResult {
            AdminAccounts::<T>::try_mutate(account, |maybe| -> DispatchResult {
                let admin_account = maybe.as_mut().ok_or(Error::<T>::PersonalNotFound)?;
                ensure!(
                    admin_account.status == AdminAccountStatus::Pending,
                    Error::<T>::PersonalNotActive
                );
                admin_account.status = AdminAccountStatus::Active;
                admin_account.updated_at = frame_system::Pallet::<T>::block_number();
                Ok(())
            })
        }

        pub(crate) fn do_remove_pending_admin_account(account: T::AccountId) -> DispatchResult {
            let admin_account =
                AdminAccounts::<T>::get(account.clone()).ok_or(Error::<T>::PersonalNotFound)?;
            ensure!(
                admin_account.status == AdminAccountStatus::Pending,
                Error::<T>::PersonalNotActive
            );
            AdminAccounts::<T>::remove(account);
            Ok(())
        }

        pub(crate) fn do_close_admin_account(account: T::AccountId) -> DispatchResult {
            let admin_account =
                AdminAccounts::<T>::get(account.clone()).ok_or(Error::<T>::PersonalNotFound)?;
            ensure!(
                admin_account.status == AdminAccountStatus::Active,
                Error::<T>::PersonalNotActive
            );
            AdminAccounts::<T>::remove(account);
            Ok(())
        }

        pub(crate) fn admin_account_with_status(
            institution_code: votingengine::types::InstitutionCode,
            account: T::AccountId,
            status: AdminAccountStatus,
        ) -> Option<AdminAccountOf<T>> {
            if institution_code != votingengine::types::PMUL {
                return None;
            }
            let admin_account = AdminAccounts::<T>::get(account)?;
            if admin_account.status == status
                && admin_account.kind == AdminAccountKind::PersonalMultisig
            {
                Some(admin_account)
            } else {
                None
            }
        }

        pub fn active_admin_account_exists(
            institution_code: votingengine::types::InstitutionCode,
            account: T::AccountId,
        ) -> bool {
            Self::admin_account_with_status(institution_code, account, AdminAccountStatus::Active)
                .is_some()
        }

        pub fn is_active_account_admin(
            institution_code: votingengine::types::InstitutionCode,
            account: T::AccountId,
            who: &T::AccountId,
        ) -> bool {
            let Some(admin_account) = Self::admin_account_with_status(
                institution_code,
                account,
                AdminAccountStatus::Active,
            ) else {
                return false;
            };
            admin_account.admins.iter().any(|admin| admin == who)
        }

        pub fn active_account_admins(
            institution_code: votingengine::types::InstitutionCode,
            account: T::AccountId,
        ) -> Option<Vec<T::AccountId>> {
            Some(
                Self::admin_account_with_status(
                    institution_code,
                    account,
                    AdminAccountStatus::Active,
                )?
                .admins
                .into_inner(),
            )
        }

        pub fn active_account_admins_len(
            institution_code: votingengine::types::InstitutionCode,
            account: T::AccountId,
        ) -> Option<u32> {
            Some(
                Self::admin_account_with_status(
                    institution_code,
                    account,
                    AdminAccountStatus::Active,
                )?
                .admins
                .len() as u32,
            )
        }

        pub fn pending_account_exists_for_snapshot(
            institution_code: votingengine::types::InstitutionCode,
            account: T::AccountId,
        ) -> bool {
            Self::admin_account_with_status(institution_code, account, AdminAccountStatus::Pending)
                .is_some()
        }

        pub fn is_pending_account_admin_for_snapshot(
            institution_code: votingengine::types::InstitutionCode,
            account: T::AccountId,
            who: &T::AccountId,
        ) -> bool {
            let Some(admin_account) = Self::admin_account_with_status(
                institution_code,
                account,
                AdminAccountStatus::Pending,
            ) else {
                return false;
            };
            admin_account.admins.iter().any(|admin| admin == who)
        }

        pub fn pending_account_admins_for_snapshot(
            institution_code: votingengine::types::InstitutionCode,
            account: T::AccountId,
        ) -> Option<Vec<T::AccountId>> {
            Some(
                Self::admin_account_with_status(
                    institution_code,
                    account,
                    AdminAccountStatus::Pending,
                )?
                .admins
                .into_inner(),
            )
        }

        pub fn pending_account_admins_len_for_snapshot(
            institution_code: votingengine::types::InstitutionCode,
            account: T::AccountId,
        ) -> Option<u32> {
            Some(
                Self::admin_account_with_status(
                    institution_code,
                    account,
                    AdminAccountStatus::Pending,
                )?
                .admins
                .len() as u32,
            )
        }

        pub(crate) fn create_pending_admin_account_for_proposal(
            proposal_id: u64,
            institution_id: T::AccountId,
            kind: AdminAccountKind,
            admins: &AdminsOf<T>,
            creator: &T::AccountId,
        ) -> DispatchResult {
            Self::ensure_lifecycle_proposal(
                proposal_id,
                crate::MODULE_TAG,
                institution_id.clone(),
                STATUS_VOTING,
                false,
            )?;
            Self::do_create_pending_admin_account(
                institution_id,
                kind,
                admins.iter().cloned().collect(),
                creator.clone(),
            )
        }

        pub(crate) fn activate_admin_account(
            proposal_id: u64,
            institution_id: T::AccountId,
        ) -> DispatchResult {
            Self::ensure_lifecycle_proposal(
                proposal_id,
                crate::MODULE_TAG,
                institution_id.clone(),
                STATUS_PASSED,
                true,
            )?;
            Self::do_activate_admin_account(institution_id)
        }

        pub(crate) fn remove_pending_admin_account(
            proposal_id: u64,
            institution_id: T::AccountId,
        ) -> DispatchResult {
            let proposal = votingengine::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            ensure!(
                matches!(proposal.status, STATUS_REJECTED | STATUS_EXECUTION_FAILED),
                Error::<T>::ProposalActionNotFound
            );
            Self::ensure_lifecycle_proposal(
                proposal_id,
                crate::MODULE_TAG,
                institution_id.clone(),
                proposal.status,
                false,
            )?;
            Self::do_remove_pending_admin_account(institution_id)
        }

        pub(crate) fn close_admin_account(
            proposal_id: u64,
            institution_id: T::AccountId,
        ) -> DispatchResult {
            Self::ensure_lifecycle_proposal(
                proposal_id,
                crate::MODULE_TAG,
                institution_id.clone(),
                STATUS_PASSED,
                true,
            )?;
            Self::do_close_admin_account(institution_id)
        }

        /// 统一解码本 pallet 的 ProposalData 前缀,避免回调和手动 cleanup 各写一套边界判断。
        pub(crate) fn decode_module_action(raw: &[u8]) -> Result<(u8, &[u8]), DispatchError> {
            let tag = crate::MODULE_TAG;
            ensure!(
                raw.len() > tag.len() && &raw[..tag.len()] == tag,
                Error::<T>::ProposalActionNotFound
            );
            Ok((raw[tag.len()], &raw[tag.len() + 1..]))
        }
    }
}

impl<T: pallet::Config> AdminAccountLifecycle<T::AccountId> for pallet::Pallet<T> {
    fn create_pending_admin_account_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        admin_root_account_id: T::AccountId,
        institution_code: votingengine::types::InstitutionCode,
        kind: AdminAccountKind,
        admins: Vec<T::AccountId>,
        creator: T::AccountId,
    ) -> DispatchResult {
        ensure!(
            module_tag == crate::MODULE_TAG && institution_code == votingengine::types::PMUL,
            pallet::Error::<T>::NotPersonalAccount
        );
        Self::ensure_lifecycle_proposal(
            proposal_id,
            module_tag,
            admin_root_account_id.clone(),
            STATUS_VOTING,
            false,
        )?;
        Self::do_create_pending_admin_account(admin_root_account_id, kind, admins, creator)
    }

    fn activate_admin_account_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        admin_root_account_id: T::AccountId,
    ) -> DispatchResult {
        ensure!(
            module_tag == crate::MODULE_TAG,
            pallet::Error::<T>::NotPersonalAccount
        );
        Self::ensure_lifecycle_proposal(
            proposal_id,
            module_tag,
            admin_root_account_id.clone(),
            STATUS_PASSED,
            true,
        )?;
        Self::do_activate_admin_account(admin_root_account_id)
    }

    fn remove_pending_admin_account_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        admin_root_account_id: T::AccountId,
    ) -> DispatchResult {
        ensure!(
            module_tag == crate::MODULE_TAG,
            pallet::Error::<T>::NotPersonalAccount
        );
        let proposal = votingengine::Pallet::<T>::proposals(proposal_id)
            .ok_or(pallet::Error::<T>::ProposalActionNotFound)?;
        ensure!(
            matches!(proposal.status, STATUS_REJECTED | STATUS_EXECUTION_FAILED),
            pallet::Error::<T>::ProposalActionNotFound
        );
        Self::ensure_lifecycle_proposal(
            proposal_id,
            module_tag,
            admin_root_account_id.clone(),
            proposal.status,
            false,
        )?;
        Self::do_remove_pending_admin_account(admin_root_account_id)
    }

    fn close_admin_account_for_proposal(
        proposal_id: u64,
        module_tag: &[u8],
        admin_root_account_id: T::AccountId,
    ) -> DispatchResult {
        ensure!(
            module_tag == crate::MODULE_TAG,
            pallet::Error::<T>::NotPersonalAccount
        );
        Self::ensure_lifecycle_proposal(
            proposal_id,
            module_tag,
            admin_root_account_id.clone(),
            STATUS_PASSED,
            true,
        )?;
        Self::do_close_admin_account(admin_root_account_id)
    }
}

impl<T: pallet::Config> admin_primitives::AdminAccountQuery<T::AccountId> for pallet::Pallet<T> {
    fn active_admin_account_exists(
        institution_code: votingengine::types::InstitutionCode,
        admin_root_account_id: T::AccountId,
    ) -> bool {
        Self::active_admin_account_exists(institution_code, admin_root_account_id)
    }

    fn is_active_account_admin(
        institution_code: votingengine::types::InstitutionCode,
        admin_root_account_id: T::AccountId,
        who: &T::AccountId,
    ) -> bool {
        Self::is_active_account_admin(institution_code, admin_root_account_id, who)
    }

    fn active_account_admins(
        institution_code: votingengine::types::InstitutionCode,
        admin_root_account_id: T::AccountId,
    ) -> Option<Vec<T::AccountId>> {
        Self::active_account_admins(institution_code, admin_root_account_id)
    }

    fn active_account_admins_len(
        institution_code: votingengine::types::InstitutionCode,
        admin_root_account_id: T::AccountId,
    ) -> Option<u32> {
        Self::active_account_admins_len(institution_code, admin_root_account_id)
    }

    fn pending_account_exists_for_snapshot(
        institution_code: votingengine::types::InstitutionCode,
        admin_root_account_id: T::AccountId,
    ) -> bool {
        Self::pending_account_exists_for_snapshot(institution_code, admin_root_account_id)
    }

    fn is_pending_account_admin_for_snapshot(
        institution_code: votingengine::types::InstitutionCode,
        admin_root_account_id: T::AccountId,
        who: &T::AccountId,
    ) -> bool {
        Self::is_pending_account_admin_for_snapshot(institution_code, admin_root_account_id, who)
    }

    fn pending_account_admins_for_snapshot(
        institution_code: votingengine::types::InstitutionCode,
        admin_root_account_id: T::AccountId,
    ) -> Option<Vec<T::AccountId>> {
        Self::pending_account_admins_for_snapshot(institution_code, admin_root_account_id)
    }

    fn pending_account_admins_len_for_snapshot(
        institution_code: votingengine::types::InstitutionCode,
        admin_root_account_id: T::AccountId,
    ) -> Option<u32> {
        Self::pending_account_admins_len_for_snapshot(institution_code, admin_root_account_id)
    }

    fn legal_representative(
        _institution_code: votingengine::types::InstitutionCode,
        _admin_root_account_id: T::AccountId,
    ) -> Option<T::AccountId> {
        None
    }
}

// ──── 投票终态回调:个人多签创建/关闭提案的执行落地 ────
//
// 投票统一由投票引擎承担,提案通过(或否决)经
// `votingengine::InternalVoteResultCallback` tuple 广播回来。
// 本 Executor 按 `MODULE_TAG + ACTION_*` 前缀认领本模块提案,
// approved=true 分派 execute_create / execute_close;approved=false 清理 Pending 存储。
// ──── PersonalMultisigQuery 实现:对 multisig-transfer / runtime config 暴露查询 ────

impl<T: pallet::Config> traits::PersonalMultisigQuery<T::AccountId> for pallet::Pallet<T> {
    fn lookup_admin_config(
        addr: &T::AccountId,
    ) -> Option<primitives::multisig::MultisigConfigSnapshot<T::AccountId>> {
        let account = pallet::PersonalAccounts::<T>::get(addr)?;
        if account.status != types::PersonalStatus::Active {
            return None;
        }
        let account = addr.clone();
        let institution_code = votingengine::types::PMUL;
        let admins = pallet::Pallet::<T>::active_account_admins(institution_code, account.clone())?;
        let admins_len =
            pallet::Pallet::<T>::active_account_admins_len(institution_code, account.clone())?;
        let threshold = <T as Config>::InternalVoteEngine::active_dynamic_threshold(
            institution_code,
            account.clone(),
        )?;
        Some(primitives::multisig::MultisigConfigSnapshot {
            admins,
            admins_len,
            threshold,
        })
    }

    fn is_active(addr: &T::AccountId) -> bool {
        matches!(
            pallet::PersonalAccounts::<T>::get(addr).map(|a| a.status),
            Some(types::PersonalStatus::Active)
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
        if !votingengine::Pallet::<T>::is_proposal_owner(proposal_id, crate::MODULE_TAG) {
            return Ok(ProposalExecutionOutcome::Ignored);
        }
        let raw = votingengine::Pallet::<T>::get_proposal_data(proposal_id)
            .ok_or(pallet::Error::<T>::ProposalActionNotFound)?;

        if !raw.starts_with(crate::MODULE_TAG) {
            if !approved {
                return Ok(ProposalExecutionOutcome::Executed);
            }
            let action =
                AdminSetChangeAction::<T::AccountId, pallet::AdminsOf<T>>::decode(&mut &raw[..])
                    .map_err(|_| pallet::Error::<T>::ProposalActionNotFound)?;
            return match pallet::Pallet::<T>::try_execute_set_change_from_action(
                proposal_id,
                action,
            ) {
                Ok(()) => Ok(ProposalExecutionOutcome::Executed),
                Err(_) => {
                    pallet::Pallet::<T>::deposit_event(
                        pallet::Event::<T>::AdminSetChangeExecutionFailed { proposal_id },
                    );
                    Ok(ProposalExecutionOutcome::FatalFailed)
                }
            };
        }
        let (action_byte, payload) = pallet::Pallet::<T>::decode_module_action(&raw)?;

        if approved {
            match action_byte {
                ACTION_CREATE => {
                    let action = pallet::PersonalCreateActionOf::<T>::decode(&mut &payload[..])
                        .map_err(|_| pallet::Error::<T>::ProposalActionNotFound)?;
                    let outcome = with_transaction(
                        || -> TransactionOutcome<Result<ProposalExecutionOutcome, sp_runtime::DispatchError>> {
                            match crate::execute::execute_create_with_finalizer::<T>(
                                proposal_id,
                                &action,
                            ) {
                                Ok(()) => TransactionOutcome::Commit(Ok(ProposalExecutionOutcome::Executed)),
                                Err(_) => TransactionOutcome::Rollback(Ok(
                                    ProposalExecutionOutcome::FatalFailed,
                                )),
                            }
                        },
                    )?;
                    Ok(outcome)
                }
                ACTION_CLOSE => {
                    let action = pallet::PersonalCloseActionOf::<T>::decode(&mut &payload[..])
                        .map_err(|_| pallet::Error::<T>::ProposalActionNotFound)?;
                    let outcome = with_transaction(
                        || -> TransactionOutcome<Result<ProposalExecutionOutcome, sp_runtime::DispatchError>> {
                            match crate::execute::execute_close_with_finalizer::<T>(
                                proposal_id,
                                &action,
                            ) {
                                Ok(()) => TransactionOutcome::Commit(Ok(ProposalExecutionOutcome::Executed)),
                                Err(_) => TransactionOutcome::Rollback(Ok(
                                    ProposalExecutionOutcome::FatalFailed,
                                )),
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
                    let action = pallet::PersonalCreateActionOf::<T>::decode(&mut &payload[..])
                        .map_err(|_| pallet::Error::<T>::ProposalActionNotFound)?;
                    crate::execute::cleanup_pending_create::<T>(proposal_id, &action, true)?;
                    Ok(ProposalExecutionOutcome::Executed)
                }
                ACTION_CLOSE => {
                    let action = pallet::PersonalCloseActionOf::<T>::decode(&mut &payload[..])
                        .map_err(|_| pallet::Error::<T>::ProposalActionNotFound)?;
                    pallet::PendingCloseProposal::<T>::remove(&action.account);
                    Ok(ProposalExecutionOutcome::Executed)
                }
                _ => Ok(ProposalExecutionOutcome::Ignored),
            }
        }
    }

    fn on_execution_failed_terminal(proposal_id: u64) -> DispatchResult {
        let raw = match votingengine::Pallet::<T>::get_proposal_data(proposal_id) {
            Some(raw) if raw.starts_with(crate::MODULE_TAG) => raw,
            _ => return Ok(()),
        };
        let (action_byte, payload) = pallet::Pallet::<T>::decode_module_action(&raw)?;
        match action_byte {
            ACTION_CREATE => {
                let action = pallet::PersonalCreateActionOf::<T>::decode(&mut &payload[..])
                    .map_err(|_| pallet::Error::<T>::ProposalActionNotFound)?;
                if crate::execute::cleanup_pending_create::<T>(proposal_id, &action, false)? {
                    pallet::Pallet::<T>::deposit_event(pallet::Event::<T>::CreateExecutionFailed {
                        proposal_id,
                        account: action.account,
                    });
                }
            }
            ACTION_CLOSE => {
                let action = pallet::PersonalCloseActionOf::<T>::decode(&mut &payload[..])
                    .map_err(|_| pallet::Error::<T>::ProposalActionNotFound)?;
                if pallet::PendingCloseProposal::<T>::take(&action.account).is_some() {
                    pallet::Pallet::<T>::deposit_event(pallet::Event::<T>::CloseExecutionFailed {
                        proposal_id,
                        account: action.account,
                    });
                }
            }
            _ => {}
        }
        Ok(())
    }
}
