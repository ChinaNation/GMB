//! 立法投票单测:阈值纯函数 + 单院/两院/特别案投票机制。

use super::*;
use frame_support::{assert_noop, assert_ok, BoundedVec};
use votingengine::types::{
    legislation_house_decided, legislation_house_final_passed, legislation_referendum_final_passed,
    LEG_VOTE_IMPORTANT, LEG_VOTE_REGULAR, LEG_VOTE_SECOND_READING, LEG_VOTE_SPECIAL,
};
use votingengine::{
    STAGE_LEG_HOUSE, STAGE_LEG_REFERENDUM, STATUS_EXECUTED, STATUS_REJECTED, STATUS_VOTING,
};

// ───────────────── 阈值纯函数(宪法第十八条精确端点)─────────────────

#[test]
fn house_final_passed_thresholds() {
    // 常规案:>80% 参与 且 ≥60% 赞成(参与者基数)。total=10。
    assert!(legislation_house_final_passed(LEG_VOTE_REGULAR, 10, 6, 3)); // casted=9>8, 6/9≥60%
    assert!(!legislation_house_final_passed(LEG_VOTE_REGULAR, 10, 5, 3)); // 5/8=62.5%但 casted=8 不>8
    assert!(!legislation_house_final_passed(LEG_VOTE_REGULAR, 10, 5, 4)); // 5/9=55%<60%
                                                                          // 重要案:>90% 参与 且 ≥70% 赞成。
    assert!(legislation_house_final_passed(LEG_VOTE_IMPORTANT, 10, 7, 3)); // casted=10>9, 7/10≥70%
    assert!(!legislation_house_final_passed(
        LEG_VOTE_IMPORTANT,
        10,
        8,
        1
    )); // casted=9 不>9
        // 二审:全员参与 且 ≥50% 赞成 且 反对<20%。
    assert!(legislation_house_final_passed(
        LEG_VOTE_SECOND_READING,
        10,
        9,
        1
    )); // 全员10,9赞成,1反对<2
    assert!(!legislation_house_final_passed(
        LEG_VOTE_SECOND_READING,
        10,
        8,
        1
    )); // casted=9≠10,未全员
    assert!(!legislation_house_final_passed(
        LEG_VOTE_SECOND_READING,
        10,
        8,
        2
    )); // 反对2不<20%
        // 特别案内部:全员 且 ≥70% 赞成。
    assert!(legislation_house_final_passed(LEG_VOTE_SPECIAL, 10, 7, 3)); // 全员10,7赞成
    assert!(!legislation_house_final_passed(LEG_VOTE_SPECIAL, 10, 6, 4)); // 6<70%
}

#[test]
fn house_decided_early() {
    // 全员已投 → 立即判定。
    assert_eq!(
        legislation_house_decided(LEG_VOTE_REGULAR, 10, 7, 3),
        Some(true)
    );
    // 反对超限 → 提前否决(常规案反对>40%)。
    assert_eq!(
        legislation_house_decided(LEG_VOTE_REGULAR, 10, 0, 5),
        Some(false)
    ); // 5>4
       // 未全员、未超限 → 未决。
    assert_eq!(legislation_house_decided(LEG_VOTE_REGULAR, 10, 3, 2), None);
}

#[test]
fn referendum_threshold() {
    // ≥70% 参与 且 ≥70% 赞成。eligible=100。
    assert!(legislation_referendum_final_passed(100, 56, 14)); // 参与70,赞成56/70=80%
    assert!(!legislation_referendum_final_passed(100, 50, 19)); // 参与69<70
    assert!(!legislation_referendum_final_passed(100, 48, 22)); // 参与70,赞成48/70≈68%<70%
}

// ───────────────── 单院投票 ─────────────────

#[test]
fn single_house_regular_passes_on_full_participation() {
    new_test_ext().execute_with(|| {
        let pid = create(member(1), single_house(), LEG_VOTE_REGULAR);
        // 10 名议员全投:7 赞成 3 反对 → 通过。
        for i in 1u8..=7 {
            assert_ok!(cast(member(i), pid, true));
        }
        for i in 8u8..=10 {
            assert_ok!(cast(member(i), pid, false));
        }
        assert_eq!(status(pid), STATUS_EXECUTED);
    });
}

#[test]
fn single_house_regular_rejected_when_opposition_exceeds_cap() {
    new_test_ext().execute_with(|| {
        let pid = create(member(1), single_house(), LEG_VOTE_REGULAR);
        // 反对达 5 票(>40% of 10)→ 提前否决。
        for i in 1u8..=5 {
            assert_ok!(cast(member(i), pid, false));
        }
        assert_eq!(status(pid), STATUS_REJECTED);
    });
}

#[test]
fn double_vote_rejected() {
    new_test_ext().execute_with(|| {
        let pid = create(member(1), single_house(), LEG_VOTE_REGULAR);
        assert_ok!(cast(member(1), pid, true));
        assert_noop!(
            Lib::do_cast_house_vote(member(1), pid, true),
            votingengine::Error::<Test>::AlreadyVoted
        );
    });
}

#[test]
fn non_member_cannot_vote() {
    new_test_ext().execute_with(|| {
        let pid = create(member(1), single_house(), LEG_VOTE_REGULAR);
        // member(15) 属 house2,不在 house1 快照内。
        assert_noop!(
            Lib::do_cast_house_vote(member(15), pid, true),
            votingengine::Error::<Test>::NoPermission
        );
    });
}

