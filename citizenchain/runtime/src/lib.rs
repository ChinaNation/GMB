#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub mod apis;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod configs;

extern crate alloc;
use alloc::vec::Vec;
use sp_runtime::{
    generic,
    traits::{BlakeTwo256, IdentifyAccount, Verify},
    MultiAddress, MultiSignature,
};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

pub use frame_system::Call as SystemCall;
pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

pub mod genesis_config_presets;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
    use super::*;
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
}

// To learn more about runtime versioning, see:
// https://docs.substrate.io/main-docs/build/upgrade#runtime-versioning
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: alloc::borrow::Cow::Borrowed("gmb-runtime"),
    impl_name: alloc::borrow::Cow::Borrowed("gmb-runtime"),
    authoring_version: 1,
    // The version of the runtime specification. A full node will not attempt to use its native
    //   runtime in substitute for the on-chain Wasm runtime unless all of `spec_name`,
    //   `spec_version`, and `authoring_version` are the same between Wasm and native.
    // This value is set to 100 to notify Polkadot-JS App (https://polkadot.js.org/apps) to use
    //   the compatible custom types.
    spec_version: 1,
    impl_version: 1,
    apis: apis::RUNTIME_API_VERSIONS,
    transaction_version: 1,
    system_version: 1,
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

// 为兼容模板中可能使用的 UNIT 命名，保留 UNIT 并指向 1 元（100 分）。
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

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
    generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, TxExtension>;

/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, TxExtension>;

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
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

    // 纯 PoW 共识链：移除 Aura/Grandpa，保留基础系统与业务模块。
    #[runtime::pallet_index(2)]
    pub type Balances = pallet_balances;

    #[runtime::pallet_index(3)]
    pub type TransactionPayment = pallet_transaction_payment;

    // 链下交易手续费与清算上链模块
    #[runtime::pallet_index(4)]
    pub type OffchainTransactionFee = offchain_transaction_fee;

    // 省储行质押利息模块：按年度给固定省储行账户发放质押利息
    #[runtime::pallet_index(5)]
    pub type ShengBankStakeInterest = shengbank_stake_interest;

    // 全节点 PoW 发行模块：出块成功后发放固定铸块奖励
    #[runtime::pallet_index(6)]
    pub type FullnodePowReward = fullnode_pow_reward;

    // 决议发行执行模块：仅执行，不负责提案/投票
    #[runtime::pallet_index(7)]
    pub type ResolutionIssuanceIss = resolution_issuance_iss;

    // 决议发行治理模块：负责提案与联合投票流程
    #[runtime::pallet_index(8)]
    pub type ResolutionIssuanceGov = resolution_issuance_gov;

    // 投票引擎模块：提供联合投票/内部投票/公民投票
    #[runtime::pallet_index(9)]
    pub type VotingEngineSystem = voting_engine_system;

    // SFID 绑定与资格校验：统一处理绑定、验签、资格查询
    #[runtime::pallet_index(10)]
    pub type SfidCodeAuth = sfid_code_auth;

    // 公民轻节点发行：仅负责认证奖励发放
    #[runtime::pallet_index(11)]
    pub type CitizenLightnodeIssuance = citizen_lightnode_issuance;

    // 管理员治理模块：本机构管理员更换事项（走内部投票）
    #[runtime::pallet_index(12)]
    pub type AdminsOriginGov = admins_origin_gov;

    // 运行时升级治理模块：提案与联合投票通过后触发 set_code。
    #[runtime::pallet_index(13)]
    pub type RuntimeRootUpgrade = runtime_root_upgrade;

    // 决议销毁治理模块：本机构内部投票通过后销毁本机构交易地址余额
    #[runtime::pallet_index(14)]
    pub type ResolutionDestroGov = resolution_destro_gov;

    // 多签交易模块：duoqian_address 创建/注销与半数签名校验
    #[runtime::pallet_index(17)]
    pub type DuoqianTransactionPow = duoqian_transaction_pow;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_and_currency_constants_are_consistent() {
        assert_eq!(YUAN, 100 * FEN);
        assert_eq!(UNIT, YUAN);
        assert_eq!(HOURS, MINUTES * 60);
        assert_eq!(DAYS, HOURS * 24);
        assert_eq!(SLOT_DURATION, MILLI_SECS_PER_BLOCK);
    }

    #[test]
    fn runtime_version_and_block_types_are_sane() {
        assert_eq!(VERSION.spec_name.as_ref(), "gmb-runtime");
        assert_eq!(VERSION.impl_name.as_ref(), "gmb-runtime");
        assert!(VERSION.spec_version >= 1);

        let _opaque_block_id: opaque::BlockId = generic::BlockId::Number(0);
        let _runtime_block_id: BlockId = generic::BlockId::Number(0);
    }
}
