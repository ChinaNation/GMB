#![cfg_attr(not(feature = "std"), no_std)]

/// 模块标识前缀，用于在 ProposalData 中区分不同业务模块，防止跨模块误解码。
pub const MODULE_TAG: &[u8] = b"dq-mgmt";

pub use pallet::*;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod weights;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::storage::with_transaction;
use frame_support::{
    ensure,
    pallet_prelude::*,
    traits::{Currency, ExistenceRequirement, OnUnbalanced, ReservableCurrency},
    BoundedVec,
};
use frame_system::pallet_prelude::*;
use institution_asset_guard::{InstitutionAssetAction, InstitutionAssetGuard};
use scale_info::TypeInfo;
use sp_core::sr25519::{Public as Sr25519Public, Signature as Sr25519Signature};
use sp_runtime::{
    traits::{Hash, Zero},
    SaturatedConversion, TransactionOutcome,
};
use sp_std::{collections::btree_set::BTreeSet, prelude::*};
use voting_engine_system::{InstitutionPalletId, STATUS_EXECUTED, STATUS_PASSED, STATUS_REJECTED};

type BalanceOf<T> =
    <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// SFID 登记机构下的账户角色枚举，决定地址派生走哪个 op_tag：
/// - `Main`：所有机构的主账户，preimage 不含 account_name，走 `OP_MAIN = 0x00`。
/// - `Fee`：所有机构的费用账户，preimage 不含 account_name，走 `OP_FEE = 0x01`。
/// - `Named(account_name)`：SFID 机构自定义命名账户（临时 / 工资 / 运营等），走
///   `OP_INSTITUTION = 0x05`，account_name 非空且不得为保留名 `"主账户"`/`"费用账户"`。
///
/// 见 `primitives::core_const::{OP_MAIN, OP_FEE, OP_INSTITUTION}` 常量定义。
#[derive(Clone, Copy, Debug)]
pub enum InstitutionAccountRole<'a> {
    Main,
    Fee,
    Named(&'a [u8]),
}

/// 机构账户角色保留名：这两个中文字串必须强制走 Role::Main / Role::Fee，
/// 禁止被误当作 Named 命名账户落到 OP_INSTITUTION。
const RESERVED_NAME_MAIN: &[u8] = "主账户".as_bytes();
const RESERVED_NAME_FEE: &[u8] = "费用账户".as_bytes();

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

/// SFID 机构登记验签抽象：链上按省分流验签。
/// - `signing_province = Some(p)`：runtime 查该省的省级签名公钥（`ShengSigningPubkey[p]`）验签；
/// - `signing_province = None`：fallback 用 `SfidMainAccount` 当前主公钥验签（兼容旧调用）。
pub trait SfidInstitutionVerifier<AccountName, Nonce, Signature> {
    fn verify_institution_registration(
        sfid_id: &[u8],
        account_name: &AccountName,
        nonce: &Nonce,
        signature: &Signature,
        signing_province: Option<&[u8]>,
    ) -> bool;
}

