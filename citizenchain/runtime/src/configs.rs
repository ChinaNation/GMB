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
use admin_primitives::{AdminAccountQuery, InstitutionAdminQuery};
use alloc::vec::Vec;
use codec::Decode;
use entity_primitives::InstitutionMultisigQuery;
use entity_primitives::InstitutionRoleQuery;
#[cfg(not(feature = "runtime-benchmarks"))]
use frame_support::traits::UnfilteredDispatchable;
use frame_support::{
    derive_impl,
    dispatch::DispatchResult,
    parameter_types,
    traits::{
        fungible::{Balanced, Credit},
        ConstU128, ConstU32, ConstU64, ConstU8, Contains, EnsureOrigin, FindAuthor, OnUnbalanced,
        VariantCountOf,
    },
    weights::{
        constants::{RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND},
        ConstantMultiplier, Weight,
    },
};
use frame_system::limits::{BlockLength, BlockWeights};
use onchain::NrcAccountProvider as _;
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
    AccountId, Assets, Balance, Balances, Block, BlockNumber, CitizenIssuance, ElectionVote,
    GenesisPallet, Hash, InternalVote, JointVote, LegislationVote, LegislationYuan, Nonce,
    PalletInfo, PrivateAdmins, PrivateManage, PublicAdmins, PublicManage, Runtime, RuntimeCall,
    RuntimeEvent, RuntimeFreezeReason, RuntimeHoldReason, RuntimeOrigin, RuntimeTask, System,
    BLOCK_HASH_COUNT, EXISTENTIAL_DEPOSIT, VERSION,
};
#[cfg(not(feature = "runtime-benchmarks"))]
use super::{ResolutionIssuance, RuntimeUpgrade};

const NORMAL_DISPATCH_RATIO: Perbill =
    Perbill::from_percent(primitives::core_const::NORMAL_DISPATCH_PERCENT);

