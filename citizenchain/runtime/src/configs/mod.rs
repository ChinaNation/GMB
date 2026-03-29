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
        fungible::{Balanced, Credit, Inspect},
        tokens::{Fortitude, Preservation},
        ConstU128, ConstU32, ConstU64, ConstU8, Contains, EnsureOrigin, FindAuthor, OnUnbalanced,
        UnfilteredDispatchable, VariantCountOf,
    },
    weights::{
        constants::{RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND},
        ConstantMultiplier, Weight,
    },
    PalletId,
};
use frame_system::limits::{BlockLength, BlockWeights};
use onchain_transaction_pow::NrcAccountProvider as _;
use pallet_transaction_payment::{ConstFeeMultiplier, Multiplier};
use sp_core::{sr25519, Void};
use sp_io::{crypto::sr25519_verify, hashing::blake2_256};
#[allow(unused_imports)]
use sp_runtime::traits::Hash as _;
use sp_runtime::{
    traits::{AccountIdConversion, One},
    Perbill,
};
use sp_version::RuntimeVersion;

// Local module imports
use super::{
    AccountId, Address, Balance, Balances, Block, BlockNumber, CitizenLightnodeIssuance,
    GenesisPallet, Hash, Nonce, PalletInfo, ResolutionIssuanceIss, Runtime, RuntimeCall,
    RuntimeEvent, RuntimeFreezeReason, RuntimeHoldReason, RuntimeOrigin, RuntimeTask, System,
    VotingEngineSystem, BLOCK_HASH_COUNT, EXISTENTIAL_DEPOSIT, SLOT_DURATION, VERSION,
};
#[cfg(not(feature = "runtime-benchmarks"))]
use super::{ResolutionIssuanceGov, RuntimeRootUpgrade};

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

fn is_reserved_fee_account(address: &AccountId) -> bool {
    primitives::china::china_ch::CHINA_CH.iter().any(|n| {
        primitives::china::china_ch::shenfen_fee_id_to_bytes(n.shenfen_fee_id)
            .map(|pid| {
                let fee_account: AccountId = PalletId(pid).into_account_truncating();
                address == &fee_account
            })
            .unwrap_or(false)
    })
}

fn is_reserved_duoqian_account(address: &AccountId) -> bool {
    let raw: &[u8] = address.as_ref();
    if raw.len() != 32 {
        return false;
    }
    let mut addr = [0u8; 32];
    addr.copy_from_slice(raw);
    primitives::china::china_zb::is_reserved_duoqian_address(&addr)
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
            RuntimeCall::Balances(pallet_balances::Call::force_unreserve { who, .. }) => {
                !is_keyless_multi_address(who)
            }
            RuntimeCall::Balances(pallet_balances::Call::force_set_balance { who, .. }) => {
                !is_keyless_multi_address(who)
            }
            // force_adjust_total_issuance 直接影响全局发行量；统一在 BaseCallFilter 禁用外部入口。
            RuntimeCall::Balances(pallet_balances::Call::force_adjust_total_issuance {
                ..
            }) => false,
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
    /// 中文注释：全局调用过滤器，禁止 keyless_address 参与 force_* 余额调用，并封禁强制总发行量调整入口。
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
    type WeightInfo = pallet_timestamp::weights::SubstrateWeight<Runtime>;
}

impl pallet_balances::Config for Runtime {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    /// The type for recording an account's balance.
    type Balance = Balance;
    /// The ubiquitous event type.
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = RuntimeDustHandler;
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
    pub const MaxGrandpaAuthorities: u32 = 64;
    pub const MaxGrandpaNominators: u32 = 0;
    // 中文注释：保留最近若干 set_id 与会话映射，便于后续接入等值投票追溯/举报能力。
    pub const MaxSetIdSessionEntries: u64 = 128;
}

impl pallet_grandpa::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type MaxAuthorities = MaxGrandpaAuthorities;
    type MaxNominators = MaxGrandpaNominators;
    type MaxSetIdSessionEntries = MaxSetIdSessionEntries;
    // 中文注释：当前版本不启用链上等值投票惩罚（无 session/historical 证明体系）。
    // 但保留 MaxSetIdSessionEntries 以便后续平滑接入。
    type KeyOwnerProof = Void;
    type EquivocationReportSystem = ();
}

parameter_types! {
    pub FeeMultiplier: Multiplier = Multiplier::one();
}

impl pallet_transaction_payment::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnChargeTransaction = onchain_transaction_pow::PowOnchainChargeAdapter<
        Balances,
        onchain_transaction_pow::PowOnchainFeeRouter<
            Runtime,
            Balances,
            PowDigestAuthor,
            RuntimeNrcAccountProvider,
        >,
        PowTxAmountExtractor,
        RuntimeFeePayerExtractor,
    >;
    type OperationalFeeMultiplier = ConstU8<{ primitives::core_const::OPERATIONAL_FEE_MULTIPLIER }>;
    type WeightToFee = ConstantMultiplier<Balance, ConstU128<0>>;
    type LengthToFee = ConstantMultiplier<Balance, ConstU128<0>>;
    type FeeMultiplierUpdate = ConstFeeMultiplier<FeeMultiplier>;
    type WeightInfo = pallet_transaction_payment::weights::SubstrateWeight<Runtime>;
}

impl onchain_transaction_pow::pallet::Config for Runtime {}

pub struct RuntimeNrcAccountProvider;

impl onchain_transaction_pow::NrcAccountProvider<AccountId> for RuntimeNrcAccountProvider {
    fn nrc_account() -> Option<AccountId> {
        Some(AccountId::new(
            primitives::china::china_cb::CHINA_CB[0].duoqian_address,
        ))
    }
}

pub struct RuntimeDustHandler;

impl OnUnbalanced<Credit<AccountId, Balances>> for RuntimeDustHandler {
    fn on_nonzero_unbalanced(amount: Credit<AccountId, Balances>) {
        if let Some(nrc_account) = RuntimeNrcAccountProvider::nrc_account() {
            if let Err(remaining) = Balances::resolve(&nrc_account, amount) {
                drop(remaining);
            }
        } else {
            drop(amount);
        }
    }
}

pub struct PowTxAmountExtractor;

