//! # 立法投票 pallet (legislation-vote)
//!
//! 立法机构专属投票模式(ADR-027,公民宪法第45/46条)。投票引擎「头等模式」:
//! `PROPOSAL_KIND_LEGISLATION`,共享核心 `votingengine`(Proposals/AdminSnapshot/状态机/
//! 公投快照验签/清理/反向索引),只本地保管计票账本。三个既有投票 sub-pallet
//! (internal/joint/citizen)逻辑零改动。
//!
//! 阶段(ADR-027,当前五类提案 + 特别案公投 + 行政签署/三人会签/护宪终审):
//! - `STAGE_LEG_HOUSE` 内部表决:单院(市立法会)一段;两院(国家/省立法院 众→参;教委会→参议会)顺序两段。
//! - `STAGE_LEG_REFERENDUM` 强制公投:仅特别案(含核心修宪),内部全过后强制进入,公投通过即生效不签署。
//! - `STAGE_LEG_SIGN` 行政签署:非特别案内部全过后,行政机构法定代表人(市长/省长/总统)签署。
//!   市级无救济(否决=否决/30天超时=通过);省国级否决或超时 → 会签。
//! - `STAGE_LEG_OVERRIDE` 三人会签(省/国家级):立法院院长 + 参议长 + 众议长,全签=生效/任一否决或超时=否决。
//!
//! 计票口径:按现任议员/委员管理员快照总数算参与率/赞成率(`votingengine::types`
//! 的立法阈值纯函数),投票期满 finalize 统一判定;结果已确定时可提前决。

#![cfg_attr(not(feature = "std"), no_std)]

pub mod weights;

#[cfg(test)]
mod tests;

pub use pallet::*;

use entity_primitives::InstitutionMultisigQuery;
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
        legislation_referendum_final_passed, InstitutionCode, ProposalSubjectCidNumbers,
        LEG_VOTE_SPECIAL,
    },
    CitizenIdentityReader, InternalAdminProvider, InternalProposalMutexKind, PopulationScope,
    Proposal, PROPOSAL_KIND_LEGISLATION, STAGE_LEG_CONSTITUTION_GUARD, STAGE_LEG_HOUSE,
    STAGE_LEG_OVERRIDE, STAGE_LEG_REFERENDUM, STAGE_LEG_SIGN, STATUS_PASSED, STATUS_REJECTED,
    STATUS_VOTING,
};

/// 法律全文大对象类型标记(写入 votingengine `ProposalObject`),与 legislation-yuan 对齐。
pub const PROPOSAL_OBJECT_KIND_LAW_TEXT: u8 = 2;

/// 单部法律最多院数,单一真源在 `votingengine::types::MAX_LEGISLATION_HOUSES`。
pub const MAX_HOUSES: u32 = votingengine::types::MAX_LEGISLATION_HOUSES;

/// 护宪大法官法定人数(宪法第20条):7 人。
pub const CONSTITUTION_GUARD_MEMBERS: u32 = 7;

