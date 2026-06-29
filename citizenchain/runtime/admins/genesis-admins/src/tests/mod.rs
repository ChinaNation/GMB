#![cfg(test)]

use super::*;
use admin_primitives::{AdminAccountKind, AdminAccountStatus, AdminProfile, AdminSource};
use frame_support::BoundedVec;
use frame_support::{
    assert_noop, assert_ok, derive_impl,
    traits::{ConstU32, ConstU64},
};
use frame_system as system;
use primitives::cid::{
    china::{china_cb::CHINA_CB, china_ch::CHINA_CH, china_sf::CHINA_SF, china_zf::CHINA_ZF},
    code::{code_bytes, ProvinceCode, NJD, NRC, PROVINCE_CODE_INFOS},
};
use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};
use votingengine::types::institution_code_from_cid_number;

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
    pub type VotingEngine = votingengine;

    #[runtime::pallet_index(2)]
    pub type GenesisAdmins = super;

    #[runtime::pallet_index(3)]
    pub type InternalVote = internal_vote;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl system::Config for Test {
    type Block = Block;
    type AccountId = AccountId32;
    type Lookup = IdentityLookup<Self::AccountId>;
}

pub struct TestInternalAdminProvider;

impl votingengine::InternalAdminProvider<AccountId32> for TestInternalAdminProvider {
    fn is_internal_admin(
        institution_code: votingengine::types::InstitutionCode,
        institution: AccountId32,
        who: &AccountId32,
    ) -> bool {
        GenesisAdmins::is_active_account_admin(institution_code, institution, who)
    }

    fn get_admin_list(
        institution_code: votingengine::types::InstitutionCode,
        institution: AccountId32,
    ) -> Option<Vec<AccountId32>> {
        GenesisAdmins::active_account_admins(institution_code, institution)
    }
}

impl votingengine::InternalAdminsLenProvider<AccountId32> for TestInternalAdminProvider {
    fn admins_len(
        institution_code: votingengine::types::InstitutionCode,
        institution: AccountId32,
    ) -> Option<u32> {
        GenesisAdmins::active_account_admins_len(institution_code, institution)
    }
}

pub struct TestTimeProvider;
impl frame_support::traits::UnixTime for TestTimeProvider {
    fn now() -> core::time::Duration {
        core::time::Duration::from_secs(1_782_864_000)
    }
}

impl votingengine::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxVoteNonceLength = ConstU32<64>;
    type MaxVoteSignatureLength = ConstU32<64>;
    type MaxAutoFinalizePerBlock = ConstU32<64>;
    type MaxProposalsPerExpiry = ConstU32<128>;
    type MaxInternalProposalMutexBindings = ConstU32<256>;
    type MaxActiveProposals = ConstU32<10>;
    type MaxCleanupStepsPerBlock = ConstU32<8>;
    type MaxCleanupQueueBucketLimit = ConstU32<50>;
    type MaxCleanupScheduleOffset = ConstU32<100>;
    type CleanupKeysPerStep = ConstU32<64>;
    type MaxProposalDataLen = ConstU32<{ 8 * 1024 }>;
    type MaxProposalObjectLen = ConstU32<{ 10 * 1024 }>;
    type MaxModuleTagLen = ConstU32<32>;
    type MaxManualExecutionAttempts = ConstU32<3>;
    type ExecutionRetryGraceBlocks = ConstU64<216>;
    type MaxExecutionRetryDeadlinesPerBlock = ConstU32<128>;
    type MaxPendingRetryExpirationsPerBlock = ConstU32<16>;
    type CidEligibility = ();
    type PopulationSnapshotVerifier = ();
    type JointVoteResultCallback = ();
    type InternalVoteResultCallback = ();
    type InternalAdminProvider = TestInternalAdminProvider;
    type InternalAdminsLenProvider = TestInternalAdminProvider;
    // 中文注释：测试上限沿用真实 runtime，覆盖联邦注册局的大管理员集合。
    type MaxAdminsPerInstitution = ConstU32<1989>;
    type TimeProvider = TestTimeProvider;
    type WeightInfo = ();
    type InternalFinalizer = InternalVote;
    type InternalCleanup = InternalVote;
    type JointFinalizer = ();
    type JointCleanup = ();
    type LegislationVoteResultCallback = ();
    type LegislationFinalizer = ();
    type LegislationCleanup = ();
    type ElectionVoteResultCallback = ();
    type ElectionFinalizer = ();
    type ElectionCleanup = ();
}

