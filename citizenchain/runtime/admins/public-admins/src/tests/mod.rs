#![cfg(test)]

use super::*;
use admin_primitives::{
    AdminAccountKind, AdminAccountStatus, AdminProfile, AdminSource, ADMIN_CID_NUMBER_MAX_BYTES,
    ADMIN_NAME_MAX_BYTES,
};
use frame_support::BoundedVec;
use frame_support::{
    assert_noop, assert_ok, derive_impl,
    traits::{ConstU32, ConstU64},
};
use frame_system as system;
use primitives::cid::code::code_bytes;
use primitives::count_const::NRC_ADMIN_COUNT;
use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};

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
    pub type PublicAdmins = super;

    #[runtime::pallet_index(3)]
    pub type InternalVote = internal_vote;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl system::Config for Test {
    type Block = Block;
    type AccountId = AccountId32;
    type Lookup = IdentityLookup<Self::AccountId>;
}

pub struct TestTimeProvider;
impl frame_support::traits::UnixTime for TestTimeProvider {
    fn now() -> core::time::Duration {
        core::time::Duration::from_secs(1_782_864_000)
    }
}

pub struct TestInstitutionQuery;
impl entity_primitives::InstitutionMultisigQuery<AccountId32> for TestInstitutionQuery {
    fn lookup_cid(addr: &AccountId32) -> Option<std::vec::Vec<u8>> {
        let mut cid = b"TEST-PUB-".to_vec();
        let bytes: &[u8] = addr.as_ref();
        cid.extend_from_slice(&bytes[..4]);
        Some(cid)
    }

    fn lookup_org(_addr: &AccountId32) -> Option<InstitutionCode> {
        None
    }

    fn lookup_admin_config(
        _addr: &AccountId32,
    ) -> Option<primitives::multisig::MultisigConfigSnapshot<AccountId32>> {
        None
    }