impl onchain_transaction_pow::CallAmount<AccountId, RuntimeCall, Balance> for PowTxAmountExtractor {
    fn amount(
        who: &AccountId,
        call: &RuntimeCall,
    ) -> onchain_transaction_pow::AmountExtractResult<Balance> {
        match call {
            RuntimeCall::Balances(pallet_balances::Call::transfer_allow_death {
                value, ..
            }) => onchain_transaction_pow::AmountExtractResult::Amount(*value),
            RuntimeCall::Balances(pallet_balances::Call::transfer_keep_alive { value, .. }) => {
                onchain_transaction_pow::AmountExtractResult::Amount(*value)
            }
            RuntimeCall::Balances(pallet_balances::Call::force_transfer { value, .. }) => {
                onchain_transaction_pow::AmountExtractResult::Amount(*value)
            }
            RuntimeCall::Balances(pallet_balances::Call::force_unreserve { amount, .. }) => {
                onchain_transaction_pow::AmountExtractResult::Amount(*amount)
            }
            RuntimeCall::Balances(pallet_balances::Call::force_set_balance {
                new_free, ..
            }) => onchain_transaction_pow::AmountExtractResult::Amount(*new_free),
            RuntimeCall::Balances(pallet_balances::Call::force_adjust_total_issuance {
                delta,
                ..
            }) => onchain_transaction_pow::AmountExtractResult::Amount(*delta),
            RuntimeCall::Balances(pallet_balances::Call::burn { value, .. }) => {
                onchain_transaction_pow::AmountExtractResult::Amount(*value)
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
                onchain_transaction_pow::AmountExtractResult::Amount(value)
            }
            RuntimeCall::DuoqianManagePow(duoqian_manage_pow::pallet::Call::propose_create {
                amount,
                ..
            }) => onchain_transaction_pow::AmountExtractResult::Amount(*amount),
            RuntimeCall::DuoqianManagePow(duoqian_manage_pow::pallet::Call::propose_close {
                duoqian_address,
                ..
            }) => onchain_transaction_pow::AmountExtractResult::Amount(Balances::free_balance(
                duoqian_address,
            )),
            // 投票调用不涉及资金转移，无金额
            RuntimeCall::DuoqianManagePow(duoqian_manage_pow::pallet::Call::vote_create {
                ..
            }) => onchain_transaction_pow::AmountExtractResult::NoAmount,
            RuntimeCall::DuoqianManagePow(duoqian_manage_pow::pallet::Call::vote_close {
                ..
            }) => onchain_transaction_pow::AmountExtractResult::NoAmount,
            // 中文注释：以下调用类型明确属于“无金额交易”，放行且不计算手续费。
            RuntimeCall::System(_) => onchain_transaction_pow::AmountExtractResult::NoAmount,
            RuntimeCall::Timestamp(_) => onchain_transaction_pow::AmountExtractResult::NoAmount,
            RuntimeCall::FullnodePowReward(_) => {
                onchain_transaction_pow::AmountExtractResult::NoAmount
            }
            RuntimeCall::ShengBankStakeInterest(_) => {
                onchain_transaction_pow::AmountExtractResult::NoAmount
            }
            RuntimeCall::ResolutionIssuanceIss(_) => {
                onchain_transaction_pow::AmountExtractResult::NoAmount
            }
            RuntimeCall::ResolutionIssuanceGov(_) => {
                onchain_transaction_pow::AmountExtractResult::NoAmount
            }
            RuntimeCall::VotingEngineSystem(_) => {
                onchain_transaction_pow::AmountExtractResult::NoAmount
            }
            RuntimeCall::SfidCodeAuth(_) => onchain_transaction_pow::AmountExtractResult::NoAmount,
            RuntimeCall::CitizenLightnodeIssuance(_) => {
                onchain_transaction_pow::AmountExtractResult::NoAmount
            }
            RuntimeCall::AdminsOriginGov(_) => {
                onchain_transaction_pow::AmountExtractResult::NoAmount
            }
            RuntimeCall::RuntimeRootUpgrade(_) => {
                onchain_transaction_pow::AmountExtractResult::NoAmount
            }
            RuntimeCall::ResolutionDestroGov(_) => {
                onchain_transaction_pow::AmountExtractResult::NoAmount
            }
            RuntimeCall::GrandpaKeyGov(_) => onchain_transaction_pow::AmountExtractResult::NoAmount,
            RuntimeCall::DuoqianManagePow(_) => {
                onchain_transaction_pow::AmountExtractResult::NoAmount
            }
            // 机构转账提案/投票：全部免费（手续费在投票通过执行转账时由 pallet 内部扣取并分账）
            RuntimeCall::DuoqianTransferPow(_) => {
                onchain_transaction_pow::AmountExtractResult::NoAmount
            }
            // 中文注释：对 Balances 未覆盖分支按 Unknown 拒绝，避免”有金额但漏提取”。
            RuntimeCall::Balances(_) => onchain_transaction_pow::AmountExtractResult::Unknown,
            _ => onchain_transaction_pow::AmountExtractResult::Unknown,
        }
    }
}

pub struct RuntimeFeePayerExtractor;

impl onchain_transaction_pow::CallFeePayer<AccountId, RuntimeCall> for RuntimeFeePayerExtractor {
    fn fee_payer(_who: &AccountId, _call: &RuntimeCall) -> Option<AccountId> {
        // 机构转账提案/投票已改为 NoAmount（免费），无需转嫁手续费。
        // 手续费在投票通过后由 pallet 内部按分账规则扣取。
        None
    }
}

/// 省储行质押利息模块配置：
/// - 使用 Balances 作为铸币/记账货币
/// - 每年区块数统一采用 primitives 中的制度常量
impl shengbank_stake_interest::Config for Runtime {
    type Currency = Balances;
    type BlocksPerYear = ConstU64<{ primitives::pow_const::BLOCKS_PER_YEAR }>;
    type WeightInfo = shengbank_stake_interest::weights::SubstrateWeight<Runtime>;
}

/// PoW 作者解析器：
/// 从区块 pre-runtime digest 中读取 POW_ENGINE_ID 的负载（sr25519 公钥），
/// 派生为 AccountId。配合 seal 中的签名实现矿工身份密码学绑定。
pub struct PowDigestAuthor;

