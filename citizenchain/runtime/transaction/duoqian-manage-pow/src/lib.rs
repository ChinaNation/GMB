#![cfg_attr(not(feature = "std"), no_std)]

/// 模块标识前缀，用于在 ProposalData 中区分不同业务模块，防止跨模块误解码。
pub const MODULE_TAG: &[u8] = b"dq-mgmt";

pub use pallet::*;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod weights;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    ensure,
    pallet_prelude::*,
    traits::{Currency, ExistenceRequirement, OnUnbalanced, ReservableCurrency},
    BoundedVec,
};
use frame_system::pallet_prelude::*;
use institution_asset_guard::{InstitutionAssetAction, InstitutionAssetGuard};
use scale_info::TypeInfo;
use sp_runtime::{traits::{Hash, Zero}, SaturatedConversion, TransactionOutcome};
use frame_support::storage::with_transaction;
use sp_std::{collections::btree_set::BTreeSet, prelude::*};
use voting_engine_system::{InstitutionPalletId, STATUS_EXECUTED, STATUS_PASSED, STATUS_REJECTED};

type BalanceOf<T> =
    <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// 账户地址合法性抽象：用于校验 duoqian_address 是否为本链合法哈希地址。
pub trait DuoqianAddressValidator<AccountId> {
    fn is_valid(address: &AccountId) -> bool;
}

impl<AccountId> DuoqianAddressValidator<AccountId> for () {
    fn is_valid(_address: &AccountId) -> bool {
        true
    }
}

/// 保留地址校验抽象：用于拦截制度保留地址被 duoqian 抢注册。
pub trait DuoqianReservedAddressChecker<AccountId> {
    fn is_reserved(address: &AccountId) -> bool;
}

impl<AccountId> DuoqianReservedAddressChecker<AccountId> for () {
    fn is_reserved(_address: &AccountId) -> bool {
        false
    }
}

/// 转出源地址保护：用于禁止制度保留地址作为资金转出源。
pub trait ProtectedSourceChecker<AccountId> {
    fn is_protected(address: &AccountId) -> bool;
}

impl<AccountId> ProtectedSourceChecker<AccountId> for () {
    fn is_protected(_address: &AccountId) -> bool {
        false
    }
}

/// SFID 机构登记验签抽象：链上只信任 SFID 对 `sfid_id + name + register_nonce` 的主公钥签名。
pub trait SfidInstitutionVerifier<Name, Nonce, Signature> {
    fn verify_institution_registration(
        sfid_id: &[u8],
        name: &Name,
        nonce: &Nonce,
        signature: &Signature,
    ) -> bool;
}

impl<Name, Nonce, Signature> SfidInstitutionVerifier<Name, Nonce, Signature> for () {
    fn verify_institution_registration(
        _sfid_id: &[u8],
        _name: &Name,
        _nonce: &Nonce,
        _signature: &Signature,
    ) -> bool {
        false
    }
}

/// 多签账户状态
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    Copy,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
)]
pub enum DuoqianStatus {
    /// 提案投票中，尚未激活
    Pending,
    /// 已激活（投票通过并入金完成）
    Active,
}

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
#[scale_info(skip_type_params(AdminList))]
pub struct DuoqianAccount<AdminList, AccountId, BlockNumber> {
    pub admin_count: u32,
    pub threshold: u32,
    pub duoqian_admins: AdminList,
    pub creator: AccountId,
    pub created_at: BlockNumber,
    pub status: DuoqianStatus,
}

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
pub struct RegisteredInstitution<SfidId, SfidName> {
    pub sfid_id: SfidId,
    pub name: SfidName,
}

/// 创建多签账户提案的业务数据（存入投票引擎 ProposalData）
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct CreateDuoqianAction<AccountId, Balance> {
    pub duoqian_address: AccountId,
    pub proposer: AccountId,
    pub admin_count: u32,
    pub threshold: u32,
    pub amount: Balance,
}

/// 关闭多签账户提案的业务数据
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct CloseDuoqianAction<AccountId> {
    pub duoqian_address: AccountId,
    pub beneficiary: AccountId,
    pub proposer: AccountId,
}

/// 个人多签账户元数据（存储在 PersonalDuoqianInfo 中）
#[derive(
    Encode, Decode, DecodeWithMemTracking, Clone, RuntimeDebug, TypeInfo, MaxEncodedLen,
    PartialEq, Eq,
)]
pub struct PersonalDuoqianMeta<AccountId, Name> {
    pub creator: AccountId,
    pub name: Name,
}

