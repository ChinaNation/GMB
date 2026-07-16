#![cfg(test)]

//! 立法投票 sub-pallet 单测 mock runtime。
//!
//! System + VotingEngine + InternalVote(供 votingengine 必填 finalizer)+ LegislationVote。
//! votingengine::Config 通过 TrackHandlers 注册 LegislationVote，
//! LegislationVoteResultCallback 装 `()`(本 sub-pallet 单测只验投票机制,不验业务写法律)。
//! TestInternalAdminProvider 定义两院议员名册;公投公民资格从测试用 CitizenIdentityReader 返回。

use frame_support::{
    derive_impl,
    traits::{ConstU32, ConstU64, Hooks},
};
use frame_system as system;
use primitives::cid::{
    china::{china_jy::CHINA_JY, china_lf::CHINA_LF, china_zf::CHINA_ZF},
    code::institution_code_from_cid_number,
};
use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};
use std::cell::RefCell;

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

    #[runtime::pallet_index(99)]
    pub type InternalVote = internal_vote;

    #[runtime::pallet_index(2)]
    pub type LegislationVote = super;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl system::Config for Test {
    type Block = Block;
    type AccountId = AccountId32;
    type Lookup = IdentityLookup<Self::AccountId>;
}

// ───────── 测试机构 CID / 议员名册 ─────────
fn bounded_cid(cid_number: &str) -> votingengine::types::CidNumber {
    cid_number
        .as_bytes()
        .to_vec()
        .try_into()
        .expect("built-in institution CID should fit")
}

/// 业务发起机构与代表表决机构分离，权限统一按 actor CID 下的 admins 校验。
pub fn actor_cid_number() -> votingengine::types::CidNumber {
    bounded_cid(CHINA_ZF[5].cid_number)
}

pub fn house1() -> votingengine::types::CidNumber {
    bounded_cid(CHINA_LF[2].cid_number)
}
pub fn house2() -> votingengine::types::CidNumber {
    bounded_cid(CHINA_LF[1].cid_number)
}
pub fn house3() -> votingengine::types::CidNumber {
    bounded_cid(CHINA_JY[0].cid_number)
}
/// house1 议员 = 账户 [1..=10];house2 议员 = 账户 [11..=20]。
pub fn member(idx: u8) -> AccountId32 {
    AccountId32::new([idx; 32])
}

const DEFAULT_GUARD_MEMBER_IDS: [u8; 7] = [101, 102, 103, 104, 105, 106, 107];

thread_local! {
    static GUARD_MEMBER_IDS: RefCell<std::vec::Vec<u8>> =
        RefCell::new(DEFAULT_GUARD_MEMBER_IDS.to_vec());
}

pub fn set_guard_member_ids(ids: &[u8]) {
    GUARD_MEMBER_IDS.with(|members| {
        *members.borrow_mut() = ids.to_vec();
    });
}

// 签署机构(ADR-027 修订):行政机构 + 立法院(两院级,供院长)。
pub fn exec_body() -> votingengine::types::CidNumber {
    bounded_cid(CHINA_ZF[0].cid_number)
}
/// 行政首长(市长/省长/总统)= 行政机构法定代表人。
pub fn exec_rep() -> AccountId32 {
    AccountId32::new([81u8; 32])
}
pub fn leg_body() -> votingengine::types::CidNumber {
    bounded_cid(CHINA_LF[0].cid_number)
}
/// 立法院院长 = 立法院法定代表人。
pub fn leg_rep() -> AccountId32 {
    AccountId32::new([71u8; 32])
}

pub struct TestCitizenIdentityReader;
pub struct TestInternalAdminProvider;

thread_local! {
    static TEST_POPULATION_SNAPSHOT_ID: RefCell<u64> = const { RefCell::new(0) };
}

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

    fn create_population_snapshot(
        _scope: &votingengine::PopulationScope,
    ) -> Result<(u64, u64), sp_runtime::DispatchError> {
        let snapshot_id = TEST_POPULATION_SNAPSHOT_ID.with(|next| {
            let mut next = next.borrow_mut();
            let snapshot_id = *next;
            *next = (*next).saturating_add(1);
            snapshot_id
        });
        Ok((snapshot_id, 100))
    }

    fn can_vote_at(_who: &AccountId32, _snapshot_id: u64) -> bool {
        true
    }
}

