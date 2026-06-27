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
use admin_primitives::AdminAccountQuery;
use alloc::vec::Vec;
use codec::Decode;
#[cfg(not(feature = "runtime-benchmarks"))]
use codec::Encode;
#[cfg(not(feature = "runtime-benchmarks"))]
use frame_support::traits::UnfilteredDispatchable;
use frame_support::{
    derive_impl,
    dispatch::DispatchResult,
    parameter_types,
    traits::{
        fungible::{Balanced, Credit, Inspect},
        tokens::{Fortitude, Preservation},
        ConstU128, ConstU32, ConstU64, ConstU8, Contains, EnsureOrigin, FindAuthor, OnUnbalanced,
        VariantCountOf,
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
#[cfg(not(feature = "runtime-benchmarks"))]
use sp_core::sr25519;
use sp_core::Void;
#[cfg(not(feature = "runtime-benchmarks"))]
use sp_io::crypto::sr25519_verify;
#[allow(unused_imports)]
use sp_runtime::traits::Hash as _;
use sp_runtime::{traits::One, Perbill};
use sp_version::RuntimeVersion;

// Local module imports
use super::{
    AccountId, Address, Assets, Balance, Balances, Block, BlockNumber, CitizenIssuance,
    GenesisPallet, Hash, InternalVote, JointVote, LegislationVote, LegislationYuan, Nonce,
    PalletInfo, PrivateAdmins, PublicAdmins, Runtime, RuntimeCall, RuntimeEvent,
    RuntimeFreezeReason, RuntimeHoldReason, RuntimeOrigin, RuntimeTask, System, BLOCK_HASH_COUNT,
    EXISTENTIAL_DEPOSIT, SLOT_DURATION, VERSION,
};
#[cfg(not(feature = "runtime-benchmarks"))]
use super::{ResolutionIssuance, RuntimeUpgrade};

const NORMAL_DISPATCH_RATIO: Perbill =
    Perbill::from_percent(primitives::core_const::NORMAL_DISPATCH_PERCENT);

parameter_types! {
    pub const BlockHashCount: BlockNumber = BLOCK_HASH_COUNT;
    /// 中文注释：使用 BlockNumber 类型声明重试宽限期，避免与具体 u32 常量类型耦合。
    pub const VotingExecutionRetryGraceBlocks: BlockNumber = 21_600;
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
///
/// 中文注释:本链全新创世,无历史链上数据需迁移,故为空。
/// 将来链上线后如需单块迁移,在此 tuple 挂入 `OnRuntimeUpgrade` 即可。
#[allow(unused_parens)]
type SingleBlockMigrations = ();

pub fn is_stake_account(address: &AccountId) -> bool {
    primitives::china::china_ch::CHINA_CH
        .iter()
        .any(|n| address == &AccountId::new(n.stake_account))
}

fn is_reserved_fee_account(address: &AccountId) -> bool {
    primitives::china::china_ch::CHINA_CH
        .iter()
        .any(|n| address == &AccountId::new(n.fee_account))
}

/// 检查是否为国储会安全基金账户。
fn is_safety_fund_account(address: &AccountId) -> bool {
    address == &AccountId::new(primitives::china::china_cb::SAFETY_FUND_ACCOUNT)
}

/// 检查是否为国储会两和基金账户。
fn is_nrc_he_account(address: &AccountId) -> bool {
    address == &AccountId::new(primitives::china::china_cb::NRC_HE_ACCOUNT)
}

/// 检查是否为储委会费用账户（44 个机构的 fee_account）。
fn is_cb_fee_account(address: &AccountId) -> bool {
    primitives::china::china_cb::CHINA_CB
        .iter()
        .any(|n| address == &AccountId::new(n.fee_account))
}

fn is_reserved_main_account(address: &AccountId) -> bool {
    let raw: &[u8] = address.as_ref();
    if raw.len() != 32 {
        return false;
    }
    let mut addr = [0u8; 32];
    addr.copy_from_slice(raw);
    primitives::china::china_zb::is_reserved_main_account(&addr)
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
            // ADR-011 铁律:pallet_assets 内核所有原生 extrinsic 一律 reject。
            // 业务调用必须经由 OnchainIssuance::propose_* → InternalVote/JointVote callback → 内部 root 调用。
            // 中文注释:任何外部 extrinsic 直接打到 pallet_assets 全部不入块,
            // 这是用户代币治理唯一入口铁律的链端兜底。
            RuntimeCall::Assets(_) => false,
            // 未启用模块:onchain-issuance(ADR-011 用户代币,当前为空壳,任务卡 A/B 实装前)
            // 与 offchain-transaction(链下清算行,业务未启用)一律 reject 外部 extrinsic,
            // 保留 pallet 与 storage;日后启用只需删除对应分支并走一次 setCode,无需重新创世。
            RuntimeCall::OnchainIssuance(_) => false,
            RuntimeCall::OffchainTransaction(_) => false,
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
    /// 中文注释：全局调用过滤器，禁止 stake_account 参与 force_* 余额调用，并封禁强制总发行量调整入口。
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
            RuntimeSafetyFundAccountProvider,
        >,
        RuntimeFeeKindClassifier,
        RuntimeFeePayerExtractor,
    >;
    type OperationalFeeMultiplier = ConstU8<{ primitives::fee_policy::OPERATIONAL_FEE_MULTIPLIER }>;
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
            primitives::china::china_cb::CHINA_CB[0].fee_account,
        ))
    }
}

pub struct RuntimeSafetyFundAccountProvider;

