#![cfg_attr(not(feature = "std"), no_std)]

/// 模块标识前缀，用于在 ProposalData 中区分不同业务模块，防止跨模块误解码。
/// 长度 8 字节（`b"org-mgmt"`）；admins-change / citizenwallet / citizenapp 三方解码必须保持一致。
pub const MODULE_TAG: &[u8] = b"org-mgmt";

pub use pallet::*;
pub mod address;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod close;
pub mod common;
pub mod institution;
pub mod traits;
pub mod weights;

#[cfg(test)]
mod tests;

use admins_change::AdminAccountLifecycle;
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
    AccountValidator, CidInstitutionVerifier, InstitutionMultisigQuery, ProtectedSourceChecker,
    ReservedAccountGuard,
};
use votingengine::{
    InternalVoteEngine, InternalVoteResultCallback, ProposalExecutionOutcome, STATUS_REJECTED,
};

pub use address::{InstitutionAccountRole, RESERVED_NAME_FEE, RESERVED_NAME_MAIN};
pub use institution::types::{
    CloseInstitutionAction, CreateInstitutionAccount, CreateInstitutionAction,
    InstitutionAccountInfo, InstitutionInfo, InstitutionInitialAccount, InstitutionLifecycleStatus,
    RegisteredInstitution,
};