impl TestInternalAdminProvider {
    fn institution_admins(
        institution_code: primitives::cid::code::InstitutionCode,
        cid_number: &[u8],
    ) -> Option<sp_runtime::sp_std::vec::Vec<AccountId32>> {
        let cid_text = core::str::from_utf8(cid_number).ok()?;
        if institution_code_from_cid_number(cid_text) != Some(institution_code) {
            return None;
        }
        if cid_number == actor_cid_number().as_slice() {
            Some(sp_runtime::sp_std::vec![member(1), member(50)])
        } else if cid_number == house1().as_slice() {
            Some((1u8..=10).map(member).collect())
        } else if cid_number == house2().as_slice() {
            Some((11u8..=20).map(member).collect())
        } else if cid_number == house3().as_slice() {
            Some((1u8..=10).map(member).collect())
        } else {
            None
        }
    }
}

/// 代表机构管理员与发起机构权限全部按 CID 路由，不再用机构账户充当主体。
impl votingengine::InternalAdminProvider<AccountId32> for TestInternalAdminProvider {
    fn is_institution_admin(
        institution_code: primitives::cid::code::InstitutionCode,
        cid_number: &[u8],
        who: &AccountId32,
    ) -> bool {
        Self::institution_admins(institution_code, cid_number)
            .map(|list| list.iter().any(|admin| admin == who))
            .unwrap_or(false)
    }

    fn get_institution_admins(
        institution_code: primitives::cid::code::InstitutionCode,
        cid_number: &[u8],
    ) -> Option<sp_runtime::sp_std::vec::Vec<AccountId32>> {
        Self::institution_admins(institution_code, cid_number)
    }

    /// 法定代表人:众议长=house1[member 1] / 参议长=house2[member 11] / 院长=leg_rep / 行政首长=exec_rep。
    fn legal_representative(cid_number: &[u8]) -> Option<AccountId32> {
        if cid_number == house1().as_slice() {
            Some(member(1))
        } else if cid_number == house2().as_slice() {
            Some(member(11))
        } else if cid_number == house3().as_slice() {
            Some(member(1))
        } else if cid_number == leg_body().as_slice() {
            Some(leg_rep())
        } else if cid_number == exec_body().as_slice() {
            Some(exec_rep())
        } else {
            None
        }
    }
    /// 护宪大法官默认 7 人 = 账户 [101..=107](测试注入;生产按 NJD admins 的 admin_role 过滤)。
    fn constitution_guard_members() -> sp_runtime::sp_std::vec::Vec<AccountId32> {
        GUARD_MEMBER_IDS.with(|ids| ids.borrow().iter().copied().map(member).collect())
    }
}

pub struct TestTimeProvider;
impl frame_support::traits::UnixTime for TestTimeProvider {
    fn now() -> core::time::Duration {
        core::time::Duration::from_secs(1_782_864_000)
    }
}

