#![cfg(test)]

use super::*;
/// NodeGuard 镜像 `CidRecord` 的完整字段序和状态判别值；任一变化都属于 storage 契约变化。
#[test]
fn cid_record_scale_contract_matches_node_guard() {
    use codec::Encode;

    assert_eq!(CidRecordStatus::Active.encode(), vec![0]);
    assert_eq!(CidRecordStatus::Revoked.encode(), vec![1]);

    let record = CidRecord {
        registrar_cid_number: registrar_cid_number(),
        commitment: [2u8; 32],
        residence_province_code: b"GD".to_vec().try_into().expect("province"),
        residence_city_code: b"001".to_vec().try_into().expect("city"),
        status: CidRecordStatus::Revoked,
        registered_at: 8u32,
        revoked_at: Some(9u32),
    };
    assert_eq!(
        record.encode(),
        (
            registrar_cid_number().to_vec(),
            [2u8; 32],
            b"GD".to_vec(),
            b"001".to_vec(),
            CidRecordStatus::Revoked,
            8u32,
            Some(9u32),
        )
            .encode()
    );
}
use frame_support::{assert_noop, assert_ok, derive_impl, parameter_types, traits::Hooks};
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
    pub const MaxPopulationDaysPerBlock: u32 = 366;
    pub const MaxPopulationTransitionsPerBlock: u32 = 2;
    pub MaxPopulationMaintenanceWeightPerBlock: frame_support::weights::Weight = frame_support::weights::Weight::MAX;
}

/// 固定链上时间:2026-07-02 00:00 UTC(UTC+8 为 2026-07-02 08:00,
/// 折算日期 20260702),让默认夹具护照(20260630-20360630)处于有效期窗口。
pub struct FixedTime;
impl frame_support::traits::UnixTime for FixedTime {
    fn now() -> core::time::Duration {
        TEST_TIME_SECS.with(|value| core::time::Duration::from_secs(value.get()))
    }
}

std::thread_local! {
    static TEST_TIME_SECS: core::cell::Cell<u64> = const { core::cell::Cell::new(1_782_950_400) };
}

fn set_day_offset(days: i64) {
    TEST_TIME_SECS.with(|value| {
        let delta = days.unsigned_abs().saturating_mul(86_400);
        let timestamp = if days >= 0 {
            1_782_950_400u64.saturating_add(delta)
        } else {
            1_782_950_400u64.saturating_sub(delta)
        };
        value.set(timestamp);
    });
}

pub struct TestCitizenIdentityAuthority;
impl CitizenIdentityAuthority<u64, pallet::SignatureOf<Test>> for TestCitizenIdentityAuthority {
    fn can_manage_voting_identity(
        registrar: &u64,
        actor_cid_number: &[u8],
        actor_role_code: &[u8],
        residence_province_code: &[u8],
        residence_city_code: &[u8],
        _level: CitizenIdentityLevel,
        _action_code: u32,
    ) -> bool {
        *registrar == 100
            && actor_cid_number == registrar_cid_number().as_slice()
            && actor_role_code == registrar_role_code().as_slice()
            && residence_province_code == b"43"
            && residence_city_code == b"4301"
    }

