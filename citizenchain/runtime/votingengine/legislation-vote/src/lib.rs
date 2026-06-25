//! # 立法投票 pallet (legislation-vote)
//!
//! 立法机构专属投票模式(ADR-027,公民宪法第四十四/四十五条)。投票引擎「头等模式」:
//! `PROPOSAL_KIND_LEGISLATION`,共享核心 `votingengine`(Proposals/AdminSnapshot/状态机/
//! 公投快照验签/清理/反向索引),只本地保管计票账本。三个既有投票 sub-pallet
//! (internal/joint/citizen)逻辑零改动。
//!
//! 阶段(ADR-027 修订 2026-06-25,5 类提案删二审 + 行政签署/三人会签):
//! - `STAGE_LEG_HOUSE` 内部表决:单院(市立法会)一段;两院(国家/省立法院 众→参;教委会→参议会)顺序两段。
//! - `STAGE_LEG_REFERENDUM` 强制公投:仅特别案(含核心修宪),内部全过后强制进入,公投通过即生效不签署。
//! - `STAGE_LEG_SIGN` 行政签署:非特别案内部全过后,行政机构法定代表人(市长/省长/总统)签署。
//!   市级无救济(否决=否决/30天超时=通过);省国级否决或超时 → 会签。
//! - `STAGE_LEG_OVERRIDE` 三人会签(省/国家级):立法院院长 + 参议长 + 众议长,全签=生效/任一否决或超时=否决。
//!
//! 计票口径:按现任议员/委员管理员快照总数算参与率/赞成率/反对率(`votingengine::types`
//! 的立法阈值纯函数),投票期满 finalize 统一判定(全员已投或反对超限可提前决)。

#![cfg_attr(not(feature = "std"), no_std)]

pub mod weights;

#[cfg(test)]
mod tests;

pub use pallet::*;

use frame_support::{
    ensure,
    pallet_prelude::DispatchResult,
    storage::{with_transaction, TransactionOutcome},
};
use sp_runtime::traits::{One, Saturating};
use sp_runtime::DispatchError;

use primitives::count_const::VOTING_DURATION_BLOCKS;

use votingengine::{
    pallet::{Proposals, ProposalsByExpiry},
    types::{
        legislation_house_decided, legislation_house_final_passed,
        legislation_referendum_final_passed, InstitutionCode, LEG_VOTE_SPECIAL,
    },
    CidEligibility, InternalAdminProvider, InternalProposalMutexKind, PopulationSnapshotVerifier,
    Proposal, PROPOSAL_KIND_LEGISLATION, STAGE_LEG_HOUSE, STAGE_LEG_OVERRIDE, STAGE_LEG_REFERENDUM,
    STAGE_LEG_SIGN, STATUS_PASSED, STATUS_REJECTED, STATUS_VOTING,
};

/// 法律全文大对象类型标记(写入 votingengine `ProposalObject`),与 legislation-yuan 对齐。
pub const PROPOSAL_OBJECT_KIND_LAW_TEXT: u8 = 2;