impl onchain_transaction::SafetyFundAccountProvider<AccountId>
    for RuntimeSafetyFundAccountProvider
{
    fn safety_fund_account() -> AccountId {
        AccountId::new(primitives::china::china_cb::SAFETY_FUND_ACCOUNT)
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

pub struct RuntimeFeeKindClassifier;

impl onchain_transaction::CallFeeKind<AccountId, RuntimeCall, Balance>
    for RuntimeFeeKindClassifier
{
    fn fee_kind(
        who: &AccountId,
        call: &RuntimeCall,
    ) -> onchain_transaction::FeeChargeKind<Balance> {
        use onchain_transaction::FeeChargeKind;

        match call {
            RuntimeCall::Balances(pallet_balances::Call::transfer_allow_death {
                value, ..
            }) => FeeChargeKind::OnchainAmount(*value),
            RuntimeCall::Balances(pallet_balances::Call::transfer_keep_alive { value, .. }) => {
                FeeChargeKind::OnchainAmount(*value)
            }
            RuntimeCall::Balances(pallet_balances::Call::force_transfer { value, .. }) => {
                FeeChargeKind::OnchainAmount(*value)
            }
            RuntimeCall::Balances(pallet_balances::Call::force_unreserve { amount, .. }) => {
                FeeChargeKind::OnchainAmount(*amount)
            }
            RuntimeCall::Balances(pallet_balances::Call::force_set_balance {
                new_free, ..
            }) => FeeChargeKind::OnchainAmount(*new_free),
            RuntimeCall::Balances(pallet_balances::Call::force_adjust_total_issuance {
                delta,
                ..
            }) => FeeChargeKind::OnchainAmount(*delta),
            RuntimeCall::Balances(pallet_balances::Call::burn { value, .. }) => {
                FeeChargeKind::OnchainAmount(*value)
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
                FeeChargeKind::OnchainAmount(value)
            }
            // 中文注释：PersonalAdmins 的 propose_create/propose_close 是治理提案交易，
            // 交易本身固定收 1 元；执行阶段的资金手续费由对应 pallet 内部按金额另行处理。
            RuntimeCall::PersonalAdmins(personal_admins::pallet::Call::propose_create {
                ..
            })
            | RuntimeCall::PersonalAdmins(personal_admins::pallet::Call::propose_close {
                ..
            }) => FeeChargeKind::VoteFlat,
            RuntimeCall::PersonalAdmins(_) => FeeChargeKind::VoteFlat,
            RuntimeCall::OrganizationManage(
                organization_manage::pallet::Call::propose_create_institution { .. },
            )
            | RuntimeCall::OrganizationManage(organization_manage::pallet::Call::propose_close {
                ..
            }) => FeeChargeKind::VoteFlat,
            // 中文注释：CID 注册由签名账户提交，不属于系统自动调用，按治理操作固定 1 元。
            RuntimeCall::OrganizationManage(
                organization_manage::pallet::Call::register_cid_institution { .. },
            ) => FeeChargeKind::VoteFlat,
            // 付费调用交易：多签管理其他操作（cleanup_X 等）按投票统一价 1 元/次
            RuntimeCall::OrganizationManage(_) => FeeChargeKind::VoteFlat,
            // 免费调用交易：系统内部 / 自动化 / 货币政策类
            RuntimeCall::System(_) => FeeChargeKind::Free,
            RuntimeCall::Timestamp(_) => FeeChargeKind::Free,
            RuntimeCall::ProvincialBankInterest(_) => FeeChargeKind::Free,
            RuntimeCall::CitizenIssuance(_) => FeeChargeKind::Free,
            // GRANDPA pallet:report_equivocation(签名版)/ report_equivocation_unsigned(unsigned 路径
            // 不走 ChargeTransactionPayment) / note_stalled(Root,本链无 sudo 实际不可达)。
            // 等价证据上报本就属公益保护链稳定运行,统一免费。
            RuntimeCall::Grandpa(_) => FeeChargeKind::Free,
            // 中文注释：决议发行 / 决议销毁的 propose_X 是治理提案交易，固定 1 元；
            // 维护型 Root / 系统型调用免费。
            RuntimeCall::ResolutionIssuance(ref issuance_call) => match issuance_call {
                resolution_issuance::pallet::Call::propose_resolution_issuance { .. } => {
                    FeeChargeKind::VoteFlat
                }
                _ => FeeChargeKind::Free,
            },
            RuntimeCall::ResolutionDestro(resolution_destro::pallet::Call::propose_destroy {
                ..
            }) => FeeChargeKind::VoteFlat,
            RuntimeCall::ResolutionDestro(_) => FeeChargeKind::Free,
            // 投票引擎主 pallet 公开 call 共 3 个:
            //   finalize_proposal — 任意人推动超时结算,免费;
            //   retry_passed_proposal / cancel_passed_proposal — 管理员手动重试/取消,VOTE_FLAT_FEE。
            RuntimeCall::VotingEngine(ref ve_call) => match ve_call {
                votingengine::pallet::Call::finalize_proposal { .. } => FeeChargeKind::Free,
                _ => FeeChargeKind::VoteFlat,
            },
            // CidSystem 全部 6 个 extrinsic(含 bind_cid / unbind_cid 等)按投票统一价 1 元/次。
            RuntimeCall::CidSystem(_) => FeeChargeKind::VoteFlat,
            // FullnodeIssuance bind_reward_wallet / rebind_reward_wallet:1 元/次。
            RuntimeCall::FullnodeIssuance(_) => FeeChargeKind::VoteFlat,
            // 手动重试/取消统一收口至 votingengine::retry_passed_proposal /
            // cancel_passed_proposal(在 RuntimeCall::VotingEngine 分支按 VOTE_FLAT_FEE 处理)。
            // 业务 pallet 的 propose_X / cleanup_X 全部按 VOTE_FLAT_FEE 收费(1 元/次)。
            RuntimeCall::GenesisAdmins(_)
            | RuntimeCall::PublicAdmins(_)
            | RuntimeCall::PrivateAdmins(_) => FeeChargeKind::VoteFlat,
            RuntimeCall::RuntimeUpgrade(_) => FeeChargeKind::VoteFlat,
            RuntimeCall::GrandpaKeyChange(_) => FeeChargeKind::VoteFlat,
            // 立法院模块 propose_enact_law / propose_amend_law / propose_repeal_law 是治理提案交易,
            // 固定按投票统一价 1 元/次(ADR-027),与其它治理 pallet 的 propose_X 一致。
            RuntimeCall::LegislationYuan(_) => FeeChargeKind::VoteFlat,
            // 中文注释：多签转账 propose_X 只是创建治理提案，交易本身固定收 1 元；
            // 真正转账执行时，multisig-transfer 内部再按转出金额 × 0.1% 收链上交易费。
            RuntimeCall::MultisigTransfer(ref dt_call) => match dt_call {
                multisig_transfer::pallet::Call::propose_transfer { .. }
                | multisig_transfer::pallet::Call::propose_safety_fund_transfer { .. }
                | multisig_transfer::pallet::Call::propose_sweep_to_main { .. } => {
                    FeeChargeKind::VoteFlat
                }
                // 兜底:未来若新增非金额型管理 extrinsic 按投票统一价 1 元/次。
                _ => FeeChargeKind::VoteFlat,
            },
            // 清算行(L2)扫码支付清算。
            RuntimeCall::OffchainTransaction(ref offchain_call) => {
                match offchain_call {
                    // L3 充值 / 提现:按金额计费(链上资金交易 0.1% 最低 0.1 元)
                    offchain_transaction::pallet::Call::deposit { amount } => {
                        FeeChargeKind::OnchainAmount(*amount)
                    }
                    offchain_transaction::pallet::Call::withdraw { amount } => {
                        FeeChargeKind::OnchainAmount(*amount)
                    }
                    // 中文注释：清算行批次 V2 是链下交易费，结算执行阶段已经把
                    // Σ batch[i].fee_amount 转给清算行费用账户，本层只标记类别不二次分账。
                    offchain_transaction::pallet::Call::submit_offchain_batch_v2 {
                        batch, ..
                    } => {
                        let mut total_fee: u128 = 0;
                        for item in batch.iter() {
                            total_fee = total_fee.saturating_add(item.fee_amount);
                        }
                        FeeChargeKind::OffchainFee(total_fee)
                    }
                    // 全局费率上限调整(Root Origin,免费)
                    offchain_transaction::pallet::Call::set_max_l2_fee_rate { .. } => {
                        FeeChargeKind::Free
                    }
                    // 其他付费调用(bind_clearing_bank / switch_bank / propose_l2_fee_rate):
                    // 按投票统一价 1 元/次
                    _ => FeeChargeKind::VoteFlat,
                }
            }
            // 3 个 mode-specific 投票 extrinsic 全部按投票统一价 1 元/次:
            //   InternalVote::cast / JointVote::cast_admin / JointVote::cast_referendum
            RuntimeCall::InternalVote(_) => FeeChargeKind::VoteFlat,
            RuntimeCall::JointVote(_) => FeeChargeKind::VoteFlat,
            // 立法投票 3 个 extrinsic(prepare_population_snapshot / cast_house_vote /
            // cast_referendum_vote)按投票统一价 1 元/次(ADR-027)。
            RuntimeCall::LegislationVote(_) => FeeChargeKind::VoteFlat,
            // OnchainIssuance 暴露 10 个 propose_X extrinsic(call_index 0..=4 业务 / 10..=14 监管)。
            // 全部按 VOTE_FLAT_FEE = 1 元/次,与 GMB 其他业务 pallet 的 propose_X 一致。
            // 1000 GMB 创建费走 onchain_issuance::fee::reserve_creation_deposit 内部 reserve(propose_issue 内部完成),
            // 与 RuntimeFeeKindClassifier 计费正交。
            RuntimeCall::OnchainIssuance(_) => FeeChargeKind::VoteFlat,
            // pallet_assets 内核所有原生 extrinsic 已被 RuntimeCallFilter 拦在入口,
            // 永远到不了本路径;此分支仅供编译期 exhaustive 检查。
            RuntimeCall::Assets(_) => FeeChargeKind::Free,
            // ElectionVote 当前是空骨架(无 extrinsic / 无 RuntimeCall 变体)。
            // 中文注释：对 Balances 未覆盖分支按 Unknown 拒绝,避免"有金额但漏提取"。
            //
            // 不再写 `_ => Unknown` 兜底:补 RuntimeCall::Grandpa 之后所有 pallet 变体已穷尽,
            // 将来新增 pallet 若忘记归类会编译期 non-exhaustive match 报错,
            // 强制开发者显式选择五类费用模型之一。
            RuntimeCall::Balances(_) => FeeChargeKind::Unknown,
        }
    }
}

pub struct RuntimeFeePayerExtractor;

impl onchain_transaction::CallFeePayer<AccountId, RuntimeCall> for RuntimeFeePayerExtractor {
    fn fee_payer(_who: &AccountId, call: &RuntimeCall) -> Option<AccountId> {
        match call {
            // 清算行 V2 批次:链上 gas 由 institution_main 的费用账户直接承担。
            //
            // **收款方主导清算**模型下,
            // institution_main = 收款方清算行主账户。fee_account_of(institution_main)
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
impl provincialbank_interest::Config for Runtime {
    type Currency = Balances;
    type BlocksPerYear = ConstU64<{ primitives::pow_const::BLOCKS_PER_YEAR }>;
    type WeightInfo = provincialbank_interest::weights::SubstrateWeight<Runtime>;
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

pub struct RuntimeAccountValidator;

impl organization_manage::AccountValidator<AccountId> for RuntimeAccountValidator {
    fn is_valid(account: &AccountId) -> bool {
        // 中文注释：禁止零账户。
        if account == &AccountId::new([0u8; 32]) {
            return false;
        }

        // 中文注释：禁止占用“国储会/省储会”的制度保留交易账户。
        if primitives::china::china_cb::CHINA_CB
            .iter()
            .any(|n| account == &AccountId::new(n.main_account))
        {
            return false;
        }

        // 中文注释：禁止占用“省储行”的制度保留交易账户。
        if primitives::china::china_ch::CHINA_CH
            .iter()
            .any(|n| account == &AccountId::new(n.main_account))
        {
            return false;
        }

        // 中文注释：禁止占用省储行费用账户（BLAKE2-256 派生）。
        if primitives::china::china_ch::CHINA_CH
            .iter()
            .any(|n| account == &AccountId::new(n.fee_account))
        {
            return false;
        }

        // 中文注释：禁止占用国储会安全基金账户。
        if is_safety_fund_account(account) {
            return false;
        }

        // 中文注释：禁止占用国储会两和基金账户。
        if is_nrc_he_account(account) {
            return false;
        }

        // 中文注释：禁止占用储委会费用账户（44 个机构）。
        if is_cb_fee_account(account) {
            return false;
        }

        true
    }
}

pub struct RuntimeReservedAccountGuard;
pub struct RuntimeCidInstitutionVerifier;

pub struct RuntimeProtectedSourceChecker;
pub struct RuntimeInstitutionAsset;

impl organization_manage::ProtectedSourceChecker<AccountId> for RuntimeProtectedSourceChecker {
    fn is_protected(address: &AccountId) -> bool {
        is_stake_account(address)
    }
}

impl institution_asset::InstitutionAsset<AccountId> for RuntimeInstitutionAsset {
    fn can_spend(source: &AccountId, action: institution_asset::InstitutionAssetAction) -> bool {
        // 中文注释：匹配顺序很重要——更具体的账户类型必须放在更宽泛的类型之前。
        // fee_account 同时出现在 CHINA_RESERVED_MAIN_ACCOUNTS 列表中（同由 BLAKE2 派生且统一保留），
        // 如果 is_reserved_main_account 先匹配，fee_account 会被错误地按主账户规则放行。

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
        if source == &AccountId::new(primitives::china::china_cb::SAFETY_FUND_ACCOUNT) {
            return matches!(
                action,
                institution_asset::InstitutionAssetAction::NrcSafetyFundTransfer
            );
        }

        // 5. 多签保留账户（范围最宽）：只允许多签转账和关闭
        if is_reserved_main_account(source) {
            return matches!(
                action,
                institution_asset::InstitutionAssetAction::MultisigTransferExecute
                    | institution_asset::InstitutionAssetAction::MultisigCloseExecute
            );
        }

        // 6. 普通账户：全放行
        true
    }
}

impl organization_manage::ReservedAccountGuard<AccountId> for RuntimeReservedAccountGuard {
    fn is_reserved(account: &AccountId) -> bool {
        // 中文注释：禁止占用省储行 stake_account（制度保留账户）。
        if primitives::china::china_ch::CHINA_CH
            .iter()
            .any(|n| account == &AccountId::new(n.stake_account))
        {
            return true;
        }

        // 中文注释：禁止占用省储行费用账户（BLAKE2-256 派生）。
        if primitives::china::china_ch::CHINA_CH
            .iter()
            .any(|n| account == &AccountId::new(n.fee_account))
        {
            return true;
        }

        // 中文注释：禁止占用国储会安全基金账户。
        if is_safety_fund_account(account) {
            return true;
        }

        // 中文注释：禁止占用国储会两和基金账户。
        if is_nrc_he_account(account) {
            return true;
        }

        // 中文注释：禁止占用储委会费用账户（44 个机构）。
        if is_cb_fee_account(account) {
            return true;
        }

        is_reserved_main_account(account)
    }
}

#[cfg(not(feature = "runtime-benchmarks"))]
fn issuer_admin_public(
    issuer_main_account: &AccountId,
    signer_pubkey: &[u8; 32],
) -> Option<sr25519::Public> {
    let signer_account = AccountId::new(*signer_pubkey);
    if !RuntimeAdminAccountQuery::is_active_admin_of_account(issuer_main_account, &signer_account) {
        return None;
    }
    Some(sr25519::Public::from_raw(*signer_pubkey))
}

#[cfg(not(feature = "runtime-benchmarks"))]
fn sr25519_signature_from_bytes(signature: &[u8]) -> Option<sr25519::Signature> {
    if signature.len() != 64 {
        return None;
    }
    let mut sig_raw = [0u8; 64];
    sig_raw.copy_from_slice(signature);
    Some(sr25519::Signature::from_raw(sig_raw))
}

impl
    organization_manage::CidInstitutionVerifier<
        AccountId,
        organization_manage::pallet::AccountNameOf<Runtime>,
        organization_manage::pallet::RegisterNonceOf<Runtime>,
        organization_manage::pallet::RegisterSignatureOf<Runtime>,
    > for RuntimeCidInstitutionVerifier
{
    fn verify_institution_registration(
        cid_number: &[u8],
        cid_full_name: &organization_manage::pallet::AccountNameOf<Runtime>,
        account_names: &[Vec<u8>],
        nonce: &organization_manage::pallet::RegisterNonceOf<Runtime>,
        signature: &organization_manage::pallet::RegisterSignatureOf<Runtime>,
        issuer_cid_number: &[u8],
        issuer_main_account: &AccountId,
        signer_pubkey: &[u8; 32],
        scope_province_name: &[u8],
        scope_city_name: &[u8],
    ) -> bool {
        #[cfg(feature = "runtime-benchmarks")]
        {
            let _ = (
                issuer_cid_number,
                issuer_main_account,
                signer_pubkey,
                scope_province_name,
                scope_city_name,
            );
            return !cid_number.is_empty()
                && !cid_full_name.is_empty()
                && !account_names.is_empty()
                && !nonce.is_empty()
                && !signature.is_empty();
        }

        #[cfg(not(feature = "runtime-benchmarks"))]
        {
            let Some(public) = issuer_admin_public(issuer_main_account, signer_pubkey) else {
                return false;
            };
            let Some(signature) = sr25519_signature_from_bytes(signature.as_slice()) else {
                return false;
            };

            // 中文注释：这里必须和 CID 端 `/registration-info` 的签名 payload 严格一致。
            // payload 字段(GMB + OP_SIGN_INST 域头由 signing_message 统一拼接):
            // genesis_hash + cid_number + cid_full_name + account_names[] + nonce
            // + 签发机构 + 作用域。
            let payload = (
                frame_system::Pallet::<Runtime>::block_hash(0),
                cid_number,
                cid_full_name.as_slice(),
                account_names,
                nonce.as_slice(),
                issuer_cid_number,
                issuer_main_account,
                signer_pubkey,
                scope_province_name,
                scope_city_name,
            );
            let msg = primitives::sign::signing_message(
                primitives::sign::OP_SIGN_INST,
                &payload.encode(),
            );

            sr25519_verify(&signature, &msg, &public)
        }
    }

    fn verify_institution_deregistration(
        scope: u8,
        cid_number: &[u8],
        account_name: &[u8],
        target_account: &AccountId,
        nonce: &organization_manage::pallet::RegisterNonceOf<Runtime>,
        signature: &organization_manage::pallet::RegisterSignatureOf<Runtime>,
        issuer_cid_number: &[u8],
        issuer_main_account: &AccountId,
        signer_pubkey: &[u8; 32],
    ) -> bool {
        #[cfg(feature = "runtime-benchmarks")]
        {
            let _ = (
                scope,
                account_name,
                target_account,
                issuer_cid_number,
                issuer_main_account,
                signer_pubkey,
            );
            return !cid_number.is_empty() && !nonce.is_empty() && !signature.is_empty();
        }

        #[cfg(not(feature = "runtime-benchmarks"))]
        {
            let Some(public) = issuer_admin_public(issuer_main_account, signer_pubkey) else {
                return false;
            };
            let Some(signature) = sr25519_signature_from_bytes(signature.as_slice()) else {
                return false;
            };

            // 中文注释:必须与 CID 端注销凭证签发 payload 严格一致。
            // payload 字段(GMB + OP_SIGN_DEREGISTER 域头由 signing_message 统一拼接):
            // genesis_hash + scope + cid_number + account_name + target_account
            // + nonce + 签发机构 + 签发管理员公钥。scope 与 target_account 入签名,
            // 防换范围/换账户重放。
            let payload = (
                frame_system::Pallet::<Runtime>::block_hash(0),
                scope,
                cid_number,
                account_name,
                target_account,
                nonce.as_slice(),
                issuer_cid_number,
                issuer_main_account,
                signer_pubkey,
            );
            let msg = primitives::sign::signing_message(
                primitives::sign::OP_SIGN_DEREGISTER,
                &payload.encode(),
            );

            sr25519_verify(&signature, &msg, &public)
        }
    }
}

impl organization_manage::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type InternalVoteEngine = InternalVote;
    type PublicAdminLifecycle = PublicAdmins;
    type PrivateAdminLifecycle = PrivateAdmins;
    type AdminAccountQuery = RuntimeAdminAccountQuery;
    type AccountValidator = RuntimeAccountValidator;
    type ReservedAccountChecker = RuntimeReservedAccountGuard;
    type ProtectedSourceChecker = RuntimeProtectedSourceChecker;
    type InstitutionAsset = RuntimeInstitutionAsset;
    type CidInstitutionVerifier = RuntimeCidInstitutionVerifier;
    type FeeRouter = TransferFeeRouter;
    type MaxAdmins = MaxAdminsPerInstitution;
    type MaxCidNumberLength = ConstU32<{ primitives::core_const::CID_NUMBER_MAX_BYTES }>;
    type MaxAccountNameLength = ConstU32<128>;
    type MaxRegisterNonceLength = ConstU32<64>;
    type MaxRegisterSignatureLength = ConstU32<64>;
    type MaxInstitutionAccounts = ConstU32<16>;
    type MinCreateAmount = ConstU128<111>;
    type MinCloseBalance = ConstU128<121>;
    type WeightInfo = organization_manage::weights::SubstrateWeight<Runtime>;
}

impl personal_admins::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type InternalVoteEngine = InternalVote;
    type AccountValidator = RuntimeAccountValidator;
    type ReservedAccountChecker = RuntimeReservedAccountGuard;
    type ProtectedSourceChecker = RuntimeProtectedSourceChecker;
    type InstitutionAsset = RuntimeInstitutionAsset;
    type FeeRouter = TransferFeeRouter;
    type MaxAccountNameLength = ConstU32<128>;
    type MaxPersonalAccountAdmins = MaxPersonalAccountAdmins;
    type MinCreateAmount = ConstU128<111>;
    type MinCloseBalance = ConstU128<121>;
    type WeightInfo = personal_admins::weights::SubstrateWeight<Runtime>;
}

// 三处 CID 验签:
// - `RuntimeCidVerifier`(BindCredential / 公民身份绑定)
// - `RuntimeCidVoteVerifier`(公民投票凭证)
// - `RuntimePopulationSnapshotVerifier`(联合提案人口快照)
// 全部按 `issuer_cid_number + issuer_main_account + signer_pubkey` 校验签发身份;
// `issuer_main_account` 的管理员真源统一由 runtime 管理员查询路由分发。

pub struct RuntimeCidVerifier;

impl
    cid_system::CidVerifier<
        AccountId,
        Hash,
        cid_system::pallet::NonceOf<Runtime>,
        cid_system::pallet::SignatureOf<Runtime>,
    > for RuntimeCidVerifier
{
    fn verify(account: &AccountId, credential: &cid_system::pallet::CredentialOf<Runtime>) -> bool {
        #[cfg(feature = "runtime-benchmarks")]
        {
            let _ = (account, credential);
            return !credential.bind_nonce.is_empty()
                && !credential.signature.is_empty()
                && !credential.issuer_cid_number.is_empty()
                && !credential.scope_province_name.is_empty();
        }

        #[cfg(not(feature = "runtime-benchmarks"))]
        {
            let Some(public) =
                issuer_admin_public(&credential.issuer_main_account, &credential.signer_pubkey)
            else {
                return false;
            };
            let Some(signature) = sr25519_signature_from_bytes(credential.signature.as_slice())
            else {
                return false;
            };

            // payload 字段(GMB + OP_SIGN_BIND 域头由 signing_message 统一拼接):
            //   block_hash(0) + account + binding_id + bind_nonce + 签发机构 + 作用域。
            let payload = (
                frame_system::Pallet::<Runtime>::block_hash(0),
                account,
                credential.binding_id,
                credential.bind_nonce.as_slice(),
                credential.issuer_cid_number.as_slice(),
                &credential.issuer_main_account,
                &credential.signer_pubkey,
                credential.scope_province_name.as_slice(),
                credential.scope_city_name.as_slice(),
            );
            let msg = primitives::sign::signing_message(
                primitives::sign::OP_SIGN_BIND,
                &payload.encode(),
            );

            sr25519_verify(&signature, &msg, &public)
        }
    }
}

pub struct RuntimeCidVoteVerifier;

impl
    cid_system::CidVoteVerifier<
        AccountId,
        Hash,
        cid_system::pallet::NonceOf<Runtime>,
        cid_system::pallet::SignatureOf<Runtime>,
    > for RuntimeCidVoteVerifier
{
    fn verify_vote(
        account: &AccountId,
        binding_id: Hash,
        proposal_id: u64,
        nonce: &cid_system::pallet::NonceOf<Runtime>,
        signature: &cid_system::pallet::SignatureOf<Runtime>,
        issuer_cid_number: &[u8],
        issuer_main_account: &AccountId,
        signer_pubkey: &[u8; 32],
        scope_province_name: &[u8],
        scope_city_name: &[u8],
    ) -> bool {
        #[cfg(feature = "runtime-benchmarks")]
        {
            let _ = (
                account,
                binding_id,
                proposal_id,
                issuer_cid_number,
                issuer_main_account,
                signer_pubkey,
                scope_province_name,
                scope_city_name,
            );
            return !nonce.is_empty() && !signature.is_empty();
        }

        #[cfg(not(feature = "runtime-benchmarks"))]
        {
            let Some(public) = issuer_admin_public(issuer_main_account, signer_pubkey) else {
                return false;
            };
            let Some(signature) = sr25519_signature_from_bytes(signature.as_slice()) else {
                return false;
            };

            // payload 字段(GMB + OP_SIGN_VOTE 域头由 signing_message 统一拼接):
            //   block_hash(0) + account + binding_id + proposal_id + nonce + 签发机构 + 作用域。
            let payload = (
                frame_system::Pallet::<Runtime>::block_hash(0),
                account,
                binding_id,
                proposal_id,
                nonce.as_slice(),
                issuer_cid_number,
                issuer_main_account,
                signer_pubkey,
                scope_province_name,
                scope_city_name,
            );
            let msg = primitives::sign::signing_message(
                primitives::sign::OP_SIGN_VOTE,
                &payload.encode(),
            );

            sr25519_verify(&signature, &msg, &public)
        }
    }
}

impl cid_system::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxCredentialNonceLength = ConstU32<64>;
    // 中文注释：CID 绑定与投票验签统一使用 64 字节原始 sr25519 签名。
    type MaxCredentialSignatureLength = ConstU32<64>;
    type CidVerifier = RuntimeCidVerifier;
    type CidVoteVerifier = RuntimeCidVoteVerifier;
    type OnCidBound = CitizenIssuance;
    // unbind_cid 由 Root 治理 origin 鉴权。
    // step2b 起结合 organization-manage 凭证体系决定最终 origin 模型（治理多签 / 省级 admin 直签）。
    type UnbindOrigin = frame_system::EnsureRoot<AccountId>;
    type WeightInfo = cid_system::weights::SubstrateWeight<Runtime>;
}

pub struct RuntimePopulationSnapshotVerifier;

impl
    votingengine::PopulationSnapshotVerifier<
        AccountId,
        votingengine::pallet::VoteNonceOf<Runtime>,
        votingengine::pallet::VoteSignatureOf<Runtime>,
    > for RuntimePopulationSnapshotVerifier
{
    fn verify_population_snapshot(
        who: &AccountId,
        eligible_total: u64,
        nonce: &votingengine::pallet::VoteNonceOf<Runtime>,
        signature: &votingengine::pallet::VoteSignatureOf<Runtime>,
        issuer_cid_number: &[u8],
        issuer_main_account: &AccountId,
        signer_pubkey: &[u8; 32],
        scope_province_name: &[u8],
        scope_city_name: &[u8],
    ) -> bool {
        #[cfg(feature = "runtime-benchmarks")]
        {
            let _ = (
                who,
                issuer_cid_number,
                issuer_main_account,
                signer_pubkey,
                scope_province_name,
                scope_city_name,
            );
            eligible_total > 0 && !nonce.is_empty() && !signature.is_empty()
        }

        #[cfg(not(feature = "runtime-benchmarks"))]
        {
            let Some(public) = issuer_admin_public(issuer_main_account, signer_pubkey) else {
                return false;
            };
            let Some(signature) = sr25519_signature_from_bytes(signature.as_slice()) else {
                return false;
            };

            // payload 字段(GMB + OP_SIGN_POP 域头由 signing_message 统一拼接):
            //   block_hash(0) + who + eligible_total + nonce + 签发机构 + 作用域。
            let payload = (
                frame_system::Pallet::<Runtime>::block_hash(0),
                who,
                eligible_total,
                nonce.as_slice(),
                issuer_cid_number,
                issuer_main_account,
                signer_pubkey,
                scope_province_name,
                scope_city_name,
            );
            let msg =
                primitives::sign::signing_message(primitives::sign::OP_SIGN_POP, &payload.encode());

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
    /// 管理员治理：单个注册机构账户管理员上限。
    ///
    /// 中文注释：物理 BoundedVec 上限必须覆盖机构账户 1989 人场景；个人账户
    /// 另由 MaxPersonalAccountAdmins 限制为 64。
    pub const MaxAdminsPerInstitution: u32 = 1989;
    /// 管理员治理：单个个人账户管理员上限。
    pub const MaxPersonalAccountAdmins: u32 = 64;
    /// GRANDPA authority set 变更生效延迟（单位：区块）。
    /// 取非 0，给运维注入新 gran 私钥预留窗口，避免立即切换导致短时失票。
    pub const GrandpaAuthoritySetChangeDelay: u32 = 30;
}

impl genesis_admins::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxAdminsPerInstitution = MaxAdminsPerInstitution;
    type MaxPersonalAccountAdmins = MaxPersonalAccountAdmins;
    type InternalVoteEngine = InternalVote;
    type PublicAdminLifecycle = PublicAdmins;
    type WeightInfo = genesis_admins::weights::SubstrateWeight<Runtime>;
}

impl public_admins::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxAdminsPerInstitution = MaxAdminsPerInstitution;
    type InternalVoteEngine = InternalVote;
    type WeightInfo = public_admins::weights::SubstrateWeight<Runtime>;
}

impl private_admins::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxAdminsPerInstitution = MaxAdminsPerInstitution;
    type InternalVoteEngine = InternalVote;
    type WeightInfo = private_admins::weights::SubstrateWeight<Runtime>;
}

impl resolution_destro::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type InternalVoteEngine = InternalVote;
    type WeightInfo = resolution_destro::weights::SubstrateWeight<Runtime>;
}

impl grandpakey_change::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type GrandpaChangeDelay = GrandpaAuthoritySetChangeDelay;
    type InternalVoteEngine = InternalVote;
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
            RuntimeSafetyFundAccountProvider,
        >;
        <FeeRouter as frame_support::traits::tokens::imbalance::OnUnbalanced<_>>::on_unbalanced(
            credit,
        );
    }
}

