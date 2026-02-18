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
    derive_impl,
    dispatch::DispatchResult,
    parameter_types,
    traits::{
        fungible::Inspect,
        tokens::{Fortitude, Preservation},
        ConstU128, ConstU32, ConstU64, ConstU8, EnsureOrigin, FindAuthor, UnfilteredDispatchable,
        VariantCountOf,
    },
    weights::{
        constants::{RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND},
        IdentityFee, Weight,
    },
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
    OffchainTransactionFee, PalletInfo, ResolutionIssuanceGov, ResolutionIssuanceIss, Runtime,
    RuntimeCall, RuntimeEvent, RuntimeFreezeReason, RuntimeHoldReason, RuntimeOrigin,
    RuntimeRootUpgrade, RuntimeTask, System, VotingEngineSystem, BLOCK_HASH_COUNT,
    EXISTENTIAL_DEPOSIT, SLOT_DURATION, VERSION,
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
        RuntimeFeePayerExtractor,
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
            RuntimeCall::OffchainTransactionFee(
                offchain_transaction_fee::pallet::Call::submit_offchain_batch {
                    institution,
                    batch_seq,
                    batch,
                    batch_signature,
                },
            ) => {
                // 中文注释：链下批次只有通过完整预检才可进入链上扣费流程；
                // 预检失败直接按 Unknown 拒绝，避免提交账户/手续费账户被异常扣费。
                if OffchainTransactionFee::precheck_submit_offchain_batch(
                    who,
                    *institution,
                    *batch_seq,
                    batch,
                    batch_signature,
                )
                .is_ok()
                    && OffchainTransactionFee::fee_account_of(*institution).is_ok()
                {
                    let sum = batch.iter().fold(0u128, |acc, item| {
                        acc.saturating_add(item.offchain_fee_amount)
                    });
                    onchain_transaction_fee::AmountExtractResult::Amount(sum)
                } else {
                    onchain_transaction_fee::AmountExtractResult::Unknown
                }
            }
            // 中文注释：以下调用类型明确属于“无金额交易”，放行且不计算手续费。
            RuntimeCall::System(_) => onchain_transaction_fee::AmountExtractResult::NoAmount,
            RuntimeCall::Timestamp(_) => onchain_transaction_fee::AmountExtractResult::NoAmount,
            RuntimeCall::OffchainTransactionFee(_) => {
                onchain_transaction_fee::AmountExtractResult::NoAmount
            }
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
            RuntimeCall::AdminsOriginGov(_) => {
                onchain_transaction_fee::AmountExtractResult::NoAmount
            }
            RuntimeCall::RuntimeRootUpgrade(_) => {
                onchain_transaction_fee::AmountExtractResult::NoAmount
            }
            RuntimeCall::ResolutionDestroGov(_) => {
                onchain_transaction_fee::AmountExtractResult::NoAmount
            }
            // 中文注释：对 Balances 未覆盖分支按 Unknown 拒绝，避免“有金额但漏提取”。
            RuntimeCall::Balances(_) => onchain_transaction_fee::AmountExtractResult::Unknown,
        }
    }
}

pub struct RuntimeFeePayerExtractor;

impl onchain_transaction_fee::CallFeePayer<AccountId, RuntimeCall> for RuntimeFeePayerExtractor {
    fn fee_payer(_who: &AccountId, call: &RuntimeCall) -> Option<AccountId> {
        match call {
            RuntimeCall::OffchainTransactionFee(
                offchain_transaction_fee::pallet::Call::submit_offchain_batch {
                    institution,
                    ..
                },
            ) => {
                // 中文注释：链下批次上链手续费永远由本机构 fee_pallet_address 承担。
                // 提交账户只负责提交，不承担手续费。
                OffchainTransactionFee::fee_account_of(*institution).ok()
            }
            _ => None,
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

impl offchain_transaction_fee::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type MaxVerifyKeyLen = ConstU32<256>;
    type MaxBatchSize = ConstU32<200000>;
    type MaxBatchSignatureLength = ConstU32<64>;
    type MaxRelaySubmitters = ConstU32<8>;
    type InternalVoteEngine = VotingEngineSystem;
    type OffchainBatchVerifier = RuntimeOffchainBatchVerifier;
}

pub struct RuntimeOffchainBatchVerifier;

impl offchain_transaction_fee::OffchainBatchVerifier for RuntimeOffchainBatchVerifier {
    fn verify(verify_key: &[u8], message: &[u8], signature: &[u8]) -> bool {
        if verify_key.len() != 32 || signature.len() != 64 {
            return false;
        }

        let mut pk_raw = [0u8; 32];
        pk_raw.copy_from_slice(verify_key);
        let public = sr25519::Public::from_raw(pk_raw);

        let mut sig_raw = [0u8; 64];
        sig_raw.copy_from_slice(signature);
        let sig = sr25519::Signature::from_raw(sig_raw);

        sr25519_verify(&sig, message, &public)
    }
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
            nonce.as_slice(),
        );
        let msg = blake2_256(&payload.encode());

        sr25519_verify(&signature, &msg, &public)
    }
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
}