parameter_types! {
    pub const BlockHashCount: BlockNumber = BLOCK_HASH_COUNT;
    /// 使用 BlockNumber 类型声明重试宽限期，避免与具体 u32 常量类型耦合。
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
/// 本链全新创世,无历史链上数据需迁移,故为空。
/// 将来链上线后如需单块迁移,在此 tuple 挂入 `OnRuntimeUpgrade` 即可。
#[allow(unused_parens)]
type SingleBlockMigrations = ();

pub fn is_stake_account(address: &AccountId) -> bool {
    primitives::cid::china::china_ch::CHINA_CH
        .iter()
        .any(|n| address == &AccountId::new(n.stake_account))
}

fn is_reserved_fee_account(address: &AccountId) -> bool {
    primitives::cid::china::china_ch::CHINA_CH
        .iter()
        .any(|n| address == &AccountId::new(n.fee_account))
}

/// 检查是否为国家储委会安全基金账户。
fn is_safety_fund_account(address: &AccountId) -> bool {
    address == &AccountId::new(primitives::cid::china::china_cb::SAFETY_FUND_ACCOUNT)
}

/// 检查是否为国家储委会两和基金账户。
fn is_nrc_he_account(address: &AccountId) -> bool {
    address == &AccountId::new(primitives::cid::china::china_cb::NRC_HE_ACCOUNT)
}

/// 检查是否为储委会费用账户（44 个机构的 fee_account）。
fn is_cb_fee_account(address: &AccountId) -> bool {
    primitives::cid::china::china_cb::CHINA_CB
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
    primitives::cid::china::china_zb::is_reserved_main_account(&addr)
}

pub struct RuntimeCallFilter;

impl Contains<RuntimeCall> for RuntimeCallFilter {
    fn contains(call: &RuntimeCall) -> bool {
        match call {
            // Balances 只作为底层余额账本和内部 Currency 能力保留。
            // 外部单账户链上转账唯一入口是 OnchainTransaction::transfer_with_remark。
            RuntimeCall::Balances(_) => false,
            // ADR-011 铁律:pallet_assets 内核所有原生 extrinsic 一律 reject。
            // 业务调用必须经由 OnchainIssuance::propose_* → InternalVote/JointVote callback → 内部 root 调用。
            // 任何外部 extrinsic 直接打到 pallet_assets 全部不入块,
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
    /// 全局调用过滤器，禁止 stake_account 参与 force_* 余额调用，并封禁强制总发行量调整入口。
    type BaseCallFilter = RuntimeCallFilter;
    type MaxConsumers = frame_support::traits::ConstU32<16>;
    type SingleBlockMigrations = SingleBlockMigrations;
}

impl pallet_timestamp::Config for Runtime {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    // 纯 PoW 共识：时间戳不再依赖 Aura 插槽回调。
    type OnTimestampSet = ();
    // PoW 找到即出块；这里只要求时间戳至少递增 1ms，不用时间戳人为节流。
    type MinimumPeriod = ConstU64<1>;
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
    // 保留最近若干 set_id 与会话映射，便于后续接入等值投票追溯/举报能力。
    pub const MaxSetIdSessionEntries: u64 = 128;
}

impl pallet_grandpa::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type MaxAuthorities = MaxGrandpaAuthorities;
    type MaxNominators = MaxGrandpaNominators;
    type MaxSetIdSessionEntries = MaxSetIdSessionEntries;
    // 当前版本不启用链上等值投票惩罚（无 session/historical 证明体系）。
    // 但保留 MaxSetIdSessionEntries 以便后续平滑接入。
    type KeyOwnerProof = Void;
    type EquivocationReportSystem = ();
}

parameter_types! {
    pub FeeMultiplier: Multiplier = Multiplier::one();
}

impl pallet_transaction_payment::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnChargeTransaction = onchain::OnchainChargeAdapter<
        Balances,
        onchain::OnchainFeeRouter<
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

impl onchain::pallet::Config for Runtime {
    type Currency = Balances;
    type MaxTransferRemarkLen = ConstU32<99>;
}

pub struct RuntimeNrcAccountProvider;

impl onchain::NrcAccountProvider<AccountId> for RuntimeNrcAccountProvider {
    fn nrc_account() -> Option<AccountId> {
        Some(AccountId::new(
            primitives::cid::china::china_cb::CHINA_CB[0].fee_account,
        ))
    }
}

pub struct RuntimeSafetyFundAccountProvider;

impl onchain::SafetyFundAccountProvider<AccountId> for RuntimeSafetyFundAccountProvider {
    fn safety_fund_account() -> AccountId {
        AccountId::new(primitives::cid::china::china_cb::SAFETY_FUND_ACCOUNT)
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

impl onchain::CallFeeKind<AccountId, RuntimeCall, Balance> for RuntimeFeeKindClassifier {
    fn fee_kind(_who: &AccountId, call: &RuntimeCall) -> onchain::FeeChargeKind<Balance> {
        use onchain::FeeChargeKind;

        match call {
            RuntimeCall::OnchainTransaction(onchain::pallet::Call::transfer_with_remark {
                amount,
                ..
            }) => FeeChargeKind::OnchainAmount(*amount),
            RuntimeCall::OnchainTransaction(_) => FeeChargeKind::Unknown,
            // PersonalManage 的 propose_create/propose_close 是治理提案交易，
            // 交易本身固定收 1 元；执行阶段的资金手续费由对应 pallet 内部按金额另行处理。
            RuntimeCall::PersonalManage(personal_manage::pallet::Call::propose_create {
                ..
            })
            | RuntimeCall::PersonalManage(personal_manage::pallet::Call::propose_close {
                ..
            }) => FeeChargeKind::VoteFlat,
            RuntimeCall::PersonalAdmins(_) => FeeChargeKind::VoteFlat,
            RuntimeCall::PersonalManage(_) => FeeChargeKind::VoteFlat,
            RuntimeCall::PublicManage(
                public_manage::pallet::Call::propose_create_public_institution { .. },
            )
            | RuntimeCall::PublicManage(
                public_manage::pallet::Call::propose_close_public_institution { .. },
            )
            | RuntimeCall::PublicManage(_) => FeeChargeKind::VoteFlat,
            RuntimeCall::PrivateManage(
                private_manage::pallet::Call::propose_create_private_institution { .. },
            )
            | RuntimeCall::PrivateManage(
                private_manage::pallet::Call::propose_close_private_institution { .. },
            )
            | RuntimeCall::PrivateManage(_) => FeeChargeKind::VoteFlat,
            RuntimeCall::AddressRegistry(_) => FeeChargeKind::VoteFlat,
            // 免费调用交易：系统内部 / 自动化类。省储行固定利息已无公开 Call，
            // 只在年度边界 finalize 自动执行，因此不再占用交易费分类分支。
            RuntimeCall::System(_) => FeeChargeKind::Free,
            RuntimeCall::Timestamp(_) => FeeChargeKind::Free,
            RuntimeCall::CitizenIssuance(_) => FeeChargeKind::Free,
            // GRANDPA pallet:report_equivocation(签名版)/ report_equivocation_unsigned(unsigned 路径
            // 不走 ChargeTransactionPayment) / note_stalled(Root,本链无 sudo 实际不可达)。
            // 等价证据上报本就属公益保护链稳定运行,统一免费。
            RuntimeCall::Grandpa(_) => FeeChargeKind::Free,
            // 决议发行 / 决议销毁的 propose_X 是治理提案交易，固定 1 元；
            // 维护型 Root / 系统型调用免费。
            RuntimeCall::ResolutionIssuance(ref issuance_call) => match issuance_call {
                resolution_issuance::pallet::Call::propose_issuance { .. } => {
                    FeeChargeKind::VoteFlat
                }
                _ => FeeChargeKind::Free,
            },
            RuntimeCall::ResolutionDestroy(resolution_destroy::pallet::Call::propose_destroy {
                ..
            }) => FeeChargeKind::VoteFlat,
            RuntimeCall::ResolutionDestroy(_) => FeeChargeKind::Free,
            // 投票引擎主 pallet 公开 call 共 3 个:
            //   finalize_proposal — 任意人推动超时结算,免费;
            //   retry_passed_proposal / cancel_passed_proposal — 管理员手动重试/取消,VOTE_FLAT_FEE。
            RuntimeCall::VotingEngine(ref ve_call) => match ve_call {
                votingengine::pallet::Call::finalize_proposal { .. } => FeeChargeKind::Free,
                _ => FeeChargeKind::VoteFlat,
            },
            // CitizenIdentity:占号/吊销是公共登记服务,免费(滥用由链上注册局
            // 授权门槛拦截);身份登记、更新、撤销和人口快照按投票统一价 1 元/次。
            RuntimeCall::CitizenIdentity(ref ci_call) => match ci_call {
                citizen_identity::pallet::Call::occupy_cid { .. }
                | citizen_identity::pallet::Call::occupy_cids_batch { .. }
                | citizen_identity::pallet::Call::revoke_cid { .. } => FeeChargeKind::Free,
                _ => FeeChargeKind::VoteFlat,
            },
            // 广场发布没有资金交易金额，按零金额进入链上费用模型，收取
            // ONCHAIN_MIN_FEE = 10 分；分账继续复用 OnchainFeeRouter 的 80/10/10。
            RuntimeCall::SquarePost(_) => FeeChargeKind::OnchainAmount(0),
            // FullnodeIssuance bind_reward_wallet / rebind_reward_wallet:1 元/次。
            RuntimeCall::FullnodeIssuance(_) => FeeChargeKind::VoteFlat,
            // 手动重试/取消统一收口至 votingengine::retry_passed_proposal /
            // cancel_passed_proposal(在 RuntimeCall::VotingEngine 分支按 VOTE_FLAT_FEE 处理)。
            // 业务 pallet 的 propose_X / cleanup_X 全部按 VOTE_FLAT_FEE 收费(1 元/次)。
            RuntimeCall::RuntimeUpgrade(_) => FeeChargeKind::VoteFlat,
            RuntimeCall::GrandpaKeyChange(_) => FeeChargeKind::VoteFlat,
            // 立法院模块 propose_enact_law / propose_amend_law / propose_repeal_law 是治理提案交易,
            // 固定按投票统一价 1 元/次(ADR-027),与其它治理 pallet 的 propose_X 一致。
            RuntimeCall::LegislationYuan(_) => FeeChargeKind::VoteFlat,
            // 多签转账 propose_X 只是创建治理提案，交易本身固定收 1 元；
            // 真正转账执行时，multisig-transfer 内部再按转出金额 × 0.1% 收链上交易费。
            RuntimeCall::MultisigTransfer(ref dt_call) => match dt_call {
                multisig::pallet::Call::propose_transfer { .. }
                | multisig::pallet::Call::propose_safety_fund_transfer { .. }
                | multisig::pallet::Call::propose_sweep_to_main { .. } => FeeChargeKind::VoteFlat,
                // 兜底:未来若新增非金额型管理 extrinsic 按投票统一价 1 元/次。
                _ => FeeChargeKind::VoteFlat,
            },
            // 清算行(L2)扫码支付清算。
            RuntimeCall::OffchainTransaction(ref offchain_call) => {
                match offchain_call {
                    // L3 充值 / 提现:按金额计费(链上资金交易 0.1% 最低 0.1 元)
                    offchain::pallet::Call::deposit { amount } => {
                        FeeChargeKind::OnchainAmount(*amount)
                    }
                    offchain::pallet::Call::withdraw { amount } => {
                        FeeChargeKind::OnchainAmount(*amount)
                    }
                    // 清算行批次 V2 是链下交易费，结算执行阶段已经把
                    // Σ batch[i].fee_amount 转给清算行费用账户，本层只标记类别不二次分账。
                    offchain::pallet::Call::submit_offchain_batch { batch, .. } => {
                        let mut total_fee: u128 = 0;
                        for item in batch.iter() {
                            total_fee = total_fee.saturating_add(item.fee_amount);
                        }
                        FeeChargeKind::OffchainFee(total_fee)
                    }
                    // 全局费率上限调整(Root Origin,免费)
                    offchain::pallet::Call::set_max_l2_fee_rate { .. } => FeeChargeKind::Free,
                    // 其他付费调用(bind_clearing_bank / switch_bank / propose_l2_fee_rate):
                    // 按投票统一价 1 元/次
                    _ => FeeChargeKind::VoteFlat,
                }
            }
            // 3 个 mode-specific 投票 extrinsic 全部按投票统一价 1 元/次:
            //   InternalVote::cast / JointVote::cast_admin / JointVote::cast_referendum
            RuntimeCall::InternalVote(_) => FeeChargeKind::VoteFlat,
            RuntimeCall::JointVote(_) => FeeChargeKind::VoteFlat,
            // 立法投票 3 个 extrinsic(prepare_population_snapshot / cast_representative_vote /
            // cast_referendum_vote)按投票统一价 1 元/次(ADR-027)。
            RuntimeCall::LegislationVote(_) => FeeChargeKind::VoteFlat,
            // 选举投票创建/投票 extrinsic 按投票统一价 1 元/次。
            RuntimeCall::ElectionVote(_) => FeeChargeKind::VoteFlat,
            // OnchainIssuance 暴露 10 个 propose_X extrinsic(call_index 0..=4 业务 / 10..=14 监管)。
            // 全部按 VOTE_FLAT_FEE = 1 元/次,与 GMB 其他业务 pallet 的 propose_X 一致。
            // 1000 GMB 创建费走 onchain_issuance::fee::reserve_creation_deposit 内部 reserve(propose_issue 内部完成),
            // 与 RuntimeFeeKindClassifier 计费正交。
            RuntimeCall::OnchainIssuance(_) => FeeChargeKind::VoteFlat,
            // pallet_assets 内核所有原生 extrinsic 已被 RuntimeCallFilter 拦在入口,
            // 永远到不了本路径;此分支仅供编译期 exhaustive 检查。
            RuntimeCall::Assets(_) => FeeChargeKind::Free,
            // Balances 外部入口已被 RuntimeCallFilter 全部拒绝;这里只保留穷尽分支。
            RuntimeCall::Balances(_) => FeeChargeKind::Unknown,
            //
            // 不再写 `_ => Unknown` 兜底:补 RuntimeCall::Grandpa 之后所有 pallet 变体已穷尽,
            // 将来新增 pallet 若忘记归类会编译期 non-exhaustive match 报错,
            // 强制开发者显式选择五类费用模型之一。
        }
    }
}

pub struct RuntimeFeePayerExtractor;

impl onchain::CallFeePayer<AccountId, RuntimeCall> for RuntimeFeePayerExtractor {
    fn fee_payer(_who: &AccountId, call: &RuntimeCall) -> Option<AccountId> {
        match call {
            // 清算行 V2 批次:链上 gas 由 institution_account 对应机构的费用账户承担。
            //
            // **收款方主导清算**模型下,
            // institution_account = 收款方清算行主账户。fee_account_of(institution_account)
            // = 收款方清算行费用账户 = 同一账户既收清算手续费又付链上 gas,自给自足闭环。
            //
            // 提交者(origin)是该机构的某个激活管理员(已在节点端解密私钥,自动签),
            // 但其个人钱包余额不参与 gas 扣费。
            RuntimeCall::OffchainTransaction(offchain::pallet::Call::submit_offchain_batch {
                institution_account,
                ..
            }) => offchain::Pallet::<Runtime>::fee_account_of(institution_account).ok(),
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

impl primitives::multisig::AccountValidator<AccountId> for RuntimeAccountValidator {
    fn is_valid(account: &AccountId) -> bool {
        // 禁止零账户。
        if account == &AccountId::new([0u8; 32]) {
            return false;
        }

        // 禁止占用“国家储委会/省储委会”的制度保留交易账户。
        if primitives::cid::china::china_cb::CHINA_CB
            .iter()
            .any(|n| account == &AccountId::new(n.main_account))
        {
            return false;
        }

        // 禁止占用“省储行”的制度保留交易账户。
        if primitives::cid::china::china_ch::CHINA_CH
            .iter()
            .any(|n| account == &AccountId::new(n.main_account))
        {
            return false;
        }

        // 禁止占用省储行费用账户（BLAKE2-256 派生）。
        if primitives::cid::china::china_ch::CHINA_CH
            .iter()
            .any(|n| account == &AccountId::new(n.fee_account))
        {
            return false;
        }

        // 禁止占用国家储委会安全基金账户。
        if is_safety_fund_account(account) {
            return false;
        }

        // 禁止占用国家储委会两和基金账户。
        if is_nrc_he_account(account) {
            return false;
        }

        // 禁止占用储委会费用账户（44 个机构）。
        if is_cb_fee_account(account) {
            return false;
        }

        true
    }
}

pub struct RuntimeReservedAccountGuard;
pub struct RuntimeCidInstitutionVerifier;
pub struct RuntimeRegistryAuthority;

pub struct RuntimeProtectedSourceChecker;
pub struct RuntimeInstitutionAsset;

impl primitives::multisig::ProtectedSourceChecker<AccountId> for RuntimeProtectedSourceChecker {
    fn is_protected(address: &AccountId) -> bool {
        is_stake_account(address)
    }
}

impl primitives::institution_asset::InstitutionAsset<AccountId> for RuntimeInstitutionAsset {
    fn can_spend(
        source: &AccountId,
        action: primitives::institution_asset::InstitutionAssetAction,
    ) -> bool {
        // 匹配顺序很重要——更具体的账户类型必须放在更宽泛的类型之前。
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
                primitives::institution_asset::InstitutionAssetAction::OffchainFeeSweepExecute
            );
        }

        // 3. 储委会费用账户（44 个机构）：只允许手续费归集
        if is_cb_fee_account(source) {
            return matches!(
                action,
                primitives::institution_asset::InstitutionAssetAction::OffchainFeeSweepExecute
            );
        }

        // 4. 国家储委会安全基金账户：只允许安全基金转账
        if source == &AccountId::new(primitives::cid::china::china_cb::SAFETY_FUND_ACCOUNT) {
            return matches!(
                action,
                primitives::institution_asset::InstitutionAssetAction::NrcSafetyFundTransfer
            );
        }

        // 5. 多签保留账户（范围最宽）：只允许多签转账和关闭
        if is_reserved_main_account(source) {
            return matches!(
                action,
                primitives::institution_asset::InstitutionAssetAction::MultisigTransferExecute
                    | primitives::institution_asset::InstitutionAssetAction::MultisigCloseExecute
            );
        }

        // 6. 普通账户：全放行
        true
    }
}

impl primitives::multisig::ReservedAccountGuard<AccountId> for RuntimeReservedAccountGuard {
    fn is_reserved(account: &AccountId) -> bool {
        // 禁止占用省储行 stake_account（制度保留账户）。
        if primitives::cid::china::china_ch::CHINA_CH
            .iter()
            .any(|n| account == &AccountId::new(n.stake_account))
        {
            return true;
        }

        // 禁止占用省储行费用账户（BLAKE2-256 派生）。
        if primitives::cid::china::china_ch::CHINA_CH
            .iter()
            .any(|n| account == &AccountId::new(n.fee_account))
        {
            return true;
        }

        // 禁止占用国家储委会安全基金账户。
        if is_safety_fund_account(account) {
            return true;
        }

        // 禁止占用国家储委会两和基金账户。
        if is_nrc_he_account(account) {
            return true;
        }

        // 禁止占用储委会费用账户（44 个机构）。
        if is_cb_fee_account(account) {
            return true;
        }

        is_reserved_main_account(account)
    }
}

fn cid_institution_code(cid_number: &[u8]) -> Option<primitives::cid::code::InstitutionCode> {
    let text = core::str::from_utf8(cid_number).ok()?;
    primitives::cid::code::institution_code_from_cid_number(text.trim())
}

fn cid_scope_codes(cid_number: &[u8]) -> Option<([u8; 2], [u8; 3])> {
    let text = core::str::from_utf8(cid_number).ok()?;
    let r5 = text.trim().split('-').next()?;
    let bytes = r5.as_bytes();
    if bytes.len() != primitives::cid::number::CID_NUMBER_SEGMENT_R5_LEN {
        return None;
    }
    let mut province_code = [0_u8; 2];
    let mut city_code = [0_u8; 3];
    province_code.copy_from_slice(&bytes[..2]);
    city_code.copy_from_slice(&bytes[2..5]);
    Some((province_code, city_code))
}

/// 联邦注册局省专员权限只认 entity 中的有效岗位任职。
///
/// `admins` 只回答“是否为联邦注册局管理员”，省级业务边界由
/// `PROVINCE_COMMISSIONER_<省码>` 岗位确定，不再读取独立省级管理员表。
fn is_active_frg_province_commissioner(
    frg_cid_number: &[u8],
    admin: &AccountId,
    province_code: &[u8],
) -> bool {
    if province_code.len() != 2
        || cid_institution_code(frg_cid_number) != Some(primitives::cid::code::FRG)
    {
        return false;
    }
    let mut code = [0_u8; 2];
    code.copy_from_slice(province_code);
    let role_code = primitives::governance_skeleton::province_commissioner_role_code(code);
    <public_manage::Pallet<Runtime> as InstitutionRoleQuery<AccountId>>::is_active_assignment(
        frg_cid_number,
        admin,
        role_code.as_slice(),
    )
}

impl entity_primitives::RegistryAuthority<AccountId> for RuntimeRegistryAuthority {
    fn can_register_institution(
        registrar: &AccountId,
        actor_cid_number: &[u8],
        credential_signer_pubkey: &[u8; 32],
        target_cid_number: &[u8],
        target_institution_code: primitives::cid::code::InstitutionCode,
        scope_province_name: &[u8],
        scope_city_name: &[u8],
    ) -> bool {
        let signer_account = AccountId::new(*credential_signer_pubkey);
        if registrar != &signer_account {
            return false;
        }
        let Some(actor_code) = cid_institution_code(actor_cid_number) else {
            return false;
        };
        if !RuntimeInstitutionAdminQuery::is_institution_admin(
            actor_code,
            actor_cid_number,
            &signer_account,
        ) {
            return false;
        }
        let Some(parsed_target_code) = cid_institution_code(target_cid_number) else {
            return false;
        };
        if parsed_target_code != target_institution_code
            || primitives::cid::code::is_fixed_governance_code(&target_institution_code)
            || primitives::institution_constraints::is_permanent_singleton_code(
                &target_institution_code,
            )
        {
            return false;
        }

        let Some((target_province_code, target_city_code)) = cid_scope_codes(target_cid_number)
        else {
            return false;
        };
        let Ok(scope_province_name) = core::str::from_utf8(scope_province_name) else {
            return false;
        };
        let Some(scope_province_code) =
            primitives::cid::code::province_code_by_name(scope_province_name)
        else {
            return false;
        };
        if scope_province_code != target_province_code {
            return false;
        }

        const CITY_REGISTRY_CODE: primitives::cid::code::InstitutionCode = *b"CREG";
        if actor_code == admin_primitives::FRG {
            return is_active_frg_province_commissioner(
                actor_cid_number,
                &signer_account,
                &target_province_code,
            );
        }

        if actor_code == CITY_REGISTRY_CODE {
            if target_institution_code == CITY_REGISTRY_CODE || scope_city_name.is_empty() {
                return false;
            }
            let Some((issuer_province_code, issuer_city_code)) = cid_scope_codes(actor_cid_number)
            else {
                return false;
            };
            // CREG 只能登记本市非 CREG 机构;市归属由 CID R5 直接校验。
            return issuer_province_code == target_province_code
                && issuer_city_code == target_city_code;
        }

        false
    }
}

pub struct RuntimeAddressAuthority;

impl address_registry::AddressUpdateAuthority<AccountId> for RuntimeAddressAuthority {
    fn can_update_catalog(who: &AccountId, actor_cid_number: &[u8]) -> bool {
        let Ok(actor_text) = core::str::from_utf8(actor_cid_number) else {
            return false;
        };
        primitives::cid::code::institution_code_from_cid_number(actor_text)
            == Some(primitives::cid::code::FRG)
            && RuntimeInstitutionAdminQuery::is_institution_admin(
                primitives::cid::code::FRG,
                actor_cid_number,
                who,
            )
    }

    fn can_update_address(
        who: &AccountId,
        actor_cid_number: &[u8],
        province_code: &[u8],
        city_code: &[u8],
    ) -> bool {
        if province_code.is_empty() || city_code.is_empty() {
            return false;
        }
        let Ok(actor_text) = core::str::from_utf8(actor_cid_number) else {
            return false;
        };
        let Some(actor_code) = primitives::cid::code::institution_code_from_cid_number(actor_text)
        else {
            return false;
        };
        if !RuntimeInstitutionAdminQuery::is_institution_admin(actor_code, actor_cid_number, who) {
            return false;
        }

        if actor_code == primitives::cid::code::FRG {
            return is_active_frg_province_commissioner(actor_cid_number, who, province_code);
        }

        const CITY_REGISTRY_CODE: primitives::cid::code::InstitutionCode = *b"CREG";
        if actor_code != CITY_REGISTRY_CODE {
            return false;
        }
        let Some((issuer_province_code, issuer_city_code)) = cid_scope_codes(actor_cid_number)
        else {
            return false;
        };
        // CREG 管理员只能更新本市地址。镇以下地址名称与完整地址仍走本市注册局。
        issuer_province_code.as_ref() == province_code && issuer_city_code.as_ref() == city_code
    }
}

impl address_registry::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type AddressAuthority = RuntimeAddressAuthority;
    type MaxCodeLen = ConstU32<16>;
    type MaxVersionLen = ConstU32<32>;
    type MaxAddressNameCodeLen = ConstU32<3>;
    type MaxAddressLocalNoLen = ConstU32<4>;
    type MaxAddressNameLen = ConstU32<96>;
    type MaxAddressDetailLen = ConstU32<128>;
}

#[cfg(not(feature = "runtime-benchmarks"))]
fn issuer_admin_public(
    actor_cid_number: &[u8],
    signer_pubkey: &[u8; 32],
) -> Option<sr25519::Public> {
    let signer_account = AccountId::new(*signer_pubkey);
    let institution_code = cid_institution_code(actor_cid_number)?;
    if !RuntimeInstitutionAdminQuery::is_institution_admin(
        institution_code,
        actor_cid_number,
        &signer_account,
    ) {
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

impl<AccountName, NonceBytes, SignatureBytes>
    entity_primitives::CidInstitutionVerifier<AccountId, AccountName, NonceBytes, SignatureBytes>
    for RuntimeCidInstitutionVerifier
where
    AccountName: AsRef<[u8]>,
    NonceBytes: AsRef<[u8]>,
    SignatureBytes: AsRef<[u8]>,
{
    fn verify_institution_registration(
        cid_number: &[u8],
        cid_full_name: &AccountName,
        cid_short_name: &[u8],
        account_names: &[Vec<u8>],
        nonce: &NonceBytes,
        signature: &SignatureBytes,
        actor_cid_number: &[u8],
        credential_signer_pubkey: &[u8; 32],
        scope_province_name: &[u8],
        scope_city_name: &[u8],
        town_code: &[u8],
    ) -> bool {
        #[cfg(feature = "runtime-benchmarks")]
        {
            let _ = (
                actor_cid_number,
                credential_signer_pubkey,
                scope_province_name,
                scope_city_name,
                cid_short_name,
                town_code,
            );
            return !cid_number.is_empty()
                && !cid_full_name.as_ref().is_empty()
                && !account_names.is_empty()
                && !nonce.as_ref().is_empty()
                && !signature.as_ref().is_empty();
        }

        #[cfg(not(feature = "runtime-benchmarks"))]
        {
            let Some(public) = issuer_admin_public(actor_cid_number, credential_signer_pubkey)
            else {
                return false;
            };
            let Some(signature) = sr25519_signature_from_bytes(signature.as_ref()) else {
                return false;
            };

            // 这里必须和身份注册局 `/registration-info` 的签名 payload 严格一致。
            // payload 字段(GMB + OP_SIGN_INST 域头由 signing_message 统一拼接):
            // genesis_hash + cid_number + cid_full_name + cid_short_name + account_names[]
            // + nonce + 签发机构 + 作用域 + town_code。
            let msg = primitives::sign::institution_registration_message(
                &frame_system::Pallet::<Runtime>::block_hash(0),
                cid_number,
                cid_full_name.as_ref(),
                cid_short_name,
                account_names,
                nonce,
                actor_cid_number,
                credential_signer_pubkey,
                scope_province_name,
                scope_city_name,
                town_code,
            );

            sr25519_verify(&signature, &msg, &public)
        }
    }

    fn verify_institution_creation(
        cid_number: &[u8],
        cid_full_name: &AccountName,
        cid_short_name: &[u8],
        legal_representative_name: &[u8],
        legal_representative_cid_number: &[u8],
        legal_representative_account: &AccountId,
        account_names: &[Vec<u8>],
        roles_payload: &[u8],
        assignments_payload: &[u8],
        nonce: &NonceBytes,
        signature: &SignatureBytes,
        actor_cid_number: &[u8],
        credential_signer_pubkey: &[u8; 32],
        scope_province_name: &[u8],
        scope_city_name: &[u8],
        town_code: &[u8],
    ) -> bool {
        #[cfg(feature = "runtime-benchmarks")]
        {
            let _ = (
                actor_cid_number,
                credential_signer_pubkey,
                scope_province_name,
                scope_city_name,
                cid_short_name,
                legal_representative_account,
                roles_payload,
                assignments_payload,
                town_code,
            );
            return !cid_number.is_empty()
                && !cid_full_name.as_ref().is_empty()
                && !legal_representative_name.is_empty()
                && !legal_representative_cid_number.is_empty()
                && !account_names.is_empty()
                && !roles_payload.is_empty()
                && !assignments_payload.is_empty()
                && !nonce.as_ref().is_empty()
                && !signature.as_ref().is_empty();
        }

        #[cfg(not(feature = "runtime-benchmarks"))]
        {
            let Some(public) = issuer_admin_public(actor_cid_number, credential_signer_pubkey)
            else {
                return false;
            };
            let Some(signature) = sr25519_signature_from_bytes(signature.as_ref()) else {
                return false;
            };

            // 机构创建凭证必须同时覆盖法定代表人三字段，防止冷签前后被替换。
            let msg = primitives::sign::institution_creation_message(
                &frame_system::Pallet::<Runtime>::block_hash(0),
                cid_number,
                cid_full_name.as_ref(),
                cid_short_name,
                legal_representative_name,
                legal_representative_cid_number,
                legal_representative_account,
                account_names,
                roles_payload,
                assignments_payload,
                nonce,
                actor_cid_number,
                credential_signer_pubkey,
                scope_province_name,
                scope_city_name,
                town_code,
            );

            sr25519_verify(&signature, &msg, &public)
        }
    }

    fn verify_institution_account_close(
        cid_number: &[u8],
        account_name: &[u8],
        target_account: &AccountId,
        nonce: &NonceBytes,
        signature: &SignatureBytes,
        credential_issuer_cid_number: &[u8],
        credential_signer_pubkey: &[u8; 32],
    ) -> bool {
        #[cfg(feature = "runtime-benchmarks")]
        {
            let _ = (
                account_name,
                target_account,
                credential_issuer_cid_number,
                credential_signer_pubkey,
            );
            return !cid_number.is_empty()
                && !nonce.as_ref().is_empty()
                && !signature.as_ref().is_empty();
        }

        #[cfg(not(feature = "runtime-benchmarks"))]
        {
            let Some(public) =
                issuer_admin_public(credential_issuer_cid_number, credential_signer_pubkey)
            else {
                return false;
            };
            let Some(signature) = sr25519_signature_from_bytes(signature.as_ref()) else {
                return false;
            };

            // 必须与身份注册局注销凭证签发 payload 严格一致。
            // payload 字段(GMB + OP_SIGN_DEREGISTER 域头由 signing_message 统一拼接):
            // genesis_hash + cid_number + account_name + target_account
            // + nonce + 签发机构 + 签发管理员公钥。scope 与 target_account 入签名,
            // 防换范围/换账户重放。
            let msg = primitives::sign::institution_account_close_message(
                &frame_system::Pallet::<Runtime>::block_hash(0),
                cid_number,
                account_name,
                target_account,
                nonce,
                credential_issuer_cid_number,
                credential_signer_pubkey,
            );

            sr25519_verify(&signature, &msg, &public)
        }
    }
}

impl public_manage::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type InternalVoteEngine = InternalVote;
    type AdminLifecycle = PublicAdmins;
    type SiblingInstitutionQuery = PrivateManage;
    type InstitutionAdminQuery = RuntimeInstitutionAdminQuery;
    type AccountValidator = RuntimeAccountValidator;
    type ReservedAccountChecker = RuntimeReservedAccountGuard;
    type ProtectedSourceChecker = RuntimeProtectedSourceChecker;
    type InstitutionAsset = RuntimeInstitutionAsset;
    type CidInstitutionVerifier = RuntimeCidInstitutionVerifier;
    type RegistryAuthority = RuntimeRegistryAuthority;
    type FeeRouter = TransferFeeRouter;
    type MaxAdmins = MaxAdminsPerInstitution;
    type MaxCidNumberLength = ConstU32<{ primitives::core_const::CID_NUMBER_MAX_BYTES }>;
    type MaxAccountNameLength = ConstU32<128>;
    type MaxRegisterNonceLength = ConstU32<64>;
    type MaxRegisterSignatureLength = ConstU32<64>;
    type MaxInstitutionAccounts = ConstU32<16>;
    type WeightInfo = public_manage::weights::SubstrateWeight<Runtime>;
}

impl private_manage::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type InternalVoteEngine = InternalVote;
    type AdminLifecycle = PrivateAdmins;
    type SiblingInstitutionQuery = PublicManage;
    type InstitutionAdminQuery = RuntimeInstitutionAdminQuery;
    type AccountValidator = RuntimeAccountValidator;
    type ReservedAccountChecker = RuntimeReservedAccountGuard;
    type ProtectedSourceChecker = RuntimeProtectedSourceChecker;
    type InstitutionAsset = RuntimeInstitutionAsset;
    type CidInstitutionVerifier = RuntimeCidInstitutionVerifier;
    type RegistryAuthority = RuntimeRegistryAuthority;
    type FeeRouter = TransferFeeRouter;
    type MaxAdmins = MaxAdminsPerInstitution;
    type MaxCidNumberLength = ConstU32<{ primitives::core_const::CID_NUMBER_MAX_BYTES }>;
    type MaxAccountNameLength = ConstU32<128>;
    type MaxRegisterNonceLength = ConstU32<64>;
    type MaxRegisterSignatureLength = ConstU32<64>;
    type MaxInstitutionAccounts = ConstU32<16>;
    type WeightInfo = private_manage::weights::SubstrateWeight<Runtime>;
}

impl personal_manage::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type InternalVoteEngine = InternalVote;
    type AccountValidator = RuntimeAccountValidator;
    type ReservedAccountChecker = RuntimeReservedAccountGuard;
    type ProtectedSourceChecker = RuntimeProtectedSourceChecker;
    type InstitutionAsset = RuntimeInstitutionAsset;
    type PersonalAdminLifecycle = personal_admins::Pallet<Runtime>;
    type PersonalAdminQuery = personal_admins::Pallet<Runtime>;
    type FeeRouter = TransferFeeRouter;
    type MaxAccountNameLength = ConstU32<128>;
    type MaxPersonalAccountAdmins = MaxPersonalAccountAdmins;
    type MinCreateAmount = ConstU128<111>;
    type MinCloseBalance = ConstU128<121>;
    type WeightInfo = personal_manage::weights::SubstrateWeight<Runtime>;
}

impl personal_admins::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type InternalVoteEngine = InternalVote;
    type MaxPersonalAccountAdmins = MaxPersonalAccountAdmins;
    type WeightInfo = personal_admins::weights::SubstrateWeight<Runtime>;
}

pub struct RuntimeCitizenIdentityAuthority;

impl
    citizen_identity::CitizenIdentityAuthority<
        AccountId,
        citizen_identity::pallet::SignatureOf<Runtime>,
    > for RuntimeCitizenIdentityAuthority
{
    fn can_manage_voting_identity(
        registrar: &AccountId,
        actor_cid_number: &[u8],
        residence_province_code: &[u8],
        residence_city_code: &[u8],
        _level: citizen_identity::CitizenIdentityLevel,
    ) -> bool {
        if residence_province_code.is_empty() || residence_city_code.is_empty() {
            return false;
        }
        let Ok(actor_text) = core::str::from_utf8(actor_cid_number) else {
            return false;
        };
        let Some(actor_code) = primitives::cid::code::institution_code_from_cid_number(actor_text)
        else {
            return false;
        };
        if !RuntimeInstitutionAdminQuery::is_institution_admin(
            actor_code,
            actor_cid_number,
            registrar,
        ) {
            return false;
        }

        if actor_code == primitives::cid::code::FRG {
            return is_active_frg_province_commissioner(
                actor_cid_number,
                registrar,
                residence_province_code,
            );
        }

        const CITY_REGISTRY_CODE: primitives::cid::code::InstitutionCode = *b"CREG";
        if actor_code != CITY_REGISTRY_CODE {
            return false;
        }
        let Some((registry_province_code, registry_city_code)) = cid_scope_codes(actor_cid_number)
        else {
            return false;
        };
        // CREG 管理员只能管理本市公民身份；出生地不参与居住地注册权限。
        registry_province_code.as_ref() == residence_province_code
            && registry_city_code.as_ref() == residence_city_code
    }

    fn verify_citizen_signature(
        wallet_account: &AccountId,
        payload: &[u8],
        signature: &citizen_identity::pallet::SignatureOf<Runtime>,
    ) -> bool {
        #[cfg(feature = "runtime-benchmarks")]
        {
            let _ = (wallet_account, payload);
            return !signature.is_empty();
        }

        #[cfg(not(feature = "runtime-benchmarks"))]
        {
            let Ok(raw_account) = <[u8; 32]>::try_from(wallet_account.as_ref()) else {
                return false;
            };
            let Some(signature) = sr25519_signature_from_bytes(signature.as_slice()) else {
                return false;
            };
            let public = sr25519::Public::from_raw(raw_account);
            let msg = primitives::sign::signing_message(
                primitives::sign::OP_SIGN_CITIZEN_IDENTITY,
                payload,
            );
            sr25519_verify(&signature, &msg, &public)
        }
    }
}

impl citizen_identity::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxCitizenSignatureLength = ConstU32<64>;
    type CitizenIdentityAuthority = RuntimeCitizenIdentityAuthority;
    type OnVotingIdentityRegistered = CitizenIssuance;
    type TimeProvider = crate::Timestamp;
    type WeightInfo = citizen_identity::weights::SubstrateWeight<Runtime>;
}

pub struct RuntimeSquarePostCitizenIdentity;

impl square_post::SquarePostCitizenIdentityProvider<AccountId>
    for RuntimeSquarePostCitizenIdentity
{
    fn cid_number(owner_account: &AccountId) -> Option<Vec<u8>> {
        citizen_identity::VotingIdentityByAccount::<Runtime>::get(owner_account).and_then(
            |identity| {
                if identity.citizen_status == citizen_identity::CitizenStatus::Normal {
                    Some(identity.cid_number.to_vec())
                } else {
                    None
                }
            },
        )
    }
}

impl square_post::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type CitizenIdentity = RuntimeSquarePostCitizenIdentity;
    type MaxSquarePostIdLen = ConstU32<64>;
    type MaxSquareCidNumberLen = ConstU32<32>;
    type MaxSquareStorageReceiptIdLen = ConstU32<96>;
    type WeightInfo = square_post::weights::SubstrateWeight<Runtime>;
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
    /// 物理 BoundedVec 上限必须覆盖机构账户 1989 人场景；个人账户
    /// 另由 MaxPersonalAccountAdmins 限制为 64。
    pub const MaxAdminsPerInstitution: u32 = 1989;
    /// 管理员治理：单个个人账户管理员上限。
    pub const MaxPersonalAccountAdmins: u32 = 64;
    /// GRANDPA authority set 变更生效延迟（单位：区块）。
    /// 取非 0，给运维注入新 gran 私钥预留窗口，避免立即切换导致短时失票。
    pub const GrandpaAuthoritySetChangeDelay: u32 = 30;
}

