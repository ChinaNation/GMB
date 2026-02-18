#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    ensure, pallet_prelude::*, traits::Currency, Blake2_128Concat, PalletId,
};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::traits::{AccountIdConversion, SaturatedConversion, Saturating, Zero};

use primitives::shengbank_nodes_const::{
    fee_pallet_id_to_bytes as shengbank_fee_pallet_id_to_bytes,
    pallet_id_to_bytes as shengbank_pallet_id_to_bytes, SHENG_BANK_NODES,
};
use voting_engine_system::{
    internal_vote::ORG_PRB, InstitutionPalletId, InternalVoteEngine, PROPOSAL_KIND_INTERNAL,
    STATUS_PASSED,
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
const BP_DENOMINATOR: u128 = 10_000;

fn institution_pallet_address(institution: InstitutionPalletId) -> Option<[u8; 32]> {
    SHENG_BANK_NODES
        .iter()
        .find(|n| shengbank_pallet_id_to_bytes(n.pallet_id) == Some(institution))
        .map(|n| n.pallet_address)
}

fn institution_fee_pallet_id(institution: InstitutionPalletId) -> Option<[u8; 8]> {
    SHENG_BANK_NODES
        .iter()
        .find(|n| shengbank_pallet_id_to_bytes(n.pallet_id) == Some(institution))
        .and_then(|n| shengbank_fee_pallet_id_to_bytes(n.fee_pallet_id))
}

fn round_div(numerator: u128, denominator: u128) -> u128 {
    numerator
        .saturating_add(denominator / 2)
        .saturating_div(denominator)
}

fn calc_offchain_fee_fen(amount_fen: u128, rate_bp: u32) -> u128 {
    let by_rate = round_div(amount_fen.saturating_mul(rate_bp as u128), BP_DENOMINATOR);
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

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct RateProposalAction {
    pub institution: InstitutionPalletId,
    pub new_rate_bp: u32,
    pub executed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct VerifyKeyProposalAction<BoundedBytes> {
    pub institution: InstitutionPalletId,
    pub new_key: BoundedBytes,
    pub executed: bool,
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
    pub executed: bool,
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
    pub type LastBatchSeq<T> = StorageMap<_, Blake2_128Concat, InstitutionPalletId, u64, ValueQuery>;

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

    /// 内部投票提案防重放（同一 proposal_id 仅能执行一次）。
    #[pallet::storage]
    pub type UsedInternalProposal<T> = StorageMap<_, Blake2_128Concat, u64, bool, ValueQuery>;

    /// 已处理链下 tx_id 防重放。
    #[pallet::storage]
    pub type ProcessedOffchainTx<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, bool, ValueQuery>;

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
    }

    #[pallet::error]
    pub enum Error<T> {
        InvalidInstitution,
        InvalidRateBp,
        InvalidFeeAmount,
        InstitutionAccountDecodeFailed,
        ProposalNotFound,
        ProposalKindMismatch,
        ProposalStatusNotPassed,
        ProposalInstitutionMismatch,
        ProposalAlreadyUsed,
        RateProposalNotFound,
        RateProposalAlreadyExecuted,
        VerifyKeyProposalNotFound,
        VerifyKeyProposalAlreadyExecuted,
        SweepProposalNotFound,
        SweepProposalAlreadyExecuted,
        UnauthorizedAdmin,
        UnauthorizedSubmitter,
        TxAlreadyProcessed,
        PackThresholdNotReached,
        EmptyBatch,
        VerifyKeyAlreadyInitialized,
        VerifyKeyMissing,
        InvalidBatchSignature,
        InsufficientFeeReserve,
        SweepAmountExceedsCap,
        RelaySubmittersNotInitialized,
        RelaySubmitterNotAllowed,
        RelaySubmittersAlreadyInitialized,
        InvalidRelaySubmittersCount,
        InvalidBatchSeq,
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
        #[pallet::weight(T::DbWeight::get().reads_writes(8, 8 + T::MaxBatchSize::get() as u64))]
        pub fn submit_offchain_batch(
            origin: OriginFor<T>,
            institution: InstitutionPalletId,
            batch_seq: u64,
            batch: BatchOf<T>,
            batch_signature: BatchSignatureOf<T>,
        ) -> DispatchResult {
            let submitter = ensure_signed(origin)?;
            Self::precheck_submit_offchain_batch(
                &submitter,
                institution,
                batch_seq,
                &batch,
                &batch_signature,
            )?;
            let fee_account = Self::institution_fee_account(institution);
            let verify_key = Self::verify_key_for(institution).ok_or(Error::<T>::VerifyKeyMissing)?;
            let now = frame_system::Pallet::<T>::block_number();
            let last = LastPackBlock::<T>::get(institution);
            let elapsed: u32 =
                <BlockNumberFor<T> as sp_runtime::traits::Saturating>::saturating_sub(now, last)
                    .saturated_into();
            let by_count = (batch.len() as u64) >= PACK_TX_THRESHOLD;
            let by_time = last.is_zero() || elapsed >= PACK_BLOCK_THRESHOLD;

            let mut total_transfer_u128: u128 = 0;
            let mut total_fee_u128: u128 = 0;

            for item in batch.iter() {
                let transfer_u128: u128 = item.transfer_amount.saturated_into();
                let fee_u128: u128 = item.offchain_fee_amount.saturated_into();

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

                total_transfer_u128 = total_transfer_u128.saturating_add(transfer_u128);
                total_fee_u128 = total_fee_u128.saturating_add(fee_u128);
                ProcessedOffchainTx::<T>::insert(item.tx_id, true);
            }

            let batch_id = NextBatchId::<T>::get();
            NextBatchId::<T>::put(batch_id.saturating_add(1));
            LastPackBlock::<T>::insert(institution, now);
            LastBatchSeq::<T>::insert(institution, batch_seq);

            let total_transfer_amount: BalanceOf<T> = total_transfer_u128.saturated_into();
            let total_offchain_fee_amount: BalanceOf<T> = total_fee_u128.saturated_into();
            let batch_hash =
                sp_io::hashing::blake2_256(&(institution, batch_seq, &batch).encode());
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
                submitter,
                batch_seq,
                batch_hash,
                signer_key_hash,
                item_count: batch.len() as u32,
                total_transfer_amount,
                total_offchain_fee_amount,
                reason_by_count: by_count,
                reason_by_time: by_time,
            });
            Ok(())
        }

        /// 省储行安装时初始化默认验证密钥，仅可初始化一次。
        /// 该初始化由机构主账户（pallet_address）执行；
        /// 后续更换必须走内部投票流程（propose_verify_key/vote_verify_key）。
        #[pallet::call_index(5)]
        #[pallet::weight(T::DbWeight::get().reads_writes(2, 1))]
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
            VerifyKeys::<T>::insert(institution, &default_key);
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
            ensure!(who == institution_account, Error::<T>::UnauthorizedSubmitter);
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
            RelaySubmitters::<T>::insert(institution, &submitters);
            Self::deposit_event(Event::<T>::RelaySubmittersInitialized {
                institution,
                count: submitter_count,
            });
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
                    executed: false,
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
            ensure!(!action.executed, Error::<T>::RateProposalAlreadyExecuted);
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
                        Self::try_execute_rate(proposal_id)?;
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
                    executed: false,
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
                !action.executed,
                Error::<T>::VerifyKeyProposalAlreadyExecuted
            );
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
                        Self::try_execute_verify_key(proposal_id)?;
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
                    executed: false,
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
            ensure!(!action.executed, Error::<T>::SweepProposalAlreadyExecuted);
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
                        Self::try_execute_sweep(proposal_id)?;
                    }
                }
            }
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
            ensure!(!batch.is_empty(), Error::<T>::EmptyBatch);
            let relay_submitters = RelaySubmitters::<T>::get(institution)
                .ok_or(Error::<T>::RelaySubmittersNotInitialized)?;
            ensure!(
                relay_submitters.iter().any(|acc| acc == submitter),
                Error::<T>::RelaySubmitterNotAllowed
            );
            let expected_seq = LastBatchSeq::<T>::get(institution).saturating_add(1);
            ensure!(batch_seq == expected_seq, Error::<T>::InvalidBatchSeq);
            let rate_bp = Self::ensure_rate_and_institution(institution)?;
            let verify_key = Self::verify_key_for(institution).ok_or(Error::<T>::VerifyKeyMissing)?;
            let message = Self::batch_signing_message(institution, batch_seq, batch);
            ensure!(
                T::OffchainBatchVerifier::verify(
                    verify_key.as_slice(),
                    message.as_slice(),
                    batch_signature.as_slice()
                ),
                Error::<T>::InvalidBatchSignature
            );

            let now = frame_system::Pallet::<T>::block_number();
            let last = LastPackBlock::<T>::get(institution);
            let elapsed: u32 =
                <BlockNumberFor<T> as sp_runtime::traits::Saturating>::saturating_sub(now, last)
                    .saturated_into();
            let by_count = (batch.len() as u64) >= PACK_TX_THRESHOLD;
            let by_time = last.is_zero() || elapsed >= PACK_BLOCK_THRESHOLD;
            ensure!(by_count || by_time, Error::<T>::PackThresholdNotReached);

            for item in batch.iter() {
                ensure!(
                    !ProcessedOffchainTx::<T>::get(item.tx_id),
                    Error::<T>::TxAlreadyProcessed
                );
                let transfer_u128: u128 = item.transfer_amount.saturated_into();
                let fee_u128: u128 = item.offchain_fee_amount.saturated_into();
                let expected_fee = calc_offchain_fee_fen(transfer_u128, rate_bp);
                ensure!(fee_u128 == expected_fee, Error::<T>::InvalidFeeAmount);
            }
            Ok(())
        }

        pub fn fee_account_of(institution: InstitutionPalletId) -> Result<T::AccountId, DispatchError> {
            ensure!(
                institution_pallet_address(institution).is_some(),
                Error::<T>::InvalidInstitution
            );
            // 中文注释：fee_account_of 仅暴露地址查询，不做任何资产转移。
            Ok(Self::institution_fee_account(institution))
        }

        /// 返回当前有效验证密钥：
        /// - 若 pending key 已到 activate_at，则优先使用 pending key；
        /// - 否则使用 current key。
        pub fn verify_key_for(institution: InstitutionPalletId) -> Option<VerifyKeyOf<T>> {
            let now = frame_system::Pallet::<T>::block_number();
            if let Some(pending) = PendingVerifyKeys::<T>::get(institution) {
                if now >= pending.activate_at {
                    return Some(pending.key);
                }
            }
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

        fn institution_fee_account(institution: InstitutionPalletId) -> T::AccountId {
            let fee_pid = institution_fee_pallet_id(institution).expect("valid institution checked");
            // 中文注释：fee_pallet_address 直接由 fee_pallet_id 派生，是独立手续费账户。
            PalletId(fee_pid).into_account_truncating()
        }

        fn institution_account(
            institution: InstitutionPalletId,
        ) -> Result<T::AccountId, DispatchError> {
            let raw =
                institution_pallet_address(institution).ok_or(Error::<T>::InvalidInstitution)?;
            T::AccountId::decode(&mut &raw[..])
                .map_err(|_| Error::<T>::InstitutionAccountDecodeFailed.into())
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
            if <T as voting_engine_system::Config>::InternalAdminProvider::is_internal_admin(
                ORG_PRB,
                institution,
                who,
            ) {
                return true;
            }

            let who_bytes = who.encode();
            if who_bytes.len() != 32 {
                return false;
            }
            let mut who_arr = [0u8; 32];
            who_arr.copy_from_slice(&who_bytes);

            SHENG_BANK_NODES
                .iter()
                .find(|n| shengbank_pallet_id_to_bytes(n.pallet_id) == Some(institution))
                .map(|n| n.admins.iter().any(|admin| *admin == who_arr))
                .unwrap_or(false)
        }

        fn try_execute_rate(proposal_id: u64) -> DispatchResult {
            let action = RateProposalActions::<T>::get(proposal_id)
                .ok_or(Error::<T>::RateProposalNotFound)?;
            ensure!(!action.executed, Error::<T>::RateProposalAlreadyExecuted);

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
            RateProposalActions::<T>::mutate(proposal_id, |maybe| {
                if let Some(inner) = maybe {
                    inner.executed = true;
                }
            });
            UsedInternalProposal::<T>::insert(proposal_id, true);

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
            ensure!(
                !action.executed,
                Error::<T>::VerifyKeyProposalAlreadyExecuted
            );

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
                PendingVerifyKeys::<T>::insert(
                    action.institution,
                    PendingVerifyKey {
                        key: action.new_key.clone(),
                        activate_at,
                    },
                );
                VerifyKeyRotationStatuses::<T>::insert(
                    action.institution,
                    VerifyKeyRotationStatus {
                        stage: VerifyKeyRotationStage::Scheduled,
                        activate_at: Some(activate_at),
                    },
                );
            } else {
                VerifyKeys::<T>::insert(action.institution, &action.new_key);
                VerifyKeyRotationStatuses::<T>::insert(
                    action.institution,
                    VerifyKeyRotationStatus {
                        stage: VerifyKeyRotationStage::Idle,
                        activate_at: None,
                    },
                );
            }
            VerifyKeyProposalActions::<T>::mutate(proposal_id, |maybe| {
                if let Some(inner) = maybe {
                    inner.executed = true;
                }
            });
            UsedInternalProposal::<T>::insert(proposal_id, true);

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
            ensure!(!action.executed, Error::<T>::SweepProposalAlreadyExecuted);

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

            let fee_account = Self::institution_fee_account(action.institution);
            let main_account = Self::institution_account(action.institution)?;

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

            let reserve_left_u128 = fee_balance_u128.saturating_sub(amount_u128);
            let reserve_left: BalanceOf<T> = reserve_left_u128.saturated_into();

            SweepProposalActions::<T>::mutate(proposal_id, |maybe| {
                if let Some(inner) = maybe {
                    inner.executed = true;
                }
            });
            UsedInternalProposal::<T>::insert(proposal_id, true);

            Self::deposit_event(Event::<T>::SweepToMainExecuted {
                proposal_id,
                institution: action.institution,
                amount: action.amount,
                reserve_left,
            });
            Ok(())
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(now: BlockNumberFor<T>) -> Weight {
            let mut reads: u64 = 0;
            let mut writes: u64 = 0;
            for node in SHENG_BANK_NODES.iter() {
                let Some(institution) = shengbank_pallet_id_to_bytes(node.pallet_id) else {
                    continue;
                };
                reads = reads.saturating_add(1);
                if let Some(pending) = PendingVerifyKeys::<T>::get(institution) {
                    if now >= pending.activate_at {
                        let key_len = pending.key.len() as u32;
                        VerifyKeys::<T>::insert(institution, pending.key);
                        PendingVerifyKeys::<T>::remove(institution);
                        VerifyKeyRotationStatuses::<T>::insert(
                            institution,
                            VerifyKeyRotationStatus {
                                stage: VerifyKeyRotationStage::Idle,
                                activate_at: None,
                            },
                        );
                        Self::deposit_event(Event::<T>::VerifyKeyRotated {
                            institution,
                            key_len,
                            activated_at: now,
                        });
                        writes = writes.saturating_add(3);
                    }
                }
            }
            T::DbWeight::get().reads_writes(reads, writes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{assert_noop, assert_ok, derive_impl, traits::ConstU32};
    use frame_system as system;
    use sp_runtime::{traits::Hash as HashT, traits::IdentityLookup, AccountId32, BuildStorage, TokenError};

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

    pub struct TestCiicEligibility;
    impl voting_engine_system::CiicEligibility<AccountId32> for TestCiicEligibility {
        fn is_eligible(_ciic: &[u8], _who: &AccountId32) -> bool {
            true
        }

        fn verify_and_consume_vote_credential(
            _ciic: &[u8],
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

    impl voting_engine_system::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type MaxCiicLength = ConstU32<64>;
        type MaxVoteNonceLength = ConstU32<64>;
        type MaxVoteSignatureLength = ConstU32<64>;
        type CiicEligibility = TestCiicEligibility;
        type PopulationSnapshotVerifier = TestPopulationSnapshotVerifier;
        type JointVoteResultCallback = ();
        type InternalAdminProvider = ();
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
    }

    fn prb_institution() -> InstitutionPalletId {
        shengbank_pallet_id_to_bytes(SHENG_BANK_NODES[0].pallet_id).expect("valid institution")
    }

    fn prb_admin(index: usize) -> AccountId32 {
        AccountId32::new(SHENG_BANK_NODES[0].admins[index])
    }

    fn prb_account() -> AccountId32 {
        AccountId32::new(institution_pallet_address(prb_institution()).expect("prb account"))
    }

    fn prb_fee_account() -> AccountId32 {
        OffchainTransactionFee::fee_account_of(prb_institution()).expect("prb fee account")
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
                recipient: recipient1,
                transfer_amount: 1_000,
                offchain_fee_amount: 1,
            };
            let item2 = BatchItemOf::<Test> {
                tx_id: <Test as frame_system::Config>::Hashing::hash(b"tx-2"),
                payer: payer2,
                recipient: recipient2,
                transfer_amount: 2_000,
                offchain_fee_amount: 1,
            };
            let batch: BatchOf<Test> = vec![item1.clone(), item2.clone()].try_into().expect("fit");
            let fee_account = OffchainTransactionFee::fee_account_of(institution).expect("fee");
            let fee_before = Balances::free_balance(&fee_account);
            assert_ok!(OffchainTransactionFee::init_verify_key(
                RuntimeOrigin::signed(prb_account()),
                institution,
                b"default-key".to_vec().try_into().expect("fit")
            ));
            let relays: RelaySubmittersOf<Test> =
                vec![relay_account(), prb_admin(0), prb_admin(1)].try_into().expect("fit");
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
            assert!(ProcessedOffchainTx::<Test>::get(item1.tx_id));
            assert!(ProcessedOffchainTx::<Test>::get(item2.tx_id));

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
            assert_eq!(OffchainTransactionFee::verify_key_for(institution), Some(old_key.clone()));

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
            assert_eq!(OffchainTransactionFee::verify_key_for(institution), Some(old_key.clone()));

            // 到达生效高度后切换为新密钥。
            System::set_block_number(
                System::block_number() + VERIFY_KEY_ROTATION_DELAY_BLOCKS as u64,
            );
            OffchainTransactionFee::on_initialize(System::block_number());
            assert_eq!(OffchainTransactionFee::verify_key_for(institution), Some(new_key));
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
            assert_noop!(
                OffchainTransactionFee::vote_sweep_to_main(
                    RuntimeOrigin::signed(prb_admin(5)),
                    1,
                    true
                ),
                Error::<Test>::InsufficientFeeReserve
            );
            assert_eq!(Balances::free_balance(&fee_account), fee_before + 200_000);
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
            let relays: RelaySubmittersOf<Test> = vec![
                relay_account(),
                prb_admin(0),
                prb_admin(1),
            ]
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
            let relays: RelaySubmittersOf<Test> =
                vec![relay_account(), prb_admin(0), prb_admin(1)].try_into().expect("fit");
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
            assert!(!ProcessedOffchainTx::<Test>::get(tx_id));

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
            assert!(ProcessedOffchainTx::<Test>::get(tx_id));
        });
    }
}