pub struct RuntimePopulationSnapshotVerifier;

impl
    voting_engine_system::PopulationSnapshotVerifier<
        AccountId,
        voting_engine_system::pallet::VoteNonceOf<Runtime>,
        voting_engine_system::pallet::VoteSignatureOf<Runtime>,
    > for RuntimePopulationSnapshotVerifier
{
    fn verify_population_snapshot(
        who: &AccountId,
        eligible_total: u64,
        nonce: &voting_engine_system::pallet::VoteNonceOf<Runtime>,
        signature: &voting_engine_system::pallet::VoteSignatureOf<Runtime>,
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
            b"GMB_CIIC_POPULATION_V1",
            frame_system::Pallet::<Runtime>::block_hash(0),
            who,
            eligible_total,
            nonce.as_slice(),
        );
        let msg = blake2_256(&payload.encode());

        sr25519_verify(&signature, &msg, &public)
    }
}

impl citizen_lightnode_issuance::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
}

parameter_types! {
    /// 决议发行治理参数（统一来源于 primitives 常量）。
    pub const ResolutionIssuanceMaxReasonLen: u32 = primitives::count_const::RESOLUTION_ISSUANCE_MAX_REASON_LEN;
    pub const ResolutionIssuanceMaxAllocations: u32 = primitives::count_const::RESOLUTION_ISSUANCE_MAX_ALLOCATIONS;
    /// Runtime 升级治理提案备注最大长度。
    pub const RuntimeUpgradeMaxReasonLen: u32 = 1024;
    /// Runtime wasm 最大长度（字节）。
    pub const RuntimeUpgradeMaxCodeSize: u32 = 5 * 1024 * 1024;
    /// 管理员治理：单机构管理员列表上限（覆盖国储会 19 人规模）。
    pub const MaxAdminsPerInstitution: u32 = 32;
}

impl admins_origin_gov::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxAdminsPerInstitution = MaxAdminsPerInstitution;
    type InternalVoteEngine = VotingEngineSystem;
}

impl resolution_destro_gov::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type InternalVoteEngine = VotingEngineSystem;
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
    let nrc_institution = primitives::reserve_nodes_const::RESERVE_NODES
        .iter()
        .find(|n| n.pallet_id == "nrcgch01")
        .and_then(|n| primitives::reserve_nodes_const::pallet_id_to_bytes(n.pallet_id))
        .expect("NRC pallet_id must be 8 bytes");

    // 中文注释：优先读取链上管理员治理模块中的“当前管理员名单”。
    if let Some(current_admins) =
        admins_origin_gov::Pallet::<Runtime>::current_admins(nrc_institution)
    {
        return current_admins.into_inner().iter().any(|admin| admin == who);
    }

    // 中文注释：兼容回退到创世静态管理员列表（尚未发生管理员替换时）。
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
    type IssuanceExecutor = ResolutionIssuanceIss;
    type JointVoteEngine = VotingEngineSystem;
    type MaxReasonLen = ResolutionIssuanceMaxReasonLen;
    type MaxAllocations = ResolutionIssuanceMaxAllocations;
    type MaxSnapshotNonceLength = ConstU32<64>;
    type MaxSnapshotSignatureLength = ConstU32<64>;
}