impl multisig_transfer::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxRemarkLen = ConstU32<256>;
    type FeeRouter = TransferFeeRouter;
    // 多签 admin 配置查询拆给两个独立 pallet。
    // 转账治理时 multisig-transfer 通过 union 调用,先问个人侧、再问机构侧。
    type PersonalQuery = personal_admins::Pallet<Runtime>;
    type InstitutionQuery = organization_manage::Pallet<Runtime>;
    type WeightInfo = multisig_transfer::weights::SubstrateWeight<Runtime>;
}

pub struct RuntimeAdminAccountQuery;

impl RuntimeAdminAccountQuery {
    fn is_active_admin_of_account(account: &AccountId, who: &AccountId) -> bool {
        if let Some(institution_code) =
            organization_manage::Pallet::<Runtime>::resolve_institution_code_for_account(account)
        {
            return Self::is_active_account_admin(institution_code, account.clone(), who);
        }

        // 中文注释：未被 organization-manage 登记的创世账户只可能走创世管理员模块。
        [
            admin_primitives::FRG,
            primitives::code::NRC,
            primitives::code::PRC,
            primitives::code::PRB,
        ]
        .iter()
        .any(|code| Self::is_active_account_admin(*code, account.clone(), who))
    }
}

impl AdminAccountQuery<AccountId> for RuntimeAdminAccountQuery {
    fn is_genesis_protected(account: &AccountId) -> bool {
        genesis_admins::Pallet::<Runtime>::is_genesis_protected(account)
    }