/// 单部法律最多院数,单一真源在 `votingengine::types::MAX_LEGISLATION_HOUSES`。
pub const MAX_HOUSES: u32 = votingengine::types::MAX_LEGISLATION_HOUSES;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    /// pallet 自身 StorageVersion(全新创世口径,恒 v1)。
    pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    /// 院序列别名:`[(机构码, 机构账户), ...]`,发起院在前、终审院在后。
    pub type HousesOf<T> =
        BoundedVec<(InstitutionCode, <T as frame_system::Config>::AccountId), ConstU32<MAX_HOUSES>>;

    /// 立法提案元数据(核心 `Proposal` 装不下的立法专属部分)。
    #[derive(
        Encode,
        Decode,
        DecodeWithMemTracking,
        CloneNoBound,
        PartialEqNoBound,
        EqNoBound,
        RuntimeDebugNoBound,
        TypeInfo,
        MaxEncodedLen,
    )]
    #[scale_info(skip_type_params(T))]
    pub struct LegislationMeta<T: Config> {
        /// 表决类型(常规 0 / 常规教育 1 / 重要 2 / 重要教育 3 / 特别 4,ADR-027 修订)
        pub vote_type: u8,
        /// 院序列
        pub houses: HousesOf<T>,
        /// 当前正在表决的院索引(单院恒 0;两院 0→1)
        pub current_house: u32,
        /// 是否需要强制公投(= 特别案)
        pub referendum_required: bool,
        /// 行政签署机构(总统府/省政府/市政府);其法定代表人=总统/省长/市长。非特别案末段签署。
        pub executive: (InstitutionCode, T::AccountId),
        /// 两院级的立法院机构(国家/省立法院);其法定代表人=院长,供三人会签。单院(市)=None。
        pub legislature: Option<(InstitutionCode, T::AccountId)>,
    }

    /// 已准备的人口快照(特别案公投分母),对标 joint-vote。
    #[derive(
        Encode,
        Decode,
        DecodeWithMemTracking,
        Clone,
        PartialEq,
        Eq,
        RuntimeDebug,
        TypeInfo,
        MaxEncodedLen,
    )]
    pub struct PreparedSnapshot<BlockNumber, Hash> {
        pub eligible_total: u64,
        pub nonce_hash: Hash,
        pub prepared_at: BlockNumber,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config + votingengine::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// 立法提案元数据:proposal_id → LegislationMeta。
    #[pallet::storage]
    pub type LegMeta<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, LegislationMeta<T>, OptionQuery>;

    /// 当前院内部表决计票(院推进时重置)。
    #[pallet::storage]
    pub type LegHouseTally<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, votingengine::VoteCountU32, ValueQuery>;

    /// 内部表决去重:(proposal_id, 议员/委员) → 赞成/反对。两院议员账户互不重叠。
    #[pallet::storage]
    pub type LegHouseVotesByAdmin<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        T::AccountId,
        bool,
        OptionQuery,
    >;

    /// 公投计票(特别案)。
    #[pallet::storage]
    pub type LegReferendumTally<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, votingengine::VoteCountU64, ValueQuery>;

    /// 公投去重:(proposal_id, binding_id) → 赞成/反对。
    #[pallet::storage]
    pub type LegReferendumVotesByBindingId<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, u64, Blake2_128Concat, T::Hash, bool, OptionQuery>;

    /// 三人会签记录(省/国家级 STAGE_LEG_OVERRIDE):proposal_id → [(签署人, 是否赞成)],
    /// 去重 + 集齐 3 个不同身份赞成判通过。签署人 ∈ {院长, 参议长, 众议长} 法定代表人。
    #[pallet::storage]
    pub type LegOverrideSigns<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        BoundedVec<(T::AccountId, bool), ConstU32<3>>,
        ValueQuery,
    >;

    /// 人口快照 nonce 永久去重(防重放)。
    #[pallet::storage]
    pub type UsedSnapshotNonce<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, bool, ValueQuery>;

    /// 待消费的人口快照:发起人 → 已验签缓存(特别案发起前一区块准备)。
    #[pallet::storage]
    pub type PendingPopulationSnapshots<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        PreparedSnapshot<BlockNumberFor<T>, T::Hash>,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 立法提案已创建(进入第一院内部表决)。
        LegislationProposalCreated {
            proposal_id: u64,
            vote_type: u8,
            houses: u32,
        },
        /// 某议员/委员投出一票。
        LegislationHouseVoteCast {
            proposal_id: u64,
            who: T::AccountId,
            approve: bool,
        },
        /// 某院通过,推进至下一院。
        LegislationHouseAdvanced { proposal_id: u64, next_house: u32 },
        /// 内部全过,推进至强制公投阶段。
        LegislationAdvancedToReferendum {
            proposal_id: u64,
            eligible_total: u64,
        },
        /// 一张公投票已投出。
        LegislationReferendumVoteCast {
            proposal_id: u64,
            who: T::AccountId,
            approve: bool,
        },
        /// 人口快照已准备(特别案公投分母)。
        PopulationSnapshotPrepared {
            who: T::AccountId,
            eligible_total: u64,
            nonce_hash: T::Hash,
        },
        /// 内部全过(非特别案),推进至行政签署阶段。
        LegislationAdvancedToSign { proposal_id: u64 },
        /// 行政首长(市长/省长/总统)已签署或否决。
        LegislationExecutiveSigned {
            proposal_id: u64,
            who: T::AccountId,
            approve: bool,
        },
        /// 退回立法院三人会签阶段(省/国家级)。
        LegislationAdvancedToOverride { proposal_id: u64 },
        /// 三人会签其一已签署或否决。
        LegislationOverrideSigned {
            proposal_id: u64,
            who: T::AccountId,
            approve: bool,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 院序列为空或超上限
        InvalidHouses,
        /// 表决类型不合法
        InvalidVoteType,
        /// 提案元数据缺失
        ProposalMetaMissing,
        /// 人口快照未准备
        PopulationSnapshotNotPrepared,
        /// 人口快照非本区块准备(过期)
        PopulationSnapshotNotCurrent,
        /// 人口快照验签失败或字段非法
        InvalidPopulationSnapshot,
        /// 公投分母未设置
        CitizenEligibleTotalNotSet,
        /// CID 持有者无公投资格
        CidNotEligible,
        /// CID 投票凭证非法
        InvalidCidVoteCredential,
        /// 提案不在该阶段(签署/会签 stage 校验)
        NotInExpectedStage,
        /// 签署人不是该机构法定代表人(行政签署)
        NotLegalRepresentative,
        /// 签署人不在三人会签合法身份集合(院长/参议长/众议长)
        NotOverrideSigner,
        /// 该身份已在本提案会签过
        AlreadySigned,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 准备特别案公投人口快照(发起特别案提案前一区块由发起人调用)。
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::prepare_population_snapshot())]
        #[allow(clippy::too_many_arguments)]
        pub fn prepare_population_snapshot(
            origin: OriginFor<T>,
            eligible_total: u64,
            snapshot_nonce: votingengine::pallet::VoteNonceOf<T>,
            signature: votingengine::pallet::VoteSignatureOf<T>,
            issuer_cid_number: BoundedVec<u8, ConstU32<128>>,
            issuer_main_account: T::AccountId,
            signer_pubkey: [u8; 32],
            scope_province_name: BoundedVec<u8, ConstU32<64>>,
            scope_city_name: BoundedVec<u8, ConstU32<64>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_prepare_population_snapshot(
                who,
                eligible_total,
                snapshot_nonce,
                signature,
                issuer_cid_number.as_slice(),
                &issuer_main_account,
                &signer_pubkey,
                scope_province_name.as_slice(),
                scope_city_name.as_slice(),
            )
        }

        /// 立法机构议员/委员对当前院投票(一人一票)。
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::cast_house_vote())]
        pub fn cast_house_vote(
            origin: OriginFor<T>,
            proposal_id: u64,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_cast_house_vote(who, proposal_id, approve)
        }

        /// 公民对特别案公投投票(CID 持有者,链上去重计票)。
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::cast_referendum_vote())]
        #[allow(clippy::too_many_arguments)]
        pub fn cast_referendum_vote(
            origin: OriginFor<T>,
            proposal_id: u64,
            binding_id: T::Hash,
            nonce: votingengine::pallet::VoteNonceOf<T>,
            signature: votingengine::pallet::VoteSignatureOf<T>,
            issuer_cid_number: BoundedVec<u8, ConstU32<128>>,
            issuer_main_account: T::AccountId,
            signer_pubkey: [u8; 32],
            scope_province_name: BoundedVec<u8, ConstU32<64>>,
            scope_city_name: BoundedVec<u8, ConstU32<64>>,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_cast_referendum_vote(
                who,
                proposal_id,
                binding_id,
                nonce,
                signature,
                issuer_cid_number,
                issuer_main_account,
                signer_pubkey,
                scope_province_name,
                scope_city_name,
                approve,
            )
        }

        /// 行政首长(机构法定代表人:市长/省长/总统)对终审通过的非特别案签署或否决。
        /// 批准=生效;否决:市级=否决,省/国级=退回三人会签。
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::cast_house_vote())]
        pub fn executive_sign(
            origin: OriginFor<T>,
            proposal_id: u64,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_executive_sign(who, proposal_id, approve)
        }

        /// 三人会签(省/国家级:立法院院长 + 参议长 + 众议长)签署或否决。
        /// 三人全批准=生效;任一否决=否决。
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::cast_house_vote())]
        pub fn override_sign(
            origin: OriginFor<T>,
            proposal_id: u64,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_override_sign(who, proposal_id, approve)
        }
    }
}

