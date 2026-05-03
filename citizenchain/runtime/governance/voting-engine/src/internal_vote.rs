use frame_support::{
    ensure,
    pallet_prelude::{BoundedVec, DispatchResult},
    storage::{with_transaction, TransactionOutcome},
};
use sp_runtime::traits::{SaturatedConversion, Saturating};

use primitives::china::china_cb::{shenfen_id_to_fixed48 as reserve_pallet_id_to_bytes, CHINA_CB};
use primitives::china::china_ch::{
    shenfen_id_to_fixed48 as shengbank_pallet_id_to_bytes, CHINA_CH,
};
use primitives::count_const::{
    NRC_INTERNAL_THRESHOLD, PRB_INTERNAL_THRESHOLD, PRC_INTERNAL_THRESHOLD, VOTING_DURATION_BLOCKS,
};

use crate::{
    pallet::{
        AdminSnapshot, Config, Error, Event, InternalTallies, InternalThresholdSnapshot,
        InternalVotesByAccount, Pallet, Proposals,
    },
    InstitutionPalletId, InternalAdminProvider, InternalProposalMutexKind,
    InternalThresholdProvider, Proposal, PROPOSAL_KIND_INTERNAL, STAGE_INTERNAL, STATUS_PASSED,
    STATUS_REJECTED,
};

pub const ORG_NRC: u8 = 0;
pub const ORG_PRC: u8 = 1;
pub const ORG_PRB: u8 = 2;
/// 注册多签/个人多签主体，管理员与阈值由 admins-change 统一主体表提供。
pub const ORG_DUOQIAN: u8 = 3;

pub fn is_valid_org(org: u8) -> bool {
    matches!(org, ORG_NRC | ORG_PRC | ORG_PRB | ORG_DUOQIAN)
}

/// 治理机构（NRC/PRC/PRB）的固定制度阈值。
/// 中文注释：国储会、省储会、省储行阈值是永久治理常量，不读取注册多签主体配置。
pub fn fixed_governance_pass_threshold(org: u8) -> Option<u32> {
    match org {
        ORG_NRC => Some(NRC_INTERNAL_THRESHOLD),
        ORG_PRC => Some(PRC_INTERNAL_THRESHOLD),
        ORG_PRB => Some(PRB_INTERNAL_THRESHOLD),
        _ => None,
    }
}

use crate::nrc_pallet_id_bytes;

fn is_valid_internal_institution<T: Config>(
    org: u8,
    institution: InstitutionPalletId,
    pending_subject: bool,
) -> bool {
    // 中文注释：内部投票里的 institution 必须与 org 类型严格对应，避免伪造”跨组织机构”。
    match org {
        // 国储会只有一个机构
        ORG_NRC => {
            !pending_subject
                && nrc_pallet_id_bytes()
                    .map(|nrc| institution == nrc)
                    .unwrap_or(false)
        }
        // 省储会从 CHINA_CB 中排除国储会
        ORG_PRC => {
            !pending_subject
                && CHINA_CB
                    .iter()
                    .skip(1)
                    .filter_map(|n| reserve_pallet_id_to_bytes(n.shenfen_id))
                    .any(|pid| pid == institution)
        }
        // 省储行从 CHINA_CH 获取
        ORG_PRB => {
            !pending_subject
                && CHINA_CH
                    .iter()
                    .filter_map(|n| shengbank_pallet_id_to_bytes(n.shenfen_id))
                    .any(|pid| pid == institution)
        }
        // 注册多签/个人多签主体：按入口语义分别查询 Active 或 Pending 主体是否存在。
        ORG_DUOQIAN if pending_subject => {
            T::InternalThresholdProvider::is_known_pending_subject(org, institution)
        }
        ORG_DUOQIAN => T::InternalThresholdProvider::is_known_subject(org, institution),
        _ => false,
    }
}

