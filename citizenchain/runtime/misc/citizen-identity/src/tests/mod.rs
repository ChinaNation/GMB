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
        actor_cid_number: &[u8],
        residence_province_code: &[u8],
        residence_city_code: &[u8],
        _level: CitizenIdentityLevel,
    ) -> bool {
        *registrar == 100
            && actor_cid_number == registrar_cid_number().as_slice()
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
            account_pubkey: tag,
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

fn name(bytes: &[u8]) -> CitizenNameBound {
    bytes.to_vec().try_into().expect("citizen name should fit")
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
            account_pubkey: tag,
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
            voting_payload(1, &citizen_cid_number("0001")),
            valid_signature(),
        ));

        assert!(VotingIdentityByAccount::<Test>::contains_key(1));
        assert_eq!(
            AccountByCid::<Test>::get(cid(&citizen_cid_number("0001"))),
            Some(1)
        );
        assert_eq!(CountryVotingCount::<Test>::get(), 1);
        assert_eq!(ProvinceVotingCount::<Test>::get(code(b"43")), 1);
        assert!(CitizenIdentity::can_vote(&1, &town_scope()));
    });
}

#[test]
fn duplicate_cid_cannot_move_to_another_wallet_account() {
    new_test_ext().execute_with(|| {
        // 占号先行:身份写入前置。
        occupy_tag("0001");

        assert_ok!(CitizenIdentity::register_voting_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            voting_payload(1, &citizen_cid_number("0001")),
            valid_signature(),
        ));

        assert_noop!(
            CitizenIdentity::register_voting_identity(
                RuntimeOrigin::signed(100),
                registrar_cid_number(),
                voting_payload(2, &citizen_cid_number("0001")),
                valid_signature(),
            ),
            Error::<Test>::CidAlreadyRegisteredToAnotherAccount
        );
    });
}

#[test]
fn updating_same_account_replaces_cid_without_double_counting() {
    new_test_ext().execute_with(|| {
        // 占号先行:身份写入前置。
        occupy_tag("0001");
        occupy_tag("0002");

        assert_ok!(CitizenIdentity::register_voting_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            voting_payload(1, &citizen_cid_number("0001")),
            valid_signature(),
        ));
        assert_ok!(CitizenIdentity::update_voting_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            voting_payload(1, &citizen_cid_number("0002")),
            valid_signature(),
        ));

        assert_eq!(
            AccountByCid::<Test>::get(cid(&citizen_cid_number("0001"))),
            None
        );
        assert_eq!(
            AccountByCid::<Test>::get(cid(&citizen_cid_number("0002"))),
            Some(1)
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
            candidate_payload(1, &citizen_cid_number("CANDIDATE")),
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
        // 占号先行:身份写入前置。
        occupy_tag("REVOKE");

        assert_ok!(CitizenIdentity::upgrade_to_candidate_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            candidate_payload(1, &citizen_cid_number("REVOKE")),
            valid_signature(),
        ));
        assert_ok!(CitizenIdentity::revoke_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            cid(&citizen_cid_number("REVOKE")),
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
        // 占号先行:身份写入前置。
        occupy_tag("0001");

        assert_ok!(CitizenIdentity::register_voting_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            voting_payload(1, &citizen_cid_number("0001")),
            valid_signature(),
        ));

        CitizenIdentity::create_governance_population_snapshot(&town_scope())
            .expect("votingengine provider can create snapshot");

        let snapshot = PopulationSnapshots::<Test>::get(0).expect("snapshot should exist");
        assert_eq!(snapshot.eligible_total, 1);
        assert_eq!(snapshot.eligibility_revision, 1);
        assert_eq!(snapshot.snapshot_date, 20260702);
        assert!(CitizenIdentity::can_vote_at_snapshot(&1, 0));
        assert_eq!(NextSnapshotId::<Test>::get(), 1);
    });
}

#[test]
fn population_snapshot_freezes_membership_before_identity_update() {
    new_test_ext().execute_with(|| {
        occupy_tag("SNAPSHOT-OLD");
        occupy_tag("SNAPSHOT-NEW");
        assert_ok!(CitizenIdentity::register_voting_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            voting_payload(1, &citizen_cid_number("SNAPSHOT-OLD")),
            valid_signature(),
        ));

        let (old_snapshot_id, old_total) =
            CitizenIdentity::create_governance_population_snapshot(&town_scope())
                .expect("old snapshot should be created");
        assert_eq!(old_total, 1);
        assert!(CitizenIdentity::can_vote_at_snapshot(&1, old_snapshot_id));

        // 同一账户迁往另一乡镇后，旧提案仍按创建时身份判断；新提案使用新身份。
        let mut moved = voting_payload(1, &citizen_cid_number("SNAPSHOT-NEW"));
        moved.residence_town_code = code(b"4301002");
        assert_ok!(CitizenIdentity::update_voting_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            moved,
            valid_signature(),
        ));

        assert!(CitizenIdentity::can_vote_at_snapshot(&1, old_snapshot_id));
        let (new_snapshot_id, new_total) =
            CitizenIdentity::create_governance_population_snapshot(&town_scope())
                .expect("new snapshot should be created");
        assert_eq!(new_total, 0);
        assert!(!CitizenIdentity::can_vote_at_snapshot(&1, new_snapshot_id));

        CitizenIdentity::release_governance_population_snapshot(old_snapshot_id);
        assert!(!PopulationSnapshots::<Test>::contains_key(old_snapshot_id));
        assert!(!CitizenIdentity::can_vote_at_snapshot(&1, old_snapshot_id));
    });
}

