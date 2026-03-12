//! 投票引擎模块 Benchmark 定义。

#![cfg(feature = "runtime-benchmarks")]

use codec::Decode;
use frame_benchmarking::{v2::*, BenchmarkError};
use primitives::china::china_cb::{shenfen_id_to_fixed48 as reserve_pallet_id_to_bytes, CHINA_CB};
use sp_runtime::traits::{Hash as HashT, SaturatedConversion, Saturating};

use crate::{
    CitizenTallies, CitizenVotesBySfid, Config, InstitutionPalletId, JointTallies, NextProposalId,
    Pallet, Proposal, Proposals, VoteCountU32, VoteCountU64, VoteNonceOf, VoteSignatureOf,
    PROPOSAL_KIND_INTERNAL, PROPOSAL_KIND_JOINT, STAGE_CITIZEN, STAGE_INTERNAL, STAGE_JOINT,
    STATUS_VOTING,
};

fn decode_account<T: Config>(raw: [u8; 32]) -> Result<T::AccountId, BenchmarkError> {
    T::AccountId::decode(&mut &raw[..])
        .map_err(|_| BenchmarkError::Stop("benchmark account must decode"))
}

fn nrc_institution() -> Result<InstitutionPalletId, BenchmarkError> {
    reserve_pallet_id_to_bytes(CHINA_CB[0].shenfen_id)
        .ok_or(BenchmarkError::Stop("NRC institution id should decode"))
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn create_internal_proposal() -> Result<(), BenchmarkError> {
        NextProposalId::<T>::put(0u64);
        let now = frame_system::Pallet::<T>::block_number();
        let proposal = Proposal {
            kind: PROPOSAL_KIND_INTERNAL,
            stage: STAGE_INTERNAL,
            status: STATUS_VOTING,
            internal_org: Some(crate::internal_vote::ORG_NRC),
            internal_institution: Some(nrc_institution()?),
            start: now,
            end: now,
            citizen_eligible_total: 0,
        };

        #[block]
        {
            let id = Pallet::<T>::allocate_proposal_id()
                .map_err(|_| BenchmarkError::Stop("id should allocate"))?;
            Proposals::<T>::insert(id, proposal);
        }

        assert!(Proposals::<T>::contains_key(0u64));
        Ok(())
    }

    #[benchmark]
    fn submit_joint_institution_vote() -> Result<(), BenchmarkError> {
        let who = decode_account::<T>(CHINA_CB[0].duoqian_address)?;
        let institution = nrc_institution()?;
        let now = frame_system::Pallet::<T>::block_number();
        let end = now.saturating_add(100u32.saturated_into());
        Proposals::<T>::insert(
            1u64,
            Proposal {
                kind: PROPOSAL_KIND_JOINT,
                stage: STAGE_JOINT,
                status: STATUS_VOTING,
                internal_org: None,
                internal_institution: None,
                start: now,
                end,
                citizen_eligible_total: 1_000,
            },
        );

        #[block]
        {
            Pallet::<T>::do_submit_joint_institution_vote(who, 1u64, institution, true)
                .map_err(|_| BenchmarkError::Stop("joint vote should succeed"))?;
        }

        Ok(())
    }

    #[benchmark]
    fn citizen_vote() -> Result<(), BenchmarkError> {
        let proposal_id = 2u64;
        let who = decode_account::<T>(CHINA_CB[0].admins[0])?;
        let now = frame_system::Pallet::<T>::block_number();
        let end = now.saturating_add(100u32.saturated_into());
        Proposals::<T>::insert(
            proposal_id,
            Proposal {
                kind: PROPOSAL_KIND_JOINT,
                stage: STAGE_CITIZEN,
                status: STATUS_VOTING,
                internal_org: None,
                internal_institution: None,
                start: now,
                end,
                citizen_eligible_total: 1_000,
            },
        );
        CitizenTallies::<T>::insert(proposal_id, VoteCountU64 { yes: 0, no: 0 });
        let sfid_hash = T::Hashing::hash(b"bench-sfid");
        let nonce: VoteNonceOf<T> = b"bench-nonce"
            .to_vec()
            .try_into()
            .map_err(|_| BenchmarkError::Stop("nonce should fit"))?;
        let signature: VoteSignatureOf<T> = b"bench-signature"
            .to_vec()
            .try_into()
            .map_err(|_| BenchmarkError::Stop("signature should fit"))?;

        #[block]
        {
            Pallet::<T>::do_citizen_vote(who, proposal_id, sfid_hash, nonce, signature, true)
                .map_err(|_| BenchmarkError::Stop("citizen vote should succeed"))?;
        }

        assert!(CitizenVotesBySfid::<T>::contains_key(
            proposal_id,
            sfid_hash
        ));
        assert_eq!(CitizenTallies::<T>::get(proposal_id).yes, 1u64);
        Ok(())
    }

    #[benchmark]
    fn finalize_proposal_internal() -> Result<(), BenchmarkError> {
        let proposal_id = 3u64;
        let one: frame_system::pallet_prelude::BlockNumberFor<T> = 1u32.saturated_into();
        frame_system::Pallet::<T>::set_block_number(one.saturating_add(one));
        let proposal = Proposal {
            kind: PROPOSAL_KIND_INTERNAL,
            stage: STAGE_INTERNAL,
            status: STATUS_VOTING,
            internal_org: Some(crate::internal_vote::ORG_NRC),
            internal_institution: Some(nrc_institution()?),
            start: one,
            end: one,
            citizen_eligible_total: 0,
        };
        Proposals::<T>::insert(proposal_id, proposal);

        #[block]
        {
            Pallet::<T>::do_finalize_internal_timeout(&proposal, proposal_id)
                .map_err(|_| BenchmarkError::Stop("internal finalize should succeed"))?;
        }

        Ok(())
    }

    #[benchmark]
    fn finalize_proposal_joint() -> Result<(), BenchmarkError> {
        let proposal_id = 4u64;
        let one: frame_system::pallet_prelude::BlockNumberFor<T> = 1u32.saturated_into();
        frame_system::Pallet::<T>::set_block_number(one.saturating_add(one));
        let proposal = Proposal {
            kind: PROPOSAL_KIND_JOINT,
            stage: STAGE_JOINT,
            status: STATUS_VOTING,
            internal_org: None,
            internal_institution: None,
            start: one,
            end: one,
            citizen_eligible_total: 10,
        };
        Proposals::<T>::insert(proposal_id, proposal);
        JointTallies::<T>::insert(proposal_id, VoteCountU32 { yes: 0, no: 1 });

        #[block]
        {
            Pallet::<T>::do_finalize_joint_timeout(&proposal, proposal_id)
                .map_err(|_| BenchmarkError::Stop("joint finalize should succeed"))?;
        }

        Ok(())
    }

    #[benchmark]
    fn finalize_proposal_citizen() -> Result<(), BenchmarkError> {
        let proposal_id = 5u64;
        let one: frame_system::pallet_prelude::BlockNumberFor<T> = 1u32.saturated_into();
        frame_system::Pallet::<T>::set_block_number(one.saturating_add(one));
        let proposal = Proposal {
            kind: PROPOSAL_KIND_JOINT,
            stage: STAGE_CITIZEN,
            status: STATUS_VOTING,
            internal_org: None,
            internal_institution: None,
            start: one,
            end: one,
            citizen_eligible_total: 10,
        };
        Proposals::<T>::insert(proposal_id, proposal);
        CitizenTallies::<T>::insert(proposal_id, VoteCountU64 { yes: 0, no: 1 });

        #[block]
        {
            Pallet::<T>::do_finalize_citizen_timeout(&proposal, proposal_id)
                .map_err(|_| BenchmarkError::Stop("citizen finalize should succeed"))?;
        }

        Ok(())
    }
}