/// 修宪终审通过阈值(宪法第21条):7 人多数通过,即 4 名及以上护宪大法官赞成。
pub const CONSTITUTION_GUARD_APPROVAL_THRESHOLD: usize = 4;

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
        /// 行政签署机构(总统府/省联邦政府/市政府);其法定代表人=总统/省长/市长。非特别案末段签署。
        pub executive: (InstitutionCode, T::AccountId),
        /// 两院级的立法院机构(国家/省立法院);其法定代表人=院长,供三人会签。单院(市)=None。
        pub legislature: Option<(InstitutionCode, T::AccountId)>,
        /// 是否修宪(tier=宪法):为真时,现有流程通过后最后进护宪大法官终审(宪法第21条)。
        pub needs_guard: bool,
        /// 特别案公投作用域。非特别案为 None。
        pub referendum_scope: Option<PopulationScope>,
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
    pub struct PreparedSnapshot<BlockNumber> {
        pub eligible_total: u64,
        pub scope: PopulationScope,
        pub prepared_at: BlockNumber,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config + votingengine::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// 机构账户 → CID 查询入口。立法提案用 CID 记录所有关联机构主体。
        type InstitutionQuery: InstitutionMultisigQuery<Self::AccountId>;
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

    /// 公投去重:(proposal_id, 公民钱包账户) → 赞成/反对。
    #[pallet::storage]
    pub type LegReferendumVotesByAccount<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        T::AccountId,
        bool,
        OptionQuery,
    >;

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

    /// 护宪大法官终审记录(仅修宪 STAGE_LEG_CONSTITUTION_GUARD):proposal_id → [(护宪大法官, 是否赞成)]。
    /// 去重 + 4 名及以上赞成判通过。成员集来自 `InternalAdminProvider::constitution_guard_members`。
    #[pallet::storage]
    pub type LegGuardSigns<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        BoundedVec<(T::AccountId, bool), ConstU32<CONSTITUTION_GUARD_MEMBERS>>,
        ValueQuery,
    >;

    /// 待消费的人口快照:发起人 → 已验签缓存(特别案发起前一区块准备)。
    #[pallet::storage]
    pub type PendingPopulationSnapshots<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        PreparedSnapshot<BlockNumberFor<T>>,
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
            scope: PopulationScope,
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
        /// 修宪通过现有流程,推进至护宪大法官终审阶段。
        LegislationAdvancedToGuard { proposal_id: u64 },
        /// 护宪大法官其一已表决(修宪终审)。
        LegislationGuardVoted {
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
        /// 人口快照作用域没有可投票公民
        InvalidPopulationSnapshot,
        /// 公投分母未设置
        CitizenEligibleTotalNotSet,
        /// 公民身份无公投资格
        CitizenNotEligible,
        /// 公投作用域缺失
        PopulationScopeMissing,
        /// 提案不在该阶段(签署/会签 stage 校验)
        NotInExpectedStage,
        /// 签署人不是该机构法定代表人(行政签署)
        NotLegalRepresentative,
        /// 签署人不在三人会签合法身份集合(院长/参议长/众议长)
        NotOverrideSigner,
        /// 该身份已在本提案会签过
        AlreadySigned,
        /// 签署人不是护宪大法官
        NotConstitutionGuard,
        /// 护宪大法官成员数不是 7 人或成员重复
        InvalidGuardMembersLen,
        /// 机构账户无法解析到唯一 CID。
        InvalidInstitutionCid,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 准备特别案公投人口快照(发起特别案提案前一区块由发起人调用)。
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::prepare_population_snapshot())]
        pub fn prepare_population_snapshot(
            origin: OriginFor<T>,
            scope: PopulationScope,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_prepare_population_snapshot(who, scope)
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

        /// 公民对特别案公投投票(链上公民身份持有者,链上按账户去重计票)。
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::cast_referendum_vote())]
        pub fn cast_referendum_vote(
            origin: OriginFor<T>,
            proposal_id: u64,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_cast_referendum_vote(who, proposal_id, approve)
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

        /// 护宪大法官对修宪提案终审表决(宪法第21条):一人一票,4名及以上赞成→生效。
        #[pallet::call_index(5)]
        #[pallet::weight(<T as Config>::WeightInfo::cast_house_vote())]
        pub fn guard_vote(origin: OriginFor<T>, proposal_id: u64, approve: bool) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_guard_vote(who, proposal_id, approve)
        }
    }
}
// 业务方法
impl<T: Config> Pallet<T> {
    fn stage_duration() -> frame_system::pallet_prelude::BlockNumberFor<T> {
        use sp_runtime::traits::SaturatedConversion;
        (VOTING_DURATION_BLOCKS as u64).saturated_into()
    }

    fn push_subject_cid(
        raw: &mut sp_runtime::sp_std::vec::Vec<sp_runtime::sp_std::vec::Vec<u8>>,
        account: &T::AccountId,
    ) -> DispatchResult {
        let cid =
            T::InstitutionQuery::lookup_cid(account).ok_or(Error::<T>::InvalidInstitutionCid)?;
        if !raw.iter().any(|existing| existing == &cid) {
            raw.push(cid);
        }
        Ok(())
    }

