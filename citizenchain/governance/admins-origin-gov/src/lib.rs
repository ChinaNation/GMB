#![cfg_attr(not(feature = "std"), no_std)]
//! 管理员权限治理模块（admins-origin-gov）
//! - 本模块只负责“更换管理员”这一类业务事项
//! - 投票流程本身由 voting-engine-system 提供（内部投票）
//! - 约束：仅替换，不增删；且仅能在本机构范围内更换

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{ensure, pallet_prelude::*, Blake2_128Concat};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_std::vec::Vec;

use primitives::count_const::{NRC_ADMIN_COUNT, PRB_ADMIN_COUNT, PRC_ADMIN_COUNT};
use primitives::reserve_nodes_const::{
    pallet_id_to_bytes as reserve_pallet_id_to_bytes,
    RESERVE_NODES,
};
use primitives::shengbank_nodes_const::{
    pallet_id_to_bytes as shengbank_pallet_id_to_bytes,
    SHENG_BANK_NODES,
};
use voting_engine_system::{
    internal_vote::{ORG_NRC, ORG_PRB, ORG_PRC},
    InstitutionPalletId,
    STATUS_PASSED,
};

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct AdminReplacementAction<AccountId> {
    /// 目标机构（8字节 pallet_id）
    pub institution: InstitutionPalletId,
    /// 被替换的管理员
    pub old_admin: AccountId,
    /// 新管理员
    pub new_admin: AccountId,
    /// 是否已经执行替换
    pub executed: bool,
}

fn str_to_pallet_id(s: &str) -> Option<InstitutionPalletId> {
    reserve_pallet_id_to_bytes(s)
}

fn str_to_shengbank_pallet_id(s: &str) -> Option<InstitutionPalletId> {
    shengbank_pallet_id_to_bytes(s)
}

fn nrc_pallet_id_bytes() -> InstitutionPalletId {
    // 中文注释：国储会ID统一从常量数组读取并转码。
    RESERVE_NODES
        .iter()
        .find(|n| n.pallet_id == "nrcgch01")
        .and_then(|n| reserve_pallet_id_to_bytes(n.pallet_id))
        .expect("NRC pallet_id must be 8 bytes")
}

fn institution_org(institution: InstitutionPalletId) -> Option<u8> {
    // 国储会固定 pallet_id
    if institution == nrc_pallet_id_bytes() {
        return Some(ORG_NRC);
    }

    if RESERVE_NODES
        .iter()
        .skip(1)
        .filter_map(|n| str_to_pallet_id(n.pallet_id))
        .any(|pid| pid == institution)
    {
        return Some(ORG_PRC);
    }

    if SHENG_BANK_NODES
        .iter()
        .filter_map(|n| str_to_shengbank_pallet_id(n.pallet_id))
        .any(|pid| pid == institution)
    {
        return Some(ORG_PRB);
    }

    None
}