    fn verify_citizen_signature(
        _account_id: &u64,
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
    type MaxPopulationDaysPerBlock = MaxPopulationDaysPerBlock;
    type MaxPopulationTransitionsPerBlock = MaxPopulationTransitionsPerBlock;
    type MaxPopulationMaintenanceWeightPerBlock = MaxPopulationMaintenanceWeightPerBlock;
    type WeightInfo = ();
}

fn new_test_ext() -> sp_io::TestExternalities {
    let storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("frame system genesis storage should build");
    let mut ext = sp_io::TestExternalities::new(storage);
    set_day_offset(0);
    ext.execute_with(|| {
        System::set_block_number(10);
        PopulationReadyDate::<Test>::put(20260702);
    });
    ext
}

fn code(bytes: &[u8]) -> AreaCodeBound {
    bytes.to_vec().try_into().expect("area code should fit")
}

fn cid(bytes: &[u8]) -> CidNumberBound {
    bytes.to_vec().try_into().expect("cid number should fit")
}

/// 测试注册局机构 CID；管理员账户 100 只作为外层 origin。
fn registrar_cid_number() -> CidNumberBound {
    cid(primitives::cid::china::china_zf::CHINA_ZF[5]
        .cid_number
        .as_bytes())
}

/// 按 tag 生成真实规则公民 CID 号(格式/校验和/机构码全合规)。
fn citizen_cid_number(tag: &str) -> alloc::vec::Vec<u8> {
    primitives::cid::generator::generate_cid_number(
        primitives::cid::generator::GenerateCidNumberInput {
            public_key: tag,
            p1: "1",
            province_code: "GD",
            province_name: "广东省",
            city_code: "001",
            city_name: "荔湾市",
            year: "2026",
            institution: "CTZN",
        },
    )
    .expect("citizen cid should generate")
    .into_bytes()
}

fn family_name(bytes: &[u8]) -> FamilyName {
    bytes.to_vec().try_into().expect("family name should fit")
}

fn given_name(bytes: &[u8]) -> GivenName {
    bytes.to_vec().try_into().expect("given name should fit")
}

/// 测试注册局管理员必须以明确岗位主体发起业务，管理员账户本身不产生权限。
fn registrar_role_code() -> RoleCodeBound {
    b"PROVINCE_COMMISSIONER_43"
        .to_vec()
        .try_into()
        .expect("registrar role code should fit")
}

/// 测试承诺哈希:由 tag 填充,幂等续用用同值。
fn commitment_for(tag: &str) -> [u8; 32] {
    let mut c = [0u8; 32];
    let bytes = tag.as_bytes();
    let n = bytes.len().min(32);
    c[..n].copy_from_slice(&bytes[..n]);
    c
}

/// 占号先行:身份写入前必须先占号(注册局 CID + 管理员 100,作用域 43/4301)。
fn occupy_tag(tag: &str) {
    assert_ok!(CitizenIdentity::occupy_cid(
        RuntimeOrigin::signed(100),
        registrar_cid_number(),
        registrar_role_code(),
        cid(&citizen_cid_number(tag)),
        commitment_for(tag),
        code(b"43"),
        code(b"4301"),
    ));
}

/// 按 tag 生成真实规则公权机构号(市政府 CGOV),供家族拒绝用例。
fn public_cid_number(tag: &str) -> alloc::vec::Vec<u8> {
    primitives::cid::generator::generate_cid_number(
        primitives::cid::generator::GenerateCidNumberInput {
            public_key: tag,
            p1: "0",
            province_code: "GD",
            province_name: "广东省",
            city_code: "001",
            city_name: "荔湾市",
            year: "2026",
            institution: "CGOV",
        },
    )
    .expect("public cid should generate")
    .into_bytes()
}

fn valid_signature() -> pallet::SignatureOf<Test> {
    b"valid".to_vec().try_into().expect("signature should fit")
}

fn voting_payload(account_id: u64, cid_number: &[u8]) -> VotingIdentityPayload<u64> {
    VotingIdentityPayload {
        cid_number: cid(cid_number),
        account_id,
        citizen_age_years: 18,
        passport_valid_from: 20260630,
        passport_valid_until: 20360630,
        citizen_status: CitizenStatus::Normal,
        residence_province_code: code(b"43"),
        residence_city_code: code(b"4301"),
        residence_town_code: code(b"4301001"),
    }
}

fn candidate_payload(account_id: u64, cid_number: &[u8]) -> CandidateIdentityPayload<u64> {
    CandidateIdentityPayload {
        voting: voting_payload(account_id, cid_number),
        birth_province_code: code(b"43"),
        birth_city_code: code(b"4301"),
        birth_town_code: code(b"4301001"),
        family_name: family_name(b"Citizen"),
        given_name: given_name(b"One"),
        citizen_sex: CitizenSex::Female,
        // 固定时间 20260702 下年龄 26 周岁,满足最小年龄。
        birth_date: 20000131,
    }
}

fn town_scope() -> PopulationScope {
    PopulationScope::Town(code(b"43"), code(b"4301"), code(b"4301001"))
}

#[test]
fn register_voting_identity_stores_identity_and_counts_scope() {
    new_test_ext().execute_with(|| {
        // 占号先行:身份写入前置。
        occupy_tag("0001");

        assert_ok!(CitizenIdentity::register_voting_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            registrar_role_code(),
            voting_payload(1, &citizen_cid_number("0001")),
            valid_signature(),
        ));

        assert!(VotingIdentityByCid::<Test>::contains_key(cid(
            &citizen_cid_number("0001")
        )));
        assert_eq!(
            AccountIdByCid::<Test>::get(cid(&citizen_cid_number("0001"))),
            Some(1)
        );
        assert_eq!(
            CidByAccountId::<Test>::get(1),
            Some(cid(&citizen_cid_number("0001")))
        );
        assert_eq!(CountryVotingCount::<Test>::get(), 1);
        assert_eq!(ProvinceVotingCount::<Test>::get(code(b"43")), 1);
        assert!(CitizenIdentity::voting_subject(&1, &town_scope()).is_some());
    });
}

#[test]
fn duplicate_cid_cannot_move_to_another_account_id() {
    new_test_ext().execute_with(|| {
        // 占号先行:身份写入前置。
        occupy_tag("0001");

        assert_ok!(CitizenIdentity::register_voting_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            registrar_role_code(),
            voting_payload(1, &citizen_cid_number("0001")),
            valid_signature(),
        ));

        assert_noop!(
            CitizenIdentity::register_voting_identity(
                RuntimeOrigin::signed(100),
                registrar_cid_number(),
                registrar_role_code(),
                voting_payload(2, &citizen_cid_number("0001")),
                valid_signature(),
            ),
            Error::<Test>::VotingIdentityAlreadyExists
        );
    });
}