    fn resolve_subject_cid_numbers(
        houses: &sp_runtime::sp_std::vec::Vec<(InstitutionCode, T::AccountId)>,
        executive: &(InstitutionCode, T::AccountId),
        legislature: &Option<(InstitutionCode, T::AccountId)>,
    ) -> Result<ProposalSubjectCidNumbers, DispatchError> {
        let mut raw = sp_runtime::sp_std::vec::Vec::new();
        for (_, account) in houses.iter() {
            Self::push_subject_cid(&mut raw, account)?;
        }
        Self::push_subject_cid(&mut raw, &executive.1)?;
        if let Some((_, account)) = legislature.as_ref() {
            Self::push_subject_cid(&mut raw, account)?;
        }
        <votingengine::Pallet<T>>::bound_subject_cid_numbers(raw)
    }

    /// 准备特别案公投人口快照:从链上公民身份模块读取并缓存分母。
    pub fn do_prepare_population_snapshot(
        who: T::AccountId,
        scope: PopulationScope,
    ) -> DispatchResult {
        let eligible_total =
            <T as votingengine::Config>::CitizenIdentityReader::population_count(&scope);
        ensure!(eligible_total > 0, Error::<T>::CitizenEligibleTotalNotSet);
        let now = <frame_system::Pallet<T>>::block_number();
        pallet::PendingPopulationSnapshots::<T>::insert(
            &who,
            pallet::PreparedSnapshot {
                eligible_total,
                scope: scope.clone(),
                prepared_at: now,
            },
        );
        Self::deposit_event(pallet::Event::<T>::PopulationSnapshotPrepared {
            who,
            eligible_total,
            scope,
        });
        Ok(())
    }