// ──────────────────────────────────────────────────────────────────
// 业务方法
// ──────────────────────────────────────────────────────────────────

impl<T: Config> Pallet<T> {
    fn stage_duration() -> frame_system::pallet_prelude::BlockNumberFor<T> {
        use sp_runtime::traits::SaturatedConversion;
        (VOTING_DURATION_BLOCKS as u64).saturated_into()
    }

    /// 准备特别案公投人口快照:验签 + 去重 + 缓存分母。
    #[allow(clippy::too_many_arguments)]
    pub fn do_prepare_population_snapshot(
        who: T::AccountId,
        eligible_total: u64,
        snapshot_nonce: votingengine::pallet::VoteNonceOf<T>,
        signature: votingengine::pallet::VoteSignatureOf<T>,
        issuer_cid_number: &[u8],
        issuer_main_account: &T::AccountId,
        signer_pubkey: &[u8; 32],
        scope_province_name: &[u8],
        scope_city_name: &[u8],
    ) -> DispatchResult {
        use sp_runtime::traits::Hash as HashT;
        ensure!(eligible_total > 0, Error::<T>::CitizenEligibleTotalNotSet);
        ensure!(
            !snapshot_nonce.is_empty(),
            Error::<T>::InvalidPopulationSnapshot
        );
        ensure!(!signature.is_empty(), Error::<T>::InvalidPopulationSnapshot);
        ensure!(
            !issuer_cid_number.is_empty(),
            Error::<T>::InvalidPopulationSnapshot
        );
        ensure!(
            !scope_province_name.is_empty(),
            Error::<T>::InvalidPopulationSnapshot
        );

        let nonce_hash = <T as frame_system::Config>::Hashing::hash(snapshot_nonce.as_slice());
        ensure!(
            !pallet::UsedSnapshotNonce::<T>::get(nonce_hash),
            Error::<T>::InvalidPopulationSnapshot
        );
        ensure!(
            <T as votingengine::Config>::PopulationSnapshotVerifier::verify_population_snapshot(
                &who,
                eligible_total,
                &snapshot_nonce,
                &signature,
                issuer_cid_number,
                issuer_main_account,
                signer_pubkey,
                scope_province_name,
                scope_city_name,
            ),
            Error::<T>::InvalidPopulationSnapshot
        );

        let now = <frame_system::Pallet<T>>::block_number();
        pallet::UsedSnapshotNonce::<T>::insert(nonce_hash, true);
        pallet::PendingPopulationSnapshots::<T>::insert(
            &who,
            pallet::PreparedSnapshot {
                eligible_total,
                nonce_hash,
                prepared_at: now,
            },
        );
        Self::deposit_event(pallet::Event::<T>::PopulationSnapshotPrepared {
            who,
            eligible_total,
            nonce_hash,
        });
        Ok(())
    }