/// 测试业务回调:模拟业务壳认领立法提案并返回 Executed(真实 runtime 接 LegislationYuan)。
pub struct TestLegislationCallback;
impl votingengine::LegislationVoteResultCallback for TestLegislationCallback {
    fn on_legislation_vote_finalized(
        _vote_proposal_id: u64,
        _approved: bool,
    ) -> Result<votingengine::ProposalExecutionOutcome, sp_runtime::DispatchError> {
        Ok(votingengine::ProposalExecutionOutcome::Executed)
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
    type CleanupKeysPerStep = ConstU32<64>;
    type MaxProposalDataLen = ConstU32<1024>;
    type MaxProposalObjectLen = ConstU32<{ 64 * 1024 }>;
    type MaxModuleTagLen = ConstU32<32>;
    type MaxManualExecutionAttempts = ConstU32<3>;
    type ExecutionRetryGraceBlocks = ConstU64<216>;
    type MaxExecutionRetryDeadlinesPerBlock = ConstU32<128>;
    type MaxCleanupActivationsPerBlock = ConstU32<50>;
    type MaxPendingRetryExpirationsPerBlock = ConstU32<16>;
    type CitizenIdentityReader = TestCitizenIdentityReader;
    type JointVoteResultCallback = ();
    type InternalVoteResultCallback = ();
    type InternalAdminProvider = TestInternalAdminProvider;
    type InternalAdminsLenProvider = ();
    type MaxAdminsPerInstitution = ConstU32<64>;
    type TimeProvider = TestTimeProvider;
    type WeightInfo = ();
    type TrackHandlers = (InternalVote, (LegislationVote, ()));
    type LegislationVoteResultCallback = (TestLegislationCallback,);
    type ElectionVoteResultCallback = ();
}

impl internal_vote::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

impl crate::pallet::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    set_guard_member_ids(&DEFAULT_GUARD_MEMBER_IDS);
    TEST_POPULATION_SNAPSHOT_ID.with(|next| *next.borrow_mut() = 0);
    let storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("test storage should build");
    let mut ext = sp_io::TestExternalities::new(storage);
    ext.execute_with(|| {
        System::set_block_number(1);
    });
    ext
}

// ───────── 测试 helper ─────────
pub type Lib = crate::pallet::Pallet<Test>;
pub use crate::pallet::RepresentativeMetas;
use crate::{RepresentativeBodies, RepresentativeRoute, RepresentativeVoteRule, VoteProcedure};

/// 创建立法提案并注册 ProposalData(设置 ProposalOwner,终态回调需要),不自动投票。
pub fn create(
    proposer: AccountId32,
    bodies: sp_runtime::sp_std::vec::Vec<votingengine::types::CidNumber>,
    rule: RepresentativeVoteRule,
) -> u64 {
    create_inner(proposer, bodies, rule, false)
}

/// 修宪提案(needs_guard=true):现有流程通过后进护宪大法官终审。
pub fn create_guard(
    proposer: AccountId32,
    bodies: sp_runtime::sp_std::vec::Vec<votingengine::types::CidNumber>,
    rule: RepresentativeVoteRule,
) -> u64 {
    create_inner(proposer, bodies, rule, true)
}

fn create_inner(
    proposer: AccountId32,
    bodies: sp_runtime::sp_std::vec::Vec<votingengine::types::CidNumber>,
    rule: RepresentativeVoteRule,
    needs_guard: bool,
) -> u64 {
    // 单院(市)=无 legislature;两院(国/省)=携带立法院。行政签署机构恒携带。
    let legislature = if bodies.len() >= 2 {
        Some(leg_body())
    } else {
        None
    };
    let bounded: RepresentativeBodies = bodies.try_into().expect("representative route bounded");
    let route = if bounded.len() == 1 {
        RepresentativeRoute::Single(bounded.first().cloned().expect("single body"))
    } else {
        RepresentativeRoute::Sequential(bounded)
    };
    let pid = Lib::do_create_representative_proposal(
        proposer,
        actor_cid_number(),
        route,
        rule,
        VoteProcedure::Legislation,
        votingengine::types::ProposalSubjectCidNumbers::new(),
        Some(crate::pallet::LegislationMeta {
            executive: exec_body(),
            legislature,
            needs_guard,
        }),
    )
    .expect("proposal created");
    let now = System::block_number();
    votingengine::Pallet::<Test>::register_proposal_data(
        pid,
        b"leg-yuan",
        sp_runtime::sp_std::vec![1u8],
        now,
    )
    .expect("register proposal data");
    pid
}

/// 单院院序列 [house1]。
pub fn single_house() -> sp_runtime::sp_std::vec::Vec<votingengine::types::CidNumber> {
    sp_runtime::sp_std::vec![house1()]
}
/// 两院院序列 [house1, house2]。
pub fn two_houses() -> sp_runtime::sp_std::vec::Vec<votingengine::types::CidNumber> {
    sp_runtime::sp_std::vec![house1(), house2()]
}