impl internal_vote::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

/// 测试桩:公权机构管理员生命周期写入口。
///
/// 中文注释:现有 genesis-admins 单测不覆盖联邦直设入口,故 4 个生命周期方法仅占位返回 Ok;
/// `set_active_admin_account_direct` 走 trait 默认(未支持)。真实直设路径在 public-admins 单测覆盖。
pub struct TestPublicLifecycle;
impl admin_primitives::AdminAccountLifecycle<AccountId32, AdminProfile<AccountId32>>
    for TestPublicLifecycle
{
    fn create_pending_admin_account_for_proposal(
        _proposal_id: u64,
        _module_tag: &[u8],
        _admin_root_account_id: AccountId32,
        _institution_code: votingengine::types::InstitutionCode,
        _kind: AdminAccountKind,
        _admins: alloc::vec::Vec<AdminProfile<AccountId32>>,
        _creator: AccountId32,
    ) -> frame_support::dispatch::DispatchResult {
        Ok(())
    }
    fn activate_admin_account_for_proposal(
        _proposal_id: u64,
        _module_tag: &[u8],
        _admin_root_account_id: AccountId32,
    ) -> frame_support::dispatch::DispatchResult {
        Ok(())
    }
    fn remove_pending_admin_account_for_proposal(
        _proposal_id: u64,
        _module_tag: &[u8],
        _admin_root_account_id: AccountId32,
    ) -> frame_support::dispatch::DispatchResult {
        Ok(())
    }
    fn close_admin_account_for_proposal(
        _proposal_id: u64,
        _module_tag: &[u8],
        _admin_root_account_id: AccountId32,
    ) -> frame_support::dispatch::DispatchResult {
        Ok(())
    }
}

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxAdminsPerInstitution = ConstU32<1989>;
    type MaxPersonalAccountAdmins = ConstU32<64>;
    type InternalVoteEngine = internal_vote::Pallet<Test>;
    type PublicAdminLifecycle = TestPublicLifecycle;
    type WeightInfo = ();
}

fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("test storage should build");
    GenesisConfig::<Test>::default()
        .assimilate_storage(&mut storage)
        .expect("genesis-admins genesis should assimilate");
    let mut ext: sp_io::TestExternalities = storage.into();
    ext.execute_with(|| System::set_block_number(1));
    ext
}

fn account(seed: u8) -> AccountId32 {
    AccountId32::new([seed; 32])
}

/// 构造仅账户、空元数据(姓名/职务/任期空)的管理员资料集合。
fn admins(count: u8) -> Vec<AdminProfile<AccountId32>> {
    (0..count)
        .map(|i| AdminProfile {
            account: account(i),
            admin_cid_number: BoundedVec::new(),
            name: BoundedVec::new(),
            admin_role: BoundedVec::new(),
            term_start: 0,
            term_end: 0,
            source: AdminSource::Genesis,
        })
        .collect()
}

fn bounded_admins(items: Vec<AdminProfile<AccountId32>>) -> AdminProfilesOf<Test> {
    items
        .try_into()
        .expect("test admins should fit MaxAdminsPerInstitution")
}

fn federal_registry_main_account() -> AccountId32 {
    CHINA_ZF
        .iter()
        .find_map(|node| {
            (institution_code_from_cid_number(node.cid_number) == Some(admin_primitives::FRG))
                .then(|| AccountId32::new(node.main_account))
        })
        .expect("FRG must exist in china_zf")
}

fn province_group_account(province_code: ProvinceCode) -> AccountId32 {
    federal_registry_province_group_account::<Test>(province_code)
        .expect("province group account should decode")
}

