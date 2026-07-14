#![cfg_attr(not(feature = "std"), no_std)]

/// 模块标识前缀，用于在 ProposalData 中区分不同业务模块，防止跨模块误解码。
/// 长度 8 字节（`b"pri-mgmt"`）；admins 模块 / citizenwallet / citizenapp 三方解码必须保持一致。
pub const MODULE_TAG: &[u8] = b"pri-mgmt";

pub use pallet::*;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod close;
pub mod common;
pub mod institution;
pub mod traits;
pub mod weights;

#[cfg(test)]
mod tests;

use admin_primitives::{
    is_private_admin_code, AdminAccountKind, AdminAccountQuery, InstitutionAdminAccountLifecycle,
};
use codec::{Decode, Encode};
use frame_support::{
    ensure,
    pallet_prelude::*,
    traits::{Currency, ReservableCurrency},
    BoundedVec,
};
use frame_system::pallet_prelude::*;
use sp_core::sr25519::Public as Sr25519Public;
use sp_std::{collections::btree_set::BTreeSet, prelude::*};
pub use traits::{
    AccountValidator, CidInstitutionVerifier, InstitutionCidQuery, InstitutionMultisigQuery,
    ProtectedSourceChecker, RegistryAuthority, ReservedAccountGuard,
};
use votingengine::{
    types::InstitutionCode, InternalVoteEngine, InternalVoteResultCallback,
    ProposalExecutionOutcome, STATUS_REJECTED,
};

pub use entity_primitives::{
    InstitutionAdminAssignment, InstitutionAssignmentSource, InstitutionAssignmentStatus,
    InstitutionRole, InstitutionRoleStatus,
};
pub use institution::role::{
    InstitutionAdminAssignmentOf, InstitutionAdminAssignmentsOf, InstitutionRoleOf,
    InstitutionRolesOf, RoleCodeOf,
};
pub use institution::types::{
    CloseInstitutionAction, CreateInstitutionAccount, InstitutionAccountInfo, InstitutionInfo,
    InstitutionInitialAccount, InstitutionLifecycleStatus, RegisteredInstitution,
};
pub use primitives::account_derive::{AccountKind, RESERVED_NAME_FEE, RESERVED_NAME_MAIN};

