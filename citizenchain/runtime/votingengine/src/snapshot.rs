//! 提案管理员快照与岗位投票人快照。
//!
//! 联合提案创建时按 `VotePlan` 锁定每个岗位主体的有效任职账户，再按 CID
//! 合并去重；投票期间不随后续任职变化。`AdminSnapshot` 只供尚未迁移的其他
//! Track 与独立个人多签路径使用，不得用于已迁移的联合投票资格判定。
//!
//! - `is_admin_in_snapshot`:查快照判断某人是否是该提案某机构的管理员
//! - `snapshot_admins_len`:快照中某机构的管理员数量
//! - `snapshot_institution_admins`:从 `InternalAdminProvider` 拉取当前管理员列表写入快照
//! - `snapshot_role_subjects`:按完整岗位主体写入任职快照和 CID 有效选民快照

use frame_support::pallet_prelude::{BoundedVec, DispatchResult};

use crate::pallet::{self, AdminSnapshot, EffectiveVoterSnapshot, Error, VoterSnapshot};
use crate::types::{AuthorizationSubject, CidNumber, InstitutionCode, ProposalSubject};
use crate::InternalAdminProvider;

impl<T: pallet::Config> pallet::Pallet<T> {
    /// 查询某完整岗位主体冻结的投票人名单。
    pub fn is_subject_voter_in_snapshot(
        proposal_id: u64,
        subject: AuthorizationSubject<CidNumber, crate::types::RoleCode, T::AccountId>,
        who: &T::AccountId,
    ) -> bool {
        VoterSnapshot::<T>::get(proposal_id, subject)
            .map(|voters| voters.iter().any(|account| account == who))
            .unwrap_or(false)
    }

    /// 查询同一机构内按账户去重后的有效投票资格。
    pub fn is_effective_voter_in_snapshot(
        proposal_id: u64,
        subject: ProposalSubject<T::AccountId>,
        who: &T::AccountId,
    ) -> bool {
        EffectiveVoterSnapshot::<T>::get(proposal_id, subject)
            .map(|voters| voters.iter().any(|account| account == who))
            .unwrap_or(false)
    }

    /// 查询同一机构有效投票人的去重人数。
    pub fn effective_voters_len(
        proposal_id: u64,
        subject: ProposalSubject<T::AccountId>,
    ) -> Option<u32> {
        EffectiveVoterSnapshot::<T>::get(proposal_id, subject).map(|voters| voters.len() as u32)
    }

    /// 冻结一个完整岗位主体的当前有效任职账户，并同步维护机构内去重投票人集合。
    pub fn snapshot_role_voters(
        proposal_id: u64,
        subject: AuthorizationSubject<CidNumber, crate::types::RoleCode, T::AccountId>,
        voters: sp_std::vec::Vec<T::AccountId>,
    ) -> DispatchResult {
        let institution_cid = match &subject {
            AuthorizationSubject::Institution(role_subject) => role_subject.cid_number.clone(),
            AuthorizationSubject::PersonalMultisig(_) => {
                return Err(Error::<T>::InvalidVotePlan.into())
            }
        };
        frame_support::ensure!(
            !VoterSnapshot::<T>::contains_key(proposal_id, &subject),
            Error::<T>::VotePlanAlreadyBound
        );
        Self::ensure_valid_voter_snapshot(voters.as_slice())?;
        let bounded = BoundedVec::<T::AccountId, T::MaxAdminsPerInstitution>::try_from(voters)
            .map_err(|_| Error::<T>::InvalidInstitution)?;

        let effective_subject = ProposalSubject::InstitutionCid(institution_cid);
        let mut effective =
            EffectiveVoterSnapshot::<T>::get(proposal_id, &effective_subject).unwrap_or_default();
        for voter in bounded.iter() {
            if !effective.iter().any(|existing| existing == voter) {
                effective
                    .try_push(voter.clone())
                    .map_err(|_| Error::<T>::InvalidInstitution)?;
            }
        }
        VoterSnapshot::<T>::insert(proposal_id, subject, bounded);
        EffectiveVoterSnapshot::<T>::insert(proposal_id, effective_subject, effective);
        Ok(())
    }

