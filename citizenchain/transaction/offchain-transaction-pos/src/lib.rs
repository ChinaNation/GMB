#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    ensure,
    pallet_prelude::*,
    storage::{with_transaction, TransactionOutcome},
    traits::ConstU32,
    traits::Currency,
    weights::Weight,
    Blake2_128Concat, PalletId,
};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::traits::{AccountIdConversion, SaturatedConversion, Saturating, Zero};
use sp_std::vec::Vec;

use primitives::china::china_ch::{
    shenfen_fee_id_to_bytes as shengbank_shenfen_fee_id_to_bytes,
    shenfen_id_to_fixed48 as shengbank_pallet_id_to_bytes, CHINA_CH,
};
use voting_engine_system::{
    internal_vote::ORG_PRB, InstitutionPalletId, InternalVoteEngine, PROPOSAL_KIND_INTERNAL,
    STATUS_PASSED, STATUS_REJECTED,
};

type BalanceOf<T> =
    <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

const OFFCHAIN_RATE_BP_MIN: u32 = 1; // 0.01%
const OFFCHAIN_RATE_BP_MAX: u32 = 10; // 0.1%
const PACK_TX_THRESHOLD: u64 = 100_000;
const PACK_BLOCK_THRESHOLD: u32 = primitives::pow_const::BLOCKS_PER_HOUR as u32; // 60分钟
const OFFCHAIN_MIN_FEE_FEN: u128 = primitives::core_const::OFFCHAIN_MIN_FEE;
const FEE_ADDRESS_MIN_RESERVE_FEN: u128 = 111_111; // 1111.11元
const FEE_SWEEP_MAX_PERCENT: u128 = 80; // 单次最多可提可用余额的80%
const INIT_RELAY_SUBMITTERS_COUNT: u32 = 3; // 初始化白名单固定3个提交账户
const VERIFY_KEY_ROTATION_DELAY_BLOCKS: u32 = primitives::pow_const::BLOCKS_PER_HOUR as u32; // 新密钥延迟生效（1小时）
const CLEARING_INSTITUTION_SWITCH_INTERVAL_BLOCKS: u64 = primitives::pow_const::BLOCKS_PER_YEAR; // 每年最多更换1次
const BP_DENOMINATOR: u128 = 10_000;
const PROCESSED_TX_RETENTION_BLOCKS: u64 = primitives::pow_const::BLOCKS_PER_YEAR;
const QUEUED_BATCH_RETENTION_BLOCKS: u64 = primitives::pow_const::BLOCKS_PER_YEAR;
const BATCH_SUMMARY_RETENTION_BLOCKS: u64 = primitives::pow_const::BLOCKS_PER_YEAR;
const MAX_QUEUE_RETRY_COUNT: u32 = 50;
const EMERGENCY_ROTATE_MIN_ADMINS: u32 = 2;