#[test]
fn genesis_build_only_inserts_genesis_admin_sources() {
    new_test_ext().execute_with(|| {
        // 中文注释：平铺 AdminAccounts 写国储会、省储会、省储行、国家司法院；
        // 联邦注册局改由 43 个省级 5 人组单独存储。
        assert_eq!(
            AdminAccounts::<Test>::iter().count(),
            CHINA_CB.len() + CHINA_CH.len() + 1
        );

        let nrc_account = AccountId32::new(CHINA_CB[0].main_account);
        let prc_account = AccountId32::new(CHINA_CB[1].main_account);
        let prb_account = AccountId32::new(CHINA_CH[0].main_account);

        for account_id in [nrc_account, prc_account, prb_account] {
            let stored = AdminAccounts::<Test>::get(account_id.clone()).expect("genesis account");
            assert_eq!(stored.kind, AdminAccountKind::GenesisInstitution);
            assert_eq!(stored.status, AdminAccountStatus::Active);
            // 创世管理员资料:来源 Genesis,姓名/职务/任期/实名 CID 一律留空。
            assert!(!stored.admins.is_empty());
            for profile in stored.admins.iter() {
                assert_eq!(profile.source, AdminSource::Genesis);
                assert!(profile.admin_cid_number.is_empty());
                assert!(profile.name.is_empty());
                assert!(profile.admin_role.is_empty());
                assert_eq!(profile.term_start, 0);
                assert_eq!(profile.term_end, 0);
            }
        }

        let njd_account = AccountId32::new(CHINA_SF[0].main_account);
        let njd = AdminAccounts::<Test>::get(njd_account).expect("NJD genesis account");
        assert_eq!(njd.institution_code, NJD);
        assert_eq!(
            njd.admins.len(),
            primitives::count_const::NJD_ADMIN_COUNT as usize
        );
        assert_eq!(
            njd.admins
                .iter()
                .filter(|profile| profile.admin_role.as_slice()
                    == admin_primitives::ADMIN_ROLE_CONSTITUTION_GUARD)
                .count(),
            5
        );
        assert_eq!(
            njd.admins
                .iter()
                .filter(|profile| profile.admin_role.as_slice()
                    == admin_primitives::ADMIN_ROLE_CHIEF_JUSTICE)
                .count(),
            1
        );
        assert_eq!(
            njd.admins
                .iter()
                .filter(|profile| profile.admin_role.as_slice()
                    == admin_primitives::ADMIN_ROLE_DEPUTY_CHIEF_JUSTICE)
                .count(),
            2
        );
        assert_eq!(
            njd.admins
                .iter()
                .filter(
                    |profile| profile.admin_role.as_slice() == admin_primitives::ADMIN_ROLE_JUSTICE
                )
                .count(),
            5
        );

        assert_eq!(
            FederalRegistryProvinceGroups::<Test>::iter().count(),
            PROVINCE_CODE_INFOS.len()
        );
        assert_eq!(
            FederalRegistryProvinceGroupAccounts::<Test>::iter().count(),
            PROVINCE_CODE_INFOS.len()
        );

        let province_code = *b"GZ";
        let group_account = province_group_account(province_code);
        let group =
            FederalRegistryProvinceGroups::<Test>::get(province_code).expect("FRG province group");
        assert_eq!(group.institution_code, admin_primitives::FRG);
        assert_eq!(group.kind, AdminAccountKind::GenesisInstitution);
        assert_eq!(group.status, AdminAccountStatus::Active);
        assert_eq!(group.admins.len(), FEDERAL_REGISTRY_PROVINCE_GROUP_SIZE);
        assert_eq!(
            FederalRegistryProvinceGroupAccounts::<Test>::get(group_account.clone()),
            Some(province_code)
        );
        assert_eq!(
            internal_vote::ActiveDynamicThresholds::<Test>::get(
                admin_primitives::FRG,
                group_account.clone()
            ),
            None
        );

        let frg_account = federal_registry_main_account();
        assert!(AdminAccounts::<Test>::get(frg_account.clone()).is_none());
        let aggregate = GenesisAdmins::active_account_admins(admin_primitives::FRG, frg_account)
            .expect("FRG aggregate admins");
        assert_eq!(
            aggregate.len(),
            PROVINCE_CODE_INFOS.len() * FEDERAL_REGISTRY_PROVINCE_GROUP_SIZE
        );
    });
}