fn is_internal_admin<T: Config>(
    org: u8,
    institution: InstitutionPalletId,
    who: &T::AccountId,
    pending_subject: bool,
) -> bool {
    if pending_subject {
        return T::InternalAdminProvider::is_pending_internal_admin(org, institution, who);
    }

    // 中文注释：生产环境仅信任动态管理员来源（链上治理替换后的最终状态）。
    T::InternalAdminProvider::is_internal_admin(org, institution, who)
}

fn internal_threshold<T: Config>(
    org: u8,
    institution: InstitutionPalletId,
    pending_subject: bool,
) -> Option<u32> {
    match org {
        ORG_NRC | ORG_PRC | ORG_PRB => {
            // 中文注释：三类治理机构阈值是制度常量，内部提案创建时也只快照固定值。
            fixed_governance_pass_threshold(org)
        }
        ORG_DUOQIAN if pending_subject => {
            // 中文注释：注册多签激活投票读取 Pending 主体阈值，并在创建时写入快照。
            T::InternalThresholdProvider::pending_pass_threshold(org, institution)
        }
        ORG_DUOQIAN => {
            // 中文注释：已激活注册多签读取主体配置阈值，并在创建时写入快照。
            T::InternalThresholdProvider::pass_threshold(org, institution)
        }
        _ => None,
    }
}

impl<T: Config> Pallet<T> {
    fn internal_stage_duration() -> frame_system::pallet_prelude::BlockNumberFor<T> {
        // 中文注释：内部投票与联合/公民投票共用统一治理时长常量，便于链上运维校准。
        (VOTING_DURATION_BLOCKS as u64).saturated_into()
    }

    pub(crate) fn do_create_internal_proposal(
        who: T::AccountId,
        org: u8,
        institution: InstitutionPalletId,
    ) -> Result<u64, sp_runtime::DispatchError> {
        Self::do_create_internal_proposal_with_subject_status(
            who,
            org,
            institution,
            false,
            InternalProposalMutexKind::Regular,
        )
    }

    pub(crate) fn do_create_pending_subject_internal_proposal(
        who: T::AccountId,
        org: u8,
        institution: InstitutionPalletId,
    ) -> Result<u64, sp_runtime::DispatchError> {
        Self::do_create_internal_proposal_with_subject_status(
            who,
            org,
            institution,
            true,
            InternalProposalMutexKind::Regular,
        )
    }

    pub(crate) fn do_create_pending_subject_internal_proposal_with_snapshot(
        who: T::AccountId,
        org: u8,
        institution: InstitutionPalletId,
        admins: sp_std::vec::Vec<T::AccountId>,
        threshold: u32,
    ) -> Result<u64, sp_runtime::DispatchError> {
        ensure!(org == ORG_DUOQIAN, Error::<T>::InvalidInternalOrg);
        ensure!(!admins.is_empty(), Error::<T>::MissingAdminSnapshot);
        ensure!(
            admins.iter().any(|admin| admin == &who),
            Error::<T>::NoPermission
        );
        for i in 0..admins.len() {
            for j in i.saturating_add(1)..admins.len() {
                ensure!(admins[i] != admins[j], Error::<T>::InvalidInstitution);
            }
        }
        let admin_count = admins.len() as u32;
        ensure!(
            threshold > 0 && threshold <= admin_count,
            Error::<T>::InvalidInternalOrg
        );
        let bounded_admins: BoundedVec<T::AccountId, T::MaxAdminsPerInstitution> = admins
            .try_into()
            .map_err(|_| Error::<T>::InvalidInstitution)?;

        let now = <frame_system::Pallet<T>>::block_number();
        let end = now.saturating_add(Self::internal_stage_duration());
        let proposal = Proposal {
            kind: PROPOSAL_KIND_INTERNAL,
            stage: STAGE_INTERNAL,
            status: crate::STATUS_VOTING,
            internal_org: Some(org),
            internal_institution: Some(institution),
            start: now,
            end,
            citizen_eligible_total: 0,
        };

        with_transaction(|| {
            let id = match Self::allocate_proposal_id() {
                Ok(id) => id,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };
            if let Err(err) =
                crate::active_proposal_limit::try_add_active_proposal::<T>(institution, id)
            {
                return TransactionOutcome::Rollback(Err(err));
            }
            if let Err(err) = Self::acquire_internal_proposal_mutex(
                id,
                org,
                institution,
                InternalProposalMutexKind::Regular,
            ) {
                return TransactionOutcome::Rollback(Err(err));
            }

            // 中文注释：Pending 主体尚未写入 admins-change 时，由业务入口提供待注册管理员快照。
            AdminSnapshot::<T>::insert(id, institution, bounded_admins);
            InternalThresholdSnapshot::<T>::insert(id, threshold);
            Proposals::<T>::insert(id, proposal);
            if let Err(err) = Self::schedule_proposal_expiry(id, end) {
                return TransactionOutcome::Rollback(Err(err));
            }
            Self::deposit_event(Event::<T>::ProposalCreated {
                proposal_id: id,
                kind: PROPOSAL_KIND_INTERNAL,
                stage: STAGE_INTERNAL,
                end,
            });
            TransactionOutcome::Commit(Ok(id))
        })
    }