impl FindAuthor<AccountId> for PowDigestAuthor {
    fn find_author<'a, I>(digests: I) -> Option<AccountId>
    where
        I: 'a + IntoIterator<Item = (sp_runtime::ConsensusEngineId, &'a [u8])>,
    {
        digests.into_iter().find_map(|(engine_id, data)| {
            if engine_id == sp_consensus_pow::POW_ENGINE_ID {
                sp_core::sr25519::Public::decode(&mut &data[..])
                    .ok()
                    .map(|public| {
                        use sp_runtime::traits::IdentifyAccount;
                        sp_runtime::MultiSigner::from(public).into_account()
                    })
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
    type WeightInfo = fullnode_pow_reward::weights::SubstrateWeight<Runtime>;
}

pub struct RuntimeDuoqianAddressValidator;

impl duoqian_manage_pow::DuoqianAddressValidator<AccountId> for RuntimeDuoqianAddressValidator {
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
pub struct RuntimeSfidInstitutionVerifier;

pub struct RuntimeProtectedSourceChecker;
pub struct RuntimeInstitutionAssetGuard;

impl duoqian_manage_pow::ProtectedSourceChecker<AccountId> for RuntimeProtectedSourceChecker {
    fn is_protected(address: &AccountId) -> bool {
        is_keyless_account(address)
    }
}

impl institution_asset_guard::InstitutionAssetGuard<AccountId> for RuntimeInstitutionAssetGuard {
    fn can_spend(
        source: &AccountId,
        action: institution_asset_guard::InstitutionAssetAction,
    ) -> bool {
        if is_keyless_account(source) {
            return false;
        }

        if is_reserved_duoqian_account(source) {
            return matches!(
                action,
                institution_asset_guard::InstitutionAssetAction::DuoqianTransferExecute
                    | institution_asset_guard::InstitutionAssetAction::DuoqianCloseExecute
            );
        }

        if is_reserved_fee_account(source) {
            return matches!(
                action,
                institution_asset_guard::InstitutionAssetAction::OffchainFeeSweepExecute
            );
        }

        true
    }
}

impl duoqian_manage_pow::DuoqianReservedAddressChecker<AccountId>
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

        is_reserved_duoqian_account(address)
    }
}

impl
    duoqian_manage_pow::SfidInstitutionVerifier<
        duoqian_manage_pow::pallet::SfidNameOf<Runtime>,
        duoqian_manage_pow::pallet::RegisterNonceOf<Runtime>,
        duoqian_manage_pow::pallet::RegisterSignatureOf<Runtime>,
    > for RuntimeSfidInstitutionVerifier
{
    fn verify_institution_registration(
        sfid_id: &[u8],
        name: &duoqian_manage_pow::pallet::SfidNameOf<Runtime>,
        nonce: &duoqian_manage_pow::pallet::RegisterNonceOf<Runtime>,
        signature: &duoqian_manage_pow::pallet::RegisterSignatureOf<Runtime>,
    ) -> bool {
        #[cfg(feature = "runtime-benchmarks")]
        {
            return !sfid_id.is_empty() && !name.is_empty() && !nonce.is_empty() && !signature.is_empty();
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
                b"GMB_SFID_INSTITUTION_V2",
                frame_system::Pallet::<Runtime>::block_hash(0),
                sfid_id,
                name.as_slice(),
                nonce.as_slice(),
            );
            let msg = blake2_256(&payload.encode());

            sr25519_verify(&signature, &msg, &public)
        }
    }
}

impl duoqian_manage_pow::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type InternalVoteEngine = voting_engine_system::Pallet<Runtime>;
    type AddressValidator = RuntimeDuoqianAddressValidator;
    type ReservedAddressChecker = RuntimeDuoqianReservedAddressChecker;
    type ProtectedSourceChecker = RuntimeProtectedSourceChecker;
    type InstitutionAssetGuard = RuntimeInstitutionAssetGuard;
    type SfidInstitutionVerifier = RuntimeSfidInstitutionVerifier;
    type FeeRouter = TransferFeeRouter;
    type MaxAdmins = ConstU32<64>;
    type MaxSfidIdLength = ConstU32<96>;
    type MaxSfidNameLength = ConstU32<128>;
    type MaxRegisterNonceLength = ConstU32<64>;
    type MaxRegisterSignatureLength = ConstU32<64>;
    type MinCreateAmount = ConstU128<111>;
    type MinCloseBalance = ConstU128<121>;
    type WeightInfo = duoqian_manage_pow::weights::SubstrateWeight<Runtime>;
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
            b"GMB_SFID_BIND_V3",
            frame_system::Pallet::<Runtime>::block_hash(0),
            account,
            credential.binding_id,
            credential.bind_nonce.as_slice(),
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
        binding_id: Hash,
        proposal_id: u64,
        nonce: &sfid_code_auth::pallet::NonceOf<Runtime>,
        signature: &sfid_code_auth::pallet::SignatureOf<Runtime>,
    ) -> bool {
        #[cfg(feature = "runtime-benchmarks")]
        {
            let _ = (account, binding_id, proposal_id);
            return !nonce.is_empty() && !signature.is_empty();
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
                b"GMB_SFID_VOTE_V3",
                frame_system::Pallet::<Runtime>::block_hash(0),
                account,
                binding_id,
                proposal_id,
                nonce.as_slice(),
            );
            let msg = blake2_256(&payload.encode());

            sr25519_verify(&signature, &msg, &public)
        }
    }
}

impl sfid_code_auth::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxCredentialNonceLength = ConstU32<64>;
    // 中文注释：SFID 绑定与投票验签统一使用 64 字节原始 sr25519 签名。
    type MaxCredentialSignatureLength = ConstU32<64>;
    type SfidVerifier = RuntimeSfidVerifier;
    type SfidVoteVerifier = RuntimeSfidVoteVerifier;
    type OnSfidBound = CitizenLightnodeIssuance;
    type WeightInfo = sfid_code_auth::weights::SubstrateWeight<Runtime>;
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
                b"GMB_SFID_POPULATION_V3",
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
    type WeightInfo = citizen_lightnode_issuance::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    /// 决议发行治理参数（统一来源于 primitives 常量）。
    pub const ResolutionIssuanceMaxReasonLen: u32 = primitives::count_const::RESOLUTION_ISSUANCE_MAX_REASON_LEN;
    pub const ResolutionIssuanceMaxAllocations: u32 = primitives::count_const::RESOLUTION_ISSUANCE_MAX_ALLOCATIONS;
    pub const ResolutionIssuanceMaxTotalIssuance: u128 = u128::MAX;
    pub const ResolutionIssuanceMaxSingleIssuance: u128 = 14_434_973_780_000;
    /// Runtime 升级治理提案备注最大长度。
    pub const RuntimeUpgradeMaxReasonLen: u32 = 1024;
    /// Runtime wasm 最大长度（字节）。
    pub const RuntimeUpgradeMaxCodeSize: u32 = 5 * 1024 * 1024;
    /// 管理员治理：单机构管理员列表上限（覆盖国储会 19 人规模）。
    pub const MaxAdminsPerInstitution: u32 = 32;
    /// GRANDPA authority set 变更生效延迟（单位：区块）。
    /// 取非 0，给运维注入新 gran 私钥预留窗口，避免立即切换导致短时失票。
    pub const GrandpaAuthoritySetChangeDelay: u32 = 30;
}

impl admins_origin_gov::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxAdminsPerInstitution = MaxAdminsPerInstitution;
    type InternalVoteEngine = VotingEngineSystem;
    type WeightInfo = admins_origin_gov::weights::SubstrateWeight<Runtime>;
}

impl resolution_destro_gov::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type InternalVoteEngine = VotingEngineSystem;
    type WeightInfo = resolution_destro_gov::weights::SubstrateWeight<Runtime>;
}

