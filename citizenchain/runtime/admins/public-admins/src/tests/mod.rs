#![cfg(test)]

use super::*;
use admin_primitives::{AdminAccountKind, AdminAccountStatus};
use frame_support::{
    assert_noop, assert_ok, derive_impl,
    traits::{ConstU32, ConstU64},
};
use frame_system as system;
use primitives::cid::code::code_bytes;
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

fn admins(count: u8) -> Vec<AccountId32> {
    (0..count).map(account).collect()
}

#[test]
fn public_admins_accept_only_non_genesis_public_codes() {
    new_test_ext().execute_with(|| {
        let root = account(10);
        assert_ok!(PublicAdmins::do_create_pending_admin_account(
            root.clone(),
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
            code_bytes("UNIN"),
            AdminAccountKind::PublicInstitution,
            admins(2),
            account(1),
        ));

        assert_noop!(
            PublicAdmins::do_create_pending_admin_account(
                account(12),
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
fn public_admins_reject_genesis_codes() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            PublicAdmins::do_create_pending_admin_account(
                account(30),
                code_bytes("NRC"),
                AdminAccountKind::PublicInstitution,
                admins(3),
                account(1),
            ),
            Error::<Test>::InvalidAdminAccountKind
        );
    });
}
