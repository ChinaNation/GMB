#![cfg_attr(not(feature = "std"), no_std)]
//! 管理员权限治理模块（admins-origin-gov）
//! - 本模块只负责“更换管理员”这一类业务事项
//! - 投票流程本身由 voting-engine-system 提供（内部投票）
//! - 约束：仅替换，不增删；且仅能在本机构范围内更换

extern crate alloc;

use alloc::vec::Vec;
use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::{
    ensure, pallet_prelude::*, traits::StorageVersion, weights::Weight, Blake2_128Concat,
    Twox64Concat,
};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::traits::Saturating;

use primitives::china::china_cb::{shenfen_id_to_fixed48 as reserve_pallet_id_to_bytes, CHINA_CB};
use primitives::china::china_ch::{
    shenfen_id_to_fixed48 as shengbank_pallet_id_to_bytes, CHINA_CH,
};
use primitives::count_const::{NRC_ADMIN_COUNT, PRB_ADMIN_COUNT, PRC_ADMIN_COUNT};
use voting_engine_system::{
    internal_vote::{ORG_NRC, ORG_PRB, ORG_PRC},
    InstitutionPalletId, STATUS_PASSED, STATUS_REJECTED,
};

pub use pallet::*;

#[derive(
    Clone, Debug, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen,
)]
pub struct AdminReplacementAction<AccountId> {
    /// 目标机构（机构标识 pallet_id）
    pub institution: InstitutionPalletId,
    /// 被替换的管理员
    pub old_admin: AccountId,
    /// 新管理员
    pub new_admin: AccountId,
    /// 是否已经执行替换
    pub executed: bool,
}

const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

fn str_to_pallet_id(s: &str) -> Option<InstitutionPalletId> {
    reserve_pallet_id_to_bytes(s)
}

fn str_to_shengbank_pallet_id(s: &str) -> Option<InstitutionPalletId> {
    shengbank_pallet_id_to_bytes(s)
}

fn nrc_pallet_id_bytes() -> Option<InstitutionPalletId> {
    // 中文注释：国储会ID统一从常量数组读取并转码。
    CHINA_CB
        .first()
        .and_then(|n| reserve_pallet_id_to_bytes(n.shenfen_id))
}