pub(crate) type BalanceOf<T> =
    <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::config]
    pub trait Config: frame_system::Config + votingengine::Config + admins_change::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// 内部投票引擎
        type InternalVoteEngine: votingengine::InternalVoteEngine<Self::AccountId>;

        type AccountValidator: AccountValidator<Self::AccountId>;
        type ReservedAccountChecker: ReservedAccountGuard<Self::AccountId>;
        type ProtectedSourceChecker: ProtectedSourceChecker<Self::AccountId>;
        type InstitutionAsset: institution_asset::InstitutionAsset<Self::AccountId>;
        type CidInstitutionVerifier: CidInstitutionVerifier<
            Self::AccountId,
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
        type MaxCidNumberLength: Get<u32>;

        /// 机构名称最大字节长度。
        #[pallet::constant]
        type MaxAccountNameLength: Get<u32>;

        #[pallet::constant]
        type MaxRegisterNonceLength: Get<u32>;

        #[pallet::constant]
        type MaxRegisterSignatureLength: Get<u32>;

        /// 单个机构创建交易最多可携带的账户数量。
        ///
        /// CID 默认包含主账户和费用账户，用户可新增其他账户；这里限制链上
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

    pub type AdminsOf<T> =
        BoundedVec<<T as frame_system::Config>::AccountId, <T as Config>::MaxAdmins>;

    pub type CidNumberOf<T> = BoundedVec<u8, <T as Config>::MaxCidNumberLength>;
    pub type AccountNameOf<T> = BoundedVec<u8, <T as Config>::MaxAccountNameLength>;
    pub type RegisterNonceOf<T> = BoundedVec<u8, <T as Config>::MaxRegisterNonceLength>;
    pub type RegisterSignatureOf<T> = BoundedVec<u8, <T as Config>::MaxRegisterSignatureLength>;
    /// 中文注释:注册凭证里的账户名列表,顺序必须与 CID `registration-info` 返回一致。
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
        AdminsOf<T>,
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
        CidNumberOf<T>,
        AccountNameOf<T>,
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        AdminsOf<T>,
        CreateInstitutionAccountsOf<T>,
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

    /// 机构级多签信息：key 为 cid_number。
    ///
    /// 链上创建的是“机构”，机构下账户只保存地址、初始余额与生命周期状态。
    /// 管理员长期真源在 admins-change，动态阈值长期真源在 internal-vote；
    /// 本表保存机构基本信息和创建快照。
    #[pallet::storage]
    #[pallet::getter(fn institution_of)]
    pub type Institutions<T: Config> =
        StorageMap<_, Blake2_128Concat, CidNumberOf<T>, InstitutionInfoOf<T>, OptionQuery>;

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

    /// 中文注释:已用注销凭证 nonce(防同一注销凭证重放/关多账户)。
    #[pallet::storage]
    pub type UsedDeregisterNonce<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, bool, ValueQuery>;

    /// 机构多签当前进行中的关闭提案 ID（防止并发注销提案）。
    /// 发起 propose_close 时写入，execute_close 成功或执行失败后清除。
    /// B 阶段 PendingCloseProposal 拆为两份:个人侧在 personal-manage 自持,
    /// 机构侧由本表承载,作用域只剩机构多签账户。
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
        /// 机构级创建提案已发起：创建者资金已 reserve，等待管理员投票。
        InstitutionCreateProposed {
            proposal_id: u64,
            cid_number: CidNumberOf<T>,
            cid_full_name: AccountNameOf<T>,
            main_account: T::AccountId,
            proposer: T::AccountId,
            accounts: CreateInstitutionAccountsOf<T>,
            admins: AdminsOf<T>,
            org: u8,
            admins_len: u32,
            threshold: u32,
            initial_total: BalanceOf<T>,
            reserve_total: BalanceOf<T>,
            expires_at: BlockNumberFor<T>,
        },
        /// 机构创建成功：机构和账户均已激活。
        InstitutionCreated {
            proposal_id: u64,
            cid_number: CidNumberOf<T>,
            main_account: T::AccountId,
            account_count: u32,
            initial_total: BalanceOf<T>,
            fee: BalanceOf<T>,
        },
        /// 机构创建执行失败：回滚后释放 pending 占用和 reserve 资金。
        InstitutionCreateExecutionFailed {
            proposal_id: u64,
            cid_number: CidNumberOf<T>,
            main_account: T::AccountId,
        },
        /// 机构创建提案被否决或超时清理：释放创建者 reserve 资金。
        InstitutionCreateRejected {
            proposal_id: u64,
            cid_number: CidNumberOf<T>,
            main_account: T::AccountId,
            reserve_total: BalanceOf<T>,
        },
        /// CID 机构登记
        CidInstitutionRegistered {
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
        /// 管理员数量不合法（必须 >=2）
        InvalidAdminsLen,
        /// 管理员数量与列表长度不一致
        AdminsLenMismatch,
        /// 机构账户管理员 org 只能是 ORG_PUP 或 ORG_OTH
        InvalidOrg,
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
        /// 机构登记 nonce 已被使用
        RegisterNonceAlreadyUsed,
        /// 机构签发凭证缺签发机构 CID 号。
        EmptyIssuerCidNumber,
        /// 机构签发凭证缺业务作用域省名。
        EmptyScopeProvinceName,
        /// 无法将派生地址转换为账户ID
        DerivedAccountDecodeFailed,
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
        /// propose_close 校验:仅机构地址可走本入口(个人地址转 personal-manage)。
        NotInstitutionAccount,
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
        /// 该多签账户已有进行中的关闭提案，不允许重复发起
        CloseAlreadyPending,
        /// 提案未被拒绝，不可清理
        ProposalNotRejected,
        /// 账户名占用保留角色名（"主账户"/"费用账户" 必须走 Role::Main/Fee，
        /// 禁止作为 Role::Named 的自定义命名参数）
        ReservedAccountName,
        /// sr25519 签名长度必须恰好为 64 字节
        MalformedSignature,
        /// 创世初始机构(联邦注册局/治理机构/顶层政府等)永不可注销关闭
        CannotCloseGenesisInstitution,
        /// 治理机构(国储会/省储会/省储行)永不可注销关闭
        CannotCloseGovernance,
        /// 注销凭证验签失败
        InvalidDeregisterCredential,
        /// 注销凭证 nonce 已使用(防重放)
        DeregisterNonceAlreadyUsed,
    }

    /// 提案操作类型标记：存储在 ProposalData 的第一个字节。
    /// ACTION = 1 永久保留空位,不复用。
    pub const ACTION_CLOSE: u8 = 2;
    pub const ACTION_CREATE_INSTITUTION: u8 = 3;

    /// 注销凭证作用域:整机构(关主账户=级联关全部账户)/ 单账户(只关该非主账户)。
    pub const SCOPE_INSTITUTION: u8 = 0;
    pub const SCOPE_ACCOUNT: u8 = 1;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // NOTE: `call_index` values are the on-chain ABI and must remain stable.

        // call_index = 0 永久保留空位,不复用

        /// CID 注册信息凭证批量登记机构账户地址。
        ///
        /// 中文注释:本入口与 CID `/registration-info` 对齐,业务字段只接收
        /// `cid_number / cid_full_name / account_names[]`。机构类型、企业类型、
        /// 所属法人关系只由 CID 系统用于候选资格判断,不再进入链上注册 payload。
        #[pallet::call_index(2)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::register_cid_institution())]
        pub fn register_cid_institution(
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
            crate::institution::register::do_register_cid_institution::<T>(
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

        /// 发起机构级创建提案。
        ///
        /// 该交易注册的是“机构”而不是单个账户。创建者必须一次性提交主账户、
        /// 费用账户以及需要初始化的自定义账户余额；交易发起时 reserve 创建者
        /// 的初始余额合计与手续费，投票通过后再划入机构各账户。
        #[pallet::call_index(5)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::propose_create_institution())]
        pub fn propose_create_institution(
            origin: OriginFor<T>,
            cid_number: CidNumberOf<T>,
            cid_full_name: AccountNameOf<T>,
            accounts: InstitutionInitialAccountsOf<T>,
            org: u8,
            admins_len: u32,
            admins: AdminsOf<T>,
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
            crate::institution::create::do_propose_create_institution::<T>(
                who,
                cid_number,
                cid_full_name,
                accounts,
                org,
                admins_len,
                admins,
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

        /// 发起"关闭机构多签账户"提案。
        ///
        /// 仅服务于 CID 注册机构地址(`AccountRegisteredCid` 命中);
        /// 个人多签关闭走 personal-manage::propose_close 入口,
        /// 输入个人地址会返回 `Error::NotInstitutionAccount`。
        #[pallet::call_index(1)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::propose_close())]
        #[allow(clippy::too_many_arguments)]
        pub fn propose_close(
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
        /// 清理已被拒绝或超时的创建/关闭提案残留状态(机构侧)。
        /// 任意签名账户可调用。用于解决投票引擎 on_initialize 超时 reject 后
        /// 本模块无法自动收到通知导致的 Pending / InstitutionPendingClose 残留。
        ///
        /// B 阶段后仅处理 ACTION_CREATE_INSTITUTION 与 ACTION_CLOSE 两类机构提案;
        /// 个人多签的清理由 personal-manage::cleanup_rejected_proposal 自持。
        #[pallet::call_index(4)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::cleanup_rejected_proposal())]
        pub fn cleanup_rejected_proposal(origin: OriginFor<T>, proposal_id: u64) -> DispatchResult {
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
        /// 按角色派生机构多签账户地址（所有机构统一走这条路径）。
        ///
        /// 派生统一调用 `primitives::core_const::derive_account`：
        /// - `Main` → `OP_MAIN + cid_number`
        /// - `Fee`  → `OP_FEE + cid_number`
        /// - `Named(account_name)` → `OP_INSTITUTION + cid_number + account_name`
        ///
        /// 保留名校验：`Named(b"主账户")`/`Named(b"费用账户")` 被拒绝（强制走 `Main`/`Fee`
        /// 分支避免命名空间重叠）；`永久质押`/`安全基金`/`两和基金` 为制度专属账户，普通机构
        /// 禁止注册，命中即拒（均返回 `ReservedAccountName`）。空 account_name 的 `Named`
        /// 返回 `EmptyAccountName`。
        pub fn derive_institution_account(
            cid_number: &[u8],
            role: InstitutionAccountRole<'_>,
        ) -> Result<T::AccountId, DispatchError> {
            let (op_tag, name_suffix): (u8, &[u8]) = match role {
                InstitutionAccountRole::Main => (primitives::core_const::OP_MAIN, &[]),
                InstitutionAccountRole::Fee => (primitives::core_const::OP_FEE, &[]),
                InstitutionAccountRole::Named(n) => {
                    ensure!(!n.is_empty(), Error::<T>::EmptyAccountName);
                    ensure!(
                        n != RESERVED_NAME_MAIN
                            && n != RESERVED_NAME_FEE
                            && !primitives::core_const::is_forbidden_account_name(n),
                        Error::<T>::ReservedAccountName
                    );
                    (primitives::core_const::OP_INSTITUTION, n)
                }
            };
            let mut payload = cid_number.to_vec();
            payload.extend_from_slice(name_suffix);
            let digest =
                primitives::core_const::derive_account(op_tag, T::SS58Prefix::get(), &payload);
            T::AccountId::decode(&mut &digest[..])
                .map_err(|_| Error::<T>::DerivedAccountDecodeFailed.into())
        }

        /// 把 CID 账户名 bytes 翻译成 `InstitutionAccountRole`：
        /// - `"主账户"` → `Main`
        /// - `"费用账户"` → `Fee`
        /// - `"永久质押"`/`"安全基金"`/`"两和基金"` → `ReservedAccountName`（制度专属，禁止注册）
        /// - 其他非空 → `Named(account_name)`
        /// - 空 → 返回 `EmptyAccountName`
        ///
        /// 这是 `register_cid_institution` 等 extrinsic 的唯一入口——禁止调用方
        /// 绕开此函数直接构造 `Role::Named("主账户")`（虽然 `derive_institution_account`
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
            } else if primitives::core_const::is_forbidden_account_name(account_name) {
                // 永久质押/安全基金/两和基金 为制度专属账户，普通 CID 机构禁止注册。
                Err(Error::<T>::ReservedAccountName.into())
            } else {
                Ok(InstitutionAccountRole::Named(account_name))
            }
        }

        // derive_personal_account 已迁至 personal-manage::Pallet,
        // organization-manage 不再提供该派生(机构地址只走 derive_institution_account)。

        pub(crate) fn ensure_unique_admins(admins: &AdminsOf<T>) -> Result<(), DispatchError> {
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

        pub(crate) fn ensure_admin_config(
            who: &T::AccountId,
            admins_len: u32,
            admins: &AdminsOf<T>,
            threshold: u32,
        ) -> DispatchResult {
            ensure!(T::MaxAdmins::get() >= 2, Error::<T>::InvalidRuntimeConfig);
            ensure!(admins_len >= 2, Error::<T>::InvalidAdminsLen);
            ensure!(
                admins.len() as u32 == admins_len,
                Error::<T>::AdminsLenMismatch
            );
            ensure!(
                threshold > 0
                    && threshold <= admins_len
                    && u64::from(threshold).saturating_mul(2) > u64::from(admins_len),
                Error::<T>::InvalidThreshold
            );
            Self::ensure_unique_admins(admins)?;
            ensure!(
                admins.iter().any(|admin| admin == who),
                Error::<T>::PermissionDenied
            );
            Ok(())
        }

        pub(crate) fn create_pending_admin_account_for_proposal(
            proposal_id: u64,
            org: u8,
            institution_id: T::AccountId,
            kind: admins_change::AdminAccountKind,
            admins: &AdminsOf<T>,
            creator: &T::AccountId,
        ) -> DispatchResult {
            admins_change::Pallet::<T>::create_pending_admin_account_for_proposal(
                proposal_id,
                crate::MODULE_TAG,
                institution_id,
                org,
                kind,
                admins.iter().cloned().collect(),
                creator.clone(),
            )
        }

        pub(crate) fn activate_admin_account(
            proposal_id: u64,
            institution_id: T::AccountId,
        ) -> DispatchResult {
            admins_change::Pallet::<T>::activate_admin_account_for_proposal(
                proposal_id,
                crate::MODULE_TAG,
                institution_id,
            )
        }

        pub(crate) fn remove_pending_admin_account(proposal_id: u64, institution_id: T::AccountId) {
            let _ = admins_change::Pallet::<T>::remove_pending_admin_account_for_proposal(
                proposal_id,
                crate::MODULE_TAG,
                institution_id,
            );
        }

        pub(crate) fn close_admin_account(
            proposal_id: u64,
            institution_id: T::AccountId,
        ) -> DispatchResult {
            admins_change::Pallet::<T>::close_admin_account_for_proposal(
                proposal_id,
                crate::MODULE_TAG,
                institution_id,
            )
        }

        /// 从任意机构多签账户反查其管理员账户账户地址。
        ///
        /// 个人多签由 personal-manage 自持；本函数仅服务机构账户。
        /// 中文注释:管理员属于机构(不属于账户)。任意机构账户(主/费用/自定义)都解析到
        /// 本机构【主账户】——即 admins-change 里承载该机构唯一管理员集的键。这样机构管理员
        /// 统一管理机构及其全部账户(创建/注销账户都由这套管理员授权)。
        pub fn resolve_admin_account_for_account(account: &T::AccountId) -> Option<T::AccountId> {
            let registered = AccountRegisteredCid::<T>::get(account)?;
            let institution = Institutions::<T>::get(&registered.cid_number)?;
            Some(institution.main_account)
        }

        /// 从任意机构账户反查管理员更换 org。
        ///
        /// 中文注释:机构账户必须使用 ORG_PUP/ORG_OTH；ORG_REN 只属于个人多签。
        pub fn resolve_org_for_account(account: &T::AccountId) -> Option<u8> {
            let registered = AccountRegisteredCid::<T>::get(account)?;
            Institutions::<T>::get(&registered.cid_number).map(|inst| inst.org)
        }

        // account_names_payload_from_initial_accounts 已迁至 institution::accounts。

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

        // 投票回调执行体:
        // - ACTION_CLOSE → crate::close::execute_institution_close_with_finalizer
        // - ACTION_CREATE_INSTITUTION 与 cleanup → crate::institution::execute
        // (ACTION_CREATE_PERSONAL 已在 B 阶段迁至 personal-manage 独立 pallet)
    }
}

// ──── InstitutionMultisigQuery 实现:对 duoqian-transfer / runtime config 暴露查询 ────
//
// 输入任意机构账户(主/费用/自创),直接以账户地址读取 admins-change 账户。
// 再按 Institutions[cid_number].org 读取 org。这条路径保证
// 机构账户只使用 ORG_PUP/ORG_OTH,不再把机构账户错误塞到 ORG_REN。

impl<T: pallet::Config> traits::InstitutionMultisigQuery<T::AccountId> for pallet::Pallet<T> {
    fn lookup_org(addr: &T::AccountId) -> Option<u8> {
        pallet::Pallet::<T>::resolve_org_for_account(addr)
    }

    fn lookup_admin_config(
        addr: &T::AccountId,
    ) -> Option<primitives::multisig::MultisigConfigSnapshot<T::AccountId>> {
        let org = Self::lookup_org(addr)?;
        let account = pallet::Pallet::<T>::resolve_admin_account_for_account(addr)?;
        let admins = admins_change::Pallet::<T>::active_account_admins(org, account.clone())?;
        let threshold =
            <T as Config>::InternalVoteEngine::active_dynamic_threshold(org, account.clone())?;
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

// ──── 投票终态回调:把已通过的多签创建/关闭提案落地到链上 ────
//
// 投票统一由投票引擎承担,提案通过(或否决)经
// [`votingengine::InternalVoteResultCallback`] 广播回来。
// 本 Executor(机构侧):
// - 按 `MODULE_TAG + ACTION_CLOSE / ACTION_CREATE_INSTITUTION` 前缀认领机构提案;
// - `approved = true` → 分派到 `institution::execute::execute_create_institution_with_finalizer`
//   / `close::execute_institution_close_with_finalizer`;执行失败发事件,不回滚投票
//   (提案保留 PASSED,可用 cleanup_rejected_proposal 或手动重试处理);
// - `approved = false` → 清理 Pending 存储(InstitutionPendingClose 等),释放地址占用。
// (ACTION_CREATE_PERSONAL 已在 B 阶段迁至 personal-manage::InternalVoteExecutor)
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
                ACTION_CREATE_INSTITUTION => {
                    let action = CreateInstitutionActionOf::<T>::decode(&mut &raw[tag.len() + 1..])
                        .map_err(|_| pallet::Error::<T>::ProposalActionNotFound)?;
                    let exec_result = with_transaction(|| {
                        match crate::institution::execute::execute_create_institution_with_finalizer::<
                            T,
                        >(proposal_id, &action, true)
                        {
                            Ok(()) => TransactionOutcome::Commit(Ok(())),
                            Err(e) => TransactionOutcome::Rollback(Err(e)),
                        }
                    });
                    if exec_result.is_err() {
                        pallet::Pallet::<T>::deposit_event(
                            pallet::Event::<T>::InstitutionCreateExecutionFailed {
                                proposal_id,
                                cid_number: action.cid_number,
                                main_account: action.main_account,
                            },
                        );
                        return Ok(ProposalExecutionOutcome::RetryableFailed);
                    }
                    return Ok(ProposalExecutionOutcome::Executed);
                }
                ACTION_CLOSE => {
                    let action =
                        CloseInstitutionAction::<T::AccountId>::decode(&mut &raw[tag.len() + 1..])
                            .map_err(|_| pallet::Error::<T>::ProposalActionNotFound)?;
                    let exec_result = with_transaction(|| {
                        match crate::close::execute_institution_close_with_finalizer::<T>(
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
            // 否决:清理 Pending 记录释放地址锁定。
            match action_byte {
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
