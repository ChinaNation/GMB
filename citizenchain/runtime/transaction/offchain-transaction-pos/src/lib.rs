#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod weights;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    ensure,
    pallet_prelude::*,
    storage::{with_transaction, TransactionOutcome},
    traits::Currency,
    traits::StorageVersion,
    weights::Weight,
    Blake2_128Concat,
};
use frame_system::pallet_prelude::*;
use institution_asset_guard::{InstitutionAssetAction, InstitutionAssetGuard};
use scale_info::TypeInfo;
use sp_runtime::traits::{SaturatedConversion, Saturating, Zero};
use sp_std::vec::Vec;

use primitives::china::china_ch::{
    shenfen_id_to_fixed48 as shengbank_pallet_id_to_bytes, CHINA_CH,
};
use voting_engine_system::{
    internal_vote::ORG_PRB, InstitutionPalletId, InternalVoteEngine, PROPOSAL_KIND_INTERNAL,
    STATUS_EXECUTED, STATUS_PASSED, STATUS_REJECTED,
};

type BalanceOf<T> =
    <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

const OFFCHAIN_RATE_BP_MIN: u32 = 1; // 0.01%
const OFFCHAIN_RATE_BP_MAX: u32 = 10; // 0.1%
const PACK_TX_THRESHOLD: u64 = 100_000;
const PACK_BLOCK_THRESHOLD: u32 = primitives::pow_const::BLOCKS_PER_HOUR as u32; // 60分钟
const OFFCHAIN_MIN_FEE_FEN: u128 = primitives::core_const::OFFCHAIN_MIN_FEE;
const BP_DENOMINATOR: u128 = 10_000;
const PROCESSED_TX_RETENTION_BLOCKS: u64 = primitives::pow_const::BLOCKS_PER_YEAR;
const QUEUED_BATCH_RETENTION_BLOCKS: u64 = primitives::pow_const::BLOCKS_PER_YEAR;
const BATCH_SUMMARY_RETENTION_BLOCKS: u64 = primitives::pow_const::BLOCKS_PER_YEAR;
const MAX_QUEUE_RETRY_COUNT: u32 = 50;
const MAX_STALE_CANCEL_SCAN_STEPS: u32 = 1024;
const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

