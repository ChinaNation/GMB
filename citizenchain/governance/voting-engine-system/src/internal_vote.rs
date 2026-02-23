#![allow(dead_code)]

#[cfg(test)]
use codec::Encode;
use frame_support::{ensure, pallet_prelude::DispatchResult};
use sp_runtime::traits::{SaturatedConversion, Saturating};

use primitives::count_const::{
    NRC_INTERNAL_THRESHOLD, PRB_INTERNAL_THRESHOLD, PRC_INTERNAL_THRESHOLD, VOTING_DURATION_BLOCKS,
};
use primitives::reserve_nodes_const::{
    pallet_id_to_bytes as reserve_pallet_id_to_bytes, CHINACB,
};
use primitives::shengbank_nodes_const::{
    pallet_id_to_bytes as shengbank_pallet_id_to_bytes, CHINACH,
};

use crate::{
    pallet::{Config, Error, Event, InternalTallies, InternalVotesByAccount, Pallet, Proposals},
    InstitutionPalletId, InternalAdminProvider, Proposal, PROPOSAL_KIND_INTERNAL, STAGE_INTERNAL,
    STATUS_PASSED,
};

pub const ORG_NRC: u8 = 0;
pub const ORG_PRC: u8 = 1;
pub const ORG_PRB: u8 = 2;

pub fn is_valid_org(org: u8) -> bool {
    matches!(org, ORG_NRC | ORG_PRC | ORG_PRB)
}

pub fn org_pass_threshold(org: u8) -> Option<u32> {
    match org {
        ORG_NRC => Some(NRC_INTERNAL_THRESHOLD),
        ORG_PRC => Some(PRC_INTERNAL_THRESHOLD),
        ORG_PRB => Some(PRB_INTERNAL_THRESHOLD),
        _ => None,
    }
}

fn nrc_pallet_id_bytes() -> InstitutionPalletId {
    CHINACB
        .iter()
        .find(|n| n.pallet_id == "nrcgch01")
        .and_then(|n| reserve_pallet_id_to_bytes(n.pallet_id))
        .expect("NRC pallet_id must be 8 bytes")
}

fn is_valid_internal_institution(org: u8, institution: InstitutionPalletId) -> bool {
    match org {
        // 国储会只有一个机构
        ORG_NRC => institution == nrc_pallet_id_bytes(),
        // 省储会从 CHINACB 中排除国储会
        ORG_PRC => CHINACB
            .iter()
            .filter(|n| n.pallet_id != "nrcgch01")
            .filter_map(|n| reserve_pallet_id_to_bytes(n.pallet_id))
            .any(|pid| pid == institution),
        // 省储行从 CHINACH 获取
        ORG_PRB => CHINACH
            .iter()
            .filter_map(|n| shengbank_pallet_id_to_bytes(n.pallet_id))
            .any(|pid| pid == institution),
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
            ORG_NRC | ORG_PRC => CHINACB
                .iter()
                .find(|n| reserve_pallet_id_to_bytes(n.pallet_id) == Some(institution))
                .map(|n| n.admins.iter().any(|admin| *admin == who_arr))
                .unwrap_or(false),
            ORG_PRB => CHINACH
                .iter()
                .find(|n| shengbank_pallet_id_to_bytes(n.pallet_id) == Some(institution))
                .map(|n| n.admins.iter().any(|admin| *admin == who_arr))
                .unwrap_or(false),
            _ => false,
        }
    }
}

impl<T: Config> Pallet<T> {
    fn internal_stage_duration() -> frame_system::pallet_prelude::BlockNumberFor<T> {
        (VOTING_DURATION_BLOCKS as u64).saturated_into()
    }

    pub(crate) fn do_create_internal_proposal(
        who: T::AccountId,
        org: u8,
        institution: InstitutionPalletId,
    ) -> Result<u64, sp_runtime::DispatchError> {
        ensure!(is_valid_org(org), Error::<T>::InvalidInternalOrg);
        ensure!(
            is_valid_internal_institution(org, institution),
            Error::<T>::InvalidInstitution
        );
        // 中文注释：内部投票仅允许该机构管理员发起
        ensure!(
            is_internal_admin::<T>(org, institution, &who),
            Error::<T>::InvalidInstitution
        );

        let id = Self::allocate_proposal_id();
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

        Proposals::<T>::insert(id, proposal);
        Self::deposit_event(Event::<T>::ProposalCreated {
            proposal_id: id,
            kind: PROPOSAL_KIND_INTERNAL,
            stage: STAGE_INTERNAL,
            end,
        });
        Ok(id)
    }

    pub(crate) fn do_internal_vote(
        who: T::AccountId,
        proposal_id: u64,
        approve: bool,
    ) -> DispatchResult {
        let mut proposal = Self::ensure_open_proposal(proposal_id)?;

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
        // 中文注释：内部投票仅允许该机构管理员投票
        ensure!(
            is_internal_admin::<T>(org, institution, &who),
            Error::<T>::InvalidInstitution
        );

        InternalVotesByAccount::<T>::insert(proposal_id, &who, approve);
        InternalTallies::<T>::mutate(proposal_id, |tally| {
            if approve {
                tally.yes = tally.yes.saturating_add(1);
            } else {
                tally.no = tally.no.saturating_add(1);
            }
        });

        Self::deposit_event(Event::<T>::InternalVoteCast {
            proposal_id,
            who,
            approve,
        });

        let threshold = org_pass_threshold(org).ok_or(Error::<T>::InvalidInternalOrg)?;
        let tally = InternalTallies::<T>::get(proposal_id);
        if tally.yes >= threshold {
            proposal.status = STATUS_PASSED;
            Proposals::<T>::insert(proposal_id, proposal);
            Self::deposit_event(Event::<T>::ProposalFinalized {
                proposal_id,
                status: STATUS_PASSED,
            });
        }

        Ok(())
    }

    pub(crate) fn do_finalize_internal_timeout(proposal_id: u64) -> DispatchResult {
        let proposal = Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;
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
