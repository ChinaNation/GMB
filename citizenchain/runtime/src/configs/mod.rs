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
        ConstU128, ConstU32, ConstU64, ConstU8, Contains, EnsureOrigin, FindAuthor,
        UnfilteredDispatchable, VariantCountOf,
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
use sp_runtime::{
    traits::{AccountIdConversion, IdentifyAccount, One},
    MultiSigner, Perbill,
};
use sp_version::RuntimeVersion;

// Local module imports
use super::{
    AccountId, Address, Balance, Balances, Block, BlockNumber, CitizenLightnodeIssuance, Hash,
    Nonce, OffchainTransactionFee, PalletInfo, ResolutionIssuanceGov, ResolutionIssuanceIss,
    Runtime, RuntimeCall, RuntimeEvent, RuntimeFreezeReason, RuntimeHoldReason, RuntimeOrigin,
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

pub fn is_keyless_account(address: &AccountId) -> bool {
    primitives::china::china_ch::CHINA_CH
        .iter()
        .any(|n| address == &AccountId::new(n.keyless_address))
}

fn is_keyless_multi_address(address: &Address) -> bool {
    match address {
        sp_runtime::MultiAddress::Id(account) => is_keyless_account(account),
        sp_runtime::MultiAddress::Address32(raw) => is_keyless_account(&AccountId::new(*raw)),
        sp_runtime::MultiAddress::Raw(raw) if raw.len() == 32 => {
            let mut out = [0u8; 32];
            out.copy_from_slice(raw.as_slice());
            is_keyless_account(&AccountId::new(out))
        }
        _ => false,
    }
}

pub struct RuntimeCallFilter;

impl Contains<RuntimeCall> for RuntimeCallFilter {
    fn contains(call: &RuntimeCall) -> bool {
        match call {
            RuntimeCall::Balances(pallet_balances::Call::force_transfer { source, .. }) => {
                !is_keyless_multi_address(source)
            }
            _ => true,
        }
    }
}

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
    /// 中文注释：全局调用过滤器，禁止从 keyless_address 发起 force_transfer。
    type BaseCallFilter = RuntimeCallFilter;
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
        onchain_transaction_fee::PowOnchainFeeRouter<
            Runtime,
            Balances,
            PowDigestAuthor,
            RuntimeNrcAccountProvider,
        >,
        PowTxAmountExtractor,
        RuntimeFeePayerExtractor,
    >;
    type OperationalFeeMultiplier = ConstU8<{ primitives::core_const::OPERATIONAL_FEE_MULTIPLIER }>;
    type WeightToFee = IdentityFee<Balance>;
    type LengthToFee = IdentityFee<Balance>;
    type FeeMultiplierUpdate = ConstFeeMultiplier<FeeMultiplier>;
    type WeightInfo = pallet_transaction_payment::weights::SubstrateWeight<Runtime>;
}

pub struct RuntimeNrcAccountProvider;

impl onchain_transaction_fee::NrcAccountProvider<AccountId> for RuntimeNrcAccountProvider {
    fn nrc_account() -> Option<AccountId> {
        Some(AccountId::new(
            primitives::china::china_cb::CHINA_CB[0].duoqian_address,
        ))
    }
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
                    let sum = batch.iter().fold(0u128, |acc: u128, item| {
                        acc.saturating_add(item.offchain_fee_amount)
                    });
                    onchain_transaction_fee::AmountExtractResult::Amount(sum)
                } else {
                    onchain_transaction_fee::AmountExtractResult::Unknown
                }
            }
            RuntimeCall::OffchainTransactionFee(
                offchain_transaction_fee::pallet::Call::process_queued_batch { queue_id },
            ) => {
                if let Ok(sum) =
                    OffchainTransactionFee::precheck_process_queued_batch(who, *queue_id)
                {
                    onchain_transaction_fee::AmountExtractResult::Amount(sum)
                } else {
                    onchain_transaction_fee::AmountExtractResult::Unknown
                }
            }
            RuntimeCall::DuoqianTransactionPow(
                duoqian_transaction_pow::pallet::Call::create_duoqian { amount, .. },
            ) => onchain_transaction_fee::AmountExtractResult::Amount(*amount),
            RuntimeCall::DuoqianTransactionPow(
                duoqian_transaction_pow::pallet::Call::close_duoqian {
                    duoqian_address, ..
                },
            ) => onchain_transaction_fee::AmountExtractResult::Amount(Balances::free_balance(
                duoqian_address,
            )),
            // 中文注释：以下调用类型明确属于“无金额交易”，放行且不计算手续费。
            RuntimeCall::System(_) => onchain_transaction_fee::AmountExtractResult::NoAmount,
            RuntimeCall::Timestamp(_) => onchain_transaction_fee::AmountExtractResult::NoAmount,
            RuntimeCall::OffchainTransactionFee(_) => {
                onchain_transaction_fee::AmountExtractResult::NoAmount
            }
            RuntimeCall::FullnodePowReward(_) => {
                onchain_transaction_fee::AmountExtractResult::NoAmount
            }
            RuntimeCall::ShengBankStakeInterest(_) => {
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
            RuntimeCall::SfidCodeAuth(_) => onchain_transaction_fee::AmountExtractResult::NoAmount,
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
            RuntimeCall::DuoqianTransactionPow(_) => {
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
                    institution, ..
                },
            ) => {
                // 中文注释：链下批次上链手续费永远由本机构 fee_pallet_address 承担。
                // 提交账户只负责提交，不承担手续费。
                OffchainTransactionFee::fee_account_of(*institution).ok()
            }
            RuntimeCall::OffchainTransactionFee(
                offchain_transaction_fee::pallet::Call::process_queued_batch { queue_id },
            ) => OffchainTransactionFee::fee_payer_for_queued_batch(*queue_id).ok(),
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
    type MaxBatchSize = ConstU32<1000>;
    type MaxBatchSignatureLength = ConstU32<64>;
    type MaxRelaySubmitters = ConstU32<8>;
    type InternalVoteEngine = VotingEngineSystem;
    type OffchainBatchVerifier = RuntimeOffchainBatchVerifier;
    type ProtectedSourceChecker = RuntimeProtectedSourceChecker;
    type WeightInfo = ();
}

