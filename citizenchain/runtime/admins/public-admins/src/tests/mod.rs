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

/// admins 保存显示姓名和授权钱包；岗位、任期、来源等由 entity 管理。
fn admins(count: u8) -> Vec<Admin<AccountId32>> {
    (0..count)
        .map(|seed| Admin {
            admin_account: account(seed),
            family_name: "管理".as_bytes().to_vec().try_into().expect("name fits"),
            given_name: "员".as_bytes().to_vec().try_into().expect("name fits"),
        })
        .collect()
}

fn indexed_admins(count: u32) -> Vec<Admin<AccountId32>> {
    (0..count)
        .map(|index| {
            let mut raw = [0u8; 32];
            raw[..4].copy_from_slice(&index.to_le_bytes());
            Admin {
                admin_account: AccountId32::new(raw),
                family_name: "管理".as_bytes().to_vec().try_into().expect("name fits"),
                given_name: "员".as_bytes().to_vec().try_into().expect("name fits"),
            }
        })
        .collect()
}

#[test]
fn public_admins_normalize_missing_person_name_before_storage() {
    new_test_ext().execute_with(|| {
        let mut input = admins(3);
        input[0].family_name = Default::default();
        input[0].given_name = Default::default();
        assert_ok!(PublicAdmins::do_set_institution_admins(
            b"GD001-CGOV0-923456789-2026".to_vec(),
            code_bytes("CGOV"),
            AdminAccountKind::PublicInstitution,
            input,
            2,
        ));
        let cid =
            AdminCidNumber::try_from(b"GD001-CGOV0-923456789-2026".to_vec()).expect("cid fits");
        let stored = AdminAccounts::<Test>::get(cid).expect("admins exist");
        assert_eq!(stored.admins[0].family_name.as_slice(), "管理".as_bytes());
        assert_eq!(stored.admins[0].given_name.as_slice(), "员".as_bytes());
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
            2,
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
            2,
        ));

        assert_noop!(
            PublicAdmins::do_set_institution_admins(
                b"GD001-SFLP0-323456789-2026".to_vec(),
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
        let cid = b"GD001-CGOV0-423456789-2026".to_vec();
        assert_ok!(PublicAdmins::do_set_institution_admins(
            cid.clone(),
            code_bytes("CGOV"),
            AdminAccountKind::PublicInstitution,
            admins(3),
            2,
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
        let nrc_threshold =
            primitives::cid::code::fixed_governance_pass_threshold(&code_bytes("NRC"))
                .expect("NRC threshold");
        assert_noop!(
            PublicAdmins::do_set_institution_admins(
                fixed.cid_number.as_bytes().to_vec(),
                code_bytes("NRC"),
                AdminAccountKind::PublicInstitution,
                admins(3),
                nrc_threshold,
            ),
            Error::<Test>::InvalidAdminsLen
        );

        assert_ok!(PublicAdmins::do_set_institution_admins(
            fixed.cid_number.as_bytes().to_vec(),
            code_bytes("NRC"),
            AdminAccountKind::PublicInstitution,
            admins(NRC_ADMIN_COUNT as u8),
            nrc_threshold,
        ));

        // 机构码相同但不是创世 CID 时，不得扩大固定人数保护范围。
        assert_ok!(PublicAdmins::do_set_institution_admins(
            b"LN001-NRC0G-123456789-2026".to_vec(),
            code_bytes("NRC"),
            AdminAccountKind::PublicInstitution,
            admins(3),
            nrc_threshold,
        ));
    });
}

#[test]
fn permanent_member_body_code_rejects_non_genesis_cid() {
    new_test_ext().execute_with(|| {
        let spec = primitives::institution_constraints::member_composition_specs()[0];
        assert_noop!(
            PublicAdmins::do_set_institution_admins(
                spec.institution.cid_number.as_bytes().to_vec(),
                spec.institution.code,
                AdminAccountKind::PublicInstitution,
                indexed_admins(spec.min_members - 1),
                spec.min_members / 2 + 1,
            ),
            Error::<Test>::InvalidAdminsLen
        );
        assert_ok!(PublicAdmins::do_set_institution_admins(
            spec.institution.cid_number.as_bytes().to_vec(),
            spec.institution.code,
            AdminAccountKind::PublicInstitution,
            indexed_admins(spec.min_members),
            spec.min_members / 2 + 1,
        ));
        // 永久单例机构码不能由另一个 CID 占用，避免同一制度身份出现第二真源。
        assert_noop!(
            PublicAdmins::do_set_institution_admins(
                b"LN001-NLG0G-123456789-2026".to_vec(),
                spec.institution.code,
                AdminAccountKind::PublicInstitution,
                admins(3),
                2,
            ),
            internal_vote::Error::<Test>::InvalidInternalCode
        );
    });
}

#[test]
fn member_body_admins_are_registered_independently_without_dynamic_threshold() {
    new_test_ext().execute_with(|| {
        let spec = primitives::institution_constraints::member_composition_specs()[0];
        let members = indexed_admins(spec.min_members);
        assert_ok!(PublicAdmins::do_set_institution_admins(
            spec.institution.cid_number.as_bytes().to_vec(),
            spec.institution.code,
            AdminAccountKind::PublicInstitution,
            members.clone(),
            spec.min_members / 2 + 1,
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
        assert_eq!(
            internal_vote::ActiveInstitutionThresholds::<Test>::get(
                AdminCidNumber::try_from(spec.institution.cid_number.as_bytes().to_vec())
                    .expect("cid fits")
            ),
            None
        );
    });
}

#[test]
fn fixed_governance_admin_update_uses_compile_time_threshold_only() {
    new_test_ext().execute_with(|| {
        let fixed = primitives::governance_skeleton::fixed_institutions()
            .into_iter()
            .find(|institution| institution.code == code_bytes("NRC"))
            .expect("NRC genesis identity");
        let code = fixed.code;
        let fixed_threshold =
            primitives::cid::code::fixed_governance_pass_threshold(&code).expect("NRC threshold");
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
            fixed_threshold,
        ));

        let replacement = (40..40 + NRC_ADMIN_COUNT as u8)
            .map(|seed| Admin {
                admin_account: account(seed),
                family_name: "管理".as_bytes().to_vec().try_into().expect("name fits"),
                given_name: "员".as_bytes().to_vec().try_into().expect("name fits"),
            })
            .collect::<Vec<_>>();
        assert_ok!(PublicAdmins::do_set_institution_admins(
            cid_number.to_vec(),
            code,
            AdminAccountKind::PublicInstitution,
            replacement.clone(),
            fixed_threshold,
        ));

        assert_eq!(
            AdminAccounts::<Test>::get(cid_number.clone())
                .expect("fixed admin account remains")
                .admins
                .to_vec(),
            replacement
        );
        // 固定治理阈值来自 institution_code 常量，不创建动态阈值 storage。
        assert_eq!(
            internal_vote::ActiveInstitutionThresholds::<Test>::get(cid_number),
            None
        );
    });
}