    /// 创建立法提案:锁定发起院管理员快照,建核心提案进入第一院内部表决。
    pub fn do_create_legislation_proposal(
        who: T::AccountId,
        houses: sp_runtime::sp_std::vec::Vec<(InstitutionCode, T::AccountId)>,
        vote_type: u8,
        executive: (InstitutionCode, T::AccountId),
        legislature: Option<(InstitutionCode, T::AccountId)>,
    ) -> Result<u64, DispatchError> {
        ensure!(!houses.is_empty(), Error::<T>::InvalidHouses);
        ensure!(vote_type <= LEG_VOTE_SPECIAL, Error::<T>::InvalidVoteType);
        let bounded_houses: pallet::HousesOf<T> = houses
            .clone()
            .try_into()
            .map_err(|_| Error::<T>::InvalidHouses)?;
        let (first_code, first_account) = houses[0].clone();
        // ADR-027 修订:提案方与表决院解耦——发起人资格由 legislation-yuan 对 proposer_body 校验,
        // 本层只锁定 houses[0](表决院)管理员快照;发起人若属表决院则自动赞成一票(国家/省两院),
        // 市级 市自治会/市教委会 委员提案时发起人不在表决院,不自动投票(市立法会从零计票)。

        let referendum_required = vote_type == LEG_VOTE_SPECIAL;
        let now = <frame_system::Pallet<T>>::block_number();
        // 特别案:消费已准备的人口快照作为公投分母。
        let eligible_total = if referendum_required {
            let prepared = pallet::PendingPopulationSnapshots::<T>::get(&who)
                .ok_or(Error::<T>::PopulationSnapshotNotPrepared)?;
            if prepared.prepared_at != now {
                pallet::PendingPopulationSnapshots::<T>::remove(&who);
                return Err(Error::<T>::PopulationSnapshotNotCurrent.into());
            }
            prepared.eligible_total
        } else {
            0
        };

        let end = now.saturating_add(Self::stage_duration());
        let proposal = Proposal {
            kind: PROPOSAL_KIND_LEGISLATION,
            stage: STAGE_LEG_HOUSE,
            status: STATUS_VOTING,
            internal_code: Some(first_code),
            internal_institution: Some(first_account.clone()),
            start: now,
            end,
            citizen_eligible_total: eligible_total,
        };

        with_transaction(|| {
            let id = match <votingengine::Pallet<T>>::allocate_proposal_id() {
                Ok(id) => id,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };
            if let Err(err) =
                votingengine::limit::try_add_active_proposal::<T>(first_account.clone(), id)
            {
                return TransactionOutcome::Rollback(Err(err));
            }
            if let Err(err) = <votingengine::Pallet<T>>::acquire_internal_proposal_mutex(
                id,
                first_code,
                first_account.clone(),
                InternalProposalMutexKind::Regular,
            ) {
                return TransactionOutcome::Rollback(Err(err));
            }
            if let Err(err) = <votingengine::Pallet<T>>::snapshot_institution_admins(
                id,
                first_code,
                first_account.clone(),
                false,
            ) {
                return TransactionOutcome::Rollback(Err(err));
            }
            if referendum_required {
                pallet::PendingPopulationSnapshots::<T>::remove(&who);
            }
            pallet::LegMeta::<T>::insert(
                id,
                pallet::LegislationMeta {
                    vote_type,
                    houses: bounded_houses,
                    current_house: 0,
                    referendum_required,
                    executive,
                    legislature,
                },
            );
            Proposals::<T>::insert(id, proposal);
            if let Err(err) = <votingengine::Pallet<T>>::schedule_proposal_expiry(id, end) {
                return TransactionOutcome::Rollback(Err(err));
            }
            <votingengine::Pallet<T>>::emit_proposal_created(
                id,
                PROPOSAL_KIND_LEGISLATION,
                STAGE_LEG_HOUSE,
                end,
            );
            TransactionOutcome::Commit(Ok(id))
        })
    }

