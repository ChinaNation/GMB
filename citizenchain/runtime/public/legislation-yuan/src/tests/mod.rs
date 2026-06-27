#![cfg(test)]

//! 立法院模块第1步单测的 mock runtime。
//!
//! 中文注释:legislation-yuan 业务壳通过 `votingengine::Config` 复用投票引擎核心,
//! 通过自身 `Config::LegislationVoteEngine` 接立法投票引擎(第1步装 `()`)。
//! mock 里:System + VotingEngine + InternalVote(供引擎 finalizer)+ LegislationYuan,
//! LegislationVoteEngine 装 `()`,InternalAdminProvider 用 TestInternalAdminProvider。

use super::*;
use crate::pallet::{Article, Chapter, ChaptersOf, HousesOf, LawProposalSummary, Section};
use frame_support::{
    derive_impl, parameter_types,
    traits::{ConstU32, ConstU64},
    BoundedVec,
};
use frame_system as system;
use primitives::code::InstitutionCode;
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

    #[runtime::pallet_index(99)]
    pub type InternalVote = internal_vote;

    #[runtime::pallet_index(2)]
    pub type LegislationYuan = super;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl system::Config for Test {
    type Block = Block;
    type AccountId = AccountId32;
    type Lookup = IdentityLookup<Self::AccountId>;
}

// ───────── 测试身份常量 ─────────
/// 立法机构链上账户(归属机构)。
pub fn owner_body() -> AccountId32 {
    AccountId32::new([9u8; 32])
}
/// 现任议员/委员(owner_body 的 admin)。
pub fn legislator() -> AccountId32 {
    AccountId32::new([1u8; 32])
}
/// 非议员账户。
pub fn outsider() -> AccountId32 {
    AccountId32::new([2u8; 32])
}
/// 测试用立法机构机构码。
pub const OWNER_CODE: InstitutionCode = *b"NLY0";

pub struct TestCidEligibility;
pub struct TestPopulationSnapshotVerifier;
pub struct TestInternalAdminProvider;

impl votingengine::CidEligibility<AccountId32, <Test as frame_system::Config>::Hash>
    for TestCidEligibility
{
    fn is_eligible(_binding_id: &<Test as frame_system::Config>::Hash, _who: &AccountId32) -> bool {
        false
    }
    fn verify_and_consume_vote_credential(
        _binding_id: &<Test as frame_system::Config>::Hash,
        _who: &AccountId32,
        _proposal_id: u64,
        _nonce: &[u8],
        _signature: &[u8],
        _issuer_cid_number: &[u8],
        _issuer_main_account: &AccountId32,
        _signer_pubkey: &[u8; 32],
        _scope_province_name: &[u8],
        _scope_city_name: &[u8],
    ) -> bool {
        false
    }
    fn cleanup_vote_credentials(_proposal_id: u64) {}
}

impl
    votingengine::PopulationSnapshotVerifier<
        AccountId32,
        votingengine::pallet::VoteNonceOf<Test>,
        votingengine::pallet::VoteSignatureOf<Test>,
    > for TestPopulationSnapshotVerifier
{
    fn verify_population_snapshot(
        _who: &AccountId32,
        _eligible_total: u64,
        _nonce: &votingengine::pallet::VoteNonceOf<Test>,
        _signature: &votingengine::pallet::VoteSignatureOf<Test>,
        _issuer_cid_number: &[u8],
        _issuer_main_account: &AccountId32,
        _signer_pubkey: &[u8; 32],
        _scope_province_name: &[u8],
        _scope_city_name: &[u8],
    ) -> bool {
        true
    }
}

/// 中文注释:legislator() 是 owner_body() 的唯一管理员;其它一律不是。
impl votingengine::InternalAdminProvider<AccountId32> for TestInternalAdminProvider {
    fn is_internal_admin(
        _institution_code: InstitutionCode,
        institution: AccountId32,
        who: &AccountId32,
    ) -> bool {
        institution == owner_body() && *who == legislator()
    }
    fn get_admin_list(
        _institution_code: InstitutionCode,
        institution: AccountId32,
    ) -> Option<Vec<AccountId32>> {
        if institution == owner_body() {
            Some(vec![legislator()])
        } else {
            None
        }
    }
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
    type MaxCleanupQueueBucketLimit = ConstU32<50>;
    type MaxCleanupScheduleOffset = ConstU32<100>;
    type MaxPendingRetryExpirationsPerBlock = ConstU32<16>;
    type CidEligibility = TestCidEligibility;
    type PopulationSnapshotVerifier = TestPopulationSnapshotVerifier;
    type JointVoteResultCallback = ();
    type InternalVoteResultCallback = ();
    type InternalAdminProvider = TestInternalAdminProvider;
    type InternalAdminsLenProvider = ();
    type MaxAdminsPerInstitution = ConstU32<32>;
    type TimeProvider = TestTimeProvider;
    type WeightInfo = ();
    type InternalFinalizer = InternalVote;
    type InternalCleanup = InternalVote;
    type JointFinalizer = ();
    type JointCleanup = ();
    type LegislationVoteResultCallback = ();
    type LegislationFinalizer = ();
    type LegislationCleanup = ();
}

