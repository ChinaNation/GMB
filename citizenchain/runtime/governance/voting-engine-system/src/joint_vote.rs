#[cfg(test)]
use codec::Encode;
use frame_support::{
    ensure,
    pallet_prelude::DispatchResult,
    storage::{with_transaction, TransactionOutcome},
};
use sp_runtime::traits::{Hash, SaturatedConversion, Saturating};

use primitives::china::china_cb::{shenfen_id_to_fixed48 as reserve_pallet_id_to_bytes, CHINA_CB};
use primitives::china::china_ch::{
    shenfen_id_to_fixed48 as shengbank_pallet_id_to_bytes, CHINA_CH,
};
use primitives::count_const::{
    JOINT_VOTE_PASS_THRESHOLD, JOINT_VOTE_TOTAL, NRC_JOINT_VOTE_WEIGHT, PRB_JOINT_VOTE_WEIGHT,
    PRC_JOINT_VOTE_WEIGHT, VOTING_DURATION_BLOCKS,
};

use crate::{
    internal_vote::{ORG_NRC, ORG_PRB, ORG_PRC},
    pallet::{
        Config, Error, Event, JointInstitutionTallies, JointTallies, JointVotesByAdmin,
        JointVotesByInstitution, Pallet, Proposals, ProposalsByExpiry, UsedPopulationSnapshotNonce,
    },
    InstitutionPalletId, InternalAdminCountProvider, InternalAdminProvider,
    InternalThresholdProvider, PopulationSnapshotVerifier, Proposal, PROPOSAL_KIND_JOINT,
    STAGE_JOINT, STATUS_PASSED,
};

use crate::nrc_pallet_id_bytes;

#[cfg(test)]
fn is_nrc_admin_account(who: &[u8; 32]) -> bool {
    CHINA_CB
        .first()
        .map(|n| n.duoqian_admins.iter().any(|admin| admin == who))
        .unwrap_or(false)
}

fn is_nrc_admin<T: Config>(who: &T::AccountId) -> bool {
    // 中文注释：生产环境仅信任动态管理员来源（链上治理替换后的最终状态）。
    #[cfg(not(test))]
    {
        let Some(nrc) = nrc_pallet_id_bytes() else {
            return false;
        };
        T::InternalAdminProvider::is_internal_admin(crate::internal_vote::ORG_NRC, nrc, who)
    }
    // 中文注释：单测环境允许回退到常量管理员，便于独立测试本 pallet。
    #[cfg(test)]
    {
        let Some(nrc) = nrc_pallet_id_bytes() else {
            return false;
        };
        if T::InternalAdminProvider::is_internal_admin(crate::internal_vote::ORG_NRC, nrc, who) {
            return true;
        }
        let who_bytes = who.encode();
        if who_bytes.len() != 32 {
            return false;
        }
        let mut who_arr = [0u8; 32];
        who_arr.copy_from_slice(&who_bytes);
        is_nrc_admin_account(&who_arr)
    }
}

fn institution_profile(id: InstitutionPalletId) -> Option<(u8, u32)> {
    // 中文注释：联合投票需要同时知道机构所属组织和联合投票权重，
    // 这里统一把 institution 映射成 (org, weight)。
    if let Some(nrc) = nrc_pallet_id_bytes() {
        if id == nrc {
            return Some((ORG_NRC, NRC_JOINT_VOTE_WEIGHT));
        }
    }

    if CHINA_CB
        .iter()
        .skip(1)
        .filter_map(|n| reserve_pallet_id_to_bytes(n.shenfen_id))
        .any(|pid| pid == id)
    {
        return Some((ORG_PRC, PRC_JOINT_VOTE_WEIGHT));
    }

    if CHINA_CH
        .iter()
        .filter_map(|n| shengbank_pallet_id_to_bytes(n.shenfen_id))
        .any(|pid| pid == id)
    {
        return Some((ORG_PRB, PRB_JOINT_VOTE_WEIGHT));
    }

    None
}

pub fn institution_info(id: InstitutionPalletId) -> Option<u32> {
    institution_profile(id).map(|(_, weight)| weight)
}

