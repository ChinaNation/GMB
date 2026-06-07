//! 双层 ID + 反向索引专项测试(PR-Y / spec_version v1)。
//!
//! 覆盖:
//! - 主键 `proposal_id: u64` 全局纯单调,跨业务跨年都唯一连续 +1
//! - 展示号 `ProposalDisplayId[id] = (year, seq_in_year)` 跨年自动重置
//! - 4 张反向索引在 `register_proposal_data` 时同事务写入
//! - 清理路径(`FinalCleanup`)同步删除反向索引 + 展示号
//! - migrations/v1 回填存量提案的展示号与索引

use super::*;
// 这些 storage 在 votingengine 主 crate;dual_id 测试通过 super::* 拿到 votingengine::pallet 的 re-import。

/// 走 `_with_data` 路径触发 `register_proposal_data` 与反向索引写入。
fn create_general_internal_proposal_with_data_via_engine(
    who: AccountId32,
    org: u8,
    institution: AccountId32,
    module_tag: &[u8],
) -> u64 {
    <InternalVote as InternalVoteEngine<AccountId32>>::create_general_internal_proposal_with_data(
        who,
        org,
        institution,
        module_tag,
        b"payload".to_vec(),
    )
    .expect("internal proposal with data should be created")
}

/// 主键全局单调:首个 = 0,逐次 +1。
#[test]
fn proposal_id_is_globally_monotonic_starting_from_zero() {
    new_test_ext().execute_with(|| {
        let id0 = create_internal_proposal_via_engine(nrc_admin(0), ORG_NRC, nrc_pid());
        let id1 = create_internal_proposal_via_engine(nrc_admin(0), ORG_NRC, nrc_pid());
        let id2 = create_internal_proposal_via_engine(nrc_admin(0), ORG_NRC, nrc_pid());
        assert_eq!(id0, 0);
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(NextProposalId::<Test>::get(), 3);
    });
}

/// 跨年:主键单调累加,展示号 `seq_in_year` 重置为 0。
#[test]
fn display_meta_seq_in_year_resets_across_year_boundary() {
    new_test_ext().execute_with(|| {
        // 2026 年内两条
        let id0 = create_internal_proposal_via_engine(nrc_admin(0), ORG_NRC, nrc_pid());
        let id1 = create_internal_proposal_via_engine(nrc_admin(0), ORG_NRC, nrc_pid());
        assert_eq!(id0, 0);
        assert_eq!(id1, 1);
        let d0 = ProposalDisplayId::<Test>::get(id0).unwrap();
        let d1 = ProposalDisplayId::<Test>::get(id1).unwrap();
        assert_eq!(d0.year, 2026);
        assert_eq!(d0.seq_in_year, 0);
        assert_eq!(d1.year, 2026);
        assert_eq!(d1.seq_in_year, 1);

        // 跨到 2027
        set_test_now_secs(1_830_297_599);
        let id2 = create_internal_proposal_via_engine(nrc_admin(0), ORG_NRC, nrc_pid());
        assert_eq!(id2, 2); // 主键继续 +1
        let d2 = ProposalDisplayId::<Test>::get(id2).unwrap();
        assert_eq!(d2.year, 2027);
        assert_eq!(d2.seq_in_year, 0); // 年内序号重置
    });
}

/// 跨年累加器解 cap:无论多少都不返回 `YearCounterOverflow`(u32 上限 42.9 亿)。
///
/// 这里直接灌 100 万 + 1 条到 `YearProposalCounter`,模拟"千万级/年"目标的边界。
#[test]
fn year_proposal_counter_no_longer_capped_at_one_million() {
    new_test_ext().execute_with(|| {
        // 先创建一条让 CurrentProposalYear 进入 2026 分支(否则 stored_year=0 触发重置)
        let _ = create_internal_proposal_via_engine(nrc_admin(0), ORG_NRC, nrc_pid());
        // 强制把 YearProposalCounter 设为 v0 旧 cap,看新代码会不会再拒
        YearProposalCounter::<Test>::put(1_000_000u32);
        // 仍能成功创建(v0 在此处会 ProposalIdOverflow)
        let id = create_internal_proposal_via_engine(nrc_admin(0), ORG_NRC, nrc_pid());
        let display = ProposalDisplayId::<Test>::get(id).unwrap();
        assert_eq!(display.seq_in_year, 1_000_000);
        assert_eq!(YearProposalCounter::<Test>::get(), 1_000_001);
    });
}