impl public_admins::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxAdminsPerInstitution = MaxAdminsPerInstitution;
    type InternalVoteEngine = InternalVote;
}

impl private_admins::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type MaxAdminsPerInstitution = MaxAdminsPerInstitution;
    type InternalVoteEngine = InternalVote;
}

impl resolution_destroy::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type InternalVoteEngine = InternalVote;
    type InstitutionQuery = RuntimeInstitutionQuery;
    type WeightInfo = resolution_destroy::weights::SubstrateWeight<Runtime>;
}

impl grandpakey_change::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type GrandpaChangeDelay = GrandpaAuthoritySetChangeDelay;
    type InternalVoteEngine = InternalVote;
    type WeightInfo = grandpakey_change::weights::SubstrateWeight<Runtime>;
}

/// 转账提案手续费分账适配器：将旧 Currency NegativeImbalance 转换后
/// 交给现有 OnchainFeeRouter 处理（80% 全节点 / 10% 国家储委会 / 10% 安全基金）。
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

        type FeeRouter = onchain::OnchainFeeRouter<
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

impl multisig::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type InternalVoteEngine = InternalVote;
    type InstitutionAsset = RuntimeInstitutionAsset;
    type ProtectedSourceChecker = RuntimeProtectedSourceChecker;
    type MaxRemarkLen = ConstU32<256>;
    type FeeRouter = TransferFeeRouter;
    // 多签 admin 配置查询拆给个人生命周期 pallet 与 runtime 机构聚合查询。
    // 转账治理时 multisig-transfer 通过 union 调用,先问个人侧、再问机构侧。
    type PersonalQuery = personal_manage::Pallet<Runtime>;
    type InstitutionQuery = RuntimeInstitutionQuery;
    type WeightInfo = multisig::weights::SubstrateWeight<Runtime>;
}

