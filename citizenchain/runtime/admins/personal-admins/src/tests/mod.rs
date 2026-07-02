#![cfg(test)]

use super::*;
use frame_support::{
    assert_ok, derive_impl,
    traits::{ConstU32, UnixTime},
    BoundedVec,
};
use frame_system as system;
use sp_core::{sr25519, Pair as PairT};
use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};
use votingengine::types::{InstitutionCode, PMUL};

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
    pub type InternalVote = internal_vote;

    #[runtime::pallet_index(3)]
    pub type PersonalAdmins = super;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl system::Config for Test {
    type Block = Block;
    type AccountId = AccountId32;
    type Lookup = IdentityLookup<Self::AccountId>;
}

pub struct TestCitizenIdentityReader;
impl votingengine::CitizenIdentityReader<AccountId32> for TestCitizenIdentityReader {
    fn can_vote(_who: &AccountId32, _scope: &votingengine::PopulationScope) -> bool {
        true
    }

    fn can_be_candidate(_who: &AccountId32, _scope: &votingengine::PopulationScope) -> bool {
        true
    }

    fn population_count(_scope: &votingengine::PopulationScope) -> u64 {
        100
    }
}

pub struct TestInternalAdminProvider;
impl votingengine::InternalAdminProvider<AccountId32> for TestInternalAdminProvider {
    fn is_internal_admin(
        institution_code: InstitutionCode,
        institution: AccountId32,
        who: &AccountId32,
    ) -> bool {
        PersonalAdmins::is_active_account_admin(institution_code, institution, who)
    }

    fn get_admin_list(
        institution_code: InstitutionCode,
        institution: AccountId32,
    ) -> Option<Vec<AccountId32>> {
        PersonalAdmins::active_account_admins(institution_code, institution)
    }

    fn is_pending_internal_admin(
        institution_code: InstitutionCode,
        institution: AccountId32,
        who: &AccountId32,
    ) -> bool {
        PersonalAdmins::is_pending_account_admin_for_snapshot(institution_code, institution, who)
    }

    fn get_pending_admin_list(
        institution_code: InstitutionCode,
        institution: AccountId32,
    ) -> Option<Vec<AccountId32>> {
        PersonalAdmins::pending_account_admins_for_snapshot(institution_code, institution)
    }
}

pub struct TestInternalAdminsLenProvider;
impl votingengine::InternalAdminsLenProvider<AccountId32> for TestInternalAdminsLenProvider {
    fn admins_len(institution_code: InstitutionCode, institution: AccountId32) -> Option<u32> {
        PersonalAdmins::active_account_admins_len(institution_code, institution)
    }
}

pub struct TestTimeProvider;
impl UnixTime for TestTimeProvider {
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
    type CleanupKeysPerStep = ConstU32<64>;
    type CitizenIdentityReader = TestCitizenIdentityReader;
    type JointVoteResultCallback = ();
    type InternalVoteResultCallback = crate::InternalVoteExecutor<Test>;
    type InternalAdminProvider = TestInternalAdminProvider;
    type InternalAdminsLenProvider = TestInternalAdminsLenProvider;
    type MaxAdminsPerInstitution = ConstU32<1989>;
    type MaxProposalDataLen = ConstU32<1024>;
    type MaxProposalObjectLen = ConstU32<{ 10 * 1024 }>;
    type MaxModuleTagLen = ConstU32<32>;
    type MaxManualExecutionAttempts = ConstU32<3>;
    type ExecutionRetryGraceBlocks = frame_support::traits::ConstU64<216>;
    type MaxExecutionRetryDeadlinesPerBlock = ConstU32<128>;
    type MaxCleanupQueueBucketLimit = ConstU32<50>;
    type MaxCleanupScheduleOffset = ConstU32<100>;
    type MaxPendingRetryExpirationsPerBlock = ConstU32<16>;
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

impl pallet::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type InternalVoteEngine = internal_vote::Pallet<Test>;
    type MaxPersonalAccountAdmins = ConstU32<64>;
    type WeightInfo = ();
}

fn admin(index: u8) -> AccountId32 {
    let mut seed = [0u8; 32];
    seed[0] = 0x71;
    seed[1] = index;
    AccountId32::new(sr25519::Pair::from_seed(&seed).public().0)
}

fn personal_account() -> AccountId32 {
    AccountId32::new([0x51; 32])
}

fn admins(items: &[AccountId32]) -> pallet::AdminsOf<Test> {
    BoundedVec::try_from(items.to_vec()).expect("admins fit")
}

fn seed_active_admin_account(account: AccountId32, admins: pallet::AdminsOf<Test>, threshold: u32) {
    internal_vote::ActiveDynamicThresholds::<Test>::insert(PMUL, account.clone(), threshold);
    pallet::AdminAccounts::<Test>::insert(
        account.clone(),
        admin_primitives::AdminAccount {
            institution_code: PMUL,
            kind: admin_primitives::AdminAccountKind::PersonalMultisig,
            admins,
            creator: admin(0),
            created_at: 1,
            updated_at: 1,
            status: admin_primitives::AdminAccountStatus::Active,
        },
    );
}

fn new_test_ext() -> sp_io::TestExternalities {
    let storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("test storage should build");
    let mut ext: sp_io::TestExternalities = storage.into();
    ext.execute_with(|| System::set_block_number(1));
    ext
}

#[test]
fn propose_admin_set_change_updates_personal_admins_and_threshold() {
    new_test_ext().execute_with(|| {
        let account = personal_account();
        let old_admins = admins(&[admin(0), admin(1), admin(2)]);
        seed_active_admin_account(account.clone(), old_admins.clone(), 2);

        let next_admins = admins(&[admin(0), admin(3), admin(4), admin(5)]);
        assert_ok!(PersonalAdmins::propose_admin_set_change(
            RuntimeOrigin::signed(admin(0)),
            PMUL,
            account.clone(),
            next_admins.clone(),
            3,
        ));
        let proposal_id = votingengine::Pallet::<Test>::next_proposal_id().saturating_sub(1);

        // 提案创建时发起人 admin(0) 已自动赞成，阈值 2 时只需再补一票。
        assert_ok!(internal_vote::Pallet::<Test>::do_internal_vote(
            admin(1),
            proposal_id,
            true
        ));

        let admin_account = pallet::AdminAccounts::<Test>::get(account.clone())
            .expect("admin account should exist");
        assert_eq!(admin_account.admins, next_admins);
        assert_eq!(
            internal_vote::ActiveDynamicThresholds::<Test>::get(PMUL, account),
            Some(3)
        );
    });
}