    pub(crate) fn do_create_admin_set_mutation_internal_proposal(
        who: T::AccountId,
        org: u8,
        institution: InstitutionPalletId,
    ) -> Result<u64, sp_runtime::DispatchError> {
        Self::do_create_internal_proposal_with_subject_status(
            who,
            org,
            institution,
            false,
            InternalProposalMutexKind::AdminSetMutationExclusive,
        )
    }

    /// 创建普通内部提案,但**显式传 threshold**(不走 `internal_threshold` 反查)。
    ///
    /// 用于"主体生命周期"语义的内部提案——业务规则要求全员通过(threshold = admins.len()),
    /// 而不是用户自定义 m-of-n。admins 仍从 active 主体反查并写入 AdminSnapshot。
    ///
    /// 业务方在 ORG_DUOQIAN 的关闭场景调用,传 `subject.admins.len() as u32`。
    pub(crate) fn do_create_internal_proposal_with_explicit_threshold(
        who: T::AccountId,
        org: u8,
        institution: InstitutionPalletId,
        threshold: u32,
    ) -> Result<u64, sp_runtime::DispatchError> {
        ensure!(is_valid_org(org), Error::<T>::InvalidInternalOrg);
        ensure!(
            is_valid_internal_institution::<T>(org, institution, false),
            Error::<T>::InvalidInstitution
        );
        ensure!(
            is_internal_admin::<T>(org, institution, &who, false),
            Error::<T>::NoPermission
        );
        ensure!(threshold > 0, Error::<T>::InvalidInternalOrg);

        let now = <frame_system::Pallet<T>>::block_number();
        let end = now.saturating_add(Self::internal_stage_duration());

        let proposal = Proposal {
            kind: PROPOSAL_KIND_INTERNAL,
            stage: STAGE_INTERNAL,
            status: crate::STATUS_VOTING,
            internal_org: Some(org),
            internal_institution: Some(institution),
            start: now,
            end,
            citizen_eligible_total: 0,
        };

        with_transaction(|| {
            let id = match Self::allocate_proposal_id() {
                Ok(id) => id,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };

            if let Err(err) =
                crate::active_proposal_limit::try_add_active_proposal::<T>(institution, id)
            {
                return TransactionOutcome::Rollback(Err(err));
            }
            if let Err(err) = Self::acquire_internal_proposal_mutex(
                id,
                org,
                institution,
                InternalProposalMutexKind::Regular,
            ) {
                return TransactionOutcome::Rollback(Err(err));
            }

            if let Err(err) = Self::snapshot_institution_admins(id, org, institution, false) {
                return TransactionOutcome::Rollback(Err(err));
            }
            if !Self::is_admin_in_snapshot(id, institution, &who) {
                frame_support::defensive!(
                    "do_create_internal_proposal_with_explicit_threshold: proposer is missing from admin snapshot"
                );
                return TransactionOutcome::Rollback(Err(Error::<T>::NoPermission.into()));
            }
            // 中文注释:校验 threshold 不超过快照管理员数量。
            let snapshot_size =
                AdminSnapshot::<T>::get(id, institution).map(|admins| admins.len() as u32);
            if let Some(size) = snapshot_size {
                if threshold > size {
                    return TransactionOutcome::Rollback(Err(Error::<T>::InvalidInternalOrg.into()));
                }
            }
            InternalThresholdSnapshot::<T>::insert(id, threshold);

            Proposals::<T>::insert(id, proposal);
            if let Err(err) = Self::schedule_proposal_expiry(id, end) {
                return TransactionOutcome::Rollback(Err(err));
            }
            Self::deposit_event(Event::<T>::ProposalCreated {
                proposal_id: id,
                kind: PROPOSAL_KIND_INTERNAL,
                stage: STAGE_INTERNAL,
                end,
            });
            TransactionOutcome::Commit(Ok(id))
        })
    }

