#![cfg_attr(not(feature = "std"), no_std)]

/// 模块标识前缀，用于在 ProposalData 中区分不同业务模块，防止跨模块误解码。
pub const MODULE_TAG: &[u8] = b"dq-mgmt";

pub use pallet::*;
pub mod address;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod close;
pub mod common;
pub mod execute;
pub mod institution;
pub mod personal;
pub mod traits;
pub mod weights;

pub use traits::{
    DuoqianAddressValidator, DuoqianReservedAddressChecker, ProtectedSourceChecker,
    SfidInstitutionVerifier,
};
pub use common::{account_to_institution_id, sfid_id_to_institution_id};

use admins_change::SubjectLifecycle;
use codec::{Decode, Encode};
use frame_support::{
    ensure,
    pallet_prelude::*,
    traits::{Currency, ReservableCurrency},
    BoundedVec,
};
use frame_system::pallet_prelude::*;
use sp_core::sr25519::Public as Sr25519Public;
use sp_runtime::traits::Hash;
use sp_std::{collections::btree_set::BTreeSet, prelude::*};
use voting_engine::{
    InstitutionPalletId, InternalVoteResultCallback, ProposalExecutionOutcome, STATUS_REJECTED,
};

pub use address::{InstitutionAccountRole, RESERVED_NAME_FEE, RESERVED_NAME_MAIN};
pub use institution::types::{
    CreateInstitutionAccount, CreateInstitutionAction, InstitutionAccountInfo, InstitutionInfo,
    InstitutionInitialAccount, InstitutionLifecycleStatus, RegisteredInstitution,
};
pub use personal::types::{
    CloseDuoqianAction, CreateDuoqianAction, DuoqianAccount, DuoqianStatus, PersonalDuoqianMeta,
};