    /// 创建立法提案:锁定发起院管理员快照,建核心提案进入第一院内部表决。
    #[allow(clippy::too_many_arguments)]
    pub fn do_create_legislation_proposal(
        who: T::AccountId,
        houses: sp_runtime::sp_std::vec::Vec<(InstitutionCode, T::AccountId)>,
        vote_type: u8,
        executive: (InstitutionCode, T::AccountId),
        legislature: Option<(InstitutionCode, T::AccountId)>,
        needs_guard: bool,
    ) -> Result<u64, DispatchError> {
        ensure!(!houses.is_empty(), Error::<T>::InvalidHouses);
        ensure!(vote_type <= LEG_VOTE_SPECIAL, Error::<T>::InvalidVoteType);
        let bounded_houses: pallet::HousesOf<T> = houses
            .clone()
            .try_into()
            .map_err(|_| Error::<T>::InvalidHouses)?;
        let (first_code, first_account) = houses[0].clone();
        let subject_cid_numbers =
            Self::resolve_subject_cid_numbers(&houses, &executive, &legislature)?;
        // ADR-027 修订:提案方与表决院解耦——发起人资格由 legislation-yuan 对 proposer_body 校验,
        // 本层只锁定 houses[0](表决院)管理员快照;发起人若属表决院则自动赞成一票(国家/省两院),
        // 市级 市自治会/市教委会 委员提案时发起人不在表决院,不自动投票(市立法会从零计票)。

        let referendum_required = vote_type == LEG_VOTE_SPECIAL;
        let now = <frame_system::Pallet<T>>::block_number();
        // 特别案:消费已准备的人口快照作为公投分母。
        let (eligible_total, referendum_scope) = if referendum_required {
            let prepared = pallet::PendingPopulationSnapshots::<T>::get(&who)
                .ok_or(Error::<T>::PopulationSnapshotNotPrepared)?;
            if prepared.prepared_at != now {
                pallet::PendingPopulationSnapshots::<T>::remove(&who);
                return Err(Error::<T>::PopulationSnapshotNotCurrent.into());
            }
            (prepared.eligible_total, Some(prepared.scope))
        } else {
            (0, None)
        };

        let end = now.saturating_add(Self::stage_duration());
        let proposal = Proposal {
            kind: PROPOSAL_KIND_LEGISLATION,
            stage: STAGE_LEG_HOUSE,
            status: STATUS_VOTING,
            internal_code: Some(first_code),
            account_context: Some(first_account.clone()),
            subject_cid_numbers,
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
                votingengine::limit::try_add_active_proposals::<T>(proposal.subject_keys(), id)
            {
                return TransactionOutcome::Rollback(Err(err));
            }
            // 立法提案可能关联多机构,互斥锁以所有关联 CID 为主体占用。
            for subject in proposal.subject_keys() {
                if let Err(err) = <votingengine::Pallet<T>>::acquire_internal_proposal_mutex(
                    id,
                    subject,
                    InternalProposalMutexKind::Regular,
                ) {
                    return TransactionOutcome::Rollback(Err(err));
                }
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
                    needs_guard,
                    referendum_scope,
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
                            p.account_context = Some(next_account.clone());
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
            Self::finalize_or_guard(proposal_id, meta.needs_guard)
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
        // 三人(院长+参议长+众议长)全批准 → 生效(修宪则转护宪终审)。
        if approvals >= 3 {
            Self::finalize_or_guard(proposal_id, meta.needs_guard)
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
            Self::finalize_or_guard(proposal_id, meta.needs_guard)
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

    /// 成功终态统一出口:修宪(needs_guard)→ 进护宪大法官终审;否则直接 PASSED。
    fn finalize_or_guard(proposal_id: u64, needs_guard: bool) -> DispatchResult {
        if needs_guard {
            Self::advance_to_guard(proposal_id)
        } else {
            <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, STATUS_PASSED)
        }
    }

    /// 修宪现有流程通过 → 进入护宪大法官终审阶段(宪法第21条)。
    fn advance_to_guard(proposal_id: u64) -> DispatchResult {
        pallet::LegGuardSigns::<T>::remove(proposal_id);
        Self::transition_stage(proposal_id, STAGE_LEG_CONSTITUTION_GUARD)?;
        Self::deposit_event(pallet::Event::<T>::LegislationAdvancedToGuard { proposal_id });
        Ok(())
    }

    /// 护宪大法官终审表决(仅修宪):7 人一人一票,4 名及以上赞成→生效;4 名及以上反对→否决。
    pub fn do_guard_vote(who: T::AccountId, proposal_id: u64, approve: bool) -> DispatchResult {
        let proposal =
            Proposals::<T>::get(proposal_id).ok_or(votingengine::Error::<T>::ProposalNotFound)?;
        ensure!(
            proposal.status == STATUS_VOTING,
            Error::<T>::NotInExpectedStage
        );
        ensure!(
            proposal.stage == STAGE_LEG_CONSTITUTION_GUARD,
            Error::<T>::NotInExpectedStage
        );
        let members =
            <T as votingengine::Config>::InternalAdminProvider::constitution_guard_members();
        ensure!(
            members.len() == CONSTITUTION_GUARD_MEMBERS as usize,
            Error::<T>::InvalidGuardMembersLen
        );
        for (idx, member) in members.iter().enumerate() {
            ensure!(
                !members.iter().skip(idx + 1).any(|other| other == member),
                Error::<T>::InvalidGuardMembersLen
            );
        }
        ensure!(
            members.iter().any(|m| m == &who),
            Error::<T>::NotConstitutionGuard
        );
        let mut signs = pallet::LegGuardSigns::<T>::get(proposal_id);
        ensure!(
            !signs.iter().any(|(s, _)| s == &who),
            Error::<T>::AlreadySigned
        );
        Self::deposit_event(pallet::Event::<T>::LegislationGuardVoted {
            proposal_id,
            who: who.clone(),
            approve,
        });
        signs
            .try_push((who, approve))
            .map_err(|_| Error::<T>::AlreadySigned)?;
        let yes = signs.iter().filter(|(_, a)| *a).count();
        let no = signs.iter().filter(|(_, a)| !*a).count();
        pallet::LegGuardSigns::<T>::insert(proposal_id, signs);
        if yes >= CONSTITUTION_GUARD_APPROVAL_THRESHOLD {
            // 7 人多数通过:4 名及以上赞成 → 生效。
            <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, STATUS_PASSED)
        } else if no >= CONSTITUTION_GUARD_APPROVAL_THRESHOLD {
            // 4 名及以上反对 → 已不可能达到 4 名赞成,否决。
            <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, STATUS_REJECTED)
        } else {
            Ok(())
        }
    }

    /// 护宪大法官终审超时:未获4名及以上赞成 → 否决。
    pub fn do_finalize_guard_timeout(
        proposal: &Proposal<frame_system::pallet_prelude::BlockNumberFor<T>, T::AccountId>,
        proposal_id: u64,
    ) -> DispatchResult {
        ensure!(
            proposal.stage == STAGE_LEG_CONSTITUTION_GUARD,
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

    /// 公投投票:读取链上公民身份资格 + 按账户去重计票(期满计票,本入口不提前判定)。
    pub fn do_cast_referendum_vote(
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
            proposal.stage == STAGE_LEG_REFERENDUM,
            votingengine::Error::<T>::InvalidProposalStage
        );
        ensure!(
            proposal.citizen_eligible_total > 0,
            Error::<T>::CitizenEligibleTotalNotSet
        );
        let meta = pallet::LegMeta::<T>::get(proposal_id).ok_or(Error::<T>::ProposalMetaMissing)?;
        let scope = meta
            .referendum_scope
            .ok_or(Error::<T>::PopulationScopeMissing)?;
        ensure!(
            <T as votingengine::Config>::CitizenIdentityReader::can_vote(&who, &scope),
            Error::<T>::CitizenNotEligible
        );
        ensure!(
            !pallet::LegReferendumVotesByAccount::<T>::contains_key(proposal_id, &who),
            votingengine::Error::<T>::AlreadyVoted
        );

        pallet::LegReferendumVotesByAccount::<T>::insert(proposal_id, &who, approve);
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
        if legislation_referendum_final_passed(proposal.citizen_eligible_total, tally.yes, tally.no)
        {
            // 公投通过:修宪(特别案)转护宪大法官终审,否则直接生效。
            let meta =
                pallet::LegMeta::<T>::get(proposal_id).ok_or(Error::<T>::ProposalMetaMissing)?;
            Self::finalize_or_guard(proposal_id, meta.needs_guard)
        } else {
            <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, STATUS_REJECTED)
        }
    }
}
// trait 实现(供 votingengine 核心 + 业务壳接入)
impl<T: Config> votingengine::LegislationVoteEngine<T::AccountId> for Pallet<T> {
    fn create_legislation_proposal(
        who: T::AccountId,
        houses: sp_runtime::sp_std::vec::Vec<(InstitutionCode, T::AccountId)>,
        vote_type: u8,
        executive: (InstitutionCode, T::AccountId),
        legislature: Option<(InstitutionCode, T::AccountId)>,
        needs_guard: bool,
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
                needs_guard,
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

    fn finalize_legislation_guard_timeout(
        proposal: &Proposal<frame_system::pallet_prelude::BlockNumberFor<T>, T::AccountId>,
        proposal_id: u64,
    ) -> DispatchResult {
        Self::do_finalize_guard_timeout(proposal, proposal_id)
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
            pallet::LegReferendumVotesByAccount::<T>::clear_prefix(proposal_id, limit, None);
        (result.unique, result.maybe_cursor.is_some())
    }

    fn cleanup_legislation_terminal(proposal_id: u64) {
        pallet::LegMeta::<T>::remove(proposal_id);
        pallet::LegHouseTally::<T>::remove(proposal_id);
        pallet::LegReferendumTally::<T>::remove(proposal_id);
        pallet::LegOverrideSigns::<T>::remove(proposal_id);
        pallet::LegGuardSigns::<T>::remove(proposal_id);
    }
}