fn institution_pallet_address(institution: InstitutionPalletId) -> Option<[u8; 32]> {
    CHINA_CH
        .iter()
        .find(|n| shengbank_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
        .map(|n| n.duoqian_address)
}

fn institution_fee_address(institution: InstitutionPalletId) -> Option<[u8; 32]> {
    CHINA_CH
        .iter()
        .find(|n| shengbank_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
        .map(|n| n.fee_address)
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
    let half_up_threshold = (denominator / 2).saturating_add(denominator % 2);
    let should_round_up = remainder >= half_up_threshold;
    if should_round_up {
        Some(quotient.saturating_add(1))
    } else {
        Some(quotient)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FeeCalcError {
    AmountOverflow,
}

fn calc_offchain_fee_fen(amount_fen: u128, rate_bp: u32) -> Result<u128, FeeCalcError> {
    let numerator = amount_fen
        .checked_mul(rate_bp as u128)
        .ok_or(FeeCalcError::AmountOverflow)?;
    // 中文注释：费率计算先按 bp 换算，再做四舍五入到“分”，最后再套最低手续费保护。
    let by_rate =
        round_div(numerator, BP_DENOMINATOR).expect("BP_DENOMINATOR must be non-zero; qed");
    Ok(by_rate.max(OFFCHAIN_MIN_FEE_FEN))
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

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;
    use voting_engine_system::InternalAdminProvider;

    #[pallet::config]
    pub trait Config: frame_system::Config + voting_engine_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Currency: Currency<Self::AccountId>;

        #[pallet::constant]
        type MaxBatchSize: Get<u32>;

        #[pallet::constant]
        type MaxBatchSignatureLength: Get<u32>;

        /// 中文注释：内部投票引擎，返回真实 proposal_id。
        type InternalVoteEngine: voting_engine_system::InternalVoteEngine<Self::AccountId>;

        type ProtectedSourceChecker: ProtectedSourceChecker<Self::AccountId>;
        type InstitutionAssetGuard: institution_asset_guard::InstitutionAssetGuard<Self::AccountId>;
        type WeightInfo: crate::weights::WeightInfo;
    }

    pub type BatchSignatureOf<T> = BoundedVec<u8, <T as Config>::MaxBatchSignatureLength>;

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
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
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

    #[pallet::storage]
    pub type QueuedBatchPruneCursor<T> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    pub type BatchSummaryPruneCursor<T> = StorageValue<_, u64, ValueQuery>;

    /// 收款账户绑定的链下清算省储行。
    #[pallet::storage]
    #[pallet::getter(fn recipient_clearing_institution)]
    pub type RecipientClearingInstitution<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, InstitutionPalletId, OptionQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub initial_rates: Vec<(Vec<u8>, u32)>,
        #[serde(skip)]
        pub _phantom: core::marker::PhantomData<T>,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                initial_rates: Vec::new(),
                _phantom: Default::default(),
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
        InternalProposalExecutionFailed {
            proposal_id: u64,
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
        OffchainStaleQueuedBatchesCancelled {
            institution: InstitutionPalletId,
            cancelled: u32,
            operator: T::AccountId,
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
        UnauthorizedAdmin,
        UnauthorizedSubmitter,
        TxAlreadyProcessed,
        PackThresholdNotReached,
        EmptyBatch,
        InvalidBatchSignature,
        ProtectedSource,
        InvalidBatchSeq,
        QueuedBacklogExists,
        RecipientClearingInstitutionNotBound,
        RecipientClearingInstitutionMismatch,
        QueuedBatchNotFound,
        QueuedBatchAlreadyProcessed,
        QueuedBatchNotProcessed,
        QueuedBatchNotFailed,
        QueuedBatchNotPending,
        QueuedBatchNotSkippable,
        BatchSummaryNotFound,
        ProcessedTxNotFound,
        QueueRetentionNotReached,
        BatchSummaryRetentionNotReached,
        ProcessedTxRetentionNotReached,
        MaxQueueRetryExceeded,
        ProposalActionNotFound,
        ProposalNotPrunable,
        ProposalExecutionRetryNotAllowed,
        CounterOverflow,
        InvalidOperationCount,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 省储行链下批次上链：
        /// - 可由任意中继账户提交（fee_address 无私钥）；
        /// - 达到 N 或 T 触发条件才允许提交；
        /// - 必须通过”本机构验证密钥”对批次做签名校验；
        /// - 执行时主金额 payer->recipient，链下手续费 payer->fee_address；
        /// - 本次上链交易的链上手续费由 fee_address 自动承担。
        #[pallet::call_index(0)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::submit_offchain_batch(T::MaxBatchSize::get()))]
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
            // 中文注释：直接管理员验证 + sr25519 签名校验，不再需要 verify key。
            let (t2, by_count, by_time) =
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

        /// 收款方账户绑定链下清算省储行：
        /// - 不限制绑定次数和更换频率（每次调用收取最低手续费）。
        #[pallet::call_index(9)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
        pub fn bind_clearing_institution(
            origin: OriginFor<T>,
            institution: InstitutionPalletId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                institution_pallet_address(institution).is_some(),
                Error::<T>::InvalidInstitution
            );

            let switched = RecipientClearingInstitution::<T>::get(&who)
                .map_or(false, |current| current != institution);

            RecipientClearingInstitution::<T>::insert(&who, institution);
            Self::deposit_event(Event::<T>::RecipientClearingInstitutionBound {
                recipient: who,
                institution,
                switched,
            });
            Ok(())
        }

        /// 将批次持久化进入出队队列（先落库，再由管理员反复重试打包）。
        #[pallet::call_index(10)]
        #[pallet::weight(<T as pallet::Config>::WeightInfo::enqueue_offchain_batch(T::MaxBatchSize::get()))]
        pub fn enqueue_offchain_batch(
            origin: OriginFor<T>,
            institution: InstitutionPalletId,
            batch_seq: u64,
            batch: BatchOf<T>,
            batch_signature: BatchSignatureOf<T>,
        ) -> DispatchResult {
            let submitter = ensure_signed(origin)?;
            ensure!(!batch.is_empty(), Error::<T>::EmptyBatch);
            // 中文注释：入队时验证管理员身份和 sr25519 签名。
            ensure!(
                Self::is_prb_admin(institution, &submitter),
                Error::<T>::UnauthorizedAdmin
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
            // 中文注释：直接用提交者公钥验证批量签名，无需额外 verify key。
            let submitter_bytes: [u8; 32] = submitter.encode()[..32].try_into()
                .map_err(|_| Error::<T>::InvalidBatchSignature)?;
            let message = Self::batch_signing_message(institution, batch_seq, &batch);
            let sig = sp_core::sr25519::Signature::try_from(batch_signature.as_slice())
                .map_err(|_| Error::<T>::InvalidBatchSignature)?;
            let pub_key = sp_core::sr25519::Public::from_raw(submitter_bytes);
            ensure!(
                sp_io::crypto::sr25519_verify(&sig, message.as_slice(), &pub_key),
                Error::<T>::InvalidBatchSignature
            );

            let t2 = institution_t2_code(institution).ok_or(Error::<T>::InvalidInstitution)?;
            let fee_sum_u128 =
                Self::validate_batch_items(&batch, institution, t2, rate_bp, true, true)?;

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
        #[pallet::weight(<T as pallet::Config>::WeightInfo::process_queued_batch(T::MaxBatchSize::get()))]
        pub fn process_queued_batch(origin: OriginFor<T>, queue_id: u64) -> DispatchResult {
            let submitter = ensure_signed(origin)?;
            let mut queued =
                QueuedBatches::<T>::get(queue_id).ok_or(Error::<T>::QueuedBatchNotFound)?;
            ensure!(
                matches!(queued.status, QueuedBatchStatus::Pending),
                Error::<T>::QueuedBatchNotPending
            );

            let now = frame_system::Pallet::<T>::block_number();
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
            let (t2, by_count, by_time) = match precheck_result {
                Ok(v) => v,
                Err(e) => {
                    if Self::should_bubble_precheck_error(&e) {
                        return Err(e);
                    }
                    if Self::should_wait_precheck_error(&e) {
                        queued.last_error = Some(QueuedBatchLastError::WaitingForPriorBatch);
                        queued.last_attempt_at = Some(now);
                        QueuedBatches::<T>::insert(queue_id, &queued);
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
                        QueuedBatches::<T>::insert(queue_id, &queued);
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
                        QueuedBatches::<T>::insert(queue_id, &queued);
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
                    QueuedBatches::<T>::insert(queue_id, &queued);
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
                    QueuedBatches::<T>::insert(queue_id, &queued);
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
                        QueuedBatches::<T>::insert(queue_id, &queued);
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
                    QueuedBatches::<T>::insert(queue_id, &queued);
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

            T::InternalVoteEngine::cast_internal_vote(who.clone(), proposal_id, approve)?;

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

        /// 清理已处理且超过保留窗口的队列批次记录。
        #[pallet::call_index(14)]
        #[pallet::weight(T::DbWeight::get().reads_writes(3, 3 + T::MaxBatchSize::get() as u64))]
        pub fn prune_queued_batch(origin: OriginFor<T>, queue_id: u64) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            let queued =
                QueuedBatches::<T>::get(queue_id).ok_or(Error::<T>::QueuedBatchNotFound)?;
            let is_pending = matches!(queued.status, QueuedBatchStatus::Pending);
            let finalized_at = match queued.status {
                QueuedBatchStatus::Processed => queued.processed_at,
                QueuedBatchStatus::Failed => queued.last_attempt_at,
                QueuedBatchStatus::Cancelled => queued.last_attempt_at,
                QueuedBatchStatus::Pending => Some(queued.enqueued_at),
            };
            let Some(finalized_at) = finalized_at else {
                return Err(Error::<T>::QueuedBatchNotProcessed.into());
            };
            let now = frame_system::Pallet::<T>::block_number();
            let elapsed: u64 = now.saturating_sub(finalized_at).saturated_into();
            ensure!(
                elapsed >= QUEUED_BATCH_RETENTION_BLOCKS,
                Error::<T>::QueueRetentionNotReached
            );
            if is_pending {
                let current_seq = LastBatchSeq::<T>::get(queued.institution);
                let expected_seq = current_seq.saturating_add(1);
                if queued.batch_seq == expected_seq {
                    LastBatchSeq::<T>::insert(queued.institution, queued.batch_seq);
                }
            }
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

            let pruned = RateProposalActions::<T>::take(proposal_id).is_some();
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
        #[pallet::weight(T::DbWeight::get().reads_writes(3, 3 + T::MaxBatchSize::get() as u64))]
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

            let action = RateProposalActions::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            ensure!(
                Self::is_prb_admin(action.institution, &who),
                Error::<T>::UnauthorizedAdmin
            );

            with_transaction(|| {
                match Self::try_execute_rate(proposal_id) {
                    Ok(()) => TransactionOutcome::Commit(Ok(())),
                    Err(e) => TransactionOutcome::Rollback(Err(e)),
                }
            })?;
            Self::deposit_event(Event::<T>::ProposalExecutionRetried { proposal_id });
            Ok(())
        }

        /// 批量取消指定机构已过保留期的 Pending 队列批次（仅管理员）。
        #[pallet::call_index(23)]
        #[pallet::weight(T::DbWeight::get().reads_writes(
            5 + *max_count as u64 * 3,
            2 + *max_count as u64 * (4 + T::MaxBatchSize::get() as u64),
        ))]
        pub fn cancel_stale_queued_batches(
            origin: OriginFor<T>,
            institution: InstitutionPalletId,
            max_count: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                Self::is_prb_admin(institution, &who),
                Error::<T>::UnauthorizedAdmin
            );
            ensure!(max_count > 0, Error::<T>::InvalidOperationCount);

            let now = frame_system::Pallet::<T>::block_number();
            let next_queue_id = NextQueuedBatchId::<T>::get();
            // 中文注释：移除了 StaleCancelCursorByInstitution，直接从 prune cursor 开始扫描。
            let mut queue_id = QueuedBatchPruneCursor::<T>::get();
            if queue_id >= next_queue_id {
                queue_id = 0;
            }
            let mut scanned: u32 = 0;
            let mut cancelled: u32 = 0;
            while queue_id < next_queue_id
                && scanned < max_count
                && cancelled < max_count
                && scanned < MAX_STALE_CANCEL_SCAN_STEPS
            {
                scanned = scanned.saturating_add(1);
                if let Some(mut queued) = QueuedBatches::<T>::get(queue_id) {
                    if queued.institution == institution
                        && matches!(queued.status, QueuedBatchStatus::Pending)
                    {
                        let elapsed: u64 = now.saturating_sub(queued.enqueued_at).saturated_into();
                        if elapsed >= QUEUED_BATCH_RETENTION_BLOCKS {
                            let current = LastBatchSeq::<T>::get(queued.institution);
                            let expected_seq =
                                current.checked_add(1).ok_or(Error::<T>::CounterOverflow)?;
                            if queued.batch_seq == expected_seq {
                                let t2 = institution_t2_code(queued.institution)
                                    .ok_or(Error::<T>::InvalidInstitution)?;
                                queued.status = QueuedBatchStatus::Cancelled;
                                queued.last_attempt_at = Some(now);
                                queued.last_error = Some(QueuedBatchLastError::Cancelled);
                                LastBatchSeq::<T>::insert(queued.institution, queued.batch_seq);
                                QueuedBatches::<T>::insert(queue_id, queued.clone());
                                for item in queued.batch.iter() {
                                    QueuedTxIndex::<T>::remove(t2, item.tx_id);
                                }
                                cancelled = cancelled.saturating_add(1);
                                Self::deposit_event(Event::<T>::OffchainQueuedBatchCancelled {
                                    queue_id,
                                    institution: queued.institution,
                                    operator: who.clone(),
                                });
                            }
                        }
                    }
                }
                queue_id = queue_id.saturating_add(1);
            }
            Self::deposit_event(Event::<T>::OffchainStaleQueuedBatchesCancelled {
                institution,
                cancelled,
                operator: who,
            });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// 批次提交前置校验（无写入），供 runtime 扣费前判断是否允许 fee_address 承担手续费。
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
                Error::<T>::QueuedBatchNotPending
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
        ) -> Result<([u8; 2], bool, bool), DispatchError> {
            ensure!(!batch.is_empty(), Error::<T>::EmptyBatch);
            // 中文注释：验证提交者是否为机构管理员，替代原有的 relay submitter 白名单。
            ensure!(
                Self::is_prb_admin(institution, submitter),
                Error::<T>::UnauthorizedAdmin
            );
            let expected_seq = LastBatchSeq::<T>::get(institution)
                .checked_add(1)
                .ok_or(Error::<T>::CounterOverflow)?;
            ensure!(batch_seq == expected_seq, Error::<T>::InvalidBatchSeq);
            if verify_signature {
                // 中文注释：直接用提交者公钥验证批量签名，无需额外 verify key。
                let submitter_bytes: [u8; 32] = submitter.encode()[..32].try_into()
                    .map_err(|_| Error::<T>::InvalidBatchSignature)?;
                let message = Self::batch_signing_message(institution, batch_seq, batch);
                let sig = sp_core::sr25519::Signature::try_from(batch_signature.as_slice())
                    .map_err(|_| Error::<T>::InvalidBatchSignature)?;
                let pub_key = sp_core::sr25519::Public::from_raw(submitter_bytes);
                ensure!(
                    sp_io::crypto::sr25519_verify(&sig, message.as_slice(), &pub_key),
                    Error::<T>::InvalidBatchSignature
                );
            }

            let now = frame_system::Pallet::<T>::block_number();
            let last = LastPackBlock::<T>::get(institution);
            let (by_count, by_time) = Self::pack_trigger_reason(last, now, batch.len() as u64);
            // 中文注释：链下批次必须满足”按笔数触发”或”按时间触发”其一，避免碎片化频繁上链。
            ensure!(by_count || by_time, Error::<T>::PackThresholdNotReached);
            let t2 = institution_t2_code(institution).ok_or(Error::<T>::InvalidInstitution)?;
            let _ = Self::validate_batch_items(batch, institution, t2, rate_bp, verify_fee, false)?;
            Ok((t2, by_count, by_time))
        }

        fn execute_batch(
            submitter: &T::AccountId,
            institution: InstitutionPalletId,
            batch_seq: u64,
            batch: &BatchOf<T>,
            t2: [u8; 2],
            by_count: bool,
            by_time: bool,
        ) -> Result<u64, DispatchError> {
            let fee_account = Self::institution_fee_account(institution)?;
            let now = frame_system::Pallet::<T>::block_number();

            let mut total_transfer_u128: u128 = 0;
            let mut total_fee_u128: u128 = 0;
            for item in batch.iter() {
                ensure!(
                    T::InstitutionAssetGuard::can_spend(
                        &item.payer,
                        InstitutionAssetAction::OffchainBatchDebit,
                    ),
                    Error::<T>::ProtectedSource
                );
                // 中文注释：单条批次项按“主金额到账 + 链下手续费入 fee_account”两笔转账执行，
                // 任意一步失败都会被外围 with_transaction 回滚，避免批次半成功。
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
                // 中文注释：为 processed tx 追加顺序日志，供 on_idle 做有界清理，避免全表扫描。
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
            // 中文注释：使用提交者公钥的哈希作为签名者标识，替代原有的 verify key 哈希。
            let signer_key_hash = sp_io::hashing::blake2_256(&submitter.encode());

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

        fn batch_signing_message(
            institution: InstitutionPalletId,
            batch_seq: u64,
            batch: &BatchOf<T>,
        ) -> [u8; 32] {
            sp_io::hashing::blake2_256(
                &(b"GMB_OFFCHAIN_BATCH_V1", institution, batch_seq, batch).encode(),
            )
        }

        /// 中文注释：暴露签名消息构造函数供测试使用。
        #[cfg(test)]
        pub fn batch_signing_message_for_test(
            institution: InstitutionPalletId,
            batch_seq: u64,
            batch: &BatchOf<T>,
        ) -> [u8; 32] {
            Self::batch_signing_message(institution, batch_seq, batch)
        }

        fn institution_fee_account(
            institution: InstitutionPalletId,
        ) -> Result<T::AccountId, DispatchError> {
            let raw = institution_fee_address(institution)
                .ok_or(Error::<T>::InvalidInstitution)?;
            T::AccountId::decode(&mut &raw[..])
                .map_err(|_| Error::<T>::InvalidInstitution.into())
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
            // 中文注释：管理员身份校验失败应立即冒泡，不走重试逻辑。
            *e == Error::<T>::UnauthorizedAdmin.into()
        }

        fn should_ignore_precheck_error(e: &DispatchError) -> bool {
            *e == Error::<T>::PackThresholdNotReached.into()
        }

        fn should_wait_precheck_error(e: &DispatchError) -> bool {
            *e == Error::<T>::InvalidBatchSeq.into()
        }

        fn ensure_no_duplicate_tx_ids(batch: &BatchOf<T>) -> DispatchResult {
            for (idx, a) in batch.iter().enumerate() {
                for b in batch.iter().skip(idx + 1) {
                    ensure!(a.tx_id != b.tx_id, Error::<T>::DuplicateTxIdInBatch);
                }
            }
            Ok(())
        }

        fn validate_batch_items(
            batch: &BatchOf<T>,
            institution: InstitutionPalletId,
            t2: [u8; 2],
            rate_bp: u32,
            verify_fee: bool,
            check_queued_index: bool,
        ) -> Result<u128, DispatchError> {
            Self::ensure_no_duplicate_tx_ids(batch)?;
            let mut fee_sum_u128: u128 = 0;
            for item in batch.iter() {
                // 中文注释：tx_id 既不能命中已处理窗口，也不能与当前待处理队列中的项重复，
                // 这样 direct path 和 enqueue path 都能共享同一套防重放语义。
                ensure!(
                    !Self::is_processed_offchain_tx_active(t2, item.tx_id),
                    Error::<T>::TxAlreadyProcessed
                );
                if check_queued_index {
                    ensure!(
                        !QueuedTxIndex::<T>::contains_key(t2, item.tx_id),
                        Error::<T>::TxAlreadyProcessed
                    );
                }
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
                ensure!(
                    T::InstitutionAssetGuard::can_spend(
                        &item.payer,
                        InstitutionAssetAction::OffchainBatchDebit,
                    ),
                    Error::<T>::ProtectedSource
                );
                let bound = RecipientClearingInstitution::<T>::get(&item.recipient)
                    .ok_or(Error::<T>::RecipientClearingInstitutionNotBound)?;
                ensure!(
                    bound == institution,
                    Error::<T>::RecipientClearingInstitutionMismatch
                );
                if verify_fee {
                    // 中文注释：链下手续费必须和链上制度费率严格一致，不能由中继账户随意填写。
                    let transfer_u128: u128 = item.transfer_amount.saturated_into();
                    let fee_u128: u128 = item.offchain_fee_amount.saturated_into();
                    let expected_fee = calc_offchain_fee_fen(transfer_u128, rate_bp)
                        .map_err(|_| Error::<T>::TransferAmountTooLarge)?;
                    ensure!(fee_u128 == expected_fee, Error::<T>::InvalidFeeAmount);
                    fee_sum_u128 = fee_sum_u128.saturating_add(fee_u128);
                }
            }
            Ok(fee_sum_u128)
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
                QueuedBatchStatus::Pending => {
                    let elapsed: u64 = now.saturating_sub(queued.enqueued_at).saturated_into();
                    if elapsed >= QUEUED_BATCH_RETENTION_BLOCKS {
                        // 中文注释：过期 pending 批次只有在它正好处于队头时才允许推进序号；
                        // 否则会破坏机构内批次必须按 seq 线性推进的约束。
                        let current_seq = LastBatchSeq::<T>::get(queued.institution);
                        let expected_seq = current_seq.saturating_add(1);
                        if queued.batch_seq == expected_seq {
                            LastBatchSeq::<T>::insert(queued.institution, queued.batch_seq);
                        }
                        if let Some(t2) = institution_t2_code(queued.institution) {
                            for item in queued.batch.iter() {
                                QueuedTxIndex::<T>::remove(t2, item.tx_id);
                            }
                        }
                        QueuedBatches::<T>::remove(cursor);
                        Self::deposit_event(Event::<T>::QueuedBatchPruned { queue_id: cursor });
                        let Some(next_cursor) = cursor.checked_add(1) else {
                            return false;
                        };
                        QueuedBatchPruneCursor::<T>::put(next_cursor);
                        return true;
                    }
                    return false;
                }
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

        fn queued_prune_budget_hint(now: BlockNumberFor<T>) -> Option<(u64, u64)> {
            let cursor = QueuedBatchPruneCursor::<T>::get();
            let next = NextQueuedBatchId::<T>::get();
            if cursor >= next {
                return None;
            }
            let Some(queued) = QueuedBatches::<T>::get(cursor) else {
                return Some((3, 1));
            };
            let finalized_at = match queued.status {
                QueuedBatchStatus::Processed => queued.processed_at,
                QueuedBatchStatus::Failed | QueuedBatchStatus::Cancelled => queued.last_attempt_at,
                QueuedBatchStatus::Pending => {
                    let elapsed: u64 = now.saturating_sub(queued.enqueued_at).saturated_into();
                    if elapsed >= QUEUED_BATCH_RETENTION_BLOCKS {
                        return Some((4, queued.batch.len() as u64 + 3));
                    }
                    return None;
                }
            };
            let Some(finalized_at) = finalized_at else {
                return None;
            };
            let elapsed: u64 = now.saturating_sub(finalized_at).saturated_into();
            if elapsed >= QUEUED_BATCH_RETENTION_BLOCKS {
                Some((3, queued.batch.len() as u64 + 2))
            } else {
                None
            }
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

            Self::deposit_event(Event::<T>::InstitutionRateUpdated {
                proposal_id,
                institution: action.institution,
                rate_bp: action.new_rate_bp,
            });
            voting_engine_system::Pallet::<T>::set_status_and_emit(proposal_id, STATUS_EXECUTED)?;
            Ok(())
        }

    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        #[cfg(feature = "std")]
        fn integrity_test() {
            assert!(OFFCHAIN_RATE_BP_MIN <= OFFCHAIN_RATE_BP_MAX);
            assert!(BP_DENOMINATOR > 0);
            assert!(PACK_TX_THRESHOLD > 0);
            assert!(T::MaxBatchSize::get() > 0);
            assert!(MAX_QUEUE_RETRY_COUNT > 0);
            assert!(MAX_STALE_CANCEL_SCAN_STEPS > 0);
            assert!(PROCESSED_TX_RETENTION_BLOCKS > 0);
            assert!(QUEUED_BATCH_RETENTION_BLOCKS > 0);
            assert!(BATCH_SUMMARY_RETENTION_BLOCKS > 0);
        }

        fn on_initialize(_now: BlockNumberFor<T>) -> Weight {
            // 中文注释：移除了 verify key 轮换逻辑，on_initialize 不再需要处理待生效密钥。
            Weight::zero()
        }

        fn on_idle(now: BlockNumberFor<T>, remaining_weight: Weight) -> Weight {
            let db = T::DbWeight::get();
            let processed_budget = db.reads_writes(3, 4);
            let queued_peek_budget = db.reads(3);
            let summary_budget = db.reads_writes(3, 2);
            let processed_idle = db.reads(2);
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

            if remaining_weight.all_gte(consumed.saturating_add(queued_peek_budget)) {
                // 中文注释：queued batch 清理先做轻量 peek，再根据当前游标处对象估算更精确的预算，
                // 这样 on_idle 不会因为一次重清理把剩余权重吃空。
                consumed = consumed.saturating_add(queued_peek_budget);
                if let Some((reads, writes)) = Self::queued_prune_budget_hint(now) {
                    let queued_budget = db.reads_writes(reads, writes);
                    if remaining_weight.all_gte(consumed.saturating_add(queued_budget))
                        && Self::auto_prune_one_queued_batch(now)
                    {
                        consumed = consumed.saturating_add(queued_budget);
                    }
                }
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
    use sp_core::Pair;

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
        pub type OffchainTransactionPos = super;
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
    impl voting_engine_system::SfidEligibility<AccountId32, <Test as frame_system::Config>::Hash>
        for TestSfidEligibility
    {
        fn is_eligible(
            _binding_id: &<Test as frame_system::Config>::Hash,
            _who: &AccountId32,
        ) -> bool {
            true
        }

        fn verify_and_consume_vote_credential(
            _binding_id: &<Test as frame_system::Config>::Hash,
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

    // 中文注释：测试用的额外管理员账户列表（用于 sr25519 密钥对测试）。
    thread_local! {
        static EXTRA_TEST_ADMINS: core::cell::RefCell<Vec<([u8; 32], InstitutionPalletId)>> =
            core::cell::RefCell::new(Vec::new());
    }

    fn add_extra_test_admin(who: &AccountId32, institution: InstitutionPalletId) {
        let who_bytes: [u8; 32] = who.encode()[..32].try_into().expect("32 bytes");
        EXTRA_TEST_ADMINS.with(|admins| {
            admins.borrow_mut().push((who_bytes, institution));
        });
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
                voting_engine_system::internal_vote::ORG_PRB => {
                    let from_china = CHINA_CH
                        .iter()
                        .find(|n| shengbank_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                        .map(|n| n.duoqian_admins.iter().any(|admin| *admin == who_arr))
                        .unwrap_or(false);
                    if from_china {
                        return true;
                    }
                    // 中文注释：检查额外的测试管理员列表。
                    EXTRA_TEST_ADMINS.with(|admins| {
                        admins.borrow().iter().any(|(a, i)| *a == who_arr && *i == institution)
                    })
                },
                _ => false,
            }
        }
    }

    pub struct TestInternalAdminCountProvider;
    impl voting_engine_system::InternalAdminCountProvider for TestInternalAdminCountProvider {
        fn admin_count(org: u8, institution: InstitutionPalletId) -> Option<u32> {
            match org {
                voting_engine_system::internal_vote::ORG_PRB => CHINA_CH
                    .iter()
                    .find(|n| shengbank_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                    .and_then(|n| u32::try_from(n.duoqian_admins.len()).ok()),
                _ => None,
            }
        }
    }

    pub struct TestTimeProvider;
    impl frame_support::traits::UnixTime for TestTimeProvider {
        fn now() -> core::time::Duration {
            core::time::Duration::from_secs(1_782_864_000) // 2026-07-01
        }
    }

    impl voting_engine_system::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type MaxVoteNonceLength = ConstU32<64>;
        type MaxVoteSignatureLength = ConstU32<64>;
        type MaxAutoFinalizePerBlock = ConstU32<64>;
        type MaxProposalsPerExpiry = ConstU32<128>;
        type MaxCleanupStepsPerBlock = ConstU32<8>;
        type CleanupKeysPerStep = ConstU32<64>;
        type MaxProposalDataLen = ConstU32<256>;
        type MaxProposalObjectLen = ConstU32<{ 10 * 1024 }>;
        type SfidEligibility = TestSfidEligibility;
        type PopulationSnapshotVerifier = TestPopulationSnapshotVerifier;
        type JointVoteResultCallback = ();
        type InternalAdminProvider = TestInternalAdminProvider;
        type InternalAdminCountProvider = TestInternalAdminCountProvider;
        type InternalThresholdProvider = ();
        type TimeProvider = TestTimeProvider;
        type WeightInfo = ();
    }

    impl pallet::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type Currency = Balances;
        type MaxBatchSize = ConstU32<8>;
        type MaxBatchSignatureLength = ConstU32<128>;
        type InternalVoteEngine = voting_engine_system::Pallet<Test>;
        type ProtectedSourceChecker = ();
        type InstitutionAssetGuard = ();
        type WeightInfo = ();
    }

    fn prb_institution() -> InstitutionPalletId {
        shengbank_pallet_id_to_bytes(CHINA_CH[0].shenfen_id).expect("valid institution")
    }

    fn prb_admin(index: usize) -> AccountId32 {
        AccountId32::new(CHINA_CH[0].duoqian_admins[index])
    }

    fn prb_account() -> AccountId32 {
        AccountId32::new(institution_pallet_address(prb_institution()).expect("prb account"))
    }

    fn prb_fee_account() -> AccountId32 {
        OffchainTransactionPos::fee_account_of(prb_institution()).expect("prb fee account")
    }

    fn prb_t2() -> [u8; 2] {
        institution_t2_code(prb_institution()).expect("t2")
    }

    fn last_proposal_id() -> u64 {
        voting_engine_system::Pallet::<Test>::next_proposal_id().saturating_sub(1)
    }

    /// 中文注释：生成测试用 sr25519 密钥对，返回 (AccountId, Pair)。
    /// AccountId 由 sr25519 公钥字节构造，可直接用于签名验证。
    fn test_sr25519_admin(seed: &[u8; 32]) -> (AccountId32, sp_core::sr25519::Pair) {
        use sp_core::Pair;
        let pair = sp_core::sr25519::Pair::from_seed(seed);
        let pub_key = pair.public();
        let account = AccountId32::new(pub_key.0);
        (account, pair)
    }

    /// 中文注释：用指定的 sr25519 密钥对签名批次消息。
    fn sign_batch(
        pair: &sp_core::sr25519::Pair,
        institution: InstitutionPalletId,
        batch_seq: u64,
        batch: &BatchOf<Test>,
    ) -> BatchSignatureOf<Test> {
        use sp_core::Pair;
        let message = OffchainTransactionPos::batch_signing_message_for_test(institution, batch_seq, batch);
        let sig = pair.sign(message.as_slice());
        sig.0.to_vec().try_into().expect("signature fits")
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
                (prb_admin(0), 1_000),
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
        assert_eq!(calc_offchain_fee_fen(100, 1), Ok(1));
        assert_eq!(calc_offchain_fee_fen(150, 1), Ok(1));
        assert_eq!(calc_offchain_fee_fen(151, 1), Ok(1));
        assert_eq!(calc_offchain_fee_fen(1, 1), Ok(1));
    }

    #[test]
    fn calc_fee_overflow_returns_err() {
        assert_eq!(
            calc_offchain_fee_fen(u128::MAX, OFFCHAIN_RATE_BP_MAX),
            Err(FeeCalcError::AmountOverflow)
        );
    }

    #[test]
    fn institution_t2_code_uses_r5_prefix() {
        let t2 = institution_t2_code(prb_institution()).expect("t2");
        assert_eq!(t2, *b"ZS");
    }

    #[test]
    fn rate_update_requires_internal_vote_pass() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            assert_ok!(OffchainTransactionPos::propose_institution_rate(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                5
            ));
            let pid = last_proposal_id();
            for i in 0..6 {
                assert_ok!(OffchainTransactionPos::vote_institution_rate(
                    RuntimeOrigin::signed(prb_admin(i)),
                    pid,
                    true
                ));
            }
            assert_eq!(OffchainTransactionPos::rate_bp_of(institution), 5);
        });
    }

    #[test]
    fn non_admin_cannot_propose_or_submit() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            let non_admin = AccountId32::new([9u8; 32]);
            assert_noop!(
                OffchainTransactionPos::propose_institution_rate(
                    RuntimeOrigin::signed(non_admin.clone()),
                    institution,
                    3
                ),
                Error::<Test>::UnauthorizedAdmin
            );
            // 中文注释：非管理员提交批次应被拒绝。
            let (admin_account, admin_pair) = test_sr25519_admin(&[42u8; 32]);
            let batch: BatchOf<Test> = vec![BatchItemOf::<Test> {
                tx_id: <Test as frame_system::Config>::Hashing::hash(b"bad-tx"),
                payer: AccountId32::new([1u8; 32]),
                recipient: AccountId32::new([3u8; 32]),
                transfer_amount: 100,
                offchain_fee_amount: 1,
            }]
            .try_into()
            .expect("fit");
            let sig = sign_batch(&admin_pair, institution, 1, &batch);
            assert_noop!(
                OffchainTransactionPos::submit_offchain_batch(
                    RuntimeOrigin::signed(admin_account),
                    institution,
                    1,
                    batch,
                    sig,
                ),
                Error::<Test>::UnauthorizedAdmin
            );
        });
    }

    #[test]
    fn submit_batch_executes_real_settlement_and_marks_processed() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            // 中文注释：创建 sr25519 管理员账户并注册为额外管理员。
            let (admin_account, admin_pair) = test_sr25519_admin(&[42u8; 32]);
            add_extra_test_admin(&admin_account, institution);
            let _ = Balances::deposit_creating(&admin_account, 1_000);

            assert_ok!(OffchainTransactionPos::propose_institution_rate(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                1
            ));
            let pid = last_proposal_id();
            for i in 0..6 {
                assert_ok!(OffchainTransactionPos::vote_institution_rate(
                    RuntimeOrigin::signed(prb_admin(i)),
                    pid,
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
            let fee_account = OffchainTransactionPos::fee_account_of(institution).expect("fee");
            let fee_before = Balances::free_balance(&fee_account);
            assert_ok!(OffchainTransactionPos::bind_clearing_institution(
                RuntimeOrigin::signed(recipient1),
                institution
            ));
            assert_ok!(OffchainTransactionPos::bind_clearing_institution(
                RuntimeOrigin::signed(recipient2),
                institution
            ));

            let sig = sign_batch(&admin_pair, institution, 1, &batch);
            assert_ok!(OffchainTransactionPos::submit_offchain_batch(
                RuntimeOrigin::signed(admin_account.clone()),
                institution,
                1,
                batch,
                sig,
            ));

            assert_eq!(Balances::free_balance(&item1.recipient), 1_001);
            assert_eq!(Balances::free_balance(&item2.recipient), 2_001);
            assert_eq!(Balances::free_balance(&fee_account), fee_before + 2);
            let t2 = prb_t2();
            assert!(ProcessedOffchainTx::<Test>::get(t2, item1.tx_id));
            assert!(ProcessedOffchainTx::<Test>::get(t2, item2.tx_id));

            // 中文注释：重放应被拒绝。
            System::set_block_number(System::block_number() + PACK_BLOCK_THRESHOLD as u64);
            let replay: BatchOf<Test> = vec![item1].try_into().expect("fit");
            let sig2 = sign_batch(&admin_pair, institution, 2, &replay);
            assert_noop!(
                OffchainTransactionPos::submit_offchain_batch(
                    RuntimeOrigin::signed(admin_account),
                    institution,
                    2,
                    replay,
                    sig2,
                ),
                Error::<Test>::TxAlreadyProcessed
            );
        });
    }

    #[test]
    fn submit_batch_requires_valid_signature() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            let (admin_account, _admin_pair) = test_sr25519_admin(&[42u8; 32]);
            add_extra_test_admin(&admin_account, institution);
            let _ = Balances::deposit_creating(&admin_account, 1_000);
            let recipient = AccountId32::new([3u8; 32]);
            assert_ok!(OffchainTransactionPos::bind_clearing_institution(
                RuntimeOrigin::signed(recipient.clone()),
                institution
            ));

            let batch: BatchOf<Test> = vec![BatchItemOf::<Test> {
                tx_id: <Test as frame_system::Config>::Hashing::hash(b"sig-check"),
                payer: AccountId32::new([1u8; 32]),
                recipient,
                transfer_amount: 100,
                offchain_fee_amount: 1,
            }]
            .try_into()
            .expect("fit");

            // 中文注释：使用错误签名应被拒绝。
            assert_noop!(
                OffchainTransactionPos::submit_offchain_batch(
                    RuntimeOrigin::signed(admin_account),
                    institution,
                    1,
                    batch,
                    b"bad-signature-not-valid-sr25519".to_vec().try_into().expect("fit"),
                ),
                Error::<Test>::InvalidBatchSignature
            );
        });
    }

    #[test]
    fn prune_expired_proposal_action_removes_rejected_action() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            assert_ok!(OffchainTransactionPos::propose_institution_rate(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                5
            ));
            let pid = last_proposal_id();
            let proposal = voting_engine_system::Pallet::<Test>::proposals(pid).expect("proposal");
            System::set_block_number(proposal.end.saturating_add(1));
            assert!(RateProposalActions::<Test>::contains_key(pid));
            assert_ok!(OffchainTransactionPos::prune_expired_proposal_action(
                RuntimeOrigin::signed(prb_admin(0)),
                pid
            ));
            assert!(!RateProposalActions::<Test>::contains_key(pid));
        });
    }

    #[test]
    fn recipient_bind_clearing_institution_switch_freely() {
        new_test_ext().execute_with(|| {
            let recipient = AccountId32::new([3u8; 32]);
            let inst_1 = prb_institution();
            let inst_2 =
                shengbank_pallet_id_to_bytes(CHINA_CH[1].shenfen_id).expect("valid institution");
            assert_ok!(OffchainTransactionPos::bind_clearing_institution(
                RuntimeOrigin::signed(recipient.clone()),
                inst_1
            ));
            assert_eq!(
                OffchainTransactionPos::recipient_clearing_institution(recipient.clone()),
                Some(inst_1)
            );
            assert_ok!(OffchainTransactionPos::bind_clearing_institution(
                RuntimeOrigin::signed(recipient.clone()),
                inst_2
            ));
            assert_eq!(
                OffchainTransactionPos::recipient_clearing_institution(recipient),
                Some(inst_2)
            );
        });
    }

    #[test]
    fn enqueue_offchain_batch_validates_admin_and_signature() {
        new_test_ext().execute_with(|| {
            let institution = prb_institution();
            let (admin_account, admin_pair) = test_sr25519_admin(&[42u8; 32]);
            add_extra_test_admin(&admin_account, institution);
            let _ = Balances::deposit_creating(&admin_account, 1_000);

            assert_ok!(OffchainTransactionPos::propose_institution_rate(
                RuntimeOrigin::signed(prb_admin(0)),
                institution,
                1
            ));
            let pid = last_proposal_id();
            for i in 0..6 {
                assert_ok!(OffchainTransactionPos::vote_institution_rate(
                    RuntimeOrigin::signed(prb_admin(i)),
                    pid,
                    true
                ));
            }
            let payer = AccountId32::new([1u8; 32]);
            let recipient = AccountId32::new([3u8; 32]);
            assert_ok!(OffchainTransactionPos::bind_clearing_institution(
                RuntimeOrigin::signed(recipient.clone()),
                institution
            ));

            let tx_id = <Test as frame_system::Config>::Hashing::hash(b"enqueue-test");
            let batch: BatchOf<Test> = vec![BatchItemOf::<Test> {
                tx_id,
                payer,
                recipient,
                transfer_amount: 100,
                offchain_fee_amount: 1,
            }]
            .try_into()
            .expect("fit");
            let sig = sign_batch(&admin_pair, institution, 1, &batch);

            assert_ok!(OffchainTransactionPos::enqueue_offchain_batch(
                RuntimeOrigin::signed(admin_account),
                institution,
                1,
                batch,
                sig,
            ));
            assert_eq!(OffchainTransactionPos::next_queued_batch_id(), 1);
        });
    }


    #[test]
    fn prune_processed_tx_without_timestamp_is_allowed() {
        new_test_ext().execute_with(|| {
            let tx_id = <Test as frame_system::Config>::Hashing::hash(b"legacy-no-ts");
            let t2 = prb_t2();
            ProcessedOffchainTx::<Test>::insert(t2, tx_id, true);
            ProcessedOffchainTxAt::<Test>::remove(t2, tx_id);
            assert_ok!(OffchainTransactionPos::prune_processed_tx(
                RuntimeOrigin::signed(prb_admin(0)),
                t2,
                tx_id
            ));
            assert!(!ProcessedOffchainTx::<Test>::get(t2, tx_id));
        });
    }
}