    /// 查询快照中某管理员是否在指定机构的管理员名单中。
    pub fn is_admin_in_snapshot(
        proposal_id: u64,
        subject: ProposalSubject<T::AccountId>,
        who: &T::AccountId,
    ) -> bool {
        AdminSnapshot::<T>::get(proposal_id, subject)
            .map(|admins| admins.iter().any(|a| a == who))
            .unwrap_or(false)
    }

    /// 查询快照中某机构 CID 或个人多签账户的管理员数量。
    pub fn snapshot_admins_len(
        proposal_id: u64,
        subject: ProposalSubject<T::AccountId>,
    ) -> Option<u32> {
        AdminSnapshot::<T>::get(proposal_id, subject).map(|admins| admins.len() as u32)
    }

    fn ensure_valid_admin_snapshot(admins: &[T::AccountId]) -> DispatchResult {
        // 内部投票一旦创建就只认快照；空快照会导致提案无人可投，
        // 重复管理员会破坏“一管理员一票”的票权语义，所以必须在写快照前拒绝。
        frame_support::ensure!(!admins.is_empty(), Error::<T>::MissingAdminSnapshot);
        for i in 0..admins.len() {
            for j in i.saturating_add(1)..admins.len() {
                frame_support::ensure!(admins[i] != admins[j], Error::<T>::InvalidInstitution);
            }
        }
        Ok(())
    }

    fn ensure_valid_voter_snapshot(voters: &[T::AccountId]) -> DispatchResult {
        frame_support::ensure!(!voters.is_empty(), Error::<T>::MissingVoterSnapshot);
        for i in 0..voters.len() {
            for j in i.saturating_add(1)..voters.len() {
                frame_support::ensure!(voters[i] != voters[j], Error::<T>::InvalidInstitution);
            }
        }
        Ok(())
    }

    /// 将当前管理员列表写入快照存储。
    /// 如果管理员数量超过 MaxAdminsPerInstitution,触发 defensive 告警。
    pub fn snapshot_institution_admins(
        proposal_id: u64,
        institution_code: InstitutionCode,
        cid_number: CidNumber,
    ) -> DispatchResult {
        let admins = T::InternalAdminProvider::get_institution_admins(
            institution_code,
            cid_number.as_slice(),
        )
        .ok_or(Error::<T>::InvalidInstitution)?;

        Self::write_admin_snapshot(
            proposal_id,
            ProposalSubject::InstitutionCid(cid_number),
            admins,
        )
    }

    /// 将个人多签当前或待注册管理员列表写入快照。
    pub fn snapshot_personal_admins(
        proposal_id: u64,
        personal_account: T::AccountId,
        pending_account: bool,
    ) -> DispatchResult {
        let admins = if pending_account {
            T::InternalAdminProvider::get_pending_personal_admins(personal_account.clone())
        } else {
            T::InternalAdminProvider::get_personal_admins(personal_account.clone())
        }
        .ok_or(Error::<T>::InvalidInstitution)?;

        Self::write_admin_snapshot(
            proposal_id,
            ProposalSubject::PersonalAccount(personal_account),
            admins,
        )
    }

    fn write_admin_snapshot(
        proposal_id: u64,
        subject: ProposalSubject<T::AccountId>,
        admins: sp_std::vec::Vec<T::AccountId>,
    ) -> DispatchResult {
        Self::ensure_valid_admin_snapshot(admins.as_slice())?;

        match BoundedVec::<T::AccountId, T::MaxAdminsPerInstitution>::try_from(admins) {
            Ok(bounded) => {
                AdminSnapshot::<T>::insert(proposal_id, subject, bounded);
                Ok(())
            }
            Err(_) => {
                frame_support::defensive!(
                    "snapshot_institution_admins: admin list exceeds MaxAdminsPerInstitution, snapshot not written"
                );
                Err(Error::<T>::InvalidInstitution.into())
            }
        }
    }
}