#[test]
fn updating_identity_cannot_replace_permanent_cid() {
    new_test_ext().execute_with(|| {
        // 占号先行:身份写入前置。
        occupy_tag("0001");
        occupy_tag("0002");

        assert_ok!(CitizenIdentity::register_voting_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            registrar_role_code(),
            voting_payload(1, &citizen_cid_number("0001")),
            valid_signature(),
        ));
        assert_noop!(
            CitizenIdentity::update_voting_identity(
                RuntimeOrigin::signed(100),
                registrar_cid_number(),
                registrar_role_code(),
                voting_payload(1, &citizen_cid_number("0002")),
                valid_signature(),
            ),
            Error::<Test>::CidAccountIdBindingMismatch
        );
        assert_eq!(
            AccountIdByCid::<Test>::get(cid(&citizen_cid_number("0001"))),
            Some(1)
        );
        assert_eq!(
            AccountIdByCid::<Test>::get(cid(&citizen_cid_number("0002"))),
            None
        );
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
        // 占号先行:身份写入前置。
        occupy_tag("CANDIDATE");

        assert_ok!(CitizenIdentity::upgrade_to_candidate_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            registrar_role_code(),
            candidate_payload(1, &citizen_cid_number("CANDIDATE")),
            valid_signature(),
        ));

        assert!(CandidateIdentityByCid::<Test>::contains_key(cid(
            &citizen_cid_number("CANDIDATE")
        )));
        assert!(CitizenIdentity::voting_subject(&1, &town_scope()).is_some());
        assert!(CitizenIdentity::candidate_subject(&1, &town_scope()).is_some());
    });
}

#[test]
fn citizen_subject_requires_active_bidirectional_cid_account_id_binding() {
    new_test_ext().execute_with(|| {
        occupy_tag("SUBJECT");
        let cid_number = citizen_cid_number("SUBJECT");
        assert_ok!(CitizenIdentity::register_voting_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            registrar_role_code(),
            voting_payload(1, &cid_number),
            valid_signature(),
        ));

        assert_eq!(
            CitizenIdentity::citizen_subject(&1),
            Some(CitizenSubject {
                cid_number: cid(&cid_number),
                account_id: 1,
            })
        );

        // 反向绑定与账户存储键不一致时 fail-closed，不能只凭裸账户形成主体。
        AccountIdByCid::<Test>::insert(cid(&cid_number), 2);
        assert_eq!(CitizenIdentity::citizen_subject(&1), None);
        assert!(CitizenIdentity::voting_subject(&1, &town_scope()).is_none());
    });
}

#[test]
fn citizen_subject_rejects_revoked_identity_and_cid() {
    new_test_ext().execute_with(|| {
        occupy_tag("SUBJECT-REVOKED");
        let cid_number = citizen_cid_number("SUBJECT-REVOKED");
        assert_ok!(CitizenIdentity::register_voting_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            registrar_role_code(),
            voting_payload(1, &cid_number),
            valid_signature(),
        ));
        assert_ok!(CitizenIdentity::revoke_cid(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            registrar_role_code(),
            cid(&cid_number),
        ));

        assert_eq!(CitizenIdentity::citizen_subject(&1), None);
    });
}

#[test]
fn candidate_identity_requires_family_name_and_given_name_separately() {
    new_test_ext().execute_with(|| {
        occupy_tag("EMPTY-FAMILY");
        let mut empty_family = candidate_payload(1, &citizen_cid_number("EMPTY-FAMILY"));
        empty_family.family_name = Default::default();
        assert_noop!(
            CitizenIdentity::upgrade_to_candidate_identity(
                RuntimeOrigin::signed(100),
                registrar_cid_number(),
                registrar_role_code(),
                empty_family,
                valid_signature(),
            ),
            Error::<Test>::EmptyFamilyName
        );

        occupy_tag("EMPTY-GIVEN");
        let mut empty_given = candidate_payload(2, &citizen_cid_number("EMPTY-GIVEN"));
        empty_given.given_name = Default::default();
        assert_noop!(
            CitizenIdentity::upgrade_to_candidate_identity(
                RuntimeOrigin::signed(100),
                registrar_cid_number(),
                registrar_role_code(),
                empty_given,
                valid_signature(),
            ),
            Error::<Test>::EmptyGivenName
        );
    });
}