#[test]
fn genesis_admins_accept_only_genesis_codes() {
    new_test_ext().execute_with(|| {
        let root = account(90);
        assert_ok!(GenesisAdmins::do_create_pending_admin_account(
            root.clone(),
            NRC,
            AdminAccountKind::GenesisInstitution,
            admins(19),
            account(1),
        ));
        assert!(GenesisAdmins::pending_account_exists_for_snapshot(
            NRC, root
        ));

        let njd_root = account(89);
        assert_ok!(GenesisAdmins::do_create_pending_admin_account(
            njd_root.clone(),
            NJD,
            AdminAccountKind::GenesisInstitution,
            admins(13),
            account(1),
        ));
        assert!(GenesisAdmins::pending_account_exists_for_snapshot(
            NJD, njd_root
        ));

        assert_noop!(
            GenesisAdmins::do_create_pending_admin_account(
                account(91),
                code_bytes("PRS"),
                AdminAccountKind::GenesisInstitution,
                admins(3),
                account(1),
            ),
            Error::<Test>::InvalidAdminAccountKind
        );

        assert_noop!(
            GenesisAdmins::do_create_pending_admin_account(
                account(92),
                admin_primitives::FRG,
                AdminAccountKind::GenesisInstitution,
                admins(5),
                account(1),
            ),
            Error::<Test>::FederalRegistryRequiresProvinceGroup
        );
    });
}

#[test]
fn federal_registry_admin_change_must_use_province_group_call() {
    new_test_ext().execute_with(|| {
        let frg_account = federal_registry_main_account();
        assert_noop!(
            GenesisAdmins::propose_admin_set_change(
                RuntimeOrigin::signed(account(1)),
                admin_primitives::FRG,
                frg_account,
                bounded_admins(admins(5)),
                FEDERAL_REGISTRY_PROVINCE_GROUP_THRESHOLD,
            ),
            Error::<Test>::FederalRegistryRequiresProvinceGroup
        );

        let province_code = *b"GZ";
        let group_account = province_group_account(province_code);
        let current =
            FederalRegistryProvinceGroups::<Test>::get(province_code).expect("FRG province group");
        let proposer = current.admins[0].account.clone();
        let mut replacement = current.admins.into_inner();
        replacement[0].account = account(250);

        assert_ok!(
            GenesisAdmins::propose_federal_registry_province_admin_set_change(
                RuntimeOrigin::signed(proposer),
                province_code,
                bounded_admins(replacement),
                FEDERAL_REGISTRY_PROVINCE_GROUP_THRESHOLD,
            )
        );

        let (proposal_id, proposal) = votingengine::pallet::Proposals::<Test>::iter()
            .next()
            .expect("province admin change proposal");
        assert_eq!(proposal.internal_code, Some(admin_primitives::FRG));
        assert_eq!(proposal.internal_institution, Some(group_account.clone()));
        assert_eq!(
            internal_vote::InternalThresholdSnapshot::<Test>::get(proposal_id),
            Some(FEDERAL_REGISTRY_PROVINCE_GROUP_THRESHOLD)
        );
        assert!(internal_vote::PendingAdminChangeThresholds::<Test>::get(proposal_id).is_none());
    });
}

#[test]
fn federal_registry_main_account_is_not_internal_vote_subject() {
    new_test_ext().execute_with(|| {
        let frg_account = federal_registry_main_account();
        let proposer = GenesisAdmins::active_account_admins(admin_primitives::FRG, frg_account.clone())
            .expect("FRG aggregate admins")[0]
            .clone();

        assert_noop!(
            <internal_vote::Pallet<Test> as votingengine::InternalVoteEngine<AccountId32>>::create_general_internal_proposal_with_data(
                proposer,
                admin_primitives::FRG,
                frg_account,
                MODULE_TAG,
                Vec::new(),
            ),
            votingengine::Error::<Test>::InvalidInstitution
        );
    });
}

#[test]
fn fixed_governance_admin_count_is_locked() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            GenesisAdmins::do_create_pending_admin_account(
                account(92),
                NRC,
                AdminAccountKind::GenesisInstitution,
                admins(3),
                account(1),
            ),
            Error::<Test>::InvalidAdminsLen
        );
        assert_noop!(
            GenesisAdmins::do_create_pending_admin_account(
                account(93),
                NJD,
                AdminAccountKind::GenesisInstitution,
                admins(12),
                account(1),
            ),
            Error::<Test>::InvalidAdminsLen
        );
    });
}

#[test]
fn active_genesis_account_cannot_be_closed() {
    new_test_ext().execute_with(|| {
        let nrc_account = AccountId32::new(CHINA_CB[0].main_account);
        assert_noop!(
            GenesisAdmins::do_close_admin_account(nrc_account),
            Error::<Test>::BuiltinAdminAccountCannotClose
        );
    });
}
