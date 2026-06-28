#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub mod apis;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod configs;

extern crate alloc;
use alloc::vec::Vec;
use codec::{Decode, DecodeWithMemTracking, Encode};
use frame_support::weights::Weight;
use frame_support::{dispatch::DispatchInfo, pallet_prelude::TransactionSource};
use scale_info::TypeInfo;
use sp_runtime::{
    generic,
    traits::{
        AsSystemOriginSigner, BlakeTwo256, DispatchInfoOf, Dispatchable, IdentifyAccount,
        PostDispatchInfoOf, TransactionExtension, ValidateResult, Verify,
    },
    transaction_validity::{InvalidTransaction, TransactionValidityError},
    DispatchResult, MultiAddress, MultiSignature,
};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

pub use frame_system::Call as SystemCall;
pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

pub mod genesis;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
    use super::*;
    use sp_consensus_grandpa::AuthorityId as GrandpaId;
    use sp_runtime::impl_opaque_keys;
    use sp_runtime::{
        generic,
        traits::{BlakeTwo256, Hash as HashT},
    };

    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

    /// Opaque block header type.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    /// Opaque block identifier type.
    pub type BlockId = generic::BlockId<Block>;
    /// Opaque block hash type.
    pub type Hash = <BlakeTwo256 as HashT>::Output;

    impl_opaque_keys! {
        pub struct SessionKeys {
            pub grandpa: GrandpaId,
        }
    }
}

// To learn more about runtime versioning, see:
// https://docs.substrate.io/main-docs/build/upgrade#runtime-versioning
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: alloc::borrow::Cow::Borrowed("citizenchain"),
    impl_name: alloc::borrow::Cow::Borrowed("citizenchain"),
    authoring_version: 0,
    // The version of the runtime specification. A full node will not attempt to use its native
    //   runtime in substitute for the on-chain Wasm runtime unless all of `spec_name`,
    //   `spec_version`, and `authoring_version` are the same between Wasm and native.
    // 当前 runtime 采用统一模块命名：
    // genesis-admins / public-admins / private-admins / personal-admins /
    // organization-manage / votingengine /
    // multisig-transfer / offchain-transaction / onchain-transaction / institution-asset。
    spec_version: 1,
    impl_version: 0,
    apis: apis::RUNTIME_API_VERSIONS,
    transaction_version: 0,
    system_version: 0,
};

mod block_times {
    /// This determines the average expected block time that we are targeting. Blocks will be
    /// produced at a minimum duration defined by `SLOT_DURATION`.
    ///
    /// Change this to adjust the block time. Unified source: primitives::pow_const.
    pub const MILLI_SECS_PER_BLOCK: u64 = primitives::pow_const::MILLISECS_PER_BLOCK;

    // NOTE: Currently it is not possible to change the slot duration after the chain has started.
    // Attempting to do so will brick block production.
    pub const SLOT_DURATION: u64 = MILLI_SECS_PER_BLOCK;
}
pub use block_times::*;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLI_SECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

pub const BLOCK_HASH_COUNT: BlockNumber = primitives::core_const::BLOCK_HASH_COUNT;

// 货币单位统一为“分”体系：1 表示 1 分，100 分 = 1 元。
pub const FEN: Balance = 1;
pub const YUAN: Balance = 100 * FEN;

// UNIT 别名指向 1 元（100 分）。
pub const UNIT: Balance = YUAN;

/// 账户存在最小余额（单位：分），统一采用 primitives 制度常量。
pub const EXISTENTIAL_DEPOSIT: Balance = primitives::core_const::ACCOUNT_EXISTENTIAL_DEPOSIT;

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Nonce = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// An index to a block.
pub type BlockNumber = u32;

/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, ()>;

/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;

/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;

/// The `TransactionExtension` to the basic transaction logic.
pub type TxExtension = (
    frame_system::AuthorizeCall<Runtime>,
    frame_system::CheckNonZeroSender<Runtime>,
    CheckNonStakeSender,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
    frame_metadata_hash_extension::CheckMetadataHash<Runtime>,
    frame_system::WeightReclaim<Runtime>,
);

#[derive(Encode, Decode, DecodeWithMemTracking, Clone, Eq, PartialEq, TypeInfo, Debug)]
pub struct CheckNonStakeSender;

