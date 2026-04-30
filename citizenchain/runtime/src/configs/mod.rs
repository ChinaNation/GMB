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
use alloc::vec::Vec;
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
    BoundedVec,
};
use frame_system::limits::{BlockLength, BlockWeights};
use onchain_transaction::NrcAccountProvider as _;
use pallet_transaction_payment::{ConstFeeMultiplier, Multiplier};
use sp_core::{sr25519, Void};
use sp_io::{crypto::sr25519_verify, hashing::blake2_256};
#[allow(unused_imports)]
use sp_runtime::traits::Hash as _;
use sp_runtime::{traits::One, Perbill};
use sp_version::RuntimeVersion;

// Local module imports
#[cfg(not(feature = "runtime-benchmarks"))]
use super::RuntimeUpgrade;
use super::{
    AccountId, Address, Balance, Balances, Block, BlockNumber, CitizenIssuance, GenesisPallet,
    Hash, Nonce, PalletInfo, ResolutionIssuance, Runtime, RuntimeCall, RuntimeEvent,
    RuntimeFreezeReason, RuntimeHoldReason, RuntimeOrigin, RuntimeTask, System, VotingEngine,
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

pub fn is_stake_account(address: &AccountId) -> bool {
    primitives::china::china_ch::CHINA_CH
        .iter()
        .any(|n| address == &AccountId::new(n.stake_address))
}

fn is_reserved_fee_account(address: &AccountId) -> bool {
    primitives::china::china_ch::CHINA_CH
        .iter()
        .any(|n| address == &AccountId::new(n.fee_address))
}

/// 检查是否为国储会安全基金账户。
fn is_nrc_anquan_account(address: &AccountId) -> bool {
    address == &AccountId::new(primitives::china::china_cb::NRC_ANQUAN_ADDRESS)
}

/// 检查是否为储委会费用账户（44 个机构的 fee_address）。
fn is_cb_fee_account(address: &AccountId) -> bool {
    primitives::china::china_cb::CHINA_CB
        .iter()
        .any(|n| address == &AccountId::new(n.fee_address))
}

fn is_reserved_main_account(address: &AccountId) -> bool {
    let raw: &[u8] = address.as_ref();
    if raw.len() != 32 {
        return false;
    }
    let mut addr = [0u8; 32];
    addr.copy_from_slice(raw);
    primitives::china::china_zb::is_reserved_main_address(&addr)
}

fn is_stake_multi_address(address: &Address) -> bool {
    match address {
        sp_runtime::MultiAddress::Id(account) => is_stake_account(account),
        sp_runtime::MultiAddress::Address32(raw) => is_stake_account(&AccountId::new(*raw)),
        sp_runtime::MultiAddress::Raw(raw) if raw.len() == 32 => {
            let mut out = [0u8; 32];
            out.copy_from_slice(raw.as_slice());
            is_stake_account(&AccountId::new(out))
        }
        _ => false,
    }
}

pub struct RuntimeCallFilter;

impl Contains<RuntimeCall> for RuntimeCallFilter {
    fn contains(call: &RuntimeCall) -> bool {
        match call {
            RuntimeCall::Balances(pallet_balances::Call::force_transfer { source, .. }) => {
                !is_stake_multi_address(source)
            }
            RuntimeCall::Balances(pallet_balances::Call::force_unreserve { who, .. }) => {
                !is_stake_multi_address(who)
            }
            RuntimeCall::Balances(pallet_balances::Call::force_set_balance { who, .. }) => {
                !is_stake_multi_address(who)
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
    /// 中文注释：全局调用过滤器，禁止 stake_address 参与 force_* 余额调用，并封禁强制总发行量调整入口。
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
    type OnChargeTransaction = onchain_transaction::OnchainChargeAdapter<
        Balances,
        onchain_transaction::OnchainFeeRouter<
            Runtime,
            Balances,
            PowDigestAuthor,
            RuntimeNrcAccountProvider,
        >,
        OnchainTxAmountExtractor,
        RuntimeFeePayerExtractor,
    >;
    type OperationalFeeMultiplier = ConstU8<{ primitives::core_const::OPERATIONAL_FEE_MULTIPLIER }>;
    type WeightToFee = ConstantMultiplier<Balance, ConstU128<0>>;
    type LengthToFee = ConstantMultiplier<Balance, ConstU128<0>>;
    type FeeMultiplierUpdate = ConstFeeMultiplier<FeeMultiplier>;
    type WeightInfo = pallet_transaction_payment::weights::SubstrateWeight<Runtime>;
}

impl onchain_transaction::pallet::Config for Runtime {}

pub struct RuntimeNrcAccountProvider;

impl onchain_transaction::NrcAccountProvider<AccountId> for RuntimeNrcAccountProvider {
    fn nrc_account() -> Option<AccountId> {
        Some(AccountId::new(
            primitives::china::china_cb::CHINA_CB[0].fee_address,
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

pub struct OnchainTxAmountExtractor;

impl onchain_transaction::CallAmount<AccountId, RuntimeCall, Balance> for OnchainTxAmountExtractor {
    fn amount(
        who: &AccountId,
        call: &RuntimeCall,
    ) -> onchain_transaction::AmountExtractResult<Balance> {
        match call {
            RuntimeCall::Balances(pallet_balances::Call::transfer_allow_death {
                value, ..
            }) => onchain_transaction::AmountExtractResult::Amount(*value),
            RuntimeCall::Balances(pallet_balances::Call::transfer_keep_alive { value, .. }) => {
                onchain_transaction::AmountExtractResult::Amount(*value)
            }
            RuntimeCall::Balances(pallet_balances::Call::force_transfer { value, .. }) => {
                onchain_transaction::AmountExtractResult::Amount(*value)
            }
            RuntimeCall::Balances(pallet_balances::Call::force_unreserve { amount, .. }) => {
                onchain_transaction::AmountExtractResult::Amount(*amount)
            }
            RuntimeCall::Balances(pallet_balances::Call::force_set_balance {
                new_free, ..
            }) => onchain_transaction::AmountExtractResult::Amount(*new_free),
            RuntimeCall::Balances(pallet_balances::Call::force_adjust_total_issuance {
                delta,
                ..
            }) => onchain_transaction::AmountExtractResult::Amount(*delta),
            RuntimeCall::Balances(pallet_balances::Call::burn { value, .. }) => {
                onchain_transaction::AmountExtractResult::Amount(*value)
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
                onchain_transaction::AmountExtractResult::Amount(value)
            }
            RuntimeCall::DuoqianManage(duoqian_manage::pallet::Call::propose_create {
                amount,
                ..
            }) => onchain_transaction::AmountExtractResult::Amount(*amount),
            RuntimeCall::DuoqianManage(duoqian_manage::pallet::Call::propose_create_personal {
                amount,
                ..
            }) => onchain_transaction::AmountExtractResult::Amount(*amount),
            RuntimeCall::DuoqianManage(
                duoqian_manage::pallet::Call::propose_create_institution { accounts, .. },
            ) => {
                let mut total: Balance = 0;
                for account in accounts.iter() {
                    let Some(next) = total.checked_add(account.amount) else {
                        return onchain_transaction::AmountExtractResult::Amount(100_000);
                    };
                    total = next;
                }
                onchain_transaction::AmountExtractResult::Amount(total)
            }
            RuntimeCall::DuoqianManage(duoqian_manage::pallet::Call::propose_close {
                duoqian_address,
                ..
            }) => onchain_transaction::AmountExtractResult::Amount(Balances::free_balance(
                duoqian_address,
            )),
            // 免费调用交易：SFID 注册是证明型操作
            RuntimeCall::DuoqianManage(
                duoqian_manage::pallet::Call::register_sfid_institution { .. },
            ) => onchain_transaction::AmountExtractResult::NoAmount,
            // 付费调用交易：多签管理其他操作（propose_create_personal、cleanup 等）
            RuntimeCall::DuoqianManage(_) => {
                onchain_transaction::AmountExtractResult::Amount(100000)
            }
            // 免费调用交易：系统内部调用
            RuntimeCall::System(_) => onchain_transaction::AmountExtractResult::NoAmount,
            RuntimeCall::Timestamp(_) => onchain_transaction::AmountExtractResult::NoAmount,
            RuntimeCall::ShengBankInterest(_) => onchain_transaction::AmountExtractResult::NoAmount,
            // 付费调用交易：治理/用户操作（1 元/次）
            RuntimeCall::ResolutionIssuance(ref ri_call) => {
                match ri_call {
                    // 免费：治理权限终结投票 + 设置收款白名单
                    resolution_issuance::pallet::Call::finalize_joint_vote { .. }
                    | resolution_issuance::pallet::Call::set_allowed_recipients { .. }
                    | resolution_issuance::pallet::Call::clear_executed { .. }
                    | resolution_issuance::pallet::Call::set_paused { .. } => {
                        onchain_transaction::AmountExtractResult::NoAmount
                    }
                    // 付费：管理员主动发起增发提案
                    _ => onchain_transaction::AmountExtractResult::Amount(100000),
                }
            }
            // 投票引擎:拆分"内部投票"(免费,鼓励管理员踊跃履职)与其他调用(付费)。
            // Phase 2 后公开 call 只剩 4 个:internal_vote / joint_vote / citizen_vote / finalize_proposal。
            RuntimeCall::VotingEngine(ref ve_call) => {
                match ve_call {
                    // 免费:管理员内部投票(最高频路径,0 gas 降门槛)
                    voting_engine::pallet::Call::internal_vote { .. } => {
                        onchain_transaction::AmountExtractResult::NoAmount
                    }
                    // 免费:终结已完成提案(任意人都可调,推动清理)
                    voting_engine::pallet::Call::finalize_proposal { .. } => {
                        onchain_transaction::AmountExtractResult::NoAmount
                    }
                    // 付费:用户主动参与联合/公民投票(1 元/次)
                    _ => onchain_transaction::AmountExtractResult::Amount(100000),
                }
            }
            RuntimeCall::SfidSystem(_) => onchain_transaction::AmountExtractResult::Amount(100000),
            RuntimeCall::CitizenIssuance(_) => onchain_transaction::AmountExtractResult::NoAmount,
            RuntimeCall::FullnodeIssuance(_) => {
                onchain_transaction::AmountExtractResult::Amount(100000)
            }
            RuntimeCall::AdminsChange(ref ag_call) => {
                match ag_call {
                    // 免费：触发已通过提案的执行
                    admins_change::pallet::Call::execute_admin_replacement { .. } => {
                        onchain_transaction::AmountExtractResult::NoAmount
                    }
                    // 付费：管理员主动提案/投票
                    _ => onchain_transaction::AmountExtractResult::Amount(100000),
                }
            }
            RuntimeCall::RuntimeUpgrade(ref ru_call) => {
                match ru_call {
                    // 免费：Root 权限终结联合投票
                    runtime_upgrade::pallet::Call::finalize_joint_vote { .. } => {
                        onchain_transaction::AmountExtractResult::NoAmount
                    }
                    // 付费：管理员主动提案/开发升级
                    _ => onchain_transaction::AmountExtractResult::Amount(100000),
                }
            }
            RuntimeCall::ResolutionDestro(ref rd_call) => {
                match rd_call {
                    // 免费：触发已通过提案的执行
                    resolution_destro::pallet::Call::execute_destroy { .. } => {
                        onchain_transaction::AmountExtractResult::NoAmount
                    }
                    // 付费：管理员主动提案/投票
                    _ => onchain_transaction::AmountExtractResult::Amount(100000),
                }
            }
            RuntimeCall::GrandpaKeyChange(ref gk_call) => {
                match gk_call {
                    // 免费：触发已通过提案的执行 + 取消失败变更
                    grandpakey_change::pallet::Call::execute_replace_grandpa_key { .. }
                    | grandpakey_change::pallet::Call::cancel_failed_replace_grandpa_key {
                        ..
                    } => onchain_transaction::AmountExtractResult::NoAmount,
                    // 付费：管理员主动提案/投票
                    _ => onchain_transaction::AmountExtractResult::Amount(100000),
                }
            }
            // 机构转账模块：拆分执行（免费）和提案/投票（付费）
            RuntimeCall::DuoqianTransfer(ref dt_call) => {
                match dt_call {
                    // 免费：触发已通过提案的执行（手续费在执行时从机构内部扣）
                    duoqian_transfer::pallet::Call::execute_transfer { .. } => {
                        onchain_transaction::AmountExtractResult::NoAmount
                    }
                    // 付费：管理员主动提案/投票（1 元/次）
                    _ => onchain_transaction::AmountExtractResult::Amount(100000),
                }
            }
            // 清算行(L2)扫码支付清算:Step 2b-iv-b 清理后只剩新体系 Call。
            RuntimeCall::OffchainTransaction(ref offchain_call) => {
                match offchain_call {
                    // L3 充值 / 提现:按金额计费(链上资金交易 0.1% 最低 0.1 元)
                    offchain_transaction::pallet::Call::deposit { amount } => {
                        onchain_transaction::AmountExtractResult::Amount(*amount)
                    }
                    offchain_transaction::pallet::Call::withdraw { amount } => {
                        onchain_transaction::AmountExtractResult::Amount(*amount)
                    }
                    // 清算行批次 V2 上链:按 sum(fee_amount) 计费(链下资金交易)
                    offchain_transaction::pallet::Call::submit_offchain_batch_v2 {
                        batch, ..
                    } => {
                        let mut total_fee: u128 = 0;
                        for item in batch.iter() {
                            total_fee = total_fee.saturating_add(item.fee_amount);
                        }
                        onchain_transaction::AmountExtractResult::Amount(total_fee)
                    }
                    // 全局费率上限调整(Root Origin,免费)
                    offchain_transaction::pallet::Call::set_max_l2_fee_rate { .. } => {
                        onchain_transaction::AmountExtractResult::NoAmount
                    }
                    // 其他付费调用(bind_clearing_bank / switch_bank / propose_l2_fee_rate):
                    // 固定 1 元/次
                    _ => onchain_transaction::AmountExtractResult::Amount(100000),
                }
            }
            // 中文注释：对 Balances 未覆盖分支按 Unknown 拒绝，避免”有金额但漏提取”。
            RuntimeCall::Balances(_) => onchain_transaction::AmountExtractResult::Unknown,
            _ => onchain_transaction::AmountExtractResult::Unknown,
        }
    }
}

pub struct RuntimeFeePayerExtractor;

impl onchain_transaction::CallFeePayer<AccountId, RuntimeCall> for RuntimeFeePayerExtractor {
    fn fee_payer(_who: &AccountId, call: &RuntimeCall) -> Option<AccountId> {
        match call {
            // 清算行 V2 批次:链上 gas 由 institution_main 的费用账户直接承担。
            //
            // Step 2(2026-04-27, ADR-007)修订:**收款方主导清算**模型下,
            // institution_main 现在 = 收款方清算行主账户。fee_account_of(institution_main)
            // = 收款方清算行费用账户 = 同一账户既收清算手续费又付链上 gas,自给自足闭环。
            //
            // 提交者(origin)是该机构的某个激活管理员(已在节点端解密私钥,自动签),
            // 但其个人钱包余额不参与 gas 扣费。
            RuntimeCall::OffchainTransaction(
                offchain_transaction::pallet::Call::submit_offchain_batch_v2 {
                    institution_main,
                    ..
                },
            ) => offchain_transaction::Pallet::<Runtime>::fee_account_of(institution_main).ok(),
            // 其他 offchain Call 及其他 RuntimeCall 由调用者个人账户付费。
            _ => None,
        }
    }
}

/// 省储行利息模块配置：
/// - 使用 Balances 作为铸币/记账货币
/// - 每年区块数统一采用 primitives 中的制度常量
impl shengbank_interest::Config for Runtime {
    type Currency = Balances;
    type BlocksPerYear = ConstU64<{ primitives::pow_const::BLOCKS_PER_YEAR }>;
    type WeightInfo = shengbank_interest::weights::SubstrateWeight<Runtime>;
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

/// 全节点发行模块配置：
/// - 链上货币使用 Balances
/// - 作者识别完全基于 PoW digest（不依赖 Aura/Grandpa）
impl fullnode_issuance::Config for Runtime {
    type Currency = Balances;
    type FindAuthor = PowDigestAuthor;
    type WeightInfo = fullnode_issuance::weights::SubstrateWeight<Runtime>;
}

pub struct RuntimeDuoqianAddressValidator;

impl duoqian_manage::DuoqianAddressValidator<AccountId> for RuntimeDuoqianAddressValidator {
    fn is_valid(address: &AccountId) -> bool {
        // 中文注释：禁止零地址。
        if address == &AccountId::new([0u8; 32]) {
            return false;
        }

        // 中文注释：禁止占用“国储会/省储会”的制度保留交易地址。
        if primitives::china::china_cb::CHINA_CB
            .iter()
            .any(|n| address == &AccountId::new(n.main_address))
        {
            return false;
        }

        // 中文注释：禁止占用“省储行”的制度保留交易地址。
        if primitives::china::china_ch::CHINA_CH
            .iter()
            .any(|n| address == &AccountId::new(n.main_address))
        {
            return false;
        }

        // 中文注释：禁止占用”省储行费用账户”地址（BLAKE2-256 派生）。
        if primitives::china::china_ch::CHINA_CH
            .iter()
            .any(|n| address == &AccountId::new(n.fee_address))
        {
            return false;
        }

        // 中文注释：禁止占用国储会安全基金账户地址。
        if is_nrc_anquan_account(address) {
            return false;
        }

        // 中文注释：禁止占用储委会费用账户地址（44 个机构）。
        if is_cb_fee_account(address) {
            return false;
        }

        true
    }
}

pub struct RuntimeDuoqianReservedAddressChecker;
pub struct RuntimeSfidInstitutionVerifier;

pub struct RuntimeProtectedSourceChecker;
pub struct RuntimeInstitutionAsset;

impl duoqian_manage::ProtectedSourceChecker<AccountId> for RuntimeProtectedSourceChecker {
    fn is_protected(address: &AccountId) -> bool {
        is_stake_account(address)
    }
}

impl institution_asset::InstitutionAsset<AccountId> for RuntimeInstitutionAsset {
    fn can_spend(source: &AccountId, action: institution_asset::InstitutionAssetAction) -> bool {
        // 中文注释：匹配顺序很重要——更具体的账户类型必须放在更宽泛的类型之前。
        // fee_address 同时出现在 CHINA_RESERVED_MAIN_ADDRESSES 列表中（同由 BLAKE2 派生且统一保留），
        // 如果 is_reserved_main_account 先匹配，fee_address 会被错误地按主账户规则放行。

        // 1. 无私钥系统账户：全禁
        if is_stake_account(source) {
            return false;
        }

        // 2. 省储行费用账户（最具体）：只允许手续费归集
        if is_reserved_fee_account(source) {
            return matches!(
                action,
                institution_asset::InstitutionAssetAction::OffchainFeeSweepExecute
            );
        }

        // 3. 储委会费用账户（44 个机构）：只允许手续费归集
        if is_cb_fee_account(source) {
            return matches!(
                action,
                institution_asset::InstitutionAssetAction::OffchainFeeSweepExecute
            );
        }

        // 4. 国储会安全基金账户：只允许安全基金转账
        if source == &AccountId::new(primitives::china::china_cb::NRC_ANQUAN_ADDRESS) {
            return matches!(
                action,
                institution_asset::InstitutionAssetAction::NrcSafetyFundTransfer
            );
        }

        // 5. 多签保留地址（范围最宽）：只允许多签转账和关闭
        if is_reserved_main_account(source) {
            return matches!(
                action,
                institution_asset::InstitutionAssetAction::DuoqianTransferExecute
                    | institution_asset::InstitutionAssetAction::DuoqianCloseExecute
            );
        }

        // 6. 普通账户：全放行
        true
    }
}

impl duoqian_manage::DuoqianReservedAddressChecker<AccountId>
    for RuntimeDuoqianReservedAddressChecker
{
    fn is_reserved(address: &AccountId) -> bool {
        // 中文注释：禁止占用省储行 stake_address（制度保留地址）。
        if primitives::china::china_ch::CHINA_CH
            .iter()
            .any(|n| address == &AccountId::new(n.stake_address))
        {
            return true;
        }

        // 中文注释：禁止占用省储行费用账户地址（BLAKE2-256 派生）。
        if primitives::china::china_ch::CHINA_CH
            .iter()
            .any(|n| address == &AccountId::new(n.fee_address))
        {
            return true;
        }

        // 中文注释：禁止占用国储会安全基金账户地址。
        if is_nrc_anquan_account(address) {
            return true;
        }

        // 中文注释：禁止占用储委会费用账户地址（44 个机构）。
        if is_cb_fee_account(address) {
            return true;
        }

        is_reserved_main_account(address)
    }
}

impl
    duoqian_manage::SfidInstitutionVerifier<
        duoqian_manage::pallet::AccountNameOf<Runtime>,
        duoqian_manage::pallet::RegisterNonceOf<Runtime>,
        duoqian_manage::pallet::RegisterSignatureOf<Runtime>,
    > for RuntimeSfidInstitutionVerifier
{
    fn verify_institution_registration(
        sfid_id: &[u8],
        account_name: &duoqian_manage::pallet::AccountNameOf<Runtime>,
        nonce: &duoqian_manage::pallet::RegisterNonceOf<Runtime>,
        signature: &duoqian_manage::pallet::RegisterSignatureOf<Runtime>,
        signing_province: Option<&[u8]>,
    ) -> bool {
        #[cfg(feature = "runtime-benchmarks")]
        {
            let _ = signing_province;
            return !sfid_id.is_empty()
                && !account_name.is_empty()
                && !nonce.is_empty()
                && !signature.is_empty();
        }

        #[cfg(not(feature = "runtime-benchmarks"))]
        {
            // 中文注释：按 signing_province 分流：
            //   Some(p) → 查 ShengSigningPubkey[p]，省签名公钥验签；
            //   None    → fallback 用 SfidMainAccount 当前主公钥验签。
            let public_bytes: [u8; 32] = match signing_province {
                Some(p) => match sfid_system::Pallet::<Runtime>::sheng_signing_pubkey(p) {
                    Some(k) => k,
                    None => return false,
                },
                None => match current_sfid_verify_public() {
                    Some(k) => k.0,
                    None => return false,
                },
            };
            let public = sr25519::Public::from_raw(public_bytes);

            let sig_bytes = signature.as_slice();
            if sig_bytes.len() != 64 {
                return false;
            }

            let mut sig_raw = [0u8; 64];
            sig_raw.copy_from_slice(sig_bytes);
            let signature = sr25519::Signature::from_raw(sig_raw);

            // 中文注释：统一 domain 走 DUOQIAN_DOMAIN + OP_SIGN_INST；signing_province 存在时追加到末尾防跨省 replay。
            let msg = match signing_province {
                Some(p) => {
                    let payload = (
                        primitives::core_const::DUOQIAN_DOMAIN,
                        primitives::core_const::OP_SIGN_INST,
                        frame_system::Pallet::<Runtime>::block_hash(0),
                        sfid_id,
                        account_name.as_slice(),
                        nonce.as_slice(),
                        p,
                    );
                    blake2_256(&payload.encode())
                }
                None => {
                    let payload = (
                        primitives::core_const::DUOQIAN_DOMAIN,
                        primitives::core_const::OP_SIGN_INST,
                        frame_system::Pallet::<Runtime>::block_hash(0),
                        sfid_id,
                        account_name.as_slice(),
                        nonce.as_slice(),
                    );
                    blake2_256(&payload.encode())
                }
            };

            sr25519_verify(&signature, &msg, &public)
        }
    }
}

impl duoqian_manage::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type InternalVoteEngine = voting_engine::Pallet<Runtime>;
    type AddressValidator = RuntimeDuoqianAddressValidator;
    type ReservedAddressChecker = RuntimeDuoqianReservedAddressChecker;
    type ProtectedSourceChecker = RuntimeProtectedSourceChecker;
    type InstitutionAsset = RuntimeInstitutionAsset;
    type SfidInstitutionVerifier = RuntimeSfidInstitutionVerifier;
    type FeeRouter = TransferFeeRouter;
    type MaxAdmins = ConstU32<64>;
    type MaxSfidIdLength = ConstU32<96>;
    type MaxAccountNameLength = ConstU32<128>;
    type MaxRegisterNonceLength = ConstU32<64>;
    type MaxRegisterSignatureLength = ConstU32<64>;
    // Step 2(2026-04-27, ADR-007)新增:机构元数据字段长度上限。
    // a3 为 SFR/FFR/GFR/SF 等三字符标识,8 字节足够。
    type MaxA3Length = ConstU32<8>;
    // sub_type 为 JOINT_STOCK / LIMITED_LIABILITY / NON_PROFIT 等枚举字符串。
    type MaxSubTypeLength = ConstU32<32>;
    // sr25519 签名固定 64 字节。
    // Phase 2 整改后聚合签名 `finalize_create` 已删除,此类型仍保留为 `AdminSignatureOf`
    // 的容量配置,供未来业务扩展(如链下审计签名附件)使用。
    type MaxAdminSignatureLength = ConstU32<64>;
    type MaxInstitutionAccounts = ConstU32<16>;
    type MinCreateAmount = ConstU128<111>;
    type MinCloseBalance = ConstU128<121>;
    type WeightInfo = duoqian_manage::weights::SubstrateWeight<Runtime>;
}

fn current_sfid_verify_public() -> Option<sr25519::Public> {
    let key = sfid_system::Pallet::<Runtime>::current_sfid_verify_pubkey()?;
    Some(sr25519::Public::from_raw(key))
}

pub struct RuntimeSfidVerifier;

impl
    sfid_system::SfidVerifier<
        AccountId,
        Hash,
        sfid_system::pallet::NonceOf<Runtime>,
        sfid_system::pallet::SignatureOf<Runtime>,
    > for RuntimeSfidVerifier
{
    fn verify(
        account: &AccountId,
        credential: &sfid_system::pallet::CredentialOf<Runtime>,
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
            primitives::core_const::DUOQIAN_DOMAIN,
            primitives::core_const::OP_SIGN_BIND,
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
    sfid_system::SfidVoteVerifier<
        AccountId,
        Hash,
        sfid_system::pallet::NonceOf<Runtime>,
        sfid_system::pallet::SignatureOf<Runtime>,
    > for RuntimeSfidVoteVerifier
{
    fn verify_vote(
        account: &AccountId,
        binding_id: Hash,
        proposal_id: u64,
        nonce: &sfid_system::pallet::NonceOf<Runtime>,
        signature: &sfid_system::pallet::SignatureOf<Runtime>,
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
                primitives::core_const::DUOQIAN_DOMAIN,
                primitives::core_const::OP_SIGN_VOTE,
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

impl sfid_system::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxCredentialNonceLength = ConstU32<64>;
    // 中文注释：SFID 绑定与投票验签统一使用 64 字节原始 sr25519 签名。
    type MaxCredentialSignatureLength = ConstU32<64>;
    type SfidVerifier = RuntimeSfidVerifier;
    type SfidVoteVerifier = RuntimeSfidVoteVerifier;
    type OnSfidBound = CitizenIssuance;
    type WeightInfo = sfid_system::weights::SubstrateWeight<Runtime>;
}

pub struct RuntimePopulationSnapshotVerifier;

impl
    voting_engine::PopulationSnapshotVerifier<
        AccountId,
        voting_engine::pallet::VoteNonceOf<Runtime>,
        voting_engine::pallet::VoteSignatureOf<Runtime>,
    > for RuntimePopulationSnapshotVerifier
{
    fn verify_population_snapshot(
        who: &AccountId,
        eligible_total: u64,
        nonce: &voting_engine::pallet::VoteNonceOf<Runtime>,
        signature: &voting_engine::pallet::VoteSignatureOf<Runtime>,
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
                primitives::core_const::DUOQIAN_DOMAIN,
                primitives::core_const::OP_SIGN_POP,
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

impl citizen_issuance::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type WeightInfo = citizen_issuance::weights::SubstrateWeight<Runtime>;
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
    // 必须 >= admins_change::MaxAdminsPerInstitution (32)
    // 且 >= duoqian_manage::MaxAdmins (64)，否则快照写入会静默失败。
    pub const MaxAdminsPerInstitution: u32 = 64;
    /// GRANDPA authority set 变更生效延迟（单位：区块）。
    /// 取非 0，给运维注入新 gran 私钥预留窗口，避免立即切换导致短时失票。
    pub const GrandpaAuthoritySetChangeDelay: u32 = 30;
}

impl admins_change::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxAdminsPerInstitution = MaxAdminsPerInstitution;
    type InternalVoteEngine = VotingEngine;
    type WeightInfo = admins_change::weights::SubstrateWeight<Runtime>;
}

impl resolution_destro::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type InternalVoteEngine = VotingEngine;
    type WeightInfo = resolution_destro::weights::SubstrateWeight<Runtime>;
}

impl grandpakey_change::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type GrandpaChangeDelay = GrandpaAuthoritySetChangeDelay;
    type InternalVoteEngine = VotingEngine;
    type WeightInfo = grandpakey_change::weights::SubstrateWeight<Runtime>;
}

/// 转账提案手续费分账适配器：将旧 Currency NegativeImbalance 转换后
/// 交给现有 OnchainFeeRouter 处理（80% 全节点 / 10% 国储会 / 10% 安全基金）。
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

        type FeeRouter = onchain_transaction::OnchainFeeRouter<
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

impl duoqian_transfer::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxRemarkLen = ConstU32<256>;
    type FeeRouter = TransferFeeRouter;
    type WeightInfo = duoqian_transfer::weights::SubstrateWeight<Runtime>;
}

// ---------------------------------------------------------------------------
// 链下交易清算模块配置
// ---------------------------------------------------------------------------

/// 扫码支付 Step 1 新增:SFID 机构登记表查询实现。
///
/// 委托给 `duoqian-manage` 的 SFID 地址索引和机构账户表；
/// 管理员校验再统一转给 `admins-change::Institutions`。
pub struct DuoqianSfidAccountQuery;

impl offchain_transaction::bank_check::SfidAccountQuery<AccountId> for DuoqianSfidAccountQuery {
    fn account_info(addr: &AccountId) -> Option<(Vec<u8>, Vec<u8>)> {
        duoqian_manage::AddressRegisteredSfid::<Runtime>::get(addr)
            .map(|info| (info.sfid_id.to_vec(), info.account_name.to_vec()))
    }

    fn find_address(sfid_id: &[u8], account_name: &[u8]) -> Option<AccountId> {
        let id: duoqian_manage::SfidIdOf<Runtime> = sfid_id.to_vec().try_into().ok()?;
        let an: duoqian_manage::AccountNameOf<Runtime> = account_name.to_vec().try_into().ok()?;
        duoqian_manage::SfidRegisteredAddress::<Runtime>::get(&id, &an)
    }

    fn is_active(addr: &AccountId) -> bool {
        if let Some(registered) = duoqian_manage::AddressRegisteredSfid::<Runtime>::get(addr) {
            return matches!(
                duoqian_manage::InstitutionAccounts::<Runtime>::get(
                    &registered.sfid_id,
                    &registered.account_name,
                )
                .map(|a| a.status),
                Some(duoqian_manage::InstitutionLifecycleStatus::Active)
            );
        }

        matches!(
            duoqian_manage::DuoqianAccounts::<Runtime>::get(addr).map(|a| a.status),
            Some(duoqian_manage::DuoqianStatus::Active)
        )
    }

    /// 扫码支付 Step 2 新增:判定 `who` 是否是 `bank` 多签账户的管理员之一。
    /// 用于费率提案 / 批次提交等治理动作的身份校验。
    fn is_admin_of(bank: &AccountId, who: &AccountId) -> bool {
        let Some(subject_id) =
            duoqian_manage::Pallet::<Runtime>::resolve_admin_subject_for_account(bank)
        else {
            return false;
        };
        admins_change::Pallet::<Runtime>::is_active_subject_admin(
            voting_engine::internal_vote::ORG_DUOQIAN,
            subject_id,
            who,
        )
    }

    /// Step 2(2026-04-27, ADR-007)新增:清算行资格白名单判定。
    ///
    /// 委托查 `InstitutionMetadata` storage:
    /// - SFR + sub_type=JOINT_STOCK            → ✅
    /// - FFR + parent.SFR + parent.JOINT_STOCK → ✅(parent 元数据另查)
    /// - 其他                                   → ❌
    fn is_clearing_bank_eligible(addr: &AccountId) -> bool {
        // 1. 反查地址所属的 sfid_id
        let registered = match duoqian_manage::AddressRegisteredSfid::<Runtime>::get(addr) {
            Some(info) => info,
            None => return false,
        };
        // 2. 查机构元数据
        let meta = match duoqian_manage::InstitutionMetadata::<Runtime>::get(&registered.sfid_id) {
            Some(m) => m,
            None => return false,
        };
        match meta.a3.as_slice() {
            b"SFR" => meta.sub_type.as_ref().map(|s| s.as_slice()) == Some(&b"JOINT_STOCK"[..]),
            b"FFR" => {
                let parent_id = match meta.parent_sfid_id.as_ref() {
                    Some(p) => p,
                    None => return false,
                };
                let parent_meta =
                    match duoqian_manage::InstitutionMetadata::<Runtime>::get(parent_id) {
                        Some(m) => m,
                        None => return false,
                    };
                parent_meta.a3.as_slice() == b"SFR"
                    && parent_meta.sub_type.as_ref().map(|s| s.as_slice())
                        == Some(&b"JOINT_STOCK"[..])
            }
            _ => false,
        }
    }

    /// Step 2(2026-04-27, ADR-007)新增:判定 `bank` 主账户对应的机构是否
    /// 已声明为清算行节点(链上 `ClearingBankNodes` 存在该 sfid_id 记录)。
    fn is_registered_clearing_node(bank: &AccountId) -> bool {
        let registered = match duoqian_manage::AddressRegisteredSfid::<Runtime>::get(bank) {
            Some(info) => info,
            None => return false,
        };
        // ClearingBankNodes 的 key 是 BoundedVec<u8, ConstU32<64>>,
        // 把 SfidIdOf<Runtime>(BoundedVec<u8, MaxSfidIdLength=96>) 转换过去
        let sfid_bytes: Vec<u8> = registered.sfid_id.to_vec();
        let key: BoundedVec<u8, ConstU32<64>> = match sfid_bytes.try_into() {
            Ok(b) => b,
            Err(_) => return false,
        };
        offchain_transaction::pallet::ClearingBankNodes::<Runtime>::contains_key(&key)
    }
}

impl offchain_transaction::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type MaxBatchSize = ConstU32<100_000>;
    type MaxBatchSignatureLength = ConstU32<128>;
    type InstitutionAsset = RuntimeInstitutionAsset;
    type SfidAccountQuery = DuoqianSfidAccountQuery;
    type WeightInfo = offchain_transaction::weights::SubstrateWeight<Runtime>;
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

    // 中文注释：创世后只信任链上管理员治理模块中的统一主体表。
    admins_change::Pallet::<Runtime>::is_active_subject_admin(
        voting_engine::internal_vote::ORG_NRC,
        nrc_institution,
        who,
    )
}

/// 联合提案发起权限：国储会（CHINA_CB[0]）+ 43个省储会（CHINA_CB[1..44]）。
pub struct EnsureJointProposer;

impl EnsureOrigin<RuntimeOrigin> for EnsureJointProposer {
    type Success = AccountId;

    fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
        let who = frame_system::EnsureSigned::<AccountId>::try_origin(o)?;
        if is_joint_proposer(&who) {
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

/// 国储会和省储会管理员均可发起联合提案（含运行时升级、决议发行等）。
fn is_joint_proposer(who: &AccountId) -> bool {
    use primitives::china::china_cb::{shenfen_id_to_fixed48, CHINA_CB};
    let nrc_institution = CHINA_CB
        .first()
        .and_then(|n| shenfen_id_to_fixed48(n.shenfen_id));
    for entry in CHINA_CB.iter() {
        if let Some(institution) = shenfen_id_to_fixed48(entry.shenfen_id) {
            let org = if Some(institution) == nrc_institution {
                voting_engine::internal_vote::ORG_NRC
            } else {
                voting_engine::internal_vote::ORG_PRC
            };
            if admins_change::Pallet::<Runtime>::is_active_subject_admin(org, institution, who) {
                return true;
            }
        }
    }
    false
}

impl resolution_issuance::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type ProposeOrigin = EnsureJointProposer;
    type RecipientSetOrigin = frame_system::EnsureRoot<AccountId>;
    // 中文注释：禁用外部 finalize 入口，只允许投票引擎回调路径落地结果。
    type JointVoteFinalizeOrigin = EnsureJointVoteFinalizeOrigin;
    // 中文注释：维护入口只允许 root 操作暂停与短期执行记录清理。
    type MaintenanceOrigin = frame_system::EnsureRoot<AccountId>;
    type WeightInfo = resolution_issuance::weights::SubstrateWeight<Runtime>;
    type JointVoteEngine = VotingEngine;
    type MaxReasonLen = ResolutionIssuanceMaxReasonLen;
    type MaxAllocations = ResolutionIssuanceMaxAllocations;
    type MaxSnapshotNonceLength = ConstU32<64>;
    type MaxSnapshotSignatureLength = ConstU32<64>;
    type MaxTotalIssuance = ResolutionIssuanceMaxTotalIssuance;
    type MaxSingleIssuance = ResolutionIssuanceMaxSingleIssuance;
}

impl runtime_upgrade::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type ProposeOrigin = EnsureJointProposer;
    type JointVoteEngine = VotingEngine;
    type RuntimeCodeExecutor = RuntimeSetCodeExecutor;
    type DeveloperUpgradeCheck = GenesisPallet;
    type MaxReasonLen = RuntimeUpgradeMaxReasonLen;
    type MaxRuntimeCodeSize = RuntimeUpgradeMaxCodeSize;
    type MaxSnapshotNonceLength = ConstU32<64>;
    type MaxSnapshotSignatureLength = ConstU32<64>;
    type WeightInfo = runtime_upgrade::weights::SubstrateWeight<Runtime>;
}

pub struct RuntimeSetCodeExecutor;

impl runtime_upgrade::RuntimeCodeExecutor for RuntimeSetCodeExecutor {
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

impl voting_engine::JointVoteResultCallback for RuntimeJointVoteResultCallback {
    fn on_joint_vote_finalized(vote_proposal_id: u64, approved: bool) -> DispatchResult {
        #[cfg(feature = "runtime-benchmarks")]
        {
            let _ = (vote_proposal_id, approved);
            Ok(())
        }

        #[cfg(not(feature = "runtime-benchmarks"))]
        {
            if resolution_issuance::Pallet::<Runtime>::owns_proposal(vote_proposal_id) {
                return <ResolutionIssuance as voting_engine::JointVoteResultCallback>::on_joint_vote_finalized(
                vote_proposal_id,
                approved,
            );
            }

            if runtime_upgrade::Pallet::<Runtime>::owns_proposal(vote_proposal_id) {
                return <RuntimeUpgrade as voting_engine::JointVoteResultCallback>::on_joint_vote_finalized(
                    vote_proposal_id,
                    approved,
                );
            }

            Err(sp_runtime::DispatchError::Other(
                "joint vote proposal not found in any module",
            ))
        }
    }
}

impl voting_engine::Config for Runtime {
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
    // Phase 2:内部投票终态回调注册 5 个业务 Executor。
    // 顺序按调用频率降序:transfer / multisig manage 类业务最频繁,
    // grandpa key 替换最稀有放最后(tuple iterate 时命中越早越省 gas)。
    // 每个 Executor 通过 MODULE_TAG 前缀 + 独立存储键互斥认领本模块提案,
    // 非己方提案直接 Ok(()) skip,顺序不影响行为正确性。
    type InternalVoteResultCallback = (
        duoqian_transfer::InternalVoteExecutor<Runtime>,
        duoqian_manage::InternalVoteExecutor<Runtime>,
        admins_change::InternalVoteExecutor<Runtime>,
        resolution_destro::InternalVoteExecutor<Runtime>,
        grandpakey_change::InternalVoteExecutor<Runtime>,
    );
    type InternalAdminProvider = RuntimeInternalAdminProvider;
    type InternalAdminCountProvider = RuntimeInternalAdminCountProvider;
    type InternalThresholdProvider = RuntimeInternalThresholdProvider;
    type MaxAdminsPerInstitution = MaxAdminsPerInstitution;
    type TimeProvider = pallet_timestamp::Pallet<Runtime>;
    type WeightInfo = voting_engine::weights::SubstrateWeight<Runtime>;
}

impl pow_difficulty::Config for Runtime {
    type WeightInfo = pow_difficulty::weights::SubstrateWeight<Runtime>;
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
    use crate::ResolutionDestro;
    use duoqian_manage::DuoqianReservedAddressChecker;
    use frame_support::assert_ok;
    use frame_support::traits::Currency;
    use primitives::china::china_cb::{
        shenfen_id_to_fixed48 as reserve_pallet_id_to_bytes, CHINA_CB,
    };
    use sfid_system::{SfidVerifier, SfidVoteVerifier};
    use sp_core::Pair;
    use sp_runtime::{traits::Hash as HashT, traits::IdentifyAccount, BuildStorage, MultiSigner};
    use voting_engine::{
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
            let recipient = AccountId::new(primitives::china::china_cb::CHINA_CB[1].main_address);
            let total_amount = 123u128;

            // 直接在投票引擎 ProposalData 中写入带 MODULE_TAG 前缀的业务数据
            let data = resolution_issuance::proposal::IssuanceProposalData {
                proposer: recipient.clone(),
                reason: b"runtime-integration".to_vec(),
                total_amount,
                allocations: vec![resolution_issuance::proposal::RecipientAmount {
                    recipient: recipient.clone(),
                    amount: total_amount,
                }],
            };
            let mut encoded = Vec::from(resolution_issuance::MODULE_TAG);
            encoded.extend_from_slice(&data.encode());
            voting_engine::Pallet::<Runtime>::store_proposal_data(proposal_id, encoded)
                .expect("store_proposal_data should succeed");
            voting_engine::Pallet::<Runtime>::store_proposal_meta(
                proposal_id,
                System::block_number(),
            );

            resolution_issuance::pallet::VotingProposalCount::<Runtime>::put(1u32);
            let binding_id = <Runtime as frame_system::Config>::Hashing::hash(b"cleanup-sfid");
            let nonce_hash = <Runtime as frame_system::Config>::Hashing::hash(b"cleanup-nonce");
            sfid_system::pallet::UsedVoteNonce::<Runtime>::insert(
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
                resolution_issuance::pallet::VotingProposalCount::<Runtime>::get(),
                0u32
            );

            // 中文注释：自动延迟清理由 voting-engine 自身单测覆盖，
            // 这里仅验证 runtime 包装层能正确透传到 SFID 投票凭证清理接口。
            RuntimeSfidEligibility::cleanup_vote_credentials(proposal_id);

            assert!(!sfid_system::pallet::UsedVoteNonce::<Runtime>::get(
                proposal_id,
                (binding_id, nonce_hash)
            ));

            assert!(resolution_issuance::pallet::Executed::<Runtime>::get(proposal_id).is_some());
            assert_eq!(
                resolution_issuance::pallet::TotalIssued::<Runtime>::get(),
                total_amount
            );
            assert_eq!(Balances::free_balance(&recipient), total_amount);
        });
    }

    #[test]
    fn resolution_destro_internal_vote_flow_executes_destroy_and_reduces_issuance() {
        new_test_ext().execute_with(|| {
            let nrc_institution = reserve_pallet_id_to_bytes(CHINA_CB[0].shenfen_id)
                .expect("nrc institution id must be valid");
            let nrc_account = AccountId::new(CHINA_CB[0].main_address);
            let initial_balance: Balance = 1_000;
            let destroy_amount: Balance = 100;

            let _ = Balances::deposit_creating(&nrc_account, initial_balance);
            let issuance_before = Balances::total_issuance();

            assert_ok!(ResolutionDestro::propose_destroy(
                RuntimeOrigin::signed(AccountId::new(CHINA_CB[0].duoqian_admins[0])),
                voting_engine::internal_vote::ORG_NRC,
                nrc_institution,
                destroy_amount,
            ));

            let pid = VotingEngine::next_proposal_id().saturating_sub(1);

            for i in 0..13 {
                assert_ok!(VotingEngine::internal_vote(
                    RuntimeOrigin::signed(AccountId::new(CHINA_CB[0].duoqian_admins[i])),
                    pid,
                    true,
                ));
            }

            // 提案数据由 voting-engine 延迟清理，执行后仍保留
            assert!(VotingEngine::get_proposal_data(pid).is_some());

            assert_eq!(
                Balances::free_balance(&nrc_account),
                initial_balance - destroy_amount
            );
            assert_eq!(Balances::total_issuance(), issuance_before - destroy_amount);
        });
    }

    #[test]
    fn onchain_tx_amount_extractor_covers_noamount_amount_and_unknown_paths() {
        new_test_ext().execute_with(|| {
            let who = AccountId::new([1u8; 32]);
            let recipient = AccountId::new([2u8; 32]);

            let system_call = RuntimeCall::System(frame_system::Call::remark {
                remark: b"x".to_vec(),
            });
            let no_amount = <OnchainTxAmountExtractor as onchain_transaction::CallAmount<
                AccountId,
                RuntimeCall,
                Balance,
            >>::amount(&who, &system_call);
            assert!(matches!(
                no_amount,
                onchain_transaction::AmountExtractResult::NoAmount
            ));

            let transfer_call =
                RuntimeCall::Balances(pallet_balances::Call::transfer_allow_death {
                    dest: sp_runtime::MultiAddress::Id(recipient),
                    value: 123,
                });
            let amount = <OnchainTxAmountExtractor as onchain_transaction::CallAmount<
                AccountId,
                RuntimeCall,
                Balance,
            >>::amount(&who, &transfer_call);
            match amount {
                onchain_transaction::AmountExtractResult::Amount(v) => assert_eq!(v, 123),
                _ => panic!("expected amount path"),
            }
        });
    }

    #[test]
    fn onchain_tx_amount_extractor_covers_duoqian_propose_create_and_close() {
        new_test_ext().execute_with(|| {
            let (p1, _) = sr25519::Pair::generate();
            let (p2, _) = sr25519::Pair::generate();
            let signer1 = MultiSigner::from(p1.public());
            let who: AccountId = signer1.into_account();
            let admin2: AccountId = MultiSigner::from(p2.public()).into_account();

            let duoqian_address = AccountId::new([77u8; 32]);
            let beneficiary = AccountId::new([78u8; 32]);
            let sfid_id: duoqian_manage::pallet::SfidIdOf<Runtime> =
                b"GFR-LN001-CB0C-runtime-20260222"
                    .to_vec()
                    .try_into()
                    .expect("sfid id should fit");
            let admins: duoqian_manage::pallet::DuoqianAdminsOf<Runtime> =
                vec![who.clone(), admin2.clone()]
                    .try_into()
                    .expect("admins should fit");
            // 中文注释：propose_create 新接口需要账户名称，本测试只覆盖金额提取路径。
            let account_name: duoqian_manage::pallet::AccountNameOf<Runtime> = b"runtime-test-main"
                .to_vec()
                .try_into()
                .expect("account_name should fit");

            let create_call =
                RuntimeCall::DuoqianManage(duoqian_manage::pallet::Call::propose_create {
                    sfid_id,
                    account_name,
                    admin_count: 2,
                    duoqian_admins: admins.clone(),
                    threshold: 2,
                    amount: 1_000,
                });
            let create_amount = <OnchainTxAmountExtractor as onchain_transaction::CallAmount<
                AccountId,
                RuntimeCall,
                Balance,
            >>::amount(&who, &create_call);
            match create_amount {
                onchain_transaction::AmountExtractResult::Amount(v) => assert_eq!(v, 1_000),
                _ => panic!("expected create amount"),
            }

            let _ = Balances::deposit_creating(&duoqian_address, 777);
            let close_call =
                RuntimeCall::DuoqianManage(duoqian_manage::pallet::Call::propose_close {
                    duoqian_address,
                    beneficiary,
                });
            let close_amount = <OnchainTxAmountExtractor as onchain_transaction::CallAmount<
                AccountId,
                RuntimeCall,
                Balance,
            >>::amount(&who, &close_call);
            match close_amount {
                onchain_transaction::AmountExtractResult::Amount(v) => assert_eq!(v, 777),
                _ => panic!("expected close amount"),
            }
        });
    }

    #[test]
    fn duoqian_reserved_checker_rejects_stake_and_shenfen_fee_addresses() {
        let stake = AccountId::new(primitives::china::china_ch::CHINA_CH[0].stake_address);
        assert!(RuntimeDuoqianReservedAddressChecker::is_reserved(&stake));

        let fee_account = AccountId::new(primitives::china::china_ch::CHINA_CH[0].fee_address);
        assert!(RuntimeDuoqianReservedAddressChecker::is_reserved(
            &fee_account
        ));
    }

    #[test]
    fn runtime_call_filter_blocks_force_transfer_from_stake() {
        let stake = AccountId::new(primitives::china::china_ch::CHINA_CH[0].stake_address);
        let dst = AccountId::new([9u8; 32]);

        let blocked_by_id = RuntimeCall::Balances(pallet_balances::Call::force_transfer {
            source: sp_runtime::MultiAddress::Id(stake),
            dest: sp_runtime::MultiAddress::Id(dst.clone()),
            value: 1,
        });
        assert!(!RuntimeCallFilter::contains(&blocked_by_id));

        let stake_raw = primitives::china::china_ch::CHINA_CH[0].stake_address;
        let blocked_by_32 = RuntimeCall::Balances(pallet_balances::Call::force_transfer {
            source: sp_runtime::MultiAddress::Address32(stake_raw),
            dest: sp_runtime::MultiAddress::Id(dst.clone()),
            value: 1,
        });
        assert!(!RuntimeCallFilter::contains(&blocked_by_32));

        let blocked_by_raw = RuntimeCall::Balances(pallet_balances::Call::force_transfer {
            source: sp_runtime::MultiAddress::Raw(stake_raw.to_vec()),
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
                    primitives::china::china_ch::CHINA_CH[0].stake_address,
                )),
                amount: 1,
            });
        assert!(!RuntimeCallFilter::contains(&blocked_force_unreserve));

        let blocked_force_set_balance =
            RuntimeCall::Balances(pallet_balances::Call::force_set_balance {
                who: sp_runtime::MultiAddress::Id(AccountId::new(
                    primitives::china::china_ch::CHINA_CH[0].stake_address,
                )),
                new_free: 1,
            });
        assert!(!RuntimeCallFilter::contains(&blocked_force_set_balance));
    }

    #[test]
    fn pow_digest_author_finds_pow_engine_author() {
        // 中文注释：pre_digest 现在存储 sr25519 公钥，PowDigestAuthor 解码后派生 AccountId。
        let public = sp_core::sr25519::Public::from_raw([21u8; 32]);
        let expected_account: AccountId = sp_runtime::MultiSigner::from(public).into_account();
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

            // 通过 voting-engine 的 ProposalData 写入提案数据（模块已无本地存储）
            let proposal_id = 7u64;
            let proposer = AccountId::new(CHINA_CB[0].duoqian_admins[0]);
            let reason: runtime_upgrade::pallet::ReasonOf<Runtime> =
                b"upgrade".to_vec().try_into().expect("reason");
            let code: runtime_upgrade::pallet::CodeOf<Runtime> =
                vec![1u8, 2, 3].try_into().expect("code");
            let code_hash = <Runtime as frame_system::Config>::Hashing::hash(code.as_slice());

            let proposal = runtime_upgrade::pallet::Proposal::<Runtime> {
                proposer,
                reason,
                code_hash,
                status: runtime_upgrade::pallet::ProposalStatus::Voting,
            };
            let mut encoded = Vec::from(runtime_upgrade::MODULE_TAG);
            encoded.extend_from_slice(&codec::Encode::encode(&proposal));
            assert_ok!(voting_engine::Pallet::<Runtime>::store_proposal_data(
                proposal_id,
                encoded
            ));
            assert_ok!(voting_engine::Pallet::<Runtime>::store_proposal_object(
                proposal_id,
                runtime_upgrade::pallet::PROPOSAL_OBJECT_KIND_RUNTIME_WASM,
                code.into_inner()
            ));

            // 回调拒绝 → 提案状态应变为 Rejected
            assert_ok!(RuntimeJointVoteResultCallback::on_joint_vote_finalized(
                proposal_id,
                false
            ));
            let raw = voting_engine::Pallet::<Runtime>::get_proposal_data(proposal_id)
                .expect("proposal data should exist");
            let tag = runtime_upgrade::MODULE_TAG;
            assert!(
                raw.len() >= tag.len() && &raw[..tag.len()] == tag,
                "MODULE_TAG mismatch"
            );
            let updated =
                runtime_upgrade::pallet::Proposal::<Runtime>::decode(&mut &raw[tag.len()..])
                    .expect("should decode");
            assert!(matches!(
                updated.status,
                runtime_upgrade::pallet::ProposalStatus::Rejected
            ));
        });
    }

    #[test]
    fn runtime_sfid_verifiers_and_population_snapshot_verify_with_runtime_main_key() {
        new_test_ext().execute_with(|| {
            let (pair, _) = sr25519::Pair::generate();
            let sfid_main: AccountId = MultiSigner::from(pair.public()).into_account();
            sfid_system::pallet::SfidMainAccount::<Runtime>::put(sfid_main);
            assert_eq!(
                sfid_system::Pallet::<Runtime>::current_sfid_verify_pubkey(),
                Some(pair.public().0)
            );
            assert_eq!(
                sfid_system::Pallet::<Runtime>::current_sfid_verify_pubkey(),
                Some(pair.public().0)
            );

            let account = AccountId::new([31u8; 32]);
            let binding_id = <Runtime as frame_system::Config>::Hashing::hash(b"sfid-verify");
            let bind_nonce: sfid_system::pallet::NonceOf<Runtime> =
                b"bind-nonce".to_vec().try_into().expect("nonce should fit");
            let bind_payload = (
                primitives::core_const::DUOQIAN_DOMAIN,
                primitives::core_const::OP_SIGN_BIND,
                frame_system::Pallet::<Runtime>::block_hash(0),
                &account,
                binding_id,
                bind_nonce.as_slice(),
            );
            let bind_msg = blake2_256(&bind_payload.encode());
            let bind_sig = pair.sign(&bind_msg);
            let bind_signature: sfid_system::pallet::SignatureOf<Runtime> = bind_sig
                .0
                .to_vec()
                .try_into()
                .expect("signature should fit");
            let bind_credential = sfid_system::BindCredential {
                binding_id,
                bind_nonce: bind_nonce.clone(),
                signature: bind_signature,
            };
            assert!(RuntimeSfidVerifier::verify(&account, &bind_credential));

            let bad_bind_signature: sfid_system::pallet::SignatureOf<Runtime> =
                vec![7u8; 63].try_into().expect("signature should fit");
            let bad_bind_credential = sfid_system::BindCredential {
                binding_id,
                bind_nonce,
                signature: bad_bind_signature,
            };
            assert!(!RuntimeSfidVerifier::verify(&account, &bad_bind_credential));

            let vote_nonce: sfid_system::pallet::NonceOf<Runtime> =
                b"vote-nonce".to_vec().try_into().expect("nonce should fit");
            let vote_signature: sfid_system::pallet::SignatureOf<Runtime> = pair
                .sign(&blake2_256(
                    &(
                        primitives::core_const::DUOQIAN_DOMAIN,
                        primitives::core_const::OP_SIGN_VOTE,
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

            let pop_nonce: voting_engine::pallet::VoteNonceOf<Runtime> =
                b"pop-nonce".to_vec().try_into().expect("nonce should fit");
            let pop_signature: voting_engine::pallet::VoteSignatureOf<Runtime> = pair
                .sign(&blake2_256(
                    &(
                        primitives::core_const::DUOQIAN_DOMAIN,
                        primitives::core_const::OP_SIGN_POP,
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
            sfid_system::pallet::SfidMainAccount::<Runtime>::put(sfid_main);

            let who = AccountId::new([41u8; 32]);
            let binding_id = <Runtime as frame_system::Config>::Hashing::hash(b"sfid-wrap");
            sfid_system::pallet::BindingIdToAccount::<Runtime>::insert(binding_id, who.clone());
            sfid_system::pallet::AccountToBindingId::<Runtime>::insert(who.clone(), binding_id);

            assert!(RuntimeSfidEligibility::is_eligible(&binding_id, &who));
            assert!(!RuntimeSfidEligibility::is_eligible(
                &binding_id,
                &AccountId::new([42u8; 32])
            ));

            let nonce = b"wrap-nonce";
            let vote_msg = blake2_256(
                &(
                    primitives::core_const::DUOQIAN_DOMAIN,
                    primitives::core_const::OP_SIGN_VOTE,
                    frame_system::Pallet::<Runtime>::block_hash(0),
                    &who,
                    binding_id,
                    88u64,
                    nonce.as_slice(),
                )
                    .encode(),
            );
            let signature = pair.sign(&vote_msg).0.to_vec();
            let nonce_bounded: sfid_system::pallet::NonceOf<Runtime> =
                nonce.to_vec().try_into().expect("nonce should fit");
            let signature_bounded: sfid_system::pallet::SignatureOf<Runtime> =
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

            admins_change::pallet::Institutions::<Runtime>::remove(nrc_id);
            assert!(!is_nrc_admin(&nrc_admin));
            assert!(!is_nrc_admin(&outsider));
            assert!(!RuntimeInternalAdminProvider::is_internal_admin(
                voting_engine::internal_vote::ORG_NRC,
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
            sfid_system::pallet::SfidMainAccount::<Runtime>::put(main);
            let sfid_id = b"GFR-LN001-CB0C-000000001-20260222";
            let register_nonce: duoqian_manage::pallet::RegisterNonceOf<Runtime> =
                b"register-nonce"
                    .to_vec()
                    .try_into()
                    .expect("nonce should fit");
            let register_account_name: duoqian_manage::pallet::AccountNameOf<Runtime> =
                b"test-account-name"
                    .to_vec()
                    .try_into()
                    .expect("account_name should fit");
            let register_signature: duoqian_manage::pallet::RegisterSignatureOf<Runtime> = pair
                .sign(&blake2_256(
                    &(
                        primitives::core_const::DUOQIAN_DOMAIN,
                        primitives::core_const::OP_SIGN_INST,
                        frame_system::Pallet::<Runtime>::block_hash(0),
                        sfid_id.as_slice(),
                        register_account_name.as_slice(),
                        register_nonce.as_slice(),
                    )
                        .encode(),
                ))
                .0
                .to_vec()
                .try_into()
                .expect("signature should fit");
            assert!(
                <RuntimeSfidInstitutionVerifier as duoqian_manage::SfidInstitutionVerifier<
                    duoqian_manage::pallet::AccountNameOf<Runtime>,
                    duoqian_manage::pallet::RegisterNonceOf<Runtime>,
                    duoqian_manage::pallet::RegisterSignatureOf<Runtime>,
                >>::verify_institution_registration(
                    sfid_id.as_slice(),
                    &register_account_name,
                    &register_nonce,
                    &register_signature,
                    None,
                )
            );

            let bad_signature: duoqian_manage::pallet::RegisterSignatureOf<Runtime> =
                vec![9u8; 63].try_into().expect("signature should fit");
            assert!(
                !<RuntimeSfidInstitutionVerifier as duoqian_manage::SfidInstitutionVerifier<
                    duoqian_manage::pallet::AccountNameOf<Runtime>,
                    duoqian_manage::pallet::RegisterNonceOf<Runtime>,
                    duoqian_manage::pallet::RegisterSignatureOf<Runtime>,
                >>::verify_institution_registration(
                    sfid_id.as_slice(),
                    &register_account_name,
                    &register_nonce,
                    &bad_signature,
                    None,
                )
            );
        });
    }
}

pub struct RuntimeInternalAdminProvider;

impl voting_engine::InternalAdminProvider<AccountId> for RuntimeInternalAdminProvider {
    fn is_internal_admin(
        org: u8,
        institution: voting_engine::InstitutionPalletId,
        who: &AccountId,
    ) -> bool {
        admins_change::Pallet::<Runtime>::is_active_subject_admin(org, institution, who)
    }

    fn get_admin_list(
        org: u8,
        institution: voting_engine::InstitutionPalletId,
    ) -> Option<alloc::vec::Vec<AccountId>> {
        admins_change::Pallet::<Runtime>::active_subject_admins(org, institution)
    }

    fn is_pending_internal_admin(
        org: u8,
        institution: voting_engine::InstitutionPalletId,
        who: &AccountId,
    ) -> bool {
        admins_change::Pallet::<Runtime>::is_pending_subject_admin_for_snapshot(
            org,
            institution,
            who,
        )
    }

    fn get_pending_admin_list(
        org: u8,
        institution: voting_engine::InstitutionPalletId,
    ) -> Option<alloc::vec::Vec<AccountId>> {
        admins_change::Pallet::<Runtime>::pending_subject_admins_for_snapshot(org, institution)
    }
}

pub struct RuntimeInternalThresholdProvider;

impl voting_engine::InternalThresholdProvider for RuntimeInternalThresholdProvider {
    fn pass_threshold(org: u8, institution: voting_engine::InstitutionPalletId) -> Option<u32> {
        admins_change::Pallet::<Runtime>::active_subject_threshold(org, institution)
    }

    fn pending_pass_threshold(
        org: u8,
        institution: voting_engine::InstitutionPalletId,
    ) -> Option<u32> {
        admins_change::Pallet::<Runtime>::pending_subject_threshold_for_snapshot(org, institution)
    }
}

pub struct RuntimeInternalAdminCountProvider;

impl voting_engine::InternalAdminCountProvider for RuntimeInternalAdminCountProvider {
    fn admin_count(org: u8, institution: voting_engine::InstitutionPalletId) -> Option<u32> {
        admins_change::Pallet::<Runtime>::active_subject_admin_count(org, institution)
    }
}

pub struct RuntimeSfidEligibility;

impl voting_engine::SfidEligibility<AccountId, Hash> for RuntimeSfidEligibility {
    fn is_eligible(binding_id: &Hash, who: &AccountId) -> bool {
        #[cfg(feature = "runtime-benchmarks")]
        {
            let _ = (
                who,
                sfid_system::pallet::BindingIdToAccount::<Runtime>::get(binding_id),
            );
            true
        }

        #[cfg(not(feature = "runtime-benchmarks"))]
        {
            <sfid_system::Pallet<Runtime> as sfid_system::SfidEligibilityProvider<
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
            if sfid_system::pallet::UsedVoteNonce::<Runtime>::get(
                proposal_id,
                vote_nonce_key.clone(),
            ) {
                return false;
            }

            sfid_system::pallet::UsedVoteNonce::<Runtime>::insert(
                proposal_id,
                vote_nonce_key,
                true,
            );
            true
        }

        #[cfg(not(feature = "runtime-benchmarks"))]
        {
            <sfid_system::Pallet<Runtime> as sfid_system::SfidEligibilityProvider<
                AccountId,
                Hash,
            >>::verify_and_consume_vote_credential(
                binding_id, who, proposal_id, nonce, signature
            )
        }
    }

    fn cleanup_vote_credentials(proposal_id: u64) {
        <sfid_system::Pallet<Runtime> as sfid_system::SfidEligibilityProvider<
            AccountId,
            Hash,
        >>::cleanup_vote_credentials(proposal_id)
    }

    fn cleanup_vote_credentials_chunk(
        proposal_id: u64,
        limit: u32,
    ) -> voting_engine::VoteCredentialCleanup {
        let result =
            sfid_system::pallet::UsedVoteNonce::<Runtime>::clear_prefix(proposal_id, limit, None);
        voting_engine::VoteCredentialCleanup {
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
mod asset_tests {
    use super::*;
    use institution_asset::{InstitutionAsset, InstitutionAssetAction};

    fn stake_account() -> AccountId {
        AccountId::new(primitives::china::china_ch::CHINA_CH[0].stake_address)
    }

    fn reserved_main_account() -> AccountId {
        AccountId::new(primitives::china::china_cb::CHINA_CB[1].main_address)
    }

    fn reserved_fee_account() -> AccountId {
        AccountId::new(primitives::china::china_ch::CHINA_CH[0].fee_address)
    }

    fn ordinary_account() -> AccountId {
        AccountId::new([99u8; 32])
    }

    #[test]
    fn stake_account_is_completely_blocked() {
        let account = stake_account();
        assert!(!RuntimeInstitutionAsset::can_spend(
            &account,
            InstitutionAssetAction::DuoqianTransferExecute
        ));
        assert!(!RuntimeInstitutionAsset::can_spend(
            &account,
            InstitutionAssetAction::DuoqianCloseExecute
        ));
        assert!(!RuntimeInstitutionAsset::can_spend(
            &account,
            InstitutionAssetAction::OffchainBatchDebit
        ));
        assert!(!RuntimeInstitutionAsset::can_spend(
            &account,
            InstitutionAssetAction::OffchainFeeSweepExecute
        ));
    }

    #[test]
    fn reserved_duoqian_only_allows_transfer_and_close() {
        let account = reserved_main_account();
        assert!(RuntimeInstitutionAsset::can_spend(
            &account,
            InstitutionAssetAction::DuoqianTransferExecute
        ));
        assert!(RuntimeInstitutionAsset::can_spend(
            &account,
            InstitutionAssetAction::DuoqianCloseExecute
        ));
        assert!(!RuntimeInstitutionAsset::can_spend(
            &account,
            InstitutionAssetAction::OffchainBatchDebit
        ));
        assert!(!RuntimeInstitutionAsset::can_spend(
            &account,
            InstitutionAssetAction::OffchainFeeSweepExecute
        ));
    }

    #[test]
    fn reserved_fee_account_only_allows_fee_sweep() {
        let account = reserved_fee_account();
        assert!(!RuntimeInstitutionAsset::can_spend(
            &account,
            InstitutionAssetAction::DuoqianTransferExecute
        ));
        assert!(!RuntimeInstitutionAsset::can_spend(
            &account,
            InstitutionAssetAction::DuoqianCloseExecute
        ));
        assert!(!RuntimeInstitutionAsset::can_spend(
            &account,
            InstitutionAssetAction::OffchainBatchDebit
        ));
        assert!(RuntimeInstitutionAsset::can_spend(
            &account,
            InstitutionAssetAction::OffchainFeeSweepExecute
        ));
    }

    #[test]
    fn ordinary_account_allows_all_actions() {
        let account = ordinary_account();
        assert!(RuntimeInstitutionAsset::can_spend(
            &account,
            InstitutionAssetAction::DuoqianTransferExecute
        ));
        assert!(RuntimeInstitutionAsset::can_spend(
            &account,
            InstitutionAssetAction::DuoqianCloseExecute
        ));
        assert!(RuntimeInstitutionAsset::can_spend(
            &account,
            InstitutionAssetAction::OffchainBatchDebit
        ));
        assert!(RuntimeInstitutionAsset::can_spend(
            &account,
            InstitutionAssetAction::OffchainFeeSweepExecute
        ));
    }
}
