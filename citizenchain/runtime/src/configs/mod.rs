// This is free and unencumbered software released into the public domain.
//
// Anyone is free to copy, modify, publish, use, compile, sell, or
// distribute this software, either in source code form or as a compiled
// binary, for any purpose, commercial or non-commercial, and by any
// means.
//
// In jurisdictions that recognize copyright laws, the author or authors
// of this software dedicate any and all copyright interest in the
// software to the public domain. We make this dedication for the benefit
// of the public at large and to the detriment of our heirs and
// successors. We intend this dedication to be an overt act of
// relinquishment in perpetuity of all present and future rights to this
// software under copyright law.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
// IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
// OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
// ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
// OTHER DEALINGS IN THE SOFTWARE.
//
// For more information, please refer to <http://unlicense.org>

// Substrate and Polkadot dependencies
use codec::Decode;
use codec::Encode;
use frame_support::{
    derive_impl, parameter_types,
    traits::{
        fungible::Inspect,
        tokens::{Fortitude, Preservation},
        ConstU128, ConstU32, ConstU64, ConstU8, EnsureOrigin, FindAuthor, VariantCountOf,
    },
    weights::{
        constants::{RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND},
        IdentityFee, Weight,
    },
    PalletId,
};
use frame_system::limits::{BlockLength, BlockWeights};
use pallet_transaction_payment::{ConstFeeMultiplier, Multiplier};
use sp_core::sr25519;
use sp_io::{crypto::sr25519_verify, hashing::blake2_256};
use sp_runtime::{traits::One, Perbill};
use sp_version::RuntimeVersion;

// Local module imports
use super::{
    AccountId, Balance, Balances, Block, BlockNumber, CitizenLightnodeIssuance, Hash, Nonce,
    PalletInfo, ResolutionIssuanceGov, ResolutionIssuanceIss, Runtime, RuntimeCall, RuntimeEvent,
    RuntimeFreezeReason, RuntimeHoldReason, RuntimeOrigin, RuntimeTask, System, VotingEngineSystem,
    BLOCK_HASH_COUNT, EXISTENTIAL_DEPOSIT, SLOT_DURATION, VERSION,
};

const NORMAL_DISPATCH_RATIO: Perbill =
    Perbill::from_percent(primitives::core_const::NORMAL_DISPATCH_PERCENT);

parameter_types! {
    pub const BlockHashCount: BlockNumber = BLOCK_HASH_COUNT;
    pub const Version: RuntimeVersion = VERSION;

    /// 每个区块允许 60 秒计算预算（weight ref_time）。
    pub RuntimeBlockWeights: BlockWeights = BlockWeights::with_sensible_defaults(
        Weight::from_parts(60u64 * WEIGHT_REF_TIME_PER_SECOND, u64::MAX),
        NORMAL_DISPATCH_RATIO,
    );
    pub RuntimeBlockLength: BlockLength =
        BlockLength::max_with_normal_ratio(primitives::core_const::MAX_BLOCK_BYTES, NORMAL_DISPATCH_RATIO);
    /// 公民币主链地址编号（SS58 前缀）：统一来源于 primitives 常量。
    pub const SS58Prefix: u16 = primitives::core_const::SS58_FORMAT;
}

/// All migrations of the runtime, aside from the ones declared in the pallets.
///
/// This can be a tuple of types, each implementing `OnRuntimeUpgrade`.
#[allow(unused_parens)]
type SingleBlockMigrations = ();

/// The default types are being injected by [`derive_impl`](`frame_support::derive_impl`) from
/// [`SoloChainDefaultConfig`](`struct@frame_system::config_preludes::SolochainDefaultConfig`),
/// but overridden as needed.
#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig)]
impl frame_system::Config for Runtime {
    /// The block type for the runtime.
    type Block = Block;
    /// Block & extrinsics weights: base values and limits.
    type BlockWeights = RuntimeBlockWeights;
    /// The maximum length of a block (in bytes).
    type BlockLength = RuntimeBlockLength;
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The type for storing how many extrinsics an account has signed.
    type Nonce = Nonce;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    /// The weight of database operations that the runtime can invoke.
    type DbWeight = RocksDbWeight;
    /// Version of the runtime.
    type Version = Version;
    /// The data to be stored in an account.
    type AccountData = pallet_balances::AccountData<Balance>;
    /// 地址显示编号（SS58 前缀），统一来自 primitives 制度常量。
    type SS58Prefix = SS58Prefix;
    type MaxConsumers = frame_support::traits::ConstU32<16>;
    type SingleBlockMigrations = SingleBlockMigrations;
}

impl pallet_timestamp::Config for Runtime {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    // 纯 PoW 共识：时间戳不再依赖 Aura 插槽回调。
    type OnTimestampSet = ();
    type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
    type WeightInfo = ();
}