impl grandpa_key_gov::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type GrandpaChangeDelay = GrandpaAuthoritySetChangeDelay;
    type InternalVoteEngine = VotingEngineSystem;
    type WeightInfo = grandpa_key_gov::weights::SubstrateWeight<Runtime>;
}

/// 转账提案手续费分账适配器：将旧 Currency NegativeImbalance 转换后
/// 交给现有 PowOnchainFeeRouter 处理（80% 矿工 / 10% 国储会 / 10% 销毁）。
pub struct TransferFeeRouter;

impl frame_support::traits::OnUnbalanced<pallet_balances::NegativeImbalance<Runtime>>
    for TransferFeeRouter
{
    fn on_nonzero_unbalanced(amount: pallet_balances::NegativeImbalance<Runtime>) {
        use frame_support::traits::fungible::Balanced;
        // 将旧 NegativeImbalance 转为新 Credit（金额相同，drop 行为兼容）
        let value = frame_support::traits::Imbalance::peek(&amount);
        // 消费旧 imbalance（让余额变化生效）
        drop(amount);
        // 用 Balanced trait 从"零"铸造等额 Credit 传给现有 router
        // 注意：drop(NegativeImbalance) 已将资金从流通中移除，
        // issue() 再铸回等额 Credit 让 router 分配，总量不变。
        let credit = <Balances as Balanced<AccountId>>::issue(value);

        type FeeRouter = onchain_transaction_pow::PowOnchainFeeRouter<
            Runtime,
            Balances,
            PowDigestAuthor,
            RuntimeNrcAccountProvider,
        >;
        <FeeRouter as frame_support::traits::tokens::imbalance::OnUnbalanced<_>>::on_unbalanced(
            credit,
        );
    }
}

impl duoqian_transfer_pow::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxRemarkLen = ConstU32<256>;
    type FeeRouter = TransferFeeRouter;
    type WeightInfo = duoqian_transfer_pow::weights::SubstrateWeight<Runtime>;
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
        let admin = AccountId::new(primitives::china::china_cb::CHINA_CB[0].duoqian_admins[0]);
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
    type MaxSingleIssuance = ResolutionIssuanceMaxSingleIssuance;
    type WeightInfo = resolution_issuance_iss::weights::SubstrateWeight<Runtime>;
}

impl resolution_issuance_gov::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type NrcProposeOrigin = EnsureNrcAdmin;
    type RecipientSetOrigin = frame_system::EnsureRoot<AccountId>;
    // 中文注释：禁用外部 finalize 入口，只允许投票引擎回调路径落地结果。
    type JointVoteFinalizeOrigin = EnsureJointVoteFinalizeOrigin;
    type IssuanceExecutor = ResolutionIssuanceIss;
    type WeightInfo = resolution_issuance_gov::weights::SubstrateWeight<Runtime>;
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
    type DeveloperUpgradeCheck = GenesisPallet;
    type MaxReasonLen = RuntimeUpgradeMaxReasonLen;
    type MaxRuntimeCodeSize = RuntimeUpgradeMaxCodeSize;
    type MaxSnapshotNonceLength = ConstU32<64>;
    type MaxSnapshotSignatureLength = ConstU32<64>;
    type WeightInfo = runtime_root_upgrade::weights::SubstrateWeight<Runtime>;
}

pub struct RuntimeSetCodeExecutor;

impl runtime_root_upgrade::RuntimeCodeExecutor for RuntimeSetCodeExecutor {
    fn execute_runtime_code(code: &[u8]) -> DispatchResult {
        #[cfg(feature = "runtime-benchmarks")]
        {
            // 中文注释：benchmark 需要衡量治理编排本身的真实路径，
            // 但不应真的改写 runtime :code 存储，因此这里使用成功的 no-op 执行器。
            return if code.is_empty() {
                Err(sp_runtime::DispatchError::Other("empty runtime code"))
            } else {
                Ok(())
            };
        }

        #[cfg(not(feature = "runtime-benchmarks"))]
        {
            let set_code_call = frame_system::Call::<Runtime>::set_code {
                code: code.to_vec(),
            };
            set_code_call
                .dispatch_bypass_filter(frame_system::RawOrigin::Root.into())
                .map(|_| ())
                .map_err(|e| e.error)
        }
    }
}

pub struct RuntimeJointVoteResultCallback;

impl voting_engine_system::JointVoteResultCallback for RuntimeJointVoteResultCallback {
    fn on_joint_vote_finalized(vote_proposal_id: u64, approved: bool) -> DispatchResult {
        #[cfg(feature = "runtime-benchmarks")]
        {
            let _ = (vote_proposal_id, approved);
            Ok(())
        }

        #[cfg(not(feature = "runtime-benchmarks"))]
        {
            if resolution_issuance_gov::Pallet::<Runtime>::owns_proposal(vote_proposal_id) {
                return <ResolutionIssuanceGov as voting_engine_system::JointVoteResultCallback>::on_joint_vote_finalized(
                vote_proposal_id,
                approved,
            );
            }

            // runtime_root_upgrade 使用统一 ID，尝试直接回调；
            // 如果 proposal_id 不属于该模块，on_joint_vote_finalized 会返回 ProposalNotFound。
            if let Ok(()) = <RuntimeRootUpgrade as voting_engine_system::JointVoteResultCallback>::on_joint_vote_finalized(
                vote_proposal_id,
                approved,
            ) {
                return Ok(());
            }

            Err(sp_runtime::DispatchError::Other(
                "joint vote proposal not found in any module",
            ))
        }
    }
}

impl voting_engine_system::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxVoteNonceLength = ConstU32<64>;
    type MaxVoteSignatureLength = ConstU32<64>;
    type MaxAutoFinalizePerBlock = ConstU32<2_048>;
    type MaxProposalsPerExpiry = ConstU32<2_048>;
    type MaxProposalDataLen = ConstU32<{ 100 * 1024 }>;
    type MaxProposalObjectLen = ConstU32<{ 10 * 1024 * 1024 }>;
    type MaxCleanupStepsPerBlock = ConstU32<8>;
    type CleanupKeysPerStep = ConstU32<256>;
    type SfidEligibility = RuntimeSfidEligibility;
    type PopulationSnapshotVerifier = RuntimePopulationSnapshotVerifier;
    type JointVoteResultCallback = RuntimeJointVoteResultCallback;
    type InternalAdminProvider = RuntimeInternalAdminProvider;
    type InternalAdminCountProvider = RuntimeInternalAdminCountProvider;
    type InternalThresholdProvider = RuntimeInternalThresholdProvider;
    type TimeProvider = pallet_timestamp::Pallet<Runtime>;
    type WeightInfo = voting_engine_system::weights::SubstrateWeight<Runtime>;
}