/// 机构生命周期聚合查询。
///
/// 下游交易模块只依赖本适配器；runtime 内部按公权、私权顺序查询两个生命周期 pallet。
pub struct RuntimeInstitutionQuery;

impl entity_primitives::InstitutionMultisigQuery<AccountId> for RuntimeInstitutionQuery {
    fn lookup_cid(addr: &AccountId) -> Option<Vec<u8>> {
        public_manage::Pallet::<Runtime>::lookup_cid(addr)
            .or_else(|| private_manage::Pallet::<Runtime>::lookup_cid(addr))
    }

    fn lookup_org(addr: &AccountId) -> Option<votingengine::types::InstitutionCode> {
        public_manage::Pallet::<Runtime>::lookup_org(addr)
            .or_else(|| private_manage::Pallet::<Runtime>::lookup_org(addr))
    }

    fn lookup_admin_config(
        addr: &AccountId,
    ) -> Option<primitives::multisig::MultisigConfigSnapshot<AccountId>> {
        public_manage::Pallet::<Runtime>::lookup_admin_config(addr)
            .or_else(|| private_manage::Pallet::<Runtime>::lookup_admin_config(addr))
    }

    fn account_exists(addr: &AccountId) -> bool {
        public_manage::Pallet::<Runtime>::account_exists(addr)
            || private_manage::Pallet::<Runtime>::account_exists(addr)
    }
}

