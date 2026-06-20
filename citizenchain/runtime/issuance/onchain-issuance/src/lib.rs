//! # 链上发行代币模块(onchain-issuance)
//!
//! GMB 链上"发行 GMB 之外的其他人代币"业务 pallet。第一期承载:
//!
//! - **Plain FT(同质化代币,无锚定声明)**:发行人 = SFID 注册机构 + personal-manage 个人多签
//! - **GMB 唯一计费**:创建一次性收 1000 GMB(`primitives::fee_policy::ONCHAIN_ASSET_CREATE_FEE`)
//! - **NRC 强制 monitor**:链端通过 `NrcMainAccountProvider` 全局解析,不在每条资产 storage 冗余
//! - **业务 InternalVote / 监管 JointVote**:沿用 unified_voting_entry phase 4 铁律,
//!   业务 pallet 不暴露 wrapper extrinsic,前端直调 VotingEngine
//!
//! ## 与 pallet_assets 内核的关系
//!
//! 本模块是 **唯一外壳入口**,内核挂载 `pallet_assets`(Substrate 多资产 pallet),
//! 所有原生 extrinsic 在 runtime `BaseCallFilter` 中 reject。业务调用必须经由
//! `OnchainIssuance::propose_*` → InternalVote/JointVote 通过 → callback 回调 →
//! 内部以 root 调用 `pallet_assets`。
//!
//! ## 协议位
//!
//! 用户代币用 `asset_id` 做资产编号；发行与治理账户统一使用机构多签 `AccountId`。
//!
//! ## 模块文件
//!
//! - `types.rs`     — 共用数据类型(AssetMeta / AssetClass / AssetState 等)
//! - `proposal.rs`  — ACTION 常量 + 提案体定义
//! - `validation.rs` — 入参校验(decimals 范围 / 发行机构资格 / 黑名单 hit)
//! - `blacklist.rs` — 字符串黑名单 storage + 默认词表
//! - `fee.rs`       — 创建费 reserve / unreserve / transfer(ADR-011 v2 押金机制)
//! - `execution.rs` — 业务路径(issue/mint/burn/close/transfer)桥接 pallet_assets
//! - `monitor.rs`   — NRC 监管 5 动作执行
//! - `weights.rs`   — WeightInfo 占位
//! - `benchmarks.rs` — runtime-benchmarks 占位
//! - `tests/`       — mock runtime + 业务/监管/黑名单测试

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod blacklist;
pub mod execution;
pub mod fee;
pub mod monitor;
pub mod proposal;
#[cfg(test)]
mod tests;
pub mod types;
pub mod validation;
pub mod weights;

pub use pallet::*;

/// 模块标识前缀,用于在投票引擎 ProposalData 中识别 onchain-issuance 提案。
///
/// 中文注释:与 ADR-011 第十节固定,跨端识别用户代币业务提案的稳定业务标签。
pub const MODULE_TAG: &[u8] = b"onc-iss";