impl TransactionExtension<RuntimeCall> for CheckNonStakeSender
where
    RuntimeCall: Dispatchable<Info = DispatchInfo>,
    <RuntimeCall as Dispatchable>::RuntimeOrigin: AsSystemOriginSigner<AccountId> + Clone,
{
    const IDENTIFIER: &'static str = "CheckNonStakeSender";
    type Implicit = ();
    type Val = ();
    type Pre = ();

    fn weight(&self, _call: &RuntimeCall) -> Weight {
        Weight::zero()
    }

    fn validate(
        &self,
        origin: RuntimeOrigin,
        _call: &RuntimeCall,
        _info: &DispatchInfoOf<RuntimeCall>,
        _len: usize,
        _self_implicit: Self::Implicit,
        _inherited_implication: &impl Encode,
        _source: TransactionSource,
    ) -> ValidateResult<Self::Val, RuntimeCall> {
        if let Some(who) = origin.as_system_origin_signer() {
            if configs::is_stake_account(&who) {
                return Err(InvalidTransaction::Call.into());
            }
        }
        Ok((Default::default(), (), origin))
    }

    fn prepare(
        self,
        _val: Self::Val,
        _origin: &RuntimeOrigin,
        _call: &RuntimeCall,
        _info: &DispatchInfoOf<RuntimeCall>,
        _len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        Ok(())
    }

    fn post_dispatch_details(
        _pre: Self::Pre,
        _info: &DispatchInfo,
        _post_info: &PostDispatchInfoOf<RuntimeCall>,
        _len: usize,
        _result: &DispatchResult,
    ) -> Result<Weight, TransactionValidityError> {
        Ok(Weight::zero())
    }
}

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
    generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, TxExtension>;

/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, TxExtension>;

/// Runtime upgrade migrations 集合。
///
/// 本链全新创世,无任何历史链上数据需要迁移,故为空元组。
/// 将来链上线后如需 storage 迁移,在此 tuple 按声明顺序挂入 `OnRuntimeUpgrade`。
pub type Migrations = ();

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
    Migrations,
>;