#[test]
fn revoke_identity_marks_status_and_removes_effective_population() {
    new_test_ext().execute_with(|| {
        // 占号先行:身份写入前置。
        occupy_tag("REVOKE");

        assert_ok!(CitizenIdentity::upgrade_to_candidate_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            registrar_role_code(),
            candidate_payload(1, &citizen_cid_number("REVOKE")),
            valid_signature(),
        ));
        assert_ok!(CitizenIdentity::revoke_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            registrar_role_code(),
            cid(&citizen_cid_number("REVOKE")),
        ));

        let stored = VotingIdentityByCid::<Test>::get(cid(&citizen_cid_number("REVOKE")))
            .expect("identity should remain");
        assert_eq!(stored.citizen_status, CitizenStatus::Revoked);
        assert!(!CandidateIdentityByCid::<Test>::contains_key(cid(
            &citizen_cid_number("REVOKE")
        )));
        assert_eq!(CountryVotingCount::<Test>::get(), 0);
        assert!(CitizenIdentity::voting_subject(&1, &town_scope()).is_none());
    });
}

#[test]
fn population_data_reads_current_scope_count() {
    new_test_ext().execute_with(|| {
        // 占号先行:身份写入前置。
        occupy_tag("0001");

        assert_ok!(CitizenIdentity::register_voting_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            registrar_role_code(),
            voting_payload(1, &citizen_cid_number("0001")),
            valid_signature(),
        ));

        let population_data = CitizenIdentity::governance_population_data(&town_scope())
            .expect("current population should be ready");
        assert_eq!(population_data.eligible_total, 1);
        assert_eq!(population_data.eligibility_revision, 1);
        assert_eq!(population_data.eligibility_date, 20260702);
        let voter_subject =
            CitizenIdentity::voting_subject_at_population_data(&1, &population_data)
                .expect("snapshot eligibility should return the complete citizen subject");
        assert_eq!(voter_subject.cid_number, citizen_cid_number("0001"));
        assert_eq!(voter_subject.account_id, 1);
    });
}

#[test]
fn population_data_revision_freezes_membership_before_identity_update() {
    new_test_ext().execute_with(|| {
        occupy_tag("SNAPSHOT-OLD");
        assert_ok!(CitizenIdentity::register_voting_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            registrar_role_code(),
            voting_payload(1, &citizen_cid_number("SNAPSHOT-OLD")),
            valid_signature(),
        ));

        let old_population_data = CitizenIdentity::governance_population_data(&town_scope())
            .expect("old population should be ready");
        assert_eq!(old_population_data.eligible_total, 1);
        assert!(
            CitizenIdentity::voting_subject_at_population_data(&1, &old_population_data).is_some()
        );

        // 同一账户迁往另一乡镇后，旧提案仍按创建时身份判断；新提案使用新身份。
        let mut moved = voting_payload(1, &citizen_cid_number("SNAPSHOT-OLD"));
        moved.residence_town_code = code(b"4301002");
        assert_ok!(CitizenIdentity::update_voting_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            registrar_role_code(),
            moved,
            valid_signature(),
        ));

        assert!(
            CitizenIdentity::voting_subject_at_population_data(&1, &old_population_data).is_some()
        );
        let new_population_data = CitizenIdentity::governance_population_data(&town_scope())
            .expect("new population should be ready");
        assert_eq!(new_population_data.eligible_total, 0);
        assert!(
            CitizenIdentity::voting_subject_at_population_data(&1, &new_population_data).is_none()
        );
    });
}

#[test]
fn invalid_citizen_code_is_rejected() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            CitizenIdentity::register_voting_identity(
                RuntimeOrigin::signed(100),
                registrar_cid_number(),
                registrar_role_code(),
                voting_payload(1, b"OLD-0001"),
                valid_signature(),
            ),
            Error::<Test>::InvalidCitizenCode
        );
    });
}

#[test]
fn expired_passport_cannot_vote_and_is_excluded_from_population() {
    new_test_ext().execute_with(|| {
        // 占号先行:身份写入前置。
        occupy_tag("EXPIRED");

        let mut payload = voting_payload(1, &citizen_cid_number("EXPIRED"));
        payload.passport_valid_from = 20200101;
        payload.passport_valid_until = 20250101;

        assert_ok!(CitizenIdentity::register_voting_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            registrar_role_code(),
            payload,
            valid_signature(),
        ));

        assert_eq!(CountryVotingCount::<Test>::get(), 0);
        assert!(CitizenIdentity::voting_subject(&1, &town_scope()).is_none());
        assert!(CitizenIdentity::candidate_subject(&1, &town_scope()).is_none());
    });
}

#[test]
fn not_yet_valid_passport_cannot_vote() {
    new_test_ext().execute_with(|| {
        // 占号先行:身份写入前置。
        occupy_tag("FUTURE");

        let mut payload = voting_payload(1, &citizen_cid_number("FUTURE"));
        payload.passport_valid_from = 20300101;
        payload.passport_valid_until = 20400101;

        assert_ok!(CitizenIdentity::register_voting_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            registrar_role_code(),
            payload,
            valid_signature(),
        ));

        assert_eq!(CountryVotingCount::<Test>::get(), 0);
        assert_eq!(PopulationTransitionCountByDate::<Test>::get(20300101), 1);
        assert!(CitizenIdentity::voting_subject(&1, &town_scope()).is_none());
    });
}

