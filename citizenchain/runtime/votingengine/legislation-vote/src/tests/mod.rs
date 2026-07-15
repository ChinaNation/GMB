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
use primitives::cid::code::InstitutionCode;
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

// ───────── 测试机构/议员名册 ─────────
/// 院码(占位,TestInternalAdminProvider 不按码区分,只按机构账户)。
pub const HOUSE1_CODE: InstitutionCode = *b"NLH0"; // 众议会式
pub const HOUSE2_CODE: InstitutionCode = *b"NLS0"; // 参议会式
pub const HOUSE3_CODE: InstitutionCode = *b"EDU0"; // 与第一机构共享部分管理员的教育委员会式机构

pub fn house1() -> AccountId32 {
    AccountId32::new([91u8; 32])
}
pub fn house2() -> AccountId32 {
    AccountId32::new([92u8; 32])
}
pub fn house3() -> AccountId32 {
    AccountId32::new([93u8; 32])
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

// 签署机构(ADR-027 修订):行政机构(总统府/省联邦政府/市政府)+ 立法院(两院级,供院长)。
pub const EXEC_CODE: InstitutionCode = *b"CGOV"; // 行政机构(市政府式)
pub const LEG_CODE: InstitutionCode = *b"NLG\0"; // 立法院
pub fn exec_body() -> AccountId32 {
    AccountId32::new([80u8; 32])
}
/// 行政首长(市长/省长/总统)= 行政机构法定代表人。
pub fn exec_rep() -> AccountId32 {
    AccountId32::new([81u8; 32])
}
pub fn leg_body() -> AccountId32 {
    AccountId32::new([70u8; 32])
}
/// 立法院院长 = 立法院法定代表人。
pub fn leg_rep() -> AccountId32 {
    AccountId32::new([71u8; 32])
}

pub struct TestCitizenIdentityReader;
pub struct TestInternalAdminProvider;

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
}

/// 两院议员名册:house1 = 账户 1..=10;house2 = 账户 11..=20。
impl votingengine::InternalAdminProvider<AccountId32> for TestInternalAdminProvider {
    fn is_internal_admin(
        _institution_code: InstitutionCode,
        institution: AccountId32,
        who: &AccountId32,
    ) -> bool {
        Self::get_admin_list(_institution_code, institution)
            .map(|list| list.iter().any(|a| a == who))
            .unwrap_or(false)
    }
    fn get_admin_list(
        _institution_code: InstitutionCode,
        institution: AccountId32,
    ) -> Option<sp_runtime::sp_std::vec::Vec<AccountId32>> {
        if institution == house1() {
            Some((1u8..=10).map(member).collect())
        } else if institution == house2() {
            Some((11u8..=20).map(member).collect())
        } else if institution == house3() {
            Some((1u8..=10).map(member).collect())
        } else {
            None
        }
    }
    /// 法定代表人:众议长=house1[member 1] / 参议长=house2[member 11] / 院长=leg_rep / 行政首长=exec_rep。
    fn legal_representative(
        _institution_code: InstitutionCode,
        institution: AccountId32,
    ) -> Option<AccountId32> {
        if institution == house1() {
            Some(member(1))
        } else if institution == house2() {
            Some(member(11))
        } else if institution == house3() {
            Some(member(1))
        } else if institution == leg_body() {
            Some(leg_rep())
        } else if institution == exec_body() {
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

pub struct TestInstitutionQuery;
impl entity_primitives::InstitutionMultisigQuery<AccountId32> for TestInstitutionQuery {
    fn lookup_cid(addr: &AccountId32) -> Option<std::vec::Vec<u8>> {
        let mut cid = b"TEST-LEG-".to_vec();
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

    fn is_active(_addr: &AccountId32) -> bool {
        true
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
    type MaxAutoFinalizeWeightPerBlock = votingengine::weights::BlockWeightFraction<Test, 4>;
    type MaxExecutionWeightPerBlock = votingengine::weights::BlockWeightFraction<Test, 4>;
    type MaxCleanupWeightPerBlock = votingengine::weights::BlockWeightFraction<Test, 8>;
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
    type InstitutionQuery = TestInstitutionQuery;
    type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    set_guard_member_ids(&DEFAULT_GUARD_MEMBER_IDS);
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
    bodies: sp_runtime::sp_std::vec::Vec<(InstitutionCode, AccountId32)>,
    rule: RepresentativeVoteRule,
) -> u64 {
    create_inner(proposer, bodies, rule, false)
}

/// 修宪提案(needs_guard=true):现有流程通过后进护宪大法官终审。
pub fn create_guard(
    proposer: AccountId32,
    bodies: sp_runtime::sp_std::vec::Vec<(InstitutionCode, AccountId32)>,
    rule: RepresentativeVoteRule,
) -> u64 {
    create_inner(proposer, bodies, rule, true)
}

fn create_inner(
    proposer: AccountId32,
    bodies: sp_runtime::sp_std::vec::Vec<(InstitutionCode, AccountId32)>,
    rule: RepresentativeVoteRule,
    needs_guard: bool,
) -> u64 {
    // 单院(市)=无 legislature;两院(国/省)=携带立法院。行政签署机构恒携带。
    let legislature = if bodies.len() >= 2 {
        Some((LEG_CODE, leg_body()))
    } else {
        None
    };
    let bounded: RepresentativeBodies<AccountId32> =
        bodies.try_into().expect("representative route bounded");
    let route = if bounded.len() == 1 {
        RepresentativeRoute::Single(bounded.first().cloned().expect("single body"))
    } else {
        RepresentativeRoute::Sequential(bounded)
    };
    let pid = Lib::do_create_representative_proposal(
        proposer,
        route,
        rule,
        VoteProcedure::Legislation,
        Default::default(),
        Some(crate::pallet::LegislationMeta {
            executive: (EXEC_CODE, exec_body()),
            legislature,
            needs_guard,
            referendum_scope: None,
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
pub fn single_house() -> sp_runtime::sp_std::vec::Vec<(InstitutionCode, AccountId32)> {
    sp_runtime::sp_std::vec![(HOUSE1_CODE, house1())]
}
/// 两院院序列 [house1, house2]。
pub fn two_houses() -> sp_runtime::sp_std::vec::Vec<(InstitutionCode, AccountId32)> {
    sp_runtime::sp_std::vec![(HOUSE1_CODE, house1()), (HOUSE2_CODE, house2())]
}

/// 两个管理员名册重叠的代表机构，用于验证同一钱包按机构席位分别投票。
pub fn overlapping_bodies() -> sp_runtime::sp_std::vec::Vec<(InstitutionCode, AccountId32)> {
    sp_runtime::sp_std::vec![(HOUSE1_CODE, house1()), (HOUSE3_CODE, house3())]
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