#[frame_support::pallet]
// 中文注释:框架阶段 deposit_event 自动生成的 fn 暂未被业务调用,
// 后续任务卡 A/B 实装时会消费;先抑制 dead_code 以便整体 cargo check 干净。
#[allow(dead_code)]
pub mod pallet {
    use crate::{types::OnchainAssetMeta, weights::WeightInfo};
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, ReservableCurrency},
    };
    use frame_system::ensure_signed;
    use frame_system::pallet_prelude::{BlockNumberFor, OriginFor};
    use sp_std::vec::Vec;

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// pallet_assets 内核 AssetId(u32)。
    ///
    /// 中文注释:onchain-issuance 对外使用 `asset_id`，它只表示资产编号。
    pub type OnchainAssetId = u32;

    #[pallet::config]
    pub trait Config: frame_system::Config + votingengine::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// GMB 计费货币(创建费 reserve/transfer / 投票统一价等扣款入口)。
        ///
        /// 中文注释:必须实现 ReservableCurrency 以支持 propose_issue 押金 reserve
        /// (ADR-011 v2 第六节计费机制 — 押金通过/否决时分别 transfer/退还)。
        type Currency: ReservableCurrency<Self::AccountId>;

        /// pallet_assets 内核类型绑定。runtime 接线时把 `pallet_assets::Pallet<Runtime>` 接到此处。
        type Assets: frame_support::traits::tokens::fungibles::Create<
                Self::AccountId,
                AssetId = OnchainAssetId,
                Balance = BalanceOf<Self>,
            > + frame_support::traits::tokens::fungibles::Mutate<Self::AccountId>;

        /// **NRC 治理账户(治理多签 main_account)** 提供器:
        /// - monitor 5 动作的 origin 校验:proposer ∈ admins(NRC main account)
        /// - 监管 JointVote 提案的发起人识别
        ///
        /// ADR-011 v2 修订项 #1:与 `NrcFeeAccountProvider` 语义分离,
        /// 不可复用同一 trait(v1 错误地复用了 onchain_transaction::NrcAccountProvider)。
        type NrcMainAccountProvider: NrcMainAccountProvider<Self::AccountId>;

        /// **NRC 费用账户(收创建费 fee_account)** 提供器:
        /// - 1000 GMB 创建费 unreserve 后 transfer 的目标
        /// - 与 onchain_transaction::NrcAccountProvider 语义一致(都是收钱账户)
        type NrcFeeAccountProvider: NrcFeeAccountProvider<Self::AccountId>;

        /// 资产元数据字符串字段最大长度(name / symbol / description)。
        #[pallet::constant]
        type MaxAssetNameLen: Get<u32>;
        #[pallet::constant]
        type MaxAssetSymbolLen: Get<u32>;
        #[pallet::constant]
        type MaxAssetDescriptionLen: Get<u32>;

        /// 黑名单单词最大长度与黑名单总条目上限。
        #[pallet::constant]
        type MaxBlacklistWordLen: Get<u32>;
        #[pallet::constant]
        type MaxBlacklistEntries: Get<u32>;

        /// 监管动作 reason_hash 字节长度(固定 32B = sha256)。
        #[pallet::constant]
        type ReasonHashLen: Get<u32>;

        /// `ForceCloseSchedule` 单块到期处理上限(防 on_finalize 单块过载)。
        #[pallet::constant]
        type MaxScheduledPerBlock: Get<u32>;

        type WeightInfo: WeightInfo;
    }

    /// **NRC 治理账户(治理多签 main_account)** trait — ADR-011 v2 拆分语义后的独立 trait。
    ///
    /// 实装位置:`runtime/src/configs/mod.rs::RuntimeNrcMainAccountProvider`,
    /// 返回 `china_cb[0].main_account`。
    pub trait NrcMainAccountProvider<AccountId> {
        fn nrc_main_account() -> Option<AccountId>;
    }

    /// **NRC 费用账户(收创建费 fee_account)** trait — ADR-011 v2 拆分语义后的独立 trait。
    ///
    /// 实装位置:`runtime/src/configs/mod.rs::RuntimeNrcAccountProvider`(复用既有,
    /// 返回 `china_cb[0].fee_account`,与 onchain_transaction::NrcAccountProvider 同源)。
    pub trait NrcFeeAccountProvider<AccountId> {
        fn nrc_fee_account() -> Option<AccountId>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// asset_id → 资产元数据。
    ///
    /// 中文注释:用户代币的唯一权威 storage,记录 issuer / class / decimals / state。
    #[pallet::storage]
    #[pallet::getter(fn asset_meta)]
    pub type Assets<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        OnchainAssetId,
        OnchainAssetMeta<T::AccountId>,
        OptionQuery,
    >;

    /// 下一个待分配的 AssetId(u32 自增,从 1 开始)。
    ///
    /// 中文注释:不复用 pallet_assets 自身的 id 池,onchain-issuance 自管单调递增,
    /// 简化迁移与审计。close 后的 asset_id 永久标记为 Closed,不复用。
    #[pallet::storage]
    #[pallet::getter(fn next_asset_id)]
    pub type NextAssetId<T: Config> = StorageValue<_, OnchainAssetId, ValueQuery>;

    /// 字符串黑名单:`name / symbol / description` 写入前过滤。
    ///
    /// 中文注释:GenesisConfig 注入默认词表(法币/锚定/权威/数字货币 4 类),
    /// 后续添词/删词走 RuntimeUpgrade 投票,不可 sudo。
    #[pallet::storage]
    #[pallet::getter(fn blacklist)]
    pub type Blacklist<T: Config> = StorageValue<
        _,
        BoundedVec<BoundedVec<u8, T::MaxBlacklistWordLen>, T::MaxBlacklistEntries>,
        ValueQuery,
    >;

    /// 1000 GMB 创建费押金跟踪:proposal_id → (proposer, reserved_amount)。
    ///
    /// ADR-011 v2 第六节铁律:propose_issue 时 reserve 1000 GMB → 通过则 unreserve+transfer 给 NRC fee_account;
    /// 否决/过期则 unreserve 退还。本 storage 用于 callback 阶段反查 proposer 与押金额度。
    #[pallet::storage]
    #[pallet::getter(fn issue_deposit)]
    pub type IssueDeposit<T: Config> =
        StorageMap<_, Twox64Concat, u64, (T::AccountId, BalanceOf<T>), OptionQuery>;

    /// 强制销毁倒计时调度队列:expire_block → 该块到期的 asset_id 列表。
    ///
    /// ADR-011 v2 修订项 #5:NRC `monitor_force_close` 入此队列,30 天后由 `on_finalize`
    /// 通过 `ForceCloseSchedule::take(n)` O(1) 取出处理,**避免全表扫描 Assets**。
    #[pallet::storage]
    #[pallet::getter(fn force_close_schedule)]
    pub type ForceCloseSchedule<T: Config> = StorageMap<
        _,
        Twox64Concat,
        BlockNumberFor<T>,
        BoundedVec<OnchainAssetId, T::MaxScheduledPerBlock>,
        ValueQuery,
    >;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        /// 黑名单初始词表(法币 / 锚定 / 权威 / 数字货币 4 类)。
        pub initial_blacklist: Vec<Vec<u8>>,
        #[doc(hidden)]
        pub _phantom: PhantomData<T>,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                initial_blacklist: crate::blacklist::default_blacklist_words(),
                _phantom: PhantomData,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            // 中文注释:黑名单初始化,超长词或超过条目上限 silently 跳过(应在编译期保证 fixture 正确)。
            let mut bounded: BoundedVec<
                BoundedVec<u8, T::MaxBlacklistWordLen>,
                T::MaxBlacklistEntries,
            > = BoundedVec::new();
            for word in &self.initial_blacklist {
                let bw: BoundedVec<u8, T::MaxBlacklistWordLen> = match word.clone().try_into() {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                if bounded.try_push(bw).is_err() {
                    break;
                }
            }
            Blacklist::<T>::put(bounded);
            NextAssetId::<T>::put(1u32);
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 用户代币创建成功。
        AssetIssued {
            asset_id: OnchainAssetId,
            issuer: T::AccountId,
        },
        /// 用户代币增发。
        Minted {
            asset_id: OnchainAssetId,
            to: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// 用户代币销毁。
        Burned {
            asset_id: OnchainAssetId,
            who: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// 用户代币转账。
        Transferred {
            asset_id: OnchainAssetId,
            from: T::AccountId,
            to: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// 用户代币关闭(发行方主动)。
        AssetClosed { asset_id: OnchainAssetId },
        /// 创建费押金已 reserve(propose_issue 阶段)。
        IssueDepositReserved {
            proposal_id: u64,
            who: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// 创建费押金已 transfer 给 NRC fee_account(callback 通过)。
        IssueDepositCharged {
            proposal_id: u64,
            who: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// 创建费押金已退还(callback 否决/过期)。
        IssueDepositRefunded {
            proposal_id: u64,
            who: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// NRC 监管:冻结特定持仓。
        MonitorFrozen {
            asset_id: OnchainAssetId,
            who: T::AccountId,
            reason_hash: [u8; 32],
        },
        /// NRC 监管:解冻。
        MonitorUnfrozen {
            asset_id: OnchainAssetId,
            who: T::AccountId,
            reason_hash: [u8; 32],
        },
        /// NRC 监管:强制 burn(扣押)。
        MonitorConfiscated {
            asset_id: OnchainAssetId,
            who: T::AccountId,
            amount: BalanceOf<T>,
            reason_hash: [u8; 32],
        },
        /// NRC 监管:强制划转。
        MonitorForceTransferred {
            asset_id: OnchainAssetId,
            from: T::AccountId,
            to: T::AccountId,
            amount: BalanceOf<T>,
            reason_hash: [u8; 32],
        },
        /// NRC 监管:整币封禁(已入 ForceCloseSchedule,30 天后销毁)。
        MonitorForceCloseScheduled {
            asset_id: OnchainAssetId,
            expire_block: BlockNumberFor<T>,
            reason_hash: [u8; 32],
        },
        /// NRC 监管:整币封禁到期,持仓销毁完成。
        MonitorForceCloseExecuted { asset_id: OnchainAssetId },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 发行机构账户不允许。
        IssuerNotAllowed,
        /// decimals 越界(必须 0..=18)。
        DecimalsOutOfRange,
        /// 字段命中字符串黑名单(法币 / 锚定 / 权威 / 数字货币词)。
        BlacklistedWord,
        /// 资产不存在。
        AssetNotFound,
        /// 资产已关闭/封禁,不可操作。
        AssetClosed,
        /// 提案体解码失败。
        InvalidProposalData,
        /// NRC 治理账户未配置。
        NrcMainAccountMissing,
        /// NRC 费用账户未配置。
        NrcFeeAccountMissing,
        /// AssetId 溢出(u32 自增达到上限)。
        AssetIdOverflow,
        /// pallet_assets 内核错误。
        AssetsInternal,
        /// 字符串字段超长。
        FieldTooLong,
        /// 黑名单条目数已达上限。
        BlacklistFull,
        /// 资产 class 暂不支持(第一期仅 Plain)。
        UnsupportedAssetClass,
        /// propose origin 校验未通过(业务 ACTION:proposer 不在 issuer admins;
        /// 监管 ACTION:proposer 不在 NRC admins)。
        ProposeOriginNotAllowed,
        /// metadata 不可修改(ADR-011 v2 第 5.7 节铁律)。
        MetadataImmutable,
        /// 创建费 reserve 失败(GMB 余额不足)。
        InsufficientBalanceForDeposit,
        /// 单块强制销毁队列已满。
        ScheduleFull,
    }

    /// ADR-011 v3:业务 pallet 暴露 10 个 propose_X extrinsic(call_index 0..=4 业务 / 10..=14 监管)。
    ///
    /// 不暴露 execute/cancel wrapper(走 VotingEngine::retry_passed_proposal 9.4 / cancel_passed_proposal 9.5)。
    /// 框架阶段每个 fn 只 ensure_signed + 占位 stub,业务逻辑(propose origin 校验 / reserve 押金 /
    /// internal_vote::do_create_internal_proposal 等)在后续任务卡 A/B 实装。
    ///
    /// call_index 5..=9 / 15+ 留洞不复用(永久 ABI)。
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // ---------- 业务 propose_X(InternalVote)----------

        /// 创建用户代币提案。propose 时 reserve 1000 GMB 押金,callback 通过/否决时 transfer/refund。
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::issue())]
        pub fn propose_issue(
            origin: OriginFor<T>,
            issuer_account: T::AccountId,
            class: crate::types::AssetClass,
            name: BoundedVec<u8, T::MaxAssetNameLen>,
            symbol: BoundedVec<u8, T::MaxAssetSymbolLen>,
            description: BoundedVec<u8, T::MaxAssetDescriptionLen>,
            decimals: u8,
            initial_supply: BalanceOf<T>,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            // 业务逻辑参数防 unused 警告(框架阶段)
            let _ = (
                issuer_account,
                class,
                name,
                symbol,
                description,
                decimals,
                initial_supply,
            );
            // TODO: implement business logic (任务卡 A)
            //   1. validation::ensure_issuer_allowed / ensure_decimals_in_range / ensure_class_supported
            //   2. ensure proposer ∈ admins(issuer_account)
            //   3. 字段过黑名单
            //   4. fee::reserve_creation_deposit(&who, proposal_id)
            //   5. InternalVoteEngine::create_general_internal_proposal_with_data(MODULE_TAG + ACTION_OAIS + scale-encoded fields)
            Ok(())
        }

        /// 增发提案。
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::mint())]
        pub fn propose_mint(
            origin: OriginFor<T>,
            asset_id: OnchainAssetId,
            to: T::AccountId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            let _ = (asset_id, to, amount);
            // TODO: implement business logic (任务卡 A)
            Ok(())
        }

        /// 销毁提案。
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::burn())]
        pub fn propose_burn(
            origin: OriginFor<T>,
            asset_id: OnchainAssetId,
            from: T::AccountId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            let _ = (asset_id, from, amount);
            // TODO: implement business logic (任务卡 A)
            Ok(())
        }

        /// 关闭代币提案(发行方主动)。
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::close())]
        pub fn propose_close(origin: OriginFor<T>, asset_id: OnchainAssetId) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            let _ = asset_id;
            // TODO: implement business logic (任务卡 A)
            Ok(())
        }

        /// 转账提案。
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::transfer())]
        pub fn propose_transfer(
            origin: OriginFor<T>,
            asset_id: OnchainAssetId,
            from: T::AccountId,
            to: T::AccountId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            let _ = (asset_id, from, to, amount);
            // TODO: implement business logic (任务卡 A)
            Ok(())
        }

        // ---------- 监管 propose_monitor_X(JointVote)----------

        /// NRC 监管:冻结持仓提案。
        #[pallet::call_index(10)]
        #[pallet::weight(<T as Config>::WeightInfo::monitor_freeze())]
        pub fn propose_monitor_freeze(
            origin: OriginFor<T>,
            asset_id: OnchainAssetId,
            who: T::AccountId,
            reason_hash: [u8; 32],
        ) -> DispatchResult {
            let _proposer = ensure_signed(origin)?;
            let _ = (asset_id, who, reason_hash);
            // TODO: implement business logic (任务卡 B)
            //   ensure proposer ∈ admins(NRC 机构多签 AccountId)
            //   JointVoteEngine::create_joint_proposal_with_data(...)
            Ok(())
        }

        /// NRC 监管:解冻持仓提案。
        #[pallet::call_index(11)]
        #[pallet::weight(<T as Config>::WeightInfo::monitor_unfreeze())]
        pub fn propose_monitor_unfreeze(
            origin: OriginFor<T>,
            asset_id: OnchainAssetId,
            who: T::AccountId,
            reason_hash: [u8; 32],
        ) -> DispatchResult {
            let _proposer = ensure_signed(origin)?;
            let _ = (asset_id, who, reason_hash);
            // TODO: implement business logic (任务卡 B)
            Ok(())
        }

        /// NRC 监管:强制 burn(扣押)提案。
        #[pallet::call_index(12)]
        #[pallet::weight(<T as Config>::WeightInfo::monitor_confiscate())]
        pub fn propose_monitor_confiscate(
            origin: OriginFor<T>,
            asset_id: OnchainAssetId,
            who: T::AccountId,
            amount: BalanceOf<T>,
            reason_hash: [u8; 32],
        ) -> DispatchResult {
            let _proposer = ensure_signed(origin)?;
            let _ = (asset_id, who, amount, reason_hash);
            // TODO: implement business logic (任务卡 B)
            Ok(())
        }

        /// NRC 监管:强制划转(追赃)提案。
        #[pallet::call_index(13)]
        #[pallet::weight(<T as Config>::WeightInfo::monitor_force_transfer())]
        pub fn propose_monitor_force_transfer(
            origin: OriginFor<T>,
            asset_id: OnchainAssetId,
            from: T::AccountId,
            to: T::AccountId,
            amount: BalanceOf<T>,
            reason_hash: [u8; 32],
        ) -> DispatchResult {
            let _proposer = ensure_signed(origin)?;
            let _ = (asset_id, from, to, amount, reason_hash);
            // TODO: implement business logic (任务卡 B)
            Ok(())
        }

        /// NRC 监管:整币封禁(30 天后销毁)提案。
        #[pallet::call_index(14)]
        #[pallet::weight(<T as Config>::WeightInfo::monitor_force_close())]
        pub fn propose_monitor_force_close(
            origin: OriginFor<T>,
            asset_id: OnchainAssetId,
            reason_hash: [u8; 32],
        ) -> DispatchResult {
            let _proposer = ensure_signed(origin)?;
            let _ = (asset_id, reason_hash);
            // TODO: implement business logic (任务卡 B)
            Ok(())
        }
    }
}