#[test]
fn first_population_date_must_initialize_before_identity_write() {
    new_test_ext().execute_with(|| {
        PopulationReadyDate::<Test>::kill();
        occupy_tag("BOOTSTRAP");
        assert_noop!(
            CitizenIdentity::register_voting_identity(
                RuntimeOrigin::signed(100),
                registrar_cid_number(),
                registrar_role_code(),
                voting_payload(1, &citizen_cid_number("BOOTSTRAP")),
                valid_signature(),
            ),
            Error::<Test>::PopulationDataNotReady
        );

        CitizenIdentity::on_idle(System::block_number(), frame_support::weights::Weight::MAX);
        assert_eq!(PopulationReadyDate::<Test>::get(), 20260702);
        assert_ok!(CitizenIdentity::register_voting_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            registrar_role_code(),
            voting_payload(1, &citizen_cid_number("BOOTSTRAP")),
            valid_signature(),
        ));
    });
}

#[test]
fn passport_activates_on_valid_from_and_deactivates_after_valid_until() {
    new_test_ext().execute_with(|| {
        occupy_tag("ONE-DAY");
        let mut payload = voting_payload(1, &citizen_cid_number("ONE-DAY"));
        payload.passport_valid_from = 20260703;
        payload.passport_valid_until = 20260703;
        assert_ok!(CitizenIdentity::register_voting_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            registrar_role_code(),
            payload,
            valid_signature(),
        ));
        assert_eq!(CountryVotingCount::<Test>::get(), 0);

        set_day_offset(1);
        assert!(CitizenIdentity::governance_population_data(&PopulationScope::Country).is_none());
        CitizenIdentity::on_idle(System::block_number(), frame_support::weights::Weight::MAX);
        assert_eq!(PopulationReadyDate::<Test>::get(), 20260703);
        assert_eq!(CountryVotingCount::<Test>::get(), 1);
        assert_eq!(
            CitizenIdentity::governance_population_data(&PopulationScope::Country)
                .expect("activation date should be ready")
                .eligibility_date,
            20260703
        );

        set_day_offset(2);
        CitizenIdentity::on_idle(System::block_number(), frame_support::weights::Weight::MAX);
        assert_eq!(PopulationReadyDate::<Test>::get(), 20260704);
        assert_eq!(CountryVotingCount::<Test>::get(), 0);
    });
}

#[test]
fn population_transition_limit_hides_partial_day_and_blocks_identity_changes() {
    new_test_ext().execute_with(|| {
        for (account_id, tag) in [(1, "BATCH-1"), (2, "BATCH-2"), (3, "BATCH-3")] {
            occupy_tag(tag);
            let mut payload = voting_payload(account_id, &citizen_cid_number(tag));
            payload.passport_valid_from = 20260703;
            payload.passport_valid_until = 20300101;
            assert_ok!(CitizenIdentity::register_voting_identity(
                RuntimeOrigin::signed(100),
                registrar_cid_number(),
                registrar_role_code(),
                payload,
                valid_signature(),
            ));
        }

        set_day_offset(1);
        CitizenIdentity::on_idle(System::block_number(), frame_support::weights::Weight::MAX);
        assert_eq!(PopulationReadyDate::<Test>::get(), 20260702);
        assert_eq!(PopulationTransitionCursorByDate::<Test>::get(20260703), 2);
        assert_eq!(CountryVotingCount::<Test>::get(), 2);
        assert!(CitizenIdentity::governance_population_data(&PopulationScope::Country).is_none());

        let mut update = voting_payload(1, &citizen_cid_number("BATCH-1"));
        update.passport_valid_from = 20260703;
        update.passport_valid_until = 20310101;
        assert_noop!(
            CitizenIdentity::update_voting_identity(
                RuntimeOrigin::signed(100),
                registrar_cid_number(),
                registrar_role_code(),
                update,
                valid_signature(),
            ),
            Error::<Test>::PopulationDataNotReady
        );

        CitizenIdentity::on_idle(System::block_number(), frame_support::weights::Weight::MAX);
        assert_eq!(PopulationReadyDate::<Test>::get(), 20260703);
        assert_eq!(CountryVotingCount::<Test>::get(), 3);
        assert!(CitizenIdentity::governance_population_data(&PopulationScope::Country).is_some());
    });
}