/// 机构存在性统一按 CID 查询；公权、私权只是存储分区，不形成第二身份。
pub struct RuntimeInstitutionCidQuery;

impl entity_primitives::InstitutionCidQuery<votingengine::types::CidNumber>
    for RuntimeInstitutionCidQuery
{
    fn cid_exists(cid_number: &votingengine::types::CidNumber) -> bool {
        public_manage::Institutions::<Runtime>::contains_key(cid_number)
            || private_manage::Institutions::<Runtime>::contains_key(cid_number)
    }
}

/// 通用机构治理结果路由适配器。
///
/// 已完成自身业务校验的任免/治理模块可用它按机构码选择 entity 模组；
/// `election-vote` 不使用本适配器，选举结果必须先回到 election-campaign 复核。
pub struct RuntimeInstitutionGovernanceResultHandler;

impl entity_primitives::InstitutionGovernanceResultHandler<AccountId>
    for RuntimeInstitutionGovernanceResultHandler
{
    fn apply_institution_governance_result(
        result: entity_primitives::InstitutionGovernanceResult<AccountId>,
    ) -> DispatchResult {
        if admin_primitives::is_public_admin_code(&result.institution_code) {
            return public_manage::Pallet::<Runtime>::apply_institution_governance_result(result);
        }
        if admin_primitives::is_private_admin_code(&result.institution_code) {
            return private_manage::Pallet::<Runtime>::apply_institution_governance_result(result);
        }
        Err(sp_runtime::DispatchError::Other(
            "UnsupportedInstitutionGovernanceResultCode",
        ))
    }
}

/// 机构管理员唯一查询路由：CID 是 key，公权/私权只决定落在哪个 storage pallet。
pub struct RuntimeInstitutionAdminQuery;

impl admin_primitives::InstitutionAdminQuery<AccountId> for RuntimeInstitutionAdminQuery {
    fn institution_admins_exist(
        institution_code: primitives::cid::code::InstitutionCode,
        cid_number: &[u8],
    ) -> bool {
        Self::institution_admins(institution_code, cid_number).is_some()
    }