    fn active_admin_account_exists(
        institution_code: primitives::code::InstitutionCode,
        admin_root_account_id: AccountId,
    ) -> bool {
        if admin_primitives::is_personal_admin_code(&institution_code) {
            return personal_admins::Pallet::<Runtime>::active_admin_account_exists(
                institution_code,
                admin_root_account_id,
            );
        }
        if admin_primitives::is_genesis_admin_code(&institution_code) {
            return genesis_admins::Pallet::<Runtime>::active_admin_account_exists(
                institution_code,
                admin_root_account_id,
            );
        }
        if admin_primitives::is_public_admin_code(&institution_code) {
            return public_admins::Pallet::<Runtime>::active_admin_account_exists(
                institution_code,
                admin_root_account_id,
            );
        }
        if admin_primitives::is_private_admin_code(&institution_code) {
            return private_admins::Pallet::<Runtime>::active_admin_account_exists(
                institution_code,
                admin_root_account_id,
            );
        }
        false
    }

    fn is_active_account_admin(
        institution_code: primitives::code::InstitutionCode,
        admin_root_account_id: AccountId,
        who: &AccountId,
    ) -> bool {
        if admin_primitives::is_personal_admin_code(&institution_code) {
            return personal_admins::Pallet::<Runtime>::is_active_account_admin(
                institution_code,
                admin_root_account_id,
                who,
            );
        }
        if admin_primitives::is_genesis_admin_code(&institution_code) {
            return genesis_admins::Pallet::<Runtime>::is_active_account_admin(
                institution_code,
                admin_root_account_id,
                who,
            );
        }
        if admin_primitives::is_public_admin_code(&institution_code) {
            return public_admins::Pallet::<Runtime>::is_active_account_admin(
                institution_code,
                admin_root_account_id,
                who,
            );
        }
        if admin_primitives::is_private_admin_code(&institution_code) {
            return private_admins::Pallet::<Runtime>::is_active_account_admin(
                institution_code,
                admin_root_account_id,
                who,
            );
        }
        false
    }