pub struct RuntimeDuoqianAdminAuth;

impl duoqian_transaction_pow::DuoqianAdminAuth<AccountId> for RuntimeDuoqianAdminAuth {
    type PublicKey = [u8; 32];
    type Signature = [u8; 64];

    fn is_valid_public_key(public_key: &Self::PublicKey) -> bool {
        public_key.iter().any(|b| *b != 0)
    }

    fn public_key_to_account(public_key: &Self::PublicKey) -> Option<AccountId> {
        let signer = MultiSigner::from(sr25519::Public::from_raw(*public_key));
        Some(<MultiSigner as IdentifyAccount>::into_account(signer))
    }

    fn verify_signature(
        public_key: &Self::PublicKey,
        payload: &[u8],
        signature: &Self::Signature,
    ) -> bool {
        let public = sr25519::Public::from_raw(*public_key);
        let sig = sr25519::Signature::from_raw(*signature);
        sr25519_verify(&sig, payload, &public)
    }
}

pub struct RuntimeDuoqianAddressValidator;

impl duoqian_transaction_pow::DuoqianAddressValidator<AccountId>
    for RuntimeDuoqianAddressValidator
{
    fn is_valid(address: &AccountId) -> bool {
        // 中文注释：禁止黑洞地址。
        if address == &AccountId::new([0u8; 32]) {
            return false;
        }

        // 中文注释：禁止占用“国储会/省储会”的制度保留交易地址。
        if primitives::china::china_cb::CHINA_CB
            .iter()
            .any(|n| address == &AccountId::new(n.duoqian_address))
        {
            return false;
        }

        // 中文注释：禁止占用“省储行”的制度保留交易地址。
        if primitives::china::china_ch::CHINA_CH
            .iter()
            .any(|n| address == &AccountId::new(n.duoqian_address))
        {
            return false;
        }

        // 中文注释：禁止占用“省储行手续费账户”地址（由 shenfen_fee_id 派生）。
        if primitives::china::china_ch::CHINA_CH.iter().any(|n| {
            primitives::china::china_ch::shenfen_fee_id_to_bytes(n.shenfen_fee_id)
                .map(|pid| {
                    let fee_account: AccountId = PalletId(pid).into_account_truncating();
                    address == &fee_account
                })
                .unwrap_or(false)
        }) {
            return false;
        }

        true
    }
}

pub struct RuntimeDuoqianReservedAddressChecker;
pub struct RuntimeSfidRegistryOperator;

pub struct RuntimeProtectedSourceChecker;

impl offchain_transaction_fee::ProtectedSourceChecker<AccountId> for RuntimeProtectedSourceChecker {
    fn is_protected(address: &AccountId) -> bool {
        is_keyless_account(address)
    }
}

impl duoqian_transaction_pow::ProtectedSourceChecker<AccountId> for RuntimeProtectedSourceChecker {
    fn is_protected(address: &AccountId) -> bool {
        is_keyless_account(address)
    }
}

impl duoqian_transaction_pow::DuoqianReservedAddressChecker<AccountId>
    for RuntimeDuoqianReservedAddressChecker
{
    fn is_reserved(address: &AccountId) -> bool {
        // 中文注释：禁止占用省储行 keyless_address（制度保留地址）。
        if primitives::china::china_ch::CHINA_CH
            .iter()
            .any(|n| address == &AccountId::new(n.keyless_address))
        {
            return true;
        }

        // 中文注释：禁止占用省储行手续费地址（由 shenfen_fee_id 派生）。
        if primitives::china::china_ch::CHINA_CH.iter().any(|n| {
            primitives::china::china_ch::shenfen_fee_id_to_bytes(n.shenfen_fee_id)
                .map(|pid| {
                    let fee_account: AccountId = PalletId(pid).into_account_truncating();
                    address == &fee_account
                })
                .unwrap_or(false)
        }) {
            return true;
        }

        let raw: &[u8] = address.as_ref();
        if raw.len() != 32 {
            return false;
        }
        let mut addr = [0u8; 32];
        addr.copy_from_slice(raw);
        primitives::china::china_zb::is_reserved_duoqian_address(&addr)
    }
}

impl duoqian_transaction_pow::SfidRegistryOperator<AccountId> for RuntimeSfidRegistryOperator {
    fn can_register(operator: &AccountId) -> bool {
        if sfid_code_auth::Pallet::<Runtime>::sfid_main_account()
            .as_ref()
            .map(|acc| acc == operator)
            .unwrap_or(false)
        {
            return true;
        }
        if sfid_code_auth::Pallet::<Runtime>::sfid_backup_account_1()
            .as_ref()
            .map(|acc| acc == operator)
            .unwrap_or(false)
        {
            return true;
        }
        sfid_code_auth::Pallet::<Runtime>::sfid_backup_account_2()
            .as_ref()
            .map(|acc| acc == operator)
            .unwrap_or(false)
    }
}

impl duoqian_transaction_pow::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type AdminAuth = RuntimeDuoqianAdminAuth;
    type AddressValidator = RuntimeDuoqianAddressValidator;
    type ReservedAddressChecker = RuntimeDuoqianReservedAddressChecker;
    type ProtectedSourceChecker = RuntimeProtectedSourceChecker;
    type SfidRegistryOperator = RuntimeSfidRegistryOperator;
    type MaxAdmins = ConstU32<64>;
    type MaxSfidIdLength = ConstU32<96>;
    type MinCreateAmount = ConstU128<111>;
    type MinCloseBalance = ConstU128<111>;
    type WeightInfo = ();
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

