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
use crate::{InstitutionPalletId, InternalAdminProvider};

impl<T: pallet::Config> pallet::Pallet<T> {
    /// 查询快照中某管理员是否在指定机构的管理员名单中。
    pub fn is_admin_in_snapshot(
        proposal_id: u64,
        institution: InstitutionPalletId,
        who: &T::AccountId,
    ) -> bool {
        AdminSnapshot::<T>::get(proposal_id, institution)
            .map(|admins| admins.iter().any(|a| a == who))
            .unwrap_or(false)
    }

    /// 查询快照中某机构的管理员数量。
    pub fn snapshot_admin_count(
        proposal_id: u64,
        institution: InstitutionPalletId,
    ) -> Option<u32> {
        AdminSnapshot::<T>::get(proposal_id, institution).map(|admins| admins.len() as u32)
    }

    /// 将当前管理员列表写入快照存储。
    /// 如果管理员数量超过 MaxAdminsPerInstitution,触发 defensive 告警。
    pub fn snapshot_institution_admins(
        proposal_id: u64,
        org: u8,
        institution: InstitutionPalletId,
        pending_subject: bool,
    ) -> DispatchResult {
        let admins = if pending_subject {
            T::InternalAdminProvider::get_pending_admin_list(org, institution)
        } else {
            T::InternalAdminProvider::get_admin_list(org, institution)
        }
        .ok_or(Error::<T>::InvalidInstitution)?;

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