    fn active_account_admins(
        institution_code: primitives::code::InstitutionCode,
        admin_root_account_id: AccountId,
    ) -> Option<Vec<AccountId>> {
        if admin_primitives::is_personal_admin_code(&institution_code) {
            return personal_admins::Pallet::<Runtime>::active_account_admins(
                institution_code,
                admin_root_account_id,
            );
        }
        if admin_primitives::is_genesis_admin_code(&institution_code) {
            return genesis_admins::Pallet::<Runtime>::active_account_admins(
                institution_code,
                admin_root_account_id,
            );
        }
        if admin_primitives::is_public_admin_code(&institution_code) {
            return public_admins::Pallet::<Runtime>::active_account_admins(
                institution_code,
                admin_root_account_id,
            );
        }
        if admin_primitives::is_private_admin_code(&institution_code) {
            return private_admins::Pallet::<Runtime>::active_account_admins(
                institution_code,
                admin_root_account_id,
            );
        }
        None
    }

    fn active_account_admins_len(
        institution_code: primitives::code::InstitutionCode,
        admin_root_account_id: AccountId,
    ) -> Option<u32> {
        if admin_primitives::is_personal_admin_code(&institution_code) {
            return personal_admins::Pallet::<Runtime>::active_account_admins_len(
                institution_code,
                admin_root_account_id,
            );
        }
        if admin_primitives::is_genesis_admin_code(&institution_code) {
            return genesis_admins::Pallet::<Runtime>::active_account_admins_len(
                institution_code,
                admin_root_account_id,
            );
        }
        if admin_primitives::is_public_admin_code(&institution_code) {
            return public_admins::Pallet::<Runtime>::active_account_admins_len(
                institution_code,
                admin_root_account_id,
            );
        }
        if admin_primitives::is_private_admin_code(&institution_code) {
            return private_admins::Pallet::<Runtime>::active_account_admins_len(
                institution_code,
                admin_root_account_id,
            );
        }
        None
    }

    fn pending_account_exists_for_snapshot(
        institution_code: primitives::code::InstitutionCode,
        admin_root_account_id: AccountId,
    ) -> bool {
        Self::pending_account_admins_len_for_snapshot(institution_code, admin_root_account_id)
            .is_some()
    }

    fn is_pending_account_admin_for_snapshot(
        institution_code: primitives::code::InstitutionCode,
        admin_root_account_id: AccountId,
        who: &AccountId,
    ) -> bool {
        if admin_primitives::is_personal_admin_code(&institution_code) {
            return personal_admins::Pallet::<Runtime>::is_pending_account_admin_for_snapshot(
                institution_code,
                admin_root_account_id,
                who,
            );
        }
        if admin_primitives::is_genesis_admin_code(&institution_code) {
            return genesis_admins::Pallet::<Runtime>::is_pending_account_admin_for_snapshot(
                institution_code,
                admin_root_account_id,
                who,
            );
        }
        if admin_primitives::is_public_admin_code(&institution_code) {
            return public_admins::Pallet::<Runtime>::is_pending_account_admin_for_snapshot(
                institution_code,
                admin_root_account_id,
                who,
            );
        }
        if admin_primitives::is_private_admin_code(&institution_code) {
            return private_admins::Pallet::<Runtime>::is_pending_account_admin_for_snapshot(
                institution_code,
                admin_root_account_id,
                who,
            );
        }
        false
    }

    fn pending_account_admins_for_snapshot(
        institution_code: primitives::code::InstitutionCode,
        admin_root_account_id: AccountId,
    ) -> Option<Vec<AccountId>> {
        if admin_primitives::is_personal_admin_code(&institution_code) {
            return personal_admins::Pallet::<Runtime>::pending_account_admins_for_snapshot(
                institution_code,
                admin_root_account_id,
            );
        }
        if admin_primitives::is_genesis_admin_code(&institution_code) {
            return genesis_admins::Pallet::<Runtime>::pending_account_admins_for_snapshot(
                institution_code,
                admin_root_account_id,
            );
        }
        if admin_primitives::is_public_admin_code(&institution_code) {
            return public_admins::Pallet::<Runtime>::pending_account_admins_for_snapshot(
                institution_code,
                admin_root_account_id,
            );
        }
        if admin_primitives::is_private_admin_code(&institution_code) {
            return private_admins::Pallet::<Runtime>::pending_account_admins_for_snapshot(
                institution_code,
                admin_root_account_id,
            );
        }
        None
    }

