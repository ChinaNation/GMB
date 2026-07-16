#![cfg(test)]

use super::*;
use admin_primitives::{AdminAccountKind, AdminAccountStatus};
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

    fn account_exists(_addr: &AccountId32) -> bool {
        true
    }
}

impl votingengine::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxVoteNonceLength = ConstU32<64>;
    type MaxVoteSignatureLength = ConstU32<64>;
    type MaxAutoFinalizePerBlock = ConstU32<64>;
    type MaxAutoFinalizeWeightPerBlock = votingengine::BlockWeightFraction<Test, 4>;
    type MaxExecutionWeightPerBlock = votingengine::BlockWeightFraction<Test, 4>;
    type MaxCleanupWeightPerBlock = votingengine::BlockWeightFraction<Test, 8>;
    type MaxProposalsPerExpiry = ConstU32<128>;
    type MaxInternalProposalMutexBindings = ConstU32<256>;
    type MaxActiveProposals = ConstU32<10>;
    type MaxCleanupStepsPerBlock = ConstU32<8>;
    type MaxCleanupActivationsPerBlock = ConstU32<50>;
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
    type TrackHandlers = (InternalVote, ());
    type LegislationVoteResultCallback = ();
    type ElectionVoteResultCallback = ();
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

/// admins 只保存钱包账户；岗位、任期、来源等由 entity 管理。
fn admins(count: u8) -> Vec<AccountId32> {
    (0..count).map(account).collect()
}

fn indexed_admins(count: u32) -> Vec<AccountId32> {
    (0..count)
        .map(|index| {
            let mut raw = [0u8; 32];
            raw[..4].copy_from_slice(&index.to_le_bytes());
            AccountId32::new(raw)
        })
        .collect()
}

#[test]
fn public_admins_accept_public_codes_and_reject_private_codes() {
    new_test_ext().execute_with(|| {
        let root = account(10);
        assert_ok!(PublicAdmins::do_set_active_admin_account_direct(
            root.clone(),
            b"TEST-CID".to_vec(),
            code_bytes("CGOV"),
            AdminAccountKind::PublicInstitution,
            admins(3),
            2,
        ));
        let stored = AdminAccounts::<Test>::get(root).expect("active public admins");
        assert_eq!(stored.status, AdminAccountStatus::Active);

        assert_ok!(PublicAdmins::do_set_active_admin_account_direct(
            account(11),
            b"TEST-CID".to_vec(),
            code_bytes("UNIN"),
            AdminAccountKind::PublicInstitution,
            admins(2),
            2,
        ));

        assert_noop!(
            PublicAdmins::do_set_active_admin_account_direct(
                account(12),
                b"TEST-CID".to_vec(),
                code_bytes("SFLP"),
                AdminAccountKind::PublicInstitution,
                admins(3),
                2,
            ),
            Error::<Test>::InvalidAdminAccountKind
        );
    });
}