/// 将 AccountId（32 字节）转为 InstitutionPalletId（48 字节），右填充 16 个零。
pub fn account_to_institution_id<AccountId: Encode>(account: &AccountId) -> InstitutionPalletId {
    let encoded = account.encode();
    let mut id = [0u8; 48];
    let copy_len = core::cmp::min(encoded.len(), 32);
    id[..copy_len].copy_from_slice(&encoded[..copy_len]);
    id
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;
    use voting_engine_system::InternalAdminProvider;
    use voting_engine_system::InternalVoteEngine;
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(6);

    #[pallet::config]
    pub trait Config: frame_system::Config + voting_engine_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// 内部投票引擎
        type InternalVoteEngine: voting_engine_system::InternalVoteEngine<Self::AccountId>;

        type AddressValidator: DuoqianAddressValidator<Self::AccountId>;
        type ReservedAddressChecker: DuoqianReservedAddressChecker<Self::AccountId>;
        type ProtectedSourceChecker: ProtectedSourceChecker<Self::AccountId>;
        type InstitutionAssetGuard: institution_asset_guard::InstitutionAssetGuard<Self::AccountId>;
        type SfidInstitutionVerifier: SfidInstitutionVerifier<
            SfidNameOf<Self>,
            RegisterNonceOf<Self>,
            RegisterSignatureOf<Self>,
        >;

        /// 手续费分账路由（创建入金和注销转出的手续费）
        type FeeRouter: frame_support::traits::OnUnbalanced<
            <Self::Currency as Currency<Self::AccountId>>::NegativeImbalance,
        >;

        #[pallet::constant]
        type MaxAdmins: Get<u32>;

        #[pallet::constant]
        type MaxSfidIdLength: Get<u32>;

        /// 机构名称最大字节长度。
        #[pallet::constant]
        type MaxSfidNameLength: Get<u32>;

        #[pallet::constant]
        type MaxRegisterNonceLength: Get<u32>;

        #[pallet::constant]
        type MaxRegisterSignatureLength: Get<u32>;

        /// 创建时最低入金（默认应设置为 111 分 = 1.11 元）。
        #[pallet::constant]
        type MinCreateAmount: Get<BalanceOf<Self>>;

        /// 注销时账户最低余额门槛（默认应设置为 111 分 = 1.11 元）。
        #[pallet::constant]
        type MinCloseBalance: Get<BalanceOf<Self>>;

        type WeightInfo: crate::weights::WeightInfo;
    }

    pub type DuoqianAdminsOf<T> =
        BoundedVec<<T as frame_system::Config>::AccountId, <T as Config>::MaxAdmins>;

    pub type DuoqianAccountOf<T> = DuoqianAccount<
        DuoqianAdminsOf<T>,
        <T as frame_system::Config>::AccountId,
        BlockNumberFor<T>,
    >;

    pub type SfidIdOf<T> = BoundedVec<u8, <T as Config>::MaxSfidIdLength>;
    pub type SfidNameOf<T> = BoundedVec<u8, <T as Config>::MaxSfidNameLength>;
    pub type RegisterNonceOf<T> = BoundedVec<u8, <T as Config>::MaxRegisterNonceLength>;
    pub type RegisterSignatureOf<T> = BoundedVec<u8, <T as Config>::MaxRegisterSignatureLength>;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// 多签账户配置。key 为 duoqian_address。
    #[pallet::storage]
    #[pallet::getter(fn duoqian_account_of)]
    pub type DuoqianAccounts<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, DuoqianAccountOf<T>, OptionQuery>;

    /// SFID 机构登记：(sfid_id, name) -> duoqian_address（由 blake2b_256 派生）。
    /// 同一 sfid_id 可通过不同 name 注册多个多签地址。
    #[pallet::storage]
    pub type SfidRegisteredAddress<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        SfidIdOf<T>,
        Blake2_128Concat,
        SfidNameOf<T>,
        T::AccountId,
        OptionQuery,
    >;

    /// SFID 机构登记反向索引：duoqian_address -> { sfid_id, nonce }
    #[pallet::storage]
    #[pallet::getter(fn address_registered_sfid)]
    pub type AddressRegisteredSfid<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        RegisteredInstitution<SfidIdOf<T>, SfidNameOf<T>>,
        OptionQuery,
    >;

    /// 已消费的机构登记 nonce，防止 proof 重放。
    #[pallet::storage]
    #[pallet::getter(fn used_register_nonce)]
    pub type UsedRegisterNonce<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, bool, ValueQuery>;

    /// 个人多签反向索引：duoqian_address -> { creator, name }
    #[pallet::storage]
    #[pallet::getter(fn personal_duoqian_info)]
    pub type PersonalDuoqianInfo<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        PersonalDuoqianMeta<T::AccountId, SfidNameOf<T>>,
        OptionQuery,
    >;

    /// 每个多签账户当前进行中的关闭提案 ID（防止并发注销提案）。
    /// 发起 propose_close 时写入，execute_close 成功或执行失败后清除。
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
        /// 创建多签账户提案已发起（pending 状态预写入）
        CreateDuoqianProposed {
            proposal_id: u64,
            duoqian_address: T::AccountId,
            proposer: T::AccountId,
            admin_count: u32,
            threshold: u32,
            amount: BalanceOf<T>,
        },
        /// 创建多签投票已提交
        CreateVoteSubmitted {
            proposal_id: u64,
            who: T::AccountId,
            approve: bool,
        },
        /// 多签账户创建成功（投票通过，入金完成，状态变为 Active）
        DuoqianCreated {
            proposal_id: u64,
            duoqian_address: T::AccountId,
            creator: T::AccountId,
            admin_count: u32,
            threshold: u32,
            amount: BalanceOf<T>,
            fee: BalanceOf<T>,
        },
        /// 创建提案投票通过但执行失败
        CreateExecutionFailed {
            proposal_id: u64,
            duoqian_address: T::AccountId,
        },
        /// 关闭多签账户提案已发起
        CloseDuoqianProposed {
            proposal_id: u64,
            duoqian_address: T::AccountId,
            proposer: T::AccountId,
            beneficiary: T::AccountId,
        },
        /// 关闭多签投票已提交
        CloseVoteSubmitted {
            proposal_id: u64,
            who: T::AccountId,
            approve: bool,
        },
        /// 多签账户注销成功（投票通过，余额转出，DuoqianAccounts 删除）
        DuoqianClosed {
            proposal_id: u64,
            duoqian_address: T::AccountId,
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
            fee: BalanceOf<T>,
        },
        /// 关闭提案投票通过但执行失败
        CloseExecutionFailed {
            proposal_id: u64,
            duoqian_address: T::AccountId,
        },
        /// 个人多签账户创建提案已发起
        PersonalDuoqianProposed {
            proposal_id: u64,
            duoqian_address: T::AccountId,
            proposer: T::AccountId,
            name: SfidNameOf<T>,
            admin_count: u32,
            threshold: u32,
            amount: BalanceOf<T>,
        },
        /// SFID 机构登记
        SfidInstitutionRegistered {
            sfid_id: SfidIdOf<T>,
            name: SfidNameOf<T>,
            duoqian_address: T::AccountId,
            submitter: T::AccountId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 参数不完整
        IncompleteParameters,
        /// 地址非法
        InvalidAddress,
        /// 地址为制度保留地址，不允许注册
        AddressReserved,
        /// 地址已存在（已初始化）
        AddressAlreadyExists,
        /// 管理员重复
        DuplicateAdmin,
        /// 阈值不合法
        InvalidThreshold,
        /// 金额不足
        InsufficientAmount,
        /// 创建金额低于最小门槛
        CreateAmountBelowMinimum,
        /// 注销时账户余额低于最小门槛
        CloseBalanceBelowMinimum,
        /// 权限不足
        PermissionDenied,
        /// 管理员数量不合法（必须 >=2）
        InvalidAdminCount,
        /// 管理员数量与列表长度不一致
        AdminCountMismatch,
        /// 多签账户不存在
        DuoqianNotFound,
        /// 多签账户处于 pending 状态，不可操作
        DuoqianNotActive,
        /// 注销收款地址非法（不允许等于 duoqian_address）
        InvalidBeneficiary,
        /// 资金转出源地址受保护，不允许转出
        ProtectedSource,
        /// SFID机构未登记，不允许创建
        InstitutionNotRegistered,
        /// SFID 机构登记签名无效
        InvalidSfidInstitutionSignature,
        /// SFID ID 重复登记
        SfidAlreadyRegistered,
        /// SFID ID 为空
        EmptySfidId,
        /// 机构登记 nonce 已被使用
        RegisterNonceAlreadyUsed,
        /// 无法将派生地址转换为账户ID
        DerivedAddressDecodeFailed,
        /// 账户仍有保留余额，不允许注销
        ReservedBalanceRemaining,
        /// nonce 已耗尽
        NonceOverflow,
        /// runtime 配置不合法
        InvalidRuntimeConfig,
        /// 提案投票引擎错误
        VoteEngineError,
        /// 提案业务数据未找到
        ProposalActionNotFound,
        /// 转账失败
        TransferFailed,
        /// 管理员非本提案管理员
        UnauthorizedAdmin,
        /// 机构名称为空
        EmptySfidName,
        /// 手续费扣取失败
        FeeWithdrawFailed,
        /// 注销后转账金额低于 ED
        CloseTransferBelowED,
        /// 个人多签名称为空
        EmptyPersonalName,
        /// 个人多签地址已存在（同一 creator + name）
        PersonalDuoqianAlreadyExists,
        /// 该多签账户已有进行中的关闭提案，不允许重复发起
        CloseAlreadyPending,
        /// 提案未被拒绝，不可清理
        ProposalNotRejected,
    }

    /// 提案操作类型标记：存储在 ProposalData 的第一个字节
    pub const ACTION_CREATE: u8 = 1;
    pub const ACTION_CLOSE: u8 = 2;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // NOTE: `call_index` values are the on-chain ABI and must remain stable.

        /// 发起"创建多签账户"提案。
        /// - 预写入 DuoqianAccounts（pending 状态）；
        /// - 投票引擎创建提案，业务数据存入 ProposalData；
        /// - 投票通过后由 vote_create 自动执行入金 + 激活。
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::propose_create())]
        pub fn propose_create(
            origin: OriginFor<T>,
            sfid_id: SfidIdOf<T>,
            name: SfidNameOf<T>,
            admin_count: u32,
            duoqian_admins: DuoqianAdminsOf<T>,
            threshold: u32,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                !T::ProtectedSourceChecker::is_protected(&who),
                Error::<T>::ProtectedSource
            );

            ensure!(T::MaxAdmins::get() >= 2, Error::<T>::InvalidRuntimeConfig);
            ensure!(
                amount >= T::MinCreateAmount::get(),
                Error::<T>::CreateAmountBelowMinimum
            );
            // 预检查：proposer 余额需覆盖入金 + 手续费 + ED（保留自身账户存活）
            {
                let amount_u128: u128 = amount.saturated_into();
                let fee_u128 = onchain_transaction_pow::calculate_onchain_fee(amount_u128);
                let fee: BalanceOf<T> = fee_u128.saturated_into();
                let ed = T::Currency::minimum_balance();
                let required = amount
                    .checked_add(&fee)
                    .and_then(|v| v.checked_add(&ed))
                    .ok_or(Error::<T>::InsufficientAmount)?;
                ensure!(
                    T::Currency::free_balance(&who) >= required,
                    Error::<T>::InsufficientAmount
                );
            }
            ensure!(admin_count >= 2, Error::<T>::InvalidAdminCount);
            ensure!(
                duoqian_admins.len() as u32 == admin_count,
                Error::<T>::AdminCountMismatch
            );

            let min_threshold = core::cmp::max(2, admin_count.saturating_add(1) / 2);
            ensure!(
                threshold >= min_threshold && threshold <= admin_count,
                Error::<T>::InvalidThreshold
            );

            // 检查管理员去重
            Self::ensure_unique_admins(&duoqian_admins)?;

            // 发起人必须是管理员之一
            ensure!(
                duoqian_admins.iter().any(|admin| admin == &who),
                Error::<T>::PermissionDenied
            );

            // 解析 SFID 机构登记（sfid_id + name 双键查询）
            let duoqian_address = SfidRegisteredAddress::<T>::get(&sfid_id, &name)
                .ok_or(Error::<T>::InstitutionNotRegistered)?;
            let registered = AddressRegisteredSfid::<T>::get(&duoqian_address)
                .ok_or(Error::<T>::InstitutionNotRegistered)?;
            ensure!(
                registered.sfid_id == sfid_id,
                Error::<T>::InstitutionNotRegistered
            );

            ensure!(
                !T::ReservedAddressChecker::is_reserved(&duoqian_address),
                Error::<T>::AddressReserved
            );
            ensure!(
                T::AddressValidator::is_valid(&duoqian_address),
                Error::<T>::InvalidAddress
            );
            ensure!(
                !T::ProtectedSourceChecker::is_protected(&duoqian_address),
                Error::<T>::ProtectedSource
            );
            ensure!(
                !DuoqianAccounts::<T>::contains_key(&duoqian_address),
                Error::<T>::AddressAlreadyExists
            );

            let now = frame_system::Pallet::<T>::block_number();

            // 预写入 pending 状态的 DuoqianAccounts，使投票引擎可以从中读取阈值和管理员
            DuoqianAccounts::<T>::insert(
                &duoqian_address,
                DuoqianAccount {
                    admin_count,
                    threshold,
                    duoqian_admins: duoqian_admins.clone(),
                    creator: who.clone(),
                    created_at: now,
                    status: DuoqianStatus::Pending,
                },
            );

            // 创建投票引擎提案
            let institution = account_to_institution_id(&duoqian_address);
            let org = voting_engine_system::internal_vote::ORG_DUOQIAN;
            let proposal_id =
                T::InternalVoteEngine::create_internal_proposal(who.clone(), org, institution)?;

            // 存储业务数据到投票引擎 ProposalData
            let action = CreateDuoqianAction::<T::AccountId, BalanceOf<T>> {
                duoqian_address: duoqian_address.clone(),
                proposer: who.clone(),
                admin_count,
                threshold,
                amount,
            };
            let mut data = sp_std::vec::Vec::from(crate::MODULE_TAG);
            data.push(ACTION_CREATE);
            data.extend_from_slice(&action.encode());
            voting_engine_system::Pallet::<T>::store_proposal_data(proposal_id, data)?;
            voting_engine_system::Pallet::<T>::store_proposal_meta(proposal_id, now);

            Self::deposit_event(Event::<T>::CreateDuoqianProposed {
                proposal_id,
                duoqian_address,
                proposer: who,
                admin_count,
                threshold,
                amount,
            });

            Ok(())
        }

        /// 对"创建多签账户"提案投票，达到阈值后自动执行入金并激活。
        #[pallet::call_index(3)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::vote_create())]
        pub fn vote_create(
            origin: OriginFor<T>,
            proposal_id: u64,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 读取提案数据（MODULE_TAG + ACTION_CREATE + payload）
            let raw = voting_engine_system::Pallet::<T>::get_proposal_data(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            let tag = crate::MODULE_TAG;
            ensure!(raw.len() > tag.len() && &raw[..tag.len()] == tag, Error::<T>::ProposalActionNotFound);
            ensure!(raw[tag.len()] == ACTION_CREATE, Error::<T>::ProposalActionNotFound);
            let action = CreateDuoqianAction::<T::AccountId, BalanceOf<T>>::decode(&mut &raw[tag.len() + 1..])
                .map_err(|_| Error::<T>::ProposalActionNotFound)?;

            // 校验管理员权限
            let institution = account_to_institution_id(&action.duoqian_address);
            ensure!(
                Self::is_duoqian_admin(institution, &who),
                Error::<T>::UnauthorizedAdmin
            );

            // 投票
            T::InternalVoteEngine::cast_internal_vote(who.clone(), proposal_id, approve)?;

            Self::deposit_event(Event::<T>::CreateVoteSubmitted {
                proposal_id,
                who,
                approve,
            });

            // 检查投票结果并执行或清理
            if let Some(proposal) = voting_engine_system::Pallet::<T>::proposals(proposal_id) {
                if proposal.status == STATUS_PASSED {
                    // 使用 with_transaction 保证 execute_create 内部的资金操作和状态变更原子性：
                    // 若 execute_create 中途失败（如余额不足），已执行的转账会随事务回滚。
                    let exec_result = with_transaction(|| {
                        match Self::execute_create(proposal_id, &action) {
                            Ok(()) => TransactionOutcome::Commit(Ok(())),
                            Err(e) => TransactionOutcome::Rollback(Err(e)),
                        }
                    });
                    if exec_result.is_err() {
                        // 执行失败后清理 propose_create/propose_create_personal 写入的 pending 记录，
                        // 释放地址锁定，防止地址被永久占用。
                        DuoqianAccounts::<T>::remove(&action.duoqian_address);
                        PersonalDuoqianInfo::<T>::remove(&action.duoqian_address);
                        Self::deposit_event(Event::<T>::CreateExecutionFailed {
                            proposal_id,
                            duoqian_address: action.duoqian_address,
                        });
                    }
                } else if proposal.status == STATUS_REJECTED {
                    // 提案被拒绝：清理 Pending 条目，释放地址锁定。
                    DuoqianAccounts::<T>::remove(&action.duoqian_address);
                    PersonalDuoqianInfo::<T>::remove(&action.duoqian_address);
                }
            }

            Ok(())
        }

        /// 中文注释：机构登记改为 proof 模式；任意提交者都可代发，但链上只信任 SFID MAIN 签出的字段包。
        #[pallet::call_index(2)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::register_sfid_institution())]
        pub fn register_sfid_institution(
            origin: OriginFor<T>,
            sfid_id: SfidIdOf<T>,
            name: SfidNameOf<T>,
            register_nonce: RegisterNonceOf<T>,
            signature: RegisterSignatureOf<T>,
        ) -> DispatchResult {
            let submitter = ensure_signed(origin)?;
            ensure!(!sfid_id.is_empty(), Error::<T>::EmptySfidId);
            ensure!(!name.is_empty(), Error::<T>::EmptySfidName);
            let register_nonce_hash = T::Hashing::hash(register_nonce.as_slice());
            ensure!(
                !UsedRegisterNonce::<T>::get(register_nonce_hash),
                Error::<T>::RegisterNonceAlreadyUsed
            );
            ensure!(
                T::SfidInstitutionVerifier::verify_institution_registration(
                    sfid_id.as_slice(),
                    &name,
                    &register_nonce,
                    &signature,
                ),
                Error::<T>::InvalidSfidInstitutionSignature
            );
            ensure!(
                !SfidRegisteredAddress::<T>::contains_key(&sfid_id, &name),
                Error::<T>::SfidAlreadyRegistered
            );

            let duoqian_address = Self::derive_duoqian_address_from_sfid_id(sfid_id.as_slice(), name.as_slice())?;
            ensure!(
                !AddressRegisteredSfid::<T>::contains_key(&duoqian_address),
                Error::<T>::AddressAlreadyExists
            );
            ensure!(
                !T::ReservedAddressChecker::is_reserved(&duoqian_address),
                Error::<T>::AddressReserved
            );
            ensure!(
                T::AddressValidator::is_valid(&duoqian_address),
                Error::<T>::InvalidAddress
            );
            ensure!(
                !T::ProtectedSourceChecker::is_protected(&duoqian_address),
                Error::<T>::ProtectedSource
            );

            SfidRegisteredAddress::<T>::insert(&sfid_id, &name, &duoqian_address);
            UsedRegisterNonce::<T>::insert(register_nonce_hash, true);
            AddressRegisteredSfid::<T>::insert(
                &duoqian_address,
                RegisteredInstitution {
                    sfid_id: sfid_id.clone(),
                    name: name.clone(),
                },
            );
            Self::deposit_event(Event::<T>::SfidInstitutionRegistered {
                sfid_id,
                name,
                duoqian_address,
                submitter,
            });
            Ok(())
        }

        /// 发起"关闭多签账户"提案。
        #[pallet::call_index(1)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::propose_close())]
        pub fn propose_close(
            origin: OriginFor<T>,
            duoqian_address: T::AccountId,
            beneficiary: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                !T::ProtectedSourceChecker::is_protected(&duoqian_address),
                Error::<T>::ProtectedSource
            );
            ensure!(
                T::InstitutionAssetGuard::can_spend(
                    &duoqian_address,
                    InstitutionAssetAction::DuoqianCloseExecute,
                ),
                Error::<T>::ProtectedSource
            );
            ensure!(
                beneficiary != duoqian_address,
                Error::<T>::InvalidBeneficiary
            );
            ensure!(
                !T::ReservedAddressChecker::is_reserved(&beneficiary),
                Error::<T>::InvalidBeneficiary
            );
            ensure!(
                T::AddressValidator::is_valid(&beneficiary),
                Error::<T>::InvalidAddress
            );
            ensure!(
                !T::ProtectedSourceChecker::is_protected(&beneficiary),
                Error::<T>::InvalidBeneficiary
            );

            let account =
                DuoqianAccounts::<T>::get(&duoqian_address).ok_or(Error::<T>::DuoqianNotFound)?;
            ensure!(
                account.status == DuoqianStatus::Active,
                Error::<T>::DuoqianNotActive
            );

            // 发起人必须是管理员之一
            ensure!(
                account.duoqian_admins.iter().any(|admin| admin == &who),
                Error::<T>::PermissionDenied
            );

            // 拒绝对同一多签账户发起并发注销提案
            ensure!(
                !PendingCloseProposal::<T>::contains_key(&duoqian_address),
                Error::<T>::CloseAlreadyPending
            );

            let all_balance = T::Currency::free_balance(&duoqian_address);
            ensure!(
                all_balance >= T::MinCloseBalance::get(),
                Error::<T>::CloseBalanceBelowMinimum
            );
            // 预检查：扣除手续费后转给 beneficiary 的金额需 >= ED
            {
                let balance_u128: u128 = all_balance.saturated_into();
                let fee_u128 = onchain_transaction_pow::calculate_onchain_fee(balance_u128);
                let fee: BalanceOf<T> = fee_u128.saturated_into();
                let transfer_amount = all_balance
                    .checked_sub(&fee)
                    .ok_or(Error::<T>::FeeWithdrawFailed)?;
                let ed = T::Currency::minimum_balance();
                ensure!(transfer_amount >= ed, Error::<T>::CloseTransferBelowED);
            }
            ensure!(
                T::Currency::reserved_balance(&duoqian_address).is_zero(),
                Error::<T>::ReservedBalanceRemaining
            );

            // 创建投票引擎提案
            let institution = account_to_institution_id(&duoqian_address);
            let org = voting_engine_system::internal_vote::ORG_DUOQIAN;
            let proposal_id =
                T::InternalVoteEngine::create_internal_proposal(who.clone(), org, institution)?;

            // 存储业务数据
            let action = CloseDuoqianAction {
                duoqian_address: duoqian_address.clone(),
                beneficiary: beneficiary.clone(),
                proposer: who.clone(),
            };
            let mut data = sp_std::vec::Vec::from(crate::MODULE_TAG);
            data.push(ACTION_CLOSE);
            data.extend_from_slice(&action.encode());
            voting_engine_system::Pallet::<T>::store_proposal_data(proposal_id, data)?;
            voting_engine_system::Pallet::<T>::store_proposal_meta(
                proposal_id,
                frame_system::Pallet::<T>::block_number(),
            );
            PendingCloseProposal::<T>::insert(&duoqian_address, proposal_id);

            Self::deposit_event(Event::<T>::CloseDuoqianProposed {
                proposal_id,
                duoqian_address,
                proposer: who,
                beneficiary,
            });

            Ok(())
        }

        /// 对"关闭多签账户"提案投票，达到阈值后自动执行关闭。
        #[pallet::call_index(4)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::vote_close())]
        pub fn vote_close(origin: OriginFor<T>, proposal_id: u64, approve: bool) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 读取提案数据（MODULE_TAG + ACTION_CLOSE + payload）
            let raw = voting_engine_system::Pallet::<T>::get_proposal_data(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            let tag = crate::MODULE_TAG;
            ensure!(raw.len() > tag.len() && &raw[..tag.len()] == tag, Error::<T>::ProposalActionNotFound);
            ensure!(raw[tag.len()] == ACTION_CLOSE, Error::<T>::ProposalActionNotFound);
            let action = CloseDuoqianAction::<T::AccountId>::decode(&mut &raw[tag.len() + 1..])
                .map_err(|_| Error::<T>::ProposalActionNotFound)?;

            // 校验管理员权限
            let institution = account_to_institution_id(&action.duoqian_address);
            ensure!(
                Self::is_duoqian_admin(institution, &who),
                Error::<T>::UnauthorizedAdmin
            );

            // 投票
            T::InternalVoteEngine::cast_internal_vote(who.clone(), proposal_id, approve)?;

            Self::deposit_event(Event::<T>::CloseVoteSubmitted {
                proposal_id,
                who,
                approve,
            });

            // 检查投票结果并执行或清理
            if let Some(proposal) = voting_engine_system::Pallet::<T>::proposals(proposal_id) {
                if proposal.status == STATUS_PASSED {
                    // 使用 with_transaction 保证 execute_close 内部的资金操作原子性：
                    // 若扣费或转出中途失败，已执行的操作会随事务回滚。
                    let exec_result = with_transaction(|| {
                        match Self::execute_close(proposal_id, &action) {
                            Ok(()) => TransactionOutcome::Commit(Ok(())),
                            Err(e) => TransactionOutcome::Rollback(Err(e)),
                        }
                    });
                    if exec_result.is_err() {
                        // 执行失败后清除活跃关闭提案记录，允许重新发起关闭提案（账户仍为 Active）。
                        PendingCloseProposal::<T>::remove(&action.duoqian_address);
                        Self::deposit_event(Event::<T>::CloseExecutionFailed {
                            proposal_id,
                            duoqian_address: action.duoqian_address,
                        });
                    }
                } else if proposal.status == STATUS_REJECTED {
                    // 提案被拒绝：清理 PendingCloseProposal，允许重新发起关闭。
                    PendingCloseProposal::<T>::remove(&action.duoqian_address);
                }
            }

            Ok(())
        }

        /// 发起"创建个人多签账户"提案（无需 SFID 注册）。
        ///
        /// 地址由 `creator + name` 派生：
        /// `Blake2b_256("DUOQIAN_PERSONAL_V1" || SS58_PREFIX_LE || creator.encode() || name_utf8)`
        ///
        /// 投票通过后由 vote_create 自动执行入金 + 激活（复用 execute_create）。
        #[pallet::call_index(5)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::propose_create_personal())]
        pub fn propose_create_personal(
            origin: OriginFor<T>,
            name: SfidNameOf<T>,
            admin_count: u32,
            duoqian_admins: DuoqianAdminsOf<T>,
            threshold: u32,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                !T::ProtectedSourceChecker::is_protected(&who),
                Error::<T>::ProtectedSource
            );

            ensure!(!name.is_empty(), Error::<T>::EmptyPersonalName);

            ensure!(T::MaxAdmins::get() >= 2, Error::<T>::InvalidRuntimeConfig);
            ensure!(
                amount >= T::MinCreateAmount::get(),
                Error::<T>::CreateAmountBelowMinimum
            );
            // 预检查余额
            {
                let amount_u128: u128 = amount.saturated_into();
                let fee_u128 = onchain_transaction_pow::calculate_onchain_fee(amount_u128);
                let fee: BalanceOf<T> = fee_u128.saturated_into();
                let ed = T::Currency::minimum_balance();
                let required = amount
                    .checked_add(&fee)
                    .and_then(|v| v.checked_add(&ed))
                    .ok_or(Error::<T>::InsufficientAmount)?;
                ensure!(
                    T::Currency::free_balance(&who) >= required,
                    Error::<T>::InsufficientAmount
                );
            }

            ensure!(admin_count >= 2, Error::<T>::InvalidAdminCount);
            ensure!(
                duoqian_admins.len() as u32 == admin_count,
                Error::<T>::AdminCountMismatch
            );

            let min_threshold = core::cmp::max(2, admin_count.saturating_add(1) / 2);
            ensure!(
                threshold >= min_threshold && threshold <= admin_count,
                Error::<T>::InvalidThreshold
            );

            Self::ensure_unique_admins(&duoqian_admins)?;
            ensure!(
                duoqian_admins.iter().any(|a| a == &who),
                Error::<T>::PermissionDenied
            );

            // 派生地址
            let duoqian_address =
                Self::derive_personal_duoqian_address(&who, name.as_slice())?;
            ensure!(
                !DuoqianAccounts::<T>::contains_key(&duoqian_address),
                Error::<T>::PersonalDuoqianAlreadyExists
            );
            ensure!(
                !T::ReservedAddressChecker::is_reserved(&duoqian_address),
                Error::<T>::AddressReserved
            );
            ensure!(
                T::AddressValidator::is_valid(&duoqian_address),
                Error::<T>::InvalidAddress
            );
            ensure!(
                !T::ProtectedSourceChecker::is_protected(&duoqian_address),
                Error::<T>::ProtectedSource
            );

            // 预写入 DuoqianAccounts（pending 状态）
            let now = frame_system::Pallet::<T>::block_number();
            DuoqianAccounts::<T>::insert(
                &duoqian_address,
                DuoqianAccount {
                    admin_count,
                    threshold,
                    duoqian_admins: duoqian_admins.clone(),
                    creator: who.clone(),
                    created_at: now,
                    status: DuoqianStatus::Pending,
                },
            );

            // 写入个人多签元数据
            PersonalDuoqianInfo::<T>::insert(
                &duoqian_address,
                PersonalDuoqianMeta {
                    creator: who.clone(),
                    name: name.clone(),
                },
            );

            // 创建投票引擎提案
            let institution = account_to_institution_id(&duoqian_address);
            let org = voting_engine_system::internal_vote::ORG_DUOQIAN;
            let proposal_id =
                T::InternalVoteEngine::create_internal_proposal(who.clone(), org, institution)?;

            // 存储业务数据（复用 ACTION_CREATE + CreateDuoqianAction）
            let action = CreateDuoqianAction {
                duoqian_address: duoqian_address.clone(),
                proposer: who.clone(),
                admin_count,
                threshold,
                amount,
            };
            let mut data = sp_std::vec::Vec::from(crate::MODULE_TAG);
            data.push(ACTION_CREATE);
            data.extend_from_slice(&action.encode());
            voting_engine_system::Pallet::<T>::store_proposal_data(proposal_id, data)?;
            voting_engine_system::Pallet::<T>::store_proposal_meta(proposal_id, now);

            Self::deposit_event(Event::<T>::PersonalDuoqianProposed {
                proposal_id,
                duoqian_address,
                proposer: who,
                name,
                admin_count,
                threshold,
                amount,
            });

            Ok(())
        }

        /// 清理已被拒绝或超时的创建/关闭提案残留状态。
        /// 任意签名账户可调用。用于解决投票引擎 on_initialize 超时 reject 后
        /// 本模块无法自动收到通知导致的 Pending / PendingCloseProposal 残留。
        #[pallet::call_index(6)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::cleanup_rejected_proposal())]
        pub fn cleanup_rejected_proposal(
            origin: OriginFor<T>,
            proposal_id: u64,
        ) -> DispatchResult {
            let _ = ensure_signed(origin)?;

            // 读取提案数据，校验 MODULE_TAG 后判断操作类型
            let raw = voting_engine_system::Pallet::<T>::get_proposal_data(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            let tag = crate::MODULE_TAG;
            ensure!(raw.len() > tag.len() && &raw[..tag.len()] == tag, Error::<T>::ProposalActionNotFound);
            let action_tag = raw[tag.len()];

            // 校验投票引擎状态必须为 REJECTED
            let proposal = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            ensure!(
                proposal.status == STATUS_REJECTED,
                Error::<T>::ProposalNotRejected
            );

            match action_tag {
                ACTION_CREATE => {
                    let action =
                        CreateDuoqianAction::<T::AccountId, BalanceOf<T>>::decode(&mut &raw[tag.len() + 1..])
                            .map_err(|_| Error::<T>::ProposalActionNotFound)?;
                    DuoqianAccounts::<T>::remove(&action.duoqian_address);
                    PersonalDuoqianInfo::<T>::remove(&action.duoqian_address);
                }
                ACTION_CLOSE => {
                    let action = CloseDuoqianAction::<T::AccountId>::decode(&mut &raw[tag.len() + 1..])
                        .map_err(|_| Error::<T>::ProposalActionNotFound)?;
                    PendingCloseProposal::<T>::remove(&action.duoqian_address);
                }
                _ => return Err(Error::<T>::ProposalActionNotFound.into()),
            }

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// 返回链域前缀（SS58 前缀的小端 u16 字节）
        fn chain_domain_prefix() -> [u8; 2] {
            T::SS58Prefix::get().to_le_bytes()
        }

        /// 从 sfid_id（+ 可选 name）派生 duoqian 地址。
        /// name 非空时参与派生（注册机构多签），为空时不参与（治理机构多签），保持向后兼容。
        pub fn derive_duoqian_address_from_sfid_id(
            sfid_id: &[u8],
            name: &[u8],
        ) -> Result<T::AccountId, DispatchError> {
            let mut input = b"DUOQIAN_SFID_V1".to_vec();
            input.extend_from_slice(&Self::chain_domain_prefix());
            input.extend_from_slice(sfid_id);
            if !name.is_empty() {
                input.extend_from_slice(name);
            }
            let digest = sp_runtime::traits::BlakeTwo256::hash(input.as_slice());
            T::AccountId::decode(&mut digest.as_ref())
                .map_err(|_| Error::<T>::DerivedAddressDecodeFailed.into())
        }

        /// 从 creator + name 派生个人多签地址。
        pub fn derive_personal_duoqian_address(
            creator: &T::AccountId,
            name: &[u8],
        ) -> Result<T::AccountId, DispatchError> {
            let mut input = b"DUOQIAN_PERSONAL_V1".to_vec();
            input.extend_from_slice(&Self::chain_domain_prefix());
            input.extend_from_slice(&creator.encode());
            input.extend_from_slice(name);
            let digest = sp_runtime::traits::BlakeTwo256::hash(input.as_slice());
            T::AccountId::decode(&mut digest.as_ref())
                .map_err(|_| Error::<T>::DerivedAddressDecodeFailed.into())
        }

        fn ensure_unique_admins(admins: &DuoqianAdminsOf<T>) -> Result<(), DispatchError> {
            let mut seen = BTreeSet::new();
            for admin in admins.iter() {
                ensure!(seen.insert(admin.clone()), Error::<T>::DuplicateAdmin);
            }
            Ok(())
        }

        /// 检查 who 是否是某个 duoqian 机构的管理员
        fn is_duoqian_admin(institution: InstitutionPalletId, who: &T::AccountId) -> bool {
            <T as voting_engine_system::Config>::InternalAdminProvider::is_internal_admin(
                voting_engine_system::internal_vote::ORG_DUOQIAN,
                institution,
                who,
            )
        }

        /// 执行创建：入金 + 激活 DuoqianAccounts + 更新 nonce
        fn execute_create(
            proposal_id: u64,
            action: &CreateDuoqianAction<T::AccountId, BalanceOf<T>>,
        ) -> DispatchResult {
            // 计算手续费（复用 onchain-transaction-pow 公共费率）
            let amount_u128: u128 = action.amount.saturated_into();
            let fee_u128 = onchain_transaction_pow::calculate_onchain_fee(amount_u128);
            let fee: BalanceOf<T> = fee_u128.saturated_into();

            // 入金：从提案发起人转入 duoqian_address
            T::Currency::transfer(
                &action.proposer,
                &action.duoqian_address,
                action.amount,
                ExistenceRequirement::KeepAlive,
            )
            .map_err(|_| Error::<T>::TransferFailed)?;

            // 手续费：从 proposer 额外扣取，通过 FeeRouter 分账
            if !fee.is_zero() {
                let fee_imbalance = T::Currency::withdraw(
                    &action.proposer,
                    fee,
                    frame_support::traits::WithdrawReasons::FEE,
                    ExistenceRequirement::KeepAlive,
                )
                .map_err(|_| Error::<T>::FeeWithdrawFailed)?;
                T::FeeRouter::on_unbalanced(fee_imbalance);
            }

            // 激活 DuoqianAccounts
            DuoqianAccounts::<T>::mutate(&action.duoqian_address, |maybe_account| {
                if let Some(account) = maybe_account {
                    account.status = DuoqianStatus::Active;
                }
            });


            Self::deposit_event(Event::<T>::DuoqianCreated {
                proposal_id,
                duoqian_address: action.duoqian_address.clone(),
                creator: action.proposer.clone(),
                admin_count: action.admin_count,
                threshold: action.threshold,
                amount: action.amount,
                fee,
            });

            // 标记为已执行，防止双重执行
            voting_engine_system::Pallet::<T>::set_status_and_emit(proposal_id, STATUS_EXECUTED)?;

            Ok(())
        }

        /// 执行关闭：转出余额 + 删除 DuoqianAccounts + 更新 nonce
        fn execute_close(
            proposal_id: u64,
            action: &CloseDuoqianAction<T::AccountId>,
        ) -> DispatchResult {
            ensure!(
                T::InstitutionAssetGuard::can_spend(
                    &action.duoqian_address,
                    InstitutionAssetAction::DuoqianCloseExecute,
                ),
                Error::<T>::ProtectedSource
            );
            let all_balance = T::Currency::free_balance(&action.duoqian_address);

            // 计算手续费
            let balance_u128: u128 = all_balance.saturated_into();
            let fee_u128 = onchain_transaction_pow::calculate_onchain_fee(balance_u128);
            let fee: BalanceOf<T> = fee_u128.saturated_into();
            let transfer_amount = all_balance
                .checked_sub(&fee)
                .ok_or(Error::<T>::FeeWithdrawFailed)?;

            // 确保扣除手续费后转给 beneficiary 的金额 >= ED
            let ed = T::Currency::minimum_balance();
            ensure!(transfer_amount >= ed, Error::<T>::CloseTransferBelowED);

            // 先扣手续费
            if !fee.is_zero() {
                let fee_imbalance = T::Currency::withdraw(
                    &action.duoqian_address,
                    fee,
                    frame_support::traits::WithdrawReasons::FEE,
                    ExistenceRequirement::AllowDeath,
                )
                .map_err(|_| Error::<T>::FeeWithdrawFailed)?;
                T::FeeRouter::on_unbalanced(fee_imbalance);
            }

            // 转出剩余余额
            T::Currency::transfer(
                &action.duoqian_address,
                &action.beneficiary,
                transfer_amount,
                ExistenceRequirement::AllowDeath,
            )
            .map_err(|_| Error::<T>::TransferFailed)?;

            DuoqianAccounts::<T>::remove(&action.duoqian_address);
            // 清理个人多签元数据（机构多签无此条目，remove 为 no-op）。
            PersonalDuoqianInfo::<T>::remove(&action.duoqian_address);
            // 清除活跃关闭提案记录。
            PendingCloseProposal::<T>::remove(&action.duoqian_address);

            Self::deposit_event(Event::<T>::DuoqianClosed {
                proposal_id,
                duoqian_address: action.duoqian_address.clone(),
                beneficiary: action.beneficiary.clone(),
                amount: transfer_amount,
                fee,
            });

            // 标记为已执行，防止双重执行
            voting_engine_system::Pallet::<T>::set_status_and_emit(proposal_id, STATUS_EXECUTED)?;

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{
        assert_noop, assert_ok, derive_impl,
        traits::{ConstU128, ConstU32, VariantCountOf},
    };
    use frame_system as system;
    use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};
    use voting_engine_system::internal_vote::ORG_DUOQIAN;

    type Block = frame_system::mocking::MockBlock<Test>;
    type Balance = u128;

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
        pub type Duoqian = pallet;
    }

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl system::Config for Test {
        type Block = Block;
        type AccountId = AccountId32;
        type Lookup = IdentityLookup<Self::AccountId>;
        type AccountData = pallet_balances::AccountData<Balance>;
        type Nonce = u64;
    }

    impl pallet_balances::Config for Test {
        type MaxLocks = ConstU32<0>;
        type MaxReserves = ConstU32<1>;
        type ReserveIdentifier = [u8; 8];
        type Balance = Balance;
        type RuntimeEvent = RuntimeEvent;
        type DustRemoval = ();
        type ExistentialDeposit = ConstU128<1>;
        type AccountStore = System;
        type WeightInfo = ();
        type FreezeIdentifier = RuntimeFreezeReason;
        type MaxFreezes = VariantCountOf<RuntimeFreezeReason>;
        type RuntimeHoldReason = RuntimeHoldReason;
        type RuntimeFreezeReason = RuntimeFreezeReason;
        type DoneSlashHandler = ();
    }

    pub struct TestAddressValidator;
    impl DuoqianAddressValidator<AccountId32> for TestAddressValidator {
        fn is_valid(address: &AccountId32) -> bool {
            address != &AccountId32::new([0u8; 32])
        }
    }

    pub struct TestReservedAddressChecker;
    impl DuoqianReservedAddressChecker<AccountId32> for TestReservedAddressChecker {
        fn is_reserved(address: &AccountId32) -> bool {
            *address == AccountId32::new([0xAA; 32])
        }
    }

    pub struct TestSfidInstitutionVerifier;
    impl SfidInstitutionVerifier<SfidNameOf<Test>, RegisterNonceOf<Test>, RegisterSignatureOf<Test>>
        for TestSfidInstitutionVerifier
    {
        fn verify_institution_registration(
            _sfid_id: &[u8],
            _name: &SfidNameOf<Test>,
            nonce: &RegisterNonceOf<Test>,
            signature: &RegisterSignatureOf<Test>,
        ) -> bool {
            !nonce.is_empty() && signature.as_slice() == b"register-ok"
        }
    }

    pub struct TestProtectedSourceChecker;
    impl ProtectedSourceChecker<AccountId32> for TestProtectedSourceChecker {
        fn is_protected(address: &AccountId32) -> bool {
            *address == AccountId32::new([0xCC; 32])
        }
    }

    thread_local! {
        static DENIED_CLOSE_SOURCE: core::cell::RefCell<Option<AccountId32>> = core::cell::RefCell::new(None);
    }

    pub struct TestInstitutionAssetGuard;
    impl institution_asset_guard::InstitutionAssetGuard<AccountId32> for TestInstitutionAssetGuard {
        fn can_spend(
            source: &AccountId32,
            action: institution_asset_guard::InstitutionAssetAction,
        ) -> bool {
            if !matches!(
                action,
                institution_asset_guard::InstitutionAssetAction::DuoqianCloseExecute
            ) {
                return true;
            }
            DENIED_CLOSE_SOURCE.with(|blocked| blocked.borrow().as_ref() != Some(source))
        }
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

    /// 测试用 InternalAdminProvider：从 DuoqianAccounts 读取管理员
    pub struct TestInternalAdminProvider;
    impl voting_engine_system::InternalAdminProvider<AccountId32> for TestInternalAdminProvider {
        fn is_internal_admin(org: u8, institution: InstitutionPalletId, who: &AccountId32) -> bool {
            if org != ORG_DUOQIAN {
                return false;
            }
            let account = AccountId32::decode(&mut &institution[..32]);
            let Ok(account) = account else {
                return false;
            };
            if let Some(duoqian) = DuoqianAccounts::<Test>::get(&account) {
                duoqian.duoqian_admins.iter().any(|admin| admin == who)
            } else {
                false
            }
        }

        fn get_admin_list(
            org: u8,
            institution: InstitutionPalletId,
        ) -> Option<Vec<AccountId32>> {
            if org != ORG_DUOQIAN {
                return None;
            }
            let account = AccountId32::decode(&mut &institution[..32]).ok()?;
            let duoqian = DuoqianAccounts::<Test>::get(&account)?;
            Some(duoqian.duoqian_admins.into_inner())
        }
    }

    pub struct TestInternalAdminCountProvider;
    impl voting_engine_system::InternalAdminCountProvider for TestInternalAdminCountProvider {
        fn admin_count(org: u8, institution: InstitutionPalletId) -> Option<u32> {
            if org != ORG_DUOQIAN {
                return None;
            }
            let account = AccountId32::decode(&mut &institution[..32]).ok()?;
            let duoqian = DuoqianAccounts::<Test>::get(&account)?;
            u32::try_from(duoqian.duoqian_admins.len()).ok()
        }
    }

    /// 测试用 InternalThresholdProvider：从 DuoqianAccounts 读取阈值
    pub struct TestInternalThresholdProvider;
    impl voting_engine_system::InternalThresholdProvider for TestInternalThresholdProvider {
        fn pass_threshold(org: u8, institution: InstitutionPalletId) -> Option<u32> {
            if org != ORG_DUOQIAN {
                return voting_engine_system::internal_vote::governance_org_pass_threshold(org);
            }
            let account = AccountId32::decode(&mut &institution[..32]).ok()?;
            let duoqian = DuoqianAccounts::<Test>::get(&account)?;
            Some(duoqian.threshold)
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
        type SfidEligibility = TestSfidEligibility;
        type PopulationSnapshotVerifier = TestPopulationSnapshotVerifier;
        type JointVoteResultCallback = ();
        type InternalAdminProvider = TestInternalAdminProvider;
        type InternalAdminCountProvider = TestInternalAdminCountProvider;
        type InternalThresholdProvider = TestInternalThresholdProvider;
        type MaxAdminsPerInstitution = ConstU32<64>;
        type MaxProposalDataLen = ConstU32<4096>;
        type MaxProposalObjectLen = ConstU32<{ 10 * 1024 }>;
        type TimeProvider = TestTimeProvider;
        type WeightInfo = ();
    }

    impl pallet::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type Currency = Balances;
        type InternalVoteEngine = voting_engine_system::Pallet<Test>;
        type AddressValidator = TestAddressValidator;
        type ReservedAddressChecker = TestReservedAddressChecker;
        type ProtectedSourceChecker = TestProtectedSourceChecker;
        type InstitutionAssetGuard = TestInstitutionAssetGuard;
        type SfidInstitutionVerifier = TestSfidInstitutionVerifier;
        type FeeRouter = ();
        type MaxAdmins = ConstU32<10>;
        type MaxSfidIdLength = ConstU32<96>;
        type MaxSfidNameLength = ConstU32<128>;
        type MaxRegisterNonceLength = ConstU32<64>;
        type MaxRegisterSignatureLength = ConstU32<64>;
        type MinCreateAmount = ConstU128<111>;
        type MinCloseBalance = ConstU128<121>;
        type WeightInfo = ();
    }

    fn relayer() -> AccountId32 {
        AccountId32::new([0x55; 32])
    }

    fn admin(seed: u8) -> AccountId32 {
        AccountId32::new([seed; 32])
    }

    fn register_sfid_and_get_address(tag: &str) -> (SfidIdOf<Test>, SfidNameOf<Test>, AccountId32) {
        let sfid: SfidIdOf<Test> = format!("GFR-LN001-CB0C-{}-20260222", tag)
            .as_bytes()
            .to_vec()
            .try_into()
            .expect("sfid id should fit");
        let name: SfidNameOf<Test> = format!("Test Institution {}", tag)
            .as_bytes()
            .to_vec()
            .try_into()
            .expect("name should fit");
        let register_nonce: RegisterNonceOf<Test> = b"register-nonce"
            .to_vec()
            .try_into()
            .expect("register nonce should fit");
        let signature: RegisterSignatureOf<Test> = b"register-ok"
            .to_vec()
            .try_into()
            .expect("register signature should fit");
        assert_ok!(Duoqian::register_sfid_institution(
            RuntimeOrigin::signed(relayer()),
            sfid.clone(),
            name.clone(),
            register_nonce,
            signature,
        ));
        let duoqian_address = SfidRegisteredAddress::<Test>::get(&sfid, &name)
            .expect("sfid should be registered");
        (sfid, name, duoqian_address)
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let mut storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("system genesis build should succeed");

        pallet::GenesisConfig::<Test>::default()
            .assimilate_storage(&mut storage)
            .expect("duoqian genesis build should succeed");

        // 给管理员余额
        pallet_balances::GenesisConfig::<Test> {
            balances: vec![
                (admin(1), 100_000),
                (admin(2), 100_000),
                (admin(3), 100_000),
                (admin(4), 100_000),
            ],
            dev_accounts: None,
        }
        .assimilate_storage(&mut storage)
        .expect("balances genesis build should succeed");

        sp_io::TestExternalities::new(storage)
    }

    fn last_proposal_id() -> u64 {
        voting_engine_system::Pallet::<Test>::next_proposal_id().saturating_sub(1)
    }

    fn make_admins(seeds: &[u8]) -> DuoqianAdminsOf<Test> {
        seeds
            .iter()
            .map(|s| admin(*s))
            .collect::<Vec<_>>()
            .try_into()
            .expect("admins should fit")
    }

    #[test]
    fn register_sfid_works() {
        new_test_ext().execute_with(|| {
            let (sfid, name, duoqian_address) = register_sfid_and_get_address("A001");
            assert!(SfidRegisteredAddress::<Test>::contains_key(&sfid, &name));
            assert!(AddressRegisteredSfid::<Test>::contains_key(
                &duoqian_address
            ));
        });
    }

    #[test]
    fn register_sfid_rejects_invalid_signature() {
        new_test_ext().execute_with(|| {
            let sfid: SfidIdOf<Test> = b"GFR-LN001-CB0C-Z001-20260222"
                .to_vec()
                .try_into()
                .expect("sfid id should fit");
            let name: SfidNameOf<Test> = b"Bad Institution"
                .to_vec()
                .try_into()
                .expect("name should fit");
            let register_nonce: RegisterNonceOf<Test> = b"bad-register-nonce"
                .to_vec()
                .try_into()
                .expect("register nonce should fit");
            let bad_signature: RegisterSignatureOf<Test> = b"bad-signature"
                .to_vec()
                .try_into()
                .expect("register signature should fit");
            assert_noop!(
                Duoqian::register_sfid_institution(
                    RuntimeOrigin::signed(admin(1)),
                    sfid,
                    name,
                    register_nonce,
                    bad_signature,
                ),
                Error::<Test>::InvalidSfidInstitutionSignature
            );
        });
    }

    #[test]
    fn propose_create_and_vote_to_activate() {
        new_test_ext().execute_with(|| {
            let (sfid, name, duoqian_address) = register_sfid_and_get_address("B001");
            let admins = make_admins(&[1, 2, 3]);

            // 发起创建提案
            assert_ok!(Duoqian::propose_create(
                RuntimeOrigin::signed(admin(1)),
                sfid.clone(),
                name.clone(),
                3,
                admins.clone(),
                2,
                1_000,
            ));

            // DuoqianAccounts 已预写入 pending 状态
            let account = DuoqianAccounts::<Test>::get(&duoqian_address).expect("should exist");
            assert_eq!(account.status, DuoqianStatus::Pending);
            assert_eq!(account.threshold, 2);

            let pid = last_proposal_id();

            // 两个管理员投赞成票（阈值 2）
            assert_ok!(Duoqian::vote_create(
                RuntimeOrigin::signed(admin(1)),
                pid,
                true
            ));
            assert_ok!(Duoqian::vote_create(
                RuntimeOrigin::signed(admin(2)),
                pid,
                true
            ));

            // 投票通过后 DuoqianAccounts 应该变为 Active
            let account = DuoqianAccounts::<Test>::get(&duoqian_address).expect("should exist");
            assert_eq!(account.status, DuoqianStatus::Active);

            // 资金已转入
            assert_eq!(Balances::free_balance(&duoqian_address), 1_000);
        });
    }

    #[test]
    fn propose_close_and_vote_to_close() {
        new_test_ext().execute_with(|| {
            let (sfid, name, duoqian_address) = register_sfid_and_get_address("C001");
            let admins = make_admins(&[1, 2, 3]);
            let beneficiary = admin(4);

            // 先创建
            assert_ok!(Duoqian::propose_create(
                RuntimeOrigin::signed(admin(1)),
                sfid.clone(),
                name.clone(),
                3,
                admins.clone(),
                2,
                1_000,
            ));
            let create_pid = last_proposal_id();
            assert_ok!(Duoqian::vote_create(
                RuntimeOrigin::signed(admin(1)),
                create_pid,
                true
            ));
            assert_ok!(Duoqian::vote_create(
                RuntimeOrigin::signed(admin(2)),
                create_pid,
                true
            ));

            // 确认 active
            let account = DuoqianAccounts::<Test>::get(&duoqian_address).expect("should exist");
            assert_eq!(account.status, DuoqianStatus::Active);

            // 发起关闭提案
            assert_ok!(Duoqian::propose_close(
                RuntimeOrigin::signed(admin(1)),
                duoqian_address.clone(),
                beneficiary.clone(),
            ));

            let close_pid = last_proposal_id();

            // 投票关闭
            assert_ok!(Duoqian::vote_close(
                RuntimeOrigin::signed(admin(1)),
                close_pid,
                true
            ));
            assert_ok!(Duoqian::vote_close(
                RuntimeOrigin::signed(admin(2)),
                close_pid,
                true
            ));

            // DuoqianAccounts 应该被删除
            assert!(DuoqianAccounts::<Test>::get(&duoqian_address).is_none());

            // 受益人收到余额（扣除 0.1% 手续费，最低 10 分）
            // admin(4) 原有 100_000，多签余额 1_000，fee = max(1_000 * 0.1%, 10) = 10
            // 实收 = 1_000 - 10 = 990
            assert_eq!(Balances::free_balance(&beneficiary), 100_990);
        });
    }

    #[test]
    fn non_admin_cannot_propose_create() {
        new_test_ext().execute_with(|| {
            let (sfid, name, _) = register_sfid_and_get_address("D001");
            let admins = make_admins(&[1, 2, 3]);

            // admin(4) 不在管理员列表中
            assert_noop!(
                Duoqian::propose_create(
                    RuntimeOrigin::signed(admin(4)),
                    sfid.clone(),
                    name.clone(),
                    3,
                    admins,
                    2,
                    1_000,
                ),
                Error::<Test>::PermissionDenied
            );
        });
    }

    #[test]
    fn non_admin_cannot_vote() {
        new_test_ext().execute_with(|| {
            let (sfid, name, _) = register_sfid_and_get_address("E001");
            let admins = make_admins(&[1, 2, 3]);

            assert_ok!(Duoqian::propose_create(
                RuntimeOrigin::signed(admin(1)),
                sfid.clone(),
                name.clone(),
                3,
                admins,
                2,
                1_000,
            ));

            let pid = last_proposal_id();

            // admin(4) 不在管理员列表中
            assert_noop!(
                Duoqian::vote_create(RuntimeOrigin::signed(admin(4)), pid, true),
                Error::<Test>::UnauthorizedAdmin
            );
        });
    }

    #[test]
    fn cannot_close_pending_account() {
        new_test_ext().execute_with(|| {
            let (sfid, name, duoqian_address) = register_sfid_and_get_address("F001");
            let admins = make_admins(&[1, 2, 3]);

            // propose create 但不投票通过
            assert_ok!(Duoqian::propose_create(
                RuntimeOrigin::signed(admin(1)),
                sfid,
                name.clone(),
                3,
                admins,
                2,
                1_000,
            ));

            assert_noop!(
                Duoqian::propose_close(RuntimeOrigin::signed(admin(1)), duoqian_address, admin(4),),
                Error::<Test>::DuoqianNotActive
            );
        });
    }

    #[test]
    fn propose_close_is_blocked_when_institution_guard_denies_source() {
        new_test_ext().execute_with(|| {
            let (sfid, name, duoqian_address) = register_sfid_and_get_address("F002");
            let admins = make_admins(&[1, 2, 3]);

            assert_ok!(Duoqian::propose_create(
                RuntimeOrigin::signed(admin(1)),
                sfid,
                name.clone(),
                3,
                admins,
                2,
                1_000,
            ));
            let create_pid = last_proposal_id();
            assert_ok!(Duoqian::vote_create(
                RuntimeOrigin::signed(admin(1)),
                create_pid,
                true
            ));
            assert_ok!(Duoqian::vote_create(
                RuntimeOrigin::signed(admin(2)),
                create_pid,
                true
            ));

            DENIED_CLOSE_SOURCE
                .with(|blocked| *blocked.borrow_mut() = Some(duoqian_address.clone()));

            assert_noop!(
                Duoqian::propose_close(RuntimeOrigin::signed(admin(1)), duoqian_address, admin(4),),
                Error::<Test>::ProtectedSource
            );

            DENIED_CLOSE_SOURCE.with(|blocked| *blocked.borrow_mut() = None);
        });
    }

    #[test]
    fn duplicate_admin_is_rejected() {
        new_test_ext().execute_with(|| {
            let (sfid, name, _) = register_sfid_and_get_address("G001");
            let admins: DuoqianAdminsOf<Test> = vec![admin(1), admin(1), admin(2)]
                .try_into()
                .expect("should fit");

            assert_noop!(
                Duoqian::propose_create(
                    RuntimeOrigin::signed(admin(1)),
                    sfid,
                    name.clone(),
                    3,
                    admins,
                    2,
                    1_000,
                ),
                Error::<Test>::DuplicateAdmin
            );
        });
    }

    #[test]
    fn amount_below_minimum_is_rejected() {
        new_test_ext().execute_with(|| {
            let (sfid, name, _) = register_sfid_and_get_address("H001");
            let admins = make_admins(&[1, 2, 3]);

            assert_noop!(
                Duoqian::propose_create(
                    RuntimeOrigin::signed(admin(1)),
                    sfid,
                    name.clone(),
                    3,
                    admins,
                    2,
                    10, // below MinCreateAmount of 111
                ),
                Error::<Test>::CreateAmountBelowMinimum
            );
        });
    }

    // ──── 新增：针对审查修复的专项测试 ────

    /// 修复验证：同一多签账户不能并发发起两个关闭提案。
    #[test]
    fn duplicate_close_proposal_is_rejected() {
        new_test_ext().execute_with(|| {
            let (sfid, name, duoqian_address) = register_sfid_and_get_address("I001");
            let admins = make_admins(&[1, 2]);

            // 创建并激活
            assert_ok!(Duoqian::propose_create(
                RuntimeOrigin::signed(admin(1)),
                sfid,
                name.clone(),
                2,
                admins,
                2,
                1_000,
            ));
            let create_pid = last_proposal_id();
            assert_ok!(Duoqian::vote_create(RuntimeOrigin::signed(admin(1)), create_pid, true));
            assert_ok!(Duoqian::vote_create(RuntimeOrigin::signed(admin(2)), create_pid, true));

            let beneficiary = admin(3);

            // 第一个关闭提案 — 应该成功
            assert_ok!(Duoqian::propose_close(
                RuntimeOrigin::signed(admin(1)),
                duoqian_address.clone(),
                beneficiary.clone(),
            ));

            // 第二个关闭提案 — 应该被 CloseAlreadyPending 拒绝
            assert_noop!(
                Duoqian::propose_close(
                    RuntimeOrigin::signed(admin(2)),
                    duoqian_address.clone(),
                    beneficiary.clone(),
                ),
                Error::<Test>::CloseAlreadyPending
            );
        });
    }

    /// 修复验证：execute_create 失败后地址应被释放（Pending 条目清理）。
    #[test]
    fn execute_create_failure_releases_address() {
        new_test_ext().execute_with(|| {
            let (sfid, name, duoqian_address) = register_sfid_and_get_address("J001");
            let admins = make_admins(&[1, 2]);

            assert_ok!(Duoqian::propose_create(
                RuntimeOrigin::signed(admin(1)),
                sfid.clone(),
                name.clone(),
                2,
                admins.clone(),
                2,
                1_000,
            ));
            let pid = last_proposal_id();

            // 排干 admin(1) 的余额，使 execute_create 在 transfer 时失败
            let _ = Balances::slash(&admin(1), 99_900);
            assert!(Balances::free_balance(&admin(1)) < 1_010);

            // 最后一票触发 execute_create，因余额不足应失败
            assert_ok!(Duoqian::vote_create(RuntimeOrigin::signed(admin(1)), pid, true));
            assert_ok!(Duoqian::vote_create(RuntimeOrigin::signed(admin(2)), pid, true));

            // execute_create 失败后 DuoqianAccounts 中的 Pending 条目应被清除
            assert!(
                DuoqianAccounts::<Test>::get(&duoqian_address).is_none(),
                "pending entry must be cleaned up after execute_create failure"
            );

            // PersonalDuoqianInfo 也不应残留（机构多签无条目，remove 为 no-op）
            assert!(PersonalDuoqianInfo::<Test>::get(&duoqian_address).is_none());
        });
    }

    /// 修复验证：个人多签创建流程正常。
    #[test]
    fn personal_duoqian_create_works() {
        new_test_ext().execute_with(|| {
            let name: SfidNameOf<Test> = b"Family Fund"
                .to_vec()
                .try_into()
                .expect("name should fit");
            let admins = make_admins(&[1, 2]);

            assert_ok!(Duoqian::propose_create_personal(
                RuntimeOrigin::signed(admin(1)),
                name.clone(),
                2,
                admins,
                2,
                1_000,
            ));
            let pid = last_proposal_id();

            // 派生地址
            let duoqian_address =
                Pallet::<Test>::derive_personal_duoqian_address(&admin(1), name.as_slice())
                    .expect("derive should succeed");

            // 投票通过前处于 Pending
            assert_eq!(
                DuoqianAccounts::<Test>::get(&duoqian_address)
                    .map(|a| a.status),
                Some(DuoqianStatus::Pending)
            );

            assert_ok!(Duoqian::vote_create(RuntimeOrigin::signed(admin(1)), pid, true));
            assert_ok!(Duoqian::vote_create(RuntimeOrigin::signed(admin(2)), pid, true));

            // 投票通过后变为 Active，资金已转入
            let account =
                DuoqianAccounts::<Test>::get(&duoqian_address).expect("should exist");
            assert_eq!(account.status, DuoqianStatus::Active);
            assert_eq!(Balances::free_balance(&duoqian_address), 1_000);

            // PersonalDuoqianInfo 已写入
            let meta = PersonalDuoqianInfo::<Test>::get(&duoqian_address)
                .expect("personal info should exist");
            assert_eq!(meta.creator, admin(1));
            assert_eq!(meta.name, name);
        });
    }
}
