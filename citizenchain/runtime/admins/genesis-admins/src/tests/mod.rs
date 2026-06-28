#![cfg(test)]

use super::*;
use admin_primitives::{AdminAccountKind, AdminAccountStatus};
use frame_support::{
    assert_noop, assert_ok, derive_impl,
    traits::{ConstU32, ConstU64},
};
use frame_system as system;
use primitives::cid::{
    china::{china_cb::CHINA_CB, china_ch::CHINA_CH, china_zf::CHINA_ZF},
    code::{code_bytes, NRC},
};
use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};
use votingengine::types::institution_code_from_cid_number;

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

    #[runtime::pallet_index(2)]
    pub type GenesisAdmins = super;

    #[runtime::pallet_index(3)]
    pub type InternalVote = internal_vote;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl system::Config for Test {
    type Block = Block;
    type AccountId = AccountId32;
    type Lookup = IdentityLookup<Self::AccountId>;
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
    type MaxCleanupQueueBucketLimit = ConstU32<50>;
    type MaxCleanupScheduleOffset = ConstU32<100>;
    type CleanupKeysPerStep = ConstU32<64>;
    type MaxProposalDataLen = ConstU32<{ 8 * 1024 }>;
    type MaxProposalObjectLen = ConstU32<{ 10 * 1024 }>;
    type MaxModuleTagLen = ConstU32<32>;
    type MaxManualExecutionAttempts = ConstU32<3>;
    type ExecutionRetryGraceBlocks = ConstU64<216>;
    type MaxExecutionRetryDeadlinesPerBlock = ConstU32<128>;
    type MaxPendingRetryExpirationsPerBlock = ConstU32<16>;
    type CidEligibility = ();
    type PopulationSnapshotVerifier = ();
    type JointVoteResultCallback = ();
    type InternalVoteResultCallback = ();
    type InternalAdminProvider = ();
    type InternalAdminsLenProvider = ();
    // 中文注释：测试上限沿用真实 runtime，覆盖联邦注册局的大管理员集合。
    type MaxAdminsPerInstitution = ConstU32<1989>;
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

/// 测试桩:公权机构管理员生命周期写入口。
///
/// 中文注释:现有 genesis-admins 单测不覆盖联邦直设入口,故 4 个生命周期方法仅占位返回 Ok;
/// `set_active_admin_account_direct` 走 trait 默认(未支持)。真实直设路径在 public-admins 单测覆盖。
pub struct TestPublicLifecycle;
impl admin_primitives::AdminAccountLifecycle<AccountId32> for TestPublicLifecycle {
    fn create_pending_admin_account_for_proposal(
        _proposal_id: u64,
        _module_tag: &[u8],
        _admin_root_account_id: AccountId32,
        _institution_code: votingengine::types::InstitutionCode,
        _kind: AdminAccountKind,
        _admins: alloc::vec::Vec<AccountId32>,
        _creator: AccountId32,
    ) -> frame_support::dispatch::DispatchResult {
        Ok(())
    }
    fn activate_admin_account_for_proposal(
        _proposal_id: u64,
        _module_tag: &[u8],
        _admin_root_account_id: AccountId32,
    ) -> frame_support::dispatch::DispatchResult {
        Ok(())
    }
    fn remove_pending_admin_account_for_proposal(
        _proposal_id: u64,
        _module_tag: &[u8],
        _admin_root_account_id: AccountId32,
    ) -> frame_support::dispatch::DispatchResult {
        Ok(())
    }
    fn close_admin_account_for_proposal(
        _proposal_id: u64,
        _module_tag: &[u8],
        _admin_root_account_id: AccountId32,
    ) -> frame_support::dispatch::DispatchResult {
        Ok(())
    }
}

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxAdminsPerInstitution = ConstU32<1989>;
    type MaxPersonalAccountAdmins = ConstU32<64>;
    type InternalVoteEngine = internal_vote::Pallet<Test>;
    type PublicAdminLifecycle = TestPublicLifecycle;
    type WeightInfo = ();
}

fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("test storage should build");
    GenesisConfig::<Test>::default()
        .assimilate_storage(&mut storage)
        .expect("genesis-admins genesis should assimilate");
    let mut ext: sp_io::TestExternalities = storage.into();
    ext.execute_with(|| System::set_block_number(1));
    ext
}

fn account(seed: u8) -> AccountId32 {
    AccountId32::new([seed; 32])
}

fn admins(count: u8) -> Vec<AccountId32> {
    (0..count).map(account).collect()
}

#[test]
fn genesis_build_only_inserts_genesis_admin_sources() {
    new_test_ext().execute_with(|| {
        // 中文注释：创世管理员唯一内置来源为国储会、省储会、省储行、联邦注册局；
        // 其中省储会/省储行各 43 个实例，所以总数不是 4 条账户。
        assert_eq!(
            AdminAccounts::<Test>::iter().count(),
            CHINA_CB.len() + CHINA_CH.len() + 1
        );

        let nrc_account = AccountId32::new(CHINA_CB[0].main_account);
        let prc_account = AccountId32::new(CHINA_CB[1].main_account);
        let prb_account = AccountId32::new(CHINA_CH[0].main_account);
        let frg_node = CHINA_ZF
            .iter()
            .find(|node| {
                institution_code_from_cid_number(node.cid_number) == Some(admin_primitives::FRG)
            })
            .expect("FRG must exist in china_zf");
        let frg_account = AccountId32::new(frg_node.main_account);

        for account_id in [nrc_account, prc_account, prb_account, frg_account] {
            let stored = AdminAccounts::<Test>::get(account_id.clone()).expect("genesis account");
            assert_eq!(stored.kind, AdminAccountKind::GenesisInstitution);
            assert_eq!(stored.status, AdminAccountStatus::Active);
            assert!(GenesisAdmins::is_genesis_protected(&account_id));
        }
    });
}

#[test]
fn genesis_admins_accept_only_genesis_codes() {
    new_test_ext().execute_with(|| {
        let root = account(90);
        assert_ok!(GenesisAdmins::do_create_pending_admin_account(
            root.clone(),
            admin_primitives::FRG,
            AdminAccountKind::GenesisInstitution,
            admins(3),
            account(1),
        ));
        assert!(GenesisAdmins::pending_account_exists_for_snapshot(
            admin_primitives::FRG,
            root
        ));

        assert_noop!(
            GenesisAdmins::do_create_pending_admin_account(
                account(91),
                code_bytes("PRS"),
                AdminAccountKind::GenesisInstitution,
                admins(3),
                account(1),
            ),
            Error::<Test>::InvalidAdminAccountKind
        );
    });
}

#[test]
fn fixed_governance_admin_count_is_locked() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            GenesisAdmins::do_create_pending_admin_account(
                account(92),
                NRC,
                AdminAccountKind::GenesisInstitution,
                admins(3),
                account(1),
            ),
            Error::<Test>::InvalidAdminsLen
        );
    });
}

#[test]
fn active_genesis_account_cannot_be_closed() {
    new_test_ext().execute_with(|| {
        let nrc_account = AccountId32::new(CHINA_CB[0].main_account);
        assert_noop!(
            GenesisAdmins::do_close_admin_account(nrc_account),
            Error::<Test>::BuiltinAdminAccountCannotClose
        );
    });
}
