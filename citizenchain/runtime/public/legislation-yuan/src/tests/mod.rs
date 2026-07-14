#![cfg(test)]

//! 立法院模块第1步单测的 mock runtime。
//!
//! legislation-yuan 业务壳通过 `votingengine::Config` 复用投票引擎核心,
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
use primitives::cid::code::InstitutionCode;
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

    #[runtime::pallet_index(3)]
    pub type Timestamp = pallet_timestamp;

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

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = ConstU64<1>;
    type WeightInfo = ();
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
/// 测试默认使用国家众议会作为发起机构。
pub const OWNER_CODE: InstitutionCode = *b"NRP\0";

pub struct TestCitizenIdentityReader;
impl votingengine::CitizenIdentityReader<AccountId32> for TestCitizenIdentityReader {
    fn can_vote(_who: &AccountId32, _scope: &votingengine::PopulationScope) -> bool {
        false
    }

    fn can_be_candidate(_who: &AccountId32, _scope: &votingengine::PopulationScope) -> bool {
        false
    }

    fn population_count(_scope: &votingengine::PopulationScope) -> u64 {
        100
    }
}

pub struct TestInternalAdminProvider;

fn account_tag(account: &AccountId32) -> u8 {
    let raw: &[u8] = account.as_ref();
    raw[0]
}

/// legislator() 是 owner_body() 的唯一管理员;其它一律不是。
impl votingengine::InternalAdminProvider<AccountId32> for TestInternalAdminProvider {
    fn is_internal_admin(
        _institution_code: InstitutionCode,
        institution: AccountId32,
        who: &AccountId32,
    ) -> bool {
        matches!(account_tag(&institution), 9 | 13 | 16) && *who == legislator()
    }
    fn get_admin_list(
        _institution_code: InstitutionCode,
        institution: AccountId32,
    ) -> Option<Vec<AccountId32>> {
        if matches!(account_tag(&institution), 9 | 13 | 16) {
            Some(vec![legislator()])
        } else {
            None
        }
    }
}

/// 立法路由机构查询夹具。每个账户只绑定一个机构码和一个有效 CID；
/// 市级机构统一落在 GD002，确保同市校验能真实覆盖。
pub struct TestInstitutionQuery;

impl TestInstitutionQuery {
    fn code(addr: &AccountId32) -> Option<InstitutionCode> {
        match account_tag(addr) {
            9 => Some(*b"NRP\0"),
            10 => Some(*b"NSN\0"),
            11 => Some(*b"PRS\0"),
            12 => Some(*b"NLG\0"),
            13 => Some(*b"CSLF"),
            14 => Some(*b"CLEG"),
            15 => Some(*b"CGOV"),
            16 => Some(*b"CEDU"),
            _ => None,
        }
    }

    fn cid(code: InstitutionCode) -> Vec<u8> {
        let code_text =
            primitives::cid::code::institution_code_text(&code).expect("test institution code");
        let regional =
            matches!(code, c if c == *b"CSLF" || c == *b"CLEG" || c == *b"CGOV" || c == *b"CEDU");
        primitives::cid::generator::generate_cid_number(
            primitives::cid::generator::GenerateCidNumberInput {
                account_pubkey: "0x1234",
                p1: "0",
                province_code: if regional { "GD" } else { "ZS" },
                province_name: if regional { "广东省" } else { "中枢省" },
                city_code: if regional { "002" } else { "001" },
                city_name: "测试市",
                year: "2026",
                institution: code_text,
            },
        )
        .expect("test cid")
        .into_bytes()
    }
}

impl entity_primitives::InstitutionMultisigQuery<AccountId32> for TestInstitutionQuery {
    fn lookup_cid(addr: &AccountId32) -> Option<Vec<u8>> {
        Self::code(addr).map(Self::cid)
    }

    fn lookup_org(addr: &AccountId32) -> Option<InstitutionCode> {
        Self::code(addr)
    }

    fn lookup_admin_config(
        _addr: &AccountId32,
    ) -> Option<primitives::multisig::MultisigConfigSnapshot<AccountId32>> {
        None
    }