fn institution_pallet_address(institution: InstitutionPalletId) -> Option<[u8; 32]> {
    CHINA_CH
        .iter()
        .find(|n| shengbank_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
        .map(|n| n.duoqian_address)
}

fn institution_shenfen_fee_id(institution: InstitutionPalletId) -> Option<[u8; 8]> {
    CHINA_CH
        .iter()
        .find(|n| shengbank_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
        .and_then(|n| shengbank_shenfen_fee_id_to_bytes(n.shenfen_fee_id))
}

fn institution_t2_code(institution: InstitutionPalletId) -> Option<[u8; 2]> {
    let node = CHINA_CH
        .iter()
        .find(|n| shengbank_pallet_id_to_bytes(n.shenfen_id) == Some(institution))?;
    let segment = node.shenfen_id.split('-').nth(1)?;
    let raw = segment.as_bytes();
    if raw.len() < 2 {
        return None;
    }
    if !raw[0].is_ascii_uppercase() || !raw[1].is_ascii_uppercase() {
        return None;
    }
    let mut t2 = [0u8; 2];
    t2.copy_from_slice(&raw[..2]);
    Some(t2)
}

fn round_div(numerator: u128, denominator: u128) -> Option<u128> {
    if denominator == 0 {
        return None;
    }
    let quotient = numerator / denominator;
    let remainder = numerator % denominator;
    let should_round_up = remainder >= (denominator.saturating_add(1) / 2);
    if should_round_up {
        Some(quotient.saturating_add(1))
    } else {
        Some(quotient)
    }
}

fn calc_offchain_fee_fen(amount_fen: u128, rate_bp: u32) -> u128 {
    let by_rate = match amount_fen.checked_mul(rate_bp as u128) {
        Some(numerator) => {
            round_div(numerator, BP_DENOMINATOR).expect("BP_DENOMINATOR must be non-zero; qed")
        }
        None => u128::MAX,
    };
    by_rate.max(OFFCHAIN_MIN_FEE_FEN)
}

/// 链下批次签名验证器（由 runtime 对接验证算法）。
pub trait OffchainBatchVerifier {
    fn verify(verify_key: &[u8], message: &[u8], signature: &[u8]) -> bool;
}

impl OffchainBatchVerifier for () {
    fn verify(_verify_key: &[u8], _message: &[u8], _signature: &[u8]) -> bool {
        false
    }
}

/// 付款源地址保护：用于禁止制度保留地址作为转出源。
pub trait ProtectedSourceChecker<AccountId> {
    fn is_protected(account: &AccountId) -> bool;
}

impl<AccountId> ProtectedSourceChecker<AccountId> for () {
    fn is_protected(_account: &AccountId) -> bool {
        false
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct RateProposalAction {
    pub institution: InstitutionPalletId,
    pub new_rate_bp: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct VerifyKeyProposalAction<BoundedBytes> {
    pub institution: InstitutionPalletId,
    pub new_key: BoundedBytes,
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct PendingVerifyKey<BoundedBytes, BlockNumber> {
    pub key: BoundedBytes,
    pub activate_at: BlockNumber,
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum VerifyKeyRotationStage {
    Idle,
    Scheduled,
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct VerifyKeyRotationStatus<BlockNumber> {
    pub stage: VerifyKeyRotationStage,
    pub activate_at: Option<BlockNumber>,
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct SweepProposalAction<Balance> {
    pub institution: InstitutionPalletId,
    pub amount: Balance,
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct RelaySubmittersProposalAction<BoundedSubmitters> {
    pub institution: InstitutionPalletId,
    pub submitters: BoundedSubmitters,
}

pub trait WeightInfo {
    fn submit_offchain_batch(items: u32) -> Weight;
    fn enqueue_offchain_batch(items: u32) -> Weight;
    fn process_queued_batch(items: u32) -> Weight;
}

impl WeightInfo for () {
    fn submit_offchain_batch(items: u32) -> Weight {
        frame_support::weights::constants::RocksDbWeight::get().reads_writes(9, 8 + items as u64)
    }

    fn enqueue_offchain_batch(items: u32) -> Weight {
        frame_support::weights::constants::RocksDbWeight::get().reads_writes(9, 4 + items as u64)
    }

    fn process_queued_batch(items: u32) -> Weight {
        frame_support::weights::constants::RocksDbWeight::get().reads_writes(8, 4 + items as u64)
    }
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use voting_engine_system::InternalAdminProvider;

    #[pallet::config]
    pub trait Config: frame_system::Config + voting_engine_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Currency: Currency<Self::AccountId>;

        #[pallet::constant]
        type MaxVerifyKeyLen: Get<u32>;

        #[pallet::constant]
        type MaxBatchSize: Get<u32>;

        #[pallet::constant]
        type MaxBatchSignatureLength: Get<u32>;

        #[pallet::constant]
        type MaxRelaySubmitters: Get<u32>;

        /// 中文注释：内部投票引擎，返回真实 proposal_id。
        type InternalVoteEngine: voting_engine_system::InternalVoteEngine<Self::AccountId>;

        type OffchainBatchVerifier: OffchainBatchVerifier;
        type ProtectedSourceChecker: ProtectedSourceChecker<Self::AccountId>;
        type WeightInfo: WeightInfo;
    }

    pub type VerifyKeyOf<T> = BoundedVec<u8, <T as Config>::MaxVerifyKeyLen>;
    pub type BatchSignatureOf<T> = BoundedVec<u8, <T as Config>::MaxBatchSignatureLength>;
    pub type RelaySubmittersOf<T> =
        BoundedVec<<T as frame_system::Config>::AccountId, <T as Config>::MaxRelaySubmitters>;

    #[derive(
        Encode,
        Decode,
        DecodeWithMemTracking,
        Clone,
        RuntimeDebug,
        TypeInfo,
        MaxEncodedLen,
        PartialEq,
        Eq,
    )]
    pub struct OffchainBatchItem<AccountId, Balance, Hash> {
        /// 链下交易唯一ID（防重放）
        pub tx_id: Hash,
        /// 付款方
        pub payer: AccountId,
        /// 收款方
        pub recipient: AccountId,
        /// 链下主交易金额（不作为链上手续费计费基数）
        pub transfer_amount: Balance,
        /// 链下手续费金额（链上手续费仅对这个字段计费）
        pub offchain_fee_amount: Balance,
    }

    pub type BatchItemOf<T> = OffchainBatchItem<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        <T as frame_system::Config>::Hash,
    >;
    pub type BatchOf<T> = BoundedVec<BatchItemOf<T>, <T as Config>::MaxBatchSize>;
    pub type QueuedBatchRecordOf<T> = QueuedBatchRecord<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        <T as frame_system::Config>::Hash,
        BlockNumberFor<T>,
        BatchOf<T>,
        BatchSignatureOf<T>,
    >;

    #[derive(
        Encode,
        Decode,
        DecodeWithMemTracking,
        Clone,
        RuntimeDebug,
        TypeInfo,
        MaxEncodedLen,
        PartialEq,
        Eq,
    )]
    pub struct BatchSummary<AccountId, Balance, BlockNumber> {
        pub submitter: AccountId,
        pub institution: InstitutionPalletId,
        pub batch_seq: u64,
        pub batch_hash: [u8; 32],
        pub signer_key_hash: [u8; 32],
        pub item_count: u32,
        pub total_transfer_amount: Balance,
        pub total_offchain_fee_amount: Balance,
        pub submitted_at: BlockNumber,
    }

    #[derive(
        Encode,
        Decode,
        DecodeWithMemTracking,
        Clone,
        RuntimeDebug,
        TypeInfo,
        MaxEncodedLen,
        PartialEq,
        Eq,
    )]
    pub enum QueuedBatchStatus {
        Pending,
        Processed,
        Failed,
        Cancelled,
    }

    #[derive(
        Encode,
        Decode,
        DecodeWithMemTracking,
        Clone,
        RuntimeDebug,
        TypeInfo,
        MaxEncodedLen,
        PartialEq,
        Eq,
    )]
    pub enum QueuedBatchLastError {
        PrecheckFailed,
        ExecutionFailed,
        WaitingForPriorBatch,
        PackThresholdNotReached,
        Cancelled,
    }

    #[derive(
        Encode,
        Decode,
        DecodeWithMemTracking,
        Clone,
        RuntimeDebug,
        TypeInfo,
        MaxEncodedLen,
        PartialEq,
        Eq,
    )]
    pub struct QueuedBatchRecord<AccountId, Balance, Hash, BlockNumber, Batch, BatchSignature> {
        pub institution: InstitutionPalletId,
        pub batch_seq: u64,
        pub batch: Batch,
        pub batch_signature: BatchSignature,
        pub rate_bp_snapshot: u32,
        pub status: QueuedBatchStatus,
        pub retry_count: u32,
        pub last_error: Option<QueuedBatchLastError>,
        pub enqueued_by: AccountId,
        pub enqueued_at: BlockNumber,
        pub last_attempt_at: Option<BlockNumber>,
        pub processed_at: Option<BlockNumber>,
        pub fee_sum_snapshot: Balance,
        pub marker_tx_id: Hash,
        pub verify_key_epoch_snapshot: u64,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// 各省储行链下清算费率（bp，范围1~10）。
    #[pallet::storage]
    #[pallet::getter(fn rate_bp_of)]
    pub type InstitutionRateBp<T> =
        StorageMap<_, Blake2_128Concat, InstitutionPalletId, u32, ValueQuery>;

    /// 各省储行上次打包触发区块。
    #[pallet::storage]
    #[pallet::getter(fn last_pack_block_of)]
    pub type LastPackBlock<T: Config> =
        StorageMap<_, Blake2_128Concat, InstitutionPalletId, BlockNumberFor<T>, ValueQuery>;

    /// 各省储行批次序号（u64，机构内单调递增）。
    #[pallet::storage]
    #[pallet::getter(fn last_batch_seq_of)]
    pub type LastBatchSeq<T> =
        StorageMap<_, Blake2_128Concat, InstitutionPalletId, u64, ValueQuery>;

    /// 各省储行下一可入队批次序号（与执行序号分离，支持多批次缓冲）。
    #[pallet::storage]
    #[pallet::getter(fn next_enqueue_batch_seq_of)]
    pub type NextEnqueueBatchSeq<T> =
        StorageMap<_, Blake2_128Concat, InstitutionPalletId, u64, ValueQuery>;

    /// 各省储行批次提交白名单账户（中继提交账户）。
    #[pallet::storage]
    #[pallet::getter(fn relay_submitters_of)]
    pub type RelaySubmitters<T: Config> =
        StorageMap<_, Blake2_128Concat, InstitutionPalletId, RelaySubmittersOf<T>, OptionQuery>;

    /// 各省储行链下交易验证密钥（由内部投票通过后更新）。
    #[pallet::storage]
    #[pallet::getter(fn verify_key_of)]
    pub type VerifyKeys<T: Config> =
        StorageMap<_, Blake2_128Concat, InstitutionPalletId, VerifyKeyOf<T>, OptionQuery>;

    /// 验签密钥纪元（每次实际切换生效后递增），用于失效旧纪元签名的已入队批次。
    #[pallet::storage]
    pub type VerifyKeyEpoch<T> =
        StorageMap<_, Blake2_128Concat, InstitutionPalletId, u64, ValueQuery>;

    /// 各省储行待生效验证密钥（双轨换钥）。
    #[pallet::storage]
    #[pallet::getter(fn pending_verify_key_of)]
    pub type PendingVerifyKeys<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        InstitutionPalletId,
        PendingVerifyKey<VerifyKeyOf<T>, BlockNumberFor<T>>,
        OptionQuery,
    >;

    /// 验证密钥轮换状态（前端可直接查询）。
    #[pallet::storage]
    #[pallet::getter(fn rotation_status_of)]
    pub type VerifyKeyRotationStatuses<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        InstitutionPalletId,
        VerifyKeyRotationStatus<BlockNumberFor<T>>,
        OptionQuery,
    >;

    /// 已处理链下 tx_id 防重放（按省标识 T2 + tx_id 维度，窗口约1年）。
    /// 链下系统必须保证 tx_id 全局唯一；链上只提供窗口内强防重。
    #[pallet::storage]
    pub type ProcessedOffchainTx<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, [u8; 2], Blake2_128Concat, T::Hash, bool, ValueQuery>;

    /// 已处理链下 tx_id 的写入高度（用于过期窗口控制）。
    #[pallet::storage]
    pub type ProcessedOffchainTxAt<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        [u8; 2],
        Blake2_128Concat,
        T::Hash,
        BlockNumberFor<T>,
        OptionQuery,
    >;

    /// 已处理 tx 的顺序日志（用于 on_idle 有界清理）。
    #[pallet::storage]
    pub type ProcessedTxLog<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, ([u8; 2], T::Hash, BlockNumberFor<T>), OptionQuery>;

    #[pallet::storage]
    pub type NextProcessedTxLogId<T> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    pub type ProcessedTxPruneCursor<T> = StorageValue<_, u64, ValueQuery>;

    /// 费率治理提案动作。
    #[pallet::storage]
    #[pallet::getter(fn rate_action_by_proposal)]
    pub type RateProposalActions<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, RateProposalAction, OptionQuery>;

    /// 验证密钥治理提案动作。
    #[pallet::storage]
    #[pallet::getter(fn verify_key_action_by_proposal)]
    pub type VerifyKeyProposalActions<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, VerifyKeyProposalAction<VerifyKeyOf<T>>, OptionQuery>;

    /// fee_address 划转治理提案动作。
    #[pallet::storage]
    #[pallet::getter(fn sweep_action_by_proposal)]
    pub type SweepProposalActions<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, SweepProposalAction<BalanceOf<T>>, OptionQuery>;

    /// relay 提交者白名单治理提案动作。
    #[pallet::storage]
    #[pallet::getter(fn relay_submitters_action_by_proposal)]
    pub type RelaySubmittersProposalActions<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        RelaySubmittersProposalAction<RelaySubmittersOf<T>>,
        OptionQuery,
    >;

    /// 批次ID。
    #[pallet::storage]
    #[pallet::getter(fn next_batch_id)]
    pub type NextBatchId<T> = StorageValue<_, u64, ValueQuery>;

    /// 批次摘要。
    #[pallet::storage]
    #[pallet::getter(fn batch_summary_by_id)]
    pub type BatchSummaries<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        BatchSummary<T::AccountId, BalanceOf<T>, BlockNumberFor<T>>,
        OptionQuery,
    >;

    /// 队列批次ID（持久化出队的主键）。
    #[pallet::storage]
    #[pallet::getter(fn next_queued_batch_id)]
    pub type NextQueuedBatchId<T> = StorageValue<_, u64, ValueQuery>;

    /// 持久化队列中的链下批次（失败后不丢失，可重试）。
    #[pallet::storage]
    #[pallet::getter(fn queued_batch_by_id)]
    pub type QueuedBatches<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, QueuedBatchRecordOf<T>, OptionQuery>;

    /// 待处理队列中的 tx_id 索引（用于跨入队批次防重）。
    #[pallet::storage]
    pub type QueuedTxIndex<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, [u8; 2], Blake2_128Concat, T::Hash, u64, OptionQuery>;

    /// 存在待生效 verify key 的机构索引（避免 on_initialize 全量扫描）。
    #[pallet::storage]
    #[pallet::getter(fn pending_rotation_institutions)]
    pub type PendingRotationInstitutions<T> =
        StorageValue<_, BoundedVec<InstitutionPalletId, ConstU32<64>>, ValueQuery>;

    #[pallet::storage]
    pub type QueuedBatchPruneCursor<T> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    pub type BatchSummaryPruneCursor<T> = StorageValue<_, u64, ValueQuery>;

    /// 紧急换钥的管理员确认集合（institution + key_hash 维度）。
    #[pallet::storage]
    pub type EmergencyVerifyKeyApprovals<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        InstitutionPalletId,
        Blake2_128Concat,
        [u8; 32],
        BoundedVec<T::AccountId, ConstU32<16>>,
        ValueQuery,
    >;

    /// 收款账户绑定的链下清算省储行。
    #[pallet::storage]
    #[pallet::getter(fn recipient_clearing_institution)]
    pub type RecipientClearingInstitution<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, InstitutionPalletId, OptionQuery>;

    /// 收款账户上次更换清算省储行的区块高度。
    #[pallet::storage]
    #[pallet::getter(fn recipient_last_switch_at)]
    pub type RecipientClearingInstitutionLastSwitchAt<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BlockNumberFor<T>, OptionQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub initial_rates: Vec<(Vec<u8>, u32)>,
        pub initial_verify_keys: Vec<(Vec<u8>, VerifyKeyOf<T>)>,
        pub initial_relay_submitters: Vec<(Vec<u8>, RelaySubmittersOf<T>)>,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                initial_rates: Vec::new(),
                initial_verify_keys: Vec::new(),
                initial_relay_submitters: Vec::new(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            for (idx, (institution_raw, _)) in self.initial_rates.iter().enumerate() {
                for (other_raw, _) in self.initial_rates.iter().skip(idx + 1) {
                    assert!(
                        institution_raw != other_raw,
                        "Duplicate institution in initial_rates"
                    );
                }
            }
            for (institution_raw, rate_bp) in self.initial_rates.iter() {
                let institution: InstitutionPalletId = institution_raw
                    .as_slice()
                    .try_into()
                    .expect("invalid institution id length in initial_rates");
                assert!(
                    institution_pallet_address(institution).is_some(),
                    "Invalid institution in initial_rates"
                );
                assert!(
                    (*rate_bp >= OFFCHAIN_RATE_BP_MIN) && (*rate_bp <= OFFCHAIN_RATE_BP_MAX),
                    "Invalid rate in initial_rates"
                );
                InstitutionRateBp::<T>::insert(institution, rate_bp);
            }

            for (idx, (institution_raw, _)) in self.initial_verify_keys.iter().enumerate() {
                for (other_raw, _) in self.initial_verify_keys.iter().skip(idx + 1) {
                    assert!(
                        institution_raw != other_raw,
                        "Duplicate institution in initial_verify_keys"
                    );
                }
            }
            for (institution_raw, key) in self.initial_verify_keys.iter() {
                let institution: InstitutionPalletId = institution_raw
                    .as_slice()
                    .try_into()
                    .expect("invalid institution id length in initial_verify_keys");
                assert!(
                    institution_pallet_address(institution).is_some(),
                    "Invalid institution in initial_verify_keys"
                );
                assert!(!key.is_empty(), "Empty verify key in initial_verify_keys");
                VerifyKeys::<T>::insert(institution, key);
                VerifyKeyEpoch::<T>::insert(institution, 1u64);
                VerifyKeyRotationStatuses::<T>::insert(
                    institution,
                    VerifyKeyRotationStatus {
                        stage: VerifyKeyRotationStage::Idle,
                        activate_at: None,
                    },
                );
            }

            for (idx, (institution_raw, _)) in self.initial_relay_submitters.iter().enumerate() {
                for (other_raw, _) in self.initial_relay_submitters.iter().skip(idx + 1) {
                    assert!(
                        institution_raw != other_raw,
                        "Duplicate institution in initial_relay_submitters"
                    );
                }
            }
            for (institution_raw, submitters) in self.initial_relay_submitters.iter() {
                let institution: InstitutionPalletId = institution_raw
                    .as_slice()
                    .try_into()
                    .expect("invalid institution id length in initial_relay_submitters");
                assert!(
                    institution_pallet_address(institution).is_some(),
                    "Invalid institution in initial_relay_submitters"
                );
                assert!(
                    submitters.len() as u32 == INIT_RELAY_SUBMITTERS_COUNT,
                    "Empty relay submitters in initial_relay_submitters"
                );
                Pallet::<T>::ensure_valid_relay_submitters(submitters)
                    .expect("invalid relay submitters in genesis");
                RelaySubmitters::<T>::insert(institution, submitters);
            }
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 省储行批次已上链（链上手续费由 fee_address 支付）。
        OffchainBatchSubmitted {
            batch_id: u64,
            institution: InstitutionPalletId,
            submitter: T::AccountId,
            batch_seq: u64,
            batch_hash: [u8; 32],
            signer_key_hash: [u8; 32],
            item_count: u32,
            total_transfer_amount: BalanceOf<T>,
            total_offchain_fee_amount: BalanceOf<T>,
            reason_by_count: bool,
            reason_by_time: bool,
        },
        InstitutionRateProposed {
            proposal_id: u64,
            institution: InstitutionPalletId,
            proposer: T::AccountId,
            new_rate_bp: u32,
        },
        InstitutionRateVoteSubmitted {
            proposal_id: u64,
            voter: T::AccountId,
            approve: bool,
        },
        InstitutionRateUpdated {
            proposal_id: u64,
            institution: InstitutionPalletId,
            rate_bp: u32,
        },
        VerifyKeyProposed {
            proposal_id: u64,
            institution: InstitutionPalletId,
            proposer: T::AccountId,
            key_len: u32,
        },
        VerifyKeyVoteSubmitted {
            proposal_id: u64,
            voter: T::AccountId,
            approve: bool,
        },
        VerifyKeyUpdated {
            proposal_id: u64,
            institution: InstitutionPalletId,
            key_len: u32,
        },
        VerifyKeyRotationScheduled {
            proposal_id: u64,
            institution: InstitutionPalletId,
            key_len: u32,
            activate_at: BlockNumberFor<T>,
        },
        VerifyKeyRotated {
            institution: InstitutionPalletId,
            key_len: u32,
            activated_at: BlockNumberFor<T>,
        },
        VerifyKeyEmergencyRotated {
            institution: InstitutionPalletId,
            key_len: u32,
            activated_at: BlockNumberFor<T>,
            operator: T::AccountId,
        },
        VerifyKeyEmergencyRotationApproval {
            institution: InstitutionPalletId,
            key_hash: [u8; 32],
            approver: T::AccountId,
            approvals: u32,
            required: u32,
        },
        SweepToMainProposed {
            proposal_id: u64,
            institution: InstitutionPalletId,
            proposer: T::AccountId,
            amount: BalanceOf<T>,
        },
        SweepToMainVoteSubmitted {
            proposal_id: u64,
            voter: T::AccountId,
            approve: bool,
        },
        RelaySubmittersProposed {
            proposal_id: u64,
            institution: InstitutionPalletId,
            proposer: T::AccountId,
            count: u32,
        },
        RelaySubmittersVoteSubmitted {
            proposal_id: u64,
            voter: T::AccountId,
            approve: bool,
        },
        RelaySubmittersUpdated {
            proposal_id: u64,
            institution: InstitutionPalletId,
            count: u32,
        },
        InternalProposalExecutionFailed {
            proposal_id: u64,
        },
        SweepToMainExecuted {
            proposal_id: u64,
            institution: InstitutionPalletId,
            amount: BalanceOf<T>,
            reserve_left: BalanceOf<T>,
        },
        RelaySubmittersInitialized {
            institution: InstitutionPalletId,
            count: u32,
        },
        RecipientClearingInstitutionBound {
            recipient: T::AccountId,
            institution: InstitutionPalletId,
            switched: bool,
        },
        OffchainBatchQueued {
            queue_id: u64,
            institution: InstitutionPalletId,
            batch_seq: u64,
            enqueued_by: T::AccountId,
            item_count: u32,
            fee_sum_snapshot: BalanceOf<T>,
        },
        OffchainQueuedBatchRetryFailed {
            queue_id: u64,
            institution: InstitutionPalletId,
            retry_count: u32,
            last_error: QueuedBatchLastError,
        },
        OffchainQueuedBatchDeferred {
            queue_id: u64,
            institution: InstitutionPalletId,
            reason: QueuedBatchLastError,
        },
        OffchainQueuedBatchProcessed {
            queue_id: u64,
            institution: InstitutionPalletId,
            batch_id: u64,
            retry_count: u32,
        },
        OffchainQueuedBatchFailed {
            queue_id: u64,
            institution: InstitutionPalletId,
            retry_count: u32,
            last_error: QueuedBatchLastError,
        },
        OffchainQueuedBatchCancelled {
            queue_id: u64,
            institution: InstitutionPalletId,
            operator: T::AccountId,
        },
        OffchainQueuedBatchInvalidatedByKeyRotation {
            queue_id: u64,
            institution: InstitutionPalletId,
            expected_epoch: u64,
            queued_epoch: u64,
        },
        FailedBatchSkipped {
            queue_id: u64,
            institution: InstitutionPalletId,
            batch_seq: u64,
            operator: T::AccountId,
        },
        QueuedBatchPruned {
            queue_id: u64,
        },
        BatchSummaryPruned {
            batch_id: u64,
        },
        ProcessedTxPruned {
            t2: [u8; 2],
            tx_id: T::Hash,
        },
        ProposalActionPruned {
            proposal_id: u64,
        },
        ProposalExecutionRetried {
            proposal_id: u64,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        InvalidInstitution,
        InvalidRateBp,
        InvalidFeeAmount,
        InvalidTransferAmount,
        SelfTransferNotAllowed,
        TransferAmountTooLarge,
        DuplicateTxIdInBatch,
        InstitutionAccountDecodeFailed,
        ProposalNotFound,
        ProposalKindMismatch,
        ProposalStatusNotPassed,
        ProposalInstitutionMismatch,
        RateProposalNotFound,
        RateProposalAlreadyExecuted,
        VerifyKeyProposalNotFound,
        VerifyKeyProposalAlreadyExecuted,
        SweepProposalNotFound,
        SweepProposalAlreadyExecuted,
        RelaySubmittersProposalNotFound,
        RelaySubmittersProposalAlreadyExecuted,
        UnauthorizedAdmin,
        UnauthorizedSubmitter,
        TxAlreadyProcessed,
        PackThresholdNotReached,
        EmptyBatch,
        VerifyKeyAlreadyInitialized,
        VerifyKeyMissing,
        InvalidBatchSignature,
        ProtectedSource,
        InsufficientFeeReserve,
        SweepAmountExceedsCap,
        InvalidSweepAmount,
        RelaySubmittersNotInitialized,
        RelaySubmitterNotAllowed,
        RelaySubmittersAlreadyInitialized,
        InvalidRelaySubmittersCount,
        DuplicateRelaySubmitter,
        InvalidBatchSeq,
        QueuedBacklogExists,
        RecipientClearingInstitutionNotBound,
        RecipientClearingInstitutionMismatch,
        ClearingInstitutionSwitchTooFrequent,
        QueuedBatchNotFound,
        QueuedBatchAlreadyProcessed,
        QueuedBatchNotProcessed,
        QueuedBatchNotFailed,
        QueuedBatchNotPending,
        QueuedBatchNotSkippable,
        BatchSummaryNotFound,
        ProcessedTxNotFound,
        InvalidVerifyKey,
        VerifyKeyRotationAlreadyScheduled,
        PendingRotationInstitutionsOverflow,
        QueueRetentionNotReached,
        BatchSummaryRetentionNotReached,
        ProcessedTxRetentionNotReached,
        MaxQueueRetryExceeded,
        ProposalActionNotFound,
        ProposalNotPrunable,
        ProposalExecutionRetryNotAllowed,
        CounterOverflow,
        EmergencyRotationAlreadyApproved,
        EmergencyApprovalsOverflow,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 省储行链下批次上链：
        /// - 可由任意中继账户提交（fee_pallet_address 无私钥）；
        /// - 达到 N 或 T 触发条件才允许提交；
        /// - 必须通过“本机构验证密钥”对批次做签名校验；
        /// - 执行时主金额 payer->recipient，链下手续费 payer->fee_pallet_address；
        /// - 本次上链交易的链上手续费由 fee_pallet_address 自动承担。
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::submit_offchain_batch(T::MaxBatchSize::get()))]
        pub fn submit_offchain_batch(
            origin: OriginFor<T>,
            institution: InstitutionPalletId,
            batch_seq: u64,
            batch: BatchOf<T>,
            batch_signature: BatchSignatureOf<T>,
        ) -> DispatchResult {
            let submitter = ensure_signed(origin)?;
            let expected_execute_seq = LastBatchSeq::<T>::get(institution)
                .checked_add(1)
                .ok_or(Error::<T>::CounterOverflow)?;
            let next_enqueue_seq = NextEnqueueBatchSeq::<T>::get(institution);
            ensure!(
                next_enqueue_seq == 0 || next_enqueue_seq <= expected_execute_seq,
                Error::<T>::QueuedBacklogExists
            );
            let rate_bp = Self::ensure_rate_and_institution(institution)?;
            let (t2, verify_key, by_count, by_time) =
                Self::precheck_submit_offchain_batch_with_rate(
                    &submitter,
                    institution,
                    batch_seq,
                    &batch,
                    &batch_signature,
                    rate_bp,
                    true,
                    true,
                )?;
            let _ = with_transaction(|| {
                let inner = Self::execute_batch(
                    &submitter,
                    institution,
                    batch_seq,
                    &batch,
                    t2,
                    &verify_key,
                    by_count,
                    by_time,
                );
                match inner {
                    Ok(batch_id) => TransactionOutcome::Commit(Ok(batch_id)),
                    Err(e) => TransactionOutcome::Rollback(Err(e)),
                }
            })?;
            Ok(())
        }

        /// 省储行安装时初始化默认验证密钥，仅可初始化一次。
        /// 该初始化由机构主账户（pallet_address）执行；
        /// 后续更换必须走内部投票流程（propose_verify_key/vote_verify_key）。
        #[pallet::call_index(5)]
        #[pallet::weight(T::DbWeight::get().reads_writes(2, 2))]
        pub fn init_verify_key(
            origin: OriginFor<T>,
            institution: InstitutionPalletId,
            default_key: VerifyKeyOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let institution_account = Self::institution_account(institution)?;
            ensure!(
                who == institution_account,
                Error::<T>::UnauthorizedSubmitter
            );
            ensure!(
                !VerifyKeys::<T>::contains_key(institution),
                Error::<T>::VerifyKeyAlreadyInitialized
            );
            ensure!(!default_key.is_empty(), Error::<T>::InvalidVerifyKey);
            VerifyKeys::<T>::insert(institution, &default_key);
            VerifyKeyEpoch::<T>::insert(institution, 1u64);
            VerifyKeyRotationStatuses::<T>::insert(
                institution,
                VerifyKeyRotationStatus {
                    stage: VerifyKeyRotationStage::Idle,
                    activate_at: None,
                },
            );
            Self::deposit_event(Event::<T>::VerifyKeyUpdated {
                proposal_id: 0,
                institution,
                key_len: default_key.len() as u32,
            });
            Ok(())
        }

        /// 初始化批次提交白名单（建议初始化 3 个提交账户），仅可初始化一次。
        /// 由机构主账户（pallet_address）执行。
        #[pallet::call_index(8)]
        #[pallet::weight(T::DbWeight::get().reads_writes(2, 1))]
        pub fn init_relay_submitters(
            origin: OriginFor<T>,
            institution: InstitutionPalletId,
            submitters: RelaySubmittersOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let institution_account = Self::institution_account(institution)?;
            ensure!(
                who == institution_account,
                Error::<T>::UnauthorizedSubmitter
            );
            ensure!(
                !RelaySubmitters::<T>::contains_key(institution),
                Error::<T>::RelaySubmittersAlreadyInitialized
            );
            let submitter_count: u32 = submitters.len() as u32;
            ensure!(
                submitter_count == INIT_RELAY_SUBMITTERS_COUNT
                    && submitter_count <= T::MaxRelaySubmitters::get(),
                Error::<T>::InvalidRelaySubmittersCount
            );
            Self::ensure_valid_relay_submitters(&submitters)?;
            RelaySubmitters::<T>::insert(institution, &submitters);
            Self::deposit_event(Event::<T>::RelaySubmittersInitialized {
                institution,
                count: submitter_count,
            });
            Ok(())
        }

        /// 收款方账户绑定链下清算省储行：
        /// - 首次绑定可立即生效；
        /// - 更换绑定时，每 87600 区块最多更换 1 次。
        #[pallet::call_index(9)]
        #[pallet::weight(T::DbWeight::get().reads_writes(3, 2))]
        pub fn bind_clearing_institution(
            origin: OriginFor<T>,
            institution: InstitutionPalletId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                institution_pallet_address(institution).is_some(),
                Error::<T>::InvalidInstitution
            );

            let switched = if let Some(current) = RecipientClearingInstitution::<T>::get(&who) {
                if current == institution {
                    false
                } else {
                    let now = frame_system::Pallet::<T>::block_number();
                    if let Some(last) = RecipientClearingInstitutionLastSwitchAt::<T>::get(&who) {
                        let elapsed: u64 =
                            <BlockNumberFor<T> as sp_runtime::traits::Saturating>::saturating_sub(
                                now, last,
                            )
                            .saturated_into();
                        ensure!(
                            elapsed >= CLEARING_INSTITUTION_SWITCH_INTERVAL_BLOCKS,
                            Error::<T>::ClearingInstitutionSwitchTooFrequent
                        );
                    }
                    RecipientClearingInstitutionLastSwitchAt::<T>::insert(&who, now);
                    true
                }
            } else {
                // 中文注释：首次绑定虽然 switched=false，但会启动“下次切换”的冷却计时。
                RecipientClearingInstitutionLastSwitchAt::<T>::insert(
                    &who,
                    frame_system::Pallet::<T>::block_number(),
                );
                false
            };

            RecipientClearingInstitution::<T>::insert(&who, institution);
            Self::deposit_event(Event::<T>::RecipientClearingInstitutionBound {
                recipient: who,
                institution,
                switched,
            });
            Ok(())
        }

        /// 将批次持久化进入出队队列（先落库，再由中继账户反复重试打包）。
        #[pallet::call_index(10)]
        #[pallet::weight(T::WeightInfo::enqueue_offchain_batch(T::MaxBatchSize::get()))]
        pub fn enqueue_offchain_batch(
            origin: OriginFor<T>,
            institution: InstitutionPalletId,
            batch_seq: u64,
            batch: BatchOf<T>,
            batch_signature: BatchSignatureOf<T>,
        ) -> DispatchResult {
            let submitter = ensure_signed(origin)?;
            ensure!(!batch.is_empty(), Error::<T>::EmptyBatch);
            let relay_submitters = RelaySubmitters::<T>::get(institution)
                .ok_or(Error::<T>::RelaySubmittersNotInitialized)?;
            ensure!(
                relay_submitters.iter().any(|acc| acc == &submitter),
                Error::<T>::RelaySubmitterNotAllowed
            );
            let executed_next_seq = LastBatchSeq::<T>::get(institution)
                .checked_add(1)
                .ok_or(Error::<T>::CounterOverflow)?;
            let queued_next_seq = NextEnqueueBatchSeq::<T>::get(institution);
            let expected_seq = if queued_next_seq < executed_next_seq {
                executed_next_seq
            } else {
                queued_next_seq
            };
            ensure!(batch_seq == expected_seq, Error::<T>::InvalidBatchSeq);
            let rate_bp = Self::ensure_rate_and_institution(institution)?;
            let verify_key =
                Self::verify_key_for(institution).ok_or(Error::<T>::VerifyKeyMissing)?;
            let message = Self::batch_signing_message(institution, batch_seq, &batch);
            ensure!(
                T::OffchainBatchVerifier::verify(
                    verify_key.as_slice(),
                    message.as_slice(),
                    batch_signature.as_slice()
                ),
                Error::<T>::InvalidBatchSignature
            );

            let mut fee_sum_u128: u128 = 0;
            let t2 = institution_t2_code(institution).ok_or(Error::<T>::InvalidInstitution)?;
            Self::ensure_no_duplicate_tx_ids(&batch)?;
            for item in batch.iter() {
                ensure!(
                    !Self::is_processed_offchain_tx_active(t2, item.tx_id),
                    Error::<T>::TxAlreadyProcessed
                );
                ensure!(
                    !QueuedTxIndex::<T>::contains_key(t2, item.tx_id),
                    Error::<T>::TxAlreadyProcessed
                );
                ensure!(
                    !item.transfer_amount.is_zero(),
                    Error::<T>::InvalidTransferAmount
                );
                ensure!(
                    item.payer != item.recipient,
                    Error::<T>::SelfTransferNotAllowed
                );
                ensure!(
                    !T::ProtectedSourceChecker::is_protected(&item.payer),
                    Error::<T>::ProtectedSource
                );
                let bound = RecipientClearingInstitution::<T>::get(&item.recipient)
                    .ok_or(Error::<T>::RecipientClearingInstitutionNotBound)?;
                ensure!(
                    bound == institution,
                    Error::<T>::RecipientClearingInstitutionMismatch
                );
                let transfer_u128: u128 = item.transfer_amount.saturated_into();
                let fee_u128: u128 = item.offchain_fee_amount.saturated_into();
                ensure!(
                    transfer_u128 <= u128::MAX / rate_bp as u128,
                    Error::<T>::TransferAmountTooLarge
                );
                let expected_fee = calc_offchain_fee_fen(transfer_u128, rate_bp);
                ensure!(fee_u128 == expected_fee, Error::<T>::InvalidFeeAmount);
                fee_sum_u128 = fee_sum_u128.saturating_add(fee_u128);
            }

            let queue_id = NextQueuedBatchId::<T>::get();
            let next_queue_id = queue_id.checked_add(1).ok_or(Error::<T>::CounterOverflow)?;
            let next_enqueue_seq = expected_seq
                .checked_add(1)
                .ok_or(Error::<T>::CounterOverflow)?;
            NextQueuedBatchId::<T>::put(next_queue_id);
            NextEnqueueBatchSeq::<T>::insert(institution, next_enqueue_seq);
            let now = frame_system::Pallet::<T>::block_number();
            let item_count = batch.len() as u32;
            // 中文注释：队列在入队时锁定费率快照；后续费率治理不影响已入队批次。
            let fee_sum_snapshot: BalanceOf<T> = fee_sum_u128.saturated_into();
            let verify_key_epoch_snapshot = VerifyKeyEpoch::<T>::get(institution);
            let marker_tx_id = batch
                .first()
                .map(|i| i.tx_id)
                .ok_or(Error::<T>::EmptyBatch)?;
            for item in batch.iter() {
                QueuedTxIndex::<T>::insert(t2, item.tx_id, queue_id);
            }

            QueuedBatches::<T>::insert(
                queue_id,
                QueuedBatchRecord {
                    institution,
                    batch_seq,
                    batch,
                    batch_signature,
                    rate_bp_snapshot: rate_bp,
                    status: QueuedBatchStatus::Pending,
                    retry_count: 0,
                    last_error: None,
                    enqueued_by: submitter.clone(),
                    enqueued_at: now,
                    last_attempt_at: None,
                    processed_at: None,
                    fee_sum_snapshot,
                    marker_tx_id,
                    verify_key_epoch_snapshot,
                },
            );

            Self::deposit_event(Event::<T>::OffchainBatchQueued {
                queue_id,
                institution,
                batch_seq,
                enqueued_by: submitter,
                item_count,
                fee_sum_snapshot,
            });
            Ok(())
        }

        /// 从持久化队列出队并执行；失败不丢队列，记录重试次数并可继续重试。
        #[pallet::call_index(11)]
        #[pallet::weight(T::WeightInfo::process_queued_batch(T::MaxBatchSize::get()))]
        pub fn process_queued_batch(origin: OriginFor<T>, queue_id: u64) -> DispatchResult {
            let submitter = ensure_signed(origin)?;
            let mut queued =
                QueuedBatches::<T>::get(queue_id).ok_or(Error::<T>::QueuedBatchNotFound)?;
            ensure!(
                matches!(queued.status, QueuedBatchStatus::Pending),
                Error::<T>::QueuedBatchAlreadyProcessed
            );

            let now = frame_system::Pallet::<T>::block_number();
            let current_epoch = VerifyKeyEpoch::<T>::get(queued.institution);
            if queued.verify_key_epoch_snapshot != current_epoch {
                let t2 = institution_t2_code(queued.institution)
                    .ok_or(Error::<T>::InvalidInstitution)?;
                queued.status = QueuedBatchStatus::Cancelled;
                queued.last_attempt_at = Some(now);
                queued.last_error = Some(QueuedBatchLastError::Cancelled);
                for item in queued.batch.iter() {
                    QueuedTxIndex::<T>::remove(t2, item.tx_id);
                }
                QueuedBatches::<T>::insert(queue_id, queued.clone());
                Self::deposit_event(Event::<T>::OffchainQueuedBatchInvalidatedByKeyRotation {
                    queue_id,
                    institution: queued.institution,
                    expected_epoch: current_epoch,
                    queued_epoch: queued.verify_key_epoch_snapshot,
                });
                return Ok(());
            }
            let precheck_result = Self::precheck_submit_offchain_batch_with_rate(
                &submitter,
                queued.institution,
                queued.batch_seq,
                &queued.batch,
                &queued.batch_signature,
                queued.rate_bp_snapshot,
                false,
                false,
            );
            let (t2, verify_key, by_count, by_time) = match precheck_result {
                Ok(v) => v,
                Err(e) => {
                    if Self::should_bubble_precheck_error(&e) {
                        return Err(e);
                    }
                    if Self::should_wait_precheck_error(&e) {
                        queued.last_error = Some(QueuedBatchLastError::WaitingForPriorBatch);
                        queued.last_attempt_at = Some(now);
                        QueuedBatches::<T>::insert(queue_id, queued.clone());
                        Self::deposit_event(Event::<T>::OffchainQueuedBatchRetryFailed {
                            queue_id,
                            institution: queued.institution,
                            retry_count: queued.retry_count,
                            last_error: QueuedBatchLastError::WaitingForPriorBatch,
                        });
                        return Ok(());
                    }
                    if Self::should_ignore_precheck_error(&e) {
                        queued.last_error = Some(QueuedBatchLastError::PackThresholdNotReached);
                        queued.last_attempt_at = Some(now);
                        QueuedBatches::<T>::insert(queue_id, queued.clone());
                        Self::deposit_event(Event::<T>::OffchainQueuedBatchDeferred {
                            queue_id,
                            institution: queued.institution,
                            reason: QueuedBatchLastError::PackThresholdNotReached,
                        });
                        return Ok(());
                    }
                    queued.retry_count = queued.retry_count.saturating_add(1);
                    queued.last_error = Some(QueuedBatchLastError::PrecheckFailed);
                    queued.last_attempt_at = Some(now);
                    if queued.retry_count >= MAX_QUEUE_RETRY_COUNT {
                        queued.status = QueuedBatchStatus::Failed;
                        QueuedBatches::<T>::insert(queue_id, queued.clone());
                        if let Some(t2) = institution_t2_code(queued.institution) {
                            for item in queued.batch.iter() {
                                QueuedTxIndex::<T>::remove(t2, item.tx_id);
                            }
                        }
                        Self::deposit_event(Event::<T>::OffchainQueuedBatchFailed {
                            queue_id,
                            institution: queued.institution,
                            retry_count: queued.retry_count,
                            last_error: QueuedBatchLastError::PrecheckFailed,
                        });
                        return Ok(());
                    }
                    QueuedBatches::<T>::insert(queue_id, queued.clone());
                    Self::deposit_event(Event::<T>::OffchainQueuedBatchRetryFailed {
                        queue_id,
                        institution: queued.institution,
                        retry_count: queued.retry_count,
                        last_error: QueuedBatchLastError::PrecheckFailed,
                    });
                    return Ok(());
                }
            };

            let execute_result = with_transaction(|| {
                let inner = Self::execute_batch(
                    &submitter,
                    queued.institution,
                    queued.batch_seq,
                    &queued.batch,
                    t2,
                    &verify_key,
                    by_count,
                    by_time,
                );
                match inner {
                    Ok(batch_id) => TransactionOutcome::Commit(Ok(batch_id)),
                    Err(e) => TransactionOutcome::Rollback(Err(e)),
                }
            });

            match execute_result {
                Ok(batch_id) => {
                    queued.status = QueuedBatchStatus::Processed;
                    queued.last_attempt_at = Some(now);
                    queued.processed_at = Some(now);
                    queued.last_error = None;
                    QueuedBatches::<T>::insert(queue_id, queued.clone());
                    for item in queued.batch.iter() {
                        QueuedTxIndex::<T>::remove(t2, item.tx_id);
                    }
                    Self::deposit_event(Event::<T>::OffchainQueuedBatchProcessed {
                        queue_id,
                        institution: queued.institution,
                        batch_id,
                        retry_count: queued.retry_count,
                    });
                }
                Err(_e) => {
                    queued.retry_count = queued.retry_count.saturating_add(1);
                    queued.last_error = Some(QueuedBatchLastError::ExecutionFailed);
                    queued.last_attempt_at = Some(now);
                    if queued.retry_count >= MAX_QUEUE_RETRY_COUNT {
                        queued.status = QueuedBatchStatus::Failed;
                        QueuedBatches::<T>::insert(queue_id, queued.clone());
                        for item in queued.batch.iter() {
                            QueuedTxIndex::<T>::remove(t2, item.tx_id);
                        }
                        Self::deposit_event(Event::<T>::OffchainQueuedBatchFailed {
                            queue_id,
                            institution: queued.institution,
                            retry_count: queued.retry_count,
                            last_error: QueuedBatchLastError::ExecutionFailed,
                        });
                        return Ok(());
                    }
                    QueuedBatches::<T>::insert(queue_id, queued.clone());
                    Self::deposit_event(Event::<T>::OffchainQueuedBatchRetryFailed {
                        queue_id,
                        institution: queued.institution,
                        retry_count: queued.retry_count,
                        last_error: QueuedBatchLastError::ExecutionFailed,
                    });
                }
            }
            Ok(())
        }

        /// 省储行管理员发起费率治理提案（内部投票）。
        #[pallet::call_index(1)]
        #[pallet::weight(T::DbWeight::get().reads_writes(4, 2))]
        pub fn propose_institution_rate(
            origin: OriginFor<T>,
            institution: InstitutionPalletId,
            new_rate_bp: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                (OFFCHAIN_RATE_BP_MIN..=OFFCHAIN_RATE_BP_MAX).contains(&new_rate_bp),
                Error::<T>::InvalidRateBp
            );
            ensure!(
                Self::is_prb_admin(institution, &who),
                Error::<T>::UnauthorizedAdmin
            );

            let proposal_id =
                T::InternalVoteEngine::create_internal_proposal(who.clone(), ORG_PRB, institution)?;

            RateProposalActions::<T>::insert(
                proposal_id,
                RateProposalAction {
                    institution,
                    new_rate_bp,
                },
            );

            Self::deposit_event(Event::<T>::InstitutionRateProposed {
                proposal_id,
                institution,
                proposer: who,
                new_rate_bp,
            });
            Ok(())
        }

        /// 省储行管理员对费率提案投票；通过后自动生效。
        #[pallet::call_index(2)]
        #[pallet::weight(T::DbWeight::get().reads_writes(6, 5))]
        pub fn vote_institution_rate(
            origin: OriginFor<T>,
            proposal_id: u64,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let action = RateProposalActions::<T>::get(proposal_id)
                .ok_or(Error::<T>::RateProposalNotFound)?;
            ensure!(
                Self::is_prb_admin(action.institution, &who),
                Error::<T>::UnauthorizedAdmin
            );

            voting_engine_system::Pallet::<T>::internal_vote(
                frame_system::RawOrigin::Signed(who.clone()).into(),
                proposal_id,
                approve,
            )?;

            Self::deposit_event(Event::<T>::InstitutionRateVoteSubmitted {
                proposal_id,
                voter: who,
                approve,
            });

            if approve {
                if let Some(proposal) = voting_engine_system::Pallet::<T>::proposals(proposal_id) {
                    if proposal.status == STATUS_PASSED {
                        if let Err(_e) =
                            with_transaction(|| match Self::try_execute_rate(proposal_id) {
                                Ok(()) => TransactionOutcome::Commit(Ok(())),
                                Err(e) => TransactionOutcome::Rollback(Err(e)),
                            })
                        {
                            Self::deposit_event(Event::<T>::InternalProposalExecutionFailed {
                                proposal_id,
                            });
                        }
                    }
                }
            }
            Ok(())
        }

        /// 省储行管理员发起“链下验证密钥”更新提案（内部投票）。
        #[pallet::call_index(3)]
        #[pallet::weight(T::DbWeight::get().reads_writes(4, 2))]
        pub fn propose_verify_key(
            origin: OriginFor<T>,
            institution: InstitutionPalletId,
            new_key: VerifyKeyOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(!new_key.is_empty(), Error::<T>::InvalidVerifyKey);
            ensure!(
                Self::is_prb_admin(institution, &who),
                Error::<T>::UnauthorizedAdmin
            );

            let proposal_id =
                T::InternalVoteEngine::create_internal_proposal(who.clone(), ORG_PRB, institution)?;

            VerifyKeyProposalActions::<T>::insert(
                proposal_id,
                VerifyKeyProposalAction {
                    institution,
                    new_key: new_key.clone(),
                },
            );

            Self::deposit_event(Event::<T>::VerifyKeyProposed {
                proposal_id,
                institution,
                proposer: who,
                key_len: new_key.len() as u32,
            });
            Ok(())
        }

        /// 省储行管理员对验证密钥提案投票；通过后自动生效。
        #[pallet::call_index(4)]
        #[pallet::weight(T::DbWeight::get().reads_writes(6, 5))]
        pub fn vote_verify_key(
            origin: OriginFor<T>,
            proposal_id: u64,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let action = VerifyKeyProposalActions::<T>::get(proposal_id)
                .ok_or(Error::<T>::VerifyKeyProposalNotFound)?;
            ensure!(
                Self::is_prb_admin(action.institution, &who),
                Error::<T>::UnauthorizedAdmin
            );

            voting_engine_system::Pallet::<T>::internal_vote(
                frame_system::RawOrigin::Signed(who.clone()).into(),
                proposal_id,
                approve,
            )?;

            Self::deposit_event(Event::<T>::VerifyKeyVoteSubmitted {
                proposal_id,
                voter: who,
                approve,
            });

            if approve {
                if let Some(proposal) = voting_engine_system::Pallet::<T>::proposals(proposal_id) {
                    if proposal.status == STATUS_PASSED {
                        if let Err(_e) =
                            with_transaction(|| match Self::try_execute_verify_key(proposal_id) {
                                Ok(()) => TransactionOutcome::Commit(Ok(())),
                                Err(e) => TransactionOutcome::Rollback(Err(e)),
                            })
                        {
                            Self::deposit_event(Event::<T>::InternalProposalExecutionFailed {
                                proposal_id,
                            });
                        }
                    }
                }
            }
            Ok(())
        }

        /// 省储行管理员发起 fee_address 向主多签地址划转提案（内部投票）。
        /// 约束：划转后 fee_address 至少保留 1111.11 元。
        #[pallet::call_index(6)]
        #[pallet::weight(T::DbWeight::get().reads_writes(4, 2))]
        pub fn propose_sweep_to_main(
            origin: OriginFor<T>,
            institution: InstitutionPalletId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let amount_u128: u128 = amount.saturated_into();
            ensure!(amount_u128 > 0, Error::<T>::InvalidSweepAmount);
            ensure!(
                Self::is_prb_admin(institution, &who),
                Error::<T>::UnauthorizedAdmin
            );

            let proposal_id =
                T::InternalVoteEngine::create_internal_proposal(who.clone(), ORG_PRB, institution)?;

            SweepProposalActions::<T>::insert(
                proposal_id,
                SweepProposalAction {
                    institution,
                    amount,
                },
            );

            Self::deposit_event(Event::<T>::SweepToMainProposed {
                proposal_id,
                institution,
                proposer: who,
                amount,
            });
            Ok(())
        }

        /// 省储行管理员对划转提案投票；通过后自动执行划转。
        #[pallet::call_index(7)]
        #[pallet::weight(T::DbWeight::get().reads_writes(7, 6))]
        pub fn vote_sweep_to_main(
            origin: OriginFor<T>,
            proposal_id: u64,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let action = SweepProposalActions::<T>::get(proposal_id)
                .ok_or(Error::<T>::SweepProposalNotFound)?;
            ensure!(
                Self::is_prb_admin(action.institution, &who),
                Error::<T>::UnauthorizedAdmin
            );

            voting_engine_system::Pallet::<T>::internal_vote(
                frame_system::RawOrigin::Signed(who.clone()).into(),
                proposal_id,
                approve,
            )?;

            Self::deposit_event(Event::<T>::SweepToMainVoteSubmitted {
                proposal_id,
                voter: who,
                approve,
            });

            if approve {
                if let Some(proposal) = voting_engine_system::Pallet::<T>::proposals(proposal_id) {
                    if proposal.status == STATUS_PASSED {
                        if let Err(_e) =
                            with_transaction(|| match Self::try_execute_sweep(proposal_id) {
                                Ok(()) => TransactionOutcome::Commit(Ok(())),
                                Err(e) => TransactionOutcome::Rollback(Err(e)),
                            })
                        {
                            Self::deposit_event(Event::<T>::InternalProposalExecutionFailed {
                                proposal_id,
                            });
                        }
                    }
                }
            }
            Ok(())
        }

        /// 省储行管理员发起 relay 提交者白名单更新提案（内部投票）。
        #[pallet::call_index(12)]
        #[pallet::weight(T::DbWeight::get().reads_writes(4, 2))]
        pub fn propose_relay_submitters(
            origin: OriginFor<T>,
            institution: InstitutionPalletId,
            submitters: RelaySubmittersOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                Self::is_prb_admin(institution, &who),
                Error::<T>::UnauthorizedAdmin
            );
            Self::ensure_valid_relay_submitters(&submitters)?;
            let proposal_id =
                T::InternalVoteEngine::create_internal_proposal(who.clone(), ORG_PRB, institution)?;
            RelaySubmittersProposalActions::<T>::insert(
                proposal_id,
                RelaySubmittersProposalAction {
                    institution,
                    submitters: submitters.clone(),
                },
            );
            Self::deposit_event(Event::<T>::RelaySubmittersProposed {
                proposal_id,
                institution,
                proposer: who,
                count: submitters.len() as u32,
            });
            Ok(())
        }

        /// 省储行管理员对 relay 提交者白名单提案投票；通过后自动生效。
        #[pallet::call_index(13)]
        #[pallet::weight(T::DbWeight::get().reads_writes(6, 5))]
        pub fn vote_relay_submitters(
            origin: OriginFor<T>,
            proposal_id: u64,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let action = RelaySubmittersProposalActions::<T>::get(proposal_id)
                .ok_or(Error::<T>::RelaySubmittersProposalNotFound)?;
            ensure!(
                Self::is_prb_admin(action.institution, &who),
                Error::<T>::UnauthorizedAdmin
            );
            voting_engine_system::Pallet::<T>::internal_vote(
                frame_system::RawOrigin::Signed(who.clone()).into(),
                proposal_id,
                approve,
            )?;
            Self::deposit_event(Event::<T>::RelaySubmittersVoteSubmitted {
                proposal_id,
                voter: who,
                approve,
            });

            if approve {
                if let Some(proposal) = voting_engine_system::Pallet::<T>::proposals(proposal_id) {
                    if proposal.status == STATUS_PASSED {
                        if let Err(_e) = with_transaction(|| {
                            match Self::try_execute_relay_submitters(proposal_id) {
                                Ok(()) => TransactionOutcome::Commit(Ok(())),
                                Err(e) => TransactionOutcome::Rollback(Err(e)),
                            }
                        }) {
                            Self::deposit_event(Event::<T>::InternalProposalExecutionFailed {
                                proposal_id,
                            });
                        }
                    }
                }
            }
            Ok(())
        }

        /// 清理已处理且超过保留窗口的队列批次记录。
        #[pallet::call_index(14)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
        pub fn prune_queued_batch(origin: OriginFor<T>, queue_id: u64) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            let queued =
                QueuedBatches::<T>::get(queue_id).ok_or(Error::<T>::QueuedBatchNotFound)?;
            let finalized_at = match queued.status {
                QueuedBatchStatus::Processed => queued.processed_at,
                QueuedBatchStatus::Failed => queued.last_attempt_at,
                QueuedBatchStatus::Cancelled => queued.last_attempt_at,
                QueuedBatchStatus::Pending => None,
            }
            .ok_or(Error::<T>::QueuedBatchNotProcessed)?;
            let now = frame_system::Pallet::<T>::block_number();
            let elapsed: u64 = now.saturating_sub(finalized_at).saturated_into();
            ensure!(
                elapsed >= QUEUED_BATCH_RETENTION_BLOCKS,
                Error::<T>::QueueRetentionNotReached
            );
            if let Some(t2) = institution_t2_code(queued.institution) {
                for item in queued.batch.iter() {
                    QueuedTxIndex::<T>::remove(t2, item.tx_id);
                }
            }
            QueuedBatches::<T>::remove(queue_id);
            Self::deposit_event(Event::<T>::QueuedBatchPruned { queue_id });
            Ok(())
        }

        /// 清理超过保留窗口的批次摘要。
        #[pallet::call_index(15)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
        pub fn prune_batch_summary(origin: OriginFor<T>, batch_id: u64) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            let summary =
                BatchSummaries::<T>::get(batch_id).ok_or(Error::<T>::BatchSummaryNotFound)?;
            let now = frame_system::Pallet::<T>::block_number();
            let elapsed: u64 = now.saturating_sub(summary.submitted_at).saturated_into();
            ensure!(
                elapsed >= BATCH_SUMMARY_RETENTION_BLOCKS,
                Error::<T>::BatchSummaryRetentionNotReached
            );
            BatchSummaries::<T>::remove(batch_id);
            Self::deposit_event(Event::<T>::BatchSummaryPruned { batch_id });
            Ok(())
        }

        /// 清理过期的 processed tx 记录。
        #[pallet::call_index(16)]
        #[pallet::weight(T::DbWeight::get().reads_writes(2, 2))]
        pub fn prune_processed_tx(
            origin: OriginFor<T>,
            t2: [u8; 2],
            tx_id: T::Hash,
        ) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            ensure!(
                ProcessedOffchainTx::<T>::get(t2, tx_id),
                Error::<T>::ProcessedTxNotFound
            );
            let Some(recorded_at) = ProcessedOffchainTxAt::<T>::get(t2, tx_id) else {
                ProcessedOffchainTx::<T>::remove(t2, tx_id);
                Self::deposit_event(Event::<T>::ProcessedTxPruned { t2, tx_id });
                return Ok(());
            };
            let now = frame_system::Pallet::<T>::block_number();
            let elapsed: u64 = now.saturating_sub(recorded_at).saturated_into();
            ensure!(
                elapsed >= PROCESSED_TX_RETENTION_BLOCKS,
                Error::<T>::ProcessedTxRetentionNotReached
            );
            ProcessedOffchainTx::<T>::remove(t2, tx_id);
            ProcessedOffchainTxAt::<T>::remove(t2, tx_id);
            Self::deposit_event(Event::<T>::ProcessedTxPruned { t2, tx_id });
            Ok(())
        }

        /// 清理已否决或过期的提案动作存储。
        #[pallet::call_index(17)]
        #[pallet::weight(T::DbWeight::get().reads_writes(5, 4))]
        pub fn prune_expired_proposal_action(
            origin: OriginFor<T>,
            proposal_id: u64,
        ) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            let proposal = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;
            let now = frame_system::Pallet::<T>::block_number();
            let expired = now > proposal.end;
            let prunable = proposal.status == STATUS_REJECTED || expired;
            ensure!(prunable, Error::<T>::ProposalNotPrunable);

            let mut pruned = false;
            if RateProposalActions::<T>::contains_key(proposal_id) {
                RateProposalActions::<T>::remove(proposal_id);
                pruned = true;
            }
            if VerifyKeyProposalActions::<T>::contains_key(proposal_id) {
                VerifyKeyProposalActions::<T>::remove(proposal_id);
                pruned = true;
            }
            if SweepProposalActions::<T>::contains_key(proposal_id) {
                SweepProposalActions::<T>::remove(proposal_id);
                pruned = true;
            }
            if RelaySubmittersProposalActions::<T>::contains_key(proposal_id) {
                RelaySubmittersProposalActions::<T>::remove(proposal_id);
                pruned = true;
            }
            ensure!(pruned, Error::<T>::ProposalActionNotFound);
            Self::deposit_event(Event::<T>::ProposalActionPruned { proposal_id });
            Ok(())
        }

        /// 由省储行管理员跳过失败批次，推进执行序列，解除后续批次阻塞。
        #[pallet::call_index(18)]
        #[pallet::weight(T::DbWeight::get().reads_writes(2, 2))]
        pub fn skip_failed_batch(origin: OriginFor<T>, queue_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let queued =
                QueuedBatches::<T>::get(queue_id).ok_or(Error::<T>::QueuedBatchNotFound)?;
            ensure!(
                matches!(
                    queued.status,
                    QueuedBatchStatus::Failed | QueuedBatchStatus::Cancelled
                ),
                Error::<T>::QueuedBatchNotSkippable
            );
            ensure!(
                Self::is_prb_admin(queued.institution, &who),
                Error::<T>::UnauthorizedAdmin
            );
            let current = LastBatchSeq::<T>::get(queued.institution);
            let expected_seq = current.checked_add(1).ok_or(Error::<T>::CounterOverflow)?;
            ensure!(
                queued.batch_seq == expected_seq,
                Error::<T>::InvalidBatchSeq
            );
            LastBatchSeq::<T>::insert(queued.institution, queued.batch_seq);
            Self::deposit_event(Event::<T>::FailedBatchSkipped {
                queue_id,
                institution: queued.institution,
                batch_seq: queued.batch_seq,
                operator: who,
            });
            Ok(())
        }

        /// 取消待处理队列批次（仅管理员，且仅 Pending）。
        #[pallet::call_index(19)]
        #[pallet::weight(T::DbWeight::get().reads_writes(2, 2))]
        pub fn cancel_queued_batch(origin: OriginFor<T>, queue_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let mut queued =
                QueuedBatches::<T>::get(queue_id).ok_or(Error::<T>::QueuedBatchNotFound)?;
            ensure!(
                matches!(queued.status, QueuedBatchStatus::Pending),
                Error::<T>::QueuedBatchNotPending
            );
            ensure!(
                Self::is_prb_admin(queued.institution, &who),
                Error::<T>::UnauthorizedAdmin
            );
            let t2 =
                institution_t2_code(queued.institution).ok_or(Error::<T>::InvalidInstitution)?;
            let current = LastBatchSeq::<T>::get(queued.institution);
            let expected_seq = current.checked_add(1).ok_or(Error::<T>::CounterOverflow)?;
            ensure!(
                queued.batch_seq == expected_seq,
                Error::<T>::InvalidBatchSeq
            );
            queued.status = QueuedBatchStatus::Cancelled;
            queued.last_attempt_at = Some(frame_system::Pallet::<T>::block_number());
            queued.last_error = Some(QueuedBatchLastError::Cancelled);
            LastBatchSeq::<T>::insert(queued.institution, queued.batch_seq);
            QueuedBatches::<T>::insert(queue_id, queued.clone());
            for item in queued.batch.iter() {
                QueuedTxIndex::<T>::remove(t2, item.tx_id);
            }
            Self::deposit_event(Event::<T>::OffchainQueuedBatchCancelled {
                queue_id,
                institution: queued.institution,
                operator: who,
            });
            Ok(())
        }

        /// 对已通过但执行失败的提案动作进行重试执行。
        #[pallet::call_index(20)]
        #[pallet::weight(T::DbWeight::get().reads_writes(8, 6))]
        pub fn retry_execute_proposal(origin: OriginFor<T>, proposal_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let proposal = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;
            ensure!(
                proposal.status == STATUS_PASSED,
                Error::<T>::ProposalExecutionRetryNotAllowed
            );

            let mut found = false;
            let mut institution = None;
            if let Some(action) = RateProposalActions::<T>::get(proposal_id) {
                found = true;
                institution = Some(action.institution);
            } else if let Some(action) = VerifyKeyProposalActions::<T>::get(proposal_id) {
                found = true;
                institution = Some(action.institution);
            } else if let Some(action) = SweepProposalActions::<T>::get(proposal_id) {
                found = true;
                institution = Some(action.institution);
            } else if let Some(action) = RelaySubmittersProposalActions::<T>::get(proposal_id) {
                found = true;
                institution = Some(action.institution);
            }
            ensure!(found, Error::<T>::ProposalActionNotFound);
            let institution = institution.expect("found ensures some; qed");
            ensure!(
                Self::is_prb_admin(institution, &who),
                Error::<T>::UnauthorizedAdmin
            );

            with_transaction(|| {
                let result = if RateProposalActions::<T>::contains_key(proposal_id) {
                    Self::try_execute_rate(proposal_id)
                } else if VerifyKeyProposalActions::<T>::contains_key(proposal_id) {
                    Self::try_execute_verify_key(proposal_id)
                } else if SweepProposalActions::<T>::contains_key(proposal_id) {
                    Self::try_execute_sweep(proposal_id)
                } else if RelaySubmittersProposalActions::<T>::contains_key(proposal_id) {
                    Self::try_execute_relay_submitters(proposal_id)
                } else {
                    Err(Error::<T>::ProposalActionNotFound.into())
                };
                match result {
                    Ok(()) => TransactionOutcome::Commit(Ok(())),
                    Err(e) => TransactionOutcome::Rollback(Err(e)),
                }
            })?;
            Self::deposit_event(Event::<T>::ProposalExecutionRetried { proposal_id });
            Ok(())
        }

        /// 紧急立即轮换验证密钥（跳过延迟窗口），用于旧密钥泄露场景。
        #[pallet::call_index(21)]
        #[pallet::weight(T::DbWeight::get().reads_writes(7, 5))]
        pub fn emergency_rotate_verify_key(
            origin: OriginFor<T>,
            institution: InstitutionPalletId,
            new_key: VerifyKeyOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                Self::is_prb_admin(institution, &who),
                Error::<T>::UnauthorizedAdmin
            );
            ensure!(
                institution_pallet_address(institution).is_some(),
                Error::<T>::InvalidInstitution
            );
            ensure!(!new_key.is_empty(), Error::<T>::InvalidVerifyKey);
            let key_hash = sp_io::hashing::blake2_256(new_key.as_slice());

            EmergencyVerifyKeyApprovals::<T>::try_mutate(
                institution,
                key_hash,
                |approvals| -> DispatchResult {
                    ensure!(
                        !approvals.iter().any(|acc| acc == &who),
                        Error::<T>::EmergencyRotationAlreadyApproved
                    );
                    approvals
                        .try_push(who.clone())
                        .map_err(|_| Error::<T>::EmergencyApprovalsOverflow)?;
                    Ok(())
                },
            )?;
            let approval_count =
                EmergencyVerifyKeyApprovals::<T>::get(institution, key_hash).len() as u32;
            if approval_count < EMERGENCY_ROTATE_MIN_ADMINS {
                Self::deposit_event(Event::<T>::VerifyKeyEmergencyRotationApproval {
                    institution,
                    key_hash,
                    approver: who,
                    approvals: approval_count,
                    required: EMERGENCY_ROTATE_MIN_ADMINS,
                });
                return Ok(());
            }
            EmergencyVerifyKeyApprovals::<T>::remove(institution, key_hash);

            let now = frame_system::Pallet::<T>::block_number();
            VerifyKeys::<T>::insert(institution, &new_key);
            PendingVerifyKeys::<T>::remove(institution);
            let epoch = VerifyKeyEpoch::<T>::get(institution);
            let next_epoch = epoch.checked_add(1).ok_or(Error::<T>::CounterOverflow)?;
            VerifyKeyEpoch::<T>::insert(institution, next_epoch);
            VerifyKeyRotationStatuses::<T>::insert(
                institution,
                VerifyKeyRotationStatus {
                    stage: VerifyKeyRotationStage::Idle,
                    activate_at: None,
                },
            );
            Self::deposit_event(Event::<T>::VerifyKeyEmergencyRotated {
                institution,
                key_len: new_key.len() as u32,
                activated_at: now,
                operator: who,
            });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// 批次提交前置校验（无写入），供 runtime 扣费前判断是否允许 fee_pallet_address 承担手续费。
        pub fn precheck_submit_offchain_batch(
            submitter: &T::AccountId,
            institution: InstitutionPalletId,
            batch_seq: u64,
            batch: &BatchOf<T>,
            batch_signature: &BatchSignatureOf<T>,
        ) -> Result<(), DispatchError> {
            let rate_bp = Self::ensure_rate_and_institution(institution)?;
            Self::precheck_submit_offchain_batch_with_rate(
                submitter,
                institution,
                batch_seq,
                batch,
                batch_signature,
                rate_bp,
                true,
                true,
            )?;
            Ok(())
        }

        /// 队列处理前置校验（使用入队时费率快照）。
        pub fn precheck_process_queued_batch(
            submitter: &T::AccountId,
            queue_id: u64,
        ) -> Result<BalanceOf<T>, DispatchError> {
            let queued =
                QueuedBatches::<T>::get(queue_id).ok_or(Error::<T>::QueuedBatchNotFound)?;
            ensure!(
                matches!(queued.status, QueuedBatchStatus::Pending),
                Error::<T>::QueuedBatchAlreadyProcessed
            );
            Self::precheck_submit_offchain_batch_with_rate(
                submitter,
                queued.institution,
                queued.batch_seq,
                &queued.batch,
                &queued.batch_signature,
                queued.rate_bp_snapshot,
                false,
                false,
            )?;
            Ok(queued.fee_sum_snapshot)
        }

        /// 队列批次对应的手续费支付账户（用于 runtime 扣费路由）。
        pub fn fee_payer_for_queued_batch(queue_id: u64) -> Result<T::AccountId, DispatchError> {
            let queued =
                QueuedBatches::<T>::get(queue_id).ok_or(Error::<T>::QueuedBatchNotFound)?;
            Self::institution_fee_account(queued.institution)
        }

        fn precheck_submit_offchain_batch_with_rate(
            submitter: &T::AccountId,
            institution: InstitutionPalletId,
            batch_seq: u64,
            batch: &BatchOf<T>,
            batch_signature: &BatchSignatureOf<T>,
            rate_bp: u32,
            verify_signature: bool,
            verify_fee: bool,
        ) -> Result<([u8; 2], VerifyKeyOf<T>, bool, bool), DispatchError> {
            ensure!(!batch.is_empty(), Error::<T>::EmptyBatch);
            let relay_submitters = RelaySubmitters::<T>::get(institution)
                .ok_or(Error::<T>::RelaySubmittersNotInitialized)?;
            ensure!(
                relay_submitters.iter().any(|acc| acc == submitter),
                Error::<T>::RelaySubmitterNotAllowed
            );
            let expected_seq = LastBatchSeq::<T>::get(institution)
                .checked_add(1)
                .ok_or(Error::<T>::CounterOverflow)?;
            ensure!(batch_seq == expected_seq, Error::<T>::InvalidBatchSeq);
            let verify_key =
                Self::verify_key_for(institution).ok_or(Error::<T>::VerifyKeyMissing)?;
            if verify_signature {
                let message = Self::batch_signing_message(institution, batch_seq, batch);
                ensure!(
                    T::OffchainBatchVerifier::verify(
                        verify_key.as_slice(),
                        message.as_slice(),
                        batch_signature.as_slice()
                    ),
                    Error::<T>::InvalidBatchSignature
                );
            }

            let now = frame_system::Pallet::<T>::block_number();
            let last = LastPackBlock::<T>::get(institution);
            let (by_count, by_time) = Self::pack_trigger_reason(last, now, batch.len() as u64);
            ensure!(by_count || by_time, Error::<T>::PackThresholdNotReached);
            let t2 = institution_t2_code(institution).ok_or(Error::<T>::InvalidInstitution)?;
            Self::ensure_no_duplicate_tx_ids(batch)?;

            for item in batch.iter() {
                ensure!(
                    !Self::is_processed_offchain_tx_active(t2, item.tx_id),
                    Error::<T>::TxAlreadyProcessed
                );
                ensure!(
                    !item.transfer_amount.is_zero(),
                    Error::<T>::InvalidTransferAmount
                );
                ensure!(
                    item.payer != item.recipient,
                    Error::<T>::SelfTransferNotAllowed
                );
                ensure!(
                    !T::ProtectedSourceChecker::is_protected(&item.payer),
                    Error::<T>::ProtectedSource
                );
                let bound = RecipientClearingInstitution::<T>::get(&item.recipient)
                    .ok_or(Error::<T>::RecipientClearingInstitutionNotBound)?;
                ensure!(
                    bound == institution,
                    Error::<T>::RecipientClearingInstitutionMismatch
                );
                if verify_fee {
                    let transfer_u128: u128 = item.transfer_amount.saturated_into();
                    let fee_u128: u128 = item.offchain_fee_amount.saturated_into();
                    ensure!(
                        transfer_u128 <= u128::MAX / rate_bp as u128,
                        Error::<T>::TransferAmountTooLarge
                    );
                    let expected_fee = calc_offchain_fee_fen(transfer_u128, rate_bp);
                    ensure!(fee_u128 == expected_fee, Error::<T>::InvalidFeeAmount);
                }
            }
            Ok((t2, verify_key, by_count, by_time))
        }

        fn execute_batch(
            submitter: &T::AccountId,
            institution: InstitutionPalletId,
            batch_seq: u64,
            batch: &BatchOf<T>,
            t2: [u8; 2],
            verify_key: &VerifyKeyOf<T>,
            by_count: bool,
            by_time: bool,
        ) -> Result<u64, DispatchError> {
            let fee_account = Self::institution_fee_account(institution)?;
            let now = frame_system::Pallet::<T>::block_number();

            let mut total_transfer_u128: u128 = 0;
            let mut total_fee_u128: u128 = 0;
            for item in batch.iter() {
                T::Currency::transfer(
                    &item.payer,
                    &item.recipient,
                    item.transfer_amount,
                    frame_support::traits::ExistenceRequirement::KeepAlive,
                )?;
                T::Currency::transfer(
                    &item.payer,
                    &fee_account,
                    item.offchain_fee_amount,
                    frame_support::traits::ExistenceRequirement::KeepAlive,
                )?;
                total_transfer_u128 =
                    total_transfer_u128.saturating_add(item.transfer_amount.saturated_into());
                total_fee_u128 =
                    total_fee_u128.saturating_add(item.offchain_fee_amount.saturated_into());
                ProcessedOffchainTx::<T>::insert(t2, item.tx_id, true);
                ProcessedOffchainTxAt::<T>::insert(t2, item.tx_id, now);
                let log_id = NextProcessedTxLogId::<T>::get();
                let next_log_id = log_id.checked_add(1).ok_or(Error::<T>::CounterOverflow)?;
                NextProcessedTxLogId::<T>::put(next_log_id);
                ProcessedTxLog::<T>::insert(log_id, (t2, item.tx_id, now));
            }

            let batch_id = NextBatchId::<T>::get();
            let next_batch_id = batch_id.checked_add(1).ok_or(Error::<T>::CounterOverflow)?;
            NextBatchId::<T>::put(next_batch_id);
            LastPackBlock::<T>::insert(institution, now);
            LastBatchSeq::<T>::insert(institution, batch_seq);

            let total_transfer_amount: BalanceOf<T> = total_transfer_u128.saturated_into();
            let total_offchain_fee_amount: BalanceOf<T> = total_fee_u128.saturated_into();
            let batch_hash = sp_io::hashing::blake2_256(&(institution, batch_seq, batch).encode());
            let signer_key_hash = sp_io::hashing::blake2_256(verify_key.as_slice());

            BatchSummaries::<T>::insert(
                batch_id,
                BatchSummary {
                    submitter: submitter.clone(),
                    institution,
                    batch_seq,
                    batch_hash,
                    signer_key_hash,
                    item_count: batch.len() as u32,
                    total_transfer_amount,
                    total_offchain_fee_amount,
                    submitted_at: now,
                },
            );

            Self::deposit_event(Event::<T>::OffchainBatchSubmitted {
                batch_id,
                institution,
                submitter: submitter.clone(),
                batch_seq,
                batch_hash,
                signer_key_hash,
                item_count: batch.len() as u32,
                total_transfer_amount,
                total_offchain_fee_amount,
                reason_by_count: by_count,
                reason_by_time: by_time,
            });
            Ok(batch_id)
        }

        pub fn fee_account_of(
            institution: InstitutionPalletId,
        ) -> Result<T::AccountId, DispatchError> {
            ensure!(
                institution_pallet_address(institution).is_some(),
                Error::<T>::InvalidInstitution
            );
            // 中文注释：fee_account_of 仅暴露地址查询，不做任何资产转移。
            Self::institution_fee_account(institution)
        }

        /// 返回当前有效验证密钥（仅读取已激活的 current key）。
        /// pending key 的激活统一由 on_initialize 负责，避免双路径判断漂移。
        pub fn verify_key_for(institution: InstitutionPalletId) -> Option<VerifyKeyOf<T>> {
            VerifyKeys::<T>::get(institution)
        }

        fn batch_signing_message(
            institution: InstitutionPalletId,
            batch_seq: u64,
            batch: &BatchOf<T>,
        ) -> [u8; 32] {
            sp_io::hashing::blake2_256(
                &(b"GMB_OFFCHAIN_BATCH_V1", institution, batch_seq, batch).encode(),
            )
        }

        fn institution_fee_account(
            institution: InstitutionPalletId,
        ) -> Result<T::AccountId, DispatchError> {
            let fee_pid =
                institution_shenfen_fee_id(institution).ok_or(Error::<T>::InvalidInstitution)?;
            // 中文注释：fee_pallet_address 直接由 shenfen_fee_id 派生，是独立手续费账户。
            Ok(PalletId(fee_pid).into_account_truncating())
        }

        fn institution_account(
            institution: InstitutionPalletId,
        ) -> Result<T::AccountId, DispatchError> {
            let raw =
                institution_pallet_address(institution).ok_or(Error::<T>::InvalidInstitution)?;
            T::AccountId::decode(&mut &raw[..])
                .map_err(|_| Error::<T>::InstitutionAccountDecodeFailed.into())
        }

        fn pack_trigger_reason(
            last: BlockNumberFor<T>,
            now: BlockNumberFor<T>,
            batch_len: u64,
        ) -> (bool, bool) {
            let elapsed: u32 =
                <BlockNumberFor<T> as sp_runtime::traits::Saturating>::saturating_sub(now, last)
                    .saturated_into();
            let by_count = batch_len >= PACK_TX_THRESHOLD;
            let by_time = last.is_zero() || elapsed >= PACK_BLOCK_THRESHOLD;
            (by_count, by_time)
        }

        fn should_bubble_precheck_error(e: &DispatchError) -> bool {
            *e == Error::<T>::RelaySubmitterNotAllowed.into()
                || *e == Error::<T>::RelaySubmittersNotInitialized.into()
                || *e == Error::<T>::VerifyKeyMissing.into()
        }

        fn should_ignore_precheck_error(e: &DispatchError) -> bool {
            *e == Error::<T>::PackThresholdNotReached.into()
        }

        fn should_wait_precheck_error(e: &DispatchError) -> bool {
            *e == Error::<T>::InvalidBatchSeq.into()
        }

        fn ensure_valid_relay_submitters(submitters: &RelaySubmittersOf<T>) -> DispatchResult {
            let count = submitters.len() as u32;
            ensure!(
                count > 0 && count <= T::MaxRelaySubmitters::get(),
                Error::<T>::InvalidRelaySubmittersCount
            );
            for (idx, acc) in submitters.iter().enumerate() {
                ensure!(
                    submitters.iter().skip(idx + 1).all(|next| next != acc),
                    Error::<T>::DuplicateRelaySubmitter
                );
            }
            Ok(())
        }

        fn ensure_no_duplicate_tx_ids(batch: &BatchOf<T>) -> DispatchResult {
            for (idx, a) in batch.iter().enumerate() {
                for b in batch.iter().skip(idx + 1) {
                    ensure!(a.tx_id != b.tx_id, Error::<T>::DuplicateTxIdInBatch);
                }
            }
            Ok(())
        }

        fn is_processed_offchain_tx_active(t2: [u8; 2], tx_id: T::Hash) -> bool {
            if !ProcessedOffchainTx::<T>::get(t2, tx_id) {
                return false;
            }
            let Some(recorded_at) = ProcessedOffchainTxAt::<T>::get(t2, tx_id) else {
                return true;
            };
            let now = frame_system::Pallet::<T>::block_number();
            let elapsed: u64 = now.saturating_sub(recorded_at).saturated_into();
            elapsed < PROCESSED_TX_RETENTION_BLOCKS
        }

        fn auto_prune_one_processed_tx(now: BlockNumberFor<T>) -> bool {
            let cursor = ProcessedTxPruneCursor::<T>::get();
            let next = NextProcessedTxLogId::<T>::get();
            if cursor >= next {
                return false;
            }
            let Some((t2, tx_id, recorded_at)) = ProcessedTxLog::<T>::get(cursor) else {
                let Some(next_cursor) = cursor.checked_add(1) else {
                    return false;
                };
                ProcessedTxPruneCursor::<T>::put(next_cursor);
                return true;
            };
            let elapsed: u64 = now.saturating_sub(recorded_at).saturated_into();
            if elapsed < PROCESSED_TX_RETENTION_BLOCKS {
                return false;
            }
            ProcessedOffchainTx::<T>::remove(t2, tx_id);
            ProcessedOffchainTxAt::<T>::remove(t2, tx_id);
            ProcessedTxLog::<T>::remove(cursor);
            let Some(next_cursor) = cursor.checked_add(1) else {
                return false;
            };
            ProcessedTxPruneCursor::<T>::put(next_cursor);
            Self::deposit_event(Event::<T>::ProcessedTxPruned { t2, tx_id });
            true
        }

        fn auto_prune_one_queued_batch(now: BlockNumberFor<T>) -> bool {
            let cursor = QueuedBatchPruneCursor::<T>::get();
            let next = NextQueuedBatchId::<T>::get();
            if cursor >= next {
                return false;
            }
            let Some(queued) = QueuedBatches::<T>::get(cursor) else {
                let Some(next_cursor) = cursor.checked_add(1) else {
                    return false;
                };
                QueuedBatchPruneCursor::<T>::put(next_cursor);
                return true;
            };
            let finalized_at = match queued.status {
                QueuedBatchStatus::Processed => queued.processed_at,
                QueuedBatchStatus::Failed | QueuedBatchStatus::Cancelled => queued.last_attempt_at,
                QueuedBatchStatus::Pending => None,
            };
            let Some(finalized_at) = finalized_at else {
                return false;
            };
            let elapsed: u64 = now.saturating_sub(finalized_at).saturated_into();
            if elapsed < QUEUED_BATCH_RETENTION_BLOCKS {
                return false;
            }
            if let Some(t2) = institution_t2_code(queued.institution) {
                for item in queued.batch.iter() {
                    QueuedTxIndex::<T>::remove(t2, item.tx_id);
                }
            }
            QueuedBatches::<T>::remove(cursor);
            let Some(next_cursor) = cursor.checked_add(1) else {
                return false;
            };
            QueuedBatchPruneCursor::<T>::put(next_cursor);
            Self::deposit_event(Event::<T>::QueuedBatchPruned { queue_id: cursor });
            true
        }

        fn auto_prune_one_batch_summary(now: BlockNumberFor<T>) -> bool {
            let cursor = BatchSummaryPruneCursor::<T>::get();
            let next = NextBatchId::<T>::get();
            if cursor >= next {
                return false;
            }
            let Some(summary) = BatchSummaries::<T>::get(cursor) else {
                let Some(next_cursor) = cursor.checked_add(1) else {
                    return false;
                };
                BatchSummaryPruneCursor::<T>::put(next_cursor);
                return true;
            };
            let elapsed: u64 = now.saturating_sub(summary.submitted_at).saturated_into();
            if elapsed < BATCH_SUMMARY_RETENTION_BLOCKS {
                return false;
            }
            BatchSummaries::<T>::remove(cursor);
            let Some(next_cursor) = cursor.checked_add(1) else {
                return false;
            };
            BatchSummaryPruneCursor::<T>::put(next_cursor);
            Self::deposit_event(Event::<T>::BatchSummaryPruned { batch_id: cursor });
            true
        }

        fn track_pending_rotation_institution(institution: InstitutionPalletId) -> DispatchResult {
            PendingRotationInstitutions::<T>::try_mutate(|institutions| {
                if institutions.iter().any(|id| *id == institution) {
                    return Ok(());
                }
                institutions
                    .try_push(institution)
                    .map_err(|_| Error::<T>::PendingRotationInstitutionsOverflow)?;
                Ok(())
            })
        }

        fn ensure_rate_and_institution(
            institution: InstitutionPalletId,
        ) -> Result<u32, DispatchError> {
            ensure!(
                institution_pallet_address(institution).is_some(),
                Error::<T>::InvalidInstitution
            );
            let stored = InstitutionRateBp::<T>::get(institution);
            // 中文注释：默认费率为0.01%，未设置时按最低费率执行。
            let rate_bp = if stored == 0 {
                OFFCHAIN_RATE_BP_MIN
            } else {
                stored
            };
            ensure!(
                (OFFCHAIN_RATE_BP_MIN..=OFFCHAIN_RATE_BP_MAX).contains(&rate_bp),
                Error::<T>::InvalidRateBp
            );
            Ok(rate_bp)
        }

        fn is_prb_admin(institution: InstitutionPalletId, who: &T::AccountId) -> bool {
            <T as voting_engine_system::Config>::InternalAdminProvider::is_internal_admin(
                ORG_PRB,
                institution,
                who,
            )
        }

        fn try_execute_rate(proposal_id: u64) -> DispatchResult {
            let action = RateProposalActions::<T>::get(proposal_id)
                .ok_or(Error::<T>::RateProposalNotFound)?;

            let proposal = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;
            ensure!(
                proposal.kind == PROPOSAL_KIND_INTERNAL,
                Error::<T>::ProposalKindMismatch
            );
            ensure!(
                proposal.status == STATUS_PASSED,
                Error::<T>::ProposalStatusNotPassed
            );
            ensure!(
                proposal.internal_institution == Some(action.institution),
                Error::<T>::ProposalInstitutionMismatch
            );

            InstitutionRateBp::<T>::insert(action.institution, action.new_rate_bp);
            RateProposalActions::<T>::remove(proposal_id);

            Self::deposit_event(Event::<T>::InstitutionRateUpdated {
                proposal_id,
                institution: action.institution,
                rate_bp: action.new_rate_bp,
            });
            Ok(())
        }

        fn try_execute_verify_key(proposal_id: u64) -> DispatchResult {
            let action = VerifyKeyProposalActions::<T>::get(proposal_id)
                .ok_or(Error::<T>::VerifyKeyProposalNotFound)?;

            let proposal = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;
            ensure!(
                proposal.kind == PROPOSAL_KIND_INTERNAL,
                Error::<T>::ProposalKindMismatch
            );
            ensure!(
                proposal.status == STATUS_PASSED,
                Error::<T>::ProposalStatusNotPassed
            );
            ensure!(
                proposal.internal_institution == Some(action.institution),
                Error::<T>::ProposalInstitutionMismatch
            );

            let current_exists = VerifyKeys::<T>::contains_key(action.institution);
            let now = frame_system::Pallet::<T>::block_number();
            let activate_at = now.saturating_add(VERIFY_KEY_ROTATION_DELAY_BLOCKS.into());
            if current_exists {
                ensure!(
                    !PendingVerifyKeys::<T>::contains_key(action.institution),
                    Error::<T>::VerifyKeyRotationAlreadyScheduled
                );
                PendingVerifyKeys::<T>::insert(
                    action.institution,
                    PendingVerifyKey {
                        key: action.new_key.clone(),
                        activate_at,
                    },
                );
                Self::track_pending_rotation_institution(action.institution)?;
                VerifyKeyRotationStatuses::<T>::insert(
                    action.institution,
                    VerifyKeyRotationStatus {
                        stage: VerifyKeyRotationStage::Scheduled,
                        activate_at: Some(activate_at),
                    },
                );
            } else {
                VerifyKeys::<T>::insert(action.institution, &action.new_key);
                let epoch = VerifyKeyEpoch::<T>::get(action.institution);
                let next_epoch = epoch.checked_add(1).ok_or(Error::<T>::CounterOverflow)?;
                VerifyKeyEpoch::<T>::insert(action.institution, next_epoch);
                VerifyKeyRotationStatuses::<T>::insert(
                    action.institution,
                    VerifyKeyRotationStatus {
                        stage: VerifyKeyRotationStage::Idle,
                        activate_at: None,
                    },
                );
            }
            VerifyKeyProposalActions::<T>::remove(proposal_id);

            if current_exists {
                Self::deposit_event(Event::<T>::VerifyKeyRotationScheduled {
                    proposal_id,
                    institution: action.institution,
                    key_len: action.new_key.len() as u32,
                    activate_at,
                });
            } else {
                Self::deposit_event(Event::<T>::VerifyKeyUpdated {
                    proposal_id,
                    institution: action.institution,
                    key_len: action.new_key.len() as u32,
                });
            }
            Ok(())
        }

        fn try_execute_sweep(proposal_id: u64) -> DispatchResult {
            let action = SweepProposalActions::<T>::get(proposal_id)
                .ok_or(Error::<T>::SweepProposalNotFound)?;

            let proposal = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;
            ensure!(
                proposal.kind == PROPOSAL_KIND_INTERNAL,
                Error::<T>::ProposalKindMismatch
            );
            ensure!(
                proposal.status == STATUS_PASSED,
                Error::<T>::ProposalStatusNotPassed
            );
            ensure!(
                proposal.internal_institution == Some(action.institution),
                Error::<T>::ProposalInstitutionMismatch
            );

            let fee_account = Self::institution_fee_account(action.institution)?;
            let main_account = Self::institution_account(action.institution)?;
            ensure!(
                !T::ProtectedSourceChecker::is_protected(&fee_account),
                Error::<T>::ProtectedSource
            );

            let amount_u128: u128 = action.amount.saturated_into();
            let fee_balance_u128: u128 = T::Currency::free_balance(&fee_account).saturated_into();
            let reserve_u128 = FEE_ADDRESS_MIN_RESERVE_FEN;

            ensure!(
                fee_balance_u128 >= amount_u128
                    && fee_balance_u128.saturating_sub(amount_u128) >= reserve_u128,
                Error::<T>::InsufficientFeeReserve
            );
            let available_u128 = fee_balance_u128.saturating_sub(reserve_u128);
            let cap_u128 = available_u128
                .saturating_mul(FEE_SWEEP_MAX_PERCENT)
                .saturating_div(100);
            ensure!(amount_u128 <= cap_u128, Error::<T>::SweepAmountExceedsCap);

            T::Currency::transfer(
                &fee_account,
                &main_account,
                action.amount,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )?;
            // 中文注释：fee_account 余额仅允许经内部投票划转到 main_account（pallet_address）。

            let reserve_left: BalanceOf<T> = T::Currency::free_balance(&fee_account);

            SweepProposalActions::<T>::remove(proposal_id);

            Self::deposit_event(Event::<T>::SweepToMainExecuted {
                proposal_id,
                institution: action.institution,
                amount: action.amount,
                reserve_left,
            });
            Ok(())
        }

        fn try_execute_relay_submitters(proposal_id: u64) -> DispatchResult {
            let action = RelaySubmittersProposalActions::<T>::get(proposal_id)
                .ok_or(Error::<T>::RelaySubmittersProposalNotFound)?;
            Self::ensure_valid_relay_submitters(&action.submitters)?;

            let proposal = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalNotFound)?;
            ensure!(
                proposal.kind == PROPOSAL_KIND_INTERNAL,
                Error::<T>::ProposalKindMismatch
            );
            ensure!(
                proposal.status == STATUS_PASSED,
                Error::<T>::ProposalStatusNotPassed
            );
            ensure!(
                proposal.internal_institution == Some(action.institution),
                Error::<T>::ProposalInstitutionMismatch
            );

            let count = action.submitters.len() as u32;
            RelaySubmitters::<T>::insert(action.institution, action.submitters);
            RelaySubmittersProposalActions::<T>::remove(proposal_id);
            Self::deposit_event(Event::<T>::RelaySubmittersUpdated {
                proposal_id,
                institution: action.institution,
                count,
            });
            Ok(())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(now: BlockNumberFor<T>) -> Weight {
            let institutions = PendingRotationInstitutions::<T>::get();
            let mut reads: u64 = 1;
            let mut writes: u64 = 0;
            if institutions.is_empty() {
                return T::DbWeight::get().reads_writes(reads, writes);
            }
            let mut remaining: BoundedVec<InstitutionPalletId, ConstU32<64>> =
                BoundedVec::default();
            for institution in institutions.iter() {
                reads = reads.saturating_add(1);
                if let Some(pending) = PendingVerifyKeys::<T>::get(institution) {
                    if now >= pending.activate_at {
                        let key_len = pending.key.len() as u32;
                        VerifyKeys::<T>::insert(institution, pending.key);
                        let epoch = VerifyKeyEpoch::<T>::get(institution);
                        if let Some(next_epoch) = epoch.checked_add(1) {
                            VerifyKeyEpoch::<T>::insert(institution, next_epoch);
                            writes = writes.saturating_add(1);
                        }
                        PendingVerifyKeys::<T>::remove(institution);
                        VerifyKeyRotationStatuses::<T>::insert(
                            institution,
                            VerifyKeyRotationStatus {
                                stage: VerifyKeyRotationStage::Idle,
                                activate_at: None,
                            },
                        );
                        writes = writes.saturating_add(3);
                        Self::deposit_event(Event::<T>::VerifyKeyRotated {
                            institution: *institution,
                            key_len,
                            activated_at: now,
                        });
                    } else {
                        let _ = remaining.try_push(*institution);
                    }
                }
            }
            let changed = remaining != institutions;
            if changed {
                PendingRotationInstitutions::<T>::put(remaining);
                writes = writes.saturating_add(1);
            }
            T::DbWeight::get().reads_writes(reads, writes)
        }

        fn on_idle(now: BlockNumberFor<T>, remaining_weight: Weight) -> Weight {
            let db = T::DbWeight::get();
            let processed_budget = db.reads_writes(3, 4);
            let queued_budget = db.reads_writes(3, T::MaxBatchSize::get() as u64 + 2);
            let summary_budget = db.reads_writes(3, 2);
            let processed_idle = db.reads(2);
            let queued_idle = db.reads(2);
            let summary_idle = db.reads(2);

            let mut consumed = Weight::zero();

            if remaining_weight.all_gte(consumed.saturating_add(processed_budget)) {
                let used = if Self::auto_prune_one_processed_tx(now) {
                    processed_budget
                } else {
                    processed_idle
                };
                consumed = consumed.saturating_add(used);
            }

            if remaining_weight.all_gte(consumed.saturating_add(queued_budget)) {
                let used = if Self::auto_prune_one_queued_batch(now) {
                    queued_budget
                } else {
                    queued_idle
                };
                consumed = consumed.saturating_add(used);
            }

            if remaining_weight.all_gte(consumed.saturating_add(summary_budget)) {
                let used = if Self::auto_prune_one_batch_summary(now) {
                    summary_budget
                } else {
                    summary_idle
                };
                consumed = consumed.saturating_add(used);
            }

            consumed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codec::Encode;
    use frame_support::{assert_noop, assert_ok, derive_impl, traits::ConstU32};
    use frame_system as system;
    use sp_runtime::{
        traits::Hash as HashT, traits::IdentityLookup, AccountId32, BuildStorage, TokenError,
    };

    type Balance = u128;
    type Block = frame_system::mocking::MockBlock<Test>;

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
        pub struct Test;

        #[runtime::pallet_index(0)]
        pub type System = frame_system;

        #[runtime::pallet_index(1)]
        pub type Balances = pallet_balances;

        #[runtime::pallet_index(2)]
        pub type VotingEngineSystem = voting_engine_system;

        #[runtime::pallet_index(3)]
        pub type OffchainTransactionFee = super;
    }

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl system::Config for Test {
        type Block = Block;
        type AccountId = AccountId32;
        type Lookup = IdentityLookup<Self::AccountId>;
        type AccountData = pallet_balances::AccountData<Balance>;
    }

    impl pallet_balances::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type Balance = Balance;
        type DustRemoval = ();
        type ExistentialDeposit = frame_support::traits::ConstU128<1>;
        type AccountStore = System;
        type MaxLocks = ConstU32<0>;
        type MaxReserves = ();
        type ReserveIdentifier = [u8; 8];
        type FreezeIdentifier = RuntimeFreezeReason;
        type MaxFreezes = ConstU32<0>;
        type RuntimeHoldReason = RuntimeHoldReason;
        type RuntimeFreezeReason = RuntimeFreezeReason;
        type DoneSlashHandler = ();
        type WeightInfo = ();
    }

    pub struct TestSfidEligibility;
    impl voting_engine_system::SfidEligibility<AccountId32> for TestSfidEligibility {
        fn is_eligible(_sfid: &[u8], _who: &AccountId32) -> bool {
            true
        }

        fn verify_and_consume_vote_credential(
            _sfid: &[u8],
            _who: &AccountId32,
            _proposal_id: u64,
            _nonce: &[u8],
            _signature: &[u8],
        ) -> bool {
            true
        }
    }

    pub struct TestPopulationSnapshotVerifier;
    impl
        voting_engine_system::PopulationSnapshotVerifier<
            AccountId32,
            voting_engine_system::pallet::VoteNonceOf<Test>,
            voting_engine_system::pallet::VoteSignatureOf<Test>,
        > for TestPopulationSnapshotVerifier
    {
        fn verify_population_snapshot(
            _who: &AccountId32,
            _eligible_total: u64,
            _nonce: &voting_engine_system::pallet::VoteNonceOf<Test>,
            _signature: &voting_engine_system::pallet::VoteSignatureOf<Test>,
        ) -> bool {
            true
        }
    }

    pub struct TestInternalAdminProvider;
    impl voting_engine_system::InternalAdminProvider<AccountId32> for TestInternalAdminProvider {
        fn is_internal_admin(org: u8, institution: InstitutionPalletId, who: &AccountId32) -> bool {
            let who_bytes = who.encode();
            if who_bytes.len() != 32 {
                return false;
            }
            let mut who_arr = [0u8; 32];
            who_arr.copy_from_slice(&who_bytes);
            match org {
                voting_engine_system::internal_vote::ORG_PRB => CHINA_CH
                    .iter()
                    .find(|n| shengbank_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                    .map(|n| n.admins.iter().any(|admin| *admin == who_arr))
                    .unwrap_or(false),
                _ => false,
            }
        }
    }

    impl voting_engine_system::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type MaxSfidLength = ConstU32<64>;
        type MaxVoteNonceLength = ConstU32<64>;
        type MaxVoteSignatureLength = ConstU32<64>;
        type SfidEligibility = TestSfidEligibility;
        type PopulationSnapshotVerifier = TestPopulationSnapshotVerifier;
        type JointVoteResultCallback = ();
        type InternalAdminProvider = TestInternalAdminProvider;
    }

    pub struct TestOffchainBatchVerifier;
    impl OffchainBatchVerifier for TestOffchainBatchVerifier {
        fn verify(_verify_key: &[u8], _message: &[u8], signature: &[u8]) -> bool {
            signature == b"ok"
        }
    }

    impl pallet::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type Currency = Balances;
        type MaxVerifyKeyLen = ConstU32<128>;
        type MaxBatchSize = ConstU32<8>;
        type MaxBatchSignatureLength = ConstU32<64>;
        type MaxRelaySubmitters = ConstU32<8>;
        type InternalVoteEngine = voting_engine_system::Pallet<Test>;
        type OffchainBatchVerifier = TestOffchainBatchVerifier;
        type ProtectedSourceChecker = ();
        type WeightInfo = ();
    }

    fn prb_institution() -> InstitutionPalletId {
        shengbank_pallet_id_to_bytes(CHINA_CH[0].shenfen_id).expect("valid institution")
    }

    fn prb_admin(index: usize) -> AccountId32 {
        AccountId32::new(CHINA_CH[0].admins[index])
    }

    fn prb_account() -> AccountId32 {
        AccountId32::new(institution_pallet_address(prb_institution()).expect("prb account"))
    }

    fn prb_fee_account() -> AccountId32 {
        OffchainTransactionFee::fee_account_of(prb_institution()).expect("prb fee account")
    }

    fn prb_t2() -> [u8; 2] {
        institution_t2_code(prb_institution()).expect("t2")
    }

    fn relay_account() -> AccountId32 {
        AccountId32::new([11u8; 32])
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let mut storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("storage should build");

        let payer1 = AccountId32::new([1u8; 32]);
        let payer2 = AccountId32::new([2u8; 32]);
        let recipient1 = AccountId32::new([3u8; 32]);
        let recipient2 = AccountId32::new([4u8; 32]);

        pallet_balances::GenesisConfig::<Test> {
            balances: vec![
                (payer1, 20_000),
                (payer2, 20_000),
                (recipient1, 1),
                (recipient2, 1),
                (prb_account(), 1_000),
                (prb_fee_account(), 1_000),
                (relay_account(), 1_000),
            ],
            ..Default::default()
        }
        .assimilate_storage(&mut storage)
        .expect("balances should build");

        let mut ext = sp_io::TestExternalities::new(storage);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }

    #[test]
    fn calc_fee_round_and_min_work() {
        assert_eq!(calc_offchain_fee_fen(100, 1), 1); // 1.00 *0.01%=0.01 => 1分
        assert_eq!(calc_offchain_fee_fen(150, 1), 1); // 1.50 *0.01%=0.015 => 1分（四舍五入）
        assert_eq!(calc_offchain_fee_fen(151, 1), 1); // 0.0151分 => 1分
        assert_eq!(calc_offchain_fee_fen(1, 1), 1); // 最低1分
    }

    #[test]
    fn institution_t2_code_uses_r5_prefix() {
        let t2 = institution_t2_code(prb_institution()).expect("t2");
        assert_eq!(t2, *b"ZS");
    }

    #[test]
    fn rate_and_verify_key_update_require_internal_vote_pass() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();

            assert_ok!(OffchainTransactionFee::propose_institution_rate(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                5
            ));

            for i in 0..6 {
                assert_ok!(OffchainTransactionFee::vote_institution_rate(
                    RuntimeOrigin::signed(prb_admin(i)),
                    0,
                    true
                ));
            }

            assert_eq!(OffchainTransactionFee::rate_bp_of(institution), 5);

            let key: VerifyKeyOf<Test> = b"new-verify-key".to_vec().try_into().expect("fit");
            assert_ok!(OffchainTransactionFee::propose_verify_key(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                key.clone()
            ));

            for i in 0..6 {
                assert_ok!(OffchainTransactionFee::vote_verify_key(
                    RuntimeOrigin::signed(prb_admin(i)),
                    1,
                    true
                ));
            }

            assert_eq!(
                OffchainTransactionFee::verify_key_of(institution),
                Some(key)
            );
        });
    }

    #[test]
    fn non_admin_cannot_propose_or_vote() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            let non_admin = AccountId32::new([9u8; 32]);

            assert_noop!(
                OffchainTransactionFee::propose_institution_rate(
                    RuntimeOrigin::signed(non_admin.clone()),
                    institution,
                    3
                ),
                Error::<Test>::UnauthorizedAdmin
            );

            assert_noop!(
                OffchainTransactionFee::submit_offchain_batch(
                    RuntimeOrigin::signed(relay_account()),
                    institution,
                    1,
                    vec![BatchItemOf::<Test> {
                        tx_id: <Test as frame_system::Config>::Hashing::hash(b"bad-tx"),
                        payer: AccountId32::new([1u8; 32]),
                        recipient: AccountId32::new([2u8; 32]),
                        transfer_amount: 100,
                        offchain_fee_amount: 1,
                    }]
                    .try_into()
                    .expect("fit"),
                    b"ok".to_vec().try_into().expect("fit"),
                ),
                Error::<Test>::RelaySubmittersNotInitialized
            );
        });
    }

    #[test]
    fn submit_batch_executes_real_settlement_and_marks_processed() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();

            // 先通过内部投票把费率设为1bp（0.01%），便于构造样例。
            assert_ok!(OffchainTransactionFee::propose_institution_rate(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                1
            ));
            for i in 0..6 {
                assert_ok!(OffchainTransactionFee::vote_institution_rate(
                    RuntimeOrigin::signed(prb_admin(i)),
                    0,
                    true
                ));
            }

            let payer1 = AccountId32::new([1u8; 32]);
            let payer2 = AccountId32::new([2u8; 32]);
            let recipient1 = AccountId32::new([3u8; 32]);
            let recipient2 = AccountId32::new([4u8; 32]);

            let item1 = BatchItemOf::<Test> {
                tx_id: <Test as frame_system::Config>::Hashing::hash(b"tx-1"),
                payer: payer1,
                recipient: recipient1.clone(),
                transfer_amount: 1_000,
                offchain_fee_amount: 1,
            };
            let item2 = BatchItemOf::<Test> {
                tx_id: <Test as frame_system::Config>::Hashing::hash(b"tx-2"),
                payer: payer2,
                recipient: recipient2.clone(),
                transfer_amount: 2_000,
                offchain_fee_amount: 1,
            };
            let batch: BatchOf<Test> = vec![item1.clone(), item2.clone()].try_into().expect("fit");
            let fee_account = OffchainTransactionFee::fee_account_of(institution).expect("fee");
            let fee_before = Balances::free_balance(&fee_account);
            assert_ok!(OffchainTransactionFee::bind_clearing_institution(
                RuntimeOrigin::signed(recipient1),
                institution
            ));
            assert_ok!(OffchainTransactionFee::bind_clearing_institution(
                RuntimeOrigin::signed(recipient2),
                institution
            ));
            assert_ok!(OffchainTransactionFee::init_verify_key(
                RuntimeOrigin::signed(prb_account()),
                institution,
                b"default-key".to_vec().try_into().expect("fit")
            ));
            let relays: RelaySubmittersOf<Test> = vec![relay_account(), prb_admin(0), prb_admin(1)]
                .try_into()
                .expect("fit");
            assert_ok!(OffchainTransactionFee::init_relay_submitters(
                RuntimeOrigin::signed(prb_account()),
                institution,
                relays
            ));

            // 首次允许按时间阈值提交。
            assert_ok!(OffchainTransactionFee::submit_offchain_batch(
                RuntimeOrigin::signed(relay_account()),
                institution,
                1,
                batch,
                b"ok".to_vec().try_into().expect("fit"),
            ));

            assert_eq!(Balances::free_balance(&item1.recipient), 1_001);
            assert_eq!(Balances::free_balance(&item2.recipient), 2_001);
            assert_eq!(Balances::free_balance(&fee_account), fee_before + 2);
            let t2 = prb_t2();
            assert!(ProcessedOffchainTx::<Test>::get(t2, item1.tx_id));
            assert!(ProcessedOffchainTx::<Test>::get(t2, item2.tx_id));

            // 重放应被拒绝。
            System::set_block_number(System::block_number() + PACK_BLOCK_THRESHOLD as u64);
            let replay: BatchOf<Test> = vec![item1].try_into().expect("fit");
            assert_noop!(
                OffchainTransactionFee::submit_offchain_batch(
                    RuntimeOrigin::signed(relay_account()),
                    institution,
                    2,
                    replay,
                    b"ok".to_vec().try_into().expect("fit"),
                ),
                Error::<Test>::TxAlreadyProcessed
            );
        });
    }

    #[test]
    fn enqueue_rejects_already_processed_tx_id() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            assert_ok!(OffchainTransactionFee::propose_institution_rate(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                1
            ));
            for i in 0..6 {
                assert_ok!(OffchainTransactionFee::vote_institution_rate(
                    RuntimeOrigin::signed(prb_admin(i)),
                    0,
                    true
                ));
            }
            assert_ok!(OffchainTransactionFee::init_verify_key(
                RuntimeOrigin::signed(prb_account()),
                institution,
                b"default-key".to_vec().try_into().expect("fit")
            ));
            let relays: RelaySubmittersOf<Test> = vec![relay_account(), prb_admin(0), prb_admin(1)]
                .try_into()
                .expect("fit");
            assert_ok!(OffchainTransactionFee::init_relay_submitters(
                RuntimeOrigin::signed(prb_account()),
                institution,
                relays
            ));
            let payer = AccountId32::new([1u8; 32]);
            let recipient = AccountId32::new([3u8; 32]);
            assert_ok!(OffchainTransactionFee::bind_clearing_institution(
                RuntimeOrigin::signed(recipient.clone()),
                institution
            ));

            let tx_id = <Test as frame_system::Config>::Hashing::hash(b"enqueue-replay");
            let batch: BatchOf<Test> = vec![BatchItemOf::<Test> {
                tx_id,
                payer: payer.clone(),
                recipient: recipient.clone(),
                transfer_amount: 100,
                offchain_fee_amount: 1,
            }]
            .try_into()
            .expect("fit");
            assert_ok!(OffchainTransactionFee::submit_offchain_batch(
                RuntimeOrigin::signed(relay_account()),
                institution,
                1,
                batch,
                b"ok".to_vec().try_into().expect("fit"),
            ));

            System::set_block_number(System::block_number() + PACK_BLOCK_THRESHOLD as u64);
            let replay: BatchOf<Test> = vec![BatchItemOf::<Test> {
                tx_id,
                payer,
                recipient,
                transfer_amount: 100,
                offchain_fee_amount: 1,
            }]
            .try_into()
            .expect("fit");
            assert_noop!(
                OffchainTransactionFee::enqueue_offchain_batch(
                    RuntimeOrigin::signed(relay_account()),
                    institution,
                    2,
                    replay,
                    b"ok".to_vec().try_into().expect("fit"),
                ),
                Error::<Test>::TxAlreadyProcessed
            );
        });
    }

    #[test]
    fn enqueue_rejects_tx_id_already_in_pending_queue() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            assert_ok!(OffchainTransactionFee::propose_institution_rate(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                1
            ));
            for i in 0..6 {
                assert_ok!(OffchainTransactionFee::vote_institution_rate(
                    RuntimeOrigin::signed(prb_admin(i)),
                    0,
                    true
                ));
            }
            assert_ok!(OffchainTransactionFee::init_verify_key(
                RuntimeOrigin::signed(prb_account()),
                institution,
                b"default-key".to_vec().try_into().expect("fit")
            ));
            let relays: RelaySubmittersOf<Test> = vec![relay_account(), prb_admin(0), prb_admin(1)]
                .try_into()
                .expect("fit");
            assert_ok!(OffchainTransactionFee::init_relay_submitters(
                RuntimeOrigin::signed(prb_account()),
                institution,
                relays
            ));
            let payer = AccountId32::new([1u8; 32]);
            let recipient = AccountId32::new([3u8; 32]);
            assert_ok!(OffchainTransactionFee::bind_clearing_institution(
                RuntimeOrigin::signed(recipient.clone()),
                institution
            ));
            let tx_id = <Test as frame_system::Config>::Hashing::hash(b"queued-dup");
            let b1: BatchOf<Test> = vec![BatchItemOf::<Test> {
                tx_id,
                payer: payer.clone(),
                recipient: recipient.clone(),
                transfer_amount: 100,
                offchain_fee_amount: 1,
            }]
            .try_into()
            .expect("fit");
            assert_ok!(OffchainTransactionFee::enqueue_offchain_batch(
                RuntimeOrigin::signed(relay_account()),
                institution,
                1,
                b1,
                b"ok".to_vec().try_into().expect("fit")
            ));
            let b2: BatchOf<Test> = vec![BatchItemOf::<Test> {
                tx_id,
                payer,
                recipient,
                transfer_amount: 100,
                offchain_fee_amount: 1,
            }]
            .try_into()
            .expect("fit");
            assert_noop!(
                OffchainTransactionFee::enqueue_offchain_batch(
                    RuntimeOrigin::signed(relay_account()),
                    institution,
                    2,
                    b2,
                    b"ok".to_vec().try_into().expect("fit")
                ),
                Error::<Test>::TxAlreadyProcessed
            );
        });
    }

    #[test]
    fn prune_processed_tx_removes_expired_entry() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            assert_ok!(OffchainTransactionFee::propose_institution_rate(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                1
            ));
            for i in 0..6 {
                assert_ok!(OffchainTransactionFee::vote_institution_rate(
                    RuntimeOrigin::signed(prb_admin(i)),
                    0,
                    true
                ));
            }
            assert_ok!(OffchainTransactionFee::init_verify_key(
                RuntimeOrigin::signed(prb_account()),
                institution,
                b"default-key".to_vec().try_into().expect("fit")
            ));
            let relays: RelaySubmittersOf<Test> = vec![relay_account(), prb_admin(0), prb_admin(1)]
                .try_into()
                .expect("fit");
            assert_ok!(OffchainTransactionFee::init_relay_submitters(
                RuntimeOrigin::signed(prb_account()),
                institution,
                relays
            ));
            let payer = AccountId32::new([1u8; 32]);
            let recipient = AccountId32::new([3u8; 32]);
            assert_ok!(OffchainTransactionFee::bind_clearing_institution(
                RuntimeOrigin::signed(recipient.clone()),
                institution
            ));

            let tx_id = <Test as frame_system::Config>::Hashing::hash(b"prune-processed");
            let batch: BatchOf<Test> = vec![BatchItemOf::<Test> {
                tx_id,
                payer,
                recipient,
                transfer_amount: 100,
                offchain_fee_amount: 1,
            }]
            .try_into()
            .expect("fit");
            assert_ok!(OffchainTransactionFee::submit_offchain_batch(
                RuntimeOrigin::signed(relay_account()),
                institution,
                1,
                batch,
                b"ok".to_vec().try_into().expect("fit")
            ));
            let t2 = prb_t2();
            System::set_block_number(System::block_number() + PROCESSED_TX_RETENTION_BLOCKS as u64);
            assert_ok!(OffchainTransactionFee::prune_processed_tx(
                RuntimeOrigin::signed(relay_account()),
                t2,
                tx_id
            ));
            assert!(!ProcessedOffchainTx::<Test>::get(t2, tx_id));
        });
    }

    #[test]
    fn submit_rejects_zero_or_self_transfer() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            assert_ok!(OffchainTransactionFee::propose_institution_rate(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                1
            ));
            for i in 0..6 {
                assert_ok!(OffchainTransactionFee::vote_institution_rate(
                    RuntimeOrigin::signed(prb_admin(i)),
                    0,
                    true
                ));
            }
            assert_ok!(OffchainTransactionFee::init_verify_key(
                RuntimeOrigin::signed(prb_account()),
                institution,
                b"default-key".to_vec().try_into().expect("fit")
            ));
            let relays: RelaySubmittersOf<Test> = vec![relay_account(), prb_admin(0), prb_admin(1)]
                .try_into()
                .expect("fit");
            assert_ok!(OffchainTransactionFee::init_relay_submitters(
                RuntimeOrigin::signed(prb_account()),
                institution,
                relays
            ));
            let payer = AccountId32::new([1u8; 32]);
            let recipient = AccountId32::new([3u8; 32]);
            assert_ok!(OffchainTransactionFee::bind_clearing_institution(
                RuntimeOrigin::signed(recipient.clone()),
                institution
            ));

            let zero_tx = <Test as frame_system::Config>::Hashing::hash(b"zero-transfer");
            let zero_batch: BatchOf<Test> = vec![BatchItemOf::<Test> {
                tx_id: zero_tx,
                payer: payer.clone(),
                recipient: recipient.clone(),
                transfer_amount: 0,
                offchain_fee_amount: 1,
            }]
            .try_into()
            .expect("fit");
            assert_noop!(
                OffchainTransactionFee::submit_offchain_batch(
                    RuntimeOrigin::signed(relay_account()),
                    institution,
                    1,
                    zero_batch,
                    b"ok".to_vec().try_into().expect("fit")
                ),
                Error::<Test>::InvalidTransferAmount
            );

            let self_tx = <Test as frame_system::Config>::Hashing::hash(b"self-transfer");
            let self_batch: BatchOf<Test> = vec![BatchItemOf::<Test> {
                tx_id: self_tx,
                payer: payer.clone(),
                recipient: payer,
                transfer_amount: 100,
                offchain_fee_amount: 1,
            }]
            .try_into()
            .expect("fit");
            assert_noop!(
                OffchainTransactionFee::submit_offchain_batch(
                    RuntimeOrigin::signed(relay_account()),
                    institution,
                    1,
                    self_batch,
                    b"ok".to_vec().try_into().expect("fit")
                ),
                Error::<Test>::SelfTransferNotAllowed
            );
        });
    }

    #[test]
    fn init_verify_key_only_once_and_only_institution_account() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            let key: VerifyKeyOf<Test> = b"boot-default-key".to_vec().try_into().expect("fit");

            assert_noop!(
                OffchainTransactionFee::init_verify_key(
                    RuntimeOrigin::signed(prb_admin(0)),
                    institution,
                    key.clone()
                ),
                Error::<Test>::UnauthorizedSubmitter
            );

            assert_ok!(OffchainTransactionFee::init_verify_key(
                RuntimeOrigin::signed(prb_account()),
                institution,
                key.clone()
            ));
            assert_eq!(
                OffchainTransactionFee::verify_key_of(institution),
                Some(key.clone())
            );

            assert_noop!(
                OffchainTransactionFee::init_verify_key(
                    RuntimeOrigin::signed(prb_account()),
                    institution,
                    key
                ),
                Error::<Test>::VerifyKeyAlreadyInitialized
            );
        });
    }

    #[test]
    fn propose_verify_key_rejects_empty_key() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            let empty: VerifyKeyOf<Test> = vec![].try_into().expect("fit");
            assert_noop!(
                OffchainTransactionFee::propose_verify_key(
                    RuntimeOrigin::signed(prb_admin(0)),
                    institution,
                    empty
                ),
                Error::<Test>::InvalidVerifyKey
            );
        });
    }

    #[test]
    fn relay_submitters_can_be_rotated_by_internal_vote() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            let relays: RelaySubmittersOf<Test> = vec![relay_account(), prb_admin(0), prb_admin(1)]
                .try_into()
                .expect("fit");
            assert_ok!(OffchainTransactionFee::init_relay_submitters(
                RuntimeOrigin::signed(prb_account()),
                institution,
                relays
            ));
            let new_relay = AccountId32::new([77u8; 32]);
            let new_set: RelaySubmittersOf<Test> =
                vec![new_relay.clone(), prb_admin(2), prb_admin(3)]
                    .try_into()
                    .expect("fit");
            assert_ok!(OffchainTransactionFee::propose_relay_submitters(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                new_set.clone()
            ));
            for i in 0..6 {
                assert_ok!(OffchainTransactionFee::vote_relay_submitters(
                    RuntimeOrigin::signed(prb_admin(i)),
                    0,
                    true
                ));
            }
            assert_eq!(
                OffchainTransactionFee::relay_submitters_of(institution),
                Some(new_set)
            );
        });
    }

    #[test]
    fn prune_expired_proposal_action_removes_rejected_action() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            assert_ok!(OffchainTransactionFee::propose_institution_rate(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                5
            ));
            let proposal = voting_engine_system::Pallet::<Test>::proposals(0).expect("proposal");
            System::set_block_number(proposal.end.saturating_add(1));
            assert!(RateProposalActions::<Test>::contains_key(0));
            assert_ok!(OffchainTransactionFee::prune_expired_proposal_action(
                RuntimeOrigin::signed(relay_account()),
                0
            ));
            assert!(!RateProposalActions::<Test>::contains_key(0));
        });
    }

    #[test]
    fn verify_key_rotation_uses_pending_key_after_activation() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            let old_key: VerifyKeyOf<Test> = b"old-key".to_vec().try_into().expect("fit");
            let new_key: VerifyKeyOf<Test> = b"new-key".to_vec().try_into().expect("fit");

            assert_ok!(OffchainTransactionFee::init_verify_key(
                RuntimeOrigin::signed(prb_account()),
                institution,
                old_key.clone(),
            ));
            assert_eq!(
                OffchainTransactionFee::verify_key_for(institution),
                Some(old_key.clone())
            );

            assert_ok!(OffchainTransactionFee::propose_verify_key(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                new_key.clone(),
            ));
            for i in 0..6 {
                assert_ok!(OffchainTransactionFee::vote_verify_key(
                    RuntimeOrigin::signed(prb_admin(i)),
                    0,
                    true
                ));
            }

            let status = OffchainTransactionFee::rotation_status_of(institution)
                .expect("rotation status should exist");
            assert!(matches!(status.stage, VerifyKeyRotationStage::Scheduled));
            assert!(status.activate_at.is_some());

            // 生效前仍用旧密钥。
            assert_eq!(
                OffchainTransactionFee::verify_key_for(institution),
                Some(old_key.clone())
            );

            // 到达生效高度后切换为新密钥。
            System::set_block_number(
                System::block_number() + VERIFY_KEY_ROTATION_DELAY_BLOCKS as u64,
            );
            OffchainTransactionFee::on_initialize(System::block_number());
            assert_eq!(
                OffchainTransactionFee::verify_key_for(institution),
                Some(new_key)
            );
            let status = OffchainTransactionFee::rotation_status_of(institution)
                .expect("rotation status should exist");
            assert!(matches!(status.stage, VerifyKeyRotationStage::Idle));
            assert!(status.activate_at.is_none());
        });
    }

    #[test]
    fn sweep_to_main_requires_internal_vote_and_keeps_reserve() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            let fee_account = OffchainTransactionFee::fee_account_of(institution).expect("fee");
            let main_account = prb_account();
            let fee_before = Balances::free_balance(&fee_account);
            let _ = Balances::deposit_creating(&fee_account, 300_000u128);
            let main_before = Balances::free_balance(&main_account);

            assert_ok!(OffchainTransactionFee::propose_sweep_to_main(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                100_000
            ));
            for i in 0..6 {
                assert_ok!(OffchainTransactionFee::vote_sweep_to_main(
                    RuntimeOrigin::signed(prb_admin(i)),
                    0,
                    true
                ));
            }

            assert_eq!(Balances::free_balance(&main_account), main_before + 100_000);
            assert_eq!(Balances::free_balance(&fee_account), fee_before + 200_000);
            let mut last_reserve_left = None;
            for evt in System::events().iter().rev() {
                if let RuntimeEvent::OffchainTransactionFee(Event::<Test>::SweepToMainExecuted {
                    reserve_left,
                    ..
                }) = &evt.event
                {
                    last_reserve_left = Some(*reserve_left);
                    break;
                }
            }
            assert_eq!(
                last_reserve_left,
                Some(Balances::free_balance(&fee_account))
            );

            assert_ok!(OffchainTransactionFee::propose_sweep_to_main(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                100_000
            ));
            for i in 0..5 {
                assert_ok!(OffchainTransactionFee::vote_sweep_to_main(
                    RuntimeOrigin::signed(prb_admin(i)),
                    1,
                    true
                ));
            }
            assert_ok!(OffchainTransactionFee::vote_sweep_to_main(
                RuntimeOrigin::signed(prb_admin(5)),
                1,
                true
            ));
            assert_eq!(Balances::free_balance(&fee_account), fee_before + 200_000);
            let mut has_failed_event = false;
            for evt in System::events().iter().rev() {
                if let RuntimeEvent::OffchainTransactionFee(
                    Event::<Test>::InternalProposalExecutionFailed { proposal_id },
                ) = &evt.event
                {
                    if *proposal_id == 1 {
                        has_failed_event = true;
                        break;
                    }
                }
            }
            assert!(has_failed_event);
        });
    }

    #[test]
    fn propose_sweep_to_main_rejects_zero_amount() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            assert_noop!(
                OffchainTransactionFee::propose_sweep_to_main(
                    RuntimeOrigin::signed(prb_admin(0)),
                    institution,
                    0,
                ),
                Error::<Test>::InvalidSweepAmount
            );
        });
    }

    #[test]
    fn enqueue_offchain_batch_requires_next_seq() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            assert_ok!(OffchainTransactionFee::propose_institution_rate(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                1
            ));
            for i in 0..6 {
                assert_ok!(OffchainTransactionFee::vote_institution_rate(
                    RuntimeOrigin::signed(prb_admin(i)),
                    0,
                    true
                ));
            }
            assert_ok!(OffchainTransactionFee::init_verify_key(
                RuntimeOrigin::signed(prb_account()),
                institution,
                b"default-key".to_vec().try_into().expect("fit")
            ));
            let relays: RelaySubmittersOf<Test> = vec![relay_account(), prb_admin(0), prb_admin(1)]
                .try_into()
                .expect("fit");
            assert_ok!(OffchainTransactionFee::init_relay_submitters(
                RuntimeOrigin::signed(prb_account()),
                institution,
                relays
            ));

            let payer = AccountId32::new([1u8; 32]);
            let recipient = AccountId32::new([3u8; 32]);
            assert_ok!(OffchainTransactionFee::bind_clearing_institution(
                RuntimeOrigin::signed(recipient.clone()),
                institution
            ));

            let tx_id = <Test as frame_system::Config>::Hashing::hash(b"queue-seq-next");
            let batch: BatchOf<Test> = vec![BatchItemOf::<Test> {
                tx_id,
                payer,
                recipient,
                transfer_amount: 10,
                offchain_fee_amount: 1,
            }]
            .try_into()
            .expect("fit");

            assert_noop!(
                OffchainTransactionFee::enqueue_offchain_batch(
                    RuntimeOrigin::signed(relay_account()),
                    institution,
                    2,
                    batch.clone(),
                    b"ok".to_vec().try_into().expect("fit"),
                ),
                Error::<Test>::InvalidBatchSeq
            );

            assert_ok!(OffchainTransactionFee::enqueue_offchain_batch(
                RuntimeOrigin::signed(relay_account()),
                institution,
                1,
                batch.clone(),
                b"ok".to_vec().try_into().expect("fit"),
            ));

            let tx_id_2 = <Test as frame_system::Config>::Hashing::hash(b"queue-seq-next-2");
            let batch_2: BatchOf<Test> = vec![BatchItemOf::<Test> {
                tx_id: tx_id_2,
                payer: AccountId32::new([1u8; 32]),
                recipient: AccountId32::new([3u8; 32]),
                transfer_amount: 10,
                offchain_fee_amount: 1,
            }]
            .try_into()
            .expect("fit");
            assert_ok!(OffchainTransactionFee::enqueue_offchain_batch(
                RuntimeOrigin::signed(relay_account()),
                institution,
                2,
                batch_2,
                b"ok".to_vec().try_into().expect("fit"),
            ));
            assert_eq!(
                OffchainTransactionFee::next_enqueue_batch_seq_of(institution),
                3
            );

            assert_noop!(
                OffchainTransactionFee::enqueue_offchain_batch(
                    RuntimeOrigin::signed(relay_account()),
                    institution,
                    2,
                    batch,
                    b"ok".to_vec().try_into().expect("fit"),
                ),
                Error::<Test>::InvalidBatchSeq
            );
        });
    }

    #[test]
    fn submit_offchain_batch_rejects_when_queue_backlog_exists() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            assert_ok!(OffchainTransactionFee::propose_institution_rate(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                1
            ));
            for i in 0..6 {
                assert_ok!(OffchainTransactionFee::vote_institution_rate(
                    RuntimeOrigin::signed(prb_admin(i)),
                    0,
                    true
                ));
            }
            assert_ok!(OffchainTransactionFee::init_verify_key(
                RuntimeOrigin::signed(prb_account()),
                institution,
                b"default-key".to_vec().try_into().expect("fit")
            ));
            let relays: RelaySubmittersOf<Test> = vec![relay_account(), prb_admin(0), prb_admin(1)]
                .try_into()
                .expect("fit");
            assert_ok!(OffchainTransactionFee::init_relay_submitters(
                RuntimeOrigin::signed(prb_account()),
                institution,
                relays
            ));

            let payer = AccountId32::new([1u8; 32]);
            let recipient = AccountId32::new([3u8; 32]);
            assert_ok!(OffchainTransactionFee::bind_clearing_institution(
                RuntimeOrigin::signed(recipient.clone()),
                institution
            ));
            let _ = Balances::deposit_creating(&payer, 100_000);

            let tx_id = <Test as frame_system::Config>::Hashing::hash(b"queue-backlog-1");
            let batch: BatchOf<Test> = vec![BatchItemOf::<Test> {
                tx_id,
                payer: payer.clone(),
                recipient: recipient.clone(),
                transfer_amount: 100,
                offchain_fee_amount: 1,
            }]
            .try_into()
            .expect("fit");

            assert_ok!(OffchainTransactionFee::enqueue_offchain_batch(
                RuntimeOrigin::signed(relay_account()),
                institution,
                1,
                batch.clone(),
                b"ok".to_vec().try_into().expect("fit"),
            ));

            assert_noop!(
                OffchainTransactionFee::submit_offchain_batch(
                    RuntimeOrigin::signed(relay_account()),
                    institution,
                    1,
                    batch.clone(),
                    b"ok".to_vec().try_into().expect("fit"),
                ),
                Error::<Test>::QueuedBacklogExists
            );

            System::set_block_number(System::block_number() + PACK_BLOCK_THRESHOLD as u64);
            assert_ok!(OffchainTransactionFee::process_queued_batch(
                RuntimeOrigin::signed(relay_account()),
                0
            ));

            let tx_id2 = <Test as frame_system::Config>::Hashing::hash(b"queue-backlog-2");
            let batch2: BatchOf<Test> = vec![BatchItemOf::<Test> {
                tx_id: tx_id2,
                payer,
                recipient,
                transfer_amount: 100,
                offchain_fee_amount: 1,
            }]
            .try_into()
            .expect("fit");
            System::set_block_number(System::block_number() + PACK_BLOCK_THRESHOLD as u64);
            assert_ok!(OffchainTransactionFee::submit_offchain_batch(
                RuntimeOrigin::signed(relay_account()),
                institution,
                2,
                batch2,
                b"ok".to_vec().try_into().expect("fit"),
            ));
        });
    }

    #[test]
    fn submit_batch_requires_valid_signature() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            let item = BatchItemOf::<Test> {
                tx_id: <Test as frame_system::Config>::Hashing::hash(b"sig-check"),
                payer: AccountId32::new([1u8; 32]),
                recipient: AccountId32::new([3u8; 32]),
                transfer_amount: 100,
                offchain_fee_amount: 1,
            };
            let batch: BatchOf<Test> = vec![item].try_into().expect("fit");
            assert_ok!(OffchainTransactionFee::init_verify_key(
                RuntimeOrigin::signed(prb_account()),
                institution,
                b"default-key".to_vec().try_into().expect("fit")
            ));
            let relays: RelaySubmittersOf<Test> = vec![relay_account(), prb_admin(0), prb_admin(1)]
                .try_into()
                .expect("fit");
            assert_ok!(OffchainTransactionFee::init_relay_submitters(
                RuntimeOrigin::signed(prb_account()),
                institution,
                relays
            ));

            assert_noop!(
                OffchainTransactionFee::submit_offchain_batch(
                    RuntimeOrigin::signed(relay_account()),
                    institution,
                    1,
                    batch,
                    b"bad".to_vec().try_into().expect("fit"),
                ),
                Error::<Test>::InvalidBatchSignature
            );
        });
    }

    #[test]
    fn failed_submit_does_not_consume_seq_or_mark_processed_and_can_retry() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            assert_ok!(OffchainTransactionFee::propose_institution_rate(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                1
            ));
            for i in 0..6 {
                assert_ok!(OffchainTransactionFee::vote_institution_rate(
                    RuntimeOrigin::signed(prb_admin(i)),
                    0,
                    true
                ));
            }
            assert_ok!(OffchainTransactionFee::init_verify_key(
                RuntimeOrigin::signed(prb_account()),
                institution,
                b"default-key".to_vec().try_into().expect("fit")
            ));
            let relays: RelaySubmittersOf<Test> = vec![relay_account(), prb_admin(0), prb_admin(1)]
                .try_into()
                .expect("fit");
            assert_ok!(OffchainTransactionFee::init_relay_submitters(
                RuntimeOrigin::signed(prb_account()),
                institution,
                relays
            ));

            let payer = AccountId32::new([1u8; 32]);
            let recipient = AccountId32::new([3u8; 32]);
            let tx_id = <Test as frame_system::Config>::Hashing::hash(b"retry-tx");
            let batch: BatchOf<Test> = vec![BatchItemOf::<Test> {
                tx_id,
                payer: payer.clone(),
                recipient: recipient.clone(),
                transfer_amount: 30_000, // 初始余额不足，触发执行阶段失败
                offchain_fee_amount: 3,
            }]
            .try_into()
            .expect("fit");
            assert_ok!(OffchainTransactionFee::bind_clearing_institution(
                RuntimeOrigin::signed(recipient.clone()),
                institution
            ));

            let failed = OffchainTransactionFee::submit_offchain_batch(
                RuntimeOrigin::signed(relay_account()),
                institution,
                1,
                batch.clone(),
                b"ok".to_vec().try_into().expect("fit"),
            );
            assert!(matches!(
                failed,
                Err(sp_runtime::DispatchError::Token(
                    TokenError::FundsUnavailable
                ))
            ));
            assert_eq!(OffchainTransactionFee::last_batch_seq_of(institution), 0);
            assert!(!ProcessedOffchainTx::<Test>::get(prb_t2(), tx_id));

            // 补足余额后，同一批次序号可重提并成功。
            let _ = Balances::deposit_creating(&payer, 20_000);
            assert_ok!(OffchainTransactionFee::submit_offchain_batch(
                RuntimeOrigin::signed(relay_account()),
                institution,
                1,
                batch,
                b"ok".to_vec().try_into().expect("fit"),
            ));
            assert_eq!(OffchainTransactionFee::last_batch_seq_of(institution), 1);
            assert!(ProcessedOffchainTx::<Test>::get(prb_t2(), tx_id));
        });
    }

    #[test]
    fn recipient_bind_clearing_institution_switch_once_per_year() {
        new_test_ext().execute_with(|| {
            let recipient = AccountId32::new([3u8; 32]);
            let inst_1 = prb_institution();
            let inst_2 =
                shengbank_pallet_id_to_bytes(CHINA_CH[1].shenfen_id).expect("valid institution");

            assert_ok!(OffchainTransactionFee::bind_clearing_institution(
                RuntimeOrigin::signed(recipient.clone()),
                inst_1
            ));
            assert_eq!(
                OffchainTransactionFee::recipient_clearing_institution(recipient.clone()),
                Some(inst_1)
            );

            assert_noop!(
                OffchainTransactionFee::bind_clearing_institution(
                    RuntimeOrigin::signed(recipient.clone()),
                    inst_2
                ),
                Error::<Test>::ClearingInstitutionSwitchTooFrequent
            );

            System::set_block_number(
                System::block_number() + CLEARING_INSTITUTION_SWITCH_INTERVAL_BLOCKS,
            );
            assert_ok!(OffchainTransactionFee::bind_clearing_institution(
                RuntimeOrigin::signed(recipient.clone()),
                inst_2
            ));
            assert_eq!(
                OffchainTransactionFee::recipient_clearing_institution(recipient),
                Some(inst_2)
            );
        });
    }

    #[test]
    fn submit_batch_rejects_when_recipient_not_bound_or_mismatched() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            assert_ok!(OffchainTransactionFee::propose_institution_rate(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                1
            ));
            for i in 0..6 {
                assert_ok!(OffchainTransactionFee::vote_institution_rate(
                    RuntimeOrigin::signed(prb_admin(i)),
                    0,
                    true
                ));
            }
            assert_ok!(OffchainTransactionFee::init_verify_key(
                RuntimeOrigin::signed(prb_account()),
                institution,
                b"default-key".to_vec().try_into().expect("fit")
            ));
            let relays: RelaySubmittersOf<Test> = vec![relay_account(), prb_admin(0), prb_admin(1)]
                .try_into()
                .expect("fit");
            assert_ok!(OffchainTransactionFee::init_relay_submitters(
                RuntimeOrigin::signed(prb_account()),
                institution,
                relays
            ));

            let recipient = AccountId32::new([3u8; 32]);
            let batch_unbound: BatchOf<Test> = vec![BatchItemOf::<Test> {
                tx_id: <Test as frame_system::Config>::Hashing::hash(b"recipient-unbound"),
                payer: AccountId32::new([1u8; 32]),
                recipient: recipient.clone(),
                transfer_amount: 100,
                offchain_fee_amount: 1,
            }]
            .try_into()
            .expect("fit");
            assert_noop!(
                OffchainTransactionFee::submit_offchain_batch(
                    RuntimeOrigin::signed(relay_account()),
                    institution,
                    1,
                    batch_unbound,
                    b"ok".to_vec().try_into().expect("fit"),
                ),
                Error::<Test>::RecipientClearingInstitutionNotBound
            );

            let other_inst =
                shengbank_pallet_id_to_bytes(CHINA_CH[1].shenfen_id).expect("valid institution");
            assert_ok!(OffchainTransactionFee::bind_clearing_institution(
                RuntimeOrigin::signed(recipient.clone()),
                other_inst
            ));
            let batch_mismatch: BatchOf<Test> = vec![BatchItemOf::<Test> {
                tx_id: <Test as frame_system::Config>::Hashing::hash(b"recipient-mismatch"),
                payer: AccountId32::new([1u8; 32]),
                recipient,
                transfer_amount: 100,
                offchain_fee_amount: 1,
            }]
            .try_into()
            .expect("fit");
            assert_noop!(
                OffchainTransactionFee::submit_offchain_batch(
                    RuntimeOrigin::signed(relay_account()),
                    institution,
                    1,
                    batch_mismatch,
                    b"ok".to_vec().try_into().expect("fit"),
                ),
                Error::<Test>::RecipientClearingInstitutionMismatch
            );
        });
    }

    #[test]
    fn queued_batch_persists_on_failure_and_retries_until_success() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            assert_ok!(OffchainTransactionFee::propose_institution_rate(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                1
            ));
            for i in 0..6 {
                assert_ok!(OffchainTransactionFee::vote_institution_rate(
                    RuntimeOrigin::signed(prb_admin(i)),
                    0,
                    true
                ));
            }
            assert_ok!(OffchainTransactionFee::init_verify_key(
                RuntimeOrigin::signed(prb_account()),
                institution,
                b"default-key".to_vec().try_into().expect("fit")
            ));
            let relays: RelaySubmittersOf<Test> = vec![relay_account(), prb_admin(0), prb_admin(1)]
                .try_into()
                .expect("fit");
            assert_ok!(OffchainTransactionFee::init_relay_submitters(
                RuntimeOrigin::signed(prb_account()),
                institution,
                relays
            ));

            let payer = AccountId32::new([1u8; 32]);
            let recipient = AccountId32::new([3u8; 32]);
            assert_ok!(OffchainTransactionFee::bind_clearing_institution(
                RuntimeOrigin::signed(recipient.clone()),
                institution
            ));

            let tx_id = <Test as frame_system::Config>::Hashing::hash(b"queue-retry");
            let batch: BatchOf<Test> = vec![BatchItemOf::<Test> {
                tx_id,
                payer: payer.clone(),
                recipient,
                transfer_amount: 30_000, // 初始余额不足
                offchain_fee_amount: 3,
            }]
            .try_into()
            .expect("fit");

            assert_ok!(OffchainTransactionFee::enqueue_offchain_batch(
                RuntimeOrigin::signed(relay_account()),
                institution,
                1,
                batch,
                b"ok".to_vec().try_into().expect("fit"),
            ));
            assert_eq!(OffchainTransactionFee::next_queued_batch_id(), 1);

            assert_ok!(OffchainTransactionFee::process_queued_batch(
                RuntimeOrigin::signed(relay_account()),
                0
            ));
            let queued = OffchainTransactionFee::queued_batch_by_id(0).expect("queued");
            assert!(matches!(queued.status, QueuedBatchStatus::Pending));
            assert_eq!(queued.retry_count, 1);
            assert_eq!(
                queued.last_error,
                Some(QueuedBatchLastError::ExecutionFailed)
            );
            assert_eq!(OffchainTransactionFee::last_batch_seq_of(institution), 0);
            assert!(!ProcessedOffchainTx::<Test>::get(prb_t2(), tx_id));

            let _ = Balances::deposit_creating(&payer, 20_000);
            System::set_block_number(System::block_number() + PACK_BLOCK_THRESHOLD as u64);
            assert_ok!(OffchainTransactionFee::process_queued_batch(
                RuntimeOrigin::signed(relay_account()),
                0
            ));
            let queued = OffchainTransactionFee::queued_batch_by_id(0).expect("queued");
            assert!(matches!(queued.status, QueuedBatchStatus::Processed));
            assert_eq!(OffchainTransactionFee::last_batch_seq_of(institution), 1);
            assert!(ProcessedOffchainTx::<Test>::get(prb_t2(), tx_id));
        });
    }

    #[test]
    fn stress_queued_batches_many_rounds_should_keep_monotonic_seq() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            assert_ok!(OffchainTransactionFee::propose_institution_rate(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                1
            ));
            for i in 0..6 {
                assert_ok!(OffchainTransactionFee::vote_institution_rate(
                    RuntimeOrigin::signed(prb_admin(i)),
                    0,
                    true
                ));
            }
            assert_ok!(OffchainTransactionFee::init_verify_key(
                RuntimeOrigin::signed(prb_account()),
                institution,
                b"default-key".to_vec().try_into().expect("fit")
            ));
            let relays: RelaySubmittersOf<Test> = vec![relay_account(), prb_admin(0), prb_admin(1)]
                .try_into()
                .expect("fit");
            assert_ok!(OffchainTransactionFee::init_relay_submitters(
                RuntimeOrigin::signed(prb_account()),
                institution,
                relays
            ));

            let payer = AccountId32::new([1u8; 32]);
            let recipient = AccountId32::new([3u8; 32]);
            assert_ok!(OffchainTransactionFee::bind_clearing_institution(
                RuntimeOrigin::signed(recipient.clone()),
                institution
            ));

            let rounds: u64 = 200;
            for i in 0..rounds {
                let tx_id = <Test as frame_system::Config>::Hashing::hash_of(&i);
                let batch: BatchOf<Test> = vec![BatchItemOf::<Test> {
                    tx_id,
                    payer: payer.clone(),
                    recipient: recipient.clone(),
                    transfer_amount: 10,
                    offchain_fee_amount: 1,
                }]
                .try_into()
                .expect("fit");

                assert_ok!(OffchainTransactionFee::enqueue_offchain_batch(
                    RuntimeOrigin::signed(relay_account()),
                    institution,
                    i + 1,
                    batch,
                    b"ok".to_vec().try_into().expect("fit"),
                ));
                System::set_block_number(System::block_number() + PACK_BLOCK_THRESHOLD as u64);
                assert_ok!(OffchainTransactionFee::process_queued_batch(
                    RuntimeOrigin::signed(relay_account()),
                    i
                ));
            }

            assert_eq!(
                OffchainTransactionFee::last_batch_seq_of(institution),
                rounds
            );
            let last = OffchainTransactionFee::queued_batch_by_id(rounds - 1).expect("queued");
            assert!(matches!(last.status, QueuedBatchStatus::Processed));
        });
    }

    #[test]
    fn process_queued_batch_rejects_unauthorized_submitter_without_mutating_retry() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            assert_ok!(OffchainTransactionFee::propose_institution_rate(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                1
            ));
            for i in 0..6 {
                assert_ok!(OffchainTransactionFee::vote_institution_rate(
                    RuntimeOrigin::signed(prb_admin(i)),
                    0,
                    true
                ));
            }
            assert_ok!(OffchainTransactionFee::init_verify_key(
                RuntimeOrigin::signed(prb_account()),
                institution,
                b"default-key".to_vec().try_into().expect("fit")
            ));
            let relays: RelaySubmittersOf<Test> = vec![relay_account(), prb_admin(0), prb_admin(1)]
                .try_into()
                .expect("fit");
            assert_ok!(OffchainTransactionFee::init_relay_submitters(
                RuntimeOrigin::signed(prb_account()),
                institution,
                relays
            ));
            let payer = AccountId32::new([1u8; 32]);
            let recipient = AccountId32::new([3u8; 32]);
            assert_ok!(OffchainTransactionFee::bind_clearing_institution(
                RuntimeOrigin::signed(recipient.clone()),
                institution
            ));

            let tx_id = <Test as frame_system::Config>::Hashing::hash(b"unauthorized-queued");
            let batch: BatchOf<Test> = vec![BatchItemOf::<Test> {
                tx_id,
                payer,
                recipient,
                transfer_amount: 10,
                offchain_fee_amount: 1,
            }]
            .try_into()
            .expect("fit");
            assert_ok!(OffchainTransactionFee::enqueue_offchain_batch(
                RuntimeOrigin::signed(relay_account()),
                institution,
                1,
                batch,
                b"ok".to_vec().try_into().expect("fit"),
            ));

            let outsider = AccountId32::new([99u8; 32]);
            assert_noop!(
                OffchainTransactionFee::process_queued_batch(RuntimeOrigin::signed(outsider), 0),
                Error::<Test>::RelaySubmitterNotAllowed
            );
            let queued = OffchainTransactionFee::queued_batch_by_id(0).expect("queued");
            assert_eq!(queued.retry_count, 0);
            assert!(queued.last_error.is_none());
            assert!(matches!(queued.status, QueuedBatchStatus::Pending));
        });
    }

    #[test]
    fn process_queued_batch_records_precheck_failed_and_keeps_pending() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            assert_ok!(OffchainTransactionFee::propose_institution_rate(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                1
            ));
            for i in 0..6 {
                assert_ok!(OffchainTransactionFee::vote_institution_rate(
                    RuntimeOrigin::signed(prb_admin(i)),
                    0,
                    true
                ));
            }
            assert_ok!(OffchainTransactionFee::init_verify_key(
                RuntimeOrigin::signed(prb_account()),
                institution,
                b"default-key".to_vec().try_into().expect("fit")
            ));
            let relays: RelaySubmittersOf<Test> = vec![relay_account(), prb_admin(0), prb_admin(1)]
                .try_into()
                .expect("fit");
            assert_ok!(OffchainTransactionFee::init_relay_submitters(
                RuntimeOrigin::signed(prb_account()),
                institution,
                relays
            ));

            let payer = AccountId32::new([1u8; 32]);
            let recipient = AccountId32::new([3u8; 32]);
            assert_ok!(OffchainTransactionFee::bind_clearing_institution(
                RuntimeOrigin::signed(recipient.clone()),
                institution
            ));
            let _ = Balances::deposit_creating(&payer, 100_000);

            let tx1 = <Test as frame_system::Config>::Hashing::hash(b"precheck-fail-1");
            let batch1: BatchOf<Test> = vec![BatchItemOf::<Test> {
                tx_id: tx1,
                payer: payer.clone(),
                recipient: recipient.clone(),
                transfer_amount: 100,
                offchain_fee_amount: 1,
            }]
            .try_into()
            .expect("fit");
            assert_ok!(OffchainTransactionFee::enqueue_offchain_batch(
                RuntimeOrigin::signed(relay_account()),
                institution,
                1,
                batch1,
                b"ok".to_vec().try_into().expect("fit"),
            ));
            assert_ok!(OffchainTransactionFee::process_queued_batch(
                RuntimeOrigin::signed(relay_account()),
                0
            ));

            let tx2 = <Test as frame_system::Config>::Hashing::hash(b"precheck-fail-2");
            let batch2: BatchOf<Test> = vec![BatchItemOf::<Test> {
                tx_id: tx2,
                payer,
                recipient,
                transfer_amount: 100,
                offchain_fee_amount: 1,
            }]
            .try_into()
            .expect("fit");
            assert_ok!(OffchainTransactionFee::enqueue_offchain_batch(
                RuntimeOrigin::signed(relay_account()),
                institution,
                2,
                batch2,
                b"ok".to_vec().try_into().expect("fit"),
            ));

            // 故障注入测试：人为篡改已入队数据，验证预检失败会归类为 PrecheckFailed。
            QueuedBatches::<Test>::mutate(1, |maybe| {
                if let Some(inner) = maybe {
                    inner.batch[0].transfer_amount = 0;
                }
            });
            System::set_block_number(System::block_number() + PACK_BLOCK_THRESHOLD as u64 + 1);
            assert_ok!(OffchainTransactionFee::process_queued_batch(
                RuntimeOrigin::signed(relay_account()),
                1
            ));
            let queued = OffchainTransactionFee::queued_batch_by_id(1).expect("queued");
            assert!(matches!(queued.status, QueuedBatchStatus::Pending));
            assert_eq!(queued.retry_count, 1);
            assert_eq!(
                queued.last_error,
                Some(QueuedBatchLastError::PrecheckFailed)
            );
        });
    }

    #[test]
    fn process_queued_batch_marks_failed_after_max_retry() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            assert_ok!(OffchainTransactionFee::propose_institution_rate(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                1
            ));
            for i in 0..6 {
                assert_ok!(OffchainTransactionFee::vote_institution_rate(
                    RuntimeOrigin::signed(prb_admin(i)),
                    0,
                    true
                ));
            }
            assert_ok!(OffchainTransactionFee::init_verify_key(
                RuntimeOrigin::signed(prb_account()),
                institution,
                b"default-key".to_vec().try_into().expect("fit")
            ));
            let relays: RelaySubmittersOf<Test> = vec![relay_account(), prb_admin(0), prb_admin(1)]
                .try_into()
                .expect("fit");
            assert_ok!(OffchainTransactionFee::init_relay_submitters(
                RuntimeOrigin::signed(prb_account()),
                institution,
                relays
            ));
            let payer = AccountId32::new([1u8; 32]);
            let recipient = AccountId32::new([3u8; 32]);
            assert_ok!(OffchainTransactionFee::bind_clearing_institution(
                RuntimeOrigin::signed(recipient.clone()),
                institution
            ));

            let tx = <Test as frame_system::Config>::Hashing::hash(b"max-retry");
            let batch: BatchOf<Test> = vec![BatchItemOf::<Test> {
                tx_id: tx,
                payer,
                recipient,
                transfer_amount: 100,
                offchain_fee_amount: 1,
            }]
            .try_into()
            .expect("fit");
            assert_ok!(OffchainTransactionFee::enqueue_offchain_batch(
                RuntimeOrigin::signed(relay_account()),
                institution,
                1,
                batch,
                b"ok".to_vec().try_into().expect("fit"),
            ));

            QueuedBatches::<Test>::mutate(0, |maybe| {
                if let Some(inner) = maybe {
                    inner.retry_count = MAX_QUEUE_RETRY_COUNT - 1;
                    inner.batch[0].transfer_amount = 0;
                }
            });
            assert_ok!(OffchainTransactionFee::process_queued_batch(
                RuntimeOrigin::signed(relay_account()),
                0
            ));
            let queued = OffchainTransactionFee::queued_batch_by_id(0).expect("queued");
            assert!(matches!(queued.status, QueuedBatchStatus::Failed));
            assert_eq!(queued.retry_count, MAX_QUEUE_RETRY_COUNT);
            assert_eq!(
                queued.last_error,
                Some(QueuedBatchLastError::PrecheckFailed)
            );
        });
    }

    #[test]
    fn skip_failed_batch_unblocks_waiting_sequence() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            assert_ok!(OffchainTransactionFee::propose_institution_rate(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                1
            ));
            for i in 0..6 {
                assert_ok!(OffchainTransactionFee::vote_institution_rate(
                    RuntimeOrigin::signed(prb_admin(i)),
                    0,
                    true
                ));
            }
            assert_ok!(OffchainTransactionFee::init_verify_key(
                RuntimeOrigin::signed(prb_account()),
                institution,
                b"default-key".to_vec().try_into().expect("fit")
            ));
            let relays: RelaySubmittersOf<Test> = vec![relay_account(), prb_admin(0), prb_admin(1)]
                .try_into()
                .expect("fit");
            assert_ok!(OffchainTransactionFee::init_relay_submitters(
                RuntimeOrigin::signed(prb_account()),
                institution,
                relays
            ));

            let payer = AccountId32::new([1u8; 32]);
            let recipient = AccountId32::new([3u8; 32]);
            assert_ok!(OffchainTransactionFee::bind_clearing_institution(
                RuntimeOrigin::signed(recipient.clone()),
                institution
            ));

            let tx1 = <Test as frame_system::Config>::Hashing::hash(b"skip-failed-1");
            let batch1: BatchOf<Test> = vec![BatchItemOf::<Test> {
                tx_id: tx1,
                payer: payer.clone(),
                recipient: recipient.clone(),
                transfer_amount: 100,
                offchain_fee_amount: 1,
            }]
            .try_into()
            .expect("fit");
            assert_ok!(OffchainTransactionFee::enqueue_offchain_batch(
                RuntimeOrigin::signed(relay_account()),
                institution,
                1,
                batch1,
                b"ok".to_vec().try_into().expect("fit")
            ));

            let tx2 = <Test as frame_system::Config>::Hashing::hash(b"skip-failed-2");
            let batch2: BatchOf<Test> = vec![BatchItemOf::<Test> {
                tx_id: tx2,
                payer,
                recipient,
                transfer_amount: 100,
                offchain_fee_amount: 1,
            }]
            .try_into()
            .expect("fit");
            assert_ok!(OffchainTransactionFee::enqueue_offchain_batch(
                RuntimeOrigin::signed(relay_account()),
                institution,
                2,
                batch2,
                b"ok".to_vec().try_into().expect("fit")
            ));

            // 让 seq=1 到达失败上限。
            QueuedBatches::<Test>::mutate(0, |maybe| {
                if let Some(inner) = maybe {
                    inner.retry_count = MAX_QUEUE_RETRY_COUNT - 1;
                    inner.batch[0].transfer_amount = 0;
                }
            });
            assert_ok!(OffchainTransactionFee::process_queued_batch(
                RuntimeOrigin::signed(relay_account()),
                0
            ));
            assert!(matches!(
                OffchainTransactionFee::queued_batch_by_id(0)
                    .expect("queued")
                    .status,
                QueuedBatchStatus::Failed
            ));
            assert_eq!(OffchainTransactionFee::last_batch_seq_of(institution), 0);

            // seq=2 会被阻塞。
            assert_ok!(OffchainTransactionFee::process_queued_batch(
                RuntimeOrigin::signed(relay_account()),
                1
            ));
            let blocked = OffchainTransactionFee::queued_batch_by_id(1).expect("queued");
            assert_eq!(
                blocked.last_error,
                Some(QueuedBatchLastError::WaitingForPriorBatch)
            );

            // 管理员跳过失败批次后，seq=2 可继续执行。
            assert_ok!(OffchainTransactionFee::skip_failed_batch(
                RuntimeOrigin::signed(prb_admin(0)),
                0
            ));
            assert_eq!(OffchainTransactionFee::last_batch_seq_of(institution), 1);
            assert_ok!(OffchainTransactionFee::process_queued_batch(
                RuntimeOrigin::signed(relay_account()),
                1
            ));
            assert!(matches!(
                OffchainTransactionFee::queued_batch_by_id(1)
                    .expect("queued")
                    .status,
                QueuedBatchStatus::Processed
            ));
            assert_eq!(OffchainTransactionFee::last_batch_seq_of(institution), 2);
        });
    }

    #[test]
    fn cancel_queued_batch_requires_head_seq() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            assert_ok!(OffchainTransactionFee::propose_institution_rate(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                1
            ));
            for i in 0..6 {
                assert_ok!(OffchainTransactionFee::vote_institution_rate(
                    RuntimeOrigin::signed(prb_admin(i)),
                    0,
                    true
                ));
            }
            assert_ok!(OffchainTransactionFee::init_verify_key(
                RuntimeOrigin::signed(prb_account()),
                institution,
                b"default-key".to_vec().try_into().expect("fit")
            ));
            let relays: RelaySubmittersOf<Test> = vec![relay_account(), prb_admin(0), prb_admin(1)]
                .try_into()
                .expect("fit");
            assert_ok!(OffchainTransactionFee::init_relay_submitters(
                RuntimeOrigin::signed(prb_account()),
                institution,
                relays
            ));

            let payer = AccountId32::new([1u8; 32]);
            let recipient = AccountId32::new([3u8; 32]);
            assert_ok!(OffchainTransactionFee::bind_clearing_institution(
                RuntimeOrigin::signed(recipient.clone()),
                institution
            ));

            for (seq, seed) in [(1u64, b"cancel-head-1"), (2u64, b"cancel-head-2")] {
                let tx = <Test as frame_system::Config>::Hashing::hash(seed);
                let batch: BatchOf<Test> = vec![BatchItemOf::<Test> {
                    tx_id: tx,
                    payer: payer.clone(),
                    recipient: recipient.clone(),
                    transfer_amount: 100,
                    offchain_fee_amount: 1,
                }]
                .try_into()
                .expect("fit");
                assert_ok!(OffchainTransactionFee::enqueue_offchain_batch(
                    RuntimeOrigin::signed(relay_account()),
                    institution,
                    seq,
                    batch,
                    b"ok".to_vec().try_into().expect("fit")
                ));
            }

            assert_noop!(
                OffchainTransactionFee::cancel_queued_batch(RuntimeOrigin::signed(prb_admin(0)), 1),
                Error::<Test>::InvalidBatchSeq
            );
        });
    }

    #[test]
    fn emergency_rotate_verify_key_applies_immediately() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            let old_key: VerifyKeyOf<Test> = b"default-key".to_vec().try_into().expect("fit");
            assert_ok!(OffchainTransactionFee::init_verify_key(
                RuntimeOrigin::signed(prb_account()),
                institution,
                old_key
            ));
            let pending: VerifyKeyOf<Test> = b"pending-key".to_vec().try_into().expect("fit");
            assert_ok!(OffchainTransactionFee::propose_verify_key(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                pending
            ));
            for i in 0..6 {
                assert_ok!(OffchainTransactionFee::vote_verify_key(
                    RuntimeOrigin::signed(prb_admin(i)),
                    0,
                    true
                ));
            }
            assert!(OffchainTransactionFee::pending_verify_key_of(institution).is_some());

            let emergency_key: VerifyKeyOf<Test> =
                b"emergency-key".to_vec().try_into().expect("fit");
            assert_ok!(OffchainTransactionFee::emergency_rotate_verify_key(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                emergency_key.clone(),
            ));
            assert_ok!(OffchainTransactionFee::emergency_rotate_verify_key(
                RuntimeOrigin::signed(prb_admin(1)),
                institution,
                emergency_key.clone()
            ));
            assert_eq!(
                OffchainTransactionFee::verify_key_of(institution).expect("key"),
                emergency_key
            );
            assert!(OffchainTransactionFee::pending_verify_key_of(institution).is_none());
        });
    }

    #[test]
    fn queued_batch_is_invalidated_after_emergency_key_rotation() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            assert_ok!(OffchainTransactionFee::propose_institution_rate(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                1
            ));
            for i in 0..6 {
                assert_ok!(OffchainTransactionFee::vote_institution_rate(
                    RuntimeOrigin::signed(prb_admin(i)),
                    0,
                    true
                ));
            }
            assert_ok!(OffchainTransactionFee::init_verify_key(
                RuntimeOrigin::signed(prb_account()),
                institution,
                b"default-key".to_vec().try_into().expect("fit")
            ));
            let relays: RelaySubmittersOf<Test> = vec![relay_account(), prb_admin(0), prb_admin(1)]
                .try_into()
                .expect("fit");
            assert_ok!(OffchainTransactionFee::init_relay_submitters(
                RuntimeOrigin::signed(prb_account()),
                institution,
                relays
            ));
            let payer = AccountId32::new([1u8; 32]);
            let recipient = AccountId32::new([3u8; 32]);
            assert_ok!(OffchainTransactionFee::bind_clearing_institution(
                RuntimeOrigin::signed(recipient.clone()),
                institution
            ));

            let tx = <Test as frame_system::Config>::Hashing::hash(b"rot-invalidate");
            let batch: BatchOf<Test> = vec![BatchItemOf::<Test> {
                tx_id: tx,
                payer: payer.clone(),
                recipient,
                transfer_amount: 100,
                offchain_fee_amount: 1,
            }]
            .try_into()
            .expect("fit");
            assert_ok!(OffchainTransactionFee::enqueue_offchain_batch(
                RuntimeOrigin::signed(relay_account()),
                institution,
                1,
                batch,
                b"ok".to_vec().try_into().expect("fit"),
            ));

            let emergency_key: VerifyKeyOf<Test> =
                b"emergency-key-2".to_vec().try_into().expect("fit");
            assert_ok!(OffchainTransactionFee::emergency_rotate_verify_key(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                emergency_key.clone(),
            ));
            assert_ok!(OffchainTransactionFee::emergency_rotate_verify_key(
                RuntimeOrigin::signed(prb_admin(1)),
                institution,
                emergency_key,
            ));

            let payer_before = Balances::free_balance(&payer);
            assert_ok!(OffchainTransactionFee::process_queued_batch(
                RuntimeOrigin::signed(relay_account()),
                0
            ));
            let queued = OffchainTransactionFee::queued_batch_by_id(0).expect("queued");
            assert!(matches!(queued.status, QueuedBatchStatus::Cancelled));
            assert_eq!(queued.last_error, Some(QueuedBatchLastError::Cancelled));
            assert_eq!(Balances::free_balance(&payer), payer_before);
        });
    }

    #[test]
    fn verify_key_rotation_does_not_overwrite_existing_pending() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            assert_ok!(OffchainTransactionFee::init_verify_key(
                RuntimeOrigin::signed(prb_account()),
                institution,
                b"default-key".to_vec().try_into().expect("fit")
            ));

            let key_a: VerifyKeyOf<Test> = b"pending-a".to_vec().try_into().expect("fit");
            assert_ok!(OffchainTransactionFee::propose_verify_key(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                key_a.clone()
            ));
            for i in 0..6 {
                assert_ok!(OffchainTransactionFee::vote_verify_key(
                    RuntimeOrigin::signed(prb_admin(i)),
                    0,
                    true
                ));
            }
            let pending_a = OffchainTransactionFee::pending_verify_key_of(institution).expect("a");
            assert_eq!(pending_a.key, key_a);

            let key_b: VerifyKeyOf<Test> = b"pending-b".to_vec().try_into().expect("fit");
            assert_ok!(OffchainTransactionFee::propose_verify_key(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                key_b
            ));
            for i in 0..6 {
                assert_ok!(OffchainTransactionFee::vote_verify_key(
                    RuntimeOrigin::signed(prb_admin(i)),
                    1,
                    true
                ));
            }
            let pending_after = OffchainTransactionFee::pending_verify_key_of(institution)
                .expect("pending still exists");
            assert_eq!(pending_after.key, key_a);
        });
    }

    #[test]
    fn submit_rejects_duplicate_tx_id_in_same_batch() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            assert_ok!(OffchainTransactionFee::propose_institution_rate(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                1
            ));
            for i in 0..6 {
                assert_ok!(OffchainTransactionFee::vote_institution_rate(
                    RuntimeOrigin::signed(prb_admin(i)),
                    0,
                    true
                ));
            }
            assert_ok!(OffchainTransactionFee::init_verify_key(
                RuntimeOrigin::signed(prb_account()),
                institution,
                b"default-key".to_vec().try_into().expect("fit")
            ));
            let relays: RelaySubmittersOf<Test> = vec![relay_account(), prb_admin(0), prb_admin(1)]
                .try_into()
                .expect("fit");
            assert_ok!(OffchainTransactionFee::init_relay_submitters(
                RuntimeOrigin::signed(prb_account()),
                institution,
                relays
            ));
            let payer = AccountId32::new([1u8; 32]);
            let recipient = AccountId32::new([3u8; 32]);
            assert_ok!(OffchainTransactionFee::bind_clearing_institution(
                RuntimeOrigin::signed(recipient.clone()),
                institution
            ));

            let tx = <Test as frame_system::Config>::Hashing::hash(b"dup-tx-id");
            let batch: BatchOf<Test> = vec![
                BatchItemOf::<Test> {
                    tx_id: tx,
                    payer: payer.clone(),
                    recipient: recipient.clone(),
                    transfer_amount: 100,
                    offchain_fee_amount: 1,
                },
                BatchItemOf::<Test> {
                    tx_id: tx,
                    payer,
                    recipient,
                    transfer_amount: 100,
                    offchain_fee_amount: 1,
                },
            ]
            .try_into()
            .expect("fit");
            assert_noop!(
                OffchainTransactionFee::submit_offchain_batch(
                    RuntimeOrigin::signed(relay_account()),
                    institution,
                    1,
                    batch,
                    b"ok".to_vec().try_into().expect("fit")
                ),
                Error::<Test>::DuplicateTxIdInBatch
            );
        });
    }

    #[test]
    fn prune_processed_tx_without_timestamp_is_allowed() {
        new_test_ext().execute_with(|| {
            let tx_id = <Test as frame_system::Config>::Hashing::hash(b"legacy-no-ts");
            let t2 = prb_t2();
            ProcessedOffchainTx::<Test>::insert(t2, tx_id, true);
            ProcessedOffchainTxAt::<Test>::remove(t2, tx_id);
            assert_ok!(OffchainTransactionFee::prune_processed_tx(
                RuntimeOrigin::signed(relay_account()),
                t2,
                tx_id
            ));
            assert!(!ProcessedOffchainTx::<Test>::get(t2, tx_id));
        });
    }

    #[test]
    fn process_queued_batch_pack_threshold_not_reached_does_not_consume_retry() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            assert_ok!(OffchainTransactionFee::propose_institution_rate(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                1
            ));
            for i in 0..6 {
                assert_ok!(OffchainTransactionFee::vote_institution_rate(
                    RuntimeOrigin::signed(prb_admin(i)),
                    0,
                    true
                ));
            }
            assert_ok!(OffchainTransactionFee::init_verify_key(
                RuntimeOrigin::signed(prb_account()),
                institution,
                b"default-key".to_vec().try_into().expect("fit")
            ));
            let relays: RelaySubmittersOf<Test> = vec![relay_account(), prb_admin(0), prb_admin(1)]
                .try_into()
                .expect("fit");
            assert_ok!(OffchainTransactionFee::init_relay_submitters(
                RuntimeOrigin::signed(prb_account()),
                institution,
                relays
            ));
            let payer = AccountId32::new([1u8; 32]);
            let recipient = AccountId32::new([3u8; 32]);
            assert_ok!(OffchainTransactionFee::bind_clearing_institution(
                RuntimeOrigin::signed(recipient.clone()),
                institution
            ));
            let _ = Balances::deposit_creating(&payer, 100_000);

            let tx1 = <Test as frame_system::Config>::Hashing::hash(b"pack-threshold-1");
            let batch1: BatchOf<Test> = vec![BatchItemOf::<Test> {
                tx_id: tx1,
                payer: payer.clone(),
                recipient: recipient.clone(),
                transfer_amount: 100,
                offchain_fee_amount: 1,
            }]
            .try_into()
            .expect("fit");
            assert_ok!(OffchainTransactionFee::enqueue_offchain_batch(
                RuntimeOrigin::signed(relay_account()),
                institution,
                1,
                batch1,
                b"ok".to_vec().try_into().expect("fit"),
            ));
            assert_ok!(OffchainTransactionFee::process_queued_batch(
                RuntimeOrigin::signed(relay_account()),
                0
            ));

            let tx2 = <Test as frame_system::Config>::Hashing::hash(b"pack-threshold-2");
            let batch2: BatchOf<Test> = vec![BatchItemOf::<Test> {
                tx_id: tx2,
                payer,
                recipient,
                transfer_amount: 100,
                offchain_fee_amount: 1,
            }]
            .try_into()
            .expect("fit");
            assert_ok!(OffchainTransactionFee::enqueue_offchain_batch(
                RuntimeOrigin::signed(relay_account()),
                institution,
                2,
                batch2,
                b"ok".to_vec().try_into().expect("fit"),
            ));

            assert_ok!(OffchainTransactionFee::process_queued_batch(
                RuntimeOrigin::signed(relay_account()),
                1
            ));
            let queued = OffchainTransactionFee::queued_batch_by_id(1).expect("queued");
            assert!(matches!(queued.status, QueuedBatchStatus::Pending));
            assert_eq!(queued.retry_count, 0);
            assert_eq!(
                queued.last_error,
                Some(QueuedBatchLastError::PackThresholdNotReached)
            );
        });
    }

    #[test]
    fn process_queued_batch_rejects_missing_config_without_mutating_retry() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            assert_ok!(OffchainTransactionFee::propose_institution_rate(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                1
            ));
            for i in 0..6 {
                assert_ok!(OffchainTransactionFee::vote_institution_rate(
                    RuntimeOrigin::signed(prb_admin(i)),
                    0,
                    true
                ));
            }
            assert_ok!(OffchainTransactionFee::init_verify_key(
                RuntimeOrigin::signed(prb_account()),
                institution,
                b"default-key".to_vec().try_into().expect("fit")
            ));
            let relays: RelaySubmittersOf<Test> = vec![relay_account(), prb_admin(0), prb_admin(1)]
                .try_into()
                .expect("fit");
            assert_ok!(OffchainTransactionFee::init_relay_submitters(
                RuntimeOrigin::signed(prb_account()),
                institution,
                relays
            ));
            let payer = AccountId32::new([1u8; 32]);
            let recipient = AccountId32::new([3u8; 32]);
            assert_ok!(OffchainTransactionFee::bind_clearing_institution(
                RuntimeOrigin::signed(recipient.clone()),
                institution
            ));
            let _ = Balances::deposit_creating(&payer, 100_000);

            let tx = <Test as frame_system::Config>::Hashing::hash(b"missing-config");
            let batch: BatchOf<Test> = vec![BatchItemOf::<Test> {
                tx_id: tx,
                payer,
                recipient,
                transfer_amount: 100,
                offchain_fee_amount: 1,
            }]
            .try_into()
            .expect("fit");
            assert_ok!(OffchainTransactionFee::enqueue_offchain_batch(
                RuntimeOrigin::signed(relay_account()),
                institution,
                1,
                batch,
                b"ok".to_vec().try_into().expect("fit"),
            ));

            VerifyKeys::<Test>::remove(institution);
            assert_noop!(
                OffchainTransactionFee::process_queued_batch(
                    RuntimeOrigin::signed(relay_account()),
                    0
                ),
                Error::<Test>::VerifyKeyMissing
            );
            let queued = OffchainTransactionFee::queued_batch_by_id(0).expect("queued");
            assert_eq!(queued.retry_count, 0);
            assert!(queued.last_error.is_none());
            assert!(matches!(queued.status, QueuedBatchStatus::Pending));
        });
    }

    #[test]
    fn process_queued_batch_waiting_for_prior_batch_is_observable_without_retry() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            assert_ok!(OffchainTransactionFee::propose_institution_rate(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                1
            ));
            for i in 0..6 {
                assert_ok!(OffchainTransactionFee::vote_institution_rate(
                    RuntimeOrigin::signed(prb_admin(i)),
                    0,
                    true
                ));
            }
            assert_ok!(OffchainTransactionFee::init_verify_key(
                RuntimeOrigin::signed(prb_account()),
                institution,
                b"default-key".to_vec().try_into().expect("fit")
            ));
            let relays: RelaySubmittersOf<Test> = vec![relay_account(), prb_admin(0), prb_admin(1)]
                .try_into()
                .expect("fit");
            assert_ok!(OffchainTransactionFee::init_relay_submitters(
                RuntimeOrigin::signed(prb_account()),
                institution,
                relays
            ));
            let payer = AccountId32::new([1u8; 32]);
            let recipient = AccountId32::new([3u8; 32]);
            assert_ok!(OffchainTransactionFee::bind_clearing_institution(
                RuntimeOrigin::signed(recipient.clone()),
                institution
            ));
            let _ = Balances::deposit_creating(&payer, 100_000);

            let tx1 = <Test as frame_system::Config>::Hashing::hash(b"wait-seq-1");
            let batch1: BatchOf<Test> = vec![BatchItemOf::<Test> {
                tx_id: tx1,
                payer: payer.clone(),
                recipient: recipient.clone(),
                transfer_amount: 100,
                offchain_fee_amount: 1,
            }]
            .try_into()
            .expect("fit");
            let tx2 = <Test as frame_system::Config>::Hashing::hash(b"wait-seq-2");
            let batch2: BatchOf<Test> = vec![BatchItemOf::<Test> {
                tx_id: tx2,
                payer,
                recipient,
                transfer_amount: 100,
                offchain_fee_amount: 1,
            }]
            .try_into()
            .expect("fit");

            assert_ok!(OffchainTransactionFee::enqueue_offchain_batch(
                RuntimeOrigin::signed(relay_account()),
                institution,
                1,
                batch1,
                b"ok".to_vec().try_into().expect("fit"),
            ));
            assert_ok!(OffchainTransactionFee::enqueue_offchain_batch(
                RuntimeOrigin::signed(relay_account()),
                institution,
                2,
                batch2,
                b"ok".to_vec().try_into().expect("fit"),
            ));

            // 先处理 seq=2，触发 InvalidBatchSeq，应记录 WaitingForPriorBatch 且不增加 retry。
            assert_ok!(OffchainTransactionFee::process_queued_batch(
                RuntimeOrigin::signed(relay_account()),
                1
            ));
            let queued = OffchainTransactionFee::queued_batch_by_id(1).expect("queued");
            assert!(matches!(queued.status, QueuedBatchStatus::Pending));
            assert_eq!(queued.retry_count, 0);
            assert_eq!(
                queued.last_error,
                Some(QueuedBatchLastError::WaitingForPriorBatch)
            );
        });
    }
}
