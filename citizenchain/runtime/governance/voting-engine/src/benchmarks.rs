//! 投票引擎模块 Benchmark 定义。

#![cfg(feature = "runtime-benchmarks")]

use codec::Decode;
use frame_benchmarking::{v2::*, BenchmarkError};
use primitives::china::china_cb::{shenfen_id_to_fixed48 as reserve_pallet_id_to_bytes, CHINA_CB};
use sp_runtime::traits::{Hash as HashT, SaturatedConversion, Saturating};

use crate::{
    CitizenTallies, CitizenVotesByBindingId, Config, InstitutionPalletId, InternalTallies,
    InternalVotesByAccount, JointInstitutionTallies, JointTallies, Pallet, Proposal, Proposals,
    VoteCountU32, VoteCountU64, VoteNonceOf, VoteSignatureOf, PROPOSAL_KIND_INTERNAL,
    PROPOSAL_KIND_JOINT, STAGE_CITIZEN, STAGE_INTERNAL, STAGE_JOINT, STATUS_VOTING,
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
        let who = decode_account::<T>(CHINA_CB[0].duoqian_admins[0])?;
        let institution = nrc_institution()?;
        let created_id;

        #[block]
        {
            created_id = Pallet::<T>::do_create_internal_proposal(
                who,
                crate::internal_vote::ORG_NRC,
                institution,
            )
            .map_err(|_| BenchmarkError::Stop("create internal proposal should succeed"))?;
        }

        assert!(Proposals::<T>::contains_key(created_id));
        Ok(())
    }

    /// 内部投票 benchmark:管理员对内部提案投赞成一票。
    ///
    /// 先通过 `do_create_internal_proposal` 建立有效提案 + 管理员快照,
    /// 再测 `do_internal_vote` 主路径。
    #[benchmark]
    fn internal_vote() -> Result<(), BenchmarkError> {
        let who = decode_account::<T>(CHINA_CB[0].duoqian_admins[0])?;
        let institution = nrc_institution()?;
        let proposal_id = Pallet::<T>::do_create_internal_proposal(
            who.clone(),
            crate::internal_vote::ORG_NRC,
            institution,
        )
        .map_err(|_| BenchmarkError::Stop("create internal proposal should succeed"))?;

        #[block]
        {
            Pallet::<T>::do_internal_vote(who.clone(), proposal_id, true)
                .map_err(|_| BenchmarkError::Stop("internal vote should succeed"))?;
        }

        assert!(InternalVotesByAccount::<T>::contains_key(proposal_id, &who));
        assert_eq!(InternalTallies::<T>::get(proposal_id).yes, 1u32);
        Ok(())
    }

    #[benchmark]
    fn joint_vote() -> Result<(), BenchmarkError> {
        let who = decode_account::<T>(CHINA_CB[0].duoqian_admins[0])?;
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
        JointInstitutionTallies::<T>::insert(1u64, institution, VoteCountU32 { yes: 0, no: 0 });
        // 写入管理员快照，否则 do_joint_vote 权限校验会失败。
        Pallet::<T>::snapshot_institution_admins(1u64, crate::internal_vote::ORG_NRC, institution);

        #[block]
        {
            Pallet::<T>::do_joint_vote(who, 1u64, institution, true)
                .map_err(|_| BenchmarkError::Stop("joint vote should succeed"))?;
        }

        Ok(())
    }

    #[benchmark]
    fn citizen_vote() -> Result<(), BenchmarkError> {
        let proposal_id = 2u64;
        let who = decode_account::<T>(CHINA_CB[0].duoqian_admins[0])?;
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
        let binding_id = T::Hashing::hash(b"bench-sfid");
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
            Pallet::<T>::do_citizen_vote(who, proposal_id, binding_id, nonce, signature, true)
                .map_err(|_| BenchmarkError::Stop("citizen vote should succeed"))?;
        }

        assert!(CitizenVotesByBindingId::<T>::contains_key(
            proposal_id,
            binding_id
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