impl runtime_root_upgrade::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type NrcProposeOrigin = EnsureNrcAdmin;
    type JointVoteEngine = VotingEngineSystem;
    type RuntimeCodeExecutor = RuntimeSetCodeExecutor;
    type MaxReasonLen = RuntimeUpgradeMaxReasonLen;
    type MaxRuntimeCodeSize = RuntimeUpgradeMaxCodeSize;
    type MaxSnapshotNonceLength = ConstU32<64>;
    type MaxSnapshotSignatureLength = ConstU32<64>;
}

pub struct RuntimeSetCodeExecutor;

impl runtime_root_upgrade::RuntimeCodeExecutor for RuntimeSetCodeExecutor {
    fn execute_runtime_code(code: &[u8]) -> DispatchResult {
        let set_code_call = frame_system::Call::<Runtime>::set_code {
            code: code.to_vec(),
        };
        set_code_call
            .dispatch_bypass_filter(frame_system::RawOrigin::Root.into())
            .map(|_| ())
            .map_err(|e| e.error)
    }
}

pub struct RuntimeJointVoteResultCallback;

impl voting_engine_system::JointVoteResultCallback for RuntimeJointVoteResultCallback {
    fn on_joint_vote_finalized(vote_proposal_id: u64, approved: bool) -> DispatchResult {
        if resolution_issuance_gov::Pallet::<Runtime>::joint_vote_to_gov(vote_proposal_id).is_some()
        {
            return <ResolutionIssuanceGov as voting_engine_system::JointVoteResultCallback>::on_joint_vote_finalized(
                vote_proposal_id,
                approved,
            );
        }

        if runtime_root_upgrade::Pallet::<Runtime>::joint_vote_to_gov(vote_proposal_id).is_some() {
            return <RuntimeRootUpgrade as voting_engine_system::JointVoteResultCallback>::on_joint_vote_finalized(
                vote_proposal_id,
                approved,
            );
        }

        Err(sp_runtime::DispatchError::Other(
            "joint vote mapping not found",
        ))
    }
}