fn current_sfid_verify_public() -> Option<sr25519::Public> {
    let key = sfid_code_auth::Pallet::<Runtime>::current_sfid_verify_pubkey()?;
    Some(sr25519::Public::from_raw(key))
}

pub struct RuntimeSfidVerifier;

impl
    sfid_code_auth::SfidVerifier<
        AccountId,
        Hash,
        sfid_code_auth::pallet::NonceOf<Runtime>,
        sfid_code_auth::pallet::SignatureOf<Runtime>,
    > for RuntimeSfidVerifier
{
    fn verify(
        account: &AccountId,
        credential: &sfid_code_auth::pallet::CredentialOf<Runtime>,
    ) -> bool {
        let public = match current_sfid_verify_public() {
            Some(v) => v,
            None => return false,
        };
        let sig_bytes = credential.signature.as_slice();
        if sig_bytes.len() != 64 {
            return false;
        }

        let mut sig_raw = [0u8; 64];
        sig_raw.copy_from_slice(sig_bytes);
        let signature = sr25519::Signature::from_raw(sig_raw);

        let payload = (
            b"GMB_SFID_BIND_V1",
            frame_system::Pallet::<Runtime>::block_hash(0),
            account,
            credential.sfid_code_hash,
            credential.nonce.as_slice(),
        );
        let msg = blake2_256(&payload.encode());

        sr25519_verify(&signature, &msg, &public)
    }
}

pub struct RuntimeSfidVoteVerifier;

impl
    sfid_code_auth::SfidVoteVerifier<
        AccountId,
        Hash,
        sfid_code_auth::pallet::NonceOf<Runtime>,
        sfid_code_auth::pallet::SignatureOf<Runtime>,
    > for RuntimeSfidVoteVerifier
{
    fn verify_vote(
        account: &AccountId,
        sfid_hash: Hash,
        proposal_id: u64,
        nonce: &sfid_code_auth::pallet::NonceOf<Runtime>,
        signature: &sfid_code_auth::pallet::SignatureOf<Runtime>,
    ) -> bool {
        let public = match current_sfid_verify_public() {
            Some(v) => v,
            None => return false,
        };
        let sig_bytes = signature.as_slice();
        if sig_bytes.len() != 64 {
            return false;
        }

        let mut sig_raw = [0u8; 64];
        sig_raw.copy_from_slice(sig_bytes);
        let signature = sr25519::Signature::from_raw(sig_raw);

        let payload = (
            b"GMB_SFID_VOTE_V1",
            frame_system::Pallet::<Runtime>::block_hash(0),
            account,
            sfid_hash,
            proposal_id,
            nonce.as_slice(),
        );
        let msg = blake2_256(&payload.encode());

        sr25519_verify(&signature, &msg, &public)
    }
}

impl sfid_code_auth::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxSfidLength = ConstU32<64>;
    type MaxCredentialNonceLength = ConstU32<64>;
    // 中文注释：SFID 绑定与投票验签统一使用 64 字节原始 sr25519 签名。
    type MaxCredentialSignatureLength = ConstU32<64>;
    type SfidVerifier = RuntimeSfidVerifier;
    type SfidVoteVerifier = RuntimeSfidVoteVerifier;
    type OnSfidBound = CitizenLightnodeIssuance;
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
        #[cfg(feature = "runtime-benchmarks")]
        {
            let _ = who;
            eligible_total > 0 && !nonce.is_empty() && !signature.is_empty()
        }

        #[cfg(not(feature = "runtime-benchmarks"))]
        {
            let public = match current_sfid_verify_public() {
                Some(v) => v,
                None => return false,
            };
            let sig_bytes = signature.as_slice();
            if sig_bytes.len() != 64 {
                return false;
            }

            let mut sig_raw = [0u8; 64];
            sig_raw.copy_from_slice(sig_bytes);
            let signature = sr25519::Signature::from_raw(sig_raw);

            let payload = (
                b"GMB_SFID_POPULATION_V1",
                frame_system::Pallet::<Runtime>::block_hash(0),
                who,
                eligible_total,
                nonce.as_slice(),
            );
            let msg = blake2_256(&payload.encode());

            sr25519_verify(&signature, &msg, &public)
        }
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
    pub const ResolutionIssuanceMaxTotalIssuance: u128 = u128::MAX;
    /// Runtime 升级治理提案备注最大长度。
    pub const RuntimeUpgradeMaxReasonLen: u32 = 1024;
    /// Runtime wasm 最大长度（字节）。
    pub const RuntimeUpgradeMaxCodeSize: u32 = 5 * 1024 * 1024;
    /// 管理员治理：单机构管理员列表上限（覆盖国储会 19 人规模）。
    pub const MaxAdminsPerInstitution: u32 = 32;
    /// 管理员替换提案过期清理窗口（区块数）。
    pub const AdminReplacementStaleProposalLifetime: u32 =
        primitives::count_const::VOTING_DURATION_BLOCKS * 2;
}

impl admins_origin_gov::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxAdminsPerInstitution = MaxAdminsPerInstitution;
    type StaleProposalLifetime = AdminReplacementStaleProposalLifetime;
    type InternalVoteEngine = VotingEngineSystem;
}

impl resolution_destro_gov::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type StaleProposalLifetime = AdminReplacementStaleProposalLifetime;
    type InternalVoteEngine = VotingEngineSystem;
    type WeightInfo = resolution_destro_gov::SubstrateWeight<Runtime>;
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

pub struct EnsureJointVoteFinalizeOrigin;

impl EnsureOrigin<RuntimeOrigin> for EnsureJointVoteFinalizeOrigin {
    type Success = ();

    fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
        #[cfg(feature = "runtime-benchmarks")]
        {
            return frame_system::EnsureRoot::<AccountId>::try_origin(o).map(|_| ());
        }
        #[cfg(not(feature = "runtime-benchmarks"))]
        {
            Err(o)
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn try_successful_origin() -> Result<RuntimeOrigin, ()> {
        Ok(RuntimeOrigin::from(frame_system::RawOrigin::Root))
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
        let admin = AccountId::new(primitives::china::china_cb::CHINA_CB[0].admins[0]);
        Ok(RuntimeOrigin::from(frame_system::RawOrigin::Signed(admin)))
    }
}

fn is_nrc_admin(who: &AccountId) -> bool {
    let nrc_institution = primitives::china::china_cb::CHINA_CB
        .first()
        .and_then(|n| primitives::china::china_cb::shenfen_id_to_fixed48(n.shenfen_id))
        .expect("NRC shenfen_id must be valid");

    // 中文注释：创世后只信任链上管理员治理模块中的当前管理员名单。
    if let Some(admins) = admins_origin_gov::CurrentAdmins::<Runtime>::get(nrc_institution) {
        admins.into_inner().iter().any(|admin| admin == who)
    } else {
        false
    }
}

impl resolution_issuance_iss::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    // 中文注释：协议层封死特权入口，执行发行不接受任何外部特权调用。
    type ExecuteOrigin = EnsureNoPrivilegeOrigin;
    // 中文注释：仅保留清理类维护入口，避免执行入口暴露。
    type MaintenanceOrigin = frame_system::EnsureRoot<AccountId>;
    type MaxReasonLen = ResolutionIssuanceMaxReasonLen;
    type MaxAllocations = ResolutionIssuanceMaxAllocations;
    type MaxTotalIssuance = ResolutionIssuanceMaxTotalIssuance;
    type WeightInfo = ();
}