    fn pending_account_admins_len_for_snapshot(
        institution_code: primitives::code::InstitutionCode,
        admin_root_account_id: AccountId,
    ) -> Option<u32> {
        if admin_primitives::is_personal_admin_code(&institution_code) {
            return personal_admins::Pallet::<Runtime>::pending_account_admins_len_for_snapshot(
                institution_code,
                admin_root_account_id,
            );
        }
        if admin_primitives::is_genesis_admin_code(&institution_code) {
            return genesis_admins::Pallet::<Runtime>::pending_account_admins_len_for_snapshot(
                institution_code,
                admin_root_account_id,
            );
        }
        if admin_primitives::is_public_admin_code(&institution_code) {
            return public_admins::Pallet::<Runtime>::pending_account_admins_len_for_snapshot(
                institution_code,
                admin_root_account_id,
            );
        }
        if admin_primitives::is_private_admin_code(&institution_code) {
            return private_admins::Pallet::<Runtime>::pending_account_admins_len_for_snapshot(
                institution_code,
                admin_root_account_id,
            );
        }
        None
    }

    fn legal_representative(
        institution_code: primitives::code::InstitutionCode,
        admin_root_account_id: AccountId,
    ) -> Option<AccountId> {
        if admin_primitives::is_genesis_admin_code(&institution_code) {
            return genesis_admins::Pallet::<Runtime>::legal_representative(
                institution_code,
                admin_root_account_id,
            );
        }
        if admin_primitives::is_public_admin_code(&institution_code) {
            return public_admins::Pallet::<Runtime>::legal_representative(
                institution_code,
                admin_root_account_id,
            );
        }
        if admin_primitives::is_private_admin_code(&institution_code) {
            return private_admins::Pallet::<Runtime>::legal_representative(
                institution_code,
                admin_root_account_id,
            );
        }
        None
    }
}

pub struct RuntimeAdminVoteExecutor;

impl votingengine::InternalVoteResultCallback for RuntimeAdminVoteExecutor {
    fn on_internal_vote_finalized(
        proposal_id: u64,
        approved: bool,
    ) -> Result<votingengine::ProposalExecutionOutcome, sp_runtime::DispatchError> {
        let callbacks = [
            genesis_admins::InternalVoteExecutor::<Runtime>::on_internal_vote_finalized(
                proposal_id,
                approved,
            )?,
            public_admins::InternalVoteExecutor::<Runtime>::on_internal_vote_finalized(
                proposal_id,
                approved,
            )?,
            private_admins::InternalVoteExecutor::<Runtime>::on_internal_vote_finalized(
                proposal_id,
                approved,
            )?,
        ];
        if callbacks
            .iter()
            .any(|outcome| *outcome != votingengine::ProposalExecutionOutcome::Ignored)
        {
            return Ok(votingengine::ProposalExecutionOutcome::Executed);
        }
        Ok(votingengine::ProposalExecutionOutcome::Ignored)
    }
}

// ---------------------------------------------------------------------------
// 链下交易清算模块配置
// ---------------------------------------------------------------------------

/// CID 机构登记表查询实现。
///
/// 委托给 `organization-manage` 的 CID 地址索引和机构账户表；
/// 管理员校验再统一转给 `admins 模块::AdminAccounts`。
pub struct MultisigCidAccountQuery;

impl offchain_transaction::bank_check::CidAccountQuery<AccountId> for MultisigCidAccountQuery {
    fn account_info(addr: &AccountId) -> Option<(Vec<u8>, Vec<u8>)> {
        organization_manage::AccountRegisteredCid::<Runtime>::get(addr)
            .map(|info| (info.cid_number.to_vec(), info.account_name.to_vec()))
    }

    fn find_account(cid_number: &[u8], account_name: &[u8]) -> Option<AccountId> {
        let id: organization_manage::CidNumberOf<Runtime> = cid_number.to_vec().try_into().ok()?;
        let an: organization_manage::AccountNameOf<Runtime> =
            account_name.to_vec().try_into().ok()?;
        organization_manage::CidRegisteredAccount::<Runtime>::get(&id, &an)
    }

    fn is_active(addr: &AccountId) -> bool {
        if let Some(registered) = organization_manage::AccountRegisteredCid::<Runtime>::get(addr) {
            return matches!(
                organization_manage::InstitutionAccounts::<Runtime>::get(
                    &registered.cid_number,
                    &registered.account_name,
                )
                .map(|a| a.status),
                Some(organization_manage::InstitutionLifecycleStatus::Active)
            );
        }

        // 个人多签状态查询走 personal-admins::PersonalAccounts。
        matches!(
            personal_admins::PersonalAccounts::<Runtime>::get(addr).map(|a| a.status),
            Some(personal_admins::PersonalStatus::Active)
        )
    }

    /// 判定 `who` 是否是 `bank` 多签账户的管理员之一。
    /// 用于费率提案 / 批次提交等治理动作的身份校验。
    ///
    /// 中文注释:机构账户按自身地址作为治理账户,institution_code 来自
    /// `Institutions[cid].institution_code`;PMUL 只给 personal-admins 使用。
    fn is_admin_of(bank: &AccountId, who: &AccountId) -> bool {
        let Some(account) =
            organization_manage::Pallet::<Runtime>::resolve_admin_account_for_account(bank)
        else {
            return false;
        };
        let Some(institution_code) =
            organization_manage::Pallet::<Runtime>::resolve_institution_code_for_account(bank)
        else {
            return false;
        };
        RuntimeAdminAccountQuery::is_active_account_admin(institution_code, account, who)
    }

    /// 清算行资格由 CID 系统的 eligible-search 负责筛选。
    /// 链上不保存 subject_property/sub_type/parent_cid_number,这里只确认该地址属于已注册且 Active 的
    /// CID 机构账户,避免把 CID 内部机构类型字段重复落到链上。
    fn is_clearing_bank_eligible(addr: &AccountId) -> bool {
        let registered = match organization_manage::AccountRegisteredCid::<Runtime>::get(addr) {
            Some(info) => info,
            None => return false,
        };
        matches!(
            organization_manage::InstitutionAccounts::<Runtime>::get(
                &registered.cid_number,
                &registered.account_name,
            )
            .map(|account| account.status),
            Some(organization_manage::InstitutionLifecycleStatus::Active)
        )
    }