// Create the runtime by composing the FRAME pallets that were previously configured.
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
    pub struct Runtime;

    #[runtime::pallet_index(0)]
    pub type System = frame_system;

    #[runtime::pallet_index(1)]
    pub type Timestamp = pallet_timestamp;

    // 纯 PoW 出块 + GRANDPA 最终性。
    #[runtime::pallet_index(2)]
    pub type Balances = pallet_balances;

    #[runtime::pallet_index(3)]
    pub type TransactionPayment = pallet_transaction_payment;

    #[runtime::pallet_index(15)]
    pub type Grandpa = pallet_grandpa;

    // 链上交易手续费模块：发出 FeePaid 事件，供客户端读取真实手续费
    #[runtime::pallet_index(4)]
    pub type OnchainTransaction = onchain_transaction::pallet;

    // 省储行利息模块：按年度给固定省储行账户发放质押利息
    #[runtime::pallet_index(5)]
    pub type ProvincialBankInterest = provincialbank_interest;

    // 全节点发行模块：出块成功后发放固定铸块奖励
    #[runtime::pallet_index(6)]
    pub type FullnodeIssuance = fullnode_issuance;

    // 决议发行模块：完整承载提案、联合投票回调、发行执行与维护审计
    #[runtime::pallet_index(8)]
    pub type ResolutionIssuance = resolution_issuance;

    // 投票引擎核心:Proposals/反向索引/状态机/快照/锁/清理共用基础设施
    #[runtime::pallet_index(9)]
    pub type VotingEngine = votingengine;

    // 内部投票 sub-pallet:管理员一人一票
    #[runtime::pallet_index(22)]
    pub type InternalVote = internal_vote;

    // 联合投票 sub-pallet:管理员多签 + 联合公投两阶段
    #[runtime::pallet_index(23)]
    pub type JointVote = joint_vote;

    // 选举投票 sub-pallet:选举公职人员(普选 + 机构成员互选)
    #[runtime::pallet_index(24)]
    pub type ElectionVote = election_vote;

    // CID 绑定与资格校验：统一处理绑定、验签、资格查询
    #[runtime::pallet_index(10)]
    pub type CidSystem = cid_system;

    // 公民发行：仅负责认证奖励发放
    #[runtime::pallet_index(11)]
    pub type CitizenIssuance = citizen_issuance;

    // 创世管理员模块：国储会、省储会、省储行、联邦注册局管理员。
    #[runtime::pallet_index(12)]
    pub type GenesisAdmins = genesis_admins;

    // 运行时升级治理模块：提案与联合投票通过后触发 set_code。
    #[runtime::pallet_index(13)]
    pub type RuntimeUpgrade = runtime_upgrade;

    // 决议销毁治理模块：本机构内部投票通过后销毁本机构交易地址余额
    #[runtime::pallet_index(14)]
    pub type ResolutionDestro = resolution_destro;

    // GRANDPA 密钥治理模块：国储会/省储会内部投票通过后替换 GRANDPA 投票公钥
    #[runtime::pallet_index(16)]
    pub type GrandpaKeyChange = grandpakey_change;

    // 机构管理模块：机构多签账户的注册/创建/关闭与半数签名校验（注册型多签，非宪法保留主账户）
    #[runtime::pallet_index(17)]
    pub type OrganizationManage = organization_manage;

    // 个人多签管理员模块:用户自定义多签账户的注册/创建/关闭(无 CID 归属,creator+account_name 派生)。
    // pallet_index=7。
    #[runtime::pallet_index(7)]
    pub type PersonalAdmins = personal_admins;

    // PoW 动态难度调整模块：每 600 块根据实际出块速度自动调整挖矿难度
    #[runtime::pallet_index(18)]
    pub type PowDifficulty = pow_difficulty;

    // 机构多签账户转账模块：治理机构内部投票通过后从 main_account 转账（宪法保留主账户，注册型 account）
    #[runtime::pallet_index(19)]
    pub type MultisigTransfer = multisig_transfer;

    // 创世模块：存储创世期/运行期阶段、出块目标时间、开发者直升开关、创世常量
    #[runtime::pallet_index(20)]
    pub type GenesisPallet = genesis_pallet;

    // 链下交易清算模块：省储行即时清算、批量上链、绑定清算行、费率治理
    #[runtime::pallet_index(21)]
    pub type OffchainTransaction = offchain_transaction::pallet;

    // 链上发行代币(Plain FT, ADR-011):用户(CID 机构 + personal-admins 多签)发行 GMB 之外的代币。
    // 唯一外壳入口,内核挂 pallet_assets;pallet_assets 原生 extrinsic 由 BaseCallFilter 屏蔽。
    // 当前为空壳(任务卡 A/B 未实装),OnchainIssuance 自身 propose_* 也在 RuntimeCallFilter 中 reject。
    #[runtime::pallet_index(25)]
    pub type OnchainIssuance = onchain_issuance;

    // pallet_assets 内核:多资产基础设施,所有原生 extrinsic 在 RuntimeCallFilter 中 reject。
    // 业务调用一律经由 OnchainIssuance::propose_* → InternalVote/JointVote callback → 内部 root 调用。
    #[runtime::pallet_index(26)]
    pub type Assets = pallet_assets;

    // 立法院模块:法律结构化上链 + 修法走立法投票(ADR-027)。业务壳,只承载法律数据与提案入口;
    // 表决/计票/公投归属 legislation-vote sub-pallet。
    #[runtime::pallet_index(27)]
    pub type LegislationYuan = legislation_yuan;

    // 立法投票 sub-pallet:立法机构专属投票(单院/两院/特别案强制公投,ADR-027)。
    // 投票引擎「头等模式」PROPOSAL_KIND_LEGISLATION,共享核心共享基础,只本地存计票账本。
    #[runtime::pallet_index(28)]
    pub type LegislationVote = legislation_vote;

    // 非创世公权机构管理员模块。
    #[runtime::pallet_index(29)]
    pub type PublicAdmins = public_admins;

    // 私权机构管理员模块；归属私法人的非法人由上层显式路由到这里。
    #[runtime::pallet_index(30)]
    pub type PrivateAdmins = private_admins;
}

#[cfg(test)]
mod tests;