    fn is_active(addr: &AccountId32) -> bool {
        Self::code(addr).is_some()
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
    type CitizenIdentityReader = TestCitizenIdentityReader;
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
    type ElectionVoteResultCallback = ();
    type ElectionFinalizer = ();
    type ElectionCleanup = ();
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
    pub const MaxPendingActivations: u32 = 100;
}

impl crate::pallet::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    // 立法投票引擎单测装配为 ()(NotConfigured);端到端见 legislation-vote 测试。
    type LegislationVoteEngine = ();
    type InstitutionQuery = TestInstitutionQuery;
    type MaxTitleLen = MaxTitleLen;
    type MaxTextLen = MaxTextLen;
    type MaxClausesPerArticle = MaxClausesPerArticle;
    type MaxArticlesPerSection = MaxArticlesPerSection;
    type MaxSectionsPerChapter = MaxSectionsPerChapter;
    type MaxChaptersPerLaw = MaxChaptersPerLaw;
    type MaxLawsPerScope = MaxLawsPerScope;
    type MaxPendingActivations = MaxPendingActivations;
    type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("test storage should build");
    let mut ext = sp_io::TestExternalities::new(storage);
    ext.execute_with(|| {
        System::set_block_number(1);
        Timestamp::set_timestamp(1_000);
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

/// 构造两章宪法:核心章(第一章总则)+ 一般章(第二章),供第十九条章→档位测试。
/// 核心章(`chapters[0]`)条款改动要求特别案,一般章条款改动要求重要案。
pub fn chapters_core_general(
    core_articles: Vec<Article<Test>>,
    general_articles: Vec<Article<Test>>,
) -> ChaptersOf<Test> {
    let make_chapter = |number: u32, ch_title: &str, articles: Vec<Article<Test>>| Chapter::<Test> {
        number,
        title: title(ch_title.as_bytes()),
        title_en: None,
        sections: BoundedVec::try_from(vec![Section::<Test> {
            number: 1,
            title: title("第一节".as_bytes()),
            title_en: None,
            articles: BoundedVec::try_from(articles).expect("articles within bound"),
        }])
        .expect("sections within bound"),
    };
    BoundedVec::try_from(vec![
        make_chapter(1, "第一章", core_articles),
        make_chapter(2, "第二章", general_articles),
    ])
    .expect("chapters within bound")
}

pub fn title(s: &[u8]) -> BoundedVec<u8, MaxTitleLen> {
    BoundedVec::try_from(s.to_vec()).expect("title within bound")
}

/// 国家两院序列：国家众议会 → 国家参议会。
pub fn houses() -> HousesOf<Test> {
    BoundedVec::try_from(vec![
        (OWNER_CODE, owner_body()),
        (*b"NSN\0", AccountId32::new([10u8; 32])),
    ])
    .expect("houses within bound")
}

// 国家行政签署机构(法定代表人=签署人)。
pub const EXEC_CODE: InstitutionCode = *b"PRS\0";
pub fn exec_body() -> AccountId32 {
    AccountId32::new([11u8; 32])
}
/// 提案机构 =(OWNER_CODE, owner_body());legislator() 是其管理员。
pub fn proposer_body() -> (InstitutionCode, AccountId32) {
    (OWNER_CODE, owner_body())
}
/// 行政签署机构 =(EXEC_CODE, exec_body())。
pub fn executive() -> (InstitutionCode, AccountId32) {
    (EXEC_CODE, exec_body())
}

pub fn legislature() -> Option<(InstitutionCode, AccountId32)> {
    Some((*b"NLG\0", AccountId32::new([12u8; 32])))
}

pub fn municipal_houses() -> HousesOf<Test> {
    BoundedVec::try_from(vec![(*b"CLEG", AccountId32::new([14u8; 32]))])
        .expect("municipal houses within bound")
}

pub fn municipal_proposer_body() -> (InstitutionCode, AccountId32) {
    (*b"CSLF", AccountId32::new([13u8; 32]))
}

pub fn municipal_education_proposer_body() -> (InstitutionCode, AccountId32) {
    (*b"CEDU", AccountId32::new([16u8; 32]))
}

pub fn municipal_executive() -> (InstitutionCode, AccountId32) {
    (*b"CGOV", AccountId32::new([15u8; 32]))
}

/// 直接构造一个 Enact 提案摘要(用于直调 write_law_version 预置法律)。
pub fn enact_summary(
    tier: Tier,
    scope_code: u32,
    vote_type: VoteType,
    title_bytes: &[u8],
) -> LawProposalSummary<Test> {
    let (houses, proposer_body, executive, legislature) = match tier {
        Tier::Municipal => (
            municipal_houses(),
            municipal_proposer_body(),
            municipal_executive(),
            None,
        ),
        _ => (houses(), proposer_body(), executive(), legislature()),
    };
    LawProposalSummary::<Test> {
        action: LawAction::Enact,
        law_id: 0,
        tier,
        scope_code,
        houses,
        proposer_body,
        executive,
        legislature,
        vote_type,
        title: title(title_bytes),
        title_en: None,
        content_hash: [0u8; 32],
        effective_at: 0,
    }
}

pub use crate::pallet::{
    Law, LawVersion, LawVersions, Laws, LawsByScope, NextLawId, PendingActivations,
};
pub type Lib = crate::pallet::Pallet<Test>;

mod cases;