fn institution_org(institution: InstitutionPalletId) -> Option<u8> {
    // 国储会固定 shenfen_id
    if Some(institution) == nrc_pallet_id_bytes() {
        return Some(ORG_NRC);
    }

    if CHINA_CB
        .iter()
        .skip(1)
        .filter_map(|n| str_to_pallet_id(n.shenfen_id))
        .any(|pid| pid == institution)
    {
        return Some(ORG_PRC);
    }

    if CHINA_CH
        .iter()
        .filter_map(|n| str_to_shengbank_pallet_id(n.shenfen_id))
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
    use voting_engine_system::InternalVoteEngine;

    #[pallet::config]
    pub trait Config: frame_system::Config + voting_engine_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        #[pallet::constant]
        /// 单个机构管理员最大数量上限（用于 BoundedVec）
        type MaxAdminsPerInstitution: Get<u32>;

        #[pallet::constant]
        /// 超过该时长仍未执行的提案可被清理。
        type StaleProposalLifetime: Get<BlockNumberFor<Self>>;

        /// 中文注释：内部投票引擎（返回真实 proposal_id，避免外部猜测 next_proposal_id）。
        type InternalVoteEngine: voting_engine_system::InternalVoteEngine<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub type ProposalActions<T: Config> =
        StorageMap<_, Twox64Concat, u64, AdminReplacementAction<T::AccountId>, OptionQuery>;

    #[pallet::storage]
    pub type CurrentAdmins<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        InstitutionPalletId,
        BoundedVec<T::AccountId, T::MaxAdminsPerInstitution>,
        OptionQuery,
    >;

    #[pallet::storage]
    pub type ProposalCreatedAt<T: Config> =
        StorageMap<_, Twox64Concat, u64, BlockNumberFor<T>, OptionQuery>;

    #[pallet::storage]
    pub type ActiveProposalByInstitution<T: Config> =
        StorageMap<_, Blake2_128Concat, InstitutionPalletId, u64, OptionQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub _phantom: core::marker::PhantomData<T>,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                _phantom: Default::default(),
            }
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        #[cfg(feature = "std")]
        fn integrity_test() {
            let required = NRC_ADMIN_COUNT.max(PRC_ADMIN_COUNT).max(PRB_ADMIN_COUNT);
            assert!(
                T::MaxAdminsPerInstitution::get() >= required,
                "MaxAdminsPerInstitution must be >= largest expected admin count"
            );
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            for node in CHINA_CB.iter() {
                let Some(institution) = reserve_pallet_id_to_bytes(node.shenfen_id) else {
                    continue;
                };
                let admins: Vec<T::AccountId> = node
                    .admins
                    .iter()
                    .map(|raw| {
                        T::AccountId::decode(&mut &raw[..])
                            .expect("reserve admin account must decode")
                    })
                    .collect();
                let bounded: BoundedVec<T::AccountId, T::MaxAdminsPerInstitution> = admins
                    .try_into()
                    .expect("reserve admins must fit MaxAdminsPerInstitution");
                CurrentAdmins::<T>::insert(institution, bounded);
            }

            for node in CHINA_CH.iter() {
                let Some(institution) = shengbank_pallet_id_to_bytes(node.shenfen_id) else {
                    continue;
                };
                let admins: Vec<T::AccountId> = node
                    .admins
                    .iter()
                    .map(|raw| {
                        T::AccountId::decode(&mut &raw[..])
                            .expect("shengbank admin account must decode")
                    })
                    .collect();
                let bounded: BoundedVec<T::AccountId, T::MaxAdminsPerInstitution> = admins
                    .try_into()
                    .expect("shengbank admins must fit MaxAdminsPerInstitution");
                CurrentAdmins::<T>::insert(institution, bounded);
            }
        }
    }

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
        /// 提案达到通过状态但自动执行失败（投票不回滚）
        AdminReplacementExecutionFailed { proposal_id: u64 },
        /// 管理员列表已完成替换执行
        AdminReplaced {
            proposal_id: u64,
            institution: InstitutionPalletId,
            old_admin: T::AccountId,
            new_admin: T::AccountId,
        },
        /// 过期且未执行的提案被清理
        StaleProposalCancelled {
            proposal_id: u64,
            institution: InstitutionPalletId,
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
        /// 同一机构已有活跃提案
        ActiveProposalExists,
        /// 提案尚未达到可清理的过期时间
        ProposalNotStale,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(
            Weight::from_parts(80_000_000, 4_096)
                .saturating_add(T::DbWeight::get().reads_writes(8, 8))
        )]
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
            Self::ensure_no_active_proposal(institution)?;

            // 2) 校验发起人与替换参数合法性
            let admins = Self::admins_for_institution(institution)?;
            ensure!(admins.contains(&who), Error::<T>::UnauthorizedAdmin);
            ensure!(admins.contains(&old_admin), Error::<T>::OldAdminNotFound);
            ensure!(
                !admins.contains(&new_admin),
                Error::<T>::NewAdminAlreadyExists
            );

            // 3) 在投票引擎中创建内部投票提案，并记录业务动作
            let proposal_id =
                T::InternalVoteEngine::create_internal_proposal(who.clone(), org, institution)?;

            ProposalActions::<T>::insert(
                proposal_id,
                AdminReplacementAction {
                    institution,
                    old_admin: old_admin.clone(),
                    new_admin: new_admin.clone(),
                    executed: false,
                },
            );
            ProposalCreatedAt::<T>::insert(proposal_id, frame_system::Pallet::<T>::block_number());
            ActiveProposalByInstitution::<T>::insert(institution, proposal_id);

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
        #[pallet::weight(
            Weight::from_parts(200_000_000, 8_192)
                .saturating_add(T::DbWeight::get().reads_writes(12, 10))
        )]
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
            T::InternalVoteEngine::cast_internal_vote(who.clone(), proposal_id, approve)?;

            Self::deposit_event(Event::<T>::AdminReplacementVoteSubmitted {
                proposal_id,
                who,
                approve,
            });

            if approve {
                // 中文注释：只在内部投票状态达到 PASSED 时执行替换，避免前置赞成票被回滚。
                if let Some(proposal) = voting_engine_system::Pallet::<T>::proposals(proposal_id) {
                    if proposal.status == STATUS_PASSED {
                        if Self::try_execute_replacement_from_action(proposal_id, action).is_err() {
                            Self::deposit_event(Event::<T>::AdminReplacementExecutionFailed {
                                proposal_id,
                            });
                        }
                    }
                }
            }
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(
            Weight::from_parts(120_000_000, 4_096)
                .saturating_add(T::DbWeight::get().reads_writes(8, 7))
        )]
        pub fn execute_admin_replacement(origin: OriginFor<T>, proposal_id: u64) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            Self::try_execute_replacement(proposal_id)
        }

        #[pallet::call_index(3)]
        #[pallet::weight(
            Weight::from_parts(60_000_000, 4_096)
                .saturating_add(T::DbWeight::get().reads_writes(4, 4))
        )]
        pub fn cancel_stale_proposal(origin: OriginFor<T>, proposal_id: u64) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            let action =
                ProposalActions::<T>::get(proposal_id).ok_or(Error::<T>::ProposalActionNotFound)?;
            ensure!(!action.executed, Error::<T>::ProposalAlreadyExecuted);
            let created_at = ProposalCreatedAt::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            let now = frame_system::Pallet::<T>::block_number();
            let stale_at = created_at.saturating_add(T::StaleProposalLifetime::get());
            ensure!(now >= stale_at, Error::<T>::ProposalNotStale);

            ProposalActions::<T>::remove(proposal_id);
            ProposalCreatedAt::<T>::remove(proposal_id);
            Self::remove_active_proposal_if_matches(action.institution, proposal_id);

            Self::deposit_event(Event::<T>::StaleProposalCancelled {
                proposal_id,
                institution: action.institution,
            });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn admins_for_institution(
            institution: InstitutionPalletId,
        ) -> Result<Vec<T::AccountId>, DispatchError> {
            // 中文注释：创世后只信任链上管理员状态，不再回退常量管理员。
            let stored =
                CurrentAdmins::<T>::get(institution).ok_or(Error::<T>::InvalidInstitution)?;
            Ok(stored.into_inner())
        }

        fn ensure_no_active_proposal(institution: InstitutionPalletId) -> DispatchResult {
            if let Some(existing_id) = ActiveProposalByInstitution::<T>::get(institution) {
                if let Some(action) = ProposalActions::<T>::get(existing_id) {
                    if !action.executed {
                        let still_active =
                            voting_engine_system::Pallet::<T>::proposals(existing_id)
                                .map(|proposal| proposal.status != STATUS_REJECTED)
                                .unwrap_or(false);
                        if still_active {
                            return Err(Error::<T>::ActiveProposalExists.into());
                        }
                    }
                    ProposalActions::<T>::remove(existing_id);
                    ProposalCreatedAt::<T>::remove(existing_id);
                } else {
                    ProposalCreatedAt::<T>::remove(existing_id);
                }
                // 清理悬挂索引，避免历史脏数据阻塞新提案。
                ActiveProposalByInstitution::<T>::remove(institution);
            }
            Ok(())
        }

        fn remove_active_proposal_if_matches(institution: InstitutionPalletId, proposal_id: u64) {
            if ActiveProposalByInstitution::<T>::get(institution) == Some(proposal_id) {
                ActiveProposalByInstitution::<T>::remove(institution);
            }
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
            Self::try_execute_replacement_from_action(proposal_id, action)
        }

        fn try_execute_replacement_from_action(
            proposal_id: u64,
            action: AdminReplacementAction<T::AccountId>,
        ) -> DispatchResult {
            // 仅在内部投票提案状态为 PASSED 时执行替换
            let proposal = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            ensure!(
                proposal.status == STATUS_PASSED,
                Error::<T>::ProposalNotPassed
            );

            let org = institution_org(action.institution).ok_or(Error::<T>::InvalidInstitution)?;
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
                admins
                    .try_into()
                    .map_err(|_| Error::<T>::InvalidAdminCount)?;
            CurrentAdmins::<T>::insert(action.institution, bounded);

            ProposalActions::<T>::remove(proposal_id);
            ProposalCreatedAt::<T>::remove(proposal_id);
            Self::remove_active_proposal_if_matches(action.institution, proposal_id);

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

#[cfg(test)]
mod tests {
    use super::*;
    use codec::Encode;
    use frame_support::{
        assert_noop, assert_ok, derive_impl,
        traits::{ConstU32, ConstU64},
    };
    use frame_system as system;
    use primitives::china::china_cb::{
        shenfen_id_to_fixed48 as reserve_pallet_id_to_bytes, CHINA_CB,
    };
    use primitives::china::china_ch::{
        shenfen_id_to_fixed48 as shengbank_pallet_id_to_bytes, CHINA_CH,
    };
    use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};
    use voting_engine_system::{
        internal_vote::{ORG_NRC, ORG_PRB, ORG_PRC},
        STATUS_PASSED, STATUS_REJECTED,
    };

    type Block = frame_system::mocking::MockBlock<Test>;

    #[frame_support::runtime]
    mod runtime {
        #[runtime::runtime]
        #[runtime::derive(
            RuntimeCall,
            RuntimeEvent,
            RuntimeError,
            RuntimeOrigin,
            RuntimeFreezeReason,
            RuntimeHoldReason,
            RuntimeSlashReason,
            RuntimeLockId,
            RuntimeTask,
            RuntimeViewFunction
        )]
        pub struct Test;

        #[runtime::pallet_index(0)]
        pub type System = frame_system;

        #[runtime::pallet_index(1)]
        pub type VotingEngineSystem = voting_engine_system;

        #[runtime::pallet_index(2)]
        pub type AdminsOriginGov = super;
    }

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl system::Config for Test {
        type Block = Block;
        type AccountId = AccountId32;
        type Lookup = IdentityLookup<Self::AccountId>;
    }

    pub struct TestSfidEligibility;
    impl voting_engine_system::SfidEligibility<AccountId32> for TestSfidEligibility {
        fn is_eligible(_sfid: &[u8], _who: &AccountId32) -> bool {
            true
        }

        fn verify_and_consume_vote_credential(
            _sfid: &[u8],
            _who: &AccountId32,
            _proposal_id: u64,
            _nonce: &[u8],
            _signature: &[u8],
        ) -> bool {
            true
        }
    }

    pub struct TestPopulationSnapshotVerifier;
    impl
        voting_engine_system::PopulationSnapshotVerifier<
            AccountId32,
            voting_engine_system::pallet::VoteNonceOf<Test>,
            voting_engine_system::pallet::VoteSignatureOf<Test>,
        > for TestPopulationSnapshotVerifier
    {
        fn verify_population_snapshot(
            _who: &AccountId32,
            _eligible_total: u64,
            _nonce: &voting_engine_system::pallet::VoteNonceOf<Test>,
            _signature: &voting_engine_system::pallet::VoteSignatureOf<Test>,
        ) -> bool {
            true
        }
    }

    pub struct TestInternalAdminProvider;
    impl voting_engine_system::InternalAdminProvider<AccountId32> for TestInternalAdminProvider {
        fn is_internal_admin(org: u8, institution: InstitutionPalletId, who: &AccountId32) -> bool {
            if !matches!(org, ORG_NRC | ORG_PRC | ORG_PRB) {
                return false;
            }
            if let Some(admins) = pallet::CurrentAdmins::<Test>::get(institution) {
                return admins.into_inner().iter().any(|admin| admin == who);
            }
            let who_arr = who.encode();
            if who_arr.len() != 32 {
                return false;
            }
            let mut who_raw = [0u8; 32];
            who_raw.copy_from_slice(&who_arr);
            match org {
                ORG_NRC | ORG_PRC => CHINA_CB
                    .iter()
                    .find(|n| reserve_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                    .map(|n| n.admins.iter().any(|admin| *admin == who_raw))
                    .unwrap_or(false),
                ORG_PRB => CHINA_CH
                    .iter()
                    .find(|n| shengbank_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                    .map(|n| n.admins.iter().any(|admin| *admin == who_raw))
                    .unwrap_or(false),
                _ => false,
            }
        }
    }

    impl voting_engine_system::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type MaxSfidLength = ConstU32<64>;
        type MaxVoteNonceLength = ConstU32<64>;
        type MaxVoteSignatureLength = ConstU32<64>;
        type SfidEligibility = TestSfidEligibility;
        type PopulationSnapshotVerifier = TestPopulationSnapshotVerifier;
        type JointVoteResultCallback = ();
        type InternalAdminProvider = TestInternalAdminProvider;
    }

    impl Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type MaxAdminsPerInstitution = ConstU32<32>;
        type StaleProposalLifetime = ConstU64<100>;
        type InternalVoteEngine = voting_engine_system::Pallet<Test>;
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let mut storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("test storage should build");
        GenesisConfig::<Test>::default()
            .assimilate_storage(&mut storage)
            .expect("admins-origin-gov genesis should assimilate");
        storage.into()
    }

    fn nrc_admin(index: usize) -> AccountId32 {
        AccountId32::new(CHINA_CB[0].admins[index])
    }

    fn prc_admin(index: usize) -> AccountId32 {
        AccountId32::new(CHINA_CB[1].admins[index])
    }

    fn nrc_pallet_id() -> InstitutionPalletId {
        reserve_pallet_id_to_bytes(CHINA_CB[0].shenfen_id)
            .expect("NRC shenfen_id should map to valid shenfen_id institution id")
    }

    fn prc_pallet_id() -> InstitutionPalletId {
        reserve_pallet_id_to_bytes(CHINA_CB[1].shenfen_id)
            .expect("prc pallet_id should be valid shenfen_id institution id")
    }

    fn prb_pallet_id() -> InstitutionPalletId {
        shengbank_pallet_id_to_bytes(CHINA_CH[0].shenfen_id)
            .expect("prb pallet_id should be valid shenfen_id institution id")
    }

    fn prb_admin(index: usize) -> AccountId32 {
        AccountId32::new(CHINA_CH[0].admins[index])
    }

    #[test]
    fn nrc_replacement_executes_when_yes_votes_reach_threshold() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let old_admin = nrc_admin(1);
            let new_admin = AccountId32::new([99u8; 32]);

            assert_ok!(AdminsOriginGov::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                old_admin.clone(),
                new_admin.clone()
            ));

            for i in 0..13 {
                assert_ok!(AdminsOriginGov::vote_admin_replacement(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    0,
                    true
                ));
            }

            let admins = CurrentAdmins::<Test>::get(institution)
                .expect("current admins should be stored after execution")
                .into_inner();
            assert!(admins.iter().any(|a| a == &new_admin));
            assert!(!admins.iter().any(|a| a == &old_admin));

            assert!(ProposalActions::<Test>::get(0).is_none());
        });
    }

    #[test]
    fn non_nrc_admin_cannot_propose_nrc_replacement() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            assert_noop!(
                AdminsOriginGov::propose_admin_replacement(
                    RuntimeOrigin::signed(prc_admin(0)),
                    ORG_NRC,
                    institution,
                    nrc_admin(1),
                    AccountId32::new([77u8; 32])
                ),
                Error::<Test>::UnauthorizedAdmin
            );
        });
    }

    #[test]
    fn non_nrc_admin_cannot_vote_nrc_replacement() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            assert_ok!(AdminsOriginGov::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                nrc_admin(1),
                AccountId32::new([88u8; 32])
            ));

            assert_noop!(
                AdminsOriginGov::vote_admin_replacement(
                    RuntimeOrigin::signed(prc_admin(0)),
                    0,
                    true
                ),
                Error::<Test>::UnauthorizedAdmin
            );
        });
    }

    #[test]
    fn replaced_new_admin_can_propose_next_replacement() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let old_admin = nrc_admin(1);
            let new_admin = AccountId32::new([66u8; 32]);

            assert_ok!(AdminsOriginGov::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                old_admin,
                new_admin.clone()
            ));
            for i in 0..13 {
                assert_ok!(AdminsOriginGov::vote_admin_replacement(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    0,
                    true
                ));
            }

            assert_ok!(AdminsOriginGov::propose_admin_replacement(
                RuntimeOrigin::signed(new_admin),
                ORG_NRC,
                institution,
                nrc_admin(2),
                AccountId32::new([67u8; 32])
            ));
        });
    }

    #[test]
    fn prc_replacement_executes_when_yes_votes_reach_threshold() {
        new_test_ext().execute_with(|| {
            let institution = prc_pallet_id();
            let old_admin = prc_admin(1);
            let new_admin = AccountId32::new([55u8; 32]);

            assert_ok!(AdminsOriginGov::propose_admin_replacement(
                RuntimeOrigin::signed(prc_admin(0)),
                ORG_PRC,
                institution,
                old_admin.clone(),
                new_admin.clone()
            ));

            // 省储会内部投票阈值：>=6
            for i in 0..6 {
                assert_ok!(AdminsOriginGov::vote_admin_replacement(
                    RuntimeOrigin::signed(prc_admin(i)),
                    0,
                    true
                ));
            }

            let admins = CurrentAdmins::<Test>::get(institution)
                .expect("current admins should be stored after execution")
                .into_inner();
            assert!(admins.iter().any(|a| a == &new_admin));
            assert!(!admins.iter().any(|a| a == &old_admin));
        });
    }

    #[test]
    fn prb_replacement_executes_when_yes_votes_reach_threshold() {
        new_test_ext().execute_with(|| {
            let institution = prb_pallet_id();
            let old_admin = prb_admin(1);
            let new_admin = AccountId32::new([56u8; 32]);

            assert_ok!(AdminsOriginGov::propose_admin_replacement(
                RuntimeOrigin::signed(prb_admin(0)),
                ORG_PRB,
                institution,
                old_admin.clone(),
                new_admin.clone()
            ));

            // 省储行内部投票阈值：>=6
            for i in 0..6 {
                assert_ok!(AdminsOriginGov::vote_admin_replacement(
                    RuntimeOrigin::signed(prb_admin(i)),
                    0,
                    true
                ));
            }

            let admins = CurrentAdmins::<Test>::get(institution)
                .expect("current admins should be stored after execution")
                .into_inner();
            assert!(admins.iter().any(|a| a == &new_admin));
            assert!(!admins.iter().any(|a| a == &old_admin));
        });
    }

    #[test]
    fn non_prc_admin_cannot_propose_or_vote_prc_replacement() {
        new_test_ext().execute_with(|| {
            let institution = prc_pallet_id();

            assert_noop!(
                AdminsOriginGov::propose_admin_replacement(
                    RuntimeOrigin::signed(prb_admin(0)),
                    ORG_PRC,
                    institution,
                    prc_admin(1),
                    AccountId32::new([57u8; 32])
                ),
                Error::<Test>::UnauthorizedAdmin
            );

            assert_ok!(AdminsOriginGov::propose_admin_replacement(
                RuntimeOrigin::signed(prc_admin(0)),
                ORG_PRC,
                institution,
                prc_admin(1),
                AccountId32::new([58u8; 32])
            ));

            assert_noop!(
                AdminsOriginGov::vote_admin_replacement(
                    RuntimeOrigin::signed(prb_admin(0)),
                    0,
                    true
                ),
                Error::<Test>::UnauthorizedAdmin
            );
        });
    }

    #[test]
    fn non_prb_admin_cannot_propose_or_vote_prb_replacement() {
        new_test_ext().execute_with(|| {
            let institution = prb_pallet_id();

            assert_noop!(
                AdminsOriginGov::propose_admin_replacement(
                    RuntimeOrigin::signed(prc_admin(0)),
                    ORG_PRB,
                    institution,
                    prb_admin(1),
                    AccountId32::new([59u8; 32])
                ),
                Error::<Test>::UnauthorizedAdmin
            );

            assert_ok!(AdminsOriginGov::propose_admin_replacement(
                RuntimeOrigin::signed(prb_admin(0)),
                ORG_PRB,
                institution,
                prb_admin(1),
                AccountId32::new([60u8; 32])
            ));

            assert_noop!(
                AdminsOriginGov::vote_admin_replacement(
                    RuntimeOrigin::signed(prc_admin(0)),
                    0,
                    true
                ),
                Error::<Test>::UnauthorizedAdmin
            );
        });
    }

    #[test]
    fn vote_does_not_rollback_when_auto_execute_fails() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let old_admin = nrc_admin(1);
            let new_admin = AccountId32::new([61u8; 32]);

            assert_ok!(AdminsOriginGov::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                old_admin.clone(),
                new_admin
            ));

            CurrentAdmins::<Test>::mutate(institution, |maybe| {
                let admins = maybe.as_mut().expect("institution should exist");
                let pos = admins
                    .iter()
                    .position(|a| a == &old_admin)
                    .expect("old_admin should be in admins");
                admins[pos] = nrc_admin(18);
            });

            for i in [0usize, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13] {
                assert_ok!(AdminsOriginGov::vote_admin_replacement(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    0,
                    true
                ));
            }

            let proposal =
                voting_engine_system::Pallet::<Test>::proposals(0).expect("proposal should exist");
            assert_eq!(proposal.status, STATUS_PASSED);
            assert!(
                !ProposalActions::<Test>::get(0)
                    .expect("proposal action should exist")
                    .executed
            );
            assert_noop!(
                AdminsOriginGov::execute_admin_replacement(RuntimeOrigin::signed(nrc_admin(0)), 0),
                Error::<Test>::OldAdminNotFound
            );
        });
    }

    #[test]
    fn cannot_create_second_active_proposal_for_same_institution() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();

            assert_ok!(AdminsOriginGov::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                nrc_admin(1),
                AccountId32::new([70u8; 32])
            ));

            assert_noop!(
                AdminsOriginGov::propose_admin_replacement(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    ORG_NRC,
                    institution,
                    nrc_admin(2),
                    AccountId32::new([71u8; 32])
                ),
                Error::<Test>::ActiveProposalExists
            );
        });
    }

    #[test]
    fn stale_proposal_can_be_cancelled_and_unblocks_new_proposal() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();

            assert_ok!(AdminsOriginGov::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                nrc_admin(1),
                AccountId32::new([72u8; 32])
            ));

            System::set_block_number(99);
            assert_noop!(
                AdminsOriginGov::cancel_stale_proposal(RuntimeOrigin::signed(nrc_admin(0)), 0),
                Error::<Test>::ProposalNotStale
            );

            System::set_block_number(100);
            assert_ok!(AdminsOriginGov::cancel_stale_proposal(
                RuntimeOrigin::signed(nrc_admin(0)),
                0
            ));
            assert!(ProposalActions::<Test>::get(0).is_none());
            assert!(ActiveProposalByInstitution::<Test>::get(institution).is_none());

            assert_ok!(AdminsOriginGov::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                nrc_admin(2),
                AccountId32::new([73u8; 32])
            ));
        });
    }

    #[test]
    fn org_mismatch_is_rejected() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                AdminsOriginGov::propose_admin_replacement(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    ORG_PRC,
                    nrc_pallet_id(),
                    nrc_admin(1),
                    AccountId32::new([74u8; 32])
                ),
                Error::<Test>::InstitutionOrgMismatch
            );
        });
    }

    #[test]
    fn reject_vote_does_not_trigger_execution() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let old_admin = nrc_admin(1);
            let new_admin = AccountId32::new([75u8; 32]);

            assert_ok!(AdminsOriginGov::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                old_admin.clone(),
                new_admin.clone()
            ));

            assert_ok!(AdminsOriginGov::vote_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(2)),
                0,
                false
            ));

            let admins = CurrentAdmins::<Test>::get(institution)
                .expect("current admins should remain stored")
                .into_inner();
            assert!(admins.iter().any(|a| a == &old_admin));
            assert!(!admins.iter().any(|a| a == &new_admin));
            assert!(
                !ProposalActions::<Test>::get(0)
                    .expect("proposal action should exist")
                    .executed
            );
        });
    }

    #[test]
    fn propose_fails_when_old_admin_missing() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                AdminsOriginGov::propose_admin_replacement(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    ORG_NRC,
                    nrc_pallet_id(),
                    AccountId32::new([201u8; 32]),
                    AccountId32::new([202u8; 32])
                ),
                Error::<Test>::OldAdminNotFound
            );
        });
    }

    #[test]
    fn propose_fails_when_new_admin_already_exists() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                AdminsOriginGov::propose_admin_replacement(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    ORG_NRC,
                    nrc_pallet_id(),
                    nrc_admin(1),
                    nrc_admin(2)
                ),
                Error::<Test>::NewAdminAlreadyExists
            );
        });
    }

    #[test]
    fn executed_proposal_cannot_be_executed_again() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();

            assert_ok!(AdminsOriginGov::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                nrc_admin(1),
                AccountId32::new([203u8; 32])
            ));

            for i in 0..13 {
                assert_ok!(AdminsOriginGov::vote_admin_replacement(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    0,
                    true
                ));
            }

            assert_noop!(
                AdminsOriginGov::execute_admin_replacement(RuntimeOrigin::signed(nrc_admin(0)), 0),
                Error::<Test>::ProposalActionNotFound
            );
        });
    }

    #[test]
    fn rejected_proposal_does_not_block_new_proposal() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            assert_ok!(AdminsOriginGov::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                nrc_admin(1),
                AccountId32::new([206u8; 32])
            ));

            let end = voting_engine_system::Pallet::<Test>::proposals(0)
                .expect("proposal should exist")
                .end;
            System::set_block_number(end + 1);
            assert_ok!(voting_engine_system::Pallet::<Test>::finalize_proposal(
                RuntimeOrigin::signed(nrc_admin(0)),
                0
            ));
            assert_eq!(
                voting_engine_system::Pallet::<Test>::proposals(0)
                    .expect("proposal should exist")
                    .status,
                STATUS_REJECTED
            );

            assert_ok!(AdminsOriginGov::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                nrc_admin(2),
                AccountId32::new([207u8; 32])
            ));

            assert!(ProposalActions::<Test>::get(0).is_none());
            assert!(ProposalCreatedAt::<Test>::get(0).is_none());
            assert_eq!(
                ActiveProposalByInstitution::<Test>::get(institution),
                Some(1)
            );
        });
    }

    #[test]
    fn execute_admin_replacement_succeeds_after_failed_auto_execute() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let old_admin = nrc_admin(1);
            let new_admin = AccountId32::new([208u8; 32]);

            assert_ok!(AdminsOriginGov::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                old_admin.clone(),
                new_admin.clone()
            ));

            CurrentAdmins::<Test>::mutate(institution, |maybe| {
                let admins = maybe.as_mut().expect("institution should exist");
                let old_pos = admins
                    .iter()
                    .position(|a| a == &old_admin)
                    .expect("old_admin should be in admins");
                admins[old_pos] = nrc_admin(18);
            });

            for i in [0usize, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13] {
                assert_ok!(AdminsOriginGov::vote_admin_replacement(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    0,
                    true
                ));
            }

            assert_eq!(
                voting_engine_system::Pallet::<Test>::proposals(0)
                    .expect("proposal should exist")
                    .status,
                STATUS_PASSED
            );
            assert!(ProposalActions::<Test>::get(0).is_some());

            CurrentAdmins::<Test>::mutate(institution, |maybe| {
                let admins = maybe.as_mut().expect("institution should exist");
                let restore_pos = admins
                    .iter()
                    .position(|a| a == &nrc_admin(18))
                    .expect("temporary admin marker should exist");
                admins[restore_pos] = old_admin.clone();
            });

            assert_ok!(AdminsOriginGov::execute_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                0
            ));
            let admins = CurrentAdmins::<Test>::get(institution)
                .expect("current admins should be stored")
                .into_inner();
            assert!(admins.iter().any(|a| a == &new_admin));
            assert!(!admins.iter().any(|a| a == &old_admin));
            assert!(ProposalActions::<Test>::get(0).is_none());
            assert!(ProposalCreatedAt::<Test>::get(0).is_none());
            assert!(ActiveProposalByInstitution::<Test>::get(institution).is_none());
        });
    }

    #[test]
    fn vote_below_threshold_does_not_trigger_execution() {
        new_test_ext().execute_with(|| {
            let institution = nrc_pallet_id();
            let old_admin = nrc_admin(1);
            let new_admin = AccountId32::new([204u8; 32]);

            assert_ok!(AdminsOriginGov::propose_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(0)),
                ORG_NRC,
                institution,
                old_admin.clone(),
                new_admin.clone()
            ));

            assert_ok!(AdminsOriginGov::vote_admin_replacement(
                RuntimeOrigin::signed(nrc_admin(2)),
                0,
                true
            ));

            let admins = CurrentAdmins::<Test>::get(institution)
                .expect("current admins should remain stored")
                .into_inner();
            assert!(admins.iter().any(|a| a == &old_admin));
            assert!(!admins.iter().any(|a| a == &new_admin));
            assert!(
                !ProposalActions::<Test>::get(0)
                    .expect("proposal action should exist")
                    .executed
            );
        });
    }

    #[test]
    fn invalid_institution_is_rejected() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                AdminsOriginGov::propose_admin_replacement(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    ORG_NRC,
                    [0u8; 48],
                    nrc_admin(1),
                    AccountId32::new([205u8; 32])
                ),
                Error::<Test>::InvalidInstitution
            );
        });
    }
}