impl pallet_balances::Config for Runtime {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// The ubiquitous event type.
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
    type FreezeIdentifier = RuntimeFreezeReason;
    type MaxFreezes = VariantCountOf<RuntimeFreezeReason>;
    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeFreezeReason = RuntimeFreezeReason;
    type DoneSlashHandler = ();
}

parameter_types! {
    pub FeeMultiplier: Multiplier = Multiplier::one();
}

impl pallet_transaction_payment::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnChargeTransaction = onchain_transaction_fee::PowOnchainChargeAdapter<
        Balances,
        onchain_transaction_fee::PowOnchainFeeRouter<Runtime, Balances, PowDigestAuthor>,
        PowTxAmountExtractor,
    >;
    type OperationalFeeMultiplier = ConstU8<{ primitives::core_const::OPERATIONAL_FEE_MULTIPLIER }>;
    type WeightToFee = IdentityFee<Balance>;
    type LengthToFee = IdentityFee<Balance>;
    type FeeMultiplierUpdate = ConstFeeMultiplier<FeeMultiplier>;
    type WeightInfo = pallet_transaction_payment::weights::SubstrateWeight<Runtime>;
}

pub struct PowTxAmountExtractor;

impl onchain_transaction_fee::CallAmount<AccountId, RuntimeCall, Balance> for PowTxAmountExtractor {
    fn amount(
        who: &AccountId,
        call: &RuntimeCall,
    ) -> onchain_transaction_fee::AmountExtractResult<Balance> {
        match call {
            RuntimeCall::Balances(pallet_balances::Call::transfer_allow_death {
                value, ..
            }) => onchain_transaction_fee::AmountExtractResult::Amount(*value),
            RuntimeCall::Balances(pallet_balances::Call::transfer_keep_alive { value, .. }) => {
                onchain_transaction_fee::AmountExtractResult::Amount(*value)
            }
            RuntimeCall::Balances(pallet_balances::Call::force_transfer { value, .. }) => {
                onchain_transaction_fee::AmountExtractResult::Amount(*value)
            }
            RuntimeCall::Balances(pallet_balances::Call::force_unreserve { amount, .. }) => {
                onchain_transaction_fee::AmountExtractResult::Amount(*amount)
            }
            RuntimeCall::Balances(pallet_balances::Call::force_set_balance {
                new_free, ..
            }) => onchain_transaction_fee::AmountExtractResult::Amount(*new_free),
            RuntimeCall::Balances(pallet_balances::Call::force_adjust_total_issuance {
                delta,
                ..
            }) => onchain_transaction_fee::AmountExtractResult::Amount(*delta),
            RuntimeCall::Balances(pallet_balances::Call::burn { value, .. }) => {
                onchain_transaction_fee::AmountExtractResult::Amount(*value)
            }
            RuntimeCall::Balances(pallet_balances::Call::transfer_all { keep_alive, .. }) => {
                let preservation = if *keep_alive {
                    Preservation::Preserve
                } else {
                    Preservation::Expendable
                };
                let value = <Balances as Inspect<AccountId>>::reducible_balance(
                    who,
                    preservation,
                    Fortitude::Polite,
                );
                onchain_transaction_fee::AmountExtractResult::Amount(value)
            }
            // 中文注释：以下调用类型明确属于“无金额交易”，放行且不计算手续费。
            RuntimeCall::System(_) => onchain_transaction_fee::AmountExtractResult::NoAmount,
            RuntimeCall::Timestamp(_) => onchain_transaction_fee::AmountExtractResult::NoAmount,
            RuntimeCall::FullnodePowReward(_) => {
                onchain_transaction_fee::AmountExtractResult::NoAmount
            }
            RuntimeCall::ResolutionIssuanceIss(_) => {
                onchain_transaction_fee::AmountExtractResult::NoAmount
            }
            RuntimeCall::ResolutionIssuanceGov(_) => {
                onchain_transaction_fee::AmountExtractResult::NoAmount
            }
            RuntimeCall::VotingEngineSystem(_) => {
                onchain_transaction_fee::AmountExtractResult::NoAmount
            }
            RuntimeCall::CiicCodeAuth(_) => onchain_transaction_fee::AmountExtractResult::NoAmount,
            RuntimeCall::CitizenLightnodeIssuance(_) => {
                onchain_transaction_fee::AmountExtractResult::NoAmount
            }
            // 中文注释：对 Balances 未覆盖分支按 Unknown 拒绝，避免“有金额但漏提取”。
            RuntimeCall::Balances(_) => onchain_transaction_fee::AmountExtractResult::Unknown,
        }
    }
}