impl voting_engine_system::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxCiicLength = ConstU32<64>;
    type MaxVoteNonceLength = ConstU32<64>;
    type MaxVoteSignatureLength = ConstU32<64>;
    type CiicEligibility = RuntimeCiicEligibility;
    type PopulationSnapshotVerifier = RuntimePopulationSnapshotVerifier;
    type JointVoteResultCallback = RuntimeJointVoteResultCallback;
    type InternalAdminProvider = RuntimeInternalAdminProvider;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OffchainTransactionFee;
    use crate::ResolutionDestroGov;
    use frame_support::assert_ok;
    use frame_support::traits::Currency;
    use primitives::reserve_nodes_const::{
        pallet_id_to_bytes as reserve_pallet_id_to_bytes, RESERVE_NODES,
    };
    use sp_core::Pair;
    use sp_runtime::{traits::Hash as HashT, BuildStorage};
    use voting_engine_system::JointVoteResultCallback;

    fn new_test_ext() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::<Runtime>::default()
            .build_storage()
            .expect("runtime test storage should build");
        let mut ext = sp_io::TestExternalities::new(storage);
        ext.execute_with(|| {
            System::set_block_number(1);
        });
        ext
    }

    #[test]
    fn joint_vote_callback_routes_to_resolution_issuance_and_executes() {
        new_test_ext().execute_with(|| {
            let proposal_id = 1u64;
            let joint_vote_id = 99u64;
            let recipient =
                AccountId::new(primitives::reserve_nodes_const::RESERVE_NODES[1].pallet_address);
            let total_amount = 123u128;

            let reason: resolution_issuance_gov::pallet::ReasonOf<Runtime> = b"runtime-integration"
                .to_vec()
                .try_into()
                .expect("reason should fit");
            let allocations: resolution_issuance_gov::pallet::AllocationOf<Runtime> =
                vec![resolution_issuance_gov::pallet::RecipientAmount {
                    recipient: recipient.clone(),
                    amount: total_amount,
                }]
                .try_into()
                .expect("allocations should fit");

            let proposal = resolution_issuance_gov::pallet::Proposal::<Runtime> {
                proposer: recipient.clone(),
                reason: reason.clone(),
                total_amount,
                allocations: allocations.clone(),
                vote_kind: resolution_issuance_gov::pallet::VoteKind::Joint,
                status: resolution_issuance_gov::pallet::ProposalStatus::Voting,
            };

            resolution_issuance_gov::pallet::Proposals::<Runtime>::insert(proposal_id, proposal);
            resolution_issuance_gov::pallet::GovToJointVote::<Runtime>::insert(
                proposal_id,
                joint_vote_id,
            );
            resolution_issuance_gov::pallet::JointVoteToGov::<Runtime>::insert(
                joint_vote_id,
                proposal_id,
            );

            assert_ok!(RuntimeJointVoteResultCallback::on_joint_vote_finalized(
                joint_vote_id,
                true
            ));

            let updated = resolution_issuance_gov::pallet::Proposals::<Runtime>::get(proposal_id)
                .expect("proposal should exist");
            assert!(matches!(
                updated.status,
                resolution_issuance_gov::pallet::ProposalStatus::Passed
            ));
            assert!(
                resolution_issuance_gov::pallet::GovToJointVote::<Runtime>::get(proposal_id)
                    .is_none()
            );
            assert!(
                resolution_issuance_gov::pallet::JointVoteToGov::<Runtime>::get(joint_vote_id)
                    .is_none()
            );

            assert!(resolution_issuance_iss::pallet::Executed::<Runtime>::get(
                proposal_id
            ));
            assert_eq!(
                resolution_issuance_iss::pallet::TotalIssued::<Runtime>::get(),
                total_amount
            );
            assert_eq!(Balances::free_balance(&recipient), total_amount);
        });
    }

    #[test]
    fn resolution_destro_gov_internal_vote_flow_executes_destroy_and_reduces_issuance() {
        new_test_ext().execute_with(|| {
            let nrc_institution =
                reserve_pallet_id_to_bytes("nrcgch01").expect("nrc institution id must be valid");
            let nrc_account = AccountId::new(RESERVE_NODES[0].pallet_address);
            let initial_balance: Balance = 1_000;
            let destroy_amount: Balance = 100;

            let _ = Balances::deposit_creating(&nrc_account, initial_balance);
            let issuance_before = Balances::total_issuance();

            assert_ok!(ResolutionDestroGov::propose_destroy(
                RuntimeOrigin::signed(AccountId::new(RESERVE_NODES[0].admins[0])),
                voting_engine_system::internal_vote::ORG_NRC,
                nrc_institution,
                destroy_amount,
            ));

            for i in 0..13 {
                assert_ok!(ResolutionDestroGov::vote_destroy(
                    RuntimeOrigin::signed(AccountId::new(RESERVE_NODES[0].admins[i])),
                    0,
                    true,
                ));
            }

            let action = resolution_destro_gov::Pallet::<Runtime>::proposal_action(0)
                .expect("destroy action should exist");
            assert!(action.executed);

            assert_eq!(
                Balances::free_balance(&nrc_account),
                initial_balance - destroy_amount
            );
            assert_eq!(Balances::total_issuance(), issuance_before - destroy_amount);
        });
    }

    #[test]
    fn offchain_batch_submitter_is_institution_and_fee_base_is_batch_offchain_fee_sum() {
        new_test_ext().execute_with(|| {
            let institution = primitives::shengbank_nodes_const::pallet_id_to_bytes(
                primitives::shengbank_nodes_const::SHENG_BANK_NODES[0].pallet_id,
            )
            .expect("institution id should be valid");
            let institution_account = AccountId::new(
                primitives::shengbank_nodes_const::SHENG_BANK_NODES[0].pallet_address,
            );
            let payer = AccountId::new([7u8; 32]);
            let recipient = AccountId::new([8u8; 32]);

            let _ = Balances::deposit_creating(&institution_account, 1_000);
            let _ = Balances::deposit_creating(&payer, 10_000);
            let _ = Balances::deposit_creating(&recipient, 1);

            let tx_id = <Runtime as frame_system::Config>::Hashing::hash(b"rt-offchain-batch-1");
            let item =
                offchain_transaction_fee::pallet::OffchainBatchItem::<AccountId, Balance, Hash> {
                    tx_id,
                    payer: payer.clone(),
                    recipient: recipient.clone(),
                    transfer_amount: 1_000,
                    offchain_fee_amount: 1,
                };
            let batch: offchain_transaction_fee::pallet::BatchOf<Runtime> =
                vec![item].try_into().expect("batch should fit");
            let fee_account = OffchainTransactionFee::fee_account_of(institution).expect("fee");
            let relay = AccountId::new([42u8; 32]);
            let _ = Balances::deposit_creating(&relay, 1_000);
            let (pair, _) = sr25519::Pair::generate();
            let verify_key: offchain_transaction_fee::pallet::VerifyKeyOf<Runtime> =
                pair.public().0.to_vec().try_into().expect("verify key should fit");
            assert_ok!(OffchainTransactionFee::init_verify_key(
                RuntimeOrigin::signed(institution_account.clone()),
                institution,
                verify_key,
            ));
            let submitters: offchain_transaction_fee::pallet::RelaySubmittersOf<Runtime> =
                vec![relay.clone(), institution_account.clone(), payer.clone()]
                    .try_into()
                    .expect("submitters should fit");
            assert_ok!(OffchainTransactionFee::init_relay_submitters(
                RuntimeOrigin::signed(institution_account.clone()),
                institution,
                submitters,
            ));
            let batch_sig_raw = pair
                .sign(&blake2_256(&(b"GMB_OFFCHAIN_BATCH_V1", institution, 1u64, &batch).encode()))
                .0
                .to_vec();
            let batch_signature: offchain_transaction_fee::pallet::BatchSignatureOf<Runtime> =
                batch_sig_raw.try_into().expect("signature should fit");

            let call = RuntimeCall::OffchainTransactionFee(
                offchain_transaction_fee::pallet::Call::submit_offchain_batch {
                    institution,
                    batch_seq: 1,
                    batch: batch.clone(),
                    batch_signature: batch_signature.clone(),
                },
            );
            let extracted = <PowTxAmountExtractor as onchain_transaction_fee::CallAmount<
                AccountId,
                RuntimeCall,
                Balance,
            >>::amount(&institution_account, &call);
            match extracted {
                onchain_transaction_fee::AmountExtractResult::Amount(v) => assert_eq!(v, 1),
                _ => panic!("expected amount extract result"),
            }

            let before_payer = Balances::free_balance(&payer);
            let before_recipient = Balances::free_balance(&recipient);
            let before_institution = Balances::free_balance(&institution_account);
            let _ = Balances::deposit_creating(&fee_account, 1_000);
            let before_fee = Balances::free_balance(&fee_account);

            assert_ok!(OffchainTransactionFee::submit_offchain_batch(
                RuntimeOrigin::signed(relay.clone()),
                institution,
                1,
                batch,
                batch_signature,
            ));

            assert_eq!(Balances::free_balance(&payer), before_payer - 1_001);
            assert_eq!(Balances::free_balance(&recipient), before_recipient + 1_000);
            assert_eq!(
                Balances::free_balance(&institution_account),
                before_institution
            );
            assert_eq!(Balances::free_balance(&fee_account), before_fee + 1);
        });
    }
}