    /// 判定 `bank` 主账户对应的机构是否
    /// 已声明为清算行节点(链上 `ClearingBankNodes` 存在该 cid_number 记录)。
    fn is_registered_clearing_node(bank: &AccountId) -> bool {
        let registered = match organization_manage::AccountRegisteredCid::<Runtime>::get(bank) {
            Some(info) => info,
            None => return false,
        };
        // ClearingBankNodes 的 key 是 BoundedVec<u8, ConstU32<64>>,
        // 把 CidNumberOf<Runtime>(BoundedVec<u8, MaxCidNumberLength=CID_NUMBER_MAX_BYTES>) 转换过去
        let cid_bytes: Vec<u8> = registered.cid_number.to_vec();
        let key: BoundedVec<u8, ConstU32<64>> = match cid_bytes.try_into() {
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
    type CidAccountQuery = MultisigCidAccountQuery;
    type WeightInfo = offchain_transaction::weights::SubstrateWeight<Runtime>;
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

pub(crate) fn is_nrc_admin(who: &AccountId) -> bool {
    let nrc_institution = primitives::china::china_cb::CHINA_CB
        .first()
        .map(|n| AccountId::new(n.main_account))
        .expect("NRC main_account must exist");

    // 中文注释：创世后只信任链上管理员治理模块中的统一账户表。
    RuntimeAdminAccountQuery::is_active_account_admin(
        votingengine::types::NRC,
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
        let admin = AccountId::new(primitives::china::china_cb::CHINA_CB[0].admins[0]);
        Ok(RuntimeOrigin::from(frame_system::RawOrigin::Signed(admin)))
    }
}

/// 国储会和省储会管理员均可发起联合提案（含运行时升级、决议发行等）。
fn is_joint_proposer(who: &AccountId) -> bool {
    use primitives::china::china_cb::CHINA_CB;
    for (idx, entry) in CHINA_CB.iter().enumerate() {
        let institution = AccountId::new(entry.main_account);
        let institution_code = if idx == 0 {
            votingengine::types::NRC
        } else {
            votingengine::types::PRC
        };
        if RuntimeAdminAccountQuery::is_active_account_admin(institution_code, institution, who) {
            return true;
        }
    }
    false
}

impl resolution_issuance::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type ProposeOrigin = EnsureJointProposer;
    type RecipientSetOrigin = frame_system::EnsureRoot<AccountId>;
    // 中文注释：维护入口只允许 root 操作暂停与短期执行记录清理。
    type MaintenanceOrigin = frame_system::EnsureRoot<AccountId>;
    type WeightInfo = resolution_issuance::weights::SubstrateWeight<Runtime>;
    type JointVoteEngine = JointVote;
    type MaxReasonLen = ResolutionIssuanceMaxReasonLen;
    type MaxAllocations = ResolutionIssuanceMaxAllocations;
    type MaxTotalIssuance = ResolutionIssuanceMaxTotalIssuance;
    type MaxSingleIssuance = ResolutionIssuanceMaxSingleIssuance;
}

impl runtime_upgrade::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type ProposeOrigin = EnsureJointProposer;
    type DeveloperUpgradeOrigin = EnsureNrcAdmin;
    type JointVoteEngine = JointVote;
    type RuntimeCodeExecutor = RuntimeSetCodeExecutor;
    type DeveloperUpgradeCheck = GenesisPallet;
    type MaxReasonLen = RuntimeUpgradeMaxReasonLen;
    type MaxRuntimeCodeSize = RuntimeUpgradeMaxCodeSize;
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

parameter_types! {
    // 立法院模块边界常量(ADR-027,第1步)
    pub const LegislationMaxTitleLen: u32 = 256;
    pub const LegislationMaxTextLen: u32 = 8192; // 条/款正文(宪法部分条较长)
    pub const LegislationMaxClausesPerArticle: u32 = 50;
    pub const LegislationMaxArticlesPerSection: u32 = 200;
    pub const LegislationMaxSectionsPerChapter: u32 = 50;
    pub const LegislationMaxChaptersPerLaw: u32 = 50;
    pub const LegislationMaxLawsPerScope: u32 = 1000;
    pub const LegislationMaxActivationsPerBlock: u32 = 100;
}

impl legislation_yuan::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    // 立法投票引擎接真实 legislation-vote sub-pallet(ADR-027 第2步),投票端到端流程打通。
    type LegislationVoteEngine = LegislationVote;
    type MaxTitleLen = LegislationMaxTitleLen;
    type MaxTextLen = LegislationMaxTextLen;
    type MaxClausesPerArticle = LegislationMaxClausesPerArticle;
    type MaxArticlesPerSection = LegislationMaxArticlesPerSection;
    type MaxSectionsPerChapter = LegislationMaxSectionsPerChapter;
    type MaxChaptersPerLaw = LegislationMaxChaptersPerLaw;
    type MaxLawsPerScope = LegislationMaxLawsPerScope;
    type MaxActivationsPerBlock = LegislationMaxActivationsPerBlock;
    type WeightInfo = ();
}

pub struct RuntimeJointVoteResultCallback;

impl votingengine::JointVoteResultCallback for RuntimeJointVoteResultCallback {
    fn on_joint_vote_finalized(
        vote_proposal_id: u64,
        approved: bool,
    ) -> Result<votingengine::ProposalExecutionOutcome, sp_runtime::DispatchError> {
        #[cfg(feature = "runtime-benchmarks")]
        {
            let _ = (vote_proposal_id, approved);
            Ok(votingengine::ProposalExecutionOutcome::Ignored)
        }

        #[cfg(not(feature = "runtime-benchmarks"))]
        {
            if resolution_issuance::Pallet::<Runtime>::owns_proposal(vote_proposal_id) {
                return <ResolutionIssuance as votingengine::JointVoteResultCallback>::on_joint_vote_finalized(
                vote_proposal_id,
                approved,
            );
            }

            if runtime_upgrade::Pallet::<Runtime>::owns_proposal(vote_proposal_id) {
                return <RuntimeUpgrade as votingengine::JointVoteResultCallback>::on_joint_vote_finalized(
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

impl votingengine::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxVoteNonceLength = ConstU32<64>;
    type MaxVoteSignatureLength = ConstU32<64>;
    type MaxAutoFinalizePerBlock = ConstU32<2_048>;
    type MaxProposalsPerExpiry = ConstU32<2_048>;
    type MaxInternalProposalMutexBindings = ConstU32<256>;
    type MaxActiveProposals = ConstU32<10>;
    type MaxProposalDataLen = ConstU32<{ 100 * 1024 }>;
    type MaxProposalObjectLen = ConstU32<{ 10 * 1024 * 1024 }>;
    type MaxModuleTagLen = ConstU32<32>;
    type MaxManualExecutionAttempts = ConstU32<3>;
    type ExecutionRetryGraceBlocks = VotingExecutionRetryGraceBlocks;
    type MaxExecutionRetryDeadlinesPerBlock = ConstU32<2_048>;
    type MaxPendingRetryExpirationsPerBlock = ConstU32<256>;
    type MaxCleanupStepsPerBlock = ConstU32<8>;
    type MaxCleanupQueueBucketLimit = ConstU32<512>;
    type MaxCleanupScheduleOffset = ConstU32<1_024>;
    type CleanupKeysPerStep = ConstU32<256>;
    type CidEligibility = RuntimeCidEligibility;
    type PopulationSnapshotVerifier = RuntimePopulationSnapshotVerifier;
    type JointVoteResultCallback = RuntimeJointVoteResultCallback;
    // 内部投票终态回调注册 5 个业务 Executor。
    // 顺序按调用频率降序:transfer / multisig manage 类业务最频繁,
    // grandpa key 替换最稀有放最后(tuple iterate 时命中越早越省 gas)。
    // 每个 Executor 通过 MODULE_TAG 前缀 + 独立存储键互斥认领本模块提案,
    // 非己方提案直接 Ok(()) skip,顺序不影响行为正确性。
    type InternalVoteResultCallback = (
        multisig_transfer::InternalVoteExecutor<Runtime>,
        organization_manage::InternalVoteExecutor<Runtime>,
        personal_admins::InternalVoteExecutor<Runtime>,
        RuntimeAdminVoteExecutor,
        resolution_destro::InternalVoteExecutor<Runtime>,
        grandpakey_change::InternalVoteExecutor<Runtime>,
    );
    type InternalAdminProvider = RuntimeInternalAdminProvider;
    type InternalAdminsLenProvider = RuntimeInternalAdminsLenProvider;
    type MaxAdminsPerInstitution = MaxAdminsPerInstitution;
    type TimeProvider = pallet_timestamp::Pallet<Runtime>;
    type WeightInfo = votingengine::weights::SubstrateWeight<Runtime>;
    // mode-specific finalize / cleanup 通过 trait 派发到对应 sub-pallet。
    type InternalFinalizer = InternalVote;
    type InternalCleanup = InternalVote;
    type JointFinalizer = JointVote;
    type JointCleanup = JointVote;
    // 立法投票(ADR-027):终态回调接业务壳 legislation-yuan;超时结算/清理接 legislation-vote。
    type LegislationVoteResultCallback = LegislationYuan;
    type LegislationFinalizer = LegislationVote;
    type LegislationCleanup = LegislationVote;
}

// Sub-pallet Config 注入。共用基础设施 votingengine::Config 已 impl 完;
// sub-pallet 各自 Config 需 RuntimeEvent + 自家 WeightInfo。
impl internal_vote::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = internal_vote::weights::SubstrateWeight<Runtime>;
}

impl joint_vote::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = joint_vote::weights::SubstrateWeight<Runtime>;
}

impl election_vote::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
}

impl legislation_vote::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
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

pub struct RuntimeInternalAdminProvider;

impl votingengine::InternalAdminProvider<AccountId> for RuntimeInternalAdminProvider {
    fn is_internal_admin(
        institution_code: votingengine::types::InstitutionCode,
        institution: AccountId,
        who: &AccountId,
    ) -> bool {
        RuntimeAdminAccountQuery::is_active_account_admin(institution_code, institution, who)
    }

    fn get_admin_list(
        institution_code: votingengine::types::InstitutionCode,
        institution: AccountId,
    ) -> Option<alloc::vec::Vec<AccountId>> {
        RuntimeAdminAccountQuery::active_account_admins(institution_code, institution)
    }

    fn is_pending_internal_admin(
        institution_code: votingengine::types::InstitutionCode,
        institution: AccountId,
        who: &AccountId,
    ) -> bool {
        RuntimeAdminAccountQuery::is_pending_account_admin_for_snapshot(
            institution_code,
            institution,
            who,
        )
    }

    fn get_pending_admin_list(
        institution_code: votingengine::types::InstitutionCode,
        institution: AccountId,
    ) -> Option<alloc::vec::Vec<AccountId>> {
        RuntimeAdminAccountQuery::pending_account_admins_for_snapshot(institution_code, institution)
    }