/// 省储行质押利息模块配置：
/// - 使用 Balances 作为铸币/记账货币
/// - 每年区块数统一采用 primitives 中的制度常量
impl shengbank_stake_interest::Config for Runtime {
    type Currency = Balances;
    type BlocksPerYear = ConstU64<{ primitives::pow_const::BLOCKS_PER_YEAR }>;
}

/// PoW 作者解析器：
/// 从区块 pre-runtime digest 中读取 POW_ENGINE_ID 的负载，并解码为 AccountId。
pub struct PowDigestAuthor;

impl FindAuthor<AccountId> for PowDigestAuthor {
    fn find_author<'a, I>(digests: I) -> Option<AccountId>
    where
        I: 'a + IntoIterator<Item = (sp_runtime::ConsensusEngineId, &'a [u8])>,
    {
        digests.into_iter().find_map(|(engine_id, data)| {
            if engine_id == sp_consensus_pow::POW_ENGINE_ID {
                AccountId::decode(&mut &data[..]).ok()
            } else {
                None
            }
        })
    }
}

/// 全节点 PoW 奖励模块配置：
/// - 链上货币使用 Balances
/// - 作者识别完全基于 PoW digest（不依赖 Aura/Grandpa）
impl fullnode_pow_reward::Config for Runtime {
    type Currency = Balances;
    type FindAuthor = PowDigestAuthor;
}

/// CIIC 系统发布的一次性绑定凭证验签公钥（sr25519）。
/// TODO: 上线前替换为真实 CIIC 系统公钥（32 字节）。
const CIIC_VERIFY_PUBKEY: [u8; 32] = [0u8; 32];

pub struct RuntimeCiicVerifier;

impl
    ciic_code_auth::CiicVerifier<
        AccountId,
        Hash,
        ciic_code_auth::pallet::NonceOf<Runtime>,
        ciic_code_auth::pallet::SignatureOf<Runtime>,
    > for RuntimeCiicVerifier
{
    fn verify(
        account: &AccountId,
        credential: &ciic_code_auth::pallet::CredentialOf<Runtime>,
    ) -> bool {
        let sig_bytes = credential.signature.as_slice();
        if sig_bytes.len() != 64 {
            return false;
        }

        let mut sig_raw = [0u8; 64];
        sig_raw.copy_from_slice(sig_bytes);
        let signature = sr25519::Signature::from_raw(sig_raw);
        let public = sr25519::Public::from_raw(CIIC_VERIFY_PUBKEY);

        let payload = (
            b"GMB_CIIC_BIND_V1",
            frame_system::Pallet::<Runtime>::block_hash(0),
            account,
            credential.ciic_code_hash,
            credential.nonce.as_slice(),
        );
        let msg = blake2_256(&payload.encode());

        sr25519_verify(&signature, &msg, &public)
    }
}

pub struct RuntimeCiicVoteVerifier;

impl
    ciic_code_auth::CiicVoteVerifier<
        AccountId,
        Hash,
        ciic_code_auth::pallet::NonceOf<Runtime>,
        ciic_code_auth::pallet::SignatureOf<Runtime>,
    > for RuntimeCiicVoteVerifier
{
    fn verify_vote(
        account: &AccountId,
        ciic_hash: Hash,
        proposal_id: u64,
        eligible_total: u64,
        nonce: &ciic_code_auth::pallet::NonceOf<Runtime>,
        signature: &ciic_code_auth::pallet::SignatureOf<Runtime>,
    ) -> bool {
        let sig_bytes = signature.as_slice();
        if sig_bytes.len() != 64 {
            return false;
        }

        let mut sig_raw = [0u8; 64];
        sig_raw.copy_from_slice(sig_bytes);
        let signature = sr25519::Signature::from_raw(sig_raw);
        let public = sr25519::Public::from_raw(CIIC_VERIFY_PUBKEY);

        let payload = (
            b"GMB_CIIC_VOTE_V1",
            frame_system::Pallet::<Runtime>::block_hash(0),
            account,
            ciic_hash,
            proposal_id,
            eligible_total,
            nonce.as_slice(),
        );
        let msg = blake2_256(&payload.encode());

        sr25519_verify(&signature, &msg, &public)
    }
}

parameter_types! {
    /// 中文注释：可参与公民投票总人数，后续可替换为治理参数或链上同步值。
    pub const CiicEligibleVoterCount: u64 = 0;
}

impl ciic_code_auth::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxCiicLength = ConstU32<64>;
    type MaxCredentialNonceLength = ConstU32<64>;
    // 中文注释：CIIC 绑定与投票验签统一使用 64 字节原始 sr25519 签名。
    type MaxCredentialSignatureLength = ConstU32<64>;
    type CiicVerifier = RuntimeCiicVerifier;
    type CiicVoteVerifier = RuntimeCiicVoteVerifier;
    type OnCiicBound = CitizenLightnodeIssuance;
    type EligibleVoterCount = CiicEligibleVoterCount;
}