#[test]
fn public_admins_activate_and_query_active_admins() {
    new_test_ext().execute_with(|| {
        let root = account(20);
        assert_ok!(PublicAdmins::do_set_active_admin_account_direct(
            root.clone(),
            b"TEST-CID".to_vec(),
            code_bytes("CGOV"),
            AdminAccountKind::PublicInstitution,
            admins(3),
            2,
        ));

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
fn public_admins_fix_size_only_for_full_genesis_identity() {
    new_test_ext().execute_with(|| {
        let fixed = primitives::governance_skeleton::fixed_institutions()
            .into_iter()
            .find(|institution| institution.code == code_bytes("NRC"))
            .expect("NRC genesis identity");
        assert_noop!(
            PublicAdmins::validate_admin_set_for_account(
                AdminAccountKind::PublicInstitution,
                code_bytes("NRC"),
                fixed.cid_number.as_bytes(),
                &fixed.main_account,
                &admins(3),
            ),
            Error::<Test>::InvalidAdminsLen
        );

        assert_ok!(PublicAdmins::validate_admin_set_for_account(
            AdminAccountKind::PublicInstitution,
            code_bytes("NRC"),
            fixed.cid_number.as_bytes(),
            &fixed.main_account,
            &admins(NRC_ADMIN_COUNT as u8),
        ));

        // 机构码相同但不是创世 CID + 主账户时，不得扩大固定人数保护范围。
        assert_ok!(PublicAdmins::validate_admin_set_for_account(
            AdminAccountKind::PublicInstitution,
            code_bytes("NRC"),
            b"runtime-institution",
            &[9u8; 32],
            &admins(3),
        ));
    });
}

#[test]
fn member_body_range_only_applies_to_exact_genesis_identity() {
    new_test_ext().execute_with(|| {
        let spec = primitives::institution_constraints::member_composition_specs()[0];
        assert_noop!(
            PublicAdmins::validate_admin_set_for_account(
                AdminAccountKind::PublicInstitution,
                spec.institution.code,
                spec.institution.cid_number.as_bytes(),
                &spec.institution.main_account,
                &indexed_admins(spec.min_members - 1),
            ),
            Error::<Test>::InvalidAdminsLen
        );
        assert_ok!(PublicAdmins::validate_admin_set_for_account(
            AdminAccountKind::PublicInstitution,
            spec.institution.code,
            spec.institution.cid_number.as_bytes(),
            &spec.institution.main_account,
            &indexed_admins(spec.min_members),
        ));
        assert_ok!(PublicAdmins::validate_admin_set_for_account(
            AdminAccountKind::PublicInstitution,
            spec.institution.code,
            b"ordinary-runtime-cid",
            &[9u8; 32],
            &admins(3),
        ));
    });
}

#[test]
fn first_member_body_composition_atomically_creates_admins_without_dynamic_threshold() {
    new_test_ext().execute_with(|| {
        let spec = primitives::institution_constraints::member_composition_specs()[0];
        let root = AccountId32::new(spec.institution.main_account);
        let members = indexed_admins(spec.min_members);
        assert_ok!(PublicAdmins::do_sync_active_admins_from_assignments(
            root.clone(),
            spec.institution.cid_number.as_bytes().to_vec(),
            spec.institution.code,
            members.clone(),
        ));
        assert_eq!(
            AdminAccounts::<Test>::get(root.clone())
                .expect("first composition creates admins")
                .admins
                .to_vec(),
            members
        );
        assert_eq!(
            internal_vote::ActiveDynamicThresholds::<Test>::get(spec.institution.code, root),
            None
        );
    });
}

#[test]
fn fixed_governance_assignment_sync_uses_compile_time_threshold_only() {
    new_test_ext().execute_with(|| {
        let root = account(30);
        let code = code_bytes("NRC");
        let cid_number: AdminCidNumber = b"GENESIS-NRC".to_vec().try_into().expect("cid fits");
        let initial: AdminsOf<Test> = admins(NRC_ADMIN_COUNT as u8)
            .try_into()
            .expect("fixed admins fit");
        AdminAccounts::<Test>::insert(
            root.clone(),
            admin_primitives::InstitutionAdminAccount {
                cid_number: cid_number.clone(),
                institution_code: code,
                admins: initial,
                status: AdminAccountStatus::Active,
            },
        );

        let replacement = (40..40 + NRC_ADMIN_COUNT as u8)
            .map(account)
            .collect::<Vec<_>>();
        assert_ok!(PublicAdmins::do_sync_active_admins_from_assignments(
            root.clone(),
            cid_number.to_vec(),
            code,
            replacement.clone(),
        ));

        assert_eq!(
            AdminAccounts::<Test>::get(root.clone())
                .expect("fixed admin account remains")
                .admins
                .to_vec(),
            replacement
        );
        // 固定治理阈值来自 institution_code 常量，不创建动态阈值 storage。
        assert_eq!(
            internal_vote::ActiveDynamicThresholds::<Test>::get(code, root),
            None
        );
    });
}