#[test]
fn identity_update_invalidates_old_population_transitions_by_revision() {
    new_test_ext().execute_with(|| {
        occupy_tag("RESCHEDULE");
        let mut first = voting_payload(1, &citizen_cid_number("RESCHEDULE"));
        first.passport_valid_from = 20260703;
        first.passport_valid_until = 20260703;
        assert_ok!(CitizenIdentity::register_voting_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            registrar_role_code(),
            first,
            valid_signature(),
        ));

        let mut replacement = voting_payload(1, &citizen_cid_number("RESCHEDULE"));
        replacement.passport_valid_from = 20260704;
        replacement.passport_valid_until = 20260705;
        assert_ok!(CitizenIdentity::update_voting_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            registrar_role_code(),
            replacement,
            valid_signature(),
        ));

        set_day_offset(1);
        CitizenIdentity::on_idle(System::block_number(), frame_support::weights::Weight::MAX);
        assert_eq!(CountryVotingCount::<Test>::get(), 0);
        set_day_offset(2);
        CitizenIdentity::on_idle(System::block_number(), frame_support::weights::Weight::MAX);
        assert_eq!(PopulationReadyDate::<Test>::get(), 20260704);
        assert_eq!(CountryVotingCount::<Test>::get(), 1);
    });
}

#[test]
fn strict_calendar_validation_handles_months_leap_years_and_year_boundary() {
    new_test_ext().execute_with(|| {
        assert!(CitizenIdentity::is_plausible_yyyymmdd(20240229));
        assert!(!CitizenIdentity::is_plausible_yyyymmdd(20230229));
        assert!(!CitizenIdentity::is_plausible_yyyymmdd(20260431));
        assert_eq!(
            CitizenIdentity::next_calendar_date(20240229),
            Some(20240301)
        );
        assert_eq!(
            CitizenIdentity::next_calendar_date(20261231),
            Some(20270101)
        );
        assert_eq!(CitizenIdentity::next_calendar_date(99991231), None);
    });
}

#[test]
fn population_faults_closed_when_chain_date_moves_backwards() {
    new_test_ext().execute_with(|| {
        set_day_offset(-1);
        CitizenIdentity::on_idle(System::block_number(), frame_support::weights::Weight::MAX);
        assert_eq!(
            PopulationMaintenanceFault::<Test>::get(),
            Some(PopulationFault::DateMovedBackwards)
        );
        assert!(CitizenIdentity::governance_population_data(&PopulationScope::Country).is_none());
    });
}

