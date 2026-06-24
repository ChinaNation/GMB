//! 立法院模块第1步单测(engine = `()`)。
//!
//! 覆盖:write_law_version 写入/升版/废止、on_initialize 生效调度、
//! 提案校验(非议员拒绝 / 空标题空条文 / 宪法表决类型 / 不可修改条款 / 宪法不可废)、
//! 合法提案到引擎返回 NotConfigured(确认对接点)。

use super::*;
use crate::pallet::Error;
use frame_support::{assert_noop, assert_ok, traits::Hooks};

// ───────────────── 执行器写入 / 状态机 ─────────────────

#[test]
fn enact_writes_law_and_schedules_future_activation() {
    new_test_ext().execute_with(|| {
        let mut summary = enact_summary(
            Tier::Municipal,
            1001,
            VoteType::Important,
            b"\xe5\xb8\x82\xe9\x95\xbf\xe9\x80\x89\xe4\xb8\xbe\xe6\xb3\x95",
        );
        summary.effective_at = 100; // 未来生效
        let arts = articles(vec![article(1, b"a"), article(2, b"b")]);

        assert_ok!(Lib::write_law_version(7, summary, arts, 1));

        let law = Laws::<Test>::get(0).expect("law created");
        assert_eq!(law.current_version, 1);
        assert_eq!(law.status, LawStatus::Pending); // 未到生效区块
        assert!(LawVersions::<Test>::get(0, 1).is_some());
        assert_eq!(NextLawId::<Test>::get(), 1);
        assert_eq!(Lib::list_laws(Tier::Municipal, 1001), vec![0]);
        assert_eq!(
            PendingActivation::<Test>::get(100).into_inner(),
            vec![(0u64, 1u32)]
        );

        // 到生效区块翻 Effective
        Lib::on_initialize(100);
        assert_eq!(Laws::<Test>::get(0).unwrap().status, LawStatus::Effective);
        assert!(PendingActivation::<Test>::get(100).is_empty());
    });
}

#[test]
fn enact_with_past_effective_is_immediately_effective() {
    new_test_ext().execute_with(|| {
        let summary = enact_summary(Tier::National, 0, VoteType::Regular, b"law");
        // effective_at = 0 <= now(1) → 立即生效
        assert_ok!(Lib::write_law_version(
            1,
            summary,
            articles(vec![article(1, b"a")]),
            1
        ));
        assert_eq!(Laws::<Test>::get(0).unwrap().status, LawStatus::Effective);
    });
}

#[test]
fn amend_bumps_version_and_resets_pending() {
    new_test_ext().execute_with(|| {
        let s0 = enact_summary(Tier::National, 0, VoteType::Regular, b"law");
        assert_ok!(Lib::write_law_version(
            1,
            s0,
            articles(vec![article(1, b"a")]),
            1
        ));

        let mut s1 = enact_summary(Tier::National, 0, VoteType::Regular, b"law-v2");
        s1.action = LawAction::Amend;
        s1.law_id = 0;
        s1.effective_at = 50;
        assert_ok!(Lib::write_law_version(
            2,
            s1,
            articles(vec![article(1, b"a2")]),
            1
        ));

        let law = Laws::<Test>::get(0).unwrap();
        assert_eq!(law.current_version, 2);
        assert_eq!(law.status, LawStatus::Pending);
        assert!(LawVersions::<Test>::get(0, 2).is_some());
        assert!(LawVersions::<Test>::get(0, 1).is_some()); // 历史保留
    });
}

#[test]
fn repeal_sets_status_repealed() {
    new_test_ext().execute_with(|| {
        let s0 = enact_summary(Tier::National, 0, VoteType::Regular, b"law");
        assert_ok!(Lib::write_law_version(
            1,
            s0,
            articles(vec![article(1, b"a")]),
            1
        ));

        let mut sr = enact_summary(Tier::National, 0, VoteType::Regular, b"");
        sr.action = LawAction::Repeal;
        sr.law_id = 0;
        assert_ok!(Lib::write_law_version(2, sr, articles(vec![]), 1));

        assert_eq!(Laws::<Test>::get(0).unwrap().status, LawStatus::Repealed);
    });
}

// ───────────────── 提案入口校验 ─────────────────

fn one_article() -> frame_support::BoundedVec<crate::pallet::Article<Test>, super::MaxArticlesPerLaw>
{
    articles(vec![article(1, b"content")])
}