pub(crate) type BalanceOf<T> =
    <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(3);

    #[pallet::config]
    pub trait Config: frame_system::Config + votingengine::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// 内部投票引擎
        type InternalVoteEngine: votingengine::InternalVoteEngine<Self::AccountId>;

        /// 私权机构管理员生命周期写入口。
        type AdminLifecycle: InstitutionAdminAccountLifecycle<Self::AccountId>;

        /// 兄弟机构生命周期查询入口，用于禁止同一 CID 在公权模块重复登记。
        type SiblingInstitutionQuery: InstitutionCidQuery<CidNumberOf<Self>>;

        /// 管理员统一查询入口，由 runtime 路由到公权/私权/创世管理员模块。
        type AdminAccountQuery: AdminAccountQuery<Self::AccountId>;

        type AccountValidator: AccountValidator<Self::AccountId>;
        type ReservedAccountChecker: ReservedAccountGuard<Self::AccountId>;
        type ProtectedSourceChecker: ProtectedSourceChecker<Self::AccountId>;
        type InstitutionAsset: primitives::institution_asset::InstitutionAsset<Self::AccountId>;
        type CidInstitutionVerifier: CidInstitutionVerifier<
            Self::AccountId,
            AccountNameOf<Self>,
            RegisterNonceOf<Self>,
            RegisterSignatureOf<Self>,
        >;
        /// 注册局登记授权校验入口。
        ///
        /// 注册局管理员代创建机构时,origin 是注册局管理员,目标 admins
        /// 是新机构自己的管理员;二者不能再强制相同。本 trait 负责校验 FRG/CREG
        /// 对目标 CID 与机构码是否有登记权。
        type RegistryAuthority: RegistryAuthority<Self::AccountId>;

        /// 手续费分账路由（创建入金和注销转出的手续费）
        type FeeRouter: frame_support::traits::OnUnbalanced<
            <Self::Currency as Currency<Self::AccountId>>::NegativeImbalance,
        >;

        #[pallet::constant]
        type MaxAdmins: Get<u32>;

        #[pallet::constant]
        type MaxCidNumberLength: Get<u32>;

        /// 机构全称与机构账户名共用的最大字节长度。
        #[pallet::constant]
        type MaxAccountNameLength: Get<u32>;

        #[pallet::constant]
        type MaxRegisterNonceLength: Get<u32>;

        #[pallet::constant]
        type MaxRegisterSignatureLength: Get<u32>;

        /// 单个机构注册交易最多可携带的账户数量。
        ///
        /// CID 默认包含主账户和费用账户，用户可新增其他账户；这里限制链上
        /// 初始入金列表长度，避免机构注册交易过大。
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

    pub type AdminsOf<T> =
        BoundedVec<<T as frame_system::Config>::AccountId, <T as Config>::MaxAdmins>;

    pub type CidNumberOf<T> = BoundedVec<u8, <T as Config>::MaxCidNumberLength>;
    pub type AccountNameOf<T> = BoundedVec<u8, <T as Config>::MaxAccountNameLength>;
    pub type RegisterNonceOf<T> = BoundedVec<u8, <T as Config>::MaxRegisterNonceLength>;
    pub type RegisterSignatureOf<T> = BoundedVec<u8, <T as Config>::MaxRegisterSignatureLength>;
    /// 注册凭证里的账户名列表,顺序必须与 CID `registration-info` 返回一致。
    pub type InstitutionAccountNamesOf<T> =
        BoundedVec<AccountNameOf<T>, <T as Config>::MaxInstitutionAccounts>;
    /// 机构创建时用户输入的账户初始余额列表项。
    pub type InstitutionInitialAccountOf<T> =
        InstitutionInitialAccount<AccountNameOf<T>, BalanceOf<T>>;
    /// 机构创建时用户输入的账户初始余额列表。
    pub type InstitutionInitialAccountsOf<T> =
        BoundedVec<InstitutionInitialAccountOf<T>, <T as Config>::MaxInstitutionAccounts>;
    /// 机构注册交易中保存的已派生账户项。
    pub type CreateInstitutionAccountOf<T> = CreateInstitutionAccount<
        AccountNameOf<T>,
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
    >;
    /// 机构注册交易中保存的已派生账户列表。
    pub type CreateInstitutionAccountsOf<T> =
        BoundedVec<CreateInstitutionAccountOf<T>, <T as Config>::MaxInstitutionAccounts>;
    /// 机构级信息(链上最小集)。
    pub type InstitutionInfoOf<T> = InstitutionInfo<
        BlockNumberFor<T>,
        AccountNameOf<T>,
        CidNumberOf<T>,
        <T as frame_system::Config>::AccountId,
    >;
    /// 机构账户信息。
    pub type InstitutionAccountInfoOf<T> = InstitutionAccountInfo<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        BlockNumberFor<T>,
    >;
    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// CID 机构登记：(cid_number, account_name) -> account（由 blake2b_256 派生）。
    /// 同一 cid_number 可通过不同 account_name 注册多个多签账户。
    #[pallet::storage]
    pub type CidRegisteredAccount<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        CidNumberOf<T>,
        Blake2_128Concat,
        AccountNameOf<T>,
        T::AccountId,
        OptionQuery,
    >;

    /// CID 机构登记反向索引：account -> { cid_number, nonce }
    #[pallet::storage]
    #[pallet::getter(fn account_registered_cid)]
    pub type AccountRegisteredCid<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        RegisteredInstitution<CidNumberOf<T>, AccountNameOf<T>>,
        OptionQuery,
    >;

    /// 机构级信息(链上最小集)：key 为 cid_number。
    ///
    /// 只保存全国可见的机构身份事实:名称(仅公权)、机构码、创建块号、生命周期状态。
    /// 主账户/费用账户由 (cid_number, 保留名) 派生且常驻 InstitutionAccounts,不在此重复;
    /// 管理员集合长期真源在 admins 模块,动态阈值长期真源在 internal-vote,均不在此存快照。
    #[pallet::storage]
    #[pallet::getter(fn institution_of)]
    pub type Institutions<T: Config> =
        StorageMap<_, Blake2_128Concat, CidNumberOf<T>, InstitutionInfoOf<T>, OptionQuery>;

    /// 私权机构自己的动态岗位目录。
    #[pallet::storage]
    #[pallet::getter(fn institution_role_of)]
    pub type InstitutionRoles<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        CidNumberOf<T>,
        Blake2_128Concat,
        crate::institution::role::RoleCodeOf,
        crate::institution::role::InstitutionRoleOf<T>,
        OptionQuery,
    >;

    /// 私权机构岗位上的管理员任职集合。
    #[pallet::storage]
    #[pallet::getter(fn institution_role_assignments)]
    pub type InstitutionRoleAssignments<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        CidNumberOf<T>,
        Blake2_128Concat,
        crate::institution::role::RoleCodeOf,
        crate::institution::role::RoleAssignmentsOf<T>,
        ValueQuery,
    >;

    /// 机构账户表：(cid_number, account_name) -> 账户地址与激活状态。
    #[pallet::storage]
    #[pallet::getter(fn institution_account_of)]
    pub type InstitutionAccounts<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        CidNumberOf<T>,
        Blake2_128Concat,
        AccountNameOf<T>,
        InstitutionAccountInfoOf<T>,
        OptionQuery,
    >;

    /// 已消费的机构登记 nonce，防止 proof 重放。
    #[pallet::storage]
    #[pallet::getter(fn used_register_nonce)]
    pub type UsedRegisterNonce<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, bool, ValueQuery>;

    /// 已用注销凭证 nonce(防同一注销凭证重放/关多账户)。
    #[pallet::storage]
    pub type UsedDeregisterNonce<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, bool, ValueQuery>;

    /// 私权机构多签当前进行中的关闭提案 ID（防止并发注销提案）。
    /// 发起 propose_close 时写入，execute_close 成功或执行失败后清除。
    /// PendingCloseProposal 分两份:个人侧在 personal-manage 自持,
    /// 机构侧由本表承载,作用域为私权机构多签账户。
    #[pallet::storage]
    #[pallet::getter(fn institution_pending_close)]
    pub type InstitutionPendingClose<T: Config> =
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
        /// 机构关闭提案已发起。
        InstitutionCloseProposed {
            proposal_id: u64,
            account: T::AccountId,
            proposer: T::AccountId,
            beneficiary: T::AccountId,
        },
        /// 机构关闭投票已提交。
        InstitutionCloseVoteSubmitted {
            proposal_id: u64,
            who: T::AccountId,
            approve: bool,
        },
        /// 机构关闭成功(投票通过,余额转出)。
        InstitutionClosed {
            proposal_id: u64,
            account: T::AccountId,
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
            fee: BalanceOf<T>,
        },
        /// 机构关闭执行失败。
        InstitutionCloseExecutionFailed {
            proposal_id: u64,
            account: T::AccountId,
        },
        /// 机构注册创建成功：机构、账户和管理员集合均已激活。
        InstitutionCreated {
            cid_number: CidNumberOf<T>,
            main_account: T::AccountId,
            account_count: u32,
            initial_total: BalanceOf<T>,
            fee: BalanceOf<T>,
        },
        /// 已完成的业务结果原子更新机构岗位、任职、法定代表人和 admins。
        InstitutionGovernanceApplied {
            cid_number: CidNumberOf<T>,
            institution_account: T::AccountId,
            role_changes: u32,
            assignment_changes: u32,
            admins_len: u32,
            legal_representative_updated: bool,
            result_source_ref: crate::institution::role::AssignmentSourceRefOf,
        },
        /// CID 机构登记
        CidInstitutionRegistered {
            cid_number: CidNumberOf<T>,
            account_name: AccountNameOf<T>,
            account: T::AccountId,
            submitter: T::AccountId,
        },
        /// 机构信息(全称/简称)已更新。
        InstitutionInfoUpdated {
            cid_number: CidNumberOf<T>,
            cid_full_name: AccountNameOf<T>,
            cid_short_name: AccountNameOf<T>,
            submitter: T::AccountId,
        },
        /// 已给存量机构新增账户。
        InstitutionAccountAdded {
            cid_number: CidNumberOf<T>,
            account_name: AccountNameOf<T>,
            account: T::AccountId,
            submitter: T::AccountId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 参数不完整
        IncompleteParameters,
        /// 账户非法
        InvalidAccount,
        /// 账户为制度保留账户，不允许注册
        AccountReserved,
        /// 账户已存在（已初始化）
        AccountAlreadyExists,
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
        /// 注册局无权登记目标机构
        RegistryAuthorityDenied,
        /// 管理员数量不合法（必须 >=2）
        InvalidAdminsLen,
        /// 管理员数量与列表长度不一致
        AdminsLenMismatch,
        /// 机构账户管理员机构码只能是公权/私权法人机构码。
        /// 非法人必须由 CID 上层按所属法人归属显式路由。
        InvalidInstitutionCode,
        /// 多签账户不存在
        AccountNotFound,
        /// 多签账户处于 pending 状态，不可操作
        AccountNotActive,
        /// 注销收款账户非法（不允许等于 account）
        InvalidBeneficiary,
        /// 资金转出源地址受保护，不允许转出
        ProtectedSource,
        /// CID机构未登记，不允许创建
        InstitutionNotRegistered,
        /// CID 机构登记签名无效
        InvalidCidInstitutionSignature,
        /// CID ID 重复登记
        CidAlreadyRegistered,
        /// CID ID 为空
        EmptyCidNumber,
        /// CID 号格式或机构码家族非法
        InvalidCidNumber,
        /// 目标机构不存在。
        InstitutionNotFound,
        /// 机构已整体关闭(墓碑),该 CID 号永不复用
        InstitutionAlreadyClosed,
        /// 机构登记 nonce 已被使用
        RegisterNonceAlreadyUsed,
        /// 机构签发凭证缺签发机构 CID 号。
        EmptyIssuerCidNumber,
        /// 机构签发凭证缺业务作用域省名。
        EmptyScopeProvinceName,
        /// 私权机构当前不接收镇归属,必须传空 town_code。
        InvalidTownCode,
        /// 无法将派生地址转换为账户ID
        DerivedAccountDecodeFailed,
        /// 账户仍有保留余额，不允许注销
        ReservedBalanceRemaining,
        /// nonce 已耗尽
        NonceOverflow,
        /// runtime 配置不合法
        InvalidRuntimeConfig,
        /// 提案业务数据未找到
        ProposalActionNotFound,
        /// 转账失败
        TransferFailed,
        /// 管理员非本提案管理员
        UnauthorizedAdmin,
        /// 机构账户名为空
        EmptyAccountName,
        /// 法定代表人公开姓名为空
        EmptyLegalRepresentativeName,
        /// 法定代表人公民 CID 为空
        EmptyLegalRepresentativeCidNumber,
        /// 机构级创建缺少主账户
        MissingMainAccount,
        /// 机构级创建缺少费用账户
        MissingFeeAccount,
        /// 机构级创建账户名重复
        DuplicateAccountName,
        /// 机构已经存在
        InstitutionAlreadyExists,
        /// propose_close 校验:仅机构地址可走本入口(个人地址转 personal-manage)。
        NotInstitutionAccount,
        /// 机构账户列表为空
        EmptyInstitutionAccounts,
        /// 机构账户数量超过上限
        TooManyInstitutionAccounts,
        /// 初始余额累计溢出
        InitialAmountOverflow,
        /// 手续费扣取失败
        FeeWithdrawFailed,
        /// 注销后转账金额低于 ED
        CloseTransferBelowED,
        /// 该多签账户已有进行中的关闭提案，不允许重复发起
        CloseAlreadyPending,
        /// 提案未被拒绝，不可清理
        ProposalNotRejected,
        /// 账户名占用保留角色名（"主账户"/"费用账户" 必须走 Role::Main/Fee，
        /// 禁止作为 Role::Named 的自定义命名参数）
        ReservedAccountName,
        /// sr25519 签名长度必须恰好为 64 字节
        MalformedSignature,
        /// 创世写入的封存公权机构永不可注销关闭
        CannotCloseProtectedInstitution,
        /// 治理机构(国家储委会/省储委会/省储行)永不可注销关闭
        CannotCloseGovernance,
        /// 注销凭证验签失败
        InvalidDeregisterCredential,
        /// 注销凭证 nonce 已使用(防重放)
        DeregisterNonceAlreadyUsed,
        /// 机构创建必须至少定义一个岗位。
        InstitutionRolesEmpty,
        /// 机构创建必须至少绑定一条管理员任职。
        InstitutionAssignmentsEmpty,
        /// 岗位所属 CID 与创建目标不一致。
        RoleCidMismatch,
        /// 岗位代码为空或超过边界。
        InvalidRoleCode,
        /// 岗位名称为空。
        InvalidRoleName,
        /// 初始岗位必须直接处于有效状态。
        InitialRoleMustBeActive,
        /// 同一机构内岗位代码重复。
        DuplicateRoleCode,
        /// 任职所属 CID 与创建目标不一致。
        AssignmentCidMismatch,
        /// 任职来源与当前写入流程不一致。
        InvalidAssignmentSource,
        /// 初始任职必须直接处于有效状态。
        InitialAssignmentMustBeActive,
        /// 任职引用的岗位不存在。
        AssignmentRoleNotFound,
        /// 必须设置任期的岗位没有合法任期。
        InvalidAssignmentTerm,
        /// 无任期岗位携带了任期值。
        UnexpectedAssignmentTerm,
        /// 同一管理员在同一岗位存在重复任职。
        DuplicateAssignment,
        /// 初始岗位没有任何管理员任职。
        RoleHasNoAssignment,
        /// 任职去重后的管理员数量超过机构上限。
        TooManyInstitutionAdmins,
        /// 治理结果目标不是机构主账户或与机构码不匹配。
        InvalidAssignmentResultInstitution,
        /// 治理结果没有管理员或包含重复管理员。
        InvalidAssignmentResultAdmins,
        /// 治理结果缺少投票、选举或任命追溯引用。
        AssignmentSourceRefEmpty,
        /// 治理结果没有岗位、任职或法定代表人变化。
        GovernanceResultEmpty,
        /// 单次治理结果包含的岗位或任职变化超过机构上限。
        TooManyGovernanceChanges,
        /// 同一治理结果重复提交同一个岗位定义。
        DuplicateGovernanceRoleChange,
        /// 同一治理结果重复提交同一个岗位任职集合。
        DuplicateGovernanceAssignmentChange,
        /// 已停用岗位仍有有效任职。
        InactiveRoleHasAssignments,
    }

    /// 提案操作类型标记：存储在 ProposalData 的第一个字节。
    /// ACTION = 1 永久保留空位,不复用。
    pub const ACTION_CLOSE: u8 = 2;

    /// 注销凭证作用域:整机构(关主账户=级联关全部账户)/ 单账户(只关该非主账户)。
    pub const SCOPE_INSTITUTION: u8 = 0;
    pub const SCOPE_ACCOUNT: u8 = 1;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // NOTE: `call_index` values are the on-chain ABI and must remain stable.

        // call_index = 0 永久保留空位,不复用

        /// CID 注册信息凭证批量登记机构账户地址。
        ///
        /// 本入口与身份注册局 `/registration-info` 对齐,业务字段只接收
        /// `cid_number / cid_full_name / account_names[]`。机构类型、企业类型、
        /// 所属法人关系只由身份注册局用于候选资格判断,不再进入链上注册 payload。
        #[pallet::call_index(2)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::register_cid_private_institution())]
        pub fn register_cid_private_institution(
            origin: OriginFor<T>,
            cid_number: CidNumberOf<T>,
            cid_full_name: AccountNameOf<T>,
            account_names: InstitutionAccountNamesOf<T>,
            register_nonce: RegisterNonceOf<T>,
            signature: RegisterSignatureOf<T>,
            issuer_cid_number: Vec<u8>,
            issuer_main_account: T::AccountId,
            signer_pubkey: [u8; 32],
            scope_province_name: Vec<u8>,
            scope_city_name: Vec<u8>,
        ) -> DispatchResult {
            let submitter = ensure_signed(origin)?;
            crate::institution::register::do_register_cid_private_institution::<T>(
                submitter,
                cid_number,
                cid_full_name,
                account_names,
                register_nonce,
                signature,
                issuer_cid_number,
                issuer_main_account,
                signer_pubkey,
                scope_province_name,
                scope_city_name,
            )
        }

        /// 注册创建私权机构。
        ///
        /// 该交易注册的是“机构”而不是单个账户。创建者必须一次性提交主账户、
        /// 费用账户以及需要初始化的自定义账户余额；交易成功即激活机构、账户
        /// 与管理员集合。
        #[pallet::call_index(5)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::propose_create_private_institution())]
        pub fn propose_create_private_institution(
            origin: OriginFor<T>,
            cid_number: CidNumberOf<T>,
            cid_full_name: AccountNameOf<T>,
            cid_short_name: AccountNameOf<T>,
            town_code: AccountNameOf<T>,
            legal_representative_name: AccountNameOf<T>,
            legal_representative_cid_number: CidNumberOf<T>,
            legal_representative_account: T::AccountId,
            accounts: InstitutionInitialAccountsOf<T>,
            institution_code: InstitutionCode,
            roles: crate::institution::role::InstitutionRolesOf<T>,
            assignments: crate::institution::role::InstitutionAdminAssignmentsOf<T>,
            threshold: u32,
            register_nonce: RegisterNonceOf<T>,
            signature: RegisterSignatureOf<T>,
            issuer_cid_number: Vec<u8>,
            issuer_main_account: T::AccountId,
            signer_pubkey: [u8; 32],
            scope_province_name: Vec<u8>,
            scope_city_name: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            crate::institution::create::do_propose_create_private_institution::<T>(
                who,
                cid_number,
                cid_full_name,
                cid_short_name,
                town_code,
                legal_representative_name,
                legal_representative_cid_number,
                legal_representative_account,
                accounts,
                institution_code,
                roles,
                assignments,
                threshold,
                register_nonce,
                signature,
                issuer_cid_number,
                issuer_main_account,
                signer_pubkey,
                scope_province_name,
                scope_city_name,
            )
        }

        /// 注册局更新机构全称/简称(链是机构信息唯一真源)。
        /// 机构码/CID/省市码物理编码在 CID 里,不可改故不作为参数。
        #[pallet::call_index(6)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::update_institution_info())]
        #[allow(clippy::too_many_arguments)]
        pub fn update_institution_info(
            origin: OriginFor<T>,
            cid_number: CidNumberOf<T>,
            cid_full_name: AccountNameOf<T>,
            cid_short_name: AccountNameOf<T>,
            register_nonce: RegisterNonceOf<T>,
            signature: RegisterSignatureOf<T>,
            issuer_cid_number: Vec<u8>,
            issuer_main_account: T::AccountId,
            signer_pubkey: [u8; 32],
            scope_province_name: Vec<u8>,
            scope_city_name: Vec<u8>,
        ) -> DispatchResult {
            let submitter = ensure_signed(origin)?;
            crate::institution::maintain::do_update_institution_info::<T>(
                submitter,
                cid_number,
                cid_full_name,
                cid_short_name,
                register_nonce,
                signature,
                issuer_cid_number,
                issuer_main_account,
                signer_pubkey,
                scope_province_name,
                scope_city_name,
            )
        }

        /// 给已存在机构新增账户(新账户名 → 确定性派生地址 → 上链)。
        #[pallet::call_index(7)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::add_institution_account())]
        #[allow(clippy::too_many_arguments)]
        pub fn add_institution_account(
            origin: OriginFor<T>,
            cid_number: CidNumberOf<T>,
            account_names: InstitutionAccountNamesOf<T>,
            register_nonce: RegisterNonceOf<T>,
            signature: RegisterSignatureOf<T>,
            issuer_cid_number: Vec<u8>,
            issuer_main_account: T::AccountId,
            signer_pubkey: [u8; 32],
            scope_province_name: Vec<u8>,
            scope_city_name: Vec<u8>,
        ) -> DispatchResult {
            let submitter = ensure_signed(origin)?;
            crate::institution::maintain::do_add_institution_account::<T>(
                submitter,
                cid_number,
                account_names,
                register_nonce,
                signature,
                issuer_cid_number,
                issuer_main_account,
                signer_pubkey,
                scope_province_name,
                scope_city_name,
            )
        }

        /// 发起"关闭私权机构多签账户"提案。
        ///
        /// 仅服务于 CID 注册机构地址(`AccountRegisteredCid` 命中);
        /// 个人多签关闭走 personal-manage::propose_close 入口,
        /// 输入个人地址会返回 `Error::NotInstitutionAccount`。
        #[pallet::call_index(1)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::propose_close_private_institution())]
        #[allow(clippy::too_many_arguments)]
        pub fn propose_close_private_institution(
            origin: OriginFor<T>,
            account: T::AccountId,
            beneficiary: T::AccountId,
            register_nonce: RegisterNonceOf<T>,
            signature: RegisterSignatureOf<T>,
            issuer_cid_number: Vec<u8>,
            issuer_main_account: T::AccountId,
            signer_pubkey: [u8; 32],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            crate::close::do_propose_institution_close::<T>(
                who,
                account,
                beneficiary,
                register_nonce,
                signature,
                issuer_cid_number,
                issuer_main_account,
                signer_pubkey,
            )
        }

        /// 发起"创建个人多签账户"提案（无需 CID 注册）。
        ///
        /// 地址由 `creator + account_name` 派生：
        /// 清理已被拒绝或超时的关闭提案残留状态(机构侧)。
        /// 任意签名账户可调用。用于解决投票引擎 on_initialize 超时 reject 后
        /// 本模块无法自动收到通知导致的 InstitutionPendingClose 残留。
        ///
        /// 仅处理 ACTION_CLOSE 机构关闭提案;
        /// 个人多签的清理由 personal-manage::cleanup_rejected_private_proposal 自持。
        #[pallet::call_index(4)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::cleanup_rejected_private_proposal())]
        pub fn cleanup_rejected_private_proposal(
            origin: OriginFor<T>,
            proposal_id: u64,
        ) -> DispatchResult {
            let _ = ensure_signed(origin)?;

            // 读取提案数据，校验 MODULE_TAG 后判断操作类型
            let raw = votingengine::Pallet::<T>::get_proposal_data(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            let tag = crate::MODULE_TAG;
            ensure!(
                raw.len() > tag.len() && &raw[..tag.len()] == tag,
                Error::<T>::ProposalActionNotFound
            );
            let action_tag = raw[tag.len()];

            // 校验投票引擎状态必须为 REJECTED
            let proposal = votingengine::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            ensure!(
                proposal.status == STATUS_REJECTED,
                Error::<T>::ProposalNotRejected
            );

            match action_tag {
                ACTION_CLOSE => {
                    let action =
                        CloseInstitutionAction::<T::AccountId>::decode(&mut &raw[tag.len() + 1..])
                            .map_err(|_| Error::<T>::ProposalActionNotFound)?;
                    InstitutionPendingClose::<T>::remove(&action.account);
                }
                _ => return Err(Error::<T>::ProposalActionNotFound.into()),
            }

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// 由 (cid_number, account_name) 派生机构账户地址 + 返回其 `AccountKind`。
        ///
        /// 唯一真源 = `primitives::account_derive`(op_tag/保留名/路由/payload/派生入口)。
        /// 本函数仅做薄适配:把 `account_derive` 的派生结果 + 注册策略翻译成本 pallet 的
        /// `DispatchError`,并把 32 字节 digest 解码为 `T::AccountId`:
        /// - 空 account_name → `EmptyAccountName`
        /// - `永久质押`/`安全基金`/`两和基金`(制度专属)→ `ReservedAccountName`(普通机构禁止注册)
        /// - `主账户`/`费用账户` → `InstitutionMain`/`InstitutionFee`(强制默认路由)
        /// - 其他非空 → `InstitutionNamed`
        ///
        /// 这是 `register_cid_private_institution` / 机构创建 / 关闭等入口的唯一派生入口。
        ///
        /// 返回的 `AccountKind` 借用入参 `cid_number`/`account_name`,供调用方分支判断
        /// 主账户/费用账户/自定义账户(`is_default_account` / `is_main_account`)。
        pub fn derive_registered_account<'a>(
            cid_number: &'a [u8],
            account_name: &'a [u8],
        ) -> Result<(T::AccountId, AccountKind<'a>), DispatchError> {
            ensure!(!account_name.is_empty(), Error::<T>::EmptyAccountName);
            ensure!(
                !primitives::account_derive::is_forbidden_account_name(account_name),
                Error::<T>::ReservedAccountName
            );
            // institution_kind_by_name 对非空名必返回 Some(空名已在上面拦截)。
            // 命中保留名 → Main/Fee;否则 → Named。质押/安全/两和已被 forbidden 拦下,
            // 故此处只可能得到 Main/Fee/Named 三种机构种类。
            let kind =
                primitives::account_derive::institution_kind_by_name(cid_number, account_name)
                    .ok_or(Error::<T>::EmptyAccountName)?;
            let digest = kind.derive(T::SS58Prefix::get());
            let account = T::AccountId::decode(&mut &digest[..])
                .map_err(|_| Error::<T>::DerivedAccountDecodeFailed)?;
            Ok((account, kind))
        }

        /// 该 `AccountKind` 是否为机构强制默认账户(主账户 / 费用账户)。
        pub fn is_default_account(kind: &AccountKind<'_>) -> bool {
            matches!(
                kind,
                AccountKind::InstitutionMain { .. } | AccountKind::InstitutionFee { .. }
            )
        }

        /// 该 `AccountKind` 是否为机构主账户。
        pub fn is_main_account(kind: &AccountKind<'_>) -> bool {
            matches!(kind, AccountKind::InstitutionMain { .. })
        }

        // derive_personal_account 在 personal-manage::Pallet;
        // private-manage 的机构地址只走 derive_registered_account。

        pub(crate) fn ensure_unique_admins(admins: &[T::AccountId]) -> Result<(), DispatchError> {
            let mut seen = BTreeSet::new();
            for admin in admins.iter() {
                ensure!(seen.insert(admin.clone()), Error::<T>::DuplicateAdmin);
            }
            Ok(())
        }

        /// 计算 `admins_root = blake2_256(SCALE.encode(sorted_admins))`。
        ///
        /// 排序规则:按 `AccountId` 的字节序(Substrate AccountId32 默认 Ord 即字典序)。
        /// citizenapp 端需要用同样的排序规则 + SCALE 布局,保证签名消息字节一致。
        pub fn compute_admins_root(admins: &AdminsOf<T>) -> [u8; 32] {
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

        pub(crate) fn ensure_admin_config(admins: &AdminsOf<T>, threshold: u32) -> DispatchResult {
            ensure!(T::MaxAdmins::get() >= 2, Error::<T>::InvalidRuntimeConfig);
            let admins_len = admins.len() as u32;
            ensure!(admins_len >= 2, Error::<T>::InvalidAdminsLen);
            ensure!(
                threshold > 0
                    && threshold <= admins_len
                    && u64::from(threshold).saturating_mul(2) > u64::from(admins_len),
                Error::<T>::InvalidThreshold
            );
            // 账户语义校验取 profile.admin_account:注册局代创建时,发起人是注册局管理员,
            // 目标 admins 是新机构管理员集合,这里只校验目标集合自身合法。
            Self::ensure_unique_admins(admins.as_slice())?;
            Ok(())
        }

        pub(crate) fn set_active_admin_account_direct(
            cid_number: &CidNumberOf<T>,
            institution_code: InstitutionCode,
            institution_id: T::AccountId,
            admins: &AdminsOf<T>,
            threshold: u32,
        ) -> DispatchResult {
            Self::ensure_lifecycle_institution_code(&institution_code)?;
            T::AdminLifecycle::set_active_institution_admin_account(
                crate::MODULE_TAG,
                institution_id,
                cid_number.to_vec(),
                institution_code,
                AdminAccountKind::PrivateInstitution,
                admins.iter().cloned().collect(),
                threshold,
            )
        }

        pub(crate) fn close_admin_account(
            proposal_id: u64,
            institution_code: InstitutionCode,
            institution_id: T::AccountId,
        ) -> DispatchResult {
            Self::ensure_lifecycle_institution_code(&institution_code)?;
            T::AdminLifecycle::close_institution_admin_account_for_proposal(
                proposal_id,
                crate::MODULE_TAG,
                institution_id,
            )
        }

        /// 私权机构生命周期模块只接受私权法人机构码。
        pub(crate) fn ensure_lifecycle_institution_code(
            institution_code: &InstitutionCode,
        ) -> DispatchResult {
            ensure!(
                is_private_admin_code(institution_code),
                Error::<T>::InvalidInstitutionCode
            );
            Ok(())
        }

        /// 从任意私权机构多签账户反查其管理员账户账户地址。
        ///
        /// 个人多签由 personal-manage 自持；本函数仅服务机构账户。
        /// 管理员属于机构(不属于账户)。任意机构账户(主/费用/自定义)都解析到
        /// 本机构【主账户】——即 admins 模块 里承载该机构唯一管理员集的键。这样私权机构生命周期员
        /// 统一管理机构及其全部账户(创建/注销账户都由这套管理员授权)。
        pub fn resolve_admin_account_for_account(account: &T::AccountId) -> Option<T::AccountId> {
            let registered = AccountRegisteredCid::<T>::get(account)?;
            // 主账户地址由 (cid_number, 主账户保留名) 确定性派生,与 InstitutionAccounts 中存储的一致;
            // 机构本身不再重复保存 main_account。
            let (main_account, _) = Self::derive_registered_account(
                registered.cid_number.as_slice(),
                RESERVED_NAME_MAIN,
            )
            .ok()?;
            Some(main_account)
        }

        /// 从任意机构账户反查管理员更换机构码。
        ///
        /// 机构账户必须使用公权/私权法人机构码；PMUL 只属于个人多签。
        pub fn resolve_institution_code_for_account(
            account: &T::AccountId,
        ) -> Option<InstitutionCode> {
            let registered = AccountRegisteredCid::<T>::get(account)?;
            Institutions::<T>::get(&registered.cid_number).map(|inst| inst.institution_code)
        }

        // account_names_payload_from_initial_accounts 在 institution::accounts。

        /// 把批量 register 入口的 account_names 抽成验签 payload。
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

        // 投票回调执行体:
        // - ACTION_CLOSE → crate::close::execute_institution_close_with_finalizer
        // (ACTION_CREATE_PERSONAL 在 personal-manage 独立 pallet)
    }
}