#[test]
fn candidate_identity_stores_sex_and_public_profile() {
    new_test_ext().execute_with(|| {
        // 占号先行:身份写入前置。
        occupy_tag("SEX");

        assert_ok!(CitizenIdentity::upgrade_to_candidate_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            registrar_role_code(),
            candidate_payload(1, &citizen_cid_number("SEX")),
            valid_signature(),
        ));

        let stored = CandidateIdentityByCid::<Test>::get(cid(&citizen_cid_number("SEX")))
            .expect("candidate stored");
        assert_eq!(stored.citizen_sex, CitizenSex::Female);
        assert_eq!(stored.family_name, family_name(b"Citizen"));
        assert_eq!(stored.given_name, given_name(b"One"));
        assert_eq!(stored.birth_date, 20000131);
        // 固定链上日 20260702 − 出生 20000131 → 26 周岁。
        assert_eq!(CitizenIdentity::candidate_age(&1), Some(26));
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
fn age_from_birth_date_handles_birthday_boundary() {
    new_test_ext().execute_with(|| {
        // 固定链上日 20260702。
        assert_eq!(CitizenIdentity::age_from_birth_date(20000701), Some(26)); // 生日已过
        assert_eq!(CitizenIdentity::age_from_birth_date(20000702), Some(26)); // 今日生日
        assert_eq!(CitizenIdentity::age_from_birth_date(20000703), Some(25)); // 生日未到
        assert_eq!(CitizenIdentity::age_from_birth_date(0), None); // 空
        assert_eq!(CitizenIdentity::age_from_birth_date(20300101), None); // 未来出生
    });
}

#[test]
fn candidate_birth_date_is_immutable_on_update() {
    new_test_ext().execute_with(|| {
        occupy_tag("IMMUT");
        let cid = citizen_cid_number("IMMUT");
        assert_ok!(CitizenIdentity::upgrade_to_candidate_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            registrar_role_code(),
            candidate_payload(1, &cid),
            valid_signature(),
        ));

        // 更新竞选身份时试图改出生日期 → 拒绝。
        let mut tampered = candidate_payload(1, &cid);
        tampered.birth_date = 19990101;
        assert_noop!(
            CitizenIdentity::update_candidate_identity(
                RuntimeOrigin::signed(100),
                registrar_cid_number(),
                registrar_role_code(),
                tampered,
                valid_signature(),
            ),
            Error::<Test>::BirthDateImmutable
        );
    });
}

#[test]
fn candidate_illegal_birth_date_rejected() {
    new_test_ext().execute_with(|| {
        occupy_tag("BADDOB");
        let mut payload = candidate_payload(1, &citizen_cid_number("BADDOB"));
        payload.birth_date = 20261340; // 非法月/日
        assert_noop!(
            CitizenIdentity::upgrade_to_candidate_identity(
                RuntimeOrigin::signed(100),
                registrar_cid_number(),
                registrar_role_code(),
                payload,
                valid_signature(),
            ),
            Error::<Test>::InvalidBirthDate
        );
    });
}

#[test]
fn candidate_future_birth_date_rejected() {
    new_test_ext().execute_with(|| {
        occupy_tag("FUTDOB");
        let mut payload = candidate_payload(1, &citizen_cid_number("FUTDOB"));
        payload.birth_date = 20990101; // 未来出生 → 算不出年龄
        assert_noop!(
            CitizenIdentity::upgrade_to_candidate_identity(
                RuntimeOrigin::signed(100),
                registrar_cid_number(),
                registrar_role_code(),
                payload,
                valid_signature(),
            ),
            Error::<Test>::InvalidBirthDate
        );
    });
}

#[test]
fn under_sixteen_cannot_register_onchain_identity() {
    new_test_ext().execute_with(|| {
        // 占号先行:身份写入前置。
        occupy_tag("UNDERAGE");

        let mut payload = voting_payload(1, &citizen_cid_number("UNDERAGE"));
        payload.citizen_age_years = 15;

        assert_noop!(
            CitizenIdentity::register_voting_identity(
                RuntimeOrigin::signed(100),
                registrar_cid_number(),
                registrar_role_code(),
                payload,
                valid_signature(),
            ),
            Error::<Test>::UnderVotingAge
        );
    });
}

#[test]
fn non_citizen_family_code_is_rejected() {
    new_test_ext().execute_with(|| {
        // 真实格式的公权机构号(CGOV)打到公民入口必须被家族断言拒绝。
        let institution_number = primitives::cid::generator::generate_cid_number(
            primitives::cid::generator::GenerateCidNumberInput {
                public_key: "gov",
                p1: "0",
                province_code: "GD",
                province_name: "广东省",
                city_code: "001",
                city_name: "荔湾市",
                year: "2026",
                institution: "CGOV",
            },
        )
        .expect("institution cid should generate")
        .into_bytes();

        assert_noop!(
            CitizenIdentity::register_voting_identity(
                RuntimeOrigin::signed(100),
                registrar_cid_number(),
                registrar_role_code(),
                voting_payload(1, &institution_number),
                valid_signature(),
            ),
            Error::<Test>::InvalidCitizenCode
        );
    });
}

#[test]
fn occupy_cid_is_idempotent_for_same_registrar_and_commitment() {
    new_test_ext().execute_with(|| {
        occupy_tag("OCC-1");
        // 同注册局+同承诺重复提交:幂等放行(落库失败恢复路径)。
        occupy_tag("OCC-1");
        // 承诺不同:视为撞号,拒绝。
        assert_noop!(
            CitizenIdentity::occupy_cid(
                RuntimeOrigin::signed(100),
                registrar_cid_number(),
                registrar_role_code(),
                cid(&citizen_cid_number("OCC-1")),
                commitment_for("OTHER"),
                code(b"43"),
                code(b"4301"),
            ),
            Error::<Test>::CidAlreadyOccupied
        );
    });
}

#[test]
fn occupy_cid_rejects_unauthorized_registrar_and_bad_number() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            CitizenIdentity::occupy_cid(
                RuntimeOrigin::signed(999),
                registrar_cid_number(),
                registrar_role_code(),
                cid(&citizen_cid_number("OCC-2")),
                commitment_for("OCC-2"),
                code(b"43"),
                code(b"4301"),
            ),
            Error::<Test>::UnauthorizedRegistrar
        );
        // 公权机构号打公民占号入口:家族断言拒绝。
        assert_noop!(
            CitizenIdentity::occupy_cid(
                RuntimeOrigin::signed(100),
                registrar_cid_number(),
                registrar_role_code(),
                cid(&public_cid_number("OCC-2")),
                commitment_for("OCC-2"),
                code(b"43"),
                code(b"4301"),
            ),
            Error::<Test>::InvalidCitizenCode
        );
    });
}

