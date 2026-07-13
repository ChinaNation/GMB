//! # 立法投票 pallet (legislation-vote)
//!
//! 立法机构专属投票模式(ADR-027,公民宪法第45/46条)。投票引擎「头等模式」:
//! `PROPOSAL_KIND_LEGISLATION`,共享核心 `votingengine`(Proposals/AdminSnapshot/状态机/
//! 公投快照验签/清理/反向索引),只本地保管计票账本。内部/联合/选举投票
//! sub-pallet 逻辑零改动。
//!
//! 阶段(ADR-027,当前五类提案 + 特别案公投 + 行政签署/三人会签/护宪终审):
//! - `STAGE_LEG_REPRESENTATIVE` 代表表决：单机构一段，多机构按声明顺序逐段推进。
//! - `STAGE_LEG_REFERENDUM` 强制公投:仅特别案(含核心修宪),内部全过后强制进入,公投通过即生效不签署。
//! - `STAGE_LEG_SIGN` 行政签署:非特别案内部全过后,行政机构法定代表人(市长/省长/总统)签署。
//!   市行政区无救济(否决=否决/30天超时=通过);省行政区/国家否决或超时 → 会签。
//! - `STAGE_LEG_OVERRIDE` 三人会签(省行政区/国家):立法院院长 + 参议长 + 众议长,全签=生效/任一否决或超时=否决。
//!
//! 计票口径:按现任议员/委员管理员快照总数算参与率/赞成率(`votingengine::types`
//! 的立法阈值纯函数),投票期满 finalize 统一判定;结果已确定时可提前决。

#![cfg_attr(not(feature = "std"), no_std)]

pub mod cleanup;
pub mod legislation;
pub mod representative;
pub mod result;
pub mod rules;
pub mod types;
pub mod weights;

#[cfg(test)]
mod tests;

pub use pallet::*;
pub use types::{
    LegislationProcedureConfig, LegislationVoteEngine, RepresentativeBodies, RepresentativeRoute,
    RepresentativeVoteRule, VoteProcedure, MAX_REPRESENTATIVE_BODIES,
};

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
    types::{InstitutionCode, ProposalSubjectCidNumbers},
    CitizenIdentityReader, InternalAdminProvider, InternalProposalMutexKind, PopulationScope,
    Proposal, PROPOSAL_KIND_LEGISLATION, STAGE_LEG_CONSTITUTION_GUARD, STAGE_LEG_OVERRIDE,
    STAGE_LEG_REFERENDUM, STAGE_LEG_REPRESENTATIVE, STAGE_LEG_SIGN, STATUS_PASSED, STATUS_REJECTED,
    STATUS_VOTING,
};

use crate::rules::{referendum_final_passed, representative_decided, representative_final_passed};

/// 法律全文大对象类型标记(写入 votingengine `ProposalObject`),与 legislation-yuan 对齐。
pub const PROPOSAL_OBJECT_KIND_LAW_TEXT: u8 = 2;

/// 护宪大法官法定人数(宪法第20条):7 人。
pub const CONSTITUTION_GUARD_MEMBERS: u32 = 7;