pub struct RuntimeInternalAdminProvider;

impl voting_engine_system::InternalAdminProvider<AccountId> for RuntimeInternalAdminProvider {
    fn is_internal_admin(
        _org: u8,
        institution: voting_engine_system::InstitutionPalletId,
        who: &AccountId,
    ) -> bool {
        admins_origin_gov::Pallet::<Runtime>::current_admins(institution)
            .map(|admins| admins.into_inner().iter().any(|admin| admin == who))
            .unwrap_or(false)
    }
}

pub struct RuntimeCiicEligibility;

impl voting_engine_system::CiicEligibility<AccountId> for RuntimeCiicEligibility {
    fn is_eligible(ciic: &[u8], who: &AccountId) -> bool {
        <ciic_code_auth::Pallet<Runtime> as ciic_code_auth::CiicEligibilityProvider<AccountId>>::is_eligible(ciic, who)
    }

    fn verify_and_consume_vote_credential(
        ciic: &[u8],
        who: &AccountId,
        proposal_id: u64,
        nonce: &[u8],
        signature: &[u8],
    ) -> bool {
        <ciic_code_auth::Pallet<Runtime> as ciic_code_auth::CiicEligibilityProvider<AccountId>>::verify_and_consume_vote_credential(
            ciic,
            who,
            proposal_id,
            nonce,
            signature,
        )
    }
}