    fn is_institution_admin(
        institution_code: primitives::cid::code::InstitutionCode,
        cid_number: &[u8],
        who: &AccountId,
    ) -> bool {
        if admin_primitives::is_public_admin_code(&institution_code) {
            return <public_admins::Pallet<Runtime> as admin_primitives::InstitutionAdminQuery<
                AccountId,
            >>::is_institution_admin(institution_code, cid_number, who);
        }
        if admin_primitives::is_private_admin_code(&institution_code) {
            return <private_admins::Pallet<Runtime> as admin_primitives::InstitutionAdminQuery<
                AccountId,
            >>::is_institution_admin(institution_code, cid_number, who);
        }
        if admin_primitives::is_unincorporated_admin_code(&institution_code) {
            return <public_admins::Pallet<Runtime> as admin_primitives::InstitutionAdminQuery<
                AccountId,
            >>::is_institution_admin(institution_code, cid_number, who)
                || <private_admins::Pallet<Runtime> as admin_primitives::InstitutionAdminQuery<
                    AccountId,
                >>::is_institution_admin(institution_code, cid_number, who);
        }
        false
    }

    fn institution_admins(
        institution_code: primitives::cid::code::InstitutionCode,
        cid_number: &[u8],
    ) -> Option<Vec<AccountId>> {
        if admin_primitives::is_public_admin_code(&institution_code) {
            return <public_admins::Pallet<Runtime> as admin_primitives::InstitutionAdminQuery<
                AccountId,
            >>::institution_admins(institution_code, cid_number);
        }
        if admin_primitives::is_private_admin_code(&institution_code) {
            return <private_admins::Pallet<Runtime> as admin_primitives::InstitutionAdminQuery<
                AccountId,
            >>::institution_admins(institution_code, cid_number);
        }
        if admin_primitives::is_unincorporated_admin_code(&institution_code) {
            return <public_admins::Pallet<Runtime> as admin_primitives::InstitutionAdminQuery<
                AccountId,
            >>::institution_admins(institution_code, cid_number)
            .or_else(|| {
                <private_admins::Pallet<Runtime> as admin_primitives::InstitutionAdminQuery<
                    AccountId,
                >>::institution_admins(institution_code, cid_number)
            });
        }
        None
    }

    fn institution_admins_len(
        institution_code: primitives::cid::code::InstitutionCode,
        cid_number: &[u8],
    ) -> Option<u32> {
        Self::institution_admins(institution_code, cid_number).map(|admins| admins.len() as u32)
    }
}

pub struct RuntimeAdminAccountQuery;

impl AdminAccountQuery<AccountId> for RuntimeAdminAccountQuery {
    fn active_admin_account_exists(
        institution_code: primitives::cid::code::InstitutionCode,
        personal_account: AccountId,
    ) -> bool {
        if admin_primitives::is_personal_admin_code(&institution_code) {
            return personal_admins::Pallet::<Runtime>::active_admin_account_exists(
                institution_code,
                personal_account,
            );
        }
        false
    }

    fn is_active_account_admin(
        institution_code: primitives::cid::code::InstitutionCode,
        personal_account: AccountId,
        who: &AccountId,
    ) -> bool {
        if admin_primitives::is_personal_admin_code(&institution_code) {
            return personal_admins::Pallet::<Runtime>::is_active_account_admin(
                institution_code,
                personal_account,
                who,
            );
        }
        false
    }

    fn active_account_admins(
        institution_code: primitives::cid::code::InstitutionCode,
        personal_account: AccountId,
    ) -> Option<Vec<AccountId>> {
        if admin_primitives::is_personal_admin_code(&institution_code) {
            return personal_admins::Pallet::<Runtime>::active_account_admins(
                institution_code,
                personal_account,
            );
        }
        None
    }

    fn active_account_admins_len(
        institution_code: primitives::cid::code::InstitutionCode,
        personal_account: AccountId,
    ) -> Option<u32> {
        if admin_primitives::is_personal_admin_code(&institution_code) {
            return personal_admins::Pallet::<Runtime>::active_account_admins_len(
                institution_code,
                personal_account,
            );
        }
        None
    }

    fn pending_account_exists_for_snapshot(
        institution_code: primitives::cid::code::InstitutionCode,
        personal_account: AccountId,
    ) -> bool {
        Self::pending_account_admins_len_for_snapshot(institution_code, personal_account).is_some()
    }

    fn is_pending_account_admin_for_snapshot(
        institution_code: primitives::cid::code::InstitutionCode,
        personal_account: AccountId,
        who: &AccountId,
    ) -> bool {
        if admin_primitives::is_personal_admin_code(&institution_code) {
            return personal_admins::Pallet::<Runtime>::is_pending_account_admin_for_snapshot(
                institution_code,
                personal_account,
                who,
            );
        }
        false
    }

    fn pending_account_admins_for_snapshot(
        institution_code: primitives::cid::code::InstitutionCode,
        personal_account: AccountId,
    ) -> Option<Vec<AccountId>> {
        if admin_primitives::is_personal_admin_code(&institution_code) {
            return personal_admins::Pallet::<Runtime>::pending_account_admins_for_snapshot(
                institution_code,
                personal_account,
            );
        }
        None
    }

    fn pending_account_admins_len_for_snapshot(
        institution_code: primitives::cid::code::InstitutionCode,
        personal_account: AccountId,
    ) -> Option<u32> {
        if admin_primitives::is_personal_admin_code(&institution_code) {
            return personal_admins::Pallet::<Runtime>::pending_account_admins_len_for_snapshot(
                institution_code,
                personal_account,
            );
        }
        None
    }
}

/// 机构法定代表人聚合查询。公开事实只从 entity 读取，不再经过 admins。
pub struct RuntimeInstitutionLegalRepresentativeQuery;

impl entity_primitives::InstitutionLegalRepresentativeQuery<AccountId>
    for RuntimeInstitutionLegalRepresentativeQuery
{
    fn legal_representative(cid_number: &[u8]) -> Option<AccountId> {
        let institution_code = cid_institution_code(cid_number)?;
        if admin_primitives::is_public_admin_code(&institution_code) {
            return <public_manage::Pallet<Runtime> as entity_primitives::InstitutionLegalRepresentativeQuery<AccountId>>::legal_representative(
                cid_number,
            );
        }
        if admin_primitives::is_private_admin_code(&institution_code) {
            return <private_manage::Pallet<Runtime> as entity_primitives::InstitutionLegalRepresentativeQuery<AccountId>>::legal_representative(
                cid_number,
            );
        }
        if admin_primitives::is_unincorporated_admin_code(&institution_code) {
            return <public_manage::Pallet<Runtime> as entity_primitives::InstitutionLegalRepresentativeQuery<AccountId>>::legal_representative(
                cid_number,
            )
            .or_else(|| {
                <private_manage::Pallet<Runtime> as entity_primitives::InstitutionLegalRepresentativeQuery<AccountId>>::legal_representative(
                    cid_number,
                )
            });
        }
        None
    }
}

// 链下交易清算模块配置
/// CID 机构登记表查询实现。
///
/// 委托给 runtime 的公权/私权机构生命周期聚合查询；管理员钱包校验统一转给
/// `admins` 集合查询，岗位任职事实仍只读取 entity。
pub struct MultisigCidAccountQuery;

impl offchain::bank_check::CidAccountQuery<AccountId> for MultisigCidAccountQuery {
    fn account_info(addr: &AccountId) -> Option<(Vec<u8>, Vec<u8>)> {
        public_manage::AccountRegisteredCid::<Runtime>::get(addr)
            .map(|info| (info.cid_number.to_vec(), info.account_name.to_vec()))
            .or_else(|| {
                private_manage::AccountRegisteredCid::<Runtime>::get(addr)
                    .map(|info| (info.cid_number.to_vec(), info.account_name.to_vec()))
            })
    }

    fn find_account(cid_number: &[u8], account_name: &[u8]) -> Option<AccountId> {
        let public_id: public_manage::CidNumberOf<Runtime> = cid_number.to_vec().try_into().ok()?;
        let public_name: public_manage::AccountNameOf<Runtime> =
            account_name.to_vec().try_into().ok()?;
        if let Some(info) =
            public_manage::InstitutionAccounts::<Runtime>::get(&public_id, &public_name)
        {
            return Some(info.address);
        }

        let private_id: private_manage::CidNumberOf<Runtime> =
            cid_number.to_vec().try_into().ok()?;
        let private_name: private_manage::AccountNameOf<Runtime> =
            account_name.to_vec().try_into().ok()?;
        private_manage::InstitutionAccounts::<Runtime>::get(&private_id, &private_name)
            .map(|info| info.address)
    }

    fn account_exists(addr: &AccountId) -> bool {
        RuntimeInstitutionQuery::account_exists(addr)
    }

    fn is_institution_admin(cid_number: &[u8], who: &AccountId) -> bool {
        let Some(institution_code) = core::str::from_utf8(cid_number)
            .ok()
            .and_then(primitives::cid::code::institution_code_from_cid_number)
        else {
            return false;
        };
        RuntimeInstitutionAdminQuery::is_institution_admin(institution_code, cid_number, who)
    }

    /// 清算行资格由身份注册局 eligible-search 负责筛选。
    /// 链上不保存 subject_property/sub_type/parent_cid_number，这里只确认该地址属于已登记的
    /// CID 机构账户,避免把 CID 内部机构类型字段重复落到链上。
    fn is_clearing_bank_eligible(addr: &AccountId) -> bool {
        RuntimeInstitutionQuery::account_exists(addr)
    }