impl resolution_issuance_gov::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type NrcProposeOrigin = EnsureNrcAdmin;
    type RecipientSetOrigin = frame_system::EnsureRoot<AccountId>;
    // 中文注释：禁用外部 finalize 入口，只允许投票引擎回调路径落地结果。
    type JointVoteFinalizeOrigin = EnsureJointVoteFinalizeOrigin;
    type IssuanceExecutor = ResolutionIssuanceIss;
    type IssuanceWeightInfo = ();
    type WeightInfo = resolution_issuance_gov::SubstrateWeight<Runtime>;
    type JointVoteEngine = VotingEngineSystem;
    type MaxReasonLen = ResolutionIssuanceMaxReasonLen;
    type MaxAllocations = ResolutionIssuanceMaxAllocations;
    type MaxSnapshotNonceLength = ConstU32<64>;
    type MaxSnapshotSignatureLength = ConstU32<64>;
    type MaxExecutionRetries = ConstU32<5>;
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
    type MaxSfidLength = ConstU32<64>;
    type MaxVoteNonceLength = ConstU32<64>;
    type MaxVoteSignatureLength = ConstU32<64>;
    type SfidEligibility = RuntimeSfidEligibility;
    type PopulationSnapshotVerifier = RuntimePopulationSnapshotVerifier;
    type JointVoteResultCallback = RuntimeJointVoteResultCallback;
    type InternalAdminProvider = RuntimeInternalAdminProvider;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OffchainTransactionFee;
    use crate::ResolutionDestroGov;
    use duoqian_transaction_pow::DuoqianReservedAddressChecker;
    use frame_support::assert_ok;
    use frame_support::traits::Currency;
    use offchain_transaction_fee::OffchainBatchVerifier;
    use primitives::china::china_cb::{
        shenfen_id_to_fixed48 as reserve_pallet_id_to_bytes, CHINA_CB,
    };
    use sfid_code_auth::{SfidVerifier, SfidVoteVerifier};
    use sp_core::Pair;
    use sp_runtime::{traits::Hash as HashT, traits::IdentifyAccount, BuildStorage, MultiSigner};
    use voting_engine_system::{
        InternalAdminProvider, JointVoteResultCallback, PopulationSnapshotVerifier, SfidEligibility,
    };

    fn new_test_ext() -> sp_io::TestExternalities {
        let storage = crate::RuntimeGenesisConfig::default()
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
                AccountId::new(primitives::china::china_cb::CHINA_CB[1].duoqian_address);
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

            assert!(
                resolution_issuance_iss::pallet::Executed::<Runtime>::get(proposal_id).is_some()
            );
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
            let nrc_institution = reserve_pallet_id_to_bytes(CHINA_CB[0].shenfen_id)
                .expect("nrc institution id must be valid");
            let nrc_account = AccountId::new(CHINA_CB[0].duoqian_address);
            let initial_balance: Balance = 1_000;
            let destroy_amount: Balance = 100;

            let _ = Balances::deposit_creating(&nrc_account, initial_balance);
            let issuance_before = Balances::total_issuance();

            assert_ok!(ResolutionDestroGov::propose_destroy(
                RuntimeOrigin::signed(AccountId::new(CHINA_CB[0].admins[0])),
                voting_engine_system::internal_vote::ORG_NRC,
                nrc_institution,
                destroy_amount,
            ));

            for i in 0..13 {
                assert_ok!(ResolutionDestroGov::vote_destroy(
                    RuntimeOrigin::signed(AccountId::new(CHINA_CB[0].admins[i])),
                    0,
                    true,
                ));
            }

            assert!(resolution_destro_gov::Pallet::<Runtime>::proposal_action(0).is_none());

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
            let institution = primitives::china::china_ch::shenfen_id_to_fixed48(
                primitives::china::china_ch::CHINA_CH[0].shenfen_id,
            )
            .expect("institution id should be valid");
            let institution_account =
                AccountId::new(primitives::china::china_ch::CHINA_CH[0].duoqian_address);
            let payer = AccountId::new([7u8; 32]);
            let recipient = AccountId::new([8u8; 32]);

            let _ = Balances::deposit_creating(&institution_account, 1_000);
            let _ = Balances::deposit_creating(&payer, 10_000);
            let _ = Balances::deposit_creating(&recipient, 1);
            assert_ok!(OffchainTransactionFee::bind_clearing_institution(
                RuntimeOrigin::signed(recipient.clone()),
                institution,
            ));

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
            let verify_key: offchain_transaction_fee::pallet::VerifyKeyOf<Runtime> = pair
                .public()
                .0
                .to_vec()
                .try_into()
                .expect("verify key should fit");
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
                .sign(&blake2_256(
                    &(b"GMB_OFFCHAIN_BATCH_V1", institution, 1u64, &batch).encode(),
                ))
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

    #[test]
    fn pow_tx_amount_extractor_covers_noamount_amount_and_unknown_paths() {
        new_test_ext().execute_with(|| {
            let who = AccountId::new([1u8; 32]);
            let recipient = AccountId::new([2u8; 32]);

            let system_call = RuntimeCall::System(frame_system::Call::remark {
                remark: b"x".to_vec(),
            });
            let no_amount = <PowTxAmountExtractor as onchain_transaction_fee::CallAmount<
                AccountId,
                RuntimeCall,
                Balance,
            >>::amount(&who, &system_call);
            assert!(matches!(
                no_amount,
                onchain_transaction_fee::AmountExtractResult::NoAmount
            ));

            let transfer_call =
                RuntimeCall::Balances(pallet_balances::Call::transfer_allow_death {
                    dest: sp_runtime::MultiAddress::Id(recipient),
                    value: 123,
                });
            let amount = <PowTxAmountExtractor as onchain_transaction_fee::CallAmount<
                AccountId,
                RuntimeCall,
                Balance,
            >>::amount(&who, &transfer_call);
            match amount {
                onchain_transaction_fee::AmountExtractResult::Amount(v) => assert_eq!(v, 123),
                _ => panic!("expected amount path"),
            }

            let item =
                offchain_transaction_fee::pallet::OffchainBatchItem::<AccountId, Balance, Hash> {
                    tx_id: <Runtime as frame_system::Config>::Hashing::hash(b"rt-unknown-1"),
                    payer: who.clone(),
                    recipient: AccountId::new([3u8; 32]),
                    transfer_amount: 10,
                    offchain_fee_amount: 1,
                };
            let batch: offchain_transaction_fee::pallet::BatchOf<Runtime> =
                vec![item].try_into().expect("batch should fit");
            let bad_sig: offchain_transaction_fee::pallet::BatchSignatureOf<Runtime> =
                vec![1u8; 64].try_into().expect("signature should fit");
            let offchain_call = RuntimeCall::OffchainTransactionFee(
                offchain_transaction_fee::pallet::Call::submit_offchain_batch {
                    institution: primitives::china::china_ch::shenfen_id_to_fixed48("badid000")
                        .expect("test shenfen_id should map"),
                    batch_seq: 1,
                    batch,
                    batch_signature: bad_sig,
                },
            );
            let unknown = <PowTxAmountExtractor as onchain_transaction_fee::CallAmount<
                AccountId,
                RuntimeCall,
                Balance,
            >>::amount(&who, &offchain_call);
            assert!(matches!(
                unknown,
                onchain_transaction_fee::AmountExtractResult::Unknown
            ));
        });
    }

    #[test]
    fn pow_tx_amount_extractor_covers_duoqian_create_and_close() {
        new_test_ext().execute_with(|| {
            let (p1, _) = sr25519::Pair::generate();
            let (p2, _) = sr25519::Pair::generate();
            let admin1 = p1.public().0;
            let admin2 = p2.public().0;
            let signer1 = MultiSigner::from(p1.public());
            let who: AccountId = signer1.into_account();

            let duoqian_address = AccountId::new([77u8; 32]);
            let beneficiary = AccountId::new([78u8; 32]);
            let sfid_id: duoqian_transaction_pow::pallet::SfidIdOf<Runtime> =
                b"GFR-LN001-CB0C-runtime-20260222"
                    .to_vec()
                    .try_into()
                    .expect("sfid id should fit");
            let admins: duoqian_transaction_pow::pallet::DuoqianAdminsOf<Runtime> =
                vec![admin1, admin2].try_into().expect("admins should fit");

            let create_payload = (
                b"DUOQIAN_CREATE_V1".to_vec(),
                &duoqian_address,
                2u32,
                &admins,
                1u32,
                1_000u128,
            )
                .encode();
            let create_sig = p1.sign(&create_payload).0;
            let approvals: duoqian_transaction_pow::pallet::AdminApprovalsOf<Runtime> =
                vec![duoqian_transaction_pow::AdminApproval {
                    public_key: admin1,
                    signature: create_sig,
                }]
                .try_into()
                .expect("approvals should fit");

            let create_call = RuntimeCall::DuoqianTransactionPow(
                duoqian_transaction_pow::pallet::Call::create_duoqian {
                    sfid_id,
                    admin_count: 2,
                    duoqian_admins: admins.clone(),
                    threshold: 1,
                    amount: 1_000,
                    expires_at: 9_999,
                    approvals,
                },
            );
            let create_amount = <PowTxAmountExtractor as onchain_transaction_fee::CallAmount<
                AccountId,
                RuntimeCall,
                Balance,
            >>::amount(&who, &create_call);
            match create_amount {
                onchain_transaction_fee::AmountExtractResult::Amount(v) => assert_eq!(v, 1_000),
                _ => panic!("expected create amount"),
            }

            let _ = Balances::deposit_creating(&duoqian_address, 777);
            let close_payload = (
                b"DUOQIAN_CLOSE_V1".to_vec(),
                &duoqian_address,
                &beneficiary,
                2u32,
                1u32,
                777u128,
            )
                .encode();
            let close_sig = p1.sign(&close_payload).0;
            let close_approvals: duoqian_transaction_pow::pallet::AdminApprovalsOf<Runtime> =
                vec![duoqian_transaction_pow::AdminApproval {
                    public_key: admin1,
                    signature: close_sig,
                }]
                .try_into()
                .expect("approvals should fit");
            let close_call = RuntimeCall::DuoqianTransactionPow(
                duoqian_transaction_pow::pallet::Call::close_duoqian {
                    duoqian_address,
                    beneficiary,
                    min_balance: 1,
                    expires_at: 9_999,
                    approvals: close_approvals,
                },
            );
            let close_amount = <PowTxAmountExtractor as onchain_transaction_fee::CallAmount<
                AccountId,
                RuntimeCall,
                Balance,
            >>::amount(&who, &close_call);
            match close_amount {
                onchain_transaction_fee::AmountExtractResult::Amount(v) => assert_eq!(v, 777),
                _ => panic!("expected close amount"),
            }
        });
    }

    #[test]
    fn runtime_fee_payer_extractor_routes_offchain_fee_to_fee_account() {
        new_test_ext().execute_with(|| {
            let who = AccountId::new([9u8; 32]);
            let institution = primitives::china::china_ch::shenfen_id_to_fixed48(
                primitives::china::china_ch::CHINA_CH[0].shenfen_id,
            )
            .expect("institution id should be valid");
            let fee_account =
                OffchainTransactionFee::fee_account_of(institution).expect("fee account");

            let dummy_item =
                offchain_transaction_fee::pallet::OffchainBatchItem::<AccountId, Balance, Hash> {
                    tx_id: <Runtime as frame_system::Config>::Hashing::hash(b"rt-fee-payer-1"),
                    payer: who.clone(),
                    recipient: AccountId::new([10u8; 32]),
                    transfer_amount: 1,
                    offchain_fee_amount: 1,
                };
            let batch: offchain_transaction_fee::pallet::BatchOf<Runtime> =
                vec![dummy_item].try_into().expect("batch should fit");
            let sig: offchain_transaction_fee::pallet::BatchSignatureOf<Runtime> =
                vec![0u8; 64].try_into().expect("sig should fit");
            let offchain_call = RuntimeCall::OffchainTransactionFee(
                offchain_transaction_fee::pallet::Call::submit_offchain_batch {
                    institution,
                    batch_seq: 1,
                    batch,
                    batch_signature: sig,
                },
            );
            assert_eq!(
                <RuntimeFeePayerExtractor as onchain_transaction_fee::CallFeePayer<
                    AccountId,
                    RuntimeCall,
                >>::fee_payer(&who, &offchain_call),
                Some(fee_account)
            );

            let system_call = RuntimeCall::System(frame_system::Call::remark { remark: vec![] });
            assert_eq!(
                <RuntimeFeePayerExtractor as onchain_transaction_fee::CallFeePayer<
                    AccountId,
                    RuntimeCall,
                >>::fee_payer(&who, &system_call),
                None
            );
        });
    }

    #[test]
    fn duoqian_reserved_checker_rejects_keyless_and_shenfen_fee_addresses() {
        let keyless = AccountId::new(primitives::china::china_ch::CHINA_CH[0].keyless_address);
        assert!(RuntimeDuoqianReservedAddressChecker::is_reserved(&keyless));

        let pid = primitives::china::china_ch::shenfen_fee_id_to_bytes(
            primitives::china::china_ch::CHINA_CH[0].shenfen_fee_id,
        )
        .expect("shenfen_fee_id must be 8 bytes");
        let fee_account: AccountId = PalletId(pid).into_account_truncating();
        assert!(RuntimeDuoqianReservedAddressChecker::is_reserved(
            &fee_account
        ));
    }

    #[test]
    fn runtime_call_filter_blocks_force_transfer_from_keyless() {
        let keyless = AccountId::new(primitives::china::china_ch::CHINA_CH[0].keyless_address);
        let dst = AccountId::new([9u8; 32]);

        let blocked_by_id = RuntimeCall::Balances(pallet_balances::Call::force_transfer {
            source: sp_runtime::MultiAddress::Id(keyless),
            dest: sp_runtime::MultiAddress::Id(dst.clone()),
            value: 1,
        });
        assert!(!RuntimeCallFilter::contains(&blocked_by_id));

        let keyless_raw = primitives::china::china_ch::CHINA_CH[0].keyless_address;
        let blocked_by_32 = RuntimeCall::Balances(pallet_balances::Call::force_transfer {
            source: sp_runtime::MultiAddress::Address32(keyless_raw),
            dest: sp_runtime::MultiAddress::Id(dst.clone()),
            value: 1,
        });
        assert!(!RuntimeCallFilter::contains(&blocked_by_32));

        let blocked_by_raw = RuntimeCall::Balances(pallet_balances::Call::force_transfer {
            source: sp_runtime::MultiAddress::Raw(keyless_raw.to_vec()),
            dest: sp_runtime::MultiAddress::Id(dst.clone()),
            value: 1,
        });
        assert!(!RuntimeCallFilter::contains(&blocked_by_raw));

        let allowed = RuntimeCall::Balances(pallet_balances::Call::force_transfer {
            source: sp_runtime::MultiAddress::Id(AccountId::new([8u8; 32])),
            dest: sp_runtime::MultiAddress::Id(dst),
            value: 1,
        });
        assert!(RuntimeCallFilter::contains(&allowed));
    }

    #[test]
    fn pow_digest_author_and_offchain_batch_verifier_work() {
        let author = AccountId::new([21u8; 32]);
        let encoded = author.encode();
        let digests: Vec<(sp_runtime::ConsensusEngineId, &[u8])> = vec![
            (*b"TEST", b"ignored".as_ref()),
            (sp_consensus_pow::POW_ENGINE_ID, encoded.as_slice()),
        ];
        let found = PowDigestAuthor::find_author(digests);
        assert_eq!(found, Some(author));

        let (pair, _) = sr25519::Pair::generate();
        let msg = b"batch-message";
        let sig = pair.sign(msg);
        assert!(RuntimeOffchainBatchVerifier::verify(
            &pair.public().0,
            msg,
            &sig.0
        ));
        assert!(!RuntimeOffchainBatchVerifier::verify(
            &pair.public().0[..31],
            msg,
            &sig.0
        ));
        assert!(!RuntimeOffchainBatchVerifier::verify(
            &pair.public().0,
            msg,
            &sig.0[..63]
        ));
    }

    #[test]
    fn joint_vote_callback_missing_mapping_and_runtime_upgrade_route() {
        new_test_ext().execute_with(|| {
            assert!(
                RuntimeJointVoteResultCallback::on_joint_vote_finalized(999_999, true).is_err()
            );

            let proposal_id = 7u64;
            let joint_vote_id = 70u64;
            let proposer = AccountId::new(CHINA_CB[0].admins[0]);
            let reason: runtime_root_upgrade::pallet::ReasonOf<Runtime> =
                b"upgrade".to_vec().try_into().expect("reason");
            let code: runtime_root_upgrade::pallet::CodeOf<Runtime> =
                vec![1u8, 2, 3].try_into().expect("code");
            let code_hash = <Runtime as frame_system::Config>::Hashing::hash(code.as_slice());

            runtime_root_upgrade::pallet::Proposals::<Runtime>::insert(
                proposal_id,
                runtime_root_upgrade::pallet::Proposal::<Runtime> {
                    proposer,
                    reason,
                    code_hash,
                    code,
                    status: runtime_root_upgrade::pallet::ProposalStatus::Voting,
                },
            );
            runtime_root_upgrade::pallet::GovToJointVote::<Runtime>::insert(
                proposal_id,
                joint_vote_id,
            );
            runtime_root_upgrade::pallet::JointVoteToGov::<Runtime>::insert(
                joint_vote_id,
                proposal_id,
            );

            assert_ok!(RuntimeJointVoteResultCallback::on_joint_vote_finalized(
                joint_vote_id,
                false
            ));
            let updated = runtime_root_upgrade::pallet::Proposals::<Runtime>::get(proposal_id)
                .expect("proposal should exist");
            assert!(matches!(
                updated.status,
                runtime_root_upgrade::pallet::ProposalStatus::Rejected
            ));
            assert!(
                runtime_root_upgrade::pallet::GovToJointVote::<Runtime>::get(proposal_id).is_none()
            );
            assert!(
                runtime_root_upgrade::pallet::JointVoteToGov::<Runtime>::get(joint_vote_id)
                    .is_none()
            );
        });
    }

    #[test]
    fn runtime_sfid_verifiers_and_population_snapshot_verify_with_runtime_main_key() {
        new_test_ext().execute_with(|| {
            let (pair, _) = sr25519::Pair::generate();
            let sfid_main: AccountId = MultiSigner::from(pair.public()).into_account();
            sfid_code_auth::pallet::SfidMainAccount::<Runtime>::put(sfid_main);
            assert_eq!(
                sfid_code_auth::Pallet::<Runtime>::current_sfid_verify_pubkey(),
                Some(pair.public().0)
            );
            assert_eq!(
                sfid_code_auth::Pallet::<Runtime>::current_sfid_verify_pubkey(),
                Some(pair.public().0)
            );

            let account = AccountId::new([31u8; 32]);
            let sfid_hash = <Runtime as frame_system::Config>::Hashing::hash(b"sfid-verify");
            let bind_nonce: sfid_code_auth::pallet::NonceOf<Runtime> =
                b"bind-nonce".to_vec().try_into().expect("nonce should fit");
            let bind_payload = (
                b"GMB_SFID_BIND_V1",
                frame_system::Pallet::<Runtime>::block_hash(0),
                &account,
                sfid_hash,
                bind_nonce.as_slice(),
            );
            let bind_msg = blake2_256(&bind_payload.encode());
            let bind_sig = pair.sign(&bind_msg);
            let bind_signature: sfid_code_auth::pallet::SignatureOf<Runtime> = bind_sig
                .0
                .to_vec()
                .try_into()
                .expect("signature should fit");
            let bind_credential = sfid_code_auth::BindCredential {
                sfid_code_hash: sfid_hash,
                nonce: bind_nonce.clone(),
                signature: bind_signature,
            };
            assert!(RuntimeSfidVerifier::verify(&account, &bind_credential));

            let bad_bind_signature: sfid_code_auth::pallet::SignatureOf<Runtime> =
                vec![7u8; 63].try_into().expect("signature should fit");
            let bad_bind_credential = sfid_code_auth::BindCredential {
                sfid_code_hash: sfid_hash,
                nonce: bind_nonce,
                signature: bad_bind_signature,
            };
            assert!(!RuntimeSfidVerifier::verify(&account, &bad_bind_credential));

            let vote_nonce: sfid_code_auth::pallet::NonceOf<Runtime> =
                b"vote-nonce".to_vec().try_into().expect("nonce should fit");
            let vote_signature: sfid_code_auth::pallet::SignatureOf<Runtime> = pair
                .sign(&blake2_256(
                    &(
                        b"GMB_SFID_VOTE_V1",
                        frame_system::Pallet::<Runtime>::block_hash(0),
                        &account,
                        sfid_hash,
                        9u64,
                        vote_nonce.as_slice(),
                    )
                        .encode(),
                ))
                .0
                .to_vec()
                .try_into()
                .expect("signature should fit");
            assert!(RuntimeSfidVoteVerifier::verify_vote(
                &account,
                sfid_hash,
                9,
                &vote_nonce,
                &vote_signature
            ));

            let pop_nonce: voting_engine_system::pallet::VoteNonceOf<Runtime> =
                b"pop-nonce".to_vec().try_into().expect("nonce should fit");
            let pop_signature: voting_engine_system::pallet::VoteSignatureOf<Runtime> = pair
                .sign(&blake2_256(
                    &(
                        b"GMB_SFID_POPULATION_V1",
                        frame_system::Pallet::<Runtime>::block_hash(0),
                        &account,
                        123u64,
                        pop_nonce.as_slice(),
                    )
                        .encode(),
                ))
                .0
                .to_vec()
                .try_into()
                .expect("signature should fit");
            assert!(
                RuntimePopulationSnapshotVerifier::verify_population_snapshot(
                    &account,
                    123,
                    &pop_nonce,
                    &pop_signature
                )
            );
        });
    }

    #[test]
    fn runtime_sfid_eligibility_wrapper_works_with_nonce_consumption() {
        new_test_ext().execute_with(|| {
            let (pair, _) = sr25519::Pair::generate();
            let sfid_main: AccountId = MultiSigner::from(pair.public()).into_account();
            sfid_code_auth::pallet::SfidMainAccount::<Runtime>::put(sfid_main);

            let who = AccountId::new([41u8; 32]);
            let sfid = b"sfid-wrap";
            let sfid_hash = <Runtime as frame_system::Config>::Hashing::hash(sfid);
            sfid_code_auth::pallet::SfidToAccount::<Runtime>::insert(sfid_hash, who.clone());
            sfid_code_auth::pallet::AccountToSfid::<Runtime>::insert(who.clone(), sfid_hash);

            assert!(RuntimeSfidEligibility::is_eligible(sfid, &who));
            assert!(!RuntimeSfidEligibility::is_eligible(
                sfid,
                &AccountId::new([42u8; 32])
            ));

            let nonce = b"wrap-nonce";
            let vote_msg = blake2_256(
                &(
                    b"GMB_SFID_VOTE_V1",
                    frame_system::Pallet::<Runtime>::block_hash(0),
                    &who,
                    sfid_hash,
                    88u64,
                    nonce.as_slice(),
                )
                    .encode(),
            );
            let signature = pair.sign(&vote_msg).0.to_vec();
            let nonce_bounded: sfid_code_auth::pallet::NonceOf<Runtime> =
                nonce.to_vec().try_into().expect("nonce should fit");
            let signature_bounded: sfid_code_auth::pallet::SignatureOf<Runtime> =
                signature.clone().try_into().expect("signature should fit");
            assert!(RuntimeSfidVoteVerifier::verify_vote(
                &who,
                sfid_hash,
                88,
                &nonce_bounded,
                &signature_bounded
            ));
            assert!(RuntimeSfidEligibility::verify_and_consume_vote_credential(
                sfid, &who, 88, nonce, &signature
            ));
            assert!(!RuntimeSfidEligibility::verify_and_consume_vote_credential(
                sfid, &who, 88, nonce, &signature
            ));
        });
    }

    #[test]
    fn ensure_nrc_admin_and_runtime_internal_admin_provider_paths() {
        new_test_ext().execute_with(|| {
            let nrc_id = reserve_pallet_id_to_bytes(CHINA_CB[0].shenfen_id).expect("nrc id");
            let nrc_admin = AccountId::new(CHINA_CB[0].admins[0]);
            let outsider = AccountId::new([99u8; 32]);

            let ok_origin = RuntimeOrigin::signed(nrc_admin.clone());
            assert!(<EnsureNrcAdmin as EnsureOrigin<RuntimeOrigin>>::try_origin(ok_origin).is_ok());
            let bad_origin = RuntimeOrigin::signed(outsider.clone());
            assert!(
                <EnsureNrcAdmin as EnsureOrigin<RuntimeOrigin>>::try_origin(bad_origin).is_err()
            );

            admins_origin_gov::pallet::CurrentAdmins::<Runtime>::remove(nrc_id);
            assert!(!is_nrc_admin(&nrc_admin));
            assert!(!is_nrc_admin(&outsider));
            assert!(!RuntimeInternalAdminProvider::is_internal_admin(
                voting_engine_system::internal_vote::ORG_NRC,
                nrc_id,
                &nrc_admin
            ));
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
        // 中文注释：生产逻辑只信任链上当前管理员状态；无状态则拒绝（不再回退常量）。
        if let Some(admins) = admins_origin_gov::CurrentAdmins::<Runtime>::get(institution) {
            admins.into_inner().iter().any(|admin| admin == who)
        } else {
            false
        }
    }
}

pub struct RuntimeSfidEligibility;

impl voting_engine_system::SfidEligibility<AccountId> for RuntimeSfidEligibility {
    fn is_eligible(sfid: &[u8], who: &AccountId) -> bool {
        <sfid_code_auth::Pallet<Runtime> as sfid_code_auth::SfidEligibilityProvider<AccountId>>::is_eligible(sfid, who)
    }

    fn verify_and_consume_vote_credential(
        sfid: &[u8],
        who: &AccountId,
        proposal_id: u64,
        nonce: &[u8],
        signature: &[u8],
    ) -> bool {
        <sfid_code_auth::Pallet<Runtime> as sfid_code_auth::SfidEligibilityProvider<AccountId>>::verify_and_consume_vote_credential(
            sfid,
            who,
            proposal_id,
            nonce,
            signature,
        )
    }
}