/// 修宪终审通过阈值(宪法第21条):7 人多数通过,即 4 名及以上护宪大法官赞成。
/// 口径单源在 `primitives::constitution::CONSTITUTION_GUARD_APPROVAL_THRESHOLD`(与节点守卫共用)。
pub const CONSTITUTION_GUARD_APPROVAL_THRESHOLD: usize =
    primitives::constitution::CONSTITUTION_GUARD_APPROVAL_THRESHOLD as usize;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    /// v2：代表表决元数据与法律专属元数据拆分，旧布局不迁移，开发期重新创世。
    pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

    /// 代表机构表决元数据。所有法律、任免、预算等立法机关表决共用这一份状态。
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
    pub struct RepresentativeMeta<T: Config> {
        /// 单机构或多机构顺序表决路线。
        pub route: RepresentativeRoute<T::AccountId>,
        /// 当前正在表决的机构索引。
        pub current_body: u32,
        /// 常规、重要或特别三种数学门槛。
        pub rule: RepresentativeVoteRule,
        /// 代表表决完成后直接终局，或继续法律专属程序。
        pub procedure: VoteProcedure,
    }

    /// 法律专属元数据。任免和预算提案不得创建本记录。
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
        /// 行政签署机构(总统府/省联邦政府/市政府);其法定代表人=总统/省长/市长。非特别案末段签署。
        pub executive: (InstitutionCode, T::AccountId),
        /// 两院级的立法院机构(国家/省立法院);其法定代表人=院长,供三人会签。单院(市)=None。
        pub legislature: Option<(InstitutionCode, T::AccountId)>,
        /// 是否修宪(tier=宪法):为真时,现有流程通过后最后进护宪大法官终审(宪法第21条)。
        pub needs_guard: bool,
        /// 特别案公投作用域；常规案和重要案为 None。
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

    /// 代表机构表决元数据：proposal_id → RepresentativeMeta。
    #[pallet::storage]
    pub type RepresentativeMetas<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, RepresentativeMeta<T>, OptionQuery>;

    /// 法律专属元数据：只有 `VoteProcedure::Legislation` 提案存在。
    #[pallet::storage]
    pub type LegislationMetas<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, LegislationMeta<T>, OptionQuery>;

    /// 每个代表机构阶段独立计票：(proposal_id, body_index) → yes/no。
    #[pallet::storage]
    pub type RepresentativeTallies<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        u32,
        votingengine::VoteCountU32,
        ValueQuery,
    >;

    /// 代表表决去重：(proposal_id, (body_index, account)) → 赞成/反对。
    /// `body_index` 允许同一账户在不同机构阶段分别依法投票。
    #[pallet::storage]
    pub type RepresentativeVotesByAccount<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        (u32, T::AccountId),
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

    /// 三人会签记录(省行政区/国家 STAGE_LEG_OVERRIDE):proposal_id → [(签署人, 是否赞成)],
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
        /// 立法机关代表表决提案已创建。
        RepresentativeProposalCreated {
            proposal_id: u64,
            rule: RepresentativeVoteRule,
            bodies: u32,
            procedure: VoteProcedure,
        },
        /// 某议员/委员投出一票。
        RepresentativeVoteCast {
            proposal_id: u64,
            body_index: u32,
            who: T::AccountId,
            approve: bool,
        },
        /// 当前代表机构通过，推进至下一代表机构。
        RepresentativeBodyAdvanced { proposal_id: u64, next_body: u32 },
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
        /// 退回立法院三人会签阶段(省行政区/国家)。
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
        /// 代表机构路线为空、重复、单机构误用顺序路线或超过上限。
        InvalidRepresentativeRoute,
        /// 特别门槛与直接终局程序组合非法。
        InvalidRepresentativeRule,
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

        /// 管理员按当前代表机构席位投票。
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::cast_representative_vote())]
        pub fn cast_representative_vote(
            origin: OriginFor<T>,
            proposal_id: u64,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_cast_representative_vote(who, proposal_id, approve)
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
        /// 批准=生效;否决:市行政区=否决,省行政区/国家=退回三人会签。
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::executive_sign())]
        pub fn executive_sign(
            origin: OriginFor<T>,
            proposal_id: u64,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_executive_sign(who, proposal_id, approve)
        }

        /// 三人会签(省行政区/国家:立法院院长 + 参议长 + 众议长)签署或否决。
        /// 三人全批准=生效;任一否决=否决。
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::override_sign())]
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
        #[pallet::weight(<T as Config>::WeightInfo::guard_vote())]
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
        route: &RepresentativeRoute<T::AccountId>,
        additional_subjects: ProposalSubjectCidNumbers,
        additional_institutions: &[(InstitutionCode, T::AccountId)],
    ) -> Result<ProposalSubjectCidNumbers, DispatchError> {
        let mut raw: sp_runtime::sp_std::vec::Vec<sp_runtime::sp_std::vec::Vec<u8>> =
            additional_subjects
                .into_iter()
                .map(|cid| cid.into_inner())
                .collect();
        for (_, account) in route.bodies() {
            Self::push_subject_cid(&mut raw, &account)?;
        }
        for (_, account) in additional_institutions {
            Self::push_subject_cid(&mut raw, account)?;
        }
        <votingengine::Pallet<T>>::bound_subject_cid_numbers(raw)
    }

    /// 校验路线并返回首个表决机构。路线中的机构不得重复。
    fn validate_representative_route(
        route: &RepresentativeRoute<T::AccountId>,
    ) -> Result<(InstitutionCode, T::AccountId), DispatchError> {
        let bodies = route.bodies();
        ensure!(!bodies.is_empty(), Error::<T>::InvalidRepresentativeRoute);
        match route {
            RepresentativeRoute::Single(_) => {}
            RepresentativeRoute::Sequential(sequence) => ensure!(
                sequence.len() >= 2 && sequence.len() <= MAX_REPRESENTATIVE_BODIES as usize,
                Error::<T>::InvalidRepresentativeRoute
            ),
        }
        for (index, body) in bodies.iter().enumerate() {
            ensure!(
                !bodies[..index].iter().any(|existing| existing == body),
                Error::<T>::InvalidRepresentativeRoute
            );
        }
        Ok(bodies[0].clone())
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

    /// 创建通用代表机构表决提案。业务模块只提供路线、门槛、受影响主体和 owner 数据。
    #[allow(clippy::too_many_arguments)]
    pub fn do_create_representative_proposal(
        who: T::AccountId,
        route: RepresentativeRoute<T::AccountId>,
        rule: RepresentativeVoteRule,
        procedure: VoteProcedure,
        additional_subjects: ProposalSubjectCidNumbers,
        mut legislation_meta: Option<pallet::LegislationMeta<T>>,
    ) -> Result<u64, DispatchError> {
        let (first_code, first_account) = Self::validate_representative_route(&route)?;
        ensure!(
            !(procedure == VoteProcedure::RepresentativeOnly
                && rule == RepresentativeVoteRule::Special),
            Error::<T>::InvalidRepresentativeRule
        );
        ensure!(
            (procedure == VoteProcedure::Legislation) == legislation_meta.is_some(),
            Error::<T>::ProposalMetaMissing
        );

        let mut additional_institutions = sp_runtime::sp_std::vec::Vec::new();
        if let Some(meta) = legislation_meta.as_ref() {
            additional_institutions.push(meta.executive.clone());
            if let Some(legislature) = meta.legislature.as_ref() {
                additional_institutions.push(legislature.clone());
            }
        }
        let subject_cid_numbers = Self::resolve_subject_cid_numbers(
            &route,
            additional_subjects,
            &additional_institutions,
        )?;

        let now = <frame_system::Pallet<T>>::block_number();
        // 特别案消费同一区块准备的人口快照；普通和重要案不得残留公投作用域。
        let eligible_total = if rule == RepresentativeVoteRule::Special {
            let prepared = pallet::PendingPopulationSnapshots::<T>::get(&who)
                .ok_or(Error::<T>::PopulationSnapshotNotPrepared)?;
            if prepared.prepared_at != now {
                pallet::PendingPopulationSnapshots::<T>::remove(&who);
                return Err(Error::<T>::PopulationSnapshotNotCurrent.into());
            }
            let meta = legislation_meta
                .as_mut()
                .ok_or(Error::<T>::ProposalMetaMissing)?;
            meta.referendum_scope = Some(prepared.scope);
            prepared.eligible_total
        } else {
            if let Some(meta) = legislation_meta.as_mut() {
                meta.referendum_scope = None;
            }
            0
        };

        let end = now.saturating_add(Self::stage_duration());
        let proposal = Proposal {
            kind: PROPOSAL_KIND_LEGISLATION,
            stage: STAGE_LEG_REPRESENTATIVE,
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
            if rule == RepresentativeVoteRule::Special {
                pallet::PendingPopulationSnapshots::<T>::remove(&who);
            }
            pallet::RepresentativeMetas::<T>::insert(
                id,
                pallet::RepresentativeMeta {
                    route,
                    current_body: 0,
                    rule,
                    procedure,
                },
            );
            if let Some(meta) = legislation_meta {
                pallet::LegislationMetas::<T>::insert(id, meta);
            }
            Proposals::<T>::insert(id, proposal);
            if let Err(err) = <votingengine::Pallet<T>>::schedule_proposal_expiry(id, end) {
                return TransactionOutcome::Rollback(Err(err));
            }
            <votingengine::Pallet<T>>::emit_proposal_created(
                id,
                PROPOSAL_KIND_LEGISLATION,
                STAGE_LEG_REPRESENTATIVE,
                end,
            );
            Self::deposit_event(pallet::Event::<T>::RepresentativeProposalCreated {
                proposal_id: id,
                rule,
                bodies: pallet::RepresentativeMetas::<T>::get(id)
                    .map(|meta| meta.route.len() as u32)
                    .unwrap_or_default(),
                procedure,
            });
            TransactionOutcome::Commit(Ok(id))
        })
    }

    /// 对当前代表机构投票；同一账户可在不同机构阶段分别依法投票。
    pub fn do_cast_representative_vote(
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
            proposal.stage == STAGE_LEG_REPRESENTATIVE,
            votingengine::Error::<T>::InvalidProposalStage
        );
        let meta = pallet::RepresentativeMetas::<T>::get(proposal_id)
            .ok_or(Error::<T>::ProposalMetaMissing)?;
        let (_code, institution) = meta
            .route
            .body(meta.current_body)
            .ok_or(Error::<T>::InvalidRepresentativeRoute)?;
        let vote_key = (meta.current_body, who.clone());
        ensure!(
            !pallet::RepresentativeVotesByAccount::<T>::contains_key(proposal_id, &vote_key),
            votingengine::Error::<T>::AlreadyVoted
        );
        ensure!(
            <votingengine::Pallet<T>>::is_admin_in_snapshot(proposal_id, institution.clone(), &who),
            votingengine::Error::<T>::NoPermission
        );

        pallet::RepresentativeVotesByAccount::<T>::insert(proposal_id, vote_key, approve);
        let tally =
            pallet::RepresentativeTallies::<T>::mutate(proposal_id, meta.current_body, |t| {
                if approve {
                    t.yes = t.yes.saturating_add(1);
                } else {
                    t.no = t.no.saturating_add(1);
                }
                *t
            });
        Self::deposit_event(pallet::Event::<T>::RepresentativeVoteCast {
            proposal_id,
            body_index: meta.current_body,
            who,
            approve,
        });

        let admins_len = <votingengine::Pallet<T>>::snapshot_admins_len(proposal_id, institution)
            .ok_or(votingengine::Error::<T>::MissingAdminSnapshot)?;
        match representative_decided(meta.rule, admins_len, tally.yes, tally.no) {
            Some(true) => match meta.route {
                RepresentativeRoute::Single(_) => {
                    Self::finish_single_representative_vote(proposal_id)
                }
                RepresentativeRoute::Sequential(_) => {
                    Self::advance_sequential_representative_vote(proposal_id)
                }
            },
            Some(false) => {
                <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, STATUS_REJECTED)
            }
            None => Ok(()),
        }
    }

    /// 顺序路线推进至下一个代表机构；全部完成后进入配置的后续程序。
    pub(crate) fn advance_representative_body_or_finish(proposal_id: u64) -> DispatchResult {
        let meta = pallet::RepresentativeMetas::<T>::get(proposal_id)
            .ok_or(Error::<T>::ProposalMetaMissing)?;
        let next = meta.current_body.saturating_add(1);
        if (next as usize) < meta.route.len() {
            let (next_code, next_account) = meta
                .route
                .body(next)
                .ok_or(Error::<T>::InvalidRepresentativeRoute)?;
            let now = <frame_system::Pallet<T>>::block_number();
            let end = now.saturating_add(Self::stage_duration());
            with_transaction(|| {
                // 各机构计票按 body_index 永久隔离到提案清理，不删除前一阶段审计记录。
                pallet::RepresentativeMetas::<T>::mutate(proposal_id, |maybe| {
                    if let Some(m) = maybe {
                        m.current_body = next;
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
                Self::deposit_event(pallet::Event::<T>::RepresentativeBodyAdvanced {
                    proposal_id,
                    next_body: next,
                });
                TransactionOutcome::Commit(Ok(()))
            })
        } else {
            Self::finish_representative_route(proposal_id)
        }
    }

    /// 所有代表机构通过后按强类型程序进入终局或法律专属阶段。
    pub(crate) fn finish_representative_route(proposal_id: u64) -> DispatchResult {
        let meta = pallet::RepresentativeMetas::<T>::get(proposal_id)
            .ok_or(Error::<T>::ProposalMetaMissing)?;
        match meta.procedure {
            VoteProcedure::RepresentativeOnly => {
                <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, STATUS_PASSED)
            }
            VoteProcedure::Legislation if meta.rule == RepresentativeVoteRule::Special => {
                Self::advance_to_referendum(proposal_id)
            }
            VoteProcedure::Legislation => Self::advance_to_sign(proposal_id),
        }
    }

    /// 内部全过 → 推进至强制公投阶段(对标 joint advance_to_referendum)。
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
            <votingengine::Pallet<T>>::emit_proposal_advanced_to_referendum(
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

    /// 行政首长否决/超时(省行政区/国家) → 退回立法院三人会签阶段。
    fn advance_to_override(proposal_id: u64) -> DispatchResult {
        pallet::LegOverrideSigns::<T>::remove(proposal_id);
        Self::transition_stage(proposal_id, STAGE_LEG_OVERRIDE)?;
        Self::deposit_event(pallet::Event::<T>::LegislationAdvancedToOverride { proposal_id });
        Ok(())
    }

    /// 实时查机构法定代表人(机构首脑;ADR-027 签署人)。
    fn legal_representative_of(body: &(InstitutionCode, T::AccountId)) -> Option<T::AccountId> {
        <T as votingengine::Config>::InternalAdminProvider::legal_representative(
            body.0,
            body.1.clone(),
        )
    }

    /// 行政签署:机构法定代表人(市长/省长/总统)批准=生效;否决:市行政区=否决/省行政区/国家=退回会签。
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
        let meta = pallet::LegislationMetas::<T>::get(proposal_id)
            .ok_or(Error::<T>::ProposalMetaMissing)?;
        let rep = Self::legal_representative_of(&meta.executive)
            .ok_or(Error::<T>::NotLegalRepresentative)?;
        ensure!(who == rep, Error::<T>::NotLegalRepresentative);
        Self::deposit_event(pallet::Event::<T>::LegislationExecutiveSigned {
            proposal_id,
            who,
            approve,
        });
        if approve {
            Self::finalize_or_guard(proposal_id, meta.needs_guard)
        } else if meta.legislature.is_some() {
            // 省行政区/国家:否决 → 退回三人会签救济。
            Self::advance_to_override(proposal_id)
        } else {
            // 市行政区:无救济,否决即否决。
            <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, STATUS_REJECTED)
        }
    }

    /// 三人会签合法身份(院长 + 众议长 + 参议长 = 立法院/众议会/参议会三机构法定代表人)。
    fn override_signers_for_proposal(
        proposal_id: u64,
        meta: &pallet::LegislationMeta<T>,
    ) -> sp_runtime::sp_std::vec::Vec<T::AccountId> {
        let mut out = sp_runtime::sp_std::vec::Vec::new();
        if let Some(leg) = meta.legislature.as_ref() {
            if let Some(rep) = Self::legal_representative_of(leg) {
                out.push(rep);
            }
        }
        let Some(representative) = pallet::RepresentativeMetas::<T>::get(proposal_id) else {
            return out;
        };
        for body in representative.route.bodies() {
            if let Some(rep) = Self::legal_representative_of(&body) {
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
        let meta = pallet::LegislationMetas::<T>::get(proposal_id)
            .ok_or(Error::<T>::ProposalMetaMissing)?;
        let signers = Self::override_signers_for_proposal(proposal_id, &meta);
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

    /// 行政签署阶段超时:市行政区(无 legislature)= 视为通过;省行政区/国家 = 退回三人会签。
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
        let meta = pallet::LegislationMetas::<T>::get(proposal_id)
            .ok_or(Error::<T>::ProposalMetaMissing)?;
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

    /// 当前代表机构阶段超时结算：按强类型门槛计票，通过则推进，否则否决。
    pub fn do_finalize_representative_timeout(
        proposal: &Proposal<frame_system::pallet_prelude::BlockNumberFor<T>, T::AccountId>,
        proposal_id: u64,
    ) -> DispatchResult {
        ensure!(
            proposal.stage == STAGE_LEG_REPRESENTATIVE,
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
        let meta = pallet::RepresentativeMetas::<T>::get(proposal_id)
            .ok_or(Error::<T>::ProposalMetaMissing)?;
        let (_code, institution) = meta
            .route
            .body(meta.current_body)
            .ok_or(Error::<T>::InvalidRepresentativeRoute)?;
        let admins_len = <votingengine::Pallet<T>>::snapshot_admins_len(proposal_id, institution)
            .ok_or(votingengine::Error::<T>::MissingAdminSnapshot)?;
        let tally = pallet::RepresentativeTallies::<T>::get(proposal_id, meta.current_body);
        if representative_final_passed(meta.rule, admins_len, tally.yes, tally.no) {
            match meta.route {
                RepresentativeRoute::Single(_) => {
                    Self::finish_single_representative_vote(proposal_id)
                }
                RepresentativeRoute::Sequential(_) => {
                    Self::advance_sequential_representative_vote(proposal_id)
                }
            }
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
        let meta = pallet::LegislationMetas::<T>::get(proposal_id)
            .ok_or(Error::<T>::ProposalMetaMissing)?;
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
        if referendum_final_passed(proposal.citizen_eligible_total, tally.yes, tally.no) {
            // 公投通过:修宪(特别案)转护宪大法官终审,否则直接生效。
            let meta = pallet::LegislationMetas::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalMetaMissing)?;
            Self::finalize_or_guard(proposal_id, meta.needs_guard)
        } else {
            <votingengine::Pallet<T>>::set_status_and_emit(proposal_id, STATUS_REJECTED)
        }
    }
}
// trait 实现(供 votingengine 核心 + 业务壳接入)
impl<T: Config> crate::LegislationVoteEngine<T::AccountId> for Pallet<T> {
    fn create_representative_vote(
        who: T::AccountId,
        route: RepresentativeRoute<T::AccountId>,
        rule: RepresentativeVoteRule,
        subject_cid_numbers: ProposalSubjectCidNumbers,
        module_tag: &[u8],
        data: sp_runtime::sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        let first_account = Self::validate_representative_route(&route)?.1;
        with_transaction(|| {
            let id = match Self::do_create_representative_proposal(
                who.clone(),
                route,
                rule,
                VoteProcedure::RepresentativeOnly,
                subject_cid_numbers,
                None,
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
            if <votingengine::Pallet<T>>::is_admin_in_snapshot(id, first_account, &who) {
                match Self::do_cast_representative_vote(who, id, true) {
                    Ok(()) => TransactionOutcome::Commit(Ok(id)),
                    Err(err) => TransactionOutcome::Rollback(Err(err)),
                }
            } else {
                TransactionOutcome::Commit(Ok(id))
            }
        })
    }

    fn create_legislation_vote(
        who: T::AccountId,
        route: RepresentativeRoute<T::AccountId>,
        rule: RepresentativeVoteRule,
        procedure: LegislationProcedureConfig<T::AccountId>,
        module_tag: &[u8],
        data: sp_runtime::sp_std::vec::Vec<u8>,
        object_data: sp_runtime::sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        let first_account = Self::validate_representative_route(&route)?.1;
        with_transaction(|| {
            let id = match Self::do_create_representative_proposal(
                who.clone(),
                route,
                rule,
                VoteProcedure::Legislation,
                ProposalSubjectCidNumbers::new(),
                Some(pallet::LegislationMeta {
                    executive: procedure.executive,
                    legislature: procedure.legislature,
                    needs_guard: procedure.needs_guard,
                    referendum_scope: None,
                }),
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
            // 市行政区 市自治会/市教委会 委员提案时发起人不在表决院(市立法会),不自动投票。
            if <votingengine::Pallet<T>>::is_admin_in_snapshot(id, first_account, &who) {
                match Self::do_cast_representative_vote(who, id, true) {
                    Ok(()) => TransactionOutcome::Commit(Ok(id)),
                    Err(err) => TransactionOutcome::Rollback(Err(err)),
                }
            } else {
                TransactionOutcome::Commit(Ok(id))
            }
        })
    }

    /// 读取某立法提案的强制公投结果 `(eligible, yes, no)`。
    /// 无公投分母(`citizen_eligible_total==0`,即非特别案)或提案不存在 → `None`。
    /// 公投计票 `LegReferendumTally` 在提案 90 天清理前一直保留,故核心修宪写入(护宪终审同块)时可读到。
    fn referendum_result(proposal_id: u64) -> Option<(u64, u64, u64)> {
        let eligible = <votingengine::Pallet<T>>::citizen_eligible_total_of(proposal_id)?;
        if eligible == 0 {
            return None;
        }
        let tally = pallet::LegReferendumTally::<T>::get(proposal_id);
        Some((eligible, tally.yes, tally.no))
    }

    /// 读取某修宪提案的护宪大法官终审赞成票数;无终审记录(非修宪 / 未进终审)→ `None`。
    /// 记录 `LegGuardSigns` 在提案 90 天清理前保留,故写入版本(终审通过同块)时可读到。
    fn guard_review_result(proposal_id: u64) -> Option<u32> {
        let signs = pallet::LegGuardSigns::<T>::get(proposal_id);
        if signs.is_empty() {
            return None;
        }
        Some(signs.iter().filter(|(_, approve)| *approve).count() as u32)
    }
}

impl<T: Config>
    votingengine::traits::LegislationProposalFinalizer<
        frame_system::pallet_prelude::BlockNumberFor<T>,
        T::AccountId,
    > for Pallet<T>
{
    fn finalize_legislation_representative_timeout(
        proposal: &Proposal<frame_system::pallet_prelude::BlockNumberFor<T>, T::AccountId>,
        proposal_id: u64,
    ) -> DispatchResult {
        Self::do_finalize_representative_timeout(proposal, proposal_id)
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
    fn cleanup_legislation_representative_votes_chunk(
        proposal_id: u64,
        limit: u32,
    ) -> votingengine::traits::CleanupChunkResult {
        let result =
            pallet::RepresentativeVotesByAccount::<T>::clear_prefix(proposal_id, limit, None);
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
        pallet::RepresentativeMetas::<T>::remove(proposal_id);
        pallet::LegislationMetas::<T>::remove(proposal_id);
        let _ = pallet::RepresentativeTallies::<T>::clear_prefix(
            proposal_id,
            MAX_REPRESENTATIVE_BODIES,
            None,
        );
        pallet::LegReferendumTally::<T>::remove(proposal_id);
        pallet::LegOverrideSigns::<T>::remove(proposal_id);
        pallet::LegGuardSigns::<T>::remove(proposal_id);
    }
}