#[test]
fn occupy_cids_batch_rolls_back_entirely_on_any_conflict() {
    new_test_ext().execute_with(|| {
        occupy_tag("B-TAKEN");
        let items: CidOccupyItemsBound = alloc::vec![
            CidOccupyItem {
                cid_number: cid(&citizen_cid_number("B-1")),
                commitment: commitment_for("B-1"),
            },
            CidOccupyItem {
                cid_number: cid(&citizen_cid_number("B-TAKEN")),
                commitment: commitment_for("CONFLICT"),
            },
        ]
        .try_into()
        .expect("batch fits");
        assert_noop!(
            CitizenIdentity::occupy_cids_batch(
                RuntimeOrigin::signed(100),
                registrar_cid_number(),
                registrar_role_code(),
                items,
                code(b"43"),
                code(b"4301"),
            ),
            Error::<Test>::CidAlreadyOccupied
        );
        // 整笔回滚:B-1 未被占。
        assert!(CidRegistry::<Test>::get(cid(&citizen_cid_number("B-1"))).is_none());

        // 全部合法则整批占号成功。
        let ok_items: CidOccupyItemsBound = alloc::vec![
            CidOccupyItem {
                cid_number: cid(&citizen_cid_number("B-2")),
                commitment: commitment_for("B-2"),
            },
            CidOccupyItem {
                cid_number: cid(&citizen_cid_number("B-3")),
                commitment: commitment_for("B-3"),
            },
        ]
        .try_into()
        .expect("batch fits");
        assert_ok!(CitizenIdentity::occupy_cids_batch(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            registrar_role_code(),
            ok_items,
            code(b"43"),
            code(b"4301"),
        ));
        assert!(CidRegistry::<Test>::get(cid(&citizen_cid_number("B-2"))).is_some());
        assert!(CidRegistry::<Test>::get(cid(&citizen_cid_number("B-3"))).is_some());
    });
}

#[test]
fn register_without_occupation_is_rejected() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            CitizenIdentity::register_voting_identity(
                RuntimeOrigin::signed(100),
                registrar_cid_number(),
                registrar_role_code(),
                voting_payload(1, &citizen_cid_number("NO-OCC")),
                valid_signature(),
            ),
            Error::<Test>::CidNotOccupied
        );
    });
}

#[test]
fn revoke_cid_tombstones_and_revokes_bound_identity() {
    new_test_ext().execute_with(|| {
        occupy_tag("RV-1");
        assert_ok!(CitizenIdentity::register_voting_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            registrar_role_code(),
            voting_payload(1, &citizen_cid_number("RV-1")),
            valid_signature(),
        ));
        assert_eq!(CountryVotingCount::<Test>::get(), 1);

        assert_ok!(CitizenIdentity::revoke_cid(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            registrar_role_code(),
            cid(&citizen_cid_number("RV-1")),
        ));
        // 登记表墓碑 + 身份联动吊销 + 退出人口分母。
        let rec = CidRegistry::<Test>::get(cid(&citizen_cid_number("RV-1"))).expect("record kept");
        assert_eq!(rec.status, CidRecordStatus::Revoked);
        assert_eq!(
            VotingIdentityByCid::<Test>::get(cid(&citizen_cid_number("RV-1")))
                .expect("identity kept")
                .citizen_status,
            CitizenStatus::Revoked
        );
        assert_eq!(CountryVotingCount::<Test>::get(), 0);

        // 再吊销:已墓碑。
        assert_noop!(
            CitizenIdentity::revoke_cid(
                RuntimeOrigin::signed(100),
                registrar_cid_number(),
                registrar_role_code(),
                cid(&citizen_cid_number("RV-1")),
            ),
            Error::<Test>::CidAlreadyRevoked
        );
        // 墓碑号任何人不可再占(号码永不复用)。
        assert_noop!(
            CitizenIdentity::occupy_cid(
                RuntimeOrigin::signed(100),
                registrar_cid_number(),
                registrar_role_code(),
                cid(&citizen_cid_number("RV-1")),
                commitment_for("RV-1"),
                code(b"43"),
                code(b"4301"),
            ),
            Error::<Test>::CidAlreadyOccupied
        );
        // 墓碑号也不能再注册身份：永久 CID 身份与账户绑定均保留，
        // 归属检查先于墓碑检查拦截(双保险,谁先触发都拒绝)。
        assert_noop!(
            CitizenIdentity::register_voting_identity(
                RuntimeOrigin::signed(100),
                registrar_cid_number(),
                registrar_role_code(),
                voting_payload(2, &citizen_cid_number("RV-1")),
                valid_signature(),
            ),
            Error::<Test>::CidAlreadyRevoked
        );
    });
}

#[test]
fn permanent_cid_update_keeps_registry_record_active() {
    new_test_ext().execute_with(|| {
        occupy_tag("CHG-A");
        occupy_tag("CHG-B");
        assert_ok!(CitizenIdentity::register_voting_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            registrar_role_code(),
            voting_payload(1, &citizen_cid_number("CHG-A")),
            valid_signature(),
        ));
        let mut updated = voting_payload(1, &citizen_cid_number("CHG-A"));
        updated.residence_town_code = code(b"4301002");
        assert_ok!(CitizenIdentity::update_voting_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            registrar_role_code(),
            updated,
            valid_signature(),
        ));
        assert_eq!(
            CidRegistry::<Test>::get(cid(&citizen_cid_number("CHG-A")))
                .expect("permanent record kept")
                .status,
            CidRecordStatus::Active
        );
        assert_eq!(
            CidRegistry::<Test>::get(cid(&citizen_cid_number("CHG-B")))
                .expect("unrelated occupied record kept")
                .status,
            CidRecordStatus::Active
        );
    });
}
