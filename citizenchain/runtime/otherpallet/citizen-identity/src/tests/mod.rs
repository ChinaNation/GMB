#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok, derive_impl, parameter_types};
use frame_system as system;
use sp_runtime::{traits::IdentityLookup, BuildStorage};

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
    pub type CitizenIdentity = super;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl system::Config for Test {
    type Block = Block;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
}

parameter_types! {
    pub const MaxCitizenSignatureLength: u32 = 64;
}

/// 固定链上时间:2026-07-02 00:00 UTC(UTC+8 为 2026-07-02 08:00,
/// 折算日期 20260702),让默认夹具护照(20260630-20360630)处于有效期窗口。
pub struct FixedTime;
impl frame_support::traits::UnixTime for FixedTime {
    fn now() -> core::time::Duration {
        core::time::Duration::from_secs(1_782_950_400)
    }
}

pub struct TestCitizenIdentityAuthority;
impl CitizenIdentityAuthority<u64, pallet::SignatureOf<Test>> for TestCitizenIdentityAuthority {
    fn can_manage_voting_identity(
        registrar: &u64,
        registrar_account: &u64,
        residence_province_code: &[u8],
        residence_city_code: &[u8],
        _level: CitizenIdentityLevel,
    ) -> bool {
        *registrar == 100
            && *registrar_account == 200
            && residence_province_code == b"43"
            && residence_city_code == b"4301"
    }

    fn verify_citizen_signature(
        _wallet_account: &u64,
        _payload: &[u8],
        signature: &pallet::SignatureOf<Test>,
    ) -> bool {
        signature.as_slice() == b"valid"
    }
}

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxCitizenSignatureLength = MaxCitizenSignatureLength;
    type CitizenIdentityAuthority = TestCitizenIdentityAuthority;
    type OnVotingIdentityRegistered = ();
    type TimeProvider = FixedTime;
    type WeightInfo = ();
}

fn new_test_ext() -> sp_io::TestExternalities {
    let storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("frame system genesis storage should build");
    let mut ext = sp_io::TestExternalities::new(storage);
    ext.execute_with(|| System::set_block_number(10));
    ext
}

fn code(bytes: &[u8]) -> AreaCodeBound {
    bytes.to_vec().try_into().expect("area code should fit")
}

fn cid(bytes: &[u8]) -> CidNumberBound {
    bytes.to_vec().try_into().expect("cid number should fit")
}

fn name(bytes: &[u8]) -> CitizenNameBound {
    bytes.to_vec().try_into().expect("citizen name should fit")
}

fn valid_signature() -> pallet::SignatureOf<Test> {
    b"valid".to_vec().try_into().expect("signature should fit")
}

fn voting_payload(wallet_account: u64, cid_number: &[u8]) -> VotingIdentityPayload<u64> {
    VotingIdentityPayload {
        cid_number: cid(cid_number),
        wallet_account,
        citizen_age_years: 18,
        passport_valid_from: 20260630,
        passport_valid_until: 20360630,
        citizen_status: CitizenStatus::Normal,
        residence_province_code: code(b"43"),
        residence_city_code: code(b"4301"),
        residence_town_code: code(b"4301001"),
    }
}

fn candidate_payload(wallet_account: u64, cid_number: &[u8]) -> CandidateIdentityPayload<u64> {
    CandidateIdentityPayload {
        voting: voting_payload(wallet_account, cid_number),
        birth_province_code: code(b"43"),
        birth_city_code: code(b"4301"),
        birth_town_code: code(b"4301001"),
        citizen_full_name: name(b"Citizen One"),
        citizen_sex: CitizenSex::Female,
    }
}

fn town_scope() -> PopulationScope {
    PopulationScope::Town(code(b"43"), code(b"4301"), code(b"4301001"))
}

#[test]
fn register_voting_identity_stores_identity_and_counts_scope() {
    new_test_ext().execute_with(|| {
        assert_ok!(CitizenIdentity::register_voting_identity(
            RuntimeOrigin::signed(100),
            200,
            voting_payload(1, b"CTZN-0001"),
            valid_signature(),
        ));

        assert!(VotingIdentityByAccount::<Test>::contains_key(1));
        assert_eq!(AccountByCid::<Test>::get(cid(b"CTZN-0001")), Some(1));
        assert_eq!(CountryVotingCount::<Test>::get(), 1);
        assert_eq!(ProvinceVotingCount::<Test>::get(code(b"43")), 1);
        assert!(CitizenIdentity::can_vote(&1, &town_scope()));
    });
}

#[test]
fn duplicate_cid_cannot_move_to_another_wallet_account() {
    new_test_ext().execute_with(|| {
        assert_ok!(CitizenIdentity::register_voting_identity(
            RuntimeOrigin::signed(100),
            200,
            voting_payload(1, b"CTZN-0001"),
            valid_signature(),
        ));

        assert_noop!(
            CitizenIdentity::register_voting_identity(
                RuntimeOrigin::signed(100),
                200,
                voting_payload(2, b"CTZN-0001"),
                valid_signature(),
            ),
            Error::<Test>::CidAlreadyRegisteredToAnotherAccount
        );
    });
}

#[test]
fn updating_same_account_replaces_cid_without_double_counting() {
    new_test_ext().execute_with(|| {
        assert_ok!(CitizenIdentity::register_voting_identity(
            RuntimeOrigin::signed(100),
            200,
            voting_payload(1, b"CTZN-0001"),
            valid_signature(),
        ));
        assert_ok!(CitizenIdentity::update_voting_identity(
            RuntimeOrigin::signed(100),
            200,
            voting_payload(1, b"CTZN-0002"),
            valid_signature(),
        ));

        assert_eq!(AccountByCid::<Test>::get(cid(b"CTZN-0001")), None);
        assert_eq!(AccountByCid::<Test>::get(cid(b"CTZN-0002")), Some(1));
        assert_eq!(CountryVotingCount::<Test>::get(), 1);
        assert_eq!(
            TownVotingCount::<Test>::get((code(b"43"), code(b"4301"), code(b"4301001"))),
            1
        );
    });
}

