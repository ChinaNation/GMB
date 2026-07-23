#![cfg(test)]

use super::*;
use admin_primitives::{AdminAccountKind, InstitutionAdminQuery};
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
    type MaxAdminsPerInstitution = ConstU32<1989>;
    type TimeProvider = TestTimeProvider;
    type WeightInfo = ();
    type TrackHandlers = (InternalVote, ());
    type LegislationVoteResultCallback = ();
    type ElectionVoteResultCallback = ();
}

impl internal_vote::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type InstitutionRoleProvider = ();
    type WeightInfo = ();
}

pub struct TestCitizenIdentityBinding;

impl admin_primitives::CitizenIdentityBindingQuery<AccountId32> for TestCitizenIdentityBinding {
    fn matches_citizen_account(cid_number: &[u8], account: &AccountId32) -> bool {
        cid_number == b"GZ000-CTZN6-198805200-2026" && account == &self::account(9)
    }
}

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxAdminsPerInstitution = ConstU32<1989>;
    type CitizenIdentityBinding = TestCitizenIdentityBinding;
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

/// admins 保存显示姓名和授权账户；岗位、任期、来源等由 entity 管理。
fn admins(count: u8) -> Vec<PublicAdmin<AccountId32>> {
    (0..count)
        .map(|seed| PublicAdmin {
            account_id: account(seed),
            cid_number: Default::default(),
            family_name: Default::default(),
            given_name: Default::default(),
        })
        .collect()
}

fn indexed_admins(count: u32) -> Vec<PublicAdmin<AccountId32>> {
    (0..count)
        .map(|index| {
            let mut raw = [0u8; 32];
            raw[..4].copy_from_slice(&index.to_le_bytes());
            PublicAdmin {
                account_id: AccountId32::new(raw),
                cid_number: Default::default(),
                family_name: Default::default(),
                given_name: Default::default(),
            }
        })
        .collect()
}

#[test]
fn public_admins_allow_temporarily_empty_identity_and_name_fields() {
    new_test_ext().execute_with(|| {
        let mut input = admins(3);
        input[0].family_name = Default::default();
        input[0].given_name = Default::default();
        assert_ok!(PublicAdmins::do_set_institution_admins(
            b"GD001-CGOV0-923456789-2026".to_vec(),
            code_bytes("CGOV"),
            AdminAccountKind::PublicInstitution,
            input,
        ));
        let cid =
            AdminCidNumber::try_from(b"GD001-CGOV0-923456789-2026".to_vec()).expect("cid fits");
        let stored = AdminAccounts::<Test>::get(cid).expect("admins exist");
        assert!(stored.admins[0].cid_number.is_empty());
        assert!(stored.admins[0].family_name.is_empty());
        assert!(stored.admins[0].given_name.is_empty());
    });
}

#[test]
fn public_admin_nonempty_citizen_cid_must_match_citizen_identity_binding() {
    new_test_ext().execute_with(|| {
        let mut valid = admins(1);
        valid[0].account_id = account(9);
        valid[0].cid_number = b"GZ000-CTZN6-198805200-2026"
            .to_vec()
            .try_into()
            .expect("citizen cid fits");
        assert_ok!(PublicAdmins::do_set_institution_admins(
            b"GD001-CGOV0-823456789-2026".to_vec(),
            code_bytes("CGOV"),
            AdminAccountKind::PublicInstitution,
            valid.clone(),
        ));

        valid[0].account_id = account(8);
        assert_noop!(
            PublicAdmins::do_set_institution_admins(
                b"GD001-CGOV0-723456789-2026".to_vec(),
                code_bytes("CGOV"),
                AdminAccountKind::PublicInstitution,
                valid,
            ),
            Error::<Test>::CitizenIdentityMismatch
        );
    });
}

#[test]
fn public_admins_accept_public_codes_and_reject_private_codes() {
    new_test_ext().execute_with(|| {
        let public_cid = b"GD001-CGOV0-123456789-2026".to_vec();
        assert_ok!(PublicAdmins::do_set_institution_admins(
            public_cid.clone(),
            code_bytes("CGOV"),
            AdminAccountKind::PublicInstitution,
            admins(3),
        ));
        let public_key: AdminCidNumber = public_cid.try_into().expect("cid fits");
        let stored = AdminAccounts::<Test>::get(public_key).expect("public admins exist");
        assert_eq!(stored.institution_code, code_bytes("CGOV"));
        assert_eq!(stored.admins.len(), 3);

        assert_ok!(PublicAdmins::do_set_institution_admins(
            b"GD001-UNIN0-223456789-2026".to_vec(),
            code_bytes("UNIN"),
            AdminAccountKind::PublicInstitution,
            admins(2),
        ));

        assert_noop!(
            PublicAdmins::do_set_institution_admins(
                b"GD001-SFLP0-323456789-2026".to_vec(),
                code_bytes("SFLP"),
                AdminAccountKind::PublicInstitution,
                admins(3),
            ),
            Error::<Test>::InvalidAdminAccountKind
        );
    });
}