#[test]
fn propose_enact_by_legislator_reaches_engine_notconfigured() {
    new_test_ext().execute_with(|| {
        // 合法发起人 + 合法输入 → 通过全部校验 → 调引擎 () → NotConfigured → VoteEngineCreateFailed
        assert_noop!(
            Lib::propose_enact_law(
                RuntimeOrigin::signed(legislator()),
                Tier::Municipal,
                1001,
                houses(),
                VoteType::Regular,
                title(b"law"),
                None,
                one_article(),
                100,
            ),
            Error::<Test>::VoteEngineCreateFailed
        );
    });
}

#[test]
fn propose_enact_by_non_legislator_rejected() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Lib::propose_enact_law(
                RuntimeOrigin::signed(outsider()),
                Tier::Municipal,
                1001,
                houses(),
                VoteType::Regular,
                title(b"law"),
                None,
                one_article(),
                100,
            ),
            Error::<Test>::NotLegislator
        );
    });
}

#[test]
fn propose_enact_empty_title_and_articles_rejected() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Lib::propose_enact_law(
                RuntimeOrigin::signed(legislator()),
                Tier::Municipal,
                1001,
                houses(),
                VoteType::Regular,
                title(b""),
                None,
                one_article(),
                100,
            ),
            Error::<Test>::EmptyTitle
        );
        assert_noop!(
            Lib::propose_enact_law(
                RuntimeOrigin::signed(legislator()),
                Tier::Municipal,
                1001,
                houses(),
                VoteType::Regular,
                title(b"law"),
                None,
                articles(vec![]),
                100,
            ),
            Error::<Test>::EmptyArticles
        );
    });
}

// 预置一部宪法(article 1 与 17 固定),供修法/废法校验测试。
fn seed_constitution() {
    let summary = enact_summary(Tier::Constitution, 0, VoteType::Special, b"constitution");
    assert_ok!(Lib::write_law_version(
        1,
        summary,
        articles(vec![article(1, b"yuan-1"), article(17, b"yuan-17")]),
        1,
    ));
}

#[test]
fn amend_constitution_immutable_article_rejected() {
    new_test_ext().execute_with(|| {
        seed_constitution();
        // 改第 1 条(不可修改条款)→ 拒绝
        assert_noop!(
            Lib::propose_amend_law(
                RuntimeOrigin::signed(legislator()),
                0,
                VoteType::Special,
                title(b"constitution-v2"),
                None,
                articles(vec![article(1, b"CHANGED"), article(17, b"yuan-17")]),
                200,
            ),
            Error::<Test>::ImmutableArticleViolation
        );
    });
}

#[test]
fn amend_constitution_preserving_immutable_reaches_engine() {
    new_test_ext().execute_with(|| {
        seed_constitution();
        // 第 1、17 条逐字保留,新增第 99 条 → 通过不可修改校验 → 调引擎 () → NotConfigured
        assert_noop!(
            Lib::propose_amend_law(
                RuntimeOrigin::signed(legislator()),
                0,
                VoteType::Special,
                title(b"constitution-v2"),
                None,
                articles(vec![
                    article(1, b"yuan-1"),
                    article(17, b"yuan-17"),
                    article(99, b"new")
                ]),
                200,
            ),
            Error::<Test>::VoteEngineCreateFailed
        );
    });
}

#[test]
fn amend_constitution_with_regular_vote_type_rejected() {
    new_test_ext().execute_with(|| {
        seed_constitution();
        assert_noop!(
            Lib::propose_amend_law(
                RuntimeOrigin::signed(legislator()),
                0,
                VoteType::Regular, // 宪法修改不允许常规案
                title(b"constitution-v2"),
                None,
                articles(vec![article(1, b"yuan-1"), article(17, b"yuan-17")]),
                200,
            ),
            Error::<Test>::InvalidVoteTypeForConstitution
        );
    });
}

#[test]
fn repeal_constitution_rejected() {
    new_test_ext().execute_with(|| {
        seed_constitution();
        assert_noop!(
            Lib::propose_repeal_law(RuntimeOrigin::signed(legislator()), 0, VoteType::Special),
            Error::<Test>::CannotRepealConstitution
        );
    });
}

#[test]
fn amend_nonexistent_law_rejected() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Lib::propose_amend_law(
                RuntimeOrigin::signed(legislator()),
                404,
                VoteType::Regular,
                title(b"x"),
                None,
                one_article(),
                100,
            ),
            Error::<Test>::LawNotFound
        );
    });
}