// ──── InstitutionMultisigQuery 实现:对 multisig-transfer / runtime config 暴露查询 ────
//
// 输入任意机构账户(主/费用/自创),直接以账户地址读取 admins 模块 账户。
// 再按 Institutions[cid_number].institution_code 读取机构码。这条路径保证
// 机构账户只使用公权/私权法人机构码,不再把机构账户错误塞到 PMUL。

impl<T: pallet::Config> traits::InstitutionMultisigQuery<T::AccountId> for pallet::Pallet<T> {
    fn lookup_cid(addr: &T::AccountId) -> Option<Vec<u8>> {
        pallet::AccountRegisteredCid::<T>::get(addr)
            .map(|registered| registered.cid_number.to_vec())
    }

    fn lookup_org(addr: &T::AccountId) -> Option<InstitutionCode> {
        pallet::Pallet::<T>::resolve_institution_code_for_account(addr)
    }

    fn lookup_admin_config(
        addr: &T::AccountId,
    ) -> Option<primitives::multisig::MultisigConfigSnapshot<T::AccountId>> {
        let institution_code = Self::lookup_org(addr)?;
        let account = pallet::Pallet::<T>::resolve_admin_account_for_account(addr)?;
        let admins =
            T::AdminAccountQuery::active_account_admins(institution_code, account.clone())?;
        let threshold = <T as Config>::InternalVoteEngine::active_dynamic_threshold(
            institution_code,
            account.clone(),
        )?;
        let admins_len = admins.len() as u32;
        Some(primitives::multisig::MultisigConfigSnapshot {
            admins,
            admins_len,
            threshold,
        })
    }