impl pow_difficulty_module::Config for Runtime {
    type WeightInfo = pow_difficulty_module::weights::SubstrateWeight<Runtime>;
}

frame_support::parameter_types! {
    pub const MaxDeclarationLen: u32 = 2048;
}

impl genesis_pallet::Config for Runtime {
    type WeightInfo = genesis_pallet::weights::SubstrateWeight<Runtime>;
    type MaxDeclarationLen = MaxDeclarationLen;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ResolutionDestroGov;
    use duoqian_manage_pow::DuoqianReservedAddressChecker;
    use frame_support::assert_ok;
    use frame_support::traits::Currency;
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
        use codec::Encode;
        new_test_ext().execute_with(|| {
            // 统一 ID：proposal_id 即投票引擎 ID，不再有双 ID 映射
            let proposal_id = 99u64;
            let recipient =
                AccountId::new(primitives::china::china_cb::CHINA_CB[1].duoqian_address);
            let total_amount = 123u128;

            // 直接在投票引擎 ProposalData 中写入带 MODULE_TAG 前缀的业务数据
            let data = resolution_issuance_gov::pallet::IssuanceProposalData {
                proposer: recipient.clone(),
                reason: b"runtime-integration".to_vec(),
                total_amount,
                allocations: vec![resolution_issuance_gov::pallet::RecipientAmount {
                    recipient: recipient.clone(),
                    amount: total_amount,
                }],
            };
            let mut encoded = Vec::from(resolution_issuance_gov::MODULE_TAG);
            encoded.extend_from_slice(&data.encode());
            voting_engine_system::Pallet::<Runtime>::store_proposal_data(proposal_id, encoded)
                .expect("store_proposal_data should succeed");
            voting_engine_system::Pallet::<Runtime>::store_proposal_meta(
                proposal_id,
                System::block_number(),
            );

            resolution_issuance_gov::pallet::VotingProposalCount::<Runtime>::put(1u32);
            let binding_id = <Runtime as frame_system::Config>::Hashing::hash(b"cleanup-sfid");
            let nonce_hash = <Runtime as frame_system::Config>::Hashing::hash(b"cleanup-nonce");
            sfid_code_auth::pallet::UsedVoteNonce::<Runtime>::insert(
                proposal_id,
                (binding_id, nonce_hash),
                true,
            );

            assert_ok!(RuntimeJointVoteResultCallback::on_joint_vote_finalized(
                proposal_id,
                true
            ));

            // 验证 VotingProposalCount 已递减
            assert_eq!(
                resolution_issuance_gov::pallet::VotingProposalCount::<Runtime>::get(),
                0u32
            );

            // 中文注释：自动延迟清理由 voting-engine-system 自身单测覆盖，
            // 这里仅验证 runtime 包装层能正确透传到 SFID 投票凭证清理接口。
            RuntimeSfidEligibility::cleanup_vote_credentials(proposal_id);

            assert!(!sfid_code_auth::pallet::UsedVoteNonce::<Runtime>::get(
                proposal_id,
                (binding_id, nonce_hash)
            ));

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
                RuntimeOrigin::signed(AccountId::new(CHINA_CB[0].duoqian_admins[0])),
                voting_engine_system::internal_vote::ORG_NRC,
                nrc_institution,
                destroy_amount,
            ));

            let pid = VotingEngineSystem::next_proposal_id().saturating_sub(1);

            for i in 0..13 {
                assert_ok!(ResolutionDestroGov::vote_destroy(
                    RuntimeOrigin::signed(AccountId::new(CHINA_CB[0].duoqian_admins[i])),
                    pid,
                    true,
                ));
            }

            // 提案数据由 voting-engine-system 延迟清理，执行后仍保留
            assert!(VotingEngineSystem::get_proposal_data(pid).is_some());

            assert_eq!(
                Balances::free_balance(&nrc_account),
                initial_balance - destroy_amount
            );
            assert_eq!(Balances::total_issuance(), issuance_before - destroy_amount);
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
            let no_amount = <PowTxAmountExtractor as onchain_transaction_pow::CallAmount<
                AccountId,
                RuntimeCall,
                Balance,
            >>::amount(&who, &system_call);
            assert!(matches!(
                no_amount,
                onchain_transaction_pow::AmountExtractResult::NoAmount
            ));