    /// 当前院投票:计票并按表决类型判定(通过→推进/否决)。
    pub fn do_cast_house_vote(
        who: T::AccountId,
        proposal_id: u64,
        approve: bool,
    ) -> DispatchResult {
        let proposal = <votingengine::Pallet<T>>::ensure_open_proposal(proposal_id)?;
        ensure!(
            proposal.kind == PROPOSAL_KIND_LEGISLATION,
            votingengine::Error::<T>::InvalidProposalKind
        );
        ensure!(
            proposal.stage == STAGE_LEG_HOUSE,
            votingengine::Error::<T>::InvalidProposalStage
        );
        let meta = pallet::LegMeta::<T>::get(proposal_id).ok_or(Error::<T>::ProposalMetaMissing)?;
        let (_code, institution) = meta
            .houses
            .get(meta.current_house as usize)
            .cloned()
            .ok_or(Error::<T>::InvalidHouses)?;
        ensure!(
            !pallet::LegHouseVotesByAdmin::<T>::contains_key(proposal_id, &who),
            votingengine::Error::<T>::AlreadyVoted
        );
        ensure!(
            <votingengine::Pallet<T>>::is_admin_in_snapshot(proposal_id, institution.clone(), &who),
            votingengine::Error::<T>::NoPermission
        );

        pallet::LegHouseVotesByAdmin::<T>::insert(proposal_id, &who, approve);
        let tally = pallet::LegHouseTally::<T>::mutate(proposal_id, |t| {
            if approve {
                t.yes = t.yes.saturating_add(1);
            } else {
                t.no = t.no.saturating_add(1);
            }
            *t
        });
        Self::deposit_event(pallet::Event::<T>::LegislationHouseVoteCast {
            proposal_id,
            who,
            approve,
        });

        let admins_len = <votingengine::Pallet<T>>::snapshot_admins_len(proposal_id, institution)
            .ok_or(votingengine::Error::<T>::MissingAdminSnapshot)?;
        match legislation_house_decided(meta.vote_type, admins_len, tally.yes, tally.no) {
            Some(true) => Self::advance_house_or_finalize(proposal_id, meta),
            Some(false) => {
                <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, STATUS_REJECTED)
            }
            None => Ok(()),
        }
    }

    /// 某院通过后:还有下一院则推进,否则(特别案进公投 / 其余直接 PASSED)。
    fn advance_house_or_finalize(
        proposal_id: u64,
        meta: pallet::LegislationMeta<T>,
    ) -> DispatchResult {
        let next = meta.current_house.saturating_add(1);
        if (next as usize) < meta.houses.len() {
            let (next_code, next_account) = meta.houses[next as usize].clone();
            let now = <frame_system::Pallet<T>>::block_number();
            let end = now.saturating_add(Self::stage_duration());
            with_transaction(|| {
                // 重置当前院计票(去重表保留,两院议员账户不重叠)。
                pallet::LegHouseTally::<T>::remove(proposal_id);
                pallet::LegMeta::<T>::mutate(proposal_id, |maybe| {
                    if let Some(m) = maybe {
                        m.current_house = next;
                    }
                });
                if let Err(err) = <votingengine::Pallet<T>>::snapshot_institution_admins(
                    proposal_id,
                    next_code,
                    next_account.clone(),
                    false,
                ) {
                    return TransactionOutcome::Rollback(Err(err));
                }
                let old_end =
                    match Proposals::<T>::try_mutate(
                        proposal_id,
                        |maybe| -> Result<
                            frame_system::pallet_prelude::BlockNumberFor<T>,
                            DispatchError,
                        > {
                            let p = maybe
                                .as_mut()
                                .ok_or(votingengine::Error::<T>::ProposalNotFound)?;
                            let old = p.end;
                            p.internal_code = Some(next_code);
                            p.internal_institution = Some(next_account.clone());
                            p.start = now;
                            p.end = end;
                            Ok(old)
                        },
                    ) {
                        Ok(v) => v,
                        Err(err) => return TransactionOutcome::Rollback(Err(err)),
                    };
                let old_expiry = old_end.saturating_add(One::one());
                ProposalsByExpiry::<T>::mutate(old_expiry, |ids| ids.retain(|&i| i != proposal_id));
                if let Err(err) =
                    <votingengine::Pallet<T>>::schedule_proposal_expiry(proposal_id, end)
                {
                    return TransactionOutcome::Rollback(Err(err));
                }
                Self::deposit_event(pallet::Event::<T>::LegislationHouseAdvanced {
                    proposal_id,
                    next_house: next,
                });
                TransactionOutcome::Commit(Ok(()))
            })
        } else if meta.referendum_required {
            Self::advance_to_referendum(proposal_id)
        } else {
            // 非特别案:内部全过 → 进入行政签署阶段(市长/省长/总统)。
            Self::advance_to_sign(proposal_id)
        }
    }

    /// 内部全过 → 推进至强制公投阶段(对标 joint advance_to_citizen)。
    fn advance_to_referendum(proposal_id: u64) -> DispatchResult {
        let now = <frame_system::Pallet<T>>::block_number();
        let end = now.saturating_add(Self::stage_duration());
        with_transaction(|| {
            let (eligible_total, old_end) = match Proposals::<T>::try_mutate(
                proposal_id,
                |maybe| -> Result<
                    (u64, frame_system::pallet_prelude::BlockNumberFor<T>),
                    DispatchError,
                > {
                    let p = maybe
                        .as_mut()
                        .ok_or(votingengine::Error::<T>::ProposalNotFound)?;
                    let eligible_total = p.citizen_eligible_total;
                    let old = p.end;
                    p.stage = STAGE_LEG_REFERENDUM;
                    p.start = now;
                    p.end = end;
                    Ok((eligible_total, old))
                },
            ) {
                Ok(v) => v,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };
            let old_expiry = old_end.saturating_add(One::one());
            ProposalsByExpiry::<T>::mutate(old_expiry, |ids| ids.retain(|&i| i != proposal_id));
            if let Err(err) = <votingengine::Pallet<T>>::schedule_proposal_expiry(proposal_id, end)
            {
                return TransactionOutcome::Rollback(Err(err));
            }
            <votingengine::Pallet<T>>::release_internal_proposal_mutexes(proposal_id);
            <votingengine::Pallet<T>>::emit_proposal_advanced_to_citizen(
                proposal_id,
                end,
                eligible_total,
            );
            Self::deposit_event(pallet::Event::<T>::LegislationAdvancedToReferendum {
                proposal_id,
                eligible_total,
            });
            TransactionOutcome::Commit(Ok(()))
        })
    }

    /// 通用阶段切换:写新 stage + 重置计时窗口 + 重排到期桶(签署/会签阶段共用)。
    fn transition_stage(proposal_id: u64, new_stage: u8) -> DispatchResult {
        let now = <frame_system::Pallet<T>>::block_number();
        let end = now.saturating_add(Self::stage_duration());
        with_transaction(|| {
            let old_end = match Proposals::<T>::try_mutate(
                proposal_id,
                |maybe| -> Result<frame_system::pallet_prelude::BlockNumberFor<T>, DispatchError> {
                    let p = maybe
                        .as_mut()
                        .ok_or(votingengine::Error::<T>::ProposalNotFound)?;
                    let old = p.end;
                    p.stage = new_stage;
                    p.start = now;
                    p.end = end;
                    Ok(old)
                },
            ) {
                Ok(v) => v,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };
            let old_expiry = old_end.saturating_add(One::one());
            ProposalsByExpiry::<T>::mutate(old_expiry, |ids| ids.retain(|&i| i != proposal_id));
            if let Err(err) = <votingengine::Pallet<T>>::schedule_proposal_expiry(proposal_id, end)
            {
                return TransactionOutcome::Rollback(Err(err));
            }
            TransactionOutcome::Commit(Ok(()))
        })
    }

    /// 非特别案内部全过 → 进入行政签署阶段(市长/省长/总统)。
    fn advance_to_sign(proposal_id: u64) -> DispatchResult {
        Self::transition_stage(proposal_id, STAGE_LEG_SIGN)?;
        <votingengine::Pallet<T>>::release_internal_proposal_mutexes(proposal_id);
        Self::deposit_event(pallet::Event::<T>::LegislationAdvancedToSign { proposal_id });
        Ok(())
    }

    /// 行政首长否决/超时(省国级) → 退回立法院三人会签阶段。
    fn advance_to_override(proposal_id: u64) -> DispatchResult {
        pallet::LegOverrideSigns::<T>::remove(proposal_id);
        Self::transition_stage(proposal_id, STAGE_LEG_OVERRIDE)?;
        Self::deposit_event(pallet::Event::<T>::LegislationAdvancedToOverride { proposal_id });
        Ok(())
    }

    /// 实时查机构法定代表人(机构首脑;ADR-027 签署人)。
    fn legal_rep_of(body: &(InstitutionCode, T::AccountId)) -> Option<T::AccountId> {
        <T as votingengine::Config>::InternalAdminProvider::legal_representative(
            body.0,
            body.1.clone(),
        )
    }

    /// 行政签署:机构法定代表人(市长/省长/总统)批准=生效;否决:市级=否决/省国级=退回会签。
    pub fn do_executive_sign(who: T::AccountId, proposal_id: u64, approve: bool) -> DispatchResult {
        let proposal =
            Proposals::<T>::get(proposal_id).ok_or(votingengine::Error::<T>::ProposalNotFound)?;
        ensure!(
            proposal.status == STATUS_VOTING,
            Error::<T>::NotInExpectedStage
        );
        ensure!(
            proposal.stage == STAGE_LEG_SIGN,
            Error::<T>::NotInExpectedStage
        );
        let meta = pallet::LegMeta::<T>::get(proposal_id).ok_or(Error::<T>::ProposalMetaMissing)?;
        let rep = Self::legal_rep_of(&meta.executive).ok_or(Error::<T>::NotLegalRepresentative)?;
        ensure!(who == rep, Error::<T>::NotLegalRepresentative);
        Self::deposit_event(pallet::Event::<T>::LegislationExecutiveSigned {
            proposal_id,
            who,
            approve,
        });
        if approve {
            <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, STATUS_PASSED)
        } else if meta.legislature.is_some() {
            // 省/国家级:否决 → 退回三人会签救济。
            Self::advance_to_override(proposal_id)
        } else {
            // 市级:无救济,否决即否决。
            <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, STATUS_REJECTED)
        }
    }

    /// 三人会签合法身份(院长 + 众议长 + 参议长 = 立法院/众议会/参议会三机构法定代表人)。
    fn override_signers(
        meta: &pallet::LegislationMeta<T>,
    ) -> sp_runtime::sp_std::vec::Vec<T::AccountId> {
        let mut out = sp_runtime::sp_std::vec::Vec::new();
        if let Some(leg) = meta.legislature.as_ref() {
            if let Some(rep) = Self::legal_rep_of(leg) {
                out.push(rep);
            }
        }
        for h in meta.houses.iter() {
            if let Some(rep) = Self::legal_rep_of(h) {
                out.push(rep);
            }
        }
        out
    }

    /// 三人会签:院长/参议长/众议长各一票,任一否决=否决,集齐 3 个不同身份赞成=生效。
    pub fn do_override_sign(who: T::AccountId, proposal_id: u64, approve: bool) -> DispatchResult {
        let proposal =
            Proposals::<T>::get(proposal_id).ok_or(votingengine::Error::<T>::ProposalNotFound)?;
        ensure!(
            proposal.status == STATUS_VOTING,
            Error::<T>::NotInExpectedStage
        );
        ensure!(
            proposal.stage == STAGE_LEG_OVERRIDE,
            Error::<T>::NotInExpectedStage
        );
        let meta = pallet::LegMeta::<T>::get(proposal_id).ok_or(Error::<T>::ProposalMetaMissing)?;
        let signers = Self::override_signers(&meta);
        ensure!(
            signers.iter().any(|s| s == &who),
            Error::<T>::NotOverrideSigner
        );
        let mut signs = pallet::LegOverrideSigns::<T>::get(proposal_id);
        ensure!(
            !signs.iter().any(|(s, _)| s == &who),
            Error::<T>::AlreadySigned
        );
        Self::deposit_event(pallet::Event::<T>::LegislationOverrideSigned {
            proposal_id,
            who: who.clone(),
            approve,
        });
        if !approve {
            // 任一否决即否决。
            return <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, STATUS_REJECTED);
        }
        signs
            .try_push((who, true))
            .map_err(|_| Error::<T>::AlreadySigned)?;
        let approvals = signs.iter().filter(|(_, a)| *a).count();
        pallet::LegOverrideSigns::<T>::insert(proposal_id, signs);
        // 三人(院长+参议长+众议长)全批准 → 生效。
        if approvals >= 3 {
            <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, STATUS_PASSED)
        } else {
            Ok(())
        }
    }

    /// 行政签署阶段超时:市级(无 legislature)= 视为通过;省/国级 = 退回三人会签。
    pub fn do_finalize_sign_timeout(
        proposal: &Proposal<frame_system::pallet_prelude::BlockNumberFor<T>, T::AccountId>,
        proposal_id: u64,
    ) -> DispatchResult {
        ensure!(
            proposal.stage == STAGE_LEG_SIGN,
            votingengine::Error::<T>::InvalidProposalStage
        );
        ensure!(
            proposal.status == STATUS_VOTING,
            votingengine::Error::<T>::ProposalAlreadyFinalized
        );
        let meta = pallet::LegMeta::<T>::get(proposal_id).ok_or(Error::<T>::ProposalMetaMissing)?;
        if meta.legislature.is_some() {
            Self::advance_to_override(proposal_id)
        } else {
            <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, STATUS_PASSED)
        }
    }

    /// 三人会签阶段超时:法案否决。
    pub fn do_finalize_override_timeout(
        proposal: &Proposal<frame_system::pallet_prelude::BlockNumberFor<T>, T::AccountId>,
        proposal_id: u64,
    ) -> DispatchResult {
        ensure!(
            proposal.stage == STAGE_LEG_OVERRIDE,
            votingengine::Error::<T>::InvalidProposalStage
        );
        ensure!(
            proposal.status == STATUS_VOTING,
            votingengine::Error::<T>::ProposalAlreadyFinalized
        );
        <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, STATUS_REJECTED)
    }

    /// 内部表决阶段超时结算:按表决类型期满计票,通过→推进,否则否决。
    pub fn do_finalize_house_timeout(
        proposal: &Proposal<frame_system::pallet_prelude::BlockNumberFor<T>, T::AccountId>,
        proposal_id: u64,
    ) -> DispatchResult {
        ensure!(
            proposal.stage == STAGE_LEG_HOUSE,
            votingengine::Error::<T>::InvalidProposalStage
        );
        ensure!(
            proposal.status == STATUS_VOTING,
            votingengine::Error::<T>::ProposalAlreadyFinalized
        );
        ensure!(
            <frame_system::Pallet<T>>::block_number() > proposal.end,
            votingengine::Error::<T>::VoteNotExpired
        );
        let meta = pallet::LegMeta::<T>::get(proposal_id).ok_or(Error::<T>::ProposalMetaMissing)?;
        let (_code, institution) = meta
            .houses
            .get(meta.current_house as usize)
            .cloned()
            .ok_or(Error::<T>::InvalidHouses)?;
        let admins_len = <votingengine::Pallet<T>>::snapshot_admins_len(proposal_id, institution)
            .ok_or(votingengine::Error::<T>::MissingAdminSnapshot)?;
        let tally = pallet::LegHouseTally::<T>::get(proposal_id);
        if legislation_house_final_passed(meta.vote_type, admins_len, tally.yes, tally.no) {
            Self::advance_house_or_finalize(proposal_id, meta)
        } else {
            <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, STATUS_REJECTED)
        }
    }

    /// 公投投票:CID 资格实时验签 + 链上去重计票(期满计票,本入口不提前判定)。
    #[allow(clippy::too_many_arguments)]
    pub fn do_cast_referendum_vote(
        who: T::AccountId,
        proposal_id: u64,
        binding_id: T::Hash,
        nonce: votingengine::pallet::VoteNonceOf<T>,
        signature: votingengine::pallet::VoteSignatureOf<T>,
        issuer_cid_number: frame_support::BoundedVec<u8, frame_support::traits::ConstU32<128>>,
        issuer_main_account: T::AccountId,
        signer_pubkey: [u8; 32],
        scope_province_name: frame_support::BoundedVec<u8, frame_support::traits::ConstU32<64>>,
        scope_city_name: frame_support::BoundedVec<u8, frame_support::traits::ConstU32<64>>,
        approve: bool,
    ) -> DispatchResult {
        let proposal = <votingengine::Pallet<T>>::ensure_open_proposal(proposal_id)?;
        ensure!(
            proposal.kind == PROPOSAL_KIND_LEGISLATION,
            votingengine::Error::<T>::InvalidProposalKind
        );
        ensure!(
            proposal.stage == STAGE_LEG_REFERENDUM,
            votingengine::Error::<T>::InvalidProposalStage
        );
        ensure!(
            proposal.citizen_eligible_total > 0,
            Error::<T>::CitizenEligibleTotalNotSet
        );
        ensure!(
            <T as votingengine::Config>::CidEligibility::is_eligible(&binding_id, &who),
            Error::<T>::CidNotEligible
        );
        ensure!(
            !pallet::LegReferendumVotesByBindingId::<T>::contains_key(proposal_id, binding_id),
            votingengine::Error::<T>::AlreadyVoted
        );
        ensure!(
            <T as votingengine::Config>::CidEligibility::verify_and_consume_vote_credential(
                &binding_id,
                &who,
                proposal_id,
                nonce.as_slice(),
                signature.as_slice(),
                issuer_cid_number.as_slice(),
                &issuer_main_account,
                &signer_pubkey,
                scope_province_name.as_slice(),
                scope_city_name.as_slice(),
            ),
            Error::<T>::InvalidCidVoteCredential
        );

        pallet::LegReferendumVotesByBindingId::<T>::insert(proposal_id, binding_id, approve);
        pallet::LegReferendumTally::<T>::mutate(proposal_id, |t| {
            if approve {
                t.yes = t.yes.saturating_add(1);
            } else {
                t.no = t.no.saturating_add(1);
            }
        });
        Self::deposit_event(pallet::Event::<T>::LegislationReferendumVoteCast {
            proposal_id,
            who,
            approve,
        });
        Ok(())
    }

    /// 公投阶段超时结算:按宪法 ≥70% 参与 + ≥70% 赞成判定。
    pub fn do_finalize_referendum_timeout(
        proposal: &Proposal<frame_system::pallet_prelude::BlockNumberFor<T>, T::AccountId>,
        proposal_id: u64,
    ) -> DispatchResult {
        ensure!(
            proposal.stage == STAGE_LEG_REFERENDUM,
            votingengine::Error::<T>::InvalidProposalStage
        );
        ensure!(
            proposal.status == STATUS_VOTING,
            votingengine::Error::<T>::ProposalAlreadyFinalized
        );
        ensure!(
            <frame_system::Pallet<T>>::block_number() > proposal.end,
            votingengine::Error::<T>::VoteNotExpired
        );
        let tally = pallet::LegReferendumTally::<T>::get(proposal_id);
        let status = if legislation_referendum_final_passed(
            proposal.citizen_eligible_total,
            tally.yes,
            tally.no,
        ) {
            STATUS_PASSED
        } else {
            STATUS_REJECTED
        };
        <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, status)
    }
}