    fn is_active(_addr: &AccountId32) -> bool {
        true
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
    type CitizenIdentityReader = ();
    type JointVoteResultCallback = ();
    type InternalVoteResultCallback = ();
    type InternalAdminProvider = ();
    type InternalAdminsLenProvider = ();
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

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxAdminsPerInstitution = ConstU32<1989>;
    type InternalVoteEngine = internal_vote::Pallet<Test>;
    type InstitutionQuery = TestInstitutionQuery;
    type WeightInfo = ();
}

fn new_test_ext() -> sp_io::TestExternalities {
    let storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("test storage should build");
    let mut ext: sp_io::TestExternalities = storage.into();
    ext.execute_with(|| System::set_block_number(1));
    ext
}

fn account(seed: u8) -> AccountId32 {
    AccountId32::new([seed; 32])
}

/// 构造仅账户、空元数据(姓名/职务/任期空)的管理员资料集合。
fn admins(count: u8) -> Vec<AdminProfile<AccountId32>> {
    (0..count).map(|i| profile(account(i))).collect()
}

/// 构造一条空元数据(Registry 来源)的管理员资料。
fn profile(acc: AccountId32) -> AdminProfile<AccountId32> {
    AdminProfile {
        admin_account: acc,
        admin_cid_number: BoundedVec::new(),
        admin_name: BoundedVec::new(),
        role_code: Default::default(),
        role_name: BoundedVec::new(),
        term_start: 0,
        term_end: 0,
        admin_source: AdminSource::Registry,
        admin_source_ref: Default::default(),
    }
}

/// 构造带姓名/职务/任期/实名 CID 的管理员资料。
fn profile_full(
    acc: AccountId32,
    cid: &[u8],
    admin_name: &[u8],
    role_name: &[u8],
    term_start: u32,
    term_end: u32,
) -> AdminProfile<AccountId32> {
    AdminProfile {
        admin_account: acc,
        admin_cid_number: BoundedVec::<u8, ConstU32<ADMIN_CID_NUMBER_MAX_BYTES>>::try_from(
            cid.to_vec(),
        )
        .expect("cid fits"),
        admin_name: BoundedVec::<u8, ConstU32<ADMIN_NAME_MAX_BYTES>>::try_from(admin_name.to_vec())
            .expect("name fits"),
        role_code: Default::default(),
        role_name: BoundedVec::<u8, ConstU32<ADMIN_NAME_MAX_BYTES>>::try_from(role_name.to_vec())
            .expect("title fits"),
        term_start,
        term_end,
        admin_source: AdminSource::Registry,
        admin_source_ref: Default::default(),
    }
}

#[test]
fn public_admins_accept_public_codes_and_reject_private_codes() {
    new_test_ext().execute_with(|| {
        let root = account(10);
        assert_ok!(PublicAdmins::do_create_pending_admin_account(
            root.clone(),
            b"TEST-CID".to_vec(),
            code_bytes("PRS"),
            AdminAccountKind::PublicInstitution,
            admins(3),
            account(1),
        ));
        let stored = AdminAccounts::<Test>::get(root.clone()).expect("pending public admins");
        assert_eq!(stored.kind, AdminAccountKind::PublicInstitution);
        assert_eq!(stored.status, AdminAccountStatus::Pending);
        assert!(PublicAdmins::pending_account_exists_for_snapshot(
            code_bytes("PRS"),
            root
        ));

        assert_ok!(PublicAdmins::do_create_pending_admin_account(
            account(11),
            b"TEST-CID".to_vec(),
            code_bytes("UNIN"),
            AdminAccountKind::PublicInstitution,
            admins(2),
            account(1),
        ));

        assert_noop!(
            PublicAdmins::do_create_pending_admin_account(
                account(12),
                b"TEST-CID".to_vec(),
                code_bytes("SFLP"),
                AdminAccountKind::PublicInstitution,
                admins(3),
                account(1),
            ),
            Error::<Test>::InvalidAdminAccountKind
        );
    });
}

#[test]
fn public_admins_activate_and_query_active_admins() {
    new_test_ext().execute_with(|| {
        let root = account(20);
        assert_ok!(PublicAdmins::do_create_pending_admin_account(
            root.clone(),
            b"TEST-CID".to_vec(),
            code_bytes("CGOV"),
            AdminAccountKind::PublicInstitution,
            admins(3),
            account(1),
        ));
        assert_ok!(PublicAdmins::do_activate_admin_account(root.clone()));

        assert!(PublicAdmins::is_active_account_admin(
            code_bytes("CGOV"),
            root.clone(),
            &account(0)
        ));
        assert_eq!(
            PublicAdmins::active_account_admins_len(code_bytes("CGOV"), root),
            Some(3)
        );
    });
}

#[test]
fn public_admins_store_and_query_admin_profiles() {
    new_test_ext().execute_with(|| {
        let root = account(40);
        let profiles = alloc::vec![
            profile_full(
                account(0),
                b"GD000-CTZN8-191941078-2026",
                b"Alice",
                b"Director",
                10,
                20
            ),
            profile_full(
                account(1),
                b"GD000-CTZN2-141250905-2026",
                b"Bob",
                b"Deputy",
                11,
                21
            ),
        ];
        assert_ok!(PublicAdmins::do_create_pending_admin_account(
            root.clone(),
            b"TEST-CID".to_vec(),
            code_bytes("CGOV"),
            AdminAccountKind::PublicInstitution,
            profiles.clone(),
            account(1),
        ));
        assert_ok!(PublicAdmins::do_activate_admin_account(root.clone()));

        // 账户语义路径仍只返回账户(一人一票/多签/查配置零改动)。
        assert_eq!(
            PublicAdmins::active_account_admins(code_bytes("CGOV"), root.clone()),
            Some(alloc::vec![account(0), account(1)])
        );

        // 展示路径返回完整资料,姓名/职务/任期/实名 CID 全字段往返。
        let stored = PublicAdmins::active_account_admin_profiles(code_bytes("CGOV"), root)
            .expect("profiles present");
        assert_eq!(stored, profiles);
        assert_eq!(stored[0].admin_name.to_vec(), b"Alice".to_vec());
        assert_eq!(stored[0].role_name.to_vec(), b"Director".to_vec());
        assert_eq!(
            stored[0].admin_cid_number.to_vec(),
            b"GD000-CTZN8-191941078-2026".to_vec()
        );
        assert_eq!(stored[1].term_start, 11);
        assert_eq!(stored[1].term_end, 21);
        assert_eq!(stored[1].admin_source, AdminSource::Registry);
    });
}

#[test]
fn public_admins_accept_fixed_governance_codes_with_fixed_size() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            PublicAdmins::do_create_pending_admin_account(
                account(30),
                b"TEST-CID".to_vec(),
                code_bytes("NRC"),
                AdminAccountKind::PublicInstitution,
                admins(3),
                account(1),
            ),
            Error::<Test>::InvalidAdminsLen
        );

        assert_ok!(PublicAdmins::do_create_pending_admin_account(
            account(31),
            b"TEST-CID".to_vec(),
            code_bytes("NRC"),
            AdminAccountKind::PublicInstitution,
            admins(NRC_ADMIN_COUNT as u8),
            account(1),
        ));
        assert_ok!(PublicAdmins::do_activate_admin_account(account(31)));
        assert_eq!(
            PublicAdmins::active_account_admins_len(code_bytes("NRC"), account(31)),
            Some(NRC_ADMIN_COUNT)
        );
    });
}