    /// 判定 `bank` 主账户对应的机构是否
    /// 已声明为清算行节点(链上 `ClearingBankNodes` 存在该 cid_number 记录)。
    fn is_registered_clearing_node(bank: &AccountId) -> bool {
        let Some((cid_number, _account_name)) = Self::account_info(bank) else {
            return false;
        };
        let key: offchain::InstitutionCidNumber = match cid_number.try_into() {
            Ok(b) => b,
            Err(_) => return false,
        };
        offchain::pallet::ClearingBankNodes::<Runtime>::contains_key(&key)
    }
}

impl offchain::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type MaxBatchSize = ConstU32<100_000>;
    type MaxBatchSignatureLength = ConstU32<128>;
    type InstitutionAsset = RuntimeInstitutionAsset;
    type CidAccountQuery = MultisigCidAccountQuery;
    type WeightInfo = offchain::weights::SubstrateWeight<Runtime>;
}

pub struct EnsureNrcAdmin;

#[cfg(feature = "runtime-benchmarks")]
fn seed_benchmark_public_admin_account(
    cid_number: &'static str,
    institution_code: primitives::cid::code::InstitutionCode,
    raw_admins: &[[u8; 32]],
) -> Result<AccountId, ()> {
    let first_admin = AccountId::new(raw_admins.first().copied().ok_or(())?);
    let admins: public_admins::AdminsOf<Runtime> = raw_admins
        .iter()
        .map(|raw_admin| AccountId::new(*raw_admin))
        .collect::<Vec<_>>()
        .try_into()
        .map_err(|_| ())?;
    let cid_number: admin_primitives::AdminCidNumber =
        cid_number.as_bytes().to_vec().try_into().map_err(|_| ())?;
    public_admins::AdminAccounts::<Runtime>::insert(
        cid_number,
        admin_primitives::InstitutionAdmins {
            institution_code,
            admins,
        },
    );
    Ok(first_admin)
}

#[cfg(feature = "runtime-benchmarks")]
fn seed_benchmark_joint_admins_origin() -> Result<RuntimeOrigin, ()> {
    let nrc = primitives::cid::china::china_cb::CHINA_CB
        .first()
        .ok_or(())?;
    let admin = seed_benchmark_public_admin_account(
        nrc.cid_number,
        primitives::cid::code::NRC,
        nrc.admins,
    )?;
    for entry in primitives::cid::china::china_cb::CHINA_CB.iter().skip(1) {
        seed_benchmark_public_admin_account(
            entry.cid_number,
            primitives::cid::code::PRC,
            entry.admins,
        )?;
    }
    for entry in primitives::cid::china::china_ch::CHINA_CH.iter() {
        seed_benchmark_public_admin_account(
            entry.cid_number,
            primitives::cid::code::PRB,
            entry.admins,
        )?;
    }

    Ok(RuntimeOrigin::from(frame_system::RawOrigin::Signed(admin)))
}

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
        seed_benchmark_joint_admins_origin()
    }
}

pub(crate) fn is_nrc_admin(who: &AccountId) -> bool {
    let nrc_cid_number = primitives::cid::china::china_cb::CHINA_CB
        .first()
        .map(|n| n.cid_number.as_bytes())
        .expect("NRC CID must exist");

    // 创世后只信任链上管理员治理模块中的统一账户表。
    RuntimeInstitutionAdminQuery::is_institution_admin(
        votingengine::types::NRC,
        nrc_cid_number,
        who,
    )
}

/// 联合提案发起权限：国家储委会（CHINA_CB[0]）+ 43个省储委会（CHINA_CB[1..44]）。
pub struct EnsureJointProposer;

impl EnsureOrigin<RuntimeOrigin> for EnsureJointProposer {
    type Success = AccountId;

    fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
        frame_system::EnsureSigned::<AccountId>::try_origin(o)
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn try_successful_origin() -> Result<RuntimeOrigin, ()> {
        seed_benchmark_joint_admins_origin()
    }
}

impl resolution_issuance::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type ProposeOrigin = EnsureJointProposer;
    type RecipientSetOrigin = frame_system::EnsureRoot<AccountId>;
    // 维护入口只允许 root 操作暂停与短期执行记录清理。
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
    fn execute_runtime_code(
        code: &[u8],
        pow_params: pow_difficulty::PowDifficultyParams,
        activate_at: u32,
    ) -> DispatchResult {
        #[cfg(feature = "runtime-benchmarks")]
        {
            // benchmark 需要衡量治理编排本身的真实路径，
            // 但不应真的改写 runtime :code 存储，因此这里使用成功的 no-op 执行器。
            return if code.is_empty() || pow_params.validate().is_err() || activate_at == 0 {
                Err(sp_runtime::DispatchError::Other("empty runtime code"))
            } else {
                Ok(())
            };
        }

        #[cfg(not(feature = "runtime-benchmarks"))]
        {
            super::PowDifficulty::stage_params(pow_params, activate_at)?;
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
    pub const LegislationMaxPendingActivations: u32 = 100;
}

impl legislation_yuan::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    // 立法投票引擎接真实 legislation-vote sub-pallet(ADR-027 第2步),投票端到端流程打通。
    type LegislationVoteEngine = LegislationVote;
    type InstitutionCidQuery = RuntimeInstitutionCidQuery;
    type MaxTitleLen = LegislationMaxTitleLen;
    type MaxTextLen = LegislationMaxTextLen;
    type MaxClausesPerArticle = LegislationMaxClausesPerArticle;
    type MaxArticlesPerSection = LegislationMaxArticlesPerSection;
    type MaxSectionsPerChapter = LegislationMaxSectionsPerChapter;
    type MaxChaptersPerLaw = LegislationMaxChaptersPerLaw;
    type MaxLawsPerScope = LegislationMaxLawsPerScope;
    type MaxPendingActivations = LegislationMaxPendingActivations;
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
    type MaxAutoFinalizeWeightPerBlock = votingengine::BlockWeightFraction<Runtime, 4>;
    type MaxExecutionWeightPerBlock = votingengine::BlockWeightFraction<Runtime, 4>;
    type MaxCleanupWeightPerBlock = votingengine::BlockWeightFraction<Runtime, 8>;
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
    type MaxCleanupActivationsPerBlock = ConstU32<64>;
    type CleanupKeysPerStep = ConstU32<256>;
    type CitizenIdentityReader = RuntimeCitizenIdentityReader;
    type JointVoteResultCallback = RuntimeJointVoteResultCallback;
    // 内部投票终态回调注册 5 个顶层槽位；公权/私权机构生命周期共用一个 tuple 槽位，
    // 个人多签生命周期和个人多签管理员共用一个 tuple 槽位。
    // 顺序按调用频率降序:transfer / multisig manage 类业务最频繁,
    // grandpa key 替换最稀有放最后(tuple iterate 时命中越早越省 gas)。
    // 每个 Executor 通过 MODULE_TAG 前缀 + 独立存储键互斥认领本模块提案,
    // 非己方提案直接 Ok(()) skip,顺序不影响行为正确性。
    type InternalVoteResultCallback = (
        multisig::InternalVoteExecutor<Runtime>,
        (
            public_manage::InternalVoteExecutor<Runtime>,
            private_manage::InternalVoteExecutor<Runtime>,
        ),
        (
            personal_manage::InternalVoteExecutor<Runtime>,
            personal_admins::InternalVoteExecutor<Runtime>,
        ),
        resolution_destroy::InternalVoteExecutor<Runtime>,
        grandpakey_change::InternalVoteExecutor<Runtime>,
    );
    type InternalAdminProvider = RuntimeInternalAdminProvider;
    type InternalAdminsLenProvider = RuntimeInternalAdminsLenProvider;
    type MaxAdminsPerInstitution = MaxAdminsPerInstitution;
    type TimeProvider = pallet_timestamp::Pallet<Runtime>;
    type WeightInfo = votingengine::weights::SubstrateWeight<Runtime>;
    // 四类 timeout / cleanup / mode 终态副作用通过递归 Track tuple 派发。
    type TrackHandlers = (
        InternalVote,
        (JointVote, (LegislationVote, (ElectionVote, ()))),
    );
    // 立法投票(ADR-027):终态业务回调接 legislation-yuan，Track 接 legislation-vote。
    // ProposalOwner 决定由法律、任免或预算业务认领；B1 先装配法律业务壳。
    type LegislationVoteResultCallback = (LegislationYuan,);
    type ElectionVoteResultCallback = ElectionVote;
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
    type MaxElectionOfficeCodeLen = ConstU32<64>;
    type MaxElectionCandidates = ConstU32<256>;
    // 互选选民就是目标机构完整 admins 快照，边界必须与管理员真源一致。
    type MaxMutualVoters = MaxAdminsPerInstitution;
    type WeightInfo = election_vote::weights::SubstrateWeight<Runtime>;
}

impl election_campaign::Config for Runtime {}

impl legislation_vote::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = legislation_vote::weights::SubstrateWeight<Runtime>;
}

impl pow_difficulty::Config for Runtime {
    type WeightInfo = pow_difficulty::weights::SubstrateWeight<Runtime>;
}

frame_support::parameter_types! {
    pub const MaxDeclarationLen: u32 = 2048;
}

/// 创世机构 seeding 注入实现:runtime 侧调用 institution::build。
/// Runtime 本就实现 public_manage/public_admins::Config,天然满足 build 的治理 where 约束,
/// 因此治理耦合留在 runtime 层,不再作为 genesis pallet Config 的 supertrait。
pub struct RuntimeGenesisSeeder;
impl genesis_pallet::GenesisInstitutionSeeder for RuntimeGenesisSeeder {
    fn seed() {
        genesis_pallet::institution::build::<Runtime>();
    }
}