impl citizen_lightnode_issuance::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
}

parameter_types! {
    /// 决议发行治理参数（统一来源于 primitives 常量）。
    pub const ResolutionIssuanceMaxReasonLen: u32 = primitives::count_const::RESOLUTION_ISSUANCE_MAX_REASON_LEN;
    pub const ResolutionIssuanceMaxAllocations: u32 = primitives::count_const::RESOLUTION_ISSUANCE_MAX_ALLOCATIONS;
}

pub struct NrcPalletIdProvider;
impl frame_support::traits::Get<PalletId> for NrcPalletIdProvider {
    fn get() -> PalletId {
        // 中文注释：国储会ID统一从常量数组读取并转码。
        let nrc_id_bytes = primitives::reserve_nodes_const::RESERVE_NODES
            .iter()
            .find(|n| n.pallet_id == "nrcgch01")
            .and_then(|n| primitives::reserve_nodes_const::pallet_id_to_bytes(n.pallet_id))
            .expect("NRC pallet_id must be 8 bytes");
        PalletId(nrc_id_bytes)
    }
}

/// 禁用特权原点：始终拒绝任何 Origin，确保不存在可被调用的特权入口。
pub struct EnsureNoPrivilegeOrigin;

impl EnsureOrigin<RuntimeOrigin> for EnsureNoPrivilegeOrigin {
    type Success = ();

    fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
        Err(o)
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn try_successful_origin() -> Result<RuntimeOrigin, ()> {
        Err(())
    }
}

pub struct EnsureNrcAdmin;

impl EnsureOrigin<RuntimeOrigin> for EnsureNrcAdmin {
    type Success = AccountId;

    fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
        let who = frame_system::EnsureSigned::<AccountId>::try_origin(o)?;
        if is_nrc_admin(&who) {
            Ok(who)
        } else {
            Err(RuntimeOrigin::from(frame_system::RawOrigin::Signed(who)))
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn try_successful_origin() -> Result<RuntimeOrigin, ()> {
        Err(())
    }
}

fn is_nrc_admin(who: &AccountId) -> bool {
    let who_bytes = who.encode();
    primitives::reserve_nodes_const::RESERVE_NODES
        .iter()
        .find(|n| n.pallet_id == "nrcgch01")
        .map(|nrc| {
            nrc.admins
                .iter()
                .any(|admin| admin.as_slice() == who_bytes.as_slice())
        })
        .unwrap_or(false)
}

impl resolution_issuance_iss::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    // 中文注释：协议层封死特权入口，执行发行不接受任何外部特权调用。
    type ExecuteOrigin = EnsureNoPrivilegeOrigin;
    type MaxReasonLen = ResolutionIssuanceMaxReasonLen;
    type MaxAllocations = ResolutionIssuanceMaxAllocations;
}

impl resolution_issuance_gov::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type NrcProposeOrigin = EnsureNrcAdmin;
    type NrcPalletId = NrcPalletIdProvider;
    type IssuanceExecutor = ResolutionIssuanceIss;
    type JointVoteEngine = VotingEngineSystem;
    type MaxReasonLen = ResolutionIssuanceMaxReasonLen;
    type MaxAllocations = ResolutionIssuanceMaxAllocations;
}

impl voting_engine_system::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxCiicLength = ConstU32<64>;
    type MaxVoteNonceLength = ConstU32<64>;
    type MaxVoteSignatureLength = ConstU32<64>;
    type CiicEligibility = RuntimeCiicEligibility;
    type JointVoteResultCallback = ResolutionIssuanceGov;
}

pub struct RuntimeCiicEligibility;

impl voting_engine_system::CiicEligibility<AccountId> for RuntimeCiicEligibility {
    fn is_eligible(ciic: &[u8], who: &AccountId) -> bool {
        <ciic_code_auth::Pallet<Runtime> as ciic_code_auth::CiicEligibilityProvider<AccountId>>::is_eligible(ciic, who)
    }

    fn eligible_voter_count() -> u64 {
        <ciic_code_auth::Pallet<Runtime> as ciic_code_auth::CiicEligibilityProvider<AccountId>>::eligible_voter_count()
    }

    fn verify_and_consume_vote_credential(
        ciic: &[u8],
        who: &AccountId,
        proposal_id: u64,
        eligible_total: u64,
        nonce: &[u8],
        signature: &[u8],
    ) -> bool {
        <ciic_code_auth::Pallet<Runtime> as ciic_code_auth::CiicEligibilityProvider<AccountId>>::verify_and_consume_vote_credential(
            ciic,
            who,
            proposal_id,
            eligible_total,
            nonce,
            signature,
        )
    }
}