#[test]
fn invalid_citizen_code_is_rejected() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            CitizenIdentity::register_voting_identity(
                RuntimeOrigin::signed(100),
                registrar_cid_number(),
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
        // 占号先行:身份写入前置。
        occupy_tag("EXPIRED");

        let mut payload = voting_payload(1, &citizen_cid_number("EXPIRED"));
        payload.passport_valid_from = 20200101;
        payload.passport_valid_until = 20250101;

        assert_ok!(CitizenIdentity::register_voting_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
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
        // 占号先行:身份写入前置。
        occupy_tag("FUTURE");

        let mut payload = voting_payload(1, &citizen_cid_number("FUTURE"));
        payload.passport_valid_from = 20300101;
        payload.passport_valid_until = 20400101;

        assert_ok!(CitizenIdentity::register_voting_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            payload,
            valid_signature(),
        ));

        assert!(!CitizenIdentity::can_vote(&1, &town_scope()));
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
            candidate_payload(1, &citizen_cid_number("SEX")),
            valid_signature(),
        ));

        let stored = CandidateIdentityByAccount::<Test>::get(1).expect("candidate stored");
        assert_eq!(stored.citizen_sex, CitizenSex::Female);
        assert_eq!(stored.citizen_full_name, name(b"Citizen One"));
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
                account_pubkey: "gov",
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
            voting_payload(1, &citizen_cid_number("RV-1")),
            valid_signature(),
        ));
        assert_eq!(CountryVotingCount::<Test>::get(), 1);

        assert_ok!(CitizenIdentity::revoke_cid(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            cid(&citizen_cid_number("RV-1")),
        ));
        // 登记表墓碑 + 身份联动吊销 + 退出人口分母。
        let rec = CidRegistry::<Test>::get(cid(&citizen_cid_number("RV-1"))).expect("record kept");
        assert_eq!(rec.status, CidRecordStatus::Revoked);
        assert_eq!(
            VotingIdentityByAccount::<Test>::get(1)
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
                cid(&citizen_cid_number("RV-1")),
            ),
            Error::<Test>::CidAlreadyRevoked
        );
        // 墓碑号任何人不可再占(号码永不复用)。
        assert_noop!(
            CitizenIdentity::occupy_cid(
                RuntimeOrigin::signed(100),
                registrar_cid_number(),
                cid(&citizen_cid_number("RV-1")),
                commitment_for("RV-1"),
                code(b"43"),
                code(b"4301"),
            ),
            Error::<Test>::CidAlreadyOccupied
        );
        // 墓碑号也不能再注册身份:AccountByCid 映射保留,
        // 归属检查先于墓碑检查拦截(双保险,谁先触发都拒绝)。
        assert_noop!(
            CitizenIdentity::register_voting_identity(
                RuntimeOrigin::signed(100),
                registrar_cid_number(),
                voting_payload(2, &citizen_cid_number("RV-1")),
                valid_signature(),
            ),
            Error::<Test>::CidAlreadyRegisteredToAnotherAccount
        );
    });
}

#[test]
fn changing_cid_tombstones_old_registry_record() {
    new_test_ext().execute_with(|| {
        occupy_tag("CHG-A");
        occupy_tag("CHG-B");
        assert_ok!(CitizenIdentity::register_voting_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            voting_payload(1, &citizen_cid_number("CHG-A")),
            valid_signature(),
        ));
        assert_ok!(CitizenIdentity::update_voting_identity(
            RuntimeOrigin::signed(100),
            registrar_cid_number(),
            voting_payload(1, &citizen_cid_number("CHG-B")),
            valid_signature(),
        ));
        // 换号 = 旧号登记表墓碑,永不复用。
        assert_eq!(
            CidRegistry::<Test>::get(cid(&citizen_cid_number("CHG-A")))
                .expect("old record kept")
                .status,
            CidRecordStatus::Revoked
        );
        assert_eq!(
            CidRegistry::<Test>::get(cid(&citizen_cid_number("CHG-B")))
                .expect("new record kept")
                .status,
            CidRecordStatus::Active
        );
    });
}