    fn do_create_internal_proposal_with_subject_status(
        who: T::AccountId,
        org: u8,
        institution: InstitutionPalletId,
        pending_subject: bool,
        mutex_kind: InternalProposalMutexKind,
    ) -> Result<u64, sp_runtime::DispatchError> {
        ensure!(is_valid_org(org), Error::<T>::InvalidInternalOrg);
        ensure!(
            is_valid_internal_institution::<T>(org, institution, pending_subject),
            Error::<T>::InvalidInstitution
        );
        // 中文注释：内部投票仅允许该机构管理员发起
        ensure!(
            is_internal_admin::<T>(org, institution, &who, pending_subject),
            Error::<T>::NoPermission
        );
        let threshold = internal_threshold::<T>(org, institution, pending_subject)
            .ok_or(Error::<T>::InvalidInternalOrg)?;
        // 全局活跃提案数限制（每机构最多 10 个）
        // 注意：此处只做预检，实际插入在 allocate_proposal_id 之后

        let now = <frame_system::Pallet<T>>::block_number();
        // 中文注释：end 是“最后一个允许投票的区块”，真正自动结算发生在 end + 1。
        let end = now.saturating_add(Self::internal_stage_duration());

        let proposal = Proposal {
            kind: PROPOSAL_KIND_INTERNAL,
            stage: STAGE_INTERNAL,
            status: crate::STATUS_VOTING,
            internal_org: Some(org),
            internal_institution: Some(institution),
            start: now,
            end,
            citizen_eligible_total: 0,
        };

        with_transaction(|| {
            let id = match Self::allocate_proposal_id() {
                Ok(id) => id,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };

            // 全局活跃提案数限制
            if let Err(err) =
                crate::active_proposal_limit::try_add_active_proposal::<T>(institution, id)
            {
                return TransactionOutcome::Rollback(Err(err));
            }
            // 中文注释：管理员集合变更与同一主体下的普通活跃提案互斥。
            if let Err(err) =
                Self::acquire_internal_proposal_mutex(id, org, institution, mutex_kind)
            {
                return TransactionOutcome::Rollback(Err(err));
            }

            // 中文注释：锁定该机构当前管理员与阈值快照，投票期间不再实时读取主体状态。
            if let Err(err) =
                Self::snapshot_institution_admins(id, org, institution, pending_subject)
            {
                return TransactionOutcome::Rollback(Err(err));
            }
            if !Self::is_admin_in_snapshot(id, institution, &who) {
                frame_support::defensive!(
                    "do_create_internal_proposal: proposer is missing from admin snapshot"
                );
                return TransactionOutcome::Rollback(Err(Error::<T>::NoPermission.into()));
            }
            InternalThresholdSnapshot::<T>::insert(id, threshold);

            Proposals::<T>::insert(id, proposal);
            if let Err(err) = Self::schedule_proposal_expiry(id, end) {
                return TransactionOutcome::Rollback(Err(err));
            }
            Self::deposit_event(Event::<T>::ProposalCreated {
                proposal_id: id,
                kind: PROPOSAL_KIND_INTERNAL,
                stage: STAGE_INTERNAL,
                end,
            });
            TransactionOutcome::Commit(Ok(id))
        })
    }