// ──────────────────────────────────────────────────────────────────
// trait 实现(供 votingengine 核心 + 业务壳接入)
// ──────────────────────────────────────────────────────────────────

impl<T: Config> votingengine::LegislationVoteEngine<T::AccountId> for Pallet<T> {
    fn create_legislation_proposal(
        who: T::AccountId,
        houses: sp_runtime::sp_std::vec::Vec<(InstitutionCode, T::AccountId)>,
        vote_type: u8,
        executive: (InstitutionCode, T::AccountId),
        legislature: Option<(InstitutionCode, T::AccountId)>,
        module_tag: &[u8],
        data: sp_runtime::sp_std::vec::Vec<u8>,
        object_data: sp_runtime::sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        // 表决院 houses[0] 账户(发起院/市立法会),供自动投票判定。
        let first_account = match houses.first() {
            Some((_, acct)) => acct.clone(),
            None => return Err(Error::<T>::InvalidHouses.into()),
        };
        with_transaction(|| {
            let id = match Self::do_create_legislation_proposal(
                who.clone(),
                houses,
                vote_type,
                executive,
                legislature,
            ) {
                Ok(id) => id,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };
            let now = <frame_system::Pallet<T>>::block_number();
            if let Err(err) =
                <votingengine::Pallet<T>>::register_proposal_data(id, module_tag, data, now)
            {
                return TransactionOutcome::Rollback(Err(err));
            }
            if let Err(err) = <votingengine::Pallet<T>>::store_proposal_object(
                id,
                PROPOSAL_OBJECT_KIND_LAW_TEXT,
                object_data,
            ) {
                return TransactionOutcome::Rollback(Err(err));
            }
            // 发起人若属表决院(国家/省两院:发起院=众议会/教委会)则自动赞成一票;
            // 市级 市自治会/市教委会 委员提案时发起人不在表决院(市立法会),不自动投票。
            if <votingengine::Pallet<T>>::is_admin_in_snapshot(id, first_account, &who) {
                match Self::do_cast_house_vote(who, id, true) {
                    Ok(()) => TransactionOutcome::Commit(Ok(id)),
                    Err(err) => TransactionOutcome::Rollback(Err(err)),
                }
            } else {
                TransactionOutcome::Commit(Ok(id))
            }
        })
    }
}