#[test]
fn candidate_identity_requires_full_profile_and_enables_candidate_reader() {
    new_test_ext().execute_with(|| {
        assert_ok!(CitizenIdentity::upgrade_to_candidate_identity(
            RuntimeOrigin::signed(100),
            200,
            candidate_payload(1, b"CTZN-CANDIDATE"),
            valid_signature(),
        ));

        assert!(CandidateIdentityByAccount::<Test>::contains_key(1));
        assert!(CitizenIdentity::can_vote(&1, &town_scope()));
        assert!(CitizenIdentity::can_be_candidate(&1, &town_scope()));
    });
}

#[test]
fn revoke_identity_marks_status_and_removes_population_count() {
    new_test_ext().execute_with(|| {
        assert_ok!(CitizenIdentity::upgrade_to_candidate_identity(
            RuntimeOrigin::signed(100),
            200,
            candidate_payload(1, b"CTZN-REVOKE"),
            valid_signature(),
        ));
        assert_ok!(CitizenIdentity::revoke_identity(
            RuntimeOrigin::signed(100),
            200,
            cid(b"CTZN-REVOKE"),
        ));

        let stored = VotingIdentityByAccount::<Test>::get(1).expect("identity should remain");
        assert_eq!(stored.citizen_status, CitizenStatus::Revoked);
        assert!(!CandidateIdentityByAccount::<Test>::contains_key(1));
        assert_eq!(CountryVotingCount::<Test>::get(), 0);
        assert!(!CitizenIdentity::can_vote(&1, &town_scope()));
    });
}

#[test]
fn population_snapshot_reads_current_scope_count() {
    new_test_ext().execute_with(|| {
        assert_ok!(CitizenIdentity::register_voting_identity(
            RuntimeOrigin::signed(100),
            200,
            voting_payload(1, b"CTZN-0001"),
            valid_signature(),
        ));

        assert_ok!(CitizenIdentity::start_population_snapshot(
            RuntimeOrigin::signed(1),
            town_scope(),
        ));

        let snapshot = PopulationSnapshots::<Test>::get(0).expect("snapshot should exist");
        assert_eq!(snapshot.eligible_total, 1);
        assert_eq!(NextSnapshotId::<Test>::get(), 1);
    });
}

#[test]
fn invalid_citizen_code_is_rejected() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            CitizenIdentity::register_voting_identity(
                RuntimeOrigin::signed(100),
                200,
                voting_payload(1, b"OLD-0001"),
                valid_signature(),
            ),
            Error::<Test>::InvalidCitizenCode
        );
    });
}

#[test]
fn expired_passport_cannot_vote_but_still_counts_in_population() {
    new_test_ext().execute_with(|| {
        let mut payload = voting_payload(1, b"CTZN-EXPIRED");
        payload.passport_valid_from = 20200101;
        payload.passport_valid_until = 20250101;

        assert_ok!(CitizenIdentity::register_voting_identity(
            RuntimeOrigin::signed(100),
            200,
            payload,
            valid_signature(),
        ));

        // 计数器按状态增量维护,护照过期不减分母(设计约束,见 lib.rs 注释)。
        assert_eq!(CountryVotingCount::<Test>::get(), 1);
        // 但投票资格被护照有效期窗口实时拦截。
        assert!(!CitizenIdentity::can_vote(&1, &town_scope()));
        assert!(!CitizenIdentity::can_be_candidate(&1, &town_scope()));
    });
}

#[test]
fn not_yet_valid_passport_cannot_vote() {
    new_test_ext().execute_with(|| {
        let mut payload = voting_payload(1, b"CTZN-FUTURE");
        payload.passport_valid_from = 20300101;
        payload.passport_valid_until = 20400101;

        assert_ok!(CitizenIdentity::register_voting_identity(
            RuntimeOrigin::signed(100),
            200,
            payload,
            valid_signature(),
        ));

        assert!(!CitizenIdentity::can_vote(&1, &town_scope()));
    });
}

#[test]
fn candidate_identity_stores_sex_and_public_profile() {
    new_test_ext().execute_with(|| {
        assert_ok!(CitizenIdentity::upgrade_to_candidate_identity(
            RuntimeOrigin::signed(100),
            200,
            candidate_payload(1, b"CTZN-SEX"),
            valid_signature(),
        ));

        let stored = CandidateIdentityByAccount::<Test>::get(1).expect("candidate stored");
        assert_eq!(stored.citizen_sex, CitizenSex::Female);
        assert_eq!(stored.citizen_full_name, name(b"Citizen One"));
    });
}

#[test]
fn current_date_int_matches_fixed_time() {
    new_test_ext().execute_with(|| {
        // FixedTime = 2026-07-02 00:00 UTC → UTC+8 折算 20260702。
        assert_eq!(CitizenIdentity::current_date_int(), 20260702);
    });
}

#[test]
fn under_sixteen_cannot_register_onchain_identity() {
    new_test_ext().execute_with(|| {
        let mut payload = voting_payload(1, b"CTZN-UNDERAGE");
        payload.citizen_age_years = 15;

        assert_noop!(
            CitizenIdentity::register_voting_identity(
                RuntimeOrigin::signed(100),
                200,
                payload,
                valid_signature(),
            ),
            Error::<Test>::UnderVotingAge
        );
    });
}