    pub(crate) fn do_internal_vote(
        who: T::AccountId,
        proposal_id: u64,
        approve: bool,
    ) -> DispatchResult {
        let proposal = Self::ensure_open_proposal(proposal_id)?;

        ensure!(
            proposal.kind == PROPOSAL_KIND_INTERNAL,
            Error::<T>::InvalidProposalKind
        );
        ensure!(
            proposal.stage == STAGE_INTERNAL,
            Error::<T>::InvalidProposalStage
        );
        ensure!(
            !InternalVotesByAccount::<T>::contains_key(proposal_id, &who),
            Error::<T>::AlreadyVoted
        );
        let _org = proposal
            .internal_org
            .ok_or(Error::<T>::InvalidInternalOrg)?;
        let institution = proposal
            .internal_institution
            .ok_or(Error::<T>::InvalidInstitution)?;
        // 中文注释：内部投票仅允许快照中的管理员投票，管理员更换不影响已有提案。
        ensure!(
            Self::is_admin_in_snapshot(proposal_id, institution, &who),
            Error::<T>::NoPermission
        );

        InternalVotesByAccount::<T>::insert(proposal_id, &who, approve);
        let tally = InternalTallies::<T>::mutate(proposal_id, |tally| {
            if approve {
                tally.yes = tally.yes.saturating_add(1);
            } else {
                tally.no = tally.no.saturating_add(1);
            }
            *tally
        });

        Self::deposit_event(Event::<T>::InternalVoteCast {
            proposal_id,
            who,
            approve,
        });

        let threshold = InternalThresholdSnapshot::<T>::get(proposal_id)
            .ok_or(Error::<T>::MissingThresholdSnapshot)?;
        if tally.yes >= threshold {
            // 中文注释：赞成票达到阈值，提前通过。
            Self::set_status_and_emit(proposal_id, STATUS_PASSED)?;
        } else {
            // 中文注释：检查剩余管理员全投赞成是否还能达到阈值，不能则提前否决。
            // 30 天超时只是兜底，不应让注定失败的提案空等。
            // admin_count 从快照取，保证阈值计算不受管理员更换影响。
            let admin_count = Self::snapshot_admin_count(proposal_id, institution)
                .ok_or(Error::<T>::MissingAdminSnapshot)?;
            let casted = tally.yes.saturating_add(tally.no);
            let remaining = admin_count.saturating_sub(casted);
            if tally.yes.saturating_add(remaining) < threshold {
                Self::set_status_and_emit(proposal_id, STATUS_REJECTED)?;
            }
        }

        Ok(())
    }

    pub(crate) fn do_finalize_internal_timeout(
        proposal: &crate::Proposal<frame_system::pallet_prelude::BlockNumberFor<T>>,
        proposal_id: u64,
    ) -> DispatchResult {
        // 中文注释：内部投票超时兜底否决。正常情况下提案会在投票期内提前通过或提前否决；
        // 此处仅处理投票人数不足、长期未完成或恰好卡在边界区块的情况。
        // 管理员名单与人数已在提案创建时快照，后续管理员更换不影响已有提案。
        ensure!(
            proposal.stage == STAGE_INTERNAL,
            Error::<T>::InvalidProposalStage
        );
        ensure!(
            proposal.status == crate::STATUS_VOTING,
            Error::<T>::ProposalAlreadyFinalized
        );
        ensure!(
            <frame_system::Pallet<T>>::block_number() > proposal.end,
            Error::<T>::VoteNotExpired
        );
        Self::set_status_and_emit(proposal_id, crate::STATUS_REJECTED)
    }
}