impl<AccountName, Nonce, Signature> SfidInstitutionVerifier<AccountName, Nonce, Signature> for () {
    fn verify_institution_registration(
        _sfid_id: &[u8],
        _account_name: &AccountName,
        _nonce: &Nonce,
        _signature: &Signature,
        _signing_province: Option<&[u8]>,
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
pub struct RegisteredInstitution<SfidId, AccountName> {
    pub sfid_id: SfidId,
    pub account_name: AccountName,
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

/// 创建多签账户的离线管理员签名意图（Step 1 · 多签注册离线 QR 聚合版）。
///
/// 每个管理员在 wuminapp 扫描发起人导出的 QR 后,对此结构做 sr25519 签名,
/// 回传给发起人。发起人把 N 个 (admin, signature) 聚合后一笔 `finalize_create`
/// 代投,投票引擎内部自动达阈值 → `execute_create`。
///
/// 设计要点:
/// - `admins_root = blake2_256(SCALE.encode(sorted_admins))`:让签名消息体积固定,
///   QR 小;链上 finalize 时可重算等值校验。
/// - `approve` 恒为 true:拒绝的语义不通过离线签名表达(拒绝走"不签名超时自动 reject")。
/// - 所有字段都来自 Tx 1 的 `CreateDuoqianProposed` / `PersonalDuoqianProposed`
///   event,wuminapp 扫码后即可完整重建 intent。
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct CreateVoteIntent<AccountId, Balance> {
    /// 投票引擎分配的提案 ID(Tx 1 的 event 中返回)
    pub proposal_id: u64,
    /// Tx 1 中预写入的多签账户地址
    pub duoqian_address: AccountId,
    /// Tx 1 的发起人(= `CreateDuoqianAction.proposer`)
    pub creator: AccountId,
    /// `blake2_256(SCALE.encode(sorted_admins))`,let 签名消息体积固定
    pub admins_root: [u8; 32],
    /// Tx 1 预写入的阈值
    pub threshold: u32,
    /// Tx 1 预设的入金金额
    pub amount: Balance,
    /// 固定 true,占位防误签
    pub approve: bool,
}

impl<AccountId: Encode, Balance: Encode> CreateVoteIntent<AccountId, Balance> {
    /// 根据签名域铁律构造标准签名消息 hash。
    ///
    /// preimage = DUOQIAN_DOMAIN (10B) || OP_SIGN_CREATE (1B) || SS58_PREFIX_LE (2B)
    ///         || blake2_256(SCALE.encode(self))
    /// signing_hash = blake2_256(preimage)
    ///
    /// wuminapp 扫码后与本函数等价实现(Dart 端用相同 SCALE 布局 + 相同 domain/op_tag),
    /// 链上 finalize 用本函数得到的 32 字节 hash 作为 sr25519 签名的消息体。
    pub fn signing_hash(&self, ss58_prefix: u16) -> [u8; 32] {
        let intent_hash = sp_io::hashing::blake2_256(&self.encode());
        let mut preimage = Vec::with_capacity(10 + 1 + 2 + 32);
        preimage.extend_from_slice(primitives::core_const::DUOQIAN_DOMAIN);
        preimage.push(primitives::core_const::OP_SIGN_CREATE);
        preimage.extend_from_slice(&ss58_prefix.to_le_bytes());
        preimage.extend_from_slice(&intent_hash);
        sp_io::hashing::blake2_256(&preimage)
    }
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
pub struct PersonalDuoqianMeta<AccountId, AccountName> {
    pub creator: AccountId,
    pub account_name: AccountName,
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
            AccountNameOf<Self>,
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
        type MaxAccountNameLength: Get<u32>;

        #[pallet::constant]
        type MaxRegisterNonceLength: Get<u32>;

        #[pallet::constant]
        type MaxRegisterSignatureLength: Get<u32>;

        /// 管理员 sr25519 签名最大字节数(固定 64)。
        /// 用于 `finalize_create` 聚合签名时的 BoundedVec 容量上限,防止过大输入。
        #[pallet::constant]
        type MaxAdminSignatureLength: Get<u32>;

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
    pub type AccountNameOf<T> = BoundedVec<u8, <T as Config>::MaxAccountNameLength>;
    pub type RegisterNonceOf<T> = BoundedVec<u8, <T as Config>::MaxRegisterNonceLength>;
    pub type RegisterSignatureOf<T> = BoundedVec<u8, <T as Config>::MaxRegisterSignatureLength>;

    /// 管理员离线 sr25519 签名载体(固定 64 字节)。
    pub type AdminSignatureOf<T> = BoundedVec<u8, <T as Config>::MaxAdminSignatureLength>;
    /// finalize_create 聚合签名载荷:`Vec<(管理员地址, sr25519 签名)>`,
    /// 容量上限等于该多签允许的最多管理员数。
    pub type AdminSignaturesOf<T> = BoundedVec<
        (<T as frame_system::Config>::AccountId, AdminSignatureOf<T>),
        <T as Config>::MaxAdmins,
    >;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// 多签账户配置。key 为 duoqian_address。
    #[pallet::storage]
    #[pallet::getter(fn duoqian_account_of)]
    pub type DuoqianAccounts<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, DuoqianAccountOf<T>, OptionQuery>;

    /// SFID 机构登记：(sfid_id, account_name) -> duoqian_address（由 blake2b_256 派生）。
    /// 同一 sfid_id 可通过不同 account_name 注册多个多签地址。
    #[pallet::storage]
    pub type SfidRegisteredAddress<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        SfidIdOf<T>,
        Blake2_128Concat,
        AccountNameOf<T>,
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
        RegisteredInstitution<SfidIdOf<T>, AccountNameOf<T>>,
        OptionQuery,
    >;

    /// 已消费的机构登记 nonce，防止 proof 重放。
    #[pallet::storage]
    #[pallet::getter(fn used_register_nonce)]
    pub type UsedRegisterNonce<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, bool, ValueQuery>;

    /// 个人多签反向索引：duoqian_address -> { creator, account_name }
    #[pallet::storage]
    #[pallet::getter(fn personal_duoqian_info)]
    pub type PersonalDuoqianInfo<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        PersonalDuoqianMeta<T::AccountId, AccountNameOf<T>>,
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
        /// 机构多签账户创建提案已发起（Tx 1,pending 状态预写入）。
        /// wuminapp 扫描此事件后即可构造 `CreateVoteIntent` + QR,发给其他管理员离线签名。
        CreateDuoqianProposed {
            proposal_id: u64,
            duoqian_address: T::AccountId,
            proposer: T::AccountId,
            /// 关联的 SFID 机构标识
            sfid_id: SfidIdOf<T>,
            /// 关联的机构账户名("主账户"/"费用账户"/自定义)
            account_name: AccountNameOf<T>,
            /// 管理员完整列表(供 wuminapp 构建 QR 和 admins_root)
            admins: DuoqianAdminsOf<T>,
            admin_count: u32,
            threshold: u32,
            amount: BalanceOf<T>,
            /// 投票引擎分配的超时区块(投票期最后允许区块)
            expires_at: BlockNumberFor<T>,
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
        /// 创建提案最终被拒绝(投票引擎返回 STATUS_REJECTED 后清理 Pending)
        DuoqianCreateRejected {
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
        /// 个人多签账户创建提案已发起（Tx 1,无 SFID 归属）。
        PersonalDuoqianProposed {
            proposal_id: u64,
            duoqian_address: T::AccountId,
            proposer: T::AccountId,
            account_name: AccountNameOf<T>,
            /// 管理员完整列表(供 wuminapp 构建 QR 和 admins_root)
            admins: DuoqianAdminsOf<T>,
            admin_count: u32,
            threshold: u32,
            amount: BalanceOf<T>,
            /// 投票引擎分配的超时区块(投票期最后允许区块)
            expires_at: BlockNumberFor<T>,
        },
        /// finalize_create 代投完成(不论最终状态):统计接受的签名数 + 投票引擎返回状态。
        /// 便于链下观测 "N 签提交 → 投票引擎状态" 的一一对应。
        CreateFinalized {
            proposal_id: u64,
            /// 本次 finalize_create 接受并代投成功的签名数
            signatures_accepted: u32,
            /// 调用结束时投票引擎的提案状态
            /// (STATUS_PASSED / STATUS_REJECTED / STATUS_VOTING / STATUS_EXECUTED / STATUS_EXECUTION_FAILED)
            final_status: u8,
        },
        /// SFID 机构登记
        SfidInstitutionRegistered {
            sfid_id: SfidIdOf<T>,
            account_name: AccountNameOf<T>,
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
        EmptyAccountName,
        /// 手续费扣取失败
        FeeWithdrawFailed,
        /// 注销后转账金额低于 ED
        CloseTransferBelowED,
        /// 个人多签名称为空
        EmptyPersonalName,
        /// 个人多签地址已存在（同一 creator + account_name）
        PersonalDuoqianAlreadyExists,
        /// 该多签账户已有进行中的关闭提案，不允许重复发起
        CloseAlreadyPending,
        /// 提案未被拒绝，不可清理
        ProposalNotRejected,
        /// 账户名占用保留角色名（"主账户"/"费用账户" 必须走 Role::Main/Fee，
        /// 禁止作为 Role::Named 的自定义命名参数）
        ReservedAccountName,
        /// finalize_create 提交的签名对应的 admin 不在该多签的管理员列表
        UnauthorizedSignature,
        /// finalize_create 同一 admin 在同一批签名里重复出现
        DuplicateSignature,
        /// finalize_create sr25519 签名验证失败
        InvalidSignature,
        /// finalize_create 提交的签名数量少于阈值
        InsufficientSignatures,
        /// finalize_create sr25519 签名长度必须恰好为 64 字节
        MalformedSignature,
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
            account_name: AccountNameOf<T>,
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

            // 解析 SFID 机构登记（sfid_id + account_name 双键查询）
            let duoqian_address = SfidRegisteredAddress::<T>::get(&sfid_id, &account_name)
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

            // 从投票引擎回读提案超时区块,便于 wuminapp 倒计时。
            let expires_at = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .map(|p| p.end)
                .ok_or(Error::<T>::VoteEngineError)?;

            Self::deposit_event(Event::<T>::CreateDuoqianProposed {
                proposal_id,
                duoqian_address,
                proposer: who,
                sfid_id,
                account_name,
                admins: duoqian_admins,
                admin_count,
                threshold,
                amount,
                expires_at,
            });

            Ok(())
        }

        /// finalize_create:离线聚合管理员 sr25519 签名,一笔代投 + 自动激活。
        ///
        /// 替代原 `vote_create` 的"每人一笔在线投票"模式:
        /// - 发起人(Tx 1 中锁定的 `action.proposer`)把所有线下收集的
        ///   `(admin, sr25519_signature)` 装入 `sigs`,一次上链。
        /// - 本函数逐条:成员校验 → 去重 → sr25519 验签 → 代投 → 阈值自动判定。
        /// - 达阈值 `STATUS_PASSED` → 原子执行 `execute_create`(入金 + 激活)。
        /// - 被拒绝 `STATUS_REJECTED` → 清除 Pending 记录。
        ///
        /// 签名消息:参见 `CreateVoteIntent::signing_hash`(DUOQIAN_V1 + OP_SIGN_CREATE + ss58 + intent)。
        ///
        /// 语义要点:
        /// - **发起人不必是管理员**:Tx 1 已把 proposer 锁定,Tx 2 仅代投 + 代付 gas。
        /// - **幂等性**:投票引擎内部 `AlreadyVoted` 保护,重复提交会直接失败,不会导致重复入金。
        /// - **部分补签不支持**:一次提交必须 >= 阈值;否则 `InsufficientSignatures` 拒绝。
        #[pallet::call_index(3)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::finalize_create(sigs.len() as u32))]
        pub fn finalize_create(
            origin: OriginFor<T>,
            proposal_id: u64,
            sigs: AdminSignaturesOf<T>,
        ) -> DispatchResult {
            // 任意签名账户都可代投(支付 gas)。发起人身份已在 Tx 1 中锁定。
            let _submitter = ensure_signed(origin)?;

            // 1. 读取提案业务数据 (MODULE_TAG + ACTION_CREATE + SCALE)
            let raw = voting_engine_system::Pallet::<T>::get_proposal_data(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            let tag = crate::MODULE_TAG;
            ensure!(
                raw.len() > tag.len() && &raw[..tag.len()] == tag,
                Error::<T>::ProposalActionNotFound
            );
            ensure!(
                raw[tag.len()] == ACTION_CREATE,
                Error::<T>::ProposalActionNotFound
            );
            let action = CreateDuoqianAction::<T::AccountId, BalanceOf<T>>::decode(
                &mut &raw[tag.len() + 1..],
            )
            .map_err(|_| Error::<T>::ProposalActionNotFound)?;

            // 2. 读取 Pending 状态下的 DuoqianAccount:拿到管理员集合和阈值。
            let duoqian = DuoqianAccounts::<T>::get(&action.duoqian_address)
                .ok_or(Error::<T>::DuoqianNotFound)?;

            // 3. 签名数不能少于阈值
            let sigs_len_u32 = sigs.len() as u32;
            ensure!(
                sigs_len_u32 >= duoqian.threshold,
                Error::<T>::InsufficientSignatures
            );

            // 4. 构造签名消息 hash(链上与 wuminapp 端用同一公式)
            let admins_root = Self::compute_admins_root(&duoqian.duoqian_admins);
            let intent = CreateVoteIntent::<T::AccountId, BalanceOf<T>> {
                proposal_id,
                duoqian_address: action.duoqian_address.clone(),
                creator: action.proposer.clone(),
                admins_root,
                threshold: action.threshold,
                amount: action.amount,
                approve: true,
            };
            let signing_hash = intent.signing_hash(T::SS58Prefix::get());

            // 5. 循环验签 + 代投
            let mut seen: BTreeSet<T::AccountId> = BTreeSet::new();
            let mut accepted: u32 = 0;
            for (admin, sig_bytes) in sigs.iter() {
                // 5.1 必须是该多签的管理员之一
                ensure!(
                    duoqian.duoqian_admins.iter().any(|a| a == admin),
                    Error::<T>::UnauthorizedSignature
                );
                // 5.2 同批次内去重
                ensure!(
                    seen.insert(admin.clone()),
                    Error::<T>::DuplicateSignature
                );
                // 5.3 签名长度必须是 sr25519 的 64 字节
                ensure!(
                    sig_bytes.len() == 64,
                    Error::<T>::MalformedSignature
                );
                let sig = Sr25519Signature::try_from(sig_bytes.as_slice())
                    .map_err(|_| Error::<T>::MalformedSignature)?;
                let pubkey = Self::pubkey_from_accountid(admin)?;
                ensure!(
                    sp_io::crypto::sr25519_verify(&sig, &signing_hash, &pubkey),
                    Error::<T>::InvalidSignature
                );
                // 5.4 代投;投票引擎自己做"已投过"/"快照外"/"阈值"等所有检查
                T::InternalVoteEngine::cast_internal_vote(admin.clone(), proposal_id, true)?;
                accepted = accepted.saturating_add(1);
            }

            // 6. 根据投票引擎最终状态执行或清理
            let proposal = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;

            if proposal.status == STATUS_PASSED {
                // 事务内执行,失败随事务回滚资金操作
                let exec_result =
                    with_transaction(|| match Self::execute_create(proposal_id, &action) {
                        Ok(()) => TransactionOutcome::Commit(Ok(())),
                        Err(e) => TransactionOutcome::Rollback(Err(e)),
                    });
                if exec_result.is_err() {
                    // 执行失败:清除 Pending,释放地址锁定,防止永久占用。
                    DuoqianAccounts::<T>::remove(&action.duoqian_address);
                    PersonalDuoqianInfo::<T>::remove(&action.duoqian_address);
                    Self::deposit_event(Event::<T>::CreateExecutionFailed {
                        proposal_id,
                        duoqian_address: action.duoqian_address.clone(),
                    });
                }
            } else if proposal.status == STATUS_REJECTED {
                // 提案被拒绝:清理 Pending,释放地址锁定。
                DuoqianAccounts::<T>::remove(&action.duoqian_address);
                PersonalDuoqianInfo::<T>::remove(&action.duoqian_address);
                Self::deposit_event(Event::<T>::DuoqianCreateRejected {
                    proposal_id,
                    duoqian_address: action.duoqian_address.clone(),
                });
            }

            // 读回最新 status(execute_create 内部会把 PASSED 推进到 EXECUTED)
            let final_status = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .map(|p| p.status)
                .unwrap_or(proposal.status);
            Self::deposit_event(Event::<T>::CreateFinalized {
                proposal_id,
                signatures_accepted: accepted,
                final_status,
            });

            Ok(())
        }

        /// 中文注释：机构登记改为 proof 模式；任意提交者都可代发，但链上只信任 SFID MAIN 签出的字段包。
        #[pallet::call_index(2)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::register_sfid_institution())]
        pub fn register_sfid_institution(
            origin: OriginFor<T>,
            sfid_id: SfidIdOf<T>,
            account_name: AccountNameOf<T>,
            register_nonce: RegisterNonceOf<T>,
            signature: RegisterSignatureOf<T>,
            // 中文注释：可选的省名（UTF-8 字节），传入即按省签名密钥验签；不传走 SfidMainAccount 兼容路径。
            signing_province: Option<Vec<u8>>,
        ) -> DispatchResult {
            let submitter = ensure_signed(origin)?;
            ensure!(!sfid_id.is_empty(), Error::<T>::EmptySfidId);
            ensure!(!account_name.is_empty(), Error::<T>::EmptyAccountName);
            let register_nonce_hash = T::Hashing::hash(register_nonce.as_slice());
            ensure!(
                !UsedRegisterNonce::<T>::get(register_nonce_hash),
                Error::<T>::RegisterNonceAlreadyUsed
            );
            ensure!(
                T::SfidInstitutionVerifier::verify_institution_registration(
                    sfid_id.as_slice(),
                    &account_name,
                    &register_nonce,
                    &signature,
                    signing_province.as_deref(),
                ),
                Error::<T>::InvalidSfidInstitutionSignature
            );
            ensure!(
                !SfidRegisteredAddress::<T>::contains_key(&sfid_id, &account_name),
                Error::<T>::SfidAlreadyRegistered
            );

            // 按账户名翻译到 Role（"主账户"/"费用账户" 强制走 OP_MAIN/OP_FEE 且不再拼 account_name；
            // 其他非空 account_name 走 OP_INSTITUTION 并把 account_name 拼进 preimage）。
            let role = Self::role_from_account_name(account_name.as_slice())?;
            let duoqian_address = Self::derive_institution_address(sfid_id.as_slice(), role)?;
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

            SfidRegisteredAddress::<T>::insert(&sfid_id, &account_name, &duoqian_address);
            UsedRegisterNonce::<T>::insert(register_nonce_hash, true);
            AddressRegisteredSfid::<T>::insert(
                &duoqian_address,
                RegisteredInstitution {
                    sfid_id: sfid_id.clone(),
                    account_name: account_name.clone(),
                },
            );
            Self::deposit_event(Event::<T>::SfidInstitutionRegistered {
                sfid_id,
                account_name,
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
            ensure!(
                raw.len() > tag.len() && &raw[..tag.len()] == tag,
                Error::<T>::ProposalActionNotFound
            );
            ensure!(
                raw[tag.len()] == ACTION_CLOSE,
                Error::<T>::ProposalActionNotFound
            );
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
                    let exec_result =
                        with_transaction(|| match Self::execute_close(proposal_id, &action) {
                            Ok(()) => TransactionOutcome::Commit(Ok(())),
                            Err(e) => TransactionOutcome::Rollback(Err(e)),
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
        /// 地址由 `creator + account_name` 派生：
        /// `Blake2b_256(DUOQIAN_DOMAIN || OP_PERSONAL || SS58_PREFIX_LE || creator.encode() || name_utf8)`
        ///
        /// 投票通过后由 vote_create 自动执行入金 + 激活（复用 execute_create）。
        #[pallet::call_index(5)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::propose_create_personal())]
        pub fn propose_create_personal(
            origin: OriginFor<T>,
            account_name: AccountNameOf<T>,
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

            ensure!(!account_name.is_empty(), Error::<T>::EmptyPersonalName);

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
                Self::derive_personal_duoqian_address(&who, account_name.as_slice())?;
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
                    account_name: account_name.clone(),
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

            // 从投票引擎回读提案超时区块,便于 wuminapp 倒计时。
            let expires_at = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .map(|p| p.end)
                .ok_or(Error::<T>::VoteEngineError)?;

            Self::deposit_event(Event::<T>::PersonalDuoqianProposed {
                proposal_id,
                duoqian_address,
                proposer: who,
                account_name,
                admins: duoqian_admins,
                admin_count,
                threshold,
                amount,
                expires_at,
            });

            Ok(())
        }

        /// 清理已被拒绝或超时的创建/关闭提案残留状态。
        /// 任意签名账户可调用。用于解决投票引擎 on_initialize 超时 reject 后
        /// 本模块无法自动收到通知导致的 Pending / PendingCloseProposal 残留。
        #[pallet::call_index(6)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::cleanup_rejected_proposal())]
        pub fn cleanup_rejected_proposal(origin: OriginFor<T>, proposal_id: u64) -> DispatchResult {
            let _ = ensure_signed(origin)?;

            // 读取提案数据，校验 MODULE_TAG 后判断操作类型
            let raw = voting_engine_system::Pallet::<T>::get_proposal_data(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            let tag = crate::MODULE_TAG;
            ensure!(
                raw.len() > tag.len() && &raw[..tag.len()] == tag,
                Error::<T>::ProposalActionNotFound
            );
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
                    let action = CreateDuoqianAction::<T::AccountId, BalanceOf<T>>::decode(
                        &mut &raw[tag.len() + 1..],
                    )
                    .map_err(|_| Error::<T>::ProposalActionNotFound)?;
                    DuoqianAccounts::<T>::remove(&action.duoqian_address);
                    PersonalDuoqianInfo::<T>::remove(&action.duoqian_address);
                }
                ACTION_CLOSE => {
                    let action =
                        CloseDuoqianAction::<T::AccountId>::decode(&mut &raw[tag.len() + 1..])
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

        /// 按角色派生机构多签账户地址（所有机构统一走这条路径）。
        ///
        /// 派生公式按 `role` 分支：
        /// - `Main` → `blake2_256(DUOQIAN_DOMAIN || OP_MAIN || ss58_le || sfid_id)`
        /// - `Fee`  → `blake2_256(DUOQIAN_DOMAIN || OP_FEE  || ss58_le || sfid_id)`
        /// - `Named(account_name)` → `blake2_256(DUOQIAN_DOMAIN || OP_INSTITUTION || ss58_le || sfid_id || account_name)`
        ///
        /// 保留名校验：`Named(b"主账户")` 和 `Named(b"费用账户")` 被拒绝（返回
        /// `ReservedAccountName` 错误），强制这两个角色走 `Main`/`Fee` 分支避免
        /// 命名空间重叠。空 account_name 的 `Named` 也被拒绝（返回 `EmptyAccountName`）。
        pub fn derive_institution_address(
            sfid_id: &[u8],
            role: InstitutionAccountRole<'_>,
        ) -> Result<T::AccountId, DispatchError> {
            let (op_tag, name_suffix): (u8, &[u8]) = match role {
                InstitutionAccountRole::Main => (primitives::core_const::OP_MAIN, &[]),
                InstitutionAccountRole::Fee => (primitives::core_const::OP_FEE, &[]),
                InstitutionAccountRole::Named(n) => {
                    ensure!(!n.is_empty(), Error::<T>::EmptyAccountName);
                    ensure!(
                        n != RESERVED_NAME_MAIN && n != RESERVED_NAME_FEE,
                        Error::<T>::ReservedAccountName
                    );
                    (primitives::core_const::OP_INSTITUTION, n)
                }
            };
            let mut input = primitives::core_const::DUOQIAN_DOMAIN.to_vec();
            input.push(op_tag);
            input.extend_from_slice(&Self::chain_domain_prefix());
            input.extend_from_slice(sfid_id);
            input.extend_from_slice(name_suffix);
            let digest = sp_runtime::traits::BlakeTwo256::hash(input.as_slice());
            T::AccountId::decode(&mut digest.as_ref())
                .map_err(|_| Error::<T>::DerivedAddressDecodeFailed.into())
        }

        /// 把 SFID 账户名 bytes 翻译成 `InstitutionAccountRole`：
        /// - `"主账户"` → `Main`
        /// - `"费用账户"` → `Fee`
        /// - 其他非空 → `Named(account_name)`
        /// - 空 → 返回 `EmptyAccountName`
        ///
        /// 这是 `register_sfid_institution` 等 extrinsic 的唯一入口——禁止调用方
        /// 绕开此函数直接构造 `Role::Named("主账户")`（虽然 `derive_institution_address`
        /// 里也会拦截，但这里作为第一道防线更清晰）。
        pub fn role_from_account_name(
            account_name: &[u8],
        ) -> Result<InstitutionAccountRole<'_>, DispatchError> {
            if account_name.is_empty() {
                return Err(Error::<T>::EmptyAccountName.into());
            }
            if account_name == RESERVED_NAME_MAIN {
                Ok(InstitutionAccountRole::Main)
            } else if account_name == RESERVED_NAME_FEE {
                Ok(InstitutionAccountRole::Fee)
            } else {
                Ok(InstitutionAccountRole::Named(account_name))
            }
        }

        /// 从 creator + account_name 派生个人多签地址。
        /// 统一 domain：`DUOQIAN_DOMAIN || OP_PERSONAL || ss58_le || creator_32 || account_name_utf8`。
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

        fn ensure_unique_admins(admins: &DuoqianAdminsOf<T>) -> Result<(), DispatchError> {
            let mut seen = BTreeSet::new();
            for admin in admins.iter() {
                ensure!(seen.insert(admin.clone()), Error::<T>::DuplicateAdmin);
            }
            Ok(())
        }

        /// 计算 `admins_root = blake2_256(SCALE.encode(sorted_admins))`。
        ///
        /// 排序规则:按 `AccountId` 的字节序(Substrate AccountId32 默认 Ord 即字典序)。
        /// wuminapp 端需要用同样的排序规则 + SCALE 布局,保证签名消息字节一致。
        pub fn compute_admins_root(admins: &DuoqianAdminsOf<T>) -> [u8; 32] {
            let mut sorted: Vec<T::AccountId> = admins.iter().cloned().collect();
            sorted.sort();
            sp_io::hashing::blake2_256(&sorted.encode())
        }

        /// 把 `AccountId` 编码后的前 32 字节当作 sr25519 公钥。
        ///
        /// 铁律:项目内 `AccountId = AccountId32`,其 32 字节原始内容即对应 sr25519 公钥。
        /// 与 `offchain-transaction-pos::settlement::pubkey_from_accountid` 语义对齐。
        pub fn pubkey_from_accountid(acc: &T::AccountId) -> Result<Sr25519Public, Error<T>> {
            let encoded = acc.encode();
            if encoded.len() < 32 {
                return Err(Error::<T>::MalformedSignature);
            }
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&encoded[..32]);
            Ok(Sr25519Public::from_raw(arr))
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
    use sp_core::{sr25519, Pair as PairT};
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
    impl
        SfidInstitutionVerifier<
            AccountNameOf<Test>,
            RegisterNonceOf<Test>,
            RegisterSignatureOf<Test>,
        > for TestSfidInstitutionVerifier
    {
        fn verify_institution_registration(
            _sfid_id: &[u8],
            _account_name: &AccountNameOf<Test>,
            nonce: &RegisterNonceOf<Test>,
            signature: &RegisterSignatureOf<Test>,
            _signing_province: Option<&[u8]>,
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

        fn get_admin_list(org: u8, institution: InstitutionPalletId) -> Option<Vec<AccountId32>> {
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
        type MaxAccountNameLength = ConstU32<128>;
        type MaxRegisterNonceLength = ConstU32<64>;
        type MaxRegisterSignatureLength = ConstU32<64>;
        type MaxAdminSignatureLength = ConstU32<64>;
        type MinCreateAmount = ConstU128<111>;
        type MinCloseBalance = ConstU128<121>;
        type WeightInfo = ();
    }

    fn relayer() -> AccountId32 {
        AccountId32::new([0x55; 32])
    }

    /// 从固定 seed 派生 sr25519 keypair:公钥即为 AccountId32。
    /// 这样管理员既能作为链上 origin,又能对 CreateVoteIntent 做可验证 sr25519 签名,
    /// 支撑 `finalize_create` 的离线聚合签名测试。
    fn admin_pair(seed: u8) -> (AccountId32, sr25519::Pair) {
        let mut seed_bytes = [0u8; 32];
        seed_bytes[0] = seed;
        let pair = sr25519::Pair::from_seed(&seed_bytes);
        let account = AccountId32::new(pair.public().0);
        (account, pair)
    }

    fn admin(seed: u8) -> AccountId32 {
        admin_pair(seed).0
    }

    /// 从 seed 构造 admins BoundedVec + 对应的 Pair 数组,两者 index 对齐。
    fn make_admins_keyed(seeds: &[u8]) -> (DuoqianAdminsOf<Test>, Vec<sr25519::Pair>) {
        let mut accts = Vec::with_capacity(seeds.len());
        let mut pairs = Vec::with_capacity(seeds.len());
        for s in seeds {
            let (a, p) = admin_pair(*s);
            accts.push(a);
            pairs.push(p);
        }
        let bounded: DuoqianAdminsOf<Test> =
            accts.try_into().expect("admins bounded vec should fit");
        (bounded, pairs)
    }

    /// 构造一条管理员 sr25519 签名(对 `CreateVoteIntent`)。
    fn sign_create_intent(
        pair: &sr25519::Pair,
        proposal_id: u64,
        duoqian_address: &AccountId32,
        creator: &AccountId32,
        admins: &DuoqianAdminsOf<Test>,
        threshold: u32,
        amount: u128,
    ) -> AdminSignatureOf<Test> {
        let admins_root = Pallet::<Test>::compute_admins_root(admins);
        let intent = CreateVoteIntent::<AccountId32, u128> {
            proposal_id,
            duoqian_address: duoqian_address.clone(),
            creator: creator.clone(),
            admins_root,
            threshold,
            amount,
            approve: true,
        };
        let ss58 = <Test as frame_system::Config>::SS58Prefix::get();
        let msg = intent.signing_hash(ss58);
        let sig = pair.sign(&msg);
        sig.0.to_vec().try_into().expect("sig should fit")
    }

    /// finalize_create 聚合 helper:从 pairs 前 `take` 个生成签名 + 调 extrinsic。
    fn finalize_with(
        submitter: AccountId32,
        proposal_id: u64,
        duoqian_address: &AccountId32,
        creator: &AccountId32,
        admins: &DuoqianAdminsOf<Test>,
        pairs: &[sr25519::Pair],
        threshold: u32,
        amount: u128,
        take: usize,
    ) -> frame_support::dispatch::DispatchResult {
        let sigs: AdminSignaturesOf<Test> = admins
            .iter()
            .zip(pairs.iter())
            .take(take)
            .map(|(a, p)| {
                let sig = sign_create_intent(
                    p,
                    proposal_id,
                    duoqian_address,
                    creator,
                    admins,
                    threshold,
                    amount,
                );
                (a.clone(), sig)
            })
            .collect::<Vec<_>>()
            .try_into()
            .expect("sigs vec should fit");
        Duoqian::finalize_create(RuntimeOrigin::signed(submitter), proposal_id, sigs)
    }

    fn register_sfid_with_account_name(
        tag: &str,
        account_name_bytes: &[u8],
    ) -> (SfidIdOf<Test>, AccountNameOf<Test>, AccountId32) {
        let sfid: SfidIdOf<Test> = format!("GFR-LN001-CB0C-{}-20260222", tag)
            .as_bytes()
            .to_vec()
            .try_into()
            .expect("sfid id should fit");
        let account_name: AccountNameOf<Test> = account_name_bytes
            .to_vec()
            .try_into()
            .expect("account_name should fit");
        let mut nonce_bytes = format!("register-{}-", tag).into_bytes();
        nonce_bytes.extend_from_slice(&sp_io::hashing::blake2_128(account_name_bytes));
        let register_nonce: RegisterNonceOf<Test> =
            nonce_bytes.try_into().expect("register nonce should fit");
        let signature: RegisterSignatureOf<Test> = b"register-ok"
            .to_vec()
            .try_into()
            .expect("register signature should fit");
        assert_ok!(Duoqian::register_sfid_institution(
            RuntimeOrigin::signed(relayer()),
            sfid.clone(),
            account_name.clone(),
            register_nonce,
            signature,
            None,
        ));
        let duoqian_address = SfidRegisteredAddress::<Test>::get(&sfid, &account_name)
            .expect("sfid should be registered");
        (sfid, account_name, duoqian_address)
    }

    fn register_sfid_and_get_address(
        tag: &str,
    ) -> (SfidIdOf<Test>, AccountNameOf<Test>, AccountId32) {
        let account_name = format!("Test Institution {}", tag);
        register_sfid_with_account_name(tag, account_name.as_bytes())
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
            let (sfid, account_name, duoqian_address) = register_sfid_and_get_address("A001");
            assert!(SfidRegisteredAddress::<Test>::contains_key(
                &sfid,
                &account_name
            ));
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
            let account_name: AccountNameOf<Test> = b"Bad Institution"
                .to_vec()
                .try_into()
                .expect("account_name should fit");
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
                    account_name,
                    register_nonce,
                    bad_signature,
                    None,
                ),
                Error::<Test>::InvalidSfidInstitutionSignature
            );
        });
    }

    #[test]
    fn reserved_role_names_register_to_fixed_main_and_fee_addresses() {
        new_test_ext().execute_with(|| {
            // 中文注释：同一个 sfid_id 下，"主账户"/"费用账户" 必须强制命中固定角色派生。
            let (sfid, _, main_address) =
                register_sfid_with_account_name("ROLE1", RESERVED_NAME_MAIN);
            let (same_sfid, _, fee_address) =
                register_sfid_with_account_name("ROLE1", RESERVED_NAME_FEE);

            assert_eq!(sfid, same_sfid);

            let expected_main = Pallet::<Test>::derive_institution_address(
                sfid.as_slice(),
                InstitutionAccountRole::Main,
            )
            .expect("main address should derive");
            let expected_fee = Pallet::<Test>::derive_institution_address(
                sfid.as_slice(),
                InstitutionAccountRole::Fee,
            )
            .expect("fee address should derive");

            assert_eq!(main_address, expected_main);
            assert_eq!(fee_address, expected_fee);
            assert_ne!(main_address, fee_address);
        });
    }

    #[test]
    fn reserved_role_names_cannot_fall_back_to_named_namespace() {
        new_test_ext().execute_with(|| {
            let sfid: SfidIdOf<Test> = b"GFR-LN001-CB0C-ROLE2-20260222"
                .to_vec()
                .try_into()
                .expect("sfid id should fit");

            // 中文注释：保留角色名不能通过 Named 分支落到 OP_INSTITUTION 命名空间。
            let main_err = Pallet::<Test>::derive_institution_address(
                sfid.as_slice(),
                InstitutionAccountRole::Named(RESERVED_NAME_MAIN),
            )
            .expect_err("reserved main name must be rejected");
            assert_eq!(main_err, Error::<Test>::ReservedAccountName.into());

            let fee_err = Pallet::<Test>::derive_institution_address(
                sfid.as_slice(),
                InstitutionAccountRole::Named(RESERVED_NAME_FEE),
            )
            .expect_err("reserved fee name must be rejected");
            assert_eq!(fee_err, Error::<Test>::ReservedAccountName.into());
        });
    }

    #[test]
    fn propose_create_and_finalize_to_activate() {
        new_test_ext().execute_with(|| {
            let (sfid, account_name, duoqian_address) = register_sfid_and_get_address("B001");
            let (admins, pairs) = make_admins_keyed(&[1, 2, 3]);

            // Tx 1:发起创建提案
            assert_ok!(Duoqian::propose_create(
                RuntimeOrigin::signed(admins[0].clone()),
                sfid.clone(),
                account_name.clone(),
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

            // Tx 2:离线聚合 2 个管理员 sr25519 签名,发起人一笔 finalize_create
            assert_ok!(finalize_with(
                admins[0].clone(),
                pid,
                &duoqian_address,
                &admins[0],
                &admins,
                &pairs,
                2,
                1_000,
                2,
            ));

            // 投票通过 + execute_create 已入金 + 状态 Active
            let account = DuoqianAccounts::<Test>::get(&duoqian_address).expect("should exist");
            assert_eq!(account.status, DuoqianStatus::Active);
            assert_eq!(Balances::free_balance(&duoqian_address), 1_000);
        });
    }

    #[test]
    fn propose_close_and_vote_to_close() {
        new_test_ext().execute_with(|| {
            let (sfid, account_name, duoqian_address) = register_sfid_and_get_address("C001");
            let (admins, pairs) = make_admins_keyed(&[1, 2, 3]);
            let beneficiary = admin(4);

            // 先创建(走 finalize_create 离线聚合路径)
            assert_ok!(Duoqian::propose_create(
                RuntimeOrigin::signed(admins[0].clone()),
                sfid.clone(),
                account_name.clone(),
                3,
                admins.clone(),
                2,
                1_000,
            ));
            let create_pid = last_proposal_id();
            assert_ok!(finalize_with(
                admins[0].clone(),
                create_pid,
                &duoqian_address,
                &admins[0],
                &admins,
                &pairs,
                2,
                1_000,
                2,
            ));

            // 确认 active
            let account = DuoqianAccounts::<Test>::get(&duoqian_address).expect("should exist");
            assert_eq!(account.status, DuoqianStatus::Active);

            // 发起关闭提案
            assert_ok!(Duoqian::propose_close(
                RuntimeOrigin::signed(admins[0].clone()),
                duoqian_address.clone(),
                beneficiary.clone(),
            ));

            let close_pid = last_proposal_id();

            // 关闭仍走在线投票(vote_close 本步未改造)
            assert_ok!(Duoqian::vote_close(
                RuntimeOrigin::signed(admins[0].clone()),
                close_pid,
                true
            ));
            assert_ok!(Duoqian::vote_close(
                RuntimeOrigin::signed(admins[1].clone()),
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
            let (sfid, account_name, _) = register_sfid_and_get_address("D001");
            let admins = make_admins(&[1, 2, 3]);

            // admin(4) 不在管理员列表中
            assert_noop!(
                Duoqian::propose_create(
                    RuntimeOrigin::signed(admin(4)),
                    sfid.clone(),
                    account_name.clone(),
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
    fn finalize_rejects_non_admin_signature() {
        new_test_ext().execute_with(|| {
            let (sfid, account_name, duoqian_address) = register_sfid_and_get_address("E001");
            let (admins, pairs) = make_admins_keyed(&[1, 2, 3]);

            assert_ok!(Duoqian::propose_create(
                RuntimeOrigin::signed(admins[0].clone()),
                sfid.clone(),
                account_name.clone(),
                3,
                admins.clone(),
                2,
                1_000,
            ));

            let pid = last_proposal_id();

            // 构造 sig:一个合法管理员 + 一个非管理员(admin(4) 不在 admins 列表)
            let legit_sig = sign_create_intent(
                &pairs[0],
                pid,
                &duoqian_address,
                &admins[0],
                &admins,
                2,
                1_000,
            );
            let (outsider, outsider_pair) = admin_pair(99);
            let outsider_sig = sign_create_intent(
                &outsider_pair,
                pid,
                &duoqian_address,
                &admins[0],
                &admins,
                2,
                1_000,
            );
            let sigs: AdminSignaturesOf<Test> = vec![
                (admins[0].clone(), legit_sig),
                (outsider, outsider_sig),
            ]
            .try_into()
            .expect("sigs vec should fit");

            assert_noop!(
                Duoqian::finalize_create(
                    RuntimeOrigin::signed(admins[0].clone()),
                    pid,
                    sigs
                ),
                Error::<Test>::UnauthorizedSignature
            );
        });
    }

    #[test]
    fn cannot_close_pending_account() {
        new_test_ext().execute_with(|| {
            let (sfid, account_name, duoqian_address) = register_sfid_and_get_address("F001");
            let admins = make_admins(&[1, 2, 3]);

            // propose create 但不投票通过
            assert_ok!(Duoqian::propose_create(
                RuntimeOrigin::signed(admin(1)),
                sfid,
                account_name.clone(),
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
            let (sfid, account_name, duoqian_address) = register_sfid_and_get_address("F002");
            let (admins, pairs) = make_admins_keyed(&[1, 2, 3]);

            assert_ok!(Duoqian::propose_create(
                RuntimeOrigin::signed(admins[0].clone()),
                sfid,
                account_name.clone(),
                3,
                admins.clone(),
                2,
                1_000,
            ));
            let create_pid = last_proposal_id();
            assert_ok!(finalize_with(
                admins[0].clone(),
                create_pid,
                &duoqian_address,
                &admins[0],
                &admins,
                &pairs,
                2,
                1_000,
                2,
            ));

            DENIED_CLOSE_SOURCE
                .with(|blocked| *blocked.borrow_mut() = Some(duoqian_address.clone()));

            assert_noop!(
                Duoqian::propose_close(
                    RuntimeOrigin::signed(admins[0].clone()),
                    duoqian_address,
                    admin(4),
                ),
                Error::<Test>::ProtectedSource
            );

            DENIED_CLOSE_SOURCE.with(|blocked| *blocked.borrow_mut() = None);
        });
    }

    #[test]
    fn duplicate_admin_is_rejected() {
        new_test_ext().execute_with(|| {
            let (sfid, account_name, _) = register_sfid_and_get_address("G001");
            let admins: DuoqianAdminsOf<Test> = vec![admin(1), admin(1), admin(2)]
                .try_into()
                .expect("should fit");

            assert_noop!(
                Duoqian::propose_create(
                    RuntimeOrigin::signed(admin(1)),
                    sfid,
                    account_name.clone(),
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
            let (sfid, account_name, _) = register_sfid_and_get_address("H001");
            let admins = make_admins(&[1, 2, 3]);

            assert_noop!(
                Duoqian::propose_create(
                    RuntimeOrigin::signed(admin(1)),
                    sfid,
                    account_name.clone(),
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
            let (sfid, account_name, duoqian_address) = register_sfid_and_get_address("I001");
            let (admins, pairs) = make_admins_keyed(&[1, 2]);

            // 创建并激活(走 finalize_create)
            assert_ok!(Duoqian::propose_create(
                RuntimeOrigin::signed(admins[0].clone()),
                sfid,
                account_name.clone(),
                2,
                admins.clone(),
                2,
                1_000,
            ));
            let create_pid = last_proposal_id();
            assert_ok!(finalize_with(
                admins[0].clone(),
                create_pid,
                &duoqian_address,
                &admins[0],
                &admins,
                &pairs,
                2,
                1_000,
                2,
            ));

            let beneficiary = admin(3);

            // 第一个关闭提案 — 应该成功
            assert_ok!(Duoqian::propose_close(
                RuntimeOrigin::signed(admins[0].clone()),
                duoqian_address.clone(),
                beneficiary.clone(),
            ));

            // 第二个关闭提案 — 应该被 CloseAlreadyPending 拒绝
            assert_noop!(
                Duoqian::propose_close(
                    RuntimeOrigin::signed(admins[1].clone()),
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
            let (sfid, account_name, duoqian_address) = register_sfid_and_get_address("J001");
            let (admins, pairs) = make_admins_keyed(&[1, 2]);

            assert_ok!(Duoqian::propose_create(
                RuntimeOrigin::signed(admins[0].clone()),
                sfid.clone(),
                account_name.clone(),
                2,
                admins.clone(),
                2,
                1_000,
            ));
            let pid = last_proposal_id();

            // 排干 admins[0] 的余额,使 execute_create 在 transfer 时失败
            let _ = Balances::slash(&admins[0], 99_900);
            assert!(Balances::free_balance(&admins[0]) < 1_010);

            // finalize_create 达阈值 → 触发 execute_create,因余额不足失败
            assert_ok!(finalize_with(
                admins[0].clone(),
                pid,
                &duoqian_address,
                &admins[0],
                &admins,
                &pairs,
                2,
                1_000,
                2,
            ));

            // execute_create 失败后 DuoqianAccounts 中的 Pending 条目应被清除
            assert!(
                DuoqianAccounts::<Test>::get(&duoqian_address).is_none(),
                "pending entry must be cleaned up after execute_create failure"
            );

            // PersonalDuoqianInfo 也不应残留（机构多签无条目，remove 为 no-op）
            assert!(PersonalDuoqianInfo::<Test>::get(&duoqian_address).is_none());
        });
    }

    /// 个人多签(无 SFID 归属)也走 finalize_create 离线聚合路径。
    #[test]
    fn personal_duoqian_create_works() {
        new_test_ext().execute_with(|| {
            let account_name: AccountNameOf<Test> = b"Family Fund"
                .to_vec()
                .try_into()
                .expect("account_name should fit");
            let (admins, pairs) = make_admins_keyed(&[1, 2]);

            assert_ok!(Duoqian::propose_create_personal(
                RuntimeOrigin::signed(admins[0].clone()),
                account_name.clone(),
                2,
                admins.clone(),
                2,
                1_000,
            ));
            let pid = last_proposal_id();

            // 派生地址
            let duoqian_address = Pallet::<Test>::derive_personal_duoqian_address(
                &admins[0],
                account_name.as_slice(),
            )
            .expect("derive should succeed");

            // finalize 前处于 Pending
            assert_eq!(
                DuoqianAccounts::<Test>::get(&duoqian_address).map(|a| a.status),
                Some(DuoqianStatus::Pending)
            );

            // 两人签名聚合一笔 finalize_create
            assert_ok!(finalize_with(
                admins[0].clone(),
                pid,
                &duoqian_address,
                &admins[0],
                &admins,
                &pairs,
                2,
                1_000,
                2,
            ));

            // 投票通过后变为 Active,资金已转入
            let account = DuoqianAccounts::<Test>::get(&duoqian_address).expect("should exist");
            assert_eq!(account.status, DuoqianStatus::Active);
            assert_eq!(Balances::free_balance(&duoqian_address), 1_000);

            // PersonalDuoqianInfo 已写入
            let meta = PersonalDuoqianInfo::<Test>::get(&duoqian_address)
                .expect("personal info should exist");
            assert_eq!(meta.creator, admins[0]);
            assert_eq!(meta.account_name, account_name);
        });
    }

    // ──── Step 1 离线 QR 聚合专项测试 ────

    /// finalize_create 签名数不足阈值时必须拒绝。
    #[test]
    fn finalize_create_insufficient_sigs_rejected() {
        new_test_ext().execute_with(|| {
            let (sfid, account_name, duoqian_address) = register_sfid_and_get_address("K001");
            let (admins, pairs) = make_admins_keyed(&[1, 2, 3]);

            assert_ok!(Duoqian::propose_create(
                RuntimeOrigin::signed(admins[0].clone()),
                sfid,
                account_name,
                3,
                admins.clone(),
                2,
                1_000,
            ));
            let pid = last_proposal_id();

            // 只提交 1 个签名,阈值 2,应拒绝
            assert_noop!(
                finalize_with(
                    admins[0].clone(),
                    pid,
                    &duoqian_address,
                    &admins[0],
                    &admins,
                    &pairs,
                    2,
                    1_000,
                    1,
                ),
                Error::<Test>::InsufficientSignatures
            );
        });
    }

    /// 同一 admin 在同一批签名里重复出现必须拒绝。
    #[test]
    fn finalize_create_duplicate_sig_rejected() {
        new_test_ext().execute_with(|| {
            let (sfid, account_name, duoqian_address) = register_sfid_and_get_address("K002");
            let (admins, pairs) = make_admins_keyed(&[1, 2, 3]);

            assert_ok!(Duoqian::propose_create(
                RuntimeOrigin::signed(admins[0].clone()),
                sfid,
                account_name,
                3,
                admins.clone(),
                2,
                1_000,
            ));
            let pid = last_proposal_id();

            let sig0 =
                sign_create_intent(&pairs[0], pid, &duoqian_address, &admins[0], &admins, 2, 1_000);
            let sig0_dup =
                sign_create_intent(&pairs[0], pid, &duoqian_address, &admins[0], &admins, 2, 1_000);
            let sigs: AdminSignaturesOf<Test> = vec![
                (admins[0].clone(), sig0),
                (admins[0].clone(), sig0_dup),
            ]
            .try_into()
            .expect("sigs vec should fit");

            assert_noop!(
                Duoqian::finalize_create(
                    RuntimeOrigin::signed(admins[0].clone()),
                    pid,
                    sigs
                ),
                Error::<Test>::DuplicateSignature
            );
        });
    }

    /// 被篡改的签名验证失败必须拒绝。
    #[test]
    fn finalize_create_tampered_sig_rejected() {
        new_test_ext().execute_with(|| {
            let (sfid, account_name, duoqian_address) = register_sfid_and_get_address("K003");
            let (admins, pairs) = make_admins_keyed(&[1, 2, 3]);

            assert_ok!(Duoqian::propose_create(
                RuntimeOrigin::signed(admins[0].clone()),
                sfid,
                account_name,
                3,
                admins.clone(),
                2,
                1_000,
            ));
            let pid = last_proposal_id();

            // 对错误 amount 签(wuminapp 用 999 签但链上 Tx 1 写的是 1_000)
            let wrong_sig = sign_create_intent(
                &pairs[0],
                pid,
                &duoqian_address,
                &admins[0],
                &admins,
                2,
                999,
            );
            let right_sig = sign_create_intent(
                &pairs[1],
                pid,
                &duoqian_address,
                &admins[0],
                &admins,
                2,
                1_000,
            );
            let sigs: AdminSignaturesOf<Test> = vec![
                (admins[0].clone(), wrong_sig),
                (admins[1].clone(), right_sig),
            ]
            .try_into()
            .expect("sigs vec should fit");

            assert_noop!(
                Duoqian::finalize_create(
                    RuntimeOrigin::signed(admins[0].clone()),
                    pid,
                    sigs
                ),
                Error::<Test>::InvalidSignature
            );
        });
    }

    /// 签名长度不是 64 字节必须拒绝。
    #[test]
    fn finalize_create_malformed_sig_rejected() {
        new_test_ext().execute_with(|| {
            let (sfid, account_name, duoqian_address) = register_sfid_and_get_address("K004");
            let (admins, pairs) = make_admins_keyed(&[1, 2, 3]);

            assert_ok!(Duoqian::propose_create(
                RuntimeOrigin::signed(admins[0].clone()),
                sfid,
                account_name,
                3,
                admins.clone(),
                2,
                1_000,
            ));
            let pid = last_proposal_id();

            let good =
                sign_create_intent(&pairs[0], pid, &duoqian_address, &admins[0], &admins, 2, 1_000);
            // 构造长度 32 的非法签名(应为 64)
            let bad_sig: AdminSignatureOf<Test> =
                vec![0u8; 32].try_into().expect("32 bytes fits");
            let sigs: AdminSignaturesOf<Test> = vec![
                (admins[0].clone(), good),
                (admins[1].clone(), bad_sig),
            ]
            .try_into()
            .expect("sigs vec should fit");

            assert_noop!(
                Duoqian::finalize_create(
                    RuntimeOrigin::signed(admins[0].clone()),
                    pid,
                    sigs
                ),
                Error::<Test>::MalformedSignature
            );
        });
    }

    /// finalize_create 成功后再次调用应当被投票引擎的 AlreadyVoted 保护。
    #[test]
    fn finalize_create_second_call_is_replay_protected() {
        new_test_ext().execute_with(|| {
            let (sfid, account_name, duoqian_address) = register_sfid_and_get_address("K005");
            let (admins, pairs) = make_admins_keyed(&[1, 2, 3]);

            assert_ok!(Duoqian::propose_create(
                RuntimeOrigin::signed(admins[0].clone()),
                sfid,
                account_name,
                3,
                admins.clone(),
                2,
                1_000,
            ));
            let pid = last_proposal_id();

            // 第一次 finalize:成功激活
            assert_ok!(finalize_with(
                admins[0].clone(),
                pid,
                &duoqian_address,
                &admins[0],
                &admins,
                &pairs,
                2,
                1_000,
                2,
            ));
            assert_eq!(
                DuoqianAccounts::<Test>::get(&duoqian_address).map(|a| a.status),
                Some(DuoqianStatus::Active)
            );

            // 第二次 finalize:投票引擎已 STATUS_EXECUTED → 非 Voting 状态 → 拒绝
            let second = finalize_with(
                admins[0].clone(),
                pid,
                &duoqian_address,
                &admins[0],
                &admins,
                &pairs,
                2,
                1_000,
                2,
            );
            assert!(second.is_err(), "replay must fail");

            // 余额依然等于第一次的 1_000(没有重复入金)
            assert_eq!(Balances::free_balance(&duoqian_address), 1_000);
        });
    }
}