    fn legal_representative(
        institution_code: votingengine::types::InstitutionCode,
        institution: AccountId,
    ) -> Option<AccountId> {
        RuntimeAdminAccountQuery::legal_representative(institution_code, institution)
    }
}

pub struct RuntimeInternalAdminsLenProvider;

impl votingengine::InternalAdminsLenProvider<AccountId> for RuntimeInternalAdminsLenProvider {
    fn admins_len(
        institution_code: votingengine::types::InstitutionCode,
        institution: AccountId,
    ) -> Option<u32> {
        RuntimeAdminAccountQuery::active_account_admins_len(institution_code, institution)
    }
}

pub struct RuntimeCidEligibility;

impl votingengine::CidEligibility<AccountId, Hash> for RuntimeCidEligibility {
    fn is_eligible(binding_id: &Hash, who: &AccountId) -> bool {
        #[cfg(feature = "runtime-benchmarks")]
        {
            let _ = (
                who,
                cid_system::pallet::BindingIdToAccount::<Runtime>::get(binding_id),
            );
            true
        }

        #[cfg(not(feature = "runtime-benchmarks"))]
        {
            <cid_system::Pallet<Runtime> as cid_system::CidEligibilityProvider<
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
        issuer_cid_number: &[u8],
        issuer_main_account: &AccountId,
        signer_pubkey: &[u8; 32],
        scope_province_name: &[u8],
        scope_city_name: &[u8],
    ) -> bool {
        #[cfg(feature = "runtime-benchmarks")]
        {
            let _ = (
                who,
                issuer_cid_number,
                issuer_main_account,
                signer_pubkey,
                scope_province_name,
                scope_city_name,
            );
            if nonce.is_empty() || signature.is_empty() {
                return false;
            }

            let nonce_hash = <Runtime as frame_system::Config>::Hashing::hash_of(&nonce);
            let vote_nonce_key = (binding_id.clone(), nonce_hash);
            if cid_system::pallet::UsedVoteNonce::<Runtime>::get(
                proposal_id,
                vote_nonce_key.clone(),
            ) {
                return false;
            }

            cid_system::pallet::UsedVoteNonce::<Runtime>::insert(proposal_id, vote_nonce_key, true);
            true
        }

        #[cfg(not(feature = "runtime-benchmarks"))]
        {
            <cid_system::Pallet<Runtime> as cid_system::CidEligibilityProvider<
                AccountId,
                Hash,
            >>::verify_and_consume_vote_credential(
                binding_id,
                who,
                proposal_id,
                nonce,
                signature,
                issuer_cid_number,
                issuer_main_account,
                signer_pubkey,
                scope_province_name,
                scope_city_name,
            )
        }
    }

    fn cleanup_vote_credentials(proposal_id: u64) {
        <cid_system::Pallet<Runtime> as cid_system::CidEligibilityProvider<
            AccountId,
            Hash,
        >>::cleanup_vote_credentials(proposal_id)
    }

    fn cleanup_vote_credentials_chunk(
        proposal_id: u64,
        limit: u32,
    ) -> votingengine::VoteCredentialCleanup {
        let result =
            cid_system::pallet::UsedVoteNonce::<Runtime>::clear_prefix(proposal_id, limit, None);
        votingengine::VoteCredentialCleanup {
            removed: result.unique,
            loops: result.loops,
            has_remaining: result.maybe_cursor.is_some(),
        }
    }
}

// =====================================================================
// pallet_assets 内核接入(ADR-011 第八节)+ OnchainIssuance 外壳配置
// =====================================================================
//
// 中文注释:pallet_assets 是用户代币的内核 storage / 资产记账实现,
// **所有原生 extrinsic 在 RuntimeCallFilter 中 reject**。
// 业务调用必须经由 OnchainIssuance::propose_* → InternalVote/JointVote callback →
// onchain_issuance 内部以 Root 调用 pallet_assets 的内核 API。
//
// 第一期 deposit 系列常量统一为 0(框架阶段),业务实装时再据 ADR-011 调整。
// 押金语义与 GMB 1000 元创建费正交,后者通过 onchain_issuance::fee::charge_creation_fee 直接走 GMB 转账,
// 不复用 pallet_assets 自身的 deposit 机制。

parameter_types! {
    /// 资产 metadata 字符串字段长度上限(name / symbol / description),
    /// 与 onchain_issuance::Config::MaxAssetNameLen 等参数对齐。
    pub const AssetsStringLimit: u32 = 64;
    /// 单批 destroy 时一次清理的账户/审批上限。
    pub const AssetsRemoveItemsLimit: u32 = 1000;
    /// pallet_assets 自身 deposit 系列常量(均设 0,真实计费走 onchain_issuance::fee)。
    pub const AssetsDepositZero: Balance = 0;
}

impl pallet_assets::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type RemoveItemsLimit = AssetsRemoveItemsLimit;
    type AssetId = u32;
    type AssetIdParameter = codec::Compact<u32>;
    type Currency = Balances;
    /// 中文注释:外部 extrinsic 全部被 RuntimeCallFilter reject,这里 origin 设啥不影响实际入口。
    /// CreateOrigin 接 EnsureSigned 仅为满足 trait Success=AccountId 约束;
    /// ForceOrigin 接 EnsureRoot(Success=())。OnchainIssuance 内部经 fungibles trait
    /// (Create / Mutate)直接调内核 API,不走 extrinsic origin 路径。
    type CreateOrigin =
        frame_support::traits::AsEnsureOriginWithArg<frame_system::EnsureSigned<AccountId>>;
    type ForceOrigin = frame_system::EnsureRoot<AccountId>;
    type AssetDeposit = AssetsDepositZero;
    type AssetAccountDeposit = AssetsDepositZero;
    type MetadataDepositBase = AssetsDepositZero;
    type MetadataDepositPerByte = AssetsDepositZero;
    type ApprovalDeposit = AssetsDepositZero;
    type StringLimit = AssetsStringLimit;
    type Freezer = ();
    type Holder = ();
    type Extra = ();
    type CallbackHandle = ();
    type ReserveData = ();
    type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = ();
}

/// OnchainIssuance pallet 配置(NRC 账户 / 费用账户语义分离)。
///
/// 中文注释:onchain-issuance 拆为两个独立 trait:
/// - `NrcMainAccountProvider` → 返回 NRC 治理多签账户 main_account(monitor 调用方校验用)
/// - `NrcFeeAccountProvider`  → 返回 NRC 费用账户 fee_account(创建费收款用)
/// v1 错误地复用 onchain_transaction::NrcAccountProvider(它返回 fee_account),
/// 导致 monitor 账户身份语义错。

/// NRC 治理多签账户(main_account)— monitor / 监管动作发起方校验用。
pub struct RuntimeNrcMainAccountProvider;

impl onchain_issuance::pallet::NrcMainAccountProvider<AccountId> for RuntimeNrcMainAccountProvider {
    fn nrc_main_account() -> Option<AccountId> {
        // 中文注释:china_cb[0].main_account 是 NRC 治理多签账户,与 fee_account 不同。
        primitives::china::china_cb::CHINA_CB
            .first()
            .and_then(|n| AccountId::decode(&mut &n.main_account[..]).ok())
    }
}

/// NRC 费用账户(fee_account)— 创建费 1000 GMB 收款用。
///
/// 中文注释:复用既有 `RuntimeNrcAccountProvider`(它实现 onchain_transaction::NrcAccountProvider,
/// 也返回 fee_account),通过为同 struct 再实现 onchain_issuance 自己的 trait 完成桥接,语义一致。
impl onchain_issuance::pallet::NrcFeeAccountProvider<AccountId> for RuntimeNrcAccountProvider {
    fn nrc_fee_account() -> Option<AccountId> {
        <RuntimeNrcAccountProvider as onchain_transaction::NrcAccountProvider<AccountId>>::nrc_account()
    }
}

parameter_types! {
    pub const OnchainAssetMaxNameLen: u32 = 64;
    pub const OnchainAssetMaxSymbolLen: u32 = 16;
    pub const OnchainAssetMaxDescriptionLen: u32 = 256;
    pub const OnchainAssetMaxBlacklistWordLen: u32 = 32;
    pub const OnchainAssetMaxBlacklistEntries: u32 = 256;
    pub const OnchainAssetReasonHashLen: u32 = 32;
    pub const OnchainAssetMaxScheduledPerBlock: u32 = 64;
}

impl onchain_issuance::pallet::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    /// 中文注释:Currency 必须实现 ReservableCurrency(ADR-011 v2 第六节押金机制),
    /// pallet_balances 默认实现该 trait,直接接 Balances 即可。
    type Currency = Balances;
    /// pallet_assets 内核类型绑定。onchain_issuance 通过该类型调内核 create / mint_into 等内部 API,
    /// 不走原生 extrinsic(已被 RuntimeCallFilter 拦截)。
    type Assets = Assets;
    type NrcMainAccountProvider = RuntimeNrcMainAccountProvider;
    type NrcFeeAccountProvider = RuntimeNrcAccountProvider;
    type MaxAssetNameLen = OnchainAssetMaxNameLen;
    type MaxAssetSymbolLen = OnchainAssetMaxSymbolLen;
    type MaxAssetDescriptionLen = OnchainAssetMaxDescriptionLen;
    type MaxBlacklistWordLen = OnchainAssetMaxBlacklistWordLen;
    type MaxBlacklistEntries = OnchainAssetMaxBlacklistEntries;
    type ReasonHashLen = OnchainAssetReasonHashLen;
    type MaxScheduledPerBlock = OnchainAssetMaxScheduledPerBlock;
    type WeightInfo = onchain_issuance::weights::ZeroWeight;
}
