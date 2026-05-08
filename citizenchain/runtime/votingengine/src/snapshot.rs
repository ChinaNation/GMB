//! 提案管理员快照。
//!
//! 提案创建时锁定参与机构的管理员名单(`AdminSnapshot`),投票期间不随
//! 链上管理员名单变化。投票引擎在投票时通过快照判断资格,保证管理员
//! 更换不影响已经在投票中的提案过程。
//!
//! - `is_admin_in_snapshot`:查快照判断某人是否是该提案某机构的管理员
//! - `snapshot_admin_count`:快照中某机构的管理员数量
//! - `snapshot_institution_admins`:从 `InternalAdminProvider` 拉取当前管理员列表写入快照

use frame_support::pallet_prelude::{BoundedVec, DispatchResult};

use crate::pallet::{self, AdminSnapshot, Error};
use crate::{InternalAdminProvider, SubjectId};

impl<T: pallet::Config> pallet::Pallet<T> {
    /// 查询快照中某管理员是否在指定机构的管理员名单中。
    pub fn is_admin_in_snapshot(
        proposal_id: u64,
        institution: SubjectId,
        who: &T::AccountId,
    ) -> bool {
        AdminSnapshot::<T>::get(proposal_id, institution)
            .map(|admins| admins.iter().any(|a| a == who))
            .unwrap_or(false)
    }

    /// 查询快照中某机构的管理员数量。
    pub fn snapshot_admin_count(proposal_id: u64, institution: SubjectId) -> Option<u32> {
        AdminSnapshot::<T>::get(proposal_id, institution).map(|admins| admins.len() as u32)
    }

    fn ensure_valid_admin_snapshot(admins: &[T::AccountId]) -> DispatchResult {
        // 中文注释：内部投票一旦创建就只认快照；空快照会导致提案无人可投，
        // 重复管理员会破坏“一管理员一票”的票权语义，所以必须在写快照前拒绝。
        frame_support::ensure!(!admins.is_empty(), Error::<T>::MissingAdminSnapshot);
        for i in 0..admins.len() {
            for j in i.saturating_add(1)..admins.len() {
                frame_support::ensure!(admins[i] != admins[j], Error::<T>::InvalidInstitution);
            }
        }
        Ok(())
    }

    /// 将当前管理员列表写入快照存储。
    /// 如果管理员数量超过 MaxAdminsPerInstitution,触发 defensive 告警。
    pub fn snapshot_institution_admins(
        proposal_id: u64,
        org: u8,
        institution: SubjectId,
        pending_subject: bool,
    ) -> DispatchResult {
        let admins = if pending_subject {
            T::InternalAdminProvider::get_pending_admin_list(org, institution)
        } else {
            T::InternalAdminProvider::get_admin_list(org, institution)
        }
        .ok_or(Error::<T>::InvalidInstitution)?;

        Self::ensure_valid_admin_snapshot(admins.as_slice())?;

        match BoundedVec::<T::AccountId, T::MaxAdminsPerInstitution>::try_from(admins) {
            Ok(bounded) => {
                AdminSnapshot::<T>::insert(proposal_id, institution, bounded);
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