fn expected_admin_count(org: u8) -> Option<u32> {
    match org {
        ORG_NRC => Some(NRC_ADMIN_COUNT),
        ORG_PRC => Some(PRC_ADMIN_COUNT),
        ORG_PRB => Some(PRB_ADMIN_COUNT),
        _ => None,
    }
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + voting_engine_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>>
            + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        #[pallet::constant]
        /// 单个机构管理员最大数量上限（用于 BoundedVec）
        type MaxAdminsPerInstitution: Get<u32>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn proposal_action)]
    pub type ProposalActions<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        AdminReplacementAction<T::AccountId>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn current_admins)]
    pub type CurrentAdmins<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        InstitutionPalletId,
        BoundedVec<T::AccountId, T::MaxAdminsPerInstitution>,
        OptionQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 已发起管理员更换提案（并已在投票引擎创建内部提案）
        AdminReplacementProposed {
            proposal_id: u64,
            org: u8,
            institution: InstitutionPalletId,
            proposer: T::AccountId,
            old_admin: T::AccountId,
            new_admin: T::AccountId,
        },
        /// 管理员更换提案已提交一票
        AdminReplacementVoteSubmitted {
            proposal_id: u64,
            who: T::AccountId,
            approve: bool,
        },
        /// 管理员列表已完成替换执行
        AdminReplaced {
            proposal_id: u64,
            institution: InstitutionPalletId,
            old_admin: T::AccountId,
            new_admin: T::AccountId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 无效机构
        InvalidInstitution,
        /// 机构类型与 org 参数不匹配
        InstitutionOrgMismatch,
        /// 管理员数量不符合固定人数约束
        InvalidAdminCount,
        /// 非该机构管理员，无权限
        UnauthorizedAdmin,
        /// 旧管理员不在当前名单中
        OldAdminNotFound,
        /// 新管理员已经在当前名单中
        NewAdminAlreadyExists,
        /// 找不到与投票提案绑定的管理员更换动作
        ProposalActionNotFound,
        /// 投票尚未通过，不能执行替换
        ProposalNotPassed,
        /// 该提案已执行过替换
        ProposalAlreadyExecuted,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(T::DbWeight::get().reads_writes(4, 4))]
        pub fn propose_admin_replacement(
            origin: OriginFor<T>,
            org: u8,
            institution: InstitutionPalletId,
            old_admin: T::AccountId,
            new_admin: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 1) 校验机构归属范围（国储会/省储会/省储行）
            let actual_org = institution_org(institution).ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(actual_org == org, Error::<T>::InstitutionOrgMismatch);

            // 2) 校验发起人与替换参数合法性
            let admins = Self::admins_for_institution(institution)?;
            ensure!(admins.contains(&who), Error::<T>::UnauthorizedAdmin);
            ensure!(admins.contains(&old_admin), Error::<T>::OldAdminNotFound);
            ensure!(!admins.contains(&new_admin), Error::<T>::NewAdminAlreadyExists);

            // 3) 在投票引擎中创建内部投票提案，并记录业务动作
            let proposal_id = voting_engine_system::Pallet::<T>::next_proposal_id();
            voting_engine_system::Pallet::<T>::create_internal_proposal(
                frame_system::RawOrigin::Signed(who.clone()).into(),
                org,
            )?;

            ProposalActions::<T>::insert(
                proposal_id,
                AdminReplacementAction {
                    institution,
                    old_admin: old_admin.clone(),
                    new_admin: new_admin.clone(),
                    executed: false,
                },
            );

            Self::deposit_event(Event::<T>::AdminReplacementProposed {
                proposal_id,
                org,
                institution,
                proposer: who,
                old_admin,
                new_admin,
            });
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(T::DbWeight::get().reads_writes(5, 5))]
        pub fn vote_admin_replacement(
            origin: OriginFor<T>,
            proposal_id: u64,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let action =
                ProposalActions::<T>::get(proposal_id).ok_or(Error::<T>::ProposalActionNotFound)?;
            ensure!(!action.executed, Error::<T>::ProposalAlreadyExecuted);

            // 仅目标机构管理员可参与该提案投票
            let admins = Self::admins_for_institution(action.institution)?;
            ensure!(admins.contains(&who), Error::<T>::UnauthorizedAdmin);

            // 转发到投票引擎做计票与阈值判断
            voting_engine_system::Pallet::<T>::internal_vote(
                frame_system::RawOrigin::Signed(who.clone()).into(),
                proposal_id,
                approve,
            )?;

            Self::deposit_event(Event::<T>::AdminReplacementVoteSubmitted {
                proposal_id,
                who,
                approve,
            });

            if approve {
                // 投赞成票后尝试执行：只有提案状态已 PASS 才会真正替换
                Self::try_execute_replacement(proposal_id)?;
            }
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn initial_admins_for_institution(
            institution: InstitutionPalletId,
        ) -> Option<Vec<T::AccountId>> {
            if let Some(node) = RESERVE_NODES
                .iter()
                .find(|n| str_to_pallet_id(n.pallet_id) == Some(institution))
            {
                let admins = node
                    .admins
                    .iter()
                    .filter_map(|raw| T::AccountId::decode(&mut &raw[..]).ok())
                    .collect::<Vec<_>>();
                return Some(admins);
            }

            SHENG_BANK_NODES
                .iter()
                .find(|n| str_to_pallet_id(n.pallet_id) == Some(institution))
                .map(|node| {
                    node.admins
                        .iter()
                        .filter_map(|raw| T::AccountId::decode(&mut &raw[..]).ok())
                        .collect::<Vec<_>>()
                })
        }

        fn admins_for_institution(
            institution: InstitutionPalletId,
        ) -> Result<Vec<T::AccountId>, DispatchError> {
            // 优先读取链上当前管理员列表；若无则回退到创世管理员列表
            if let Some(stored) = CurrentAdmins::<T>::get(institution) {
                return Ok(stored.into_inner());
            }

            let defaults = Self::initial_admins_for_institution(institution)
                .ok_or(Error::<T>::InvalidInstitution)?;
            Ok(defaults)
        }

        fn validate_admin_count(org: u8, admins_len: usize) -> DispatchResult {
            // 固定人数约束：国储会19，省储会9，省储行9
            let expected = expected_admin_count(org).ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                admins_len == expected as usize,
                Error::<T>::InvalidAdminCount
            );
            Ok(())
        }

        fn try_execute_replacement(proposal_id: u64) -> DispatchResult {
            let action =
                ProposalActions::<T>::get(proposal_id).ok_or(Error::<T>::ProposalActionNotFound)?;
            ensure!(!action.executed, Error::<T>::ProposalAlreadyExecuted);

            // 仅在内部投票提案状态为 PASSED 时执行替换
            let proposal = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            ensure!(proposal.status == STATUS_PASSED, Error::<T>::ProposalNotPassed);

            let org =
                institution_org(action.institution).ok_or(Error::<T>::InvalidInstitution)?;
            let mut admins = Self::admins_for_institution(action.institution)?;
            Self::validate_admin_count(org, admins.len())?;

            let old_pos = admins
                .iter()
                .position(|a| a == &action.old_admin)
                .ok_or(Error::<T>::OldAdminNotFound)?;
            ensure!(
                !admins.iter().any(|a| a == &action.new_admin),
                Error::<T>::NewAdminAlreadyExists
            );

            // 只替换，不增删：列表长度保持不变
            admins[old_pos] = action.new_admin.clone();
            Self::validate_admin_count(org, admins.len())?;

            let bounded: BoundedVec<T::AccountId, T::MaxAdminsPerInstitution> =
                admins.try_into().map_err(|_| Error::<T>::InvalidAdminCount)?;
            CurrentAdmins::<T>::insert(action.institution, bounded);

            ProposalActions::<T>::mutate(proposal_id, |maybe| {
                if let Some(inner) = maybe {
                    inner.executed = true;
                }
            });

            Self::deposit_event(Event::<T>::AdminReplaced {
                proposal_id,
                institution: action.institution,
                old_admin: action.old_admin,
                new_admin: action.new_admin,
            });

            Ok(())
        }
    }
}