fn is_joint_admin<T: Config>(
    org: u8,
    institution: InstitutionPalletId,
    who: &T::AccountId,
) -> bool {
    #[cfg(not(test))]
    {
        T::InternalAdminProvider::is_internal_admin(org, institution, who)
    }
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

pub fn is_joint_unanimous(yes_weight: u32) -> bool {
    // 中文注释：联合投票采用“票权全同意即通过”，不是简单人数多数制。
    yes_weight >= JOINT_VOTE_PASS_THRESHOLD
}

impl<T: Config> Pallet<T> {
    /// 联合投票阶段时长（30天，按区块高度计）
    fn joint_stage_duration() -> frame_system::pallet_prelude::BlockNumberFor<T> {
        (VOTING_DURATION_BLOCKS as u64).saturated_into()
    }

    /// 公民投票阶段时长（30天，按区块高度计）
    fn citizen_stage_duration() -> frame_system::pallet_prelude::BlockNumberFor<T> {
        (VOTING_DURATION_BLOCKS as u64).saturated_into()
    }

    /// 创建联合投票提案：独立计算本阶段 30 天截止区块，并在创建时锁定公民总人口快照。
    pub(crate) fn do_create_joint_proposal(
        who: T::AccountId,
        eligible_total: u64,
        snapshot_nonce: crate::pallet::VoteNonceOf<T>,
        signature: crate::pallet::VoteSignatureOf<T>,
    ) -> Result<u64, sp_runtime::DispatchError> {
        ensure!(is_nrc_admin::<T>(&who), Error::<T>::NoPermission);
        ensure!(eligible_total > 0, Error::<T>::CitizenEligibleTotalNotSet);
        ensure!(
            !snapshot_nonce.is_empty(),
            Error::<T>::InvalidPopulationSnapshot
        );
        ensure!(!signature.is_empty(), Error::<T>::InvalidPopulationSnapshot);

        let snapshot_nonce_hash = T::Hashing::hash(snapshot_nonce.as_slice());
        ensure!(
            !UsedPopulationSnapshotNonce::<T>::get(snapshot_nonce_hash),
            Error::<T>::InvalidPopulationSnapshot
        );
        ensure!(
            T::PopulationSnapshotVerifier::verify_population_snapshot(
                &who,
                eligible_total,
                &snapshot_nonce,
                &signature
            ),
            Error::<T>::InvalidPopulationSnapshot
        );

        let now = <frame_system::Pallet<T>>::block_number();
        // 中文注释：联合提案创建时就锁定公民投票分母与人口快照，后续阶段切换不再改写。
        let end = now.saturating_add(Self::joint_stage_duration());

        let proposal = Proposal {
            kind: PROPOSAL_KIND_JOINT,
            stage: STAGE_JOINT,
            status: crate::STATUS_VOTING,
            internal_org: None,
            internal_institution: None,
            start: now,
            end,
            citizen_eligible_total: eligible_total,
        };

        with_transaction(|| {
            let id = match Self::allocate_proposal_id() {
                Ok(id) => id,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };

            UsedPopulationSnapshotNonce::<T>::insert(snapshot_nonce_hash, true);
            Proposals::<T>::insert(id, proposal);
            if let Err(err) = Self::schedule_proposal_expiry(id, end) {
                return TransactionOutcome::Rollback(Err(err));
            }
            Self::deposit_event(Event::<T>::ProposalCreated {
                proposal_id: id,
                kind: PROPOSAL_KIND_JOINT,
                stage: STAGE_JOINT,
                end,
            });
            TransactionOutcome::Commit(Ok(id))
        })
    }

    pub(crate) fn do_joint_vote(
        who: T::AccountId,
        proposal_id: u64,
        institution: InstitutionPalletId,
        approve: bool,
    ) -> DispatchResult {
        let proposal = Self::ensure_open_proposal(proposal_id)?;

        ensure!(
            proposal.kind == PROPOSAL_KIND_JOINT,
            Error::<T>::InvalidProposalKind
        );
        ensure!(
            proposal.stage == STAGE_JOINT,
            Error::<T>::InvalidProposalStage
        );
        ensure!(
            !JointVotesByInstitution::<T>::contains_key(proposal_id, institution),
            Error::<T>::AlreadyVoted
        );
        let (org, _) = institution_profile(institution).ok_or(Error::<T>::InvalidInstitution)?;
        ensure!(
            is_joint_admin::<T>(org, institution, &who),
            Error::<T>::NoPermission
        );
        ensure!(
            !JointVotesByAdmin::<T>::contains_key(proposal_id, (institution, who.clone())),
            Error::<T>::AlreadyVoted
        );

        JointVotesByAdmin::<T>::insert(proposal_id, (institution, who.clone()), approve);
        let tally = JointInstitutionTallies::<T>::mutate(proposal_id, institution, |tally| {
            if approve {
                tally.yes = tally.yes.saturating_add(1);
            } else {
                tally.no = tally.no.saturating_add(1);
            }
            *tally
        });

        Self::deposit_event(Event::<T>::JointAdminVoteCast {
            proposal_id,
            institution,
            who,
            approve,
        });

        let threshold = T::InternalThresholdProvider::pass_threshold(org, institution)
            .ok_or(Error::<T>::InvalidInstitution)?;
        let admin_count = T::InternalAdminCountProvider::admin_count(org, institution)
            .ok_or(Error::<T>::InvalidInstitution)?;

        if tally.yes >= threshold {
            return Self::finalize_joint_institution_vote(proposal_id, institution, true);
        }

        let casted_votes = tally.yes.saturating_add(tally.no);
        let remaining_admins = admin_count.saturating_sub(casted_votes);
        if tally.yes.saturating_add(remaining_admins) < threshold {
            return Self::finalize_joint_institution_vote(proposal_id, institution, false);
        }

        Ok(())
    }

    fn finalize_joint_institution_vote(
        proposal_id: u64,
        institution: InstitutionPalletId,
        approved: bool,
    ) -> DispatchResult {
        ensure!(
            !JointVotesByInstitution::<T>::contains_key(proposal_id, institution),
            Error::<T>::AlreadyVoted
        );
        let weight = institution_info(institution).ok_or(Error::<T>::InvalidInstitution)?;

        JointVotesByInstitution::<T>::insert(proposal_id, institution, approved);

        let tally = JointTallies::<T>::mutate(proposal_id, |tally| {
            if approved {
                tally.yes = tally.yes.saturating_add(weight);
            } else {
                tally.no = tally.no.saturating_add(weight);
            }
            *tally
        });

        Self::deposit_event(Event::<T>::JointInstitutionVoteFinalized {
            proposal_id,
            institution,
            approved,
        });

        if approved {
            if is_joint_unanimous(tally.yes) {
                Self::set_status_and_emit(proposal_id, STATUS_PASSED)?;
                return Ok(());
            }

            if tally.yes.saturating_add(tally.no) >= JOINT_VOTE_TOTAL {
                return Self::advance_joint_to_citizen(proposal_id);
            }

            return Ok(());
        }

        // 中文注释：联合投票要求全票通过，只要任一机构已经形成“反对”结果，
        // 就可以立刻结束联合阶段并进入公民投票，无需再等待其他机构。
        Self::advance_joint_to_citizen(proposal_id)
    }

    /// 联合投票超时处理：
    /// - 若已全票通过，直接通过；
    /// - 否则进入公民投票阶段，并重新计算公民投票的 30 天时限。
    pub(crate) fn do_finalize_joint_timeout(
        proposal: &crate::Proposal<frame_system::pallet_prelude::BlockNumberFor<T>>,
        proposal_id: u64,
    ) -> DispatchResult {
        ensure!(
            proposal.stage == STAGE_JOINT,
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

        let tally = JointTallies::<T>::get(proposal_id);
        if is_joint_unanimous(tally.yes) {
            return Self::set_status_and_emit(proposal_id, STATUS_PASSED);
        }
        Self::advance_joint_to_citizen(proposal_id)
    }

    /// 联合投票未全票通过时，进入公民投票并重新计算公民投票阶段的 30 天截止区块。
    /// 中文注释：公民总人口分母必须由事项模块在提案创建时写入，这里只做阶段切换，绝不重置。
    fn advance_joint_to_citizen(proposal_id: u64) -> DispatchResult {
        let now = <frame_system::Pallet<T>>::block_number();
        let citizen_end = now.saturating_add(Self::citizen_stage_duration());
        with_transaction(|| {
            let (eligible_total, old_end) = match Proposals::<T>::try_mutate(
                proposal_id,
                |maybe| -> Result<
                    (u64, frame_system::pallet_prelude::BlockNumberFor<T>),
                    sp_runtime::DispatchError,
                > {
                    let proposal = maybe.as_mut().ok_or(Error::<T>::ProposalNotFound)?;
                    let eligible_total = proposal.citizen_eligible_total;
                    let old_end = proposal.end;
                    // 中文注释：这里只切换阶段窗口，不重算 eligible_total，保证联合阶段锁定的分母继续生效。
                    proposal.stage = crate::STAGE_CITIZEN;
                    proposal.start = now;
                    proposal.end = citizen_end;
                    Ok((eligible_total, old_end))
                },
            ) {
                Ok(v) => v,
                Err(err) => return TransactionOutcome::Rollback(Err(err)),
            };

            // 移除旧的联合投票阶段 expiry 条目，避免 on_initialize 无效查询
            ProposalsByExpiry::<T>::mutate(old_end, |ids| {
                ids.retain(|&id| id != proposal_id);
            });

            if let Err(err) = Self::schedule_proposal_expiry(proposal_id, citizen_end) {
                return TransactionOutcome::Rollback(Err(err));
            }

            Self::deposit_event(Event::<T>::ProposalAdvancedToCitizen {
                proposal_id,
                citizen_end,
                eligible_total,
            });
            TransactionOutcome::Commit(Ok(()))
        })
    }
}
