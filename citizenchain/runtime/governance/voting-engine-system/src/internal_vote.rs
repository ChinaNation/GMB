#[cfg(test)]
use codec::Encode;
use frame_support::{
    ensure,
    pallet_prelude::DispatchResult,
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
    pallet::{Config, Error, Event, InternalTallies, InternalVotesByAccount, Pallet, Proposals},
    InstitutionPalletId, InternalAdminProvider, InternalThresholdProvider, Proposal,
    PROPOSAL_KIND_INTERNAL, STAGE_INTERNAL, STATUS_PASSED, STATUS_REJECTED,
};

pub const ORG_NRC: u8 = 0;
pub const ORG_PRC: u8 = 1;
pub const ORG_PRB: u8 = 2;
/// 注册多签/个人多签主体，管理员与阈值由 admins-origin-gov 统一主体表提供。
pub const ORG_DUOQIAN: u8 = 3;

pub fn is_valid_org(org: u8) -> bool {
    matches!(org, ORG_NRC | ORG_PRC | ORG_PRB | ORG_DUOQIAN)
}

/// 治理机构（NRC/PRC/PRB）的硬编码阈值，供 `InternalThresholdProvider` 默认实现使用。
pub fn governance_org_pass_threshold(org: u8) -> Option<u32> {
    match org {
        ORG_NRC => Some(NRC_INTERNAL_THRESHOLD),
        ORG_PRC => Some(PRC_INTERNAL_THRESHOLD),
        ORG_PRB => Some(PRB_INTERNAL_THRESHOLD),
        _ => None,
    }
}

use crate::nrc_pallet_id_bytes;

fn is_valid_internal_institution<T: Config>(org: u8, institution: InstitutionPalletId) -> bool {
    // 中文注释：内部投票里的 institution 必须与 org 类型严格对应，避免伪造”跨组织机构”。
    match org {
        // 国储会只有一个机构
        ORG_NRC => nrc_pallet_id_bytes()
            .map(|nrc| institution == nrc)
            .unwrap_or(false),
        // 省储会从 CHINA_CB 中排除国储会
        ORG_PRC => CHINA_CB
            .iter()
            .skip(1)
            .filter_map(|n| reserve_pallet_id_to_bytes(n.shenfen_id))
            .any(|pid| pid == institution),
        // 省储行从 CHINA_CH 获取
        ORG_PRB => CHINA_CH
            .iter()
            .filter_map(|n| shengbank_pallet_id_to_bytes(n.shenfen_id))
            .any(|pid| pid == institution),
        // 注册多签/个人多签主体：由 InternalThresholdProvider 判断是否存在
        ORG_DUOQIAN => T::InternalThresholdProvider::pass_threshold(org, institution).is_some(),
        _ => false,
    }
}

fn is_internal_admin<T: Config>(
    org: u8,
    institution: InstitutionPalletId,
    who: &T::AccountId,
) -> bool {
    // 中文注释：生产环境仅信任动态管理员来源（链上治理替换后的最终状态）。
    #[cfg(not(test))]
    {
        T::InternalAdminProvider::is_internal_admin(org, institution, who)
    }
    // 中文注释：单测环境允许回退到常量管理员，便于独立测试本 pallet。
    #[cfg(test)]
    {
        if T::InternalAdminProvider::is_internal_admin(org, institution, who) {
            return true;
        }

        let who_bytes = who.encode();
        if who_bytes.len() != 32 {
            return false;
        }
        let mut who_arr = [0u8; 32];
        who_arr.copy_from_slice(&who_bytes);

        match org {
            ORG_NRC | ORG_PRC => CHINA_CB
                .iter()
                .find(|n| reserve_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                .map(|n| n.duoqian_admins.iter().any(|admin| *admin == who_arr))
                .unwrap_or(false),
            ORG_PRB => CHINA_CH
                .iter()
                .find(|n| shengbank_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                .map(|n| n.duoqian_admins.iter().any(|admin| *admin == who_arr))
                .unwrap_or(false),
            _ => false,
        }
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
        ensure!(is_valid_org(org), Error::<T>::InvalidInternalOrg);
        ensure!(
            is_valid_internal_institution::<T>(org, institution),
            Error::<T>::InvalidInstitution
        );
        // 中文注释：内部投票仅允许该机构管理员发起
        ensure!(
            is_internal_admin::<T>(org, institution, &who),
            Error::<T>::NoPermission
        );
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

            // 中文注释：锁定该机构当前管理员快照，投票期间只认快照内的管理员。
            Self::snapshot_institution_admins(id, org, institution);

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
        let org = proposal
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

        let threshold = T::InternalThresholdProvider::pass_threshold(org, institution)
            .ok_or(Error::<T>::InvalidInternalOrg)?;
        if tally.yes >= threshold {
            // 中文注释：赞成票达到阈值，提前通过。
            Self::set_status_and_emit(proposal_id, STATUS_PASSED)?;
        } else {
            // 中文注释：检查剩余管理员全投赞成是否还能达到阈值，不能则提前否决。
            // 30 天超时只是兜底，不应让注定失败的提案空等。
            // admin_count 从快照取，保证阈值计算不受管理员更换影响。
            let admin_count = Self::snapshot_admin_count(proposal_id, institution)
                .ok_or(Error::<T>::InvalidInternalOrg)?;
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
        // 中文注释：内部投票超时兜底否决。正常情况下提案会在投票期内提前通过或提前否决，
        // 此处仅处理极端情况（如恰好卡在边界、管理员数量变动等）。
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