#[test]
fn public_admins_activate_and_query_active_admins() {
    new_test_ext().execute_with(|| {
        let cid = b"GD001-CGOV0-423456789-2026".to_vec();
        assert_ok!(PublicAdmins::do_set_institution_admins(
            cid.clone(),
            code_bytes("CGOV"),
            AdminAccountKind::PublicInstitution,
            admins(3),
        ));

        assert!(PublicAdmins::is_institution_admin(
            code_bytes("CGOV"),
            &cid,
            &account(0)
        ));
        assert_eq!(
            PublicAdmins::institution_admins_len(code_bytes("CGOV"), &cid),
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
            PublicAdmins::do_set_institution_admins(
                fixed.cid_number.as_bytes().to_vec(),
                code_bytes("NRC"),
                AdminAccountKind::PublicInstitution,
                admins(3),
            ),
            Error::<Test>::InvalidAdminsLen
        );

        assert_ok!(PublicAdmins::do_set_institution_admins(
            fixed.cid_number.as_bytes().to_vec(),
            code_bytes("NRC"),
            AdminAccountKind::PublicInstitution,
            admins(NRC_ADMIN_COUNT as u8),
        ));

        // 机构码相同但不是创世 CID 时，不得扩大固定人数保护范围。
        assert_ok!(PublicAdmins::do_set_institution_admins(
            b"LN001-NRC0G-123456789-2026".to_vec(),
            code_bytes("NRC"),
            AdminAccountKind::PublicInstitution,
            admins(3),
        ));
    });
}

#[test]
fn permanent_member_body_protection_is_scoped_to_genesis_cid() {
    new_test_ext().execute_with(|| {
        let spec = primitives::institution_constraints::member_composition_specs()[0];
        assert_noop!(
            PublicAdmins::do_set_institution_admins(
                spec.institution.cid_number.as_bytes().to_vec(),
                spec.institution.code,
                AdminAccountKind::PublicInstitution,
                indexed_admins(spec.min_members - 1),
            ),
            Error::<Test>::InvalidAdminsLen
        );
        assert_ok!(PublicAdmins::do_set_institution_admins(
            spec.institution.cid_number.as_bytes().to_vec(),
            spec.institution.code,
            AdminAccountKind::PublicInstitution,
            indexed_admins(spec.min_members),
        ));
        // 固定创世身份只保护完整 CID；不得把保护扩大到同机构码的其它机构。
        assert_ok!(PublicAdmins::do_set_institution_admins(
            b"LN001-NLG0G-123456789-2026".to_vec(),
            spec.institution.code,
            AdminAccountKind::PublicInstitution,
            admins(3),
        ));
    });
}

#[test]
fn member_body_admins_are_registered_without_owning_institution_threshold() {
    new_test_ext().execute_with(|| {
        let spec = primitives::institution_constraints::member_composition_specs()[0];
        let members = indexed_admins(spec.min_members);
        assert_ok!(PublicAdmins::do_set_institution_admins(
            spec.institution.cid_number.as_bytes().to_vec(),
            spec.institution.code,
            AdminAccountKind::PublicInstitution,
            members.clone(),
        ));
        assert_eq!(
            AdminAccounts::<Test>::get(
                AdminCidNumber::try_from(spec.institution.cid_number.as_bytes().to_vec())
                    .expect("cid fits")
            )
            .expect("independently registered admins exist")
            .admins
            .to_vec(),
            members
        );
    });
}

#[test]
fn fixed_governance_admin_update_only_replaces_people_records() {
    new_test_ext().execute_with(|| {
        let fixed = primitives::governance_skeleton::fixed_institutions()
            .into_iter()
            .find(|institution| institution.code == code_bytes("NRC"))
            .expect("NRC genesis identity");
        let code = fixed.code;
        let cid_number: AdminCidNumber = fixed
            .cid_number
            .as_bytes()
            .to_vec()
            .try_into()
            .expect("cid fits");
        assert_ok!(PublicAdmins::do_set_institution_admins(
            cid_number.to_vec(),
            code,
            AdminAccountKind::PublicInstitution,
            admins(NRC_ADMIN_COUNT as u8),
        ));

        let replacement = (40..40 + NRC_ADMIN_COUNT as u8)
            .map(|seed| PublicAdmin {
                account_id: account(seed),
                cid_number: Default::default(),
                family_name: Default::default(),
                given_name: Default::default(),
            })
            .collect::<Vec<_>>();
        assert_ok!(PublicAdmins::do_set_institution_admins(
            cid_number.to_vec(),
            code,
            AdminAccountKind::PublicInstitution,
            replacement.clone(),
        ));

        assert_eq!(
            AdminAccounts::<Test>::get(cid_number.clone())
                .expect("fixed admin account remains")
                .admins
                .to_vec(),
            replacement
        );
    });
}