impl internal_vote::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

parameter_types! {
    pub const MaxTitleLen: u32 = 256;
    pub const MaxTextLen: u32 = 8192;
    pub const MaxClausesPerArticle: u32 = 50;
    pub const MaxArticlesPerSection: u32 = 200;
    pub const MaxSectionsPerChapter: u32 = 50;
    pub const MaxChaptersPerLaw: u32 = 50;
    pub const MaxLawsPerScope: u32 = 1000;
    pub const MaxActivationsPerBlock: u32 = 100;
}

impl crate::pallet::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    // 立法投票引擎单测装配为 ()(NotConfigured);端到端见 legislation-vote 测试。
    type LegislationVoteEngine = ();
    type MaxTitleLen = MaxTitleLen;
    type MaxTextLen = MaxTextLen;
    type MaxClausesPerArticle = MaxClausesPerArticle;
    type MaxArticlesPerSection = MaxArticlesPerSection;
    type MaxSectionsPerChapter = MaxSectionsPerChapter;
    type MaxChaptersPerLaw = MaxChaptersPerLaw;
    type MaxLawsPerScope = MaxLawsPerScope;
    type MaxActivationsPerBlock = MaxActivationsPerBlock;
    type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("test storage should build");
    let mut ext = sp_io::TestExternalities::new(storage);
    ext.execute_with(|| {
        System::set_block_number(1);
    });
    ext
}

// ───────── 测试数据构造 helper(章>节>条>款)─────────
/// 构造一条无款条文(body = 正文)。
pub fn article(number: u32, body: &[u8]) -> Article<Test> {
    Article::<Test> {
        number,
        title: BoundedVec::try_from(format!("第{number}条").into_bytes())
            .expect("title within bound"),
        title_en: None,
        body: BoundedVec::try_from(body.to_vec()).expect("body within bound"),
        body_en: None,
        clauses: BoundedVec::default(),
    }
}

/// 把若干条包成「1 章 1 节」的法律全文(测试用)。
pub fn chapters_of(articles: Vec<Article<Test>>) -> ChaptersOf<Test> {
    let section = Section::<Test> {
        number: 1,
        title: title("第一节".as_bytes()),
        title_en: None,
        articles: BoundedVec::try_from(articles).expect("articles within bound"),
    };
    let chapter = Chapter::<Test> {
        number: 1,
        title: title("第一章".as_bytes()),
        title_en: None,
        sections: BoundedVec::try_from(vec![section]).expect("sections within bound"),
    };
    BoundedVec::try_from(vec![chapter]).expect("chapters within bound")
}

pub fn title(s: &[u8]) -> BoundedVec<u8, MaxTitleLen> {
    BoundedVec::try_from(s.to_vec()).expect("title within bound")
}

/// 单院院序列(市立法会式):[(OWNER_CODE, owner_body())]。
pub fn houses() -> HousesOf<Test> {
    BoundedVec::try_from(vec![(OWNER_CODE, owner_body())]).expect("houses within bound")
}

// 签署机构(ADR-027 修订):行政机构(法定代表人=签署人)。
pub const EXEC_CODE: InstitutionCode = *b"CGOV";
pub fn exec_body() -> AccountId32 {
    AccountId32::new([80u8; 32])
}
/// 提案机构 =(OWNER_CODE, owner_body());legislator() 是其管理员。
pub fn proposer_body() -> (InstitutionCode, AccountId32) {
    (OWNER_CODE, owner_body())
}
/// 行政签署机构 =(EXEC_CODE, exec_body())。
pub fn executive() -> (InstitutionCode, AccountId32) {
    (EXEC_CODE, exec_body())
}

/// 直接构造一个 Enact 提案摘要(用于直调 write_law_version 预置法律)。
pub fn enact_summary(
    tier: Tier,
    scope_code: u32,
    vote_type: VoteType,
    title_bytes: &[u8],
) -> LawProposalSummary<Test> {
    LawProposalSummary::<Test> {
        action: LawAction::Enact,
        law_id: 0,
        tier,
        scope_code,
        houses: houses(),
        vote_type,
        title: title(title_bytes),
        title_en: None,
        content_hash: [0u8; 32],
        effective_at: 0,
    }
}

pub use crate::pallet::{
    Law, LawVersion, LawVersions, Laws, LawsByScope, NextLawId, PendingActivation,
};
pub type Lib = crate::pallet::Pallet<Test>;

mod cases;