impl<T: Config>
    votingengine::traits::LegislationProposalFinalizer<
        frame_system::pallet_prelude::BlockNumberFor<T>,
        T::AccountId,
    > for Pallet<T>
{
    fn finalize_legislation_house_timeout(
        proposal: &Proposal<frame_system::pallet_prelude::BlockNumberFor<T>, T::AccountId>,
        proposal_id: u64,
    ) -> DispatchResult {
        Self::do_finalize_house_timeout(proposal, proposal_id)
    }

    fn finalize_legislation_referendum_timeout(
        proposal: &Proposal<frame_system::pallet_prelude::BlockNumberFor<T>, T::AccountId>,
        proposal_id: u64,
    ) -> DispatchResult {
        Self::do_finalize_referendum_timeout(proposal, proposal_id)
    }

    fn finalize_legislation_sign_timeout(
        proposal: &Proposal<frame_system::pallet_prelude::BlockNumberFor<T>, T::AccountId>,
        proposal_id: u64,
    ) -> DispatchResult {
        Self::do_finalize_sign_timeout(proposal, proposal_id)
    }

    fn finalize_legislation_override_timeout(
        proposal: &Proposal<frame_system::pallet_prelude::BlockNumberFor<T>, T::AccountId>,
        proposal_id: u64,
    ) -> DispatchResult {
        Self::do_finalize_override_timeout(proposal, proposal_id)
    }
}

impl<T: Config> votingengine::traits::LegislationCleanupHandler for Pallet<T> {
    fn cleanup_legislation_house_votes_chunk(
        proposal_id: u64,
        limit: u32,
    ) -> votingengine::traits::CleanupChunkResult {
        let result = pallet::LegHouseVotesByAdmin::<T>::clear_prefix(proposal_id, limit, None);
        (result.unique, result.maybe_cursor.is_some())
    }

    fn cleanup_legislation_referendum_votes_chunk(
        proposal_id: u64,
        limit: u32,
    ) -> votingengine::traits::CleanupChunkResult {
        let result =
            pallet::LegReferendumVotesByBindingId::<T>::clear_prefix(proposal_id, limit, None);
        (result.unique, result.maybe_cursor.is_some())
    }

    fn cleanup_legislation_terminal(proposal_id: u64) {
        pallet::LegMeta::<T>::remove(proposal_id);
        pallet::LegHouseTally::<T>::remove(proposal_id);
        pallet::LegReferendumTally::<T>::remove(proposal_id);
        pallet::LegOverrideSigns::<T>::remove(proposal_id);
    }
}