#[test]
fn non_legislator_cannot_create() {
    new_test_ext().execute_with(|| {
        // member(50) 不是任何院议员。
        assert_noop!(
            Lib::do_create_legislation_proposal(member(50), single_house(), LEG_VOTE_REGULAR),
            crate::pallet::Error::<Test>::NotLegislator
        );
    });
}

// ───────────────── 两院顺序 ─────────────────

#[test]
fn two_houses_advance_then_pass() {
    new_test_ext().execute_with(|| {
        let pid = create(member(1), two_houses(), LEG_VOTE_IMPORTANT);
        // 第一院(众议会式,议员 1..=10):8 赞成 2 反对 → 通过,推进至第二院。
        for i in 1u8..=8 {
            assert_ok!(cast(member(i), pid, true));
        }
        for i in 9u8..=10 {
            assert_ok!(cast(member(i), pid, false));
        }
        // 仍在内部表决阶段,当前院推进到 1。
        assert_eq!(stage(pid), STAGE_LEG_HOUSE);
        assert_eq!(LegMeta::<Test>::get(pid).unwrap().current_house, 1);
        // 第二院(参议会式,议员 11..=20):8 赞成 2 反对 → 通过 → 整体 PASSED(重要案无公投)。
        for i in 11u8..=18 {
            assert_ok!(cast(member(i), pid, true));
        }
        for i in 19u8..=20 {
            assert_ok!(cast(member(i), pid, false));
        }
        assert_eq!(status(pid), STATUS_EXECUTED);
    });
}

// ───────────────── 特别案 → 强制公投 ─────────────────

fn prepare_snapshot(who: AccountId32, eligible_total: u64) {
    let nonce: votingengine::pallet::VoteNonceOf<Test> =
        BoundedVec::try_from(vec![1u8, 2, 3, 4]).unwrap();
    let sig: votingengine::pallet::VoteSignatureOf<Test> =
        BoundedVec::try_from(vec![9u8, 9, 9, 9]).unwrap();
    assert_ok!(Lib::do_prepare_population_snapshot(
        who,
        eligible_total,
        nonce,
        sig,
        b"issuer-cid",
        &house1(),
        &[7u8; 32],
        b"province",
        b"city",
    ));
}

#[test]
fn special_case_advances_to_referendum_then_passes() {
    new_test_ext().execute_with(|| {
        // 特别案:发起前准备人口快照(同一区块),分母 100。
        prepare_snapshot(member(1), 100);
        let pid = create(member(1), single_house(), LEG_VOTE_SPECIAL);
        // 全员 10:8 赞成 2 反对 → 内部段通过(≥70%)→ 推进至公投阶段。
        for i in 1u8..=8 {
            assert_ok!(cast(member(i), pid, true));
        }
        for i in 9u8..=10 {
            assert_ok!(cast(member(i), pid, false));
        }
        assert_eq!(stage(pid), STAGE_LEG_REFERENDUM);
        assert_eq!(status(pid), STATUS_VOTING); // 公投尚未结算

        // 公投:参与 70(56 赞成 14 反对)→ 达 ≥70% 参与 + ≥70% 赞成。
        for i in 0u64..56 {
            cast_referendum(pid, i, true);
        }
        for i in 56u64..70 {
            cast_referendum(pid, i, false);
        }
        // 期满结算(本入口不提前判定)。
        let p = votingengine::pallet::Proposals::<Test>::get(pid).unwrap();
        System::set_block_number(p.end + 1);
        finalize_referendum(pid);
        assert_eq!(status(pid), STATUS_EXECUTED);
    });
}

/// 公投投一票(用 binding_id 去重,seed 派生唯一 hash)。
fn cast_referendum(pid: u64, seed: u64, approve: bool) {
    use sp_runtime::traits::Hash as HashT;
    let binding = <Test as frame_system::Config>::Hashing::hash(&seed.to_le_bytes());
    let nonce: votingengine::pallet::VoteNonceOf<Test> = BoundedVec::try_from(vec![1u8]).unwrap();
    let sig: votingengine::pallet::VoteSignatureOf<Test> = BoundedVec::try_from(vec![1u8]).unwrap();
    frame_support::storage::with_transaction(
        || -> frame_support::storage::TransactionOutcome<sp_runtime::DispatchResult> {
            let r = Lib::do_cast_referendum_vote(
                member((seed % 200) as u8),
                pid,
                binding,
                nonce,
                sig,
                BoundedVec::try_from(b"cid".to_vec()).unwrap(),
                house1(),
                [7u8; 32],
                BoundedVec::try_from(b"prov".to_vec()).unwrap(),
                BoundedVec::try_from(b"city".to_vec()).unwrap(),
                approve,
            );
            match r {
                Ok(()) => frame_support::storage::TransactionOutcome::Commit(Ok(())),
                Err(e) => frame_support::storage::TransactionOutcome::Rollback(Err(e)),
            }
        },
    )
    .expect("referendum vote ok");
}

fn finalize_referendum(pid: u64) {
    let proposal = votingengine::pallet::Proposals::<Test>::get(pid).unwrap();
    frame_support::storage::with_transaction(
        || -> frame_support::storage::TransactionOutcome<sp_runtime::DispatchResult> {
            match Lib::do_finalize_referendum_timeout(&proposal, pid) {
                Ok(()) => frame_support::storage::TransactionOutcome::Commit(Ok(())),
                Err(e) => frame_support::storage::TransactionOutcome::Rollback(Err(e)),
            }
        },
    )
    .expect("finalize ok");
}