/// 反向索引:`register_proposal_data` 后 4 张索引各有一条。
#[test]
fn reverse_indexes_populated_after_register_proposal_data() {
    new_test_ext().execute_with(|| {
        let id = create_general_internal_proposal_with_data_via_engine(
            nrc_admin(0),
            ORG_NRC,
            nrc_pid(),
            b"test-tag",
        );

        // ProposalsByOrg
        assert!(ProposalsByOrg::<Test>::contains_key(ORG_NRC, id));
        // ProposalsByInstitution
        assert!(ProposalsByInstitution::<Test>::contains_key(nrc_pid(), id));
        // ProposalsByYear
        let display = ProposalDisplayId::<Test>::get(id).unwrap();
        assert!(ProposalsByYear::<Test>::contains_key(display.year, id));
        // ProposalsByOwner — 用注册时传入的 module_tag
        let owner = votingengine::pallet::ProposalOwner::<Test>::get(id).expect("owner present");
        assert!(ProposalsByOwner::<Test>::contains_key(owner, id));
    });
}

/// 清理路径(FinalCleanup)同步删除 4 张反向索引 + ProposalDisplayId。
#[test]
fn final_cleanup_removes_indexes_and_display_id() {
    new_test_ext().execute_with(|| {
        let id = create_general_internal_proposal_with_data_via_engine(
            nrc_admin(0),
            ORG_NRC,
            nrc_pid(),
            b"cleanup-tag",
        );
        let display = ProposalDisplayId::<Test>::get(id).unwrap();
        let owner = votingengine::pallet::ProposalOwner::<Test>::get(id).expect("owner present");

        // 走完整 FinalCleanup 路径
        VotingEngine::cleanup_proposal_indexes(id);
        votingengine::pallet::Proposals::<Test>::remove(id);
        votingengine::pallet::ProposalOwner::<Test>::remove(id);

        assert!(!ProposalDisplayId::<Test>::contains_key(id));
        assert!(!ProposalsByOrg::<Test>::contains_key(ORG_NRC, id));
        assert!(!ProposalsByInstitution::<Test>::contains_key(nrc_pid(), id));
        assert!(!ProposalsByYear::<Test>::contains_key(display.year, id));
        assert!(!ProposalsByOwner::<Test>::contains_key(owner, id));
    });
}

/// migrations/v1:`on_runtime_upgrade` 把存量 v0 提案回填 ProposalDisplayId 与索引。
///
/// 模拟 v0 状态:`Proposals[id]` 存在但 ProposalDisplayId / 反向索引为空,
/// 跑迁移后所有索引应当被回填到位。
#[test]
fn migration_v1_backfills_display_id_and_indexes_for_legacy_proposals() {
    use frame_support::traits::OnRuntimeUpgrade;
    use votingengine::migrations::v1::MigrateToV1;
    use Proposal;

    new_test_ext().execute_with(|| {
        // 模拟 v0 旧主键格式:2026000007
        let legacy_id: u64 = 2_026_000_007;
        let proposal: Proposal<u64, AccountId32> = Proposal {
            kind: PROPOSAL_KIND_INTERNAL,
            stage: STAGE_INTERNAL,
            status: STATUS_VOTING,
            internal_org: Some(ORG_NRC),
            internal_institution: Some(nrc_pid()),
            start: 1,
            end: 100,
            citizen_eligible_total: 0,
        };
        votingengine::pallet::Proposals::<Test>::insert(legacy_id, proposal);

        // 模拟存量 owner(MODULE_TAG)
        let module_tag: BoundedVec<u8, <Test as votingengine::Config>::MaxModuleTagLen> =
            b"legacy".to_vec().try_into().unwrap();
        votingengine::pallet::ProposalOwner::<Test>::insert(legacy_id, module_tag.clone());

        // 升级前:ProposalDisplayId / 反向索引均为空
        assert!(!ProposalDisplayId::<Test>::contains_key(legacy_id));
        assert!(!ProposalsByOrg::<Test>::contains_key(ORG_NRC, legacy_id));

        // 跑迁移
        let _weight = MigrateToV1::<Test>::on_runtime_upgrade();

        // 升级后:展示号已回填,4 张索引各有一条
        let display = ProposalDisplayId::<Test>::get(legacy_id).expect("backfilled");
        assert_eq!(display.year, 2026);
        assert_eq!(display.seq_in_year, 7);
        assert!(ProposalsByOrg::<Test>::contains_key(ORG_NRC, legacy_id));
        assert!(ProposalsByInstitution::<Test>::contains_key(
            nrc_pid(),
            legacy_id
        ));
        assert!(ProposalsByOwner::<Test>::contains_key(
            module_tag, legacy_id
        ));
        assert!(ProposalsByYear::<Test>::contains_key(2026u16, legacy_id));
    });
}