/// 两个管理员名册重叠的代表机构，用于验证同一钱包按机构席位分别投票。
pub fn overlapping_bodies() -> sp_runtime::sp_std::vec::Vec<votingengine::types::CidNumber> {
    sp_runtime::sp_std::vec![house1(), house3()]
}

/// 当前提案状态(从核心读)。
pub fn status(pid: u64) -> u8 {
    votingengine::pallet::Proposals::<Test>::get(pid)
        .expect("proposal exists")
        .status
}
pub fn stage(pid: u64) -> u8 {
    votingengine::pallet::Proposals::<Test>::get(pid)
        .expect("proposal exists")
        .stage
}

/// 投一票(事务内,因 set_status_and_emit 需在事务中)。
pub fn cast(who: AccountId32, pid: u64, approve: bool) -> sp_runtime::DispatchResult {
    let result = frame_support::storage::with_transaction(
        || -> frame_support::storage::TransactionOutcome<sp_runtime::DispatchResult> {
            match Lib::do_cast_representative_vote(who, pid, approve) {
                Ok(()) => frame_support::storage::TransactionOutcome::Commit(Ok(())),
                Err(e) => frame_support::storage::TransactionOutcome::Rollback(Err(e)),
            }
        },
    );
    if result.is_ok() {
        process_current_block();
    }
    result
}

/// 行政签署(事务内)。
pub fn exec_sign(who: AccountId32, pid: u64, approve: bool) -> sp_runtime::DispatchResult {
    let result = frame_support::storage::with_transaction(
        || -> frame_support::storage::TransactionOutcome<sp_runtime::DispatchResult> {
            match Lib::do_executive_sign(who, pid, approve) {
                Ok(()) => frame_support::storage::TransactionOutcome::Commit(Ok(())),
                Err(e) => frame_support::storage::TransactionOutcome::Rollback(Err(e)),
            }
        },
    );
    if result.is_ok() {
        process_current_block();
    }
    result
}

/// 三人会签(事务内)。
pub fn override_sign(who: AccountId32, pid: u64, approve: bool) -> sp_runtime::DispatchResult {
    let result = frame_support::storage::with_transaction(
        || -> frame_support::storage::TransactionOutcome<sp_runtime::DispatchResult> {
            match Lib::do_override_sign(who, pid, approve) {
                Ok(()) => frame_support::storage::TransactionOutcome::Commit(Ok(())),
                Err(e) => frame_support::storage::TransactionOutcome::Rollback(Err(e)),
            }
        },
    );
    if result.is_ok() {
        process_current_block();
    }
    result
}

/// 护宪大法官终审表决(事务内)。
pub fn guard_vote(who: AccountId32, pid: u64, approve: bool) -> sp_runtime::DispatchResult {
    let result = frame_support::storage::with_transaction(
        || -> frame_support::storage::TransactionOutcome<sp_runtime::DispatchResult> {
            match Lib::do_guard_vote(who, pid, approve) {
                Ok(()) => frame_support::storage::TransactionOutcome::Commit(Ok(())),
                Err(e) => frame_support::storage::TransactionOutcome::Rollback(Err(e)),
            }
        },
    );
    if result.is_ok() {
        process_current_block();
    }
    result
}

/// 执行当前区块维护钩子，让 PASSED 立法提案完成一次异步业务执行。
pub fn process_current_block() {
    let now = System::block_number();
    votingengine::Pallet::<Test>::on_initialize(now);
}

/// 推进到提案 end 之后并触发到期结算(用于签署/会签超时测试)。
pub fn run_to_expiry(pid: u64) {
    let end = votingengine::pallet::Proposals::<Test>::get(pid)
        .expect("proposal exists")
        .end;
    // 到期桶挂在 end+1;推进到 end+1 并触发 on_initialize 自动结算。
    System::set_block_number(end + 1);
    votingengine::Pallet::<Test>::on_initialize(end + 1);
}

mod cases;