impl genesis_pallet::Config for Runtime {
    type WeightInfo = genesis_pallet::weights::SubstrateWeight<Runtime>;
    type MaxDeclarationLen = MaxDeclarationLen;
    type InstitutionSeeder = RuntimeGenesisSeeder;
}

pub struct RuntimeInternalAdminProvider;

impl votingengine::InternalAdminProvider<AccountId> for RuntimeInternalAdminProvider {
    fn is_institution_admin(
        institution_code: votingengine::types::InstitutionCode,
        cid_number: &[u8],
        who: &AccountId,
    ) -> bool {
        RuntimeInstitutionAdminQuery::is_institution_admin(institution_code, cid_number, who)
    }

    fn get_institution_admins(
        institution_code: votingengine::types::InstitutionCode,
        cid_number: &[u8],
    ) -> Option<alloc::vec::Vec<AccountId>> {
        RuntimeInstitutionAdminQuery::institution_admins(institution_code, cid_number)
    }

    fn is_pending_personal_admin(personal_account: AccountId, who: &AccountId) -> bool {
        RuntimeAdminAccountQuery::is_pending_account_admin_for_snapshot(
            votingengine::types::PMUL,
            personal_account,
            who,
        )
    }

    fn get_pending_personal_admins(
        personal_account: AccountId,
    ) -> Option<alloc::vec::Vec<AccountId>> {
        RuntimeAdminAccountQuery::pending_account_admins_for_snapshot(
            votingengine::types::PMUL,
            personal_account,
        )
    }

    fn is_personal_admin(personal_account: AccountId, who: &AccountId) -> bool {
        RuntimeAdminAccountQuery::is_active_account_admin(
            votingengine::types::PMUL,
            personal_account,
            who,
        )
    }

    fn get_personal_admins(personal_account: AccountId) -> Option<Vec<AccountId>> {
        RuntimeAdminAccountQuery::active_account_admins(votingengine::types::PMUL, personal_account)
    }

    fn legal_representative(cid_number: &[u8]) -> Option<AccountId> {
        <RuntimeInstitutionLegalRepresentativeQuery as entity_primitives::InstitutionLegalRepresentativeQuery<AccountId>>::legal_representative(
            cid_number,
        )
    }

    fn constitution_guard_members() -> Vec<AccountId> {
        <public_manage::Pallet<Runtime> as InstitutionRoleQuery<AccountId>>::active_accounts_for_role(
            primitives::cid::china::china_sf::CHINA_SF[0]
                .cid_number
                .as_bytes(),
            primitives::governance_skeleton::ROLE_CODE_CONSTITUTION_GUARD,
        )
    }
}

pub struct RuntimeInternalAdminsLenProvider;

impl votingengine::InternalAdminsLenProvider<AccountId> for RuntimeInternalAdminsLenProvider {
    fn institution_admins_len(
        institution_code: votingengine::types::InstitutionCode,
        cid_number: &[u8],
    ) -> Option<u32> {
        RuntimeInstitutionAdminQuery::institution_admins_len(institution_code, cid_number)
    }

    fn personal_admins_len(personal_account: AccountId) -> Option<u32> {
        RuntimeAdminAccountQuery::active_account_admins_len(
            votingengine::types::PMUL,
            personal_account,
        )
    }
}

pub struct RuntimeCitizenIdentityReader;

impl votingengine::CitizenIdentityReader<AccountId> for RuntimeCitizenIdentityReader {
    fn can_vote(who: &AccountId, scope: &citizen_identity::PopulationScope) -> bool {
        <citizen_identity::Pallet<Runtime> as citizen_identity::CitizenIdentityProvider<
            AccountId,
        >>::can_vote(who, scope)
    }

    fn can_be_candidate(who: &AccountId, scope: &citizen_identity::PopulationScope) -> bool {
        <citizen_identity::Pallet<Runtime> as citizen_identity::CitizenIdentityProvider<
            AccountId,
        >>::can_be_candidate(who, scope)
    }

    fn population_count(scope: &citizen_identity::PopulationScope) -> u64 {
        <citizen_identity::Pallet<Runtime> as citizen_identity::CitizenIdentityProvider<
            AccountId,
        >>::population_count(scope)
    }

    fn create_population_snapshot(
        scope: &citizen_identity::PopulationScope,
    ) -> Result<(u64, u64), sp_runtime::DispatchError> {
        <citizen_identity::Pallet<Runtime> as citizen_identity::CitizenIdentityProvider<
            AccountId,
        >>::create_population_snapshot(scope)
    }

    fn can_vote_at(who: &AccountId, snapshot_id: u64) -> bool {
        <citizen_identity::Pallet<Runtime> as citizen_identity::CitizenIdentityProvider<
            AccountId,
        >>::can_vote_at(who, snapshot_id)
    }

    fn release_population_snapshot(snapshot_id: u64) {
        <citizen_identity::Pallet<Runtime> as citizen_identity::CitizenIdentityProvider<
            AccountId,
        >>::release_population_snapshot(snapshot_id)
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn benchmark_seed_identity(who: &AccountId, scope: &citizen_identity::PopulationScope) {
        use citizen_identity::{
            CandidateIdentity, CandidateIdentityByAccount, CitizenStatus, CountryVotingCount,
            NextEligibilityRevision, VotingEligibilityVersion, VotingEligibilityVersionCount,
            VotingEligibilityVersions, VotingIdentity, VotingIdentityByAccount,
        };

        // citizen-identity 按 timestamp 校验护照窗口；benchmark externalities 的
        // 创世时间为 0，先推进到稳定的 2027 年时间点。
        pallet_timestamp::Pallet::<Runtime>::set_timestamp(1_800_000_000_000);
        let now = frame_system::Pallet::<Runtime>::block_number();
        let identity = VotingIdentity {
            cid_number: b"benchmark-citizen"
                .to_vec()
                .try_into()
                .expect("bounded CID"),
            passport_valid_from: 19700101,
            passport_valid_until: 29991231,
            citizen_status: CitizenStatus::Normal,
            residence_province_code: Default::default(),
            residence_city_code: Default::default(),
            residence_town_code: Default::default(),
            updated_at: now,
        };
        let revision = NextEligibilityRevision::<Runtime>::get().saturating_add(1);
        let version_index = VotingEligibilityVersionCount::<Runtime>::get(who);
        if version_index > 0 {
            VotingEligibilityVersions::<Runtime>::mutate(
                who,
                version_index.saturating_sub(1),
                |version| {
                    if let Some(version) = version {
                        version.valid_until_revision = Some(revision);
                    }
                },
            );
        }
        VotingEligibilityVersions::<Runtime>::insert(
            who,
            version_index,
            VotingEligibilityVersion {
                identity: identity.clone(),
                valid_from_revision: revision,
                valid_until_revision: None,
            },
        );
        VotingEligibilityVersionCount::<Runtime>::insert(who, version_index.saturating_add(1));
        NextEligibilityRevision::<Runtime>::put(revision);
        VotingIdentityByAccount::<Runtime>::insert(who, identity);
        CandidateIdentityByAccount::<Runtime>::insert(
            who,
            CandidateIdentity {
                birth_province_code: Default::default(),
                birth_city_code: Default::default(),
                birth_town_code: Default::default(),
                citizen_full_name: b"benchmark".to_vec().try_into().expect("bounded name"),
                citizen_sex: citizen_identity::CitizenSex::Male,
                birth_date: 20000101,
                updated_at: now,
            },
        );
        match scope {
            citizen_identity::PopulationScope::Country => CountryVotingCount::<Runtime>::put(1),
            citizen_identity::PopulationScope::Province(province) => {
                citizen_identity::ProvinceVotingCount::<Runtime>::insert(province, 1)
            }
            citizen_identity::PopulationScope::City(province, city) => {
                citizen_identity::CityVotingCount::<Runtime>::insert((province, city), 1)
            }
            citizen_identity::PopulationScope::Town(province, city, town) => {
                citizen_identity::TownVotingCount::<Runtime>::insert((province, city, town), 1)
            }
        }
    }
}
// pallet_assets 内核接入(ADR-011 第八节)+ OnchainIssuance 外壳配置
//
// pallet_assets 是用户代币的内核 storage / 资产记账实现,
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
    /// 外部 extrinsic 全部被 RuntimeCallFilter reject,这里 origin 设啥不影响实际入口。
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

/// NRC 费用账户(fee_account)— 创建费 1000 GMB 收款用。
///
/// 复用既有 `RuntimeNrcAccountProvider`(它实现 onchain::NrcAccountProvider,
/// 也返回 fee_account),通过为同 struct 再实现 onchain_issuance 自己的 trait 完成桥接,语义一致。
impl onchain_issuance::pallet::NrcFeeAccountProvider<AccountId> for RuntimeNrcAccountProvider {
    fn nrc_fee_account() -> Option<AccountId> {
        <RuntimeNrcAccountProvider as onchain::NrcAccountProvider<AccountId>>::nrc_account()
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
    /// Currency 必须实现 ReservableCurrency(ADR-011 v2 第六节押金机制),
    /// pallet_balances 默认实现该 trait,直接接 Balances 即可。
    type Currency = Balances;
    /// pallet_assets 内核类型绑定。onchain_issuance 通过该类型调内核 create / mint_into 等内部 API,
    /// 不走原生 extrinsic(已被 RuntimeCallFilter 拦截)。
    type Assets = Assets;
    type InstitutionQuery = RuntimeInstitutionQuery;
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