            let transfer_call =
                RuntimeCall::Balances(pallet_balances::Call::transfer_allow_death {
                    dest: sp_runtime::MultiAddress::Id(recipient),
                    value: 123,
                });
            let amount = <PowTxAmountExtractor as onchain_transaction_pow::CallAmount<
                AccountId,
                RuntimeCall,
                Balance,
            >>::amount(&who, &transfer_call);
            match amount {
                onchain_transaction_pow::AmountExtractResult::Amount(v) => assert_eq!(v, 123),
                _ => panic!("expected amount path"),
            }
        });
    }

    #[test]
    fn pow_tx_amount_extractor_covers_duoqian_propose_create_and_close() {
        new_test_ext().execute_with(|| {
            let (p1, _) = sr25519::Pair::generate();
            let (p2, _) = sr25519::Pair::generate();
            let signer1 = MultiSigner::from(p1.public());
            let who: AccountId = signer1.into_account();
            let admin2: AccountId = MultiSigner::from(p2.public()).into_account();

            let duoqian_address = AccountId::new([77u8; 32]);
            let beneficiary = AccountId::new([78u8; 32]);
            let sfid_id: duoqian_manage_pow::pallet::SfidIdOf<Runtime> =
                b"GFR-LN001-CB0C-runtime-20260222"
                    .to_vec()
                    .try_into()
                    .expect("sfid id should fit");
            let admins: duoqian_manage_pow::pallet::DuoqianAdminsOf<Runtime> =
                vec![who.clone(), admin2.clone()]
                    .try_into()
                    .expect("admins should fit");

            let create_call =
                RuntimeCall::DuoqianManagePow(duoqian_manage_pow::pallet::Call::propose_create {
                    sfid_id,
                    admin_count: 2,
                    duoqian_admins: admins.clone(),
                    threshold: 2,
                    amount: 1_000,
                });
            let create_amount = <PowTxAmountExtractor as onchain_transaction_pow::CallAmount<
                AccountId,
                RuntimeCall,
                Balance,
            >>::amount(&who, &create_call);
            match create_amount {
                onchain_transaction_pow::AmountExtractResult::Amount(v) => assert_eq!(v, 1_000),
                _ => panic!("expected create amount"),
            }

            let _ = Balances::deposit_creating(&duoqian_address, 777);
            let close_call =
                RuntimeCall::DuoqianManagePow(duoqian_manage_pow::pallet::Call::propose_close {
                    duoqian_address,
                    beneficiary,
                });
            let close_amount = <PowTxAmountExtractor as onchain_transaction_pow::CallAmount<
                AccountId,
                RuntimeCall,
                Balance,
            >>::amount(&who, &close_call);
            match close_amount {
                onchain_transaction_pow::AmountExtractResult::Amount(v) => assert_eq!(v, 777),
                _ => panic!("expected close amount"),
            }
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

        let blocked_force_unreserve =
            RuntimeCall::Balances(pallet_balances::Call::force_unreserve {
                who: sp_runtime::MultiAddress::Id(AccountId::new(
                    primitives::china::china_ch::CHINA_CH[0].keyless_address,
                )),
                amount: 1,
            });
        assert!(!RuntimeCallFilter::contains(&blocked_force_unreserve));

        let blocked_force_set_balance =
            RuntimeCall::Balances(pallet_balances::Call::force_set_balance {
                who: sp_runtime::MultiAddress::Id(AccountId::new(
                    primitives::china::china_ch::CHINA_CH[0].keyless_address,
                )),
                new_free: 1,
            });
        assert!(!RuntimeCallFilter::contains(&blocked_force_set_balance));
    }

    #[test]
    fn pow_digest_author_finds_pow_engine_author() {
        // 中文注释：pre_digest 现在存储 sr25519 公钥，PowDigestAuthor 解码后派生 AccountId。
        let public = sp_core::sr25519::Public::from_raw([21u8; 32]);
        let expected_account: AccountId =
            sp_runtime::MultiSigner::from(public).into_account();
        let encoded = public.encode();
        let digests: Vec<(sp_runtime::ConsensusEngineId, &[u8])> = vec![
            (*b"TEST", b"ignored".as_ref()),
            (sp_consensus_pow::POW_ENGINE_ID, encoded.as_slice()),
        ];
        let found = PowDigestAuthor::find_author(digests);
        assert_eq!(found, Some(expected_account));
    }

    #[test]
    fn joint_vote_callback_missing_proposal_and_runtime_upgrade_route() {
        new_test_ext().execute_with(|| {
            // 不存在的提案 ID 应返回错误
            assert!(
                RuntimeJointVoteResultCallback::on_joint_vote_finalized(999_999, true).is_err()
            );

            // 通过 voting-engine-system 的 ProposalData 写入提案数据（模块已无本地存储）
            let proposal_id = 7u64;
            let proposer = AccountId::new(CHINA_CB[0].duoqian_admins[0]);
            let reason: runtime_root_upgrade::pallet::ReasonOf<Runtime> =
                b"upgrade".to_vec().try_into().expect("reason");
            let code: runtime_root_upgrade::pallet::CodeOf<Runtime> =
                vec![1u8, 2, 3].try_into().expect("code");
            let code_hash = <Runtime as frame_system::Config>::Hashing::hash(code.as_slice());

            let proposal = runtime_root_upgrade::pallet::Proposal::<Runtime> {
                proposer,
                reason,
                code_hash,
                has_code: true,
                status: runtime_root_upgrade::pallet::ProposalStatus::Voting,
            };
            let mut encoded = Vec::from(runtime_root_upgrade::MODULE_TAG);
            encoded.extend_from_slice(&codec::Encode::encode(&proposal));
            assert_ok!(
                voting_engine_system::Pallet::<Runtime>::store_proposal_data(proposal_id, encoded)
            );
            assert_ok!(
                voting_engine_system::Pallet::<Runtime>::store_proposal_object(
                    proposal_id,
                    runtime_root_upgrade::pallet::PROPOSAL_OBJECT_KIND_RUNTIME_WASM,
                    code.into_inner()
                )
            );

            // 回调拒绝 → 提案状态应变为 Rejected
            assert_ok!(RuntimeJointVoteResultCallback::on_joint_vote_finalized(
                proposal_id,
                false
            ));
            let raw = voting_engine_system::Pallet::<Runtime>::get_proposal_data(proposal_id)
                .expect("proposal data should exist");
            let tag = runtime_root_upgrade::MODULE_TAG;
            assert!(raw.len() >= tag.len() && &raw[..tag.len()] == tag, "MODULE_TAG mismatch");
            let updated = runtime_root_upgrade::pallet::Proposal::<Runtime>::decode(&mut &raw[tag.len()..])
                .expect("should decode");
            assert!(matches!(
                updated.status,
                runtime_root_upgrade::pallet::ProposalStatus::Rejected
            ));
            assert!(updated.has_code, "对象层数据应保留到统一清理阶段");
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
            let binding_id = <Runtime as frame_system::Config>::Hashing::hash(b"sfid-verify");
            let bind_nonce: sfid_code_auth::pallet::NonceOf<Runtime> =
                b"bind-nonce".to_vec().try_into().expect("nonce should fit");
            let bind_payload = (
                b"GMB_SFID_BIND_V3",
                frame_system::Pallet::<Runtime>::block_hash(0),
                &account,
                binding_id,
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
                binding_id,
                bind_nonce: bind_nonce.clone(),
                signature: bind_signature,
            };
            assert!(RuntimeSfidVerifier::verify(&account, &bind_credential));

            let bad_bind_signature: sfid_code_auth::pallet::SignatureOf<Runtime> =
                vec![7u8; 63].try_into().expect("signature should fit");
            let bad_bind_credential = sfid_code_auth::BindCredential {
                binding_id,
                bind_nonce,
                signature: bad_bind_signature,
            };
            assert!(!RuntimeSfidVerifier::verify(&account, &bad_bind_credential));

            let vote_nonce: sfid_code_auth::pallet::NonceOf<Runtime> =
                b"vote-nonce".to_vec().try_into().expect("nonce should fit");
            let vote_signature: sfid_code_auth::pallet::SignatureOf<Runtime> = pair
                .sign(&blake2_256(
                    &(
                        b"GMB_SFID_VOTE_V3",
                        frame_system::Pallet::<Runtime>::block_hash(0),
                        &account,
                        binding_id,
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
                binding_id,
                9,
                &vote_nonce,
                &vote_signature
            ));

            let pop_nonce: voting_engine_system::pallet::VoteNonceOf<Runtime> =
                b"pop-nonce".to_vec().try_into().expect("nonce should fit");
            let pop_signature: voting_engine_system::pallet::VoteSignatureOf<Runtime> = pair
                .sign(&blake2_256(
                    &(
                        b"GMB_SFID_POPULATION_V3",
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
            let binding_id = <Runtime as frame_system::Config>::Hashing::hash(b"sfid-wrap");
            sfid_code_auth::pallet::BindingIdToAccount::<Runtime>::insert(binding_id, who.clone());
            sfid_code_auth::pallet::AccountToBindingId::<Runtime>::insert(who.clone(), binding_id);

            assert!(RuntimeSfidEligibility::is_eligible(&binding_id, &who));
            assert!(!RuntimeSfidEligibility::is_eligible(
                &binding_id,
                &AccountId::new([42u8; 32])
            ));

            let nonce = b"wrap-nonce";
            let vote_msg = blake2_256(
                &(
                    b"GMB_SFID_VOTE_V3",
                    frame_system::Pallet::<Runtime>::block_hash(0),
                    &who,
                    binding_id,
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
                binding_id,
                88,
                &nonce_bounded,
                &signature_bounded
            ));
            assert!(RuntimeSfidEligibility::verify_and_consume_vote_credential(
                &binding_id,
                &who,
                88,
                nonce,
                &signature
            ));
            assert!(!RuntimeSfidEligibility::verify_and_consume_vote_credential(
                &binding_id,
                &who,
                88,
                nonce,
                &signature
            ));
        });
    }

    #[test]
    fn ensure_nrc_admin_and_runtime_internal_admin_provider_paths() {
        new_test_ext().execute_with(|| {
            let nrc_id = reserve_pallet_id_to_bytes(CHINA_CB[0].shenfen_id).expect("nrc id");
            let nrc_admin = AccountId::new(CHINA_CB[0].duoqian_admins[0]);
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

    #[test]
    fn runtime_sfid_institution_verifier_uses_runtime_main_key() {
        new_test_ext().execute_with(|| {
            let (pair, _) = sr25519::Pair::generate();
            let main: AccountId = MultiSigner::from(pair.public()).into_account();
            sfid_code_auth::pallet::SfidMainAccount::<Runtime>::put(main);
            let sfid_id = b"GFR-LN001-CB0C-000000001-20260222";
            let register_nonce: duoqian_manage_pow::pallet::RegisterNonceOf<Runtime> =
                b"register-nonce"
                    .to_vec()
                    .try_into()
                    .expect("nonce should fit");
            let register_name: duoqian_manage_pow::pallet::SfidNameOf<Runtime> =
                b"test-name"
                    .to_vec()
                    .try_into()
                    .expect("name should fit");
            let register_signature: duoqian_manage_pow::pallet::RegisterSignatureOf<Runtime> = pair
                .sign(&blake2_256(
                    &(
                        b"GMB_SFID_INSTITUTION_V2",
                        frame_system::Pallet::<Runtime>::block_hash(0),
                        sfid_id.as_slice(),
                        register_name.as_slice(),
                        register_nonce.as_slice(),
                    )
                        .encode(),
                ))
                .0
                .to_vec()
                .try_into()
                .expect("signature should fit");
            assert!(
                <RuntimeSfidInstitutionVerifier as duoqian_manage_pow::SfidInstitutionVerifier<
                    duoqian_manage_pow::pallet::SfidNameOf<Runtime>,
                    duoqian_manage_pow::pallet::RegisterNonceOf<Runtime>,
                    duoqian_manage_pow::pallet::RegisterSignatureOf<Runtime>,
                >>::verify_institution_registration(
                    sfid_id.as_slice(),
                    &register_name,
                    &register_nonce,
                    &register_signature,
                )
            );

            let bad_signature: duoqian_manage_pow::pallet::RegisterSignatureOf<Runtime> =
                vec![9u8; 63].try_into().expect("signature should fit");
            assert!(
                !<RuntimeSfidInstitutionVerifier as duoqian_manage_pow::SfidInstitutionVerifier<
                    duoqian_manage_pow::pallet::SfidNameOf<Runtime>,
                    duoqian_manage_pow::pallet::RegisterNonceOf<Runtime>,
                    duoqian_manage_pow::pallet::RegisterSignatureOf<Runtime>,
                >>::verify_institution_registration(
                    sfid_id.as_slice(),
                    &register_name,
                    &register_nonce,
                    &bad_signature,
                )
            );
        });
    }
}

pub struct RuntimeInternalAdminProvider;

impl voting_engine_system::InternalAdminProvider<AccountId> for RuntimeInternalAdminProvider {
    fn is_internal_admin(
        org: u8,
        institution: voting_engine_system::InstitutionPalletId,
        who: &AccountId,
    ) -> bool {
        match org {
            // 注册多签机构：从 DuoqianAccounts 读取管理员列表
            voting_engine_system::internal_vote::ORG_DUOQIAN => {
                let Ok(account) = AccountId::decode(&mut &institution[..32]) else {
                    return false;
                };
                if let Some(duoqian) = duoqian_manage_pow::DuoqianAccounts::<Runtime>::get(&account)
                {
                    duoqian.duoqian_admins.iter().any(|admin| admin == who)
                } else {
                    false
                }
            }
            // 治理机构（NRC/PRC/PRB）：从 admins_origin_gov 读取管理员
            _ => {
                if let Some(admins) = admins_origin_gov::CurrentAdmins::<Runtime>::get(institution)
                {
                    admins.into_inner().iter().any(|admin| admin == who)
                } else {
                    false
                }
            }
        }
    }
}

pub struct RuntimeInternalThresholdProvider;

impl voting_engine_system::InternalThresholdProvider for RuntimeInternalThresholdProvider {
    fn pass_threshold(
        org: u8,
        institution: voting_engine_system::InstitutionPalletId,
    ) -> Option<u32> {
        match org {
            // 治理机构：硬编码阈值
            voting_engine_system::internal_vote::ORG_NRC
            | voting_engine_system::internal_vote::ORG_PRC
            | voting_engine_system::internal_vote::ORG_PRB => {
                voting_engine_system::internal_vote::governance_org_pass_threshold(org)
            }
            // 注册多签机构：从链上 DuoqianAccounts 动态读取阈值
            voting_engine_system::internal_vote::ORG_DUOQIAN => {
                // institution 48 字节 → 解码为 AccountId32 → 查 DuoqianAccounts
                let account = AccountId::decode(&mut &institution[..32]).ok()?;
                let duoqian = duoqian_manage_pow::DuoqianAccounts::<Runtime>::get(&account)?;
                Some(duoqian.threshold)
            }
            _ => None,
        }
    }
}

pub struct RuntimeInternalAdminCountProvider;

impl voting_engine_system::InternalAdminCountProvider for RuntimeInternalAdminCountProvider {
    fn admin_count(org: u8, institution: voting_engine_system::InstitutionPalletId) -> Option<u32> {
        match org {
            // 注册多签机构：从 DuoqianAccounts 读取当前管理员人数
            voting_engine_system::internal_vote::ORG_DUOQIAN => {
                let account = AccountId::decode(&mut &institution[..32]).ok()?;
                let duoqian = duoqian_manage_pow::DuoqianAccounts::<Runtime>::get(&account)?;
                u32::try_from(duoqian.duoqian_admins.len()).ok()
            }
            // 治理机构：从 admins_origin_gov 读取当前管理员人数
            _ => admins_origin_gov::CurrentAdmins::<Runtime>::get(institution)
                .and_then(|admins| u32::try_from(admins.len()).ok()),
        }
    }
}

pub struct RuntimeSfidEligibility;

impl voting_engine_system::SfidEligibility<AccountId, Hash> for RuntimeSfidEligibility {
    fn is_eligible(binding_id: &Hash, who: &AccountId) -> bool {
        #[cfg(feature = "runtime-benchmarks")]
        {
            let _ = (
                who,
                sfid_code_auth::pallet::BindingIdToAccount::<Runtime>::get(binding_id),
            );
            true
        }

        #[cfg(not(feature = "runtime-benchmarks"))]
        {
            <sfid_code_auth::Pallet<Runtime> as sfid_code_auth::SfidEligibilityProvider<
                AccountId,
                Hash,
            >>::is_eligible(binding_id, who)
        }
    }

    fn verify_and_consume_vote_credential(
        binding_id: &Hash,
        who: &AccountId,
        proposal_id: u64,
        nonce: &[u8],
        signature: &[u8],
    ) -> bool {
        #[cfg(feature = "runtime-benchmarks")]
        {
            let _ = who;
            if nonce.is_empty() || signature.is_empty() {
                return false;
            }

            let nonce_hash = <Runtime as frame_system::Config>::Hashing::hash_of(&nonce);
            let vote_nonce_key = (binding_id.clone(), nonce_hash);
            if sfid_code_auth::pallet::UsedVoteNonce::<Runtime>::get(
                proposal_id,
                vote_nonce_key.clone(),
            ) {
                return false;
            }

            sfid_code_auth::pallet::UsedVoteNonce::<Runtime>::insert(
                proposal_id,
                vote_nonce_key,
                true,
            );
            true
        }

        #[cfg(not(feature = "runtime-benchmarks"))]
        {
            <sfid_code_auth::Pallet<Runtime> as sfid_code_auth::SfidEligibilityProvider<
                AccountId,
                Hash,
            >>::verify_and_consume_vote_credential(
                binding_id, who, proposal_id, nonce, signature
            )
        }
    }

    fn cleanup_vote_credentials(proposal_id: u64) {
        <sfid_code_auth::Pallet<Runtime> as sfid_code_auth::SfidEligibilityProvider<
            AccountId,
            Hash,
        >>::cleanup_vote_credentials(proposal_id)
    }

    fn cleanup_vote_credentials_chunk(
        proposal_id: u64,
        limit: u32,
    ) -> voting_engine_system::VoteCredentialCleanup {
        let result = sfid_code_auth::pallet::UsedVoteNonce::<Runtime>::clear_prefix(
            proposal_id,
            limit,
            None,
        );
        voting_engine_system::VoteCredentialCleanup {
            removed: result.unique,
            loops: result.loops,
            has_remaining: result.maybe_cursor.is_some(),
        }
    }
}

// ============================================================================
// 机构资金白名单允许矩阵测试
// ============================================================================

#[cfg(test)]
mod asset_guard_tests {
    use super::*;
    use institution_asset_guard::{InstitutionAssetAction, InstitutionAssetGuard};

    fn keyless_account() -> AccountId {
        AccountId::new(primitives::china::china_ch::CHINA_CH[0].keyless_address)
    }

    fn reserved_duoqian_account() -> AccountId {
        AccountId::new(primitives::china::china_cb::CHINA_CB[1].duoqian_address)
    }

    fn reserved_fee_account() -> AccountId {
        use frame_support::PalletId;
        use sp_runtime::traits::AccountIdConversion;
        let node = &primitives::china::china_ch::CHINA_CH[0];
        let pid = primitives::china::china_ch::shenfen_fee_id_to_bytes(node.shenfen_fee_id)
            .expect("fee id should be valid");
        PalletId(pid).into_account_truncating()
    }

    fn ordinary_account() -> AccountId {
        AccountId::new([99u8; 32])
    }

    #[test]
    fn keyless_account_is_completely_blocked() {
        let account = keyless_account();
        assert!(!RuntimeInstitutionAssetGuard::can_spend(
            &account,
            InstitutionAssetAction::DuoqianTransferExecute
        ));
        assert!(!RuntimeInstitutionAssetGuard::can_spend(
            &account,
            InstitutionAssetAction::DuoqianCloseExecute
        ));
        assert!(!RuntimeInstitutionAssetGuard::can_spend(
            &account,
            InstitutionAssetAction::OffchainBatchDebit
        ));
        assert!(!RuntimeInstitutionAssetGuard::can_spend(
            &account,
            InstitutionAssetAction::OffchainFeeSweepExecute
        ));
    }

    #[test]
    fn reserved_duoqian_only_allows_transfer_and_close() {
        let account = reserved_duoqian_account();
        assert!(RuntimeInstitutionAssetGuard::can_spend(
            &account,
            InstitutionAssetAction::DuoqianTransferExecute
        ));
        assert!(RuntimeInstitutionAssetGuard::can_spend(
            &account,
            InstitutionAssetAction::DuoqianCloseExecute
        ));
        assert!(!RuntimeInstitutionAssetGuard::can_spend(
            &account,
            InstitutionAssetAction::OffchainBatchDebit
        ));
        assert!(!RuntimeInstitutionAssetGuard::can_spend(
            &account,
            InstitutionAssetAction::OffchainFeeSweepExecute
        ));
    }

    #[test]
    fn reserved_fee_account_only_allows_fee_sweep() {
        let account = reserved_fee_account();
        assert!(!RuntimeInstitutionAssetGuard::can_spend(
            &account,
            InstitutionAssetAction::DuoqianTransferExecute
        ));
        assert!(!RuntimeInstitutionAssetGuard::can_spend(
            &account,
            InstitutionAssetAction::DuoqianCloseExecute
        ));
        assert!(!RuntimeInstitutionAssetGuard::can_spend(
            &account,
            InstitutionAssetAction::OffchainBatchDebit
        ));
        assert!(RuntimeInstitutionAssetGuard::can_spend(
            &account,
            InstitutionAssetAction::OffchainFeeSweepExecute
        ));
    }

    #[test]
    fn ordinary_account_allows_all_actions() {
        let account = ordinary_account();
        assert!(RuntimeInstitutionAssetGuard::can_spend(
            &account,
            InstitutionAssetAction::DuoqianTransferExecute
        ));
        assert!(RuntimeInstitutionAssetGuard::can_spend(
            &account,
            InstitutionAssetAction::DuoqianCloseExecute
        ));
        assert!(RuntimeInstitutionAssetGuard::can_spend(
            &account,
            InstitutionAssetAction::OffchainBatchDebit
        ));
        assert!(RuntimeInstitutionAssetGuard::can_spend(
            &account,
            InstitutionAssetAction::OffchainFeeSweepExecute
        ));
    }
}