    fn is_active(addr: &T::AccountId) -> bool {
        let Some(registered) = pallet::AccountRegisteredCid::<T>::get(addr) else {
            return false;
        };
        matches!(
            pallet::InstitutionAccounts::<T>::get(&registered.cid_number, &registered.account_name)
                .map(|a| a.status),
            Some(institution::types::InstitutionLifecycleStatus::Active)
        )
    }
}

impl<T: pallet::Config> traits::InstitutionCidQuery<pallet::CidNumberOf<T>> for pallet::Pallet<T> {
    fn cid_exists(cid_number: &pallet::CidNumberOf<T>) -> bool {
        pallet::Institutions::<T>::contains_key(cid_number)
            || pallet::CidRegisteredAccount::<T>::iter_prefix(cid_number)
                .next()
                .is_some()
    }
}

impl<T: pallet::Config> traits::InstitutionLegalRepresentativeQuery<T::AccountId>
    for pallet::Pallet<T>
{
    fn legal_representative(
        institution_code: InstitutionCode,
        institution: T::AccountId,
    ) -> Option<T::AccountId> {
        let registered = pallet::AccountRegisteredCid::<T>::get(&institution)?;
        let info = pallet::Institutions::<T>::get(&registered.cid_number)?;
        if info.institution_code != institution_code
            || info.status != institution::types::InstitutionLifecycleStatus::Active
        {
            return None;
        }
        info.legal_representative_account
    }
}