pub(crate) type BalanceOf<T> =
    <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(6);

    #[pallet::config]
    pub trait Config: frame_system::Config + voting_engine::Config + admins_change::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// 内部投票引擎
        type InternalVoteEngine: voting_engine::InternalVoteEngine<Self::AccountId>;

        type AddressValidator: DuoqianAddressValidator<Self::AccountId>;
        type ReservedAddressChecker: DuoqianReservedAddressChecker<Self::AccountId>;
        type ProtectedSourceChecker: ProtectedSourceChecker<Self::AccountId>;
        type InstitutionAsset: institution_asset::InstitutionAsset<Self::AccountId>;
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

        /// 单个机构创建交易最多可携带的账户数量。
        ///
        /// SFID 默认包含主账户和费用账户，用户可新增其他账户；这里限制链上
        /// 初始入金列表长度，避免机构创建提案业务数据过大。
        #[pallet::constant]
        type MaxInstitutionAccounts: Get<u32>;

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
    /// 中文注释:注册凭证里的账户名列表,顺序必须与 SFID `registration-info` 返回一致。
    pub type InstitutionAccountNamesOf<T> =
        BoundedVec<AccountNameOf<T>, <T as Config>::MaxInstitutionAccounts>;
    /// 机构创建时用户输入的账户初始余额列表项。
    pub type InstitutionInitialAccountOf<T> =
        InstitutionInitialAccount<AccountNameOf<T>, BalanceOf<T>>;
    /// 机构创建时用户输入的账户初始余额列表。
    pub type InstitutionInitialAccountsOf<T> =
        BoundedVec<InstitutionInitialAccountOf<T>, <T as Config>::MaxInstitutionAccounts>;
    /// 机构创建提案中保存的已派生账户项。
    pub type CreateInstitutionAccountOf<T> = CreateInstitutionAccount<
        AccountNameOf<T>,
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
    >;
    /// 机构创建提案中保存的已派生账户列表。
    pub type CreateInstitutionAccountsOf<T> =
        BoundedVec<CreateInstitutionAccountOf<T>, <T as Config>::MaxInstitutionAccounts>;
    /// 机构级多签信息。
    pub type InstitutionInfoOf<T> = InstitutionInfo<
        DuoqianAdminsOf<T>,
        <T as frame_system::Config>::AccountId,
        BlockNumberFor<T>,
        AccountNameOf<T>,
    >;
    /// 机构账户信息。
    pub type InstitutionAccountInfoOf<T> = InstitutionAccountInfo<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        BlockNumberFor<T>,
    >;
    /// 机构创建提案业务数据。
    pub type CreateInstitutionActionOf<T> = CreateInstitutionAction<
        SfidIdOf<T>,
        AccountNameOf<T>,
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        DuoqianAdminsOf<T>,
        CreateInstitutionAccountsOf<T>,
    >;

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

    /// 机构级多签信息：key 为 sfid_id。
    ///
    /// 链上创建的是“机构”，机构下账户只保存地址、初始余额与生命周期状态。
    /// 管理员和阈值的长期真源在 admins-change；本表保存机构基本信息和创建快照。
    #[pallet::storage]
    #[pallet::getter(fn institution_of)]
    pub type Institutions<T: Config> =
        StorageMap<_, Blake2_128Concat, SfidIdOf<T>, InstitutionInfoOf<T>, OptionQuery>;

    /// 机构账户表：(sfid_id, account_name) -> 账户地址与激活状态。
    #[pallet::storage]
    #[pallet::getter(fn institution_account_of)]
    pub type InstitutionAccounts<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        SfidIdOf<T>,
        Blake2_128Concat,
        AccountNameOf<T>,
        InstitutionAccountInfoOf<T>,
        OptionQuery,
    >;

    /// 正在投票中的机构创建提案，用于通过/拒绝时处理 reserve 资金。
    #[pallet::storage]
    #[pallet::getter(fn pending_institution_create)]
    pub type PendingInstitutionCreate<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, CreateInstitutionActionOf<T>, OptionQuery>;

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

    /// 正在投票中的个人多签创建提案,用于通过/拒绝时处理 reserve 资金。
    ///
    /// 资金模型与机构多签 `PendingInstitutionCreate` 一致(2026-05-03 整改):
    /// 发起时 reserve(amount + fee), 通过后 unreserve + transfer + withdraw fee,
    /// 否决/终态失败 unreserve。
    #[pallet::storage]
    #[pallet::getter(fn pending_personal_create)]
    pub type PendingPersonalCreate<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        CreateDuoqianAction<T::AccountId, BalanceOf<T>>,
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
        /// 机构多签账户创建提案已发起（pending 状态预写入）。
        /// wuminapp 扫描此事件后引导其他管理员到投票引擎统一入口 `internal_vote` 投票。
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
        /// 机构级创建提案已发起：创建者资金已 reserve，等待管理员投票。
        InstitutionCreateProposed {
            proposal_id: u64,
            sfid_id: SfidIdOf<T>,
            institution_name: AccountNameOf<T>,
            main_address: T::AccountId,
            proposer: T::AccountId,
            accounts: CreateInstitutionAccountsOf<T>,
            admins: DuoqianAdminsOf<T>,
            admin_count: u32,
            threshold: u32,
            initial_total: BalanceOf<T>,
            reserve_total: BalanceOf<T>,
            expires_at: BlockNumberFor<T>,
        },
        /// 机构创建成功：机构和账户均已激活。
        InstitutionCreated {
            proposal_id: u64,
            sfid_id: SfidIdOf<T>,
            main_address: T::AccountId,
            account_count: u32,
            initial_total: BalanceOf<T>,
            fee: BalanceOf<T>,
        },
        /// 机构创建执行失败：回滚后释放 pending 占用和 reserve 资金。
        InstitutionCreateExecutionFailed {
            proposal_id: u64,
            sfid_id: SfidIdOf<T>,
            main_address: T::AccountId,
        },
        /// 机构创建提案被否决或超时清理：释放创建者 reserve 资金。
        InstitutionCreateRejected {
            proposal_id: u64,
            sfid_id: SfidIdOf<T>,
            main_address: T::AccountId,
            reserve_total: BalanceOf<T>,
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
        /// 机构账户初始余额低于最小门槛
        AccountInitialAmountBelowMinimum,
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
        /// ADR-008 step2b 新增:机构登记凭证缺省份(province 改必填后空字节串拒绝)
        EmptyProvince,
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
        /// 机构级创建缺少主账户
        MissingMainAccount,
        /// 机构级创建缺少费用账户
        MissingFeeAccount,
        /// 机构级创建账户名重复
        DuplicateAccountName,
        /// 机构已经存在
        InstitutionAlreadyExists,
        /// 机构账户列表为空
        EmptyInstitutionAccounts,
        /// 机构账户数量超过上限
        TooManyInstitutionAccounts,
        /// 初始余额累计溢出
        InitialAmountOverflow,
        /// 创建者资金 reserve 失败
        ReserveFailed,
        /// reserve 释放异常
        ReserveReleaseFailed,
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

    /// 提案操作类型标记：存储在 ProposalData 的第一个字节。
    /// 删 propose_create call=0(2026-05-03)后, ACTION=1 仅用于个人多签,
    /// 改名 ACTION_CREATE_PERSONAL,SCALE 数值不变,decoder 字面识别能力不变。
    pub const ACTION_CREATE_PERSONAL: u8 = 1;
    pub const ACTION_CLOSE: u8 = 2;
    pub const ACTION_CREATE_INSTITUTION: u8 = 3;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // NOTE: `call_index` values are the on-chain ABI and must remain stable.

        // call_index = 0 已废弃 (2026-05-03):
        // 原 `propose_create` 单账户机构创建入口已删除。机构多签业务最少
        // 必须有 2 个账户(主账户 + 费用账户),统一通过 call_index=5 的
        // `propose_create_institution` 一次性创建机构整体。call_index=0 留洞,
        // 绝不复用。

        /// SFID 注册信息凭证批量登记机构账户地址。
        ///
        /// 中文注释:本入口与 SFID `/registration-info` 对齐,业务字段只接收
        /// `sfid_id / institution_name / account_names[]`。机构类型、企业类型、
        /// 所属法人关系只由 SFID 系统用于候选资格判断,不再进入链上注册 payload。
        #[pallet::call_index(2)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::register_sfid_institution())]
        pub fn register_sfid_institution(
            origin: OriginFor<T>,
            sfid_id: SfidIdOf<T>,
            institution_name: AccountNameOf<T>,
            account_names: InstitutionAccountNamesOf<T>,
            register_nonce: RegisterNonceOf<T>,
            signature: RegisterSignatureOf<T>,
            province: Vec<u8>,
            signer_admin_pubkey: [u8; 32],
        ) -> DispatchResult {
            let submitter = ensure_signed(origin)?;
            crate::institution::register::do_register_sfid_institution::<T>(
                submitter,
                sfid_id,
                institution_name,
                account_names,
                register_nonce,
                signature,
                province,
                signer_admin_pubkey,
            )
        }

        /// 发起机构级创建提案。
        ///
        /// 该交易注册的是“机构”而不是单个账户。创建者必须一次性提交主账户、
        /// 费用账户以及需要初始化的自定义账户余额；交易发起时 reserve 创建者
        /// 的初始余额合计与手续费，投票通过后再划入机构各账户。
        #[pallet::call_index(5)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::propose_create_institution())]
        pub fn propose_create_institution(
            origin: OriginFor<T>,
            sfid_id: SfidIdOf<T>,
            institution_name: AccountNameOf<T>,
            accounts: InstitutionInitialAccountsOf<T>,
            admin_count: u32,
            duoqian_admins: DuoqianAdminsOf<T>,
            threshold: u32,
            register_nonce: RegisterNonceOf<T>,
            signature: RegisterSignatureOf<T>,
            // ADR-008 step2b：必填省份 + 签名 admin pubkey,与 register_sfid_institution 同义。
            province: Vec<u8>,
            signer_admin_pubkey: [u8; 32],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            crate::institution::create::do_propose_create_institution::<T>(
                who,
                sfid_id,
                institution_name,
                accounts,
                admin_count,
                duoqian_admins,
                threshold,
                register_nonce,
                signature,
                province,
                signer_admin_pubkey,
            )
        }

        /// 发起"关闭多签账户"提案。
        ///
        /// 个人多签和机构多签共用此入口,业务体在 `crate::close::do_propose_close`,
        /// 内部按 `resolve_admin_subject_for_account` 自动归属管理员主体。
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

        /// 发起"创建个人多签账户"提案（无需 SFID 注册）。
        ///
        /// 地址由 `creator + account_name` 派生：
        /// `Blake2b_256(DUOQIAN_DOMAIN || OP_PERSONAL || SS58_PREFIX_LE || creator.encode() || name_utf8)`
        ///
        /// 投票通过后由 vote_create 自动执行入金 + 激活（复用 execute_create）。
        #[pallet::call_index(3)]
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
            crate::personal::create::do_propose_create_personal::<T>(
                who,
                account_name,
                admin_count,
                duoqian_admins,
                threshold,
                amount,
            )
        }

        /// 清理已被拒绝或超时的创建/关闭提案残留状态。
        /// 任意签名账户可调用。用于解决投票引擎 on_initialize 超时 reject 后
        /// 本模块无法自动收到通知导致的 Pending / PendingCloseProposal 残留。
        #[pallet::call_index(4)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::cleanup_rejected_proposal())]
        pub fn cleanup_rejected_proposal(origin: OriginFor<T>, proposal_id: u64) -> DispatchResult {
            let _ = ensure_signed(origin)?;

            // 读取提案数据，校验 MODULE_TAG 后判断操作类型
            let raw = voting_engine::Pallet::<T>::get_proposal_data(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            let tag = crate::MODULE_TAG;
            ensure!(
                raw.len() > tag.len() && &raw[..tag.len()] == tag,
                Error::<T>::ProposalActionNotFound
            );
            let action_tag = raw[tag.len()];

            // 校验投票引擎状态必须为 REJECTED
            let proposal = voting_engine::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            ensure!(
                proposal.status == STATUS_REJECTED,
                Error::<T>::ProposalNotRejected
            );

            match action_tag {
                ACTION_CREATE_PERSONAL => {
                    let action = CreateDuoqianAction::<T::AccountId, BalanceOf<T>>::decode(
                        &mut &raw[tag.len() + 1..],
                    )
                    .map_err(|_| Error::<T>::ProposalActionNotFound)?;
                    crate::personal::execute::cleanup_pending_personal_create::<T>(
                        proposal_id,
                        &action,
                        true,
                    );
                }
                ACTION_CREATE_INSTITUTION => {
                    let action = CreateInstitutionActionOf::<T>::decode(&mut &raw[tag.len() + 1..])
                        .map_err(|_| Error::<T>::ProposalActionNotFound)?;
                    crate::institution::execute::cleanup_pending_institution_create::<T>(
                        proposal_id,
                        &action,
                        true,
                    );
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

        pub(crate) fn ensure_unique_admins(admins: &DuoqianAdminsOf<T>) -> Result<(), DispatchError> {
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
        /// 与 `offchain-transaction::settlement::pubkey_from_accountid` 语义对齐。
        pub fn pubkey_from_accountid(acc: &T::AccountId) -> Result<Sr25519Public, Error<T>> {
            let encoded = acc.encode();
            if encoded.len() < 32 {
                return Err(Error::<T>::MalformedSignature);
            }
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&encoded[..32]);
            Ok(Sr25519Public::from_raw(arr))
        }

        pub(crate) fn ensure_admin_config(
            who: &T::AccountId,
            admin_count: u32,
            duoqian_admins: &DuoqianAdminsOf<T>,
            threshold: u32,
        ) -> DispatchResult {
            ensure!(T::MaxAdmins::get() >= 2, Error::<T>::InvalidRuntimeConfig);
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
            Self::ensure_unique_admins(duoqian_admins)?;
            ensure!(
                duoqian_admins.iter().any(|admin| admin == who),
                Error::<T>::PermissionDenied
            );
            Ok(())
        }

        pub(crate) fn create_pending_admin_subject_for_proposal(
            proposal_id: u64,
            institution_id: voting_engine::InstitutionPalletId,
            kind: admins_change::AdminSubjectKind,
            admins: &DuoqianAdminsOf<T>,
            threshold: u32,
            creator: &T::AccountId,
        ) -> DispatchResult {
            admins_change::Pallet::<T>::create_pending_subject_for_proposal(
                proposal_id,
                crate::MODULE_TAG,
                institution_id,
                voting_engine::internal_vote::ORG_DUOQIAN,
                kind,
                admins.iter().cloned().collect(),
                threshold,
                creator.clone(),
            )
        }

        pub(crate) fn activate_admin_subject(
            proposal_id: u64,
            institution_id: voting_engine::InstitutionPalletId,
        ) -> DispatchResult {
            admins_change::Pallet::<T>::activate_subject_for_proposal(
                proposal_id,
                crate::MODULE_TAG,
                institution_id,
            )
        }

        pub(crate) fn remove_pending_admin_subject(
            proposal_id: u64,
            institution_id: voting_engine::InstitutionPalletId,
        ) {
            let _ = admins_change::Pallet::<T>::remove_pending_subject_for_proposal(
                proposal_id,
                crate::MODULE_TAG,
                institution_id,
            );
        }

        pub(crate) fn close_admin_subject(
            proposal_id: u64,
            institution_id: voting_engine::InstitutionPalletId,
        ) -> DispatchResult {
            admins_change::Pallet::<T>::close_subject_for_proposal(
                proposal_id,
                crate::MODULE_TAG,
                institution_id,
            )
        }

        /// 从任意多签账户反查其管理员主体的 InstitutionPalletId。
        ///
        /// - 个人多签：account_to_institution_id(personal_address)
        /// - SFID 机构任意账户：sfid_id_to_institution_id(sfid_id)(2026-05-03 整改后直接用 sfid_id 派生)
        /// - 既非 personal 也非 SFID 注册的孤立 DuoqianAccounts：账户自身派生
        pub fn resolve_admin_subject_for_account(
            account: &T::AccountId,
        ) -> Option<InstitutionPalletId> {
            if PersonalDuoqianInfo::<T>::contains_key(account) {
                return Some(account_to_institution_id(account));
            }

            if let Some(registered) = AddressRegisteredSfid::<T>::get(account) {
                return sfid_id_to_institution_id(registered.sfid_id.as_slice())
                    .or_else(|| Some(account_to_institution_id(account)));
            }

            DuoqianAccounts::<T>::contains_key(account).then(|| account_to_institution_id(account))
        }

        // account_names_payload_from_initial_accounts 已迁至
        // institution::accounts (上一轮拆分遗留的副本于 2026-05-03 删除)。

        /// 中文注释:把批量 register 入口的 account_names 抽成验签 payload。
        pub(crate) fn account_names_payload_from_names(
            account_names: &InstitutionAccountNamesOf<T>,
        ) -> Result<Vec<Vec<u8>>, DispatchError> {
            let mut names: Vec<Vec<u8>> = Vec::with_capacity(account_names.len());
            for account_name in account_names.iter() {
                ensure!(!account_name.is_empty(), Error::<T>::EmptyAccountName);
                names.push(account_name.as_slice().to_vec());
            }
            Ok(names)
        }


        // 投票回调执行体已迁移：
        // - ACTION_CREATE_PERSONAL / ACTION_CLOSE 共用部分 → crate::execute
        // - ACTION_CREATE_INSTITUTION 与 cleanup → crate::institution::execute
    }
}

// ──── 投票终态回调:把已通过的多签创建/关闭提案落地到链上 ────
//
// Phase 2 整改后业务模块不再自行处理投票,提案通过(或否决)由投票引擎
// 通过 [`voting_engine::InternalVoteResultCallback`] 广播回来。
// 本 Executor:
// - 按 `MODULE_TAG + ACTION_CREATE_PERSONAL / ACTION_CLOSE` 前缀认领本模块提案;
// - `approved = true` → 分派到 `execute_create` / `execute_close`;执行失败
//   发事件,不回滚投票(提案保留 PASSED,可用 cleanup_rejected_proposal 或
//   手动重试处理);
// - `approved = false` → 清理 Pending 存储(DuoqianAccounts / PendingCloseProposal),
//   释放地址占用。
pub struct InternalVoteExecutor<T>(core::marker::PhantomData<T>);

impl<T: pallet::Config> InternalVoteResultCallback for InternalVoteExecutor<T> {
    fn on_internal_vote_finalized(
        proposal_id: u64,
        approved: bool,
    ) -> Result<ProposalExecutionOutcome, sp_runtime::DispatchError> {
        use frame_support::storage::{with_transaction, TransactionOutcome};
        let raw = match voting_engine::Pallet::<T>::get_proposal_data(proposal_id) {
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
                ACTION_CREATE_PERSONAL => {
                    let action = CreateDuoqianAction::<T::AccountId, BalanceOf<T>>::decode(
                        &mut &raw[tag.len() + 1..],
                    )
                    .map_err(|_| pallet::Error::<T>::ProposalActionNotFound)?;
                    let exec_result = with_transaction(
                        || match crate::execute::execute_create_with_finalizer::<T>(
                            proposal_id,
                            &action,
                            true,
                        ) {
                            Ok(()) => TransactionOutcome::Commit(Ok(())),
                            Err(e) => TransactionOutcome::Rollback(Err(e)),
                        },
                    );
                    if exec_result.is_err() {
                        pallet::Pallet::<T>::deposit_event(
                            pallet::Event::<T>::CreateExecutionFailed {
                                proposal_id,
                                duoqian_address: action.duoqian_address,
                            },
                        );
                        return Ok(ProposalExecutionOutcome::RetryableFailed);
                    }
                    return Ok(ProposalExecutionOutcome::Executed);
                }
                ACTION_CREATE_INSTITUTION => {
                    let action = CreateInstitutionActionOf::<T>::decode(&mut &raw[tag.len() + 1..])
                        .map_err(|_| pallet::Error::<T>::ProposalActionNotFound)?;
                    let exec_result = with_transaction(|| {
                        match crate::institution::execute::execute_create_institution_with_finalizer::<T>(
                            proposal_id,
                            &action,
                            true,
                        ) {
                            Ok(()) => TransactionOutcome::Commit(Ok(())),
                            Err(e) => TransactionOutcome::Rollback(Err(e)),
                        }
                    });
                    if exec_result.is_err() {
                        pallet::Pallet::<T>::deposit_event(
                            pallet::Event::<T>::InstitutionCreateExecutionFailed {
                                proposal_id,
                                sfid_id: action.sfid_id,
                                main_address: action.main_address,
                            },
                        );
                        return Ok(ProposalExecutionOutcome::RetryableFailed);
                    }
                    return Ok(ProposalExecutionOutcome::Executed);
                }
                ACTION_CLOSE => {
                    let action =
                        CloseDuoqianAction::<T::AccountId>::decode(&mut &raw[tag.len() + 1..])
                            .map_err(|_| pallet::Error::<T>::ProposalActionNotFound)?;
                    let exec_result = with_transaction(
                        || match crate::execute::execute_close_with_finalizer::<T>(
                            proposal_id,
                            &action,
                            true,
                        ) {
                            Ok(()) => TransactionOutcome::Commit(Ok(())),
                            Err(e) => TransactionOutcome::Rollback(Err(e)),
                        },
                    );
                    if exec_result.is_err() {
                        pallet::Pallet::<T>::deposit_event(
                            pallet::Event::<T>::CloseExecutionFailed {
                                proposal_id,
                                duoqian_address: action.duoqian_address,
                            },
                        );
                        return Ok(ProposalExecutionOutcome::RetryableFailed);
                    }
                    return Ok(ProposalExecutionOutcome::Executed);
                }
                _ => return Ok(ProposalExecutionOutcome::Ignored),
            }
        } else {
            // 否决:清理 Pending 记录释放地址锁定。
            match action_byte {
                ACTION_CREATE_PERSONAL => {
                    if let Ok(action) = CreateDuoqianAction::<T::AccountId, BalanceOf<T>>::decode(
                        &mut &raw[tag.len() + 1..],
                    ) {
                        crate::personal::execute::cleanup_pending_personal_create::<T>(
                            proposal_id,
                            &action,
                            true,
                        );
                    }
                }
                ACTION_CREATE_INSTITUTION => {
                    if let Ok(action) =
                        CreateInstitutionActionOf::<T>::decode(&mut &raw[tag.len() + 1..])
                    {
                        crate::institution::execute::cleanup_pending_institution_create::<T>(
                            proposal_id,
                            &action,
                            true,
                        );
                    }
                }
                ACTION_CLOSE => {
                    if let Ok(action) =
                        CloseDuoqianAction::<T::AccountId>::decode(&mut &raw[tag.len() + 1..])
                    {
                        PendingCloseProposal::<T>::remove(&action.duoqian_address);
                    }
                }
                _ => {}
            }
        }
        Ok(ProposalExecutionOutcome::Executed)
    }

    fn on_execution_failed_terminal(proposal_id: u64) -> DispatchResult {
        let raw = match voting_engine::Pallet::<T>::get_proposal_data(proposal_id) {
            Some(raw) if raw.starts_with(crate::MODULE_TAG) => raw,
            _ => return Ok(()),
        };
        let tag = crate::MODULE_TAG;
        ensure!(
            raw.len() > tag.len(),
            pallet::Error::<T>::ProposalActionNotFound
        );
        match raw[tag.len()] {
            ACTION_CREATE_PERSONAL => {
                let action = CreateDuoqianAction::<T::AccountId, BalanceOf<T>>::decode(
                    &mut &raw[tag.len() + 1..],
                )
                .map_err(|_| pallet::Error::<T>::ProposalActionNotFound)?;
                crate::personal::execute::cleanup_pending_personal_create::<T>(
                    proposal_id,
                    &action,
                    false,
                );
            }
            ACTION_CREATE_INSTITUTION => {
                let action = CreateInstitutionActionOf::<T>::decode(&mut &raw[tag.len() + 1..])
                    .map_err(|_| pallet::Error::<T>::ProposalActionNotFound)?;
                crate::institution::execute::cleanup_pending_institution_create::<T>(
                    proposal_id,
                    &action,
                    false,
                );
            }
            ACTION_CLOSE => {
                let action = CloseDuoqianAction::<T::AccountId>::decode(&mut &raw[tag.len() + 1..])
                    .map_err(|_| pallet::Error::<T>::ProposalActionNotFound)?;
                PendingCloseProposal::<T>::remove(&action.duoqian_address);
            }
            _ => {}
        }
        Ok(())
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
    use voting_engine::internal_vote::ORG_DUOQIAN;

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
        pub type VotingEngine = voting_engine;

        #[runtime::pallet_index(3)]
        pub type Duoqian = pallet;

        #[runtime::pallet_index(4)]
        pub type AdminsChange = admins_change;
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

    // ADR-008 step2b mock：模拟链端 (province, admin_pubkey) 双层匹配。
    // 默认行为(无注入):仅按"signature == register-ok + province 非空 + admin_pubkey 非全 0"形状校验。
    // 测试可通过 ACTIVATED_ROSTER thread-local 注入"已 activate 的 (province, admin)"集合,
    // 一旦注入,verifier 转入严格模式:必须命中集合才放行(对应链端 ShengSigningPubkey 查到的语义)。
    thread_local! {
        static ACTIVATED_ROSTER: core::cell::RefCell<Option<sp_std::vec::Vec<(sp_std::vec::Vec<u8>, [u8; 32])>>>
            = const { core::cell::RefCell::new(None) };
    }

    fn set_activated_roster(entries: sp_std::vec::Vec<(sp_std::vec::Vec<u8>, [u8; 32])>) {
        ACTIVATED_ROSTER.with(|r| *r.borrow_mut() = Some(entries));
    }

    fn clear_activated_roster() {
        ACTIVATED_ROSTER.with(|r| *r.borrow_mut() = None);
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
            institution_name: &AccountNameOf<Test>,
            account_names: &[Vec<u8>],
            nonce: &RegisterNonceOf<Test>,
            signature: &RegisterSignatureOf<Test>,
            province: &[u8],
            signer_admin_pubkey: &[u8; 32],
        ) -> bool {
            if institution_name.is_empty()
                || account_names.is_empty()
                || account_names.iter().any(|name| name.is_empty())
                || nonce.is_empty()
                || province.is_empty()
                || signer_admin_pubkey == &[0u8; 32]
                || signature.as_slice() != b"register-ok"
            {
                return false;
            }
            // 严格模式:若注入了 ACTIVATED_ROSTER,则 (province, admin_pubkey) 必须命中。
            let strict_hit = ACTIVATED_ROSTER.with(|r| match &*r.borrow() {
                Some(entries) => Some(
                    entries
                        .iter()
                        .any(|(p, k)| p.as_slice() == province && k == signer_admin_pubkey),
                ),
                None => None,
            });
            match strict_hit {
                Some(hit) => hit,
                None => true,
            }
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

    pub struct TestInstitutionAsset;
    impl institution_asset::InstitutionAsset<AccountId32> for TestInstitutionAsset {
        fn can_spend(
            source: &AccountId32,
            action: institution_asset::InstitutionAssetAction,
        ) -> bool {
            if !matches!(
                action,
                institution_asset::InstitutionAssetAction::DuoqianCloseExecute
            ) {
                return true;
            }
            DENIED_CLOSE_SOURCE.with(|blocked| blocked.borrow().as_ref() != Some(source))
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

    /// 测试用 InternalAdminProvider：从 admins-change 统一主体表读取管理员。
    pub struct TestInternalAdminProvider;
    impl voting_engine::InternalAdminProvider<AccountId32> for TestInternalAdminProvider {
        fn is_internal_admin(org: u8, institution: InstitutionPalletId, who: &AccountId32) -> bool {
            if org != ORG_DUOQIAN {
                return false;
            }
            admins_change::Pallet::<Test>::is_active_subject_admin(org, institution, who)
        }

        fn get_admin_list(org: u8, institution: InstitutionPalletId) -> Option<Vec<AccountId32>> {
            if org != ORG_DUOQIAN {
                return None;
            }
            admins_change::Pallet::<Test>::active_subject_admins(org, institution)
        }

        fn is_pending_internal_admin(
            org: u8,
            institution: InstitutionPalletId,
            who: &AccountId32,
        ) -> bool {
            if org != ORG_DUOQIAN {
                return false;
            }
            admins_change::Pallet::<Test>::is_pending_subject_admin_for_snapshot(
                org,
                institution,
                who,
            )
        }

        fn get_pending_admin_list(
            org: u8,
            institution: InstitutionPalletId,
        ) -> Option<Vec<AccountId32>> {
            if org != ORG_DUOQIAN {
                return None;
            }
            admins_change::Pallet::<Test>::pending_subject_admins_for_snapshot(org, institution)
        }
    }

    pub struct TestInternalAdminCountProvider;
    impl voting_engine::InternalAdminCountProvider for TestInternalAdminCountProvider {
        fn admin_count(org: u8, institution: InstitutionPalletId) -> Option<u32> {
            if org != ORG_DUOQIAN {
                return None;
            }
            admins_change::Pallet::<Test>::active_subject_admin_count(org, institution)
        }
    }

    /// 测试用 InternalThresholdProvider：从 admins-change 统一主体表读取阈值。
    pub struct TestInternalThresholdProvider;
    impl voting_engine::InternalThresholdProvider for TestInternalThresholdProvider {
        fn is_known_subject(org: u8, institution: InstitutionPalletId) -> bool {
            if org != ORG_DUOQIAN {
                return false;
            }
            admins_change::Pallet::<Test>::active_subject_exists(org, institution)
        }

        fn is_known_pending_subject(org: u8, institution: InstitutionPalletId) -> bool {
            if org != ORG_DUOQIAN {
                return false;
            }
            admins_change::Pallet::<Test>::pending_subject_exists_for_snapshot(org, institution)
        }

        fn pass_threshold(org: u8, institution: InstitutionPalletId) -> Option<u32> {
            if org != ORG_DUOQIAN {
                return voting_engine::internal_vote::fixed_governance_pass_threshold(org);
            }
            admins_change::Pallet::<Test>::active_subject_threshold(org, institution)
        }

        fn pending_pass_threshold(org: u8, institution: InstitutionPalletId) -> Option<u32> {
            if org != ORG_DUOQIAN {
                return None;
            }
            admins_change::Pallet::<Test>::pending_subject_threshold_for_snapshot(org, institution)
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
        // Phase 2:挂上本模块 Executor,提案通过后自动 execute_create / execute_close。
        type InternalVoteResultCallback = crate::InternalVoteExecutor<Test>;
        type InternalAdminProvider = TestInternalAdminProvider;
        type InternalAdminCountProvider = TestInternalAdminCountProvider;
        type InternalThresholdProvider = TestInternalThresholdProvider;
        type MaxAdminsPerInstitution = ConstU32<64>;
        type MaxProposalDataLen = ConstU32<4096>;
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

    impl pallet::Config for Test {
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
        type MinCloseBalance = ConstU128<121>;
        type WeightInfo = ();
    }

    impl admins_change::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type MaxAdminsPerInstitution = ConstU32<64>;
        type InternalVoteEngine = voting_engine::Pallet<Test>;
        type WeightInfo = ();
    }

    fn relayer() -> AccountId32 {
        AccountId32::new([0x55; 32])
    }

    /// 从固定 seed 派生 sr25519 keypair:公钥即为 AccountId32。
    /// 测试中管理员既能作为链上 origin(签发 `internal_vote`),也能对任意
    /// payload 做可验证 sr25519 签名,供签名相关回归测试复用。
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

    /// 测试辅助:走投票引擎公开 `internal_vote` extrinsic,
    /// 让 `admins` 的前 `take` 个成员各投一张赞成票。
    ///
    /// 业务模块不持有投票 call，通过后由投票引擎通过
    /// [`InternalVoteExecutor`] 自动触发 execute_create。
    fn finalize_with(
        _submitter: AccountId32,
        proposal_id: u64,
        _duoqian_address: &AccountId32,
        _creator: &AccountId32,
        admins: &DuoqianAdminsOf<Test>,
        _pairs: &[sr25519::Pair],
        _threshold: u32,
        _amount: u128,
        take: usize,
    ) -> frame_support::dispatch::DispatchResult {
        for admin in admins.iter().take(take) {
            VotingEngine::internal_vote(RuntimeOrigin::signed(admin.clone()), proposal_id, true)?;
        }
        Ok(())
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
        let account_names: InstitutionAccountNamesOf<Test> = vec![account_name.clone()]
            .try_into()
            .expect("account names should fit");
        // ADR-008 step2b：province 必填,signer_admin_pubkey 必填非全 0。mock verifier 仅做形态校验。
        // 中文注释：注册验签 payload 只包含 SFID号、机构名称、账户名称列表和 SFID 签发凭证。
        assert_ok!(Duoqian::register_sfid_institution(
            RuntimeOrigin::signed(relayer()),
            sfid.clone(),
            account_name.clone(),
            account_names,
            register_nonce,
            signature,
            b"LN".to_vec(),
            [1u8; 32],
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

    /// 个人多签测试 setup: 返回 (account_name, duoqian_address, admins, pairs)。
    /// 其中 admins[0] 是 creator，duoqian_address 由 personal 派生公式生成。
    fn setup_personal_duoqian(
        name_tag: &str,
        admin_seeds: &[u8],
    ) -> (
        AccountNameOf<Test>,
        AccountId32,
        DuoqianAdminsOf<Test>,
        Vec<sr25519::Pair>,
    ) {
        let account_name: AccountNameOf<Test> = format!("Personal {}", name_tag)
            .as_bytes()
            .to_vec()
            .try_into()
            .expect("account_name should fit");
        let (admins, pairs) = make_admins_keyed(admin_seeds);
        let creator = admins[0].clone();
        let duoqian_address =
            Duoqian::derive_personal_duoqian_address(&creator, account_name.as_slice())
                .expect("personal duoqian address derivation");
        (account_name, duoqian_address, admins, pairs)
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let mut storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("system genesis build should succeed");

        pallet::GenesisConfig::<Test>::default()
            .assimilate_storage(&mut storage)
            .expect("duoqian genesis build should succeed");

        admins_change::GenesisConfig::<Test>::default()
            .assimilate_storage(&mut storage)
            .expect("admins-change genesis build should succeed");

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
        voting_engine::Pallet::<Test>::next_proposal_id().saturating_sub(1)
    }

    fn make_admins(seeds: &[u8]) -> DuoqianAdminsOf<Test> {
        seeds
            .iter()
            .map(|s| admin(*s))
            .collect::<Vec<_>>()
            .try_into()
            .expect("admins should fit")
    }

    fn institution_accounts(items: Vec<(&[u8], u128)>) -> InstitutionInitialAccountsOf<Test> {
        items
            .into_iter()
            .map(|(name, amount)| InstitutionInitialAccount {
                account_name: name.to_vec().try_into().expect("account name should fit"),
                amount,
            })
            .collect::<Vec<_>>()
            .try_into()
            .expect("institution accounts should fit")
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
            let account_names: InstitutionAccountNamesOf<Test> = vec![account_name.clone()]
                .try_into()
                .expect("account names should fit");
            assert_noop!(
                Duoqian::register_sfid_institution(
                    RuntimeOrigin::signed(admin(1)),
                    sfid,
                    account_name.clone(),
                    account_names,
                    register_nonce,
                    bad_signature,
                    b"LN".to_vec(),
                    [1u8; 32],
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
            let (account_name, duoqian_address, admins, pairs) =
                setup_personal_duoqian("B001", &[1, 2, 3]);

            // Tx 1:发起创建提案
            assert_ok!(Duoqian::propose_create_personal(
                RuntimeOrigin::signed(admins[0].clone()),
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
                3,
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
            let (account_name, duoqian_address, admins, pairs) =
                setup_personal_duoqian("C001", &[1, 2, 3]);
            let beneficiary = admin(4);

            // 先创建(走 finalize_create 离线聚合路径)
            assert_ok!(Duoqian::propose_create_personal(
                RuntimeOrigin::signed(admins[0].clone()),
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
                3,
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

            // Phase 2:关闭走投票引擎公开 internal_vote,通过后由 Executor 自动 execute_close。
            // 2026-05-03 整改:关闭提案 threshold = admins.len(),需全员投票。
            for admin in admins.iter() {
                assert_ok!(VotingEngine::internal_vote(
                    RuntimeOrigin::signed(admin.clone()),
                    close_pid,
                    true
                ));
            }

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
            let account_name: AccountNameOf<Test> = b"Personal D001"
                .to_vec()
                .try_into()
                .expect("account_name should fit");
            let admins = make_admins(&[1, 2, 3]);

            // admin(4) 不在管理员列表中
            assert_noop!(
                Duoqian::propose_create_personal(
                    RuntimeOrigin::signed(admin(4)),
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
    fn cannot_close_pending_account() {
        new_test_ext().execute_with(|| {
            let (account_name, duoqian_address, admins, _pairs) =
                setup_personal_duoqian("F001", &[1, 2, 3]);

            // propose create 但不投票通过
            assert_ok!(Duoqian::propose_create_personal(
                RuntimeOrigin::signed(admins[0].clone()),
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
            let (account_name, duoqian_address, admins, pairs) =
                setup_personal_duoqian("F002", &[1, 2, 3]);

            assert_ok!(Duoqian::propose_create_personal(
                RuntimeOrigin::signed(admins[0].clone()),
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
                3,
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
            let account_name: AccountNameOf<Test> = b"Personal G001"
                .to_vec()
                .try_into()
                .expect("account_name should fit");
            let admins: DuoqianAdminsOf<Test> = vec![admin(1), admin(1), admin(2)]
                .try_into()
                .expect("should fit");

            assert_noop!(
                Duoqian::propose_create_personal(
                    RuntimeOrigin::signed(admin(1)),
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
            let account_name: AccountNameOf<Test> = b"Personal H001"
                .to_vec()
                .try_into()
                .expect("account_name should fit");
            let admins = make_admins(&[1, 2, 3]);

            assert_noop!(
                Duoqian::propose_create_personal(
                    RuntimeOrigin::signed(admin(1)),
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
            let (account_name, duoqian_address, admins, pairs) =
                setup_personal_duoqian("I001", &[1, 2]);

            // 创建并激活(走 finalize_create)
            assert_ok!(Duoqian::propose_create_personal(
                RuntimeOrigin::signed(admins[0].clone()),
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

    /// 验证 reserve 资金模型(2026-05-03):
    /// 提案创建时已 reserve(amount + fee),slash 发起人剩余 free 余额不影响执行,
    /// 投票通过后 unreserve + transfer 仍可成功,DuoqianAccounts 转 Active。
    #[test]
    fn reserve_model_protects_execution_against_balance_drain() {
        new_test_ext().execute_with(|| {
            let (account_name, duoqian_address, admins, pairs) =
                setup_personal_duoqian("J001", &[1, 2]);

            assert_ok!(Duoqian::propose_create_personal(
                RuntimeOrigin::signed(admins[0].clone()),
                account_name.clone(),
                2,
                admins.clone(),
                2,
                1_000,
            ));
            let pid = last_proposal_id();

            // 提案创建后 reserve(amount + fee = 1_010),发起人 reserved = 1_010
            assert_eq!(Balances::reserved_balance(&admins[0]), 1_010);

            // 排干 admins[0] 的剩余 free 余额,模拟"投票期间发起人花光 free"
            let free_after_reserve = Balances::free_balance(&admins[0]);
            let _ = Balances::slash(&admins[0], free_after_reserve);
            // free 可能被 ED 保留 1 单位,但绝不应仍有大额可用
            assert!(Balances::free_balance(&admins[0]) < 100);
            // reserved 仍然完好(slash 优先扣 free)
            assert_eq!(Balances::reserved_balance(&admins[0]), 1_010);

            // 达阈值 → callback execute_create_with_finalizer
            // unreserve(1_010) → withdraw fee 10 → transfer amount 1_000
            // 资金从 reserved 中划转,不依赖 free,所以执行成功
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
                DuoqianAccounts::<Test>::get(&duoqian_address)
                    .expect("account should be active")
                    .status,
                DuoqianStatus::Active
            );
            assert_eq!(Balances::free_balance(&duoqian_address), 1_000);
            // PendingPersonalCreate 在执行成功后清理。
            assert!(PendingPersonalCreate::<Test>::get(pid).is_none());
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

    #[test]
    fn institution_create_reserves_and_activates_all_accounts() {
        new_test_ext().execute_with(|| {
            let sfid: SfidIdOf<Test> = b"SFR-AH001-ZG1Y-883241719-20260428"
                .to_vec()
                .try_into()
                .expect("sfid should fit");
            let institution_name: AccountNameOf<Test> = "测试清算行"
                .as_bytes()
                .to_vec()
                .try_into()
                .expect("institution name should fit");
            let accounts = institution_accounts(vec![
                (RESERVED_NAME_MAIN, 2_000),
                (RESERVED_NAME_FEE, 500),
                ("运营账户".as_bytes(), 300),
            ]);
            let (admins, pairs) = make_admins_keyed(&[1, 2, 3]);
            let before_free = Balances::free_balance(&admins[0]);

            assert_ok!(Duoqian::propose_create_institution(
                RuntimeOrigin::signed(admins[0].clone()),
                sfid.clone(),
                institution_name.clone(),
                accounts.clone(),
                3,
                admins.clone(),
                2,
                b"institution-create-1".to_vec().try_into().unwrap(),
                b"register-ok".to_vec().try_into().unwrap(),
                b"AH".to_vec(),
                [1u8; 32],
            ));

            let pid = last_proposal_id();
            let main_address = Pallet::<Test>::derive_institution_address(
                sfid.as_slice(),
                InstitutionAccountRole::Main,
            )
            .expect("main should derive");
            let fee_address = Pallet::<Test>::derive_institution_address(
                sfid.as_slice(),
                InstitutionAccountRole::Fee,
            )
            .expect("fee should derive");
            let custom_address = Pallet::<Test>::derive_institution_address(
                sfid.as_slice(),
                InstitutionAccountRole::Named("运营账户".as_bytes()),
            )
            .expect("custom should derive");

            assert_eq!(Balances::reserved_balance(&admins[0]), 2_810);
            assert_eq!(
                Institutions::<Test>::get(&sfid).map(|i| i.status),
                Some(InstitutionLifecycleStatus::Pending)
            );
            assert_eq!(
                DuoqianAccounts::<Test>::get(&main_address).map(|a| a.status),
                Some(DuoqianStatus::Pending)
            );

            assert_ok!(finalize_with(
                admins[0].clone(),
                pid,
                &main_address,
                &admins[0],
                &admins,
                &pairs,
                2,
                2_800,
                3,
            ));

            assert_eq!(Balances::reserved_balance(&admins[0]), 0);
            assert_eq!(Balances::free_balance(&admins[0]), before_free - 2_810);
            assert_eq!(Balances::free_balance(&main_address), 2_000);
            assert_eq!(Balances::free_balance(&fee_address), 500);
            assert_eq!(Balances::free_balance(&custom_address), 300);
            assert_eq!(
                Institutions::<Test>::get(&sfid).map(|i| i.status),
                Some(InstitutionLifecycleStatus::Active)
            );
            assert_eq!(
                InstitutionAccounts::<Test>::get(
                    &sfid,
                    AccountNameOf::<Test>::try_from(RESERVED_NAME_MAIN.to_vec()).unwrap()
                )
                .map(|a| a.status),
                Some(InstitutionLifecycleStatus::Active)
            );
        });
    }

    #[test]
    fn institution_create_rejection_unreserves_and_cleans_indexes() {
        new_test_ext().execute_with(|| {
            let sfid: SfidIdOf<Test> = b"SFR-AH001-ZG1Y-REJECT-20260428"
                .to_vec()
                .try_into()
                .expect("sfid should fit");
            let institution_name: AccountNameOf<Test> = "拒绝清算行"
                .as_bytes()
                .to_vec()
                .try_into()
                .expect("institution name should fit");
            let accounts =
                institution_accounts(vec![(RESERVED_NAME_MAIN, 1_000), (RESERVED_NAME_FEE, 500)]);
            let admins = make_admins(&[1, 2, 3]);
            let before_free = Balances::free_balance(&admins[0]);

            assert_ok!(Duoqian::propose_create_institution(
                RuntimeOrigin::signed(admins[0].clone()),
                sfid.clone(),
                institution_name,
                accounts,
                3,
                admins.clone(),
                2,
                b"institution-create-2".to_vec().try_into().unwrap(),
                b"register-ok".to_vec().try_into().unwrap(),
                b"AH".to_vec(),
                [1u8; 32],
            ));
            let pid = last_proposal_id();
            let main_address = Pallet::<Test>::derive_institution_address(
                sfid.as_slice(),
                InstitutionAccountRole::Main,
            )
            .expect("main should derive");
            let main_name: AccountNameOf<Test> = RESERVED_NAME_MAIN.to_vec().try_into().unwrap();

            assert_eq!(Balances::reserved_balance(&admins[0]), 1_510);
            // 2026-05-03 整改:创建提案 threshold = admins.len()=3,
            // 1 票反对即可让"剩余赞成"无法达到阈值,提前否决。
            assert_ok!(VotingEngine::internal_vote(
                RuntimeOrigin::signed(admins[0].clone()),
                pid,
                false
            ));

            assert_eq!(Balances::reserved_balance(&admins[0]), 0);
            assert_eq!(Balances::free_balance(&admins[0]), before_free);
            assert!(PendingInstitutionCreate::<Test>::get(pid).is_none());
            assert!(Institutions::<Test>::get(&sfid).is_none());
            assert!(DuoqianAccounts::<Test>::get(&main_address).is_none());
            assert!(SfidRegisteredAddress::<Test>::get(&sfid, &main_name).is_none());
            assert!(AddressRegisteredSfid::<Test>::get(&main_address).is_none());
        });
    }

    #[test]
    fn institution_create_requires_main_and_fee_accounts() {
        new_test_ext().execute_with(|| {
            let sfid: SfidIdOf<Test> = b"SFR-AH001-ZG1Y-MISSING-20260428"
                .to_vec()
                .try_into()
                .expect("sfid should fit");
            let institution_name: AccountNameOf<Test> = "缺主账户机构"
                .as_bytes()
                .to_vec()
                .try_into()
                .expect("institution name should fit");
            let accounts =
                institution_accounts(vec![(RESERVED_NAME_FEE, 500), ("运营账户".as_bytes(), 300)]);
            let admins = make_admins(&[1, 2, 3]);

            assert_noop!(
                Duoqian::propose_create_institution(
                    RuntimeOrigin::signed(admins[0].clone()),
                    sfid,
                    institution_name,
                    accounts,
                    3,
                    admins,
                    2,
                    b"institution-create-3".to_vec().try_into().unwrap(),
                    b"register-ok".to_vec().try_into().unwrap(),
                    b"AH".to_vec(),
                    [1u8; 32],
                ),
                Error::<Test>::MissingMainAccount
            );
        });
    }

    #[test]
    fn institution_create_rejects_account_initial_amount_below_minimum() {
        new_test_ext().execute_with(|| {
            let sfid: SfidIdOf<Test> = b"SFR-AH001-ZG1Y-LOW-20260428"
                .to_vec()
                .try_into()
                .expect("sfid should fit");
            let institution_name: AccountNameOf<Test> = "低余额机构"
                .as_bytes()
                .to_vec()
                .try_into()
                .expect("institution name should fit");
            let accounts =
                institution_accounts(vec![(RESERVED_NAME_MAIN, 110), (RESERVED_NAME_FEE, 500)]);
            let admins = make_admins(&[1, 2, 3]);

            assert_noop!(
                Duoqian::propose_create_institution(
                    RuntimeOrigin::signed(admins[0].clone()),
                    sfid,
                    institution_name,
                    accounts,
                    3,
                    admins,
                    2,
                    b"institution-create-4".to_vec().try_into().unwrap(),
                    b"register-ok".to_vec().try_into().unwrap(),
                    b"AH".to_vec(),
                    [1u8; 32],
                ),
                Error::<Test>::AccountInitialAmountBelowMinimum
            );
        });
    }

    // ──── ADR-008 step2b：register_sfid_institution 按 (province, admin_pubkey) 双层匹配 ────

    /// 辅助:用任意 (province, admin_pubkey) 调 register_sfid_institution。
    fn register_with_admin(
        tag: &str,
        province: &[u8],
        admin_pubkey: [u8; 32],
    ) -> sp_runtime::DispatchResult {
        let sfid: SfidIdOf<Test> = format!("GFR-AH001-CB0C-{}-20260502", tag)
            .as_bytes()
            .to_vec()
            .try_into()
            .expect("sfid id should fit");
        let account_name: AccountNameOf<Test> = format!("Inst-{}", tag)
            .as_bytes()
            .to_vec()
            .try_into()
            .expect("account_name should fit");
        let register_nonce: RegisterNonceOf<Test> = format!("nonce-{}", tag)
            .into_bytes()
            .try_into()
            .expect("register nonce should fit");
        let signature: RegisterSignatureOf<Test> = b"register-ok"
            .to_vec()
            .try_into()
            .expect("register signature should fit");
        let account_names: InstitutionAccountNamesOf<Test> = vec![account_name.clone()]
            .try_into()
            .expect("account names should fit");
        Duoqian::register_sfid_institution(
            RuntimeOrigin::signed(relayer()),
            sfid,
            account_name.clone(),
            account_names,
            register_nonce,
            signature,
            province.to_vec(),
            admin_pubkey,
        )
    }

    /// step2b T1:main admin 派生签名 → 验签通过 → register 成功。
    #[test]
    fn register_with_main_admin_signature_succeeds() {
        new_test_ext().execute_with(|| {
            let main_admin: [u8; 32] = [11u8; 32];
            set_activated_roster(vec![(b"AH".to_vec(), main_admin)]);
            assert_ok!(register_with_admin("MAIN", b"AH", main_admin));
            clear_activated_roster();
        });
    }

    /// step2b T2:backup admin(同省第二把已 activate 的签名密钥)派生签名 → 验签通过。
    #[test]
    fn register_with_backup_admin_signature_succeeds() {
        new_test_ext().execute_with(|| {
            let main_admin: [u8; 32] = [11u8; 32];
            let backup_admin: [u8; 32] = [22u8; 32];
            set_activated_roster(vec![
                (b"AH".to_vec(), main_admin),
                (b"AH".to_vec(), backup_admin),
            ]);
            // 用 backup_admin 注册一个机构 → 必须 ok(同省 main + backup 各自独立签名密钥)。
            assert_ok!(register_with_admin("BACKUP", b"AH", backup_admin));
            clear_activated_roster();
        });
    }

    /// step2b T3:admin pubkey 不在 ShengAdmins 花名册 → 验签失败。
    #[test]
    fn register_with_admin_not_in_roster_rejected() {
        new_test_ext().execute_with(|| {
            // 花名册仅 main_admin,outsider 不在。
            let main_admin: [u8; 32] = [11u8; 32];
            set_activated_roster(vec![(b"AH".to_vec(), main_admin)]);
            let outsider: [u8; 32] = [99u8; 32];
            assert_noop!(
                register_with_admin("OUTSIDER", b"AH", outsider),
                Error::<Test>::InvalidSfidInstitutionSignature
            );
            clear_activated_roster();
        });
    }

    /// step2b T4:admin 在花名册但 ShengSigningPubkey 尚未 activate → 验签失败。
    /// 注:mock 把"未 activate"等价于"不在 activated_roster",链端真实 verifier 走
    /// `ShengSigningPubkey::get((province, admin))` → None → reject。
    #[test]
    fn register_signing_not_activated_for_admin_rejected() {
        new_test_ext().execute_with(|| {
            // 注入空 roster:即"链上花名册存在但 SigningPubkey 全部未 activate"。
            set_activated_roster(vec![]);
            let admin_pubkey: [u8; 32] = [11u8; 32];
            assert_noop!(
                register_with_admin("NOT-ACTIVATED", b"AH", admin_pubkey),
                Error::<Test>::InvalidSfidInstitutionSignature
            );
            clear_activated_roster();
        });
    }

    /// step2b T5:province 改必填后,空省直接 EmptyProvince(verifier 之前)。
    #[test]
    fn register_with_empty_province_rejected() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                register_with_admin("EMPTY-PROV", b"", [1u8; 32]),
                Error::<Test>::EmptyProvince
            );
        });
    }

    // ──── Step 1 离线 QR 聚合专项测试 ────

    /// finalize_create 签名数不足阈值时必须拒绝。

    /// 同一 admin 在同一批签名里重复出现必须拒绝。

    /// 被篡改的签名验证失败必须拒绝。

    /// 签名长度不是 64 字节必须拒绝。

    /// 提案通过并 execute_create 后再尝试投票应被投票引擎的 AlreadyVoted/状态检查挡住。
    #[test]
    fn passed_create_proposal_rejects_replay() {
        new_test_ext().execute_with(|| {
            let (account_name, duoqian_address, admins, pairs) =
                setup_personal_duoqian("K005", &[1, 2, 3]);

            assert_ok!(Duoqian::propose_create_personal(
                RuntimeOrigin::signed(admins[0].clone()),
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
                3,
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
                3,
            );
            assert!(second.is_err(), "replay must fail");

            // 余额依然等于第一次的 1_000(没有重复入金)
            assert_eq!(Balances::free_balance(&duoqian_address), 1_000);
        });
    }
}