// ──── 投票终态回调:把已通过的机构关闭提案落地到链上 ────
//
// 投票统一由投票引擎承担,提案通过(或否决)经
// [`votingengine::InternalVoteResultCallback`] 广播回来。
// 本 Executor(机构侧):
// - 按 `MODULE_TAG + ACTION_CLOSE` 前缀认领机构关闭提案;
// - `approved = true` → 分派到 `close::execute_institution_close_with_finalizer`;
// - `approved = false` → 清理 InstitutionPendingClose,释放地址占用。
// (ACTION_CREATE_PERSONAL 在 personal-manage::InternalVoteExecutor)
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
                ACTION_CLOSE => {
                    let action =
                        CloseInstitutionAction::<T::AccountId>::decode(&mut &raw[tag.len() + 1..])
                            .map_err(|_| pallet::Error::<T>::ProposalActionNotFound)?;
                    let exec_result = with_transaction(|| {
                        match crate::close::execute_institution_close_with_finalizer::<T>(
                            proposal_id,
                            &action,
                        ) {
                            Ok(()) => TransactionOutcome::Commit(Ok(())),
                            Err(e) => TransactionOutcome::Rollback(Err(e)),
                        }
                    });
                    if exec_result.is_err() {
                        pallet::Pallet::<T>::deposit_event(
                            pallet::Event::<T>::InstitutionCloseExecutionFailed {
                                proposal_id,
                                account: action.account,
                            },
                        );
                        return Ok(ProposalExecutionOutcome::RetryableFailed);
                    }
                    return Ok(ProposalExecutionOutcome::Executed);
                }
                _ => return Ok(ProposalExecutionOutcome::Ignored),
            }
        } else {
            // 否决:清理关闭 Pending 记录释放地址锁定。
            match action_byte {
                ACTION_CLOSE => {
                    if let Ok(action) =
                        CloseInstitutionAction::<T::AccountId>::decode(&mut &raw[tag.len() + 1..])
                    {
                        InstitutionPendingClose::<T>::remove(&action.account);
                    }
                }
                _ => {}
            }
        }
        Ok(ProposalExecutionOutcome::Executed)
    }

    fn on_execution_failed_terminal(proposal_id: u64) -> DispatchResult {
        let raw = match votingengine::Pallet::<T>::get_proposal_data(proposal_id) {
            Some(raw) if raw.starts_with(crate::MODULE_TAG) => raw,
            _ => return Ok(()),
        };
        let tag = crate::MODULE_TAG;
        ensure!(
            raw.len() > tag.len(),
            pallet::Error::<T>::ProposalActionNotFound
        );
        match raw[tag.len()] {
            ACTION_CLOSE => {
                let action =
                    CloseInstitutionAction::<T::AccountId>::decode(&mut &raw[tag.len() + 1..])
                        .map_err(|_| pallet::Error::<T>::ProposalActionNotFound)?;
                InstitutionPendingClose::<T>::remove(&action.account);
            }
            _ => {}
        }
        Ok(())
    }
}
