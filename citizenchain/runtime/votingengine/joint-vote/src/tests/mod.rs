//! 最小化测试 runtime,目前仅承载 migration v1 的 storage prefix 搬运验证。
//!
//! 业务逻辑测试在 internal-vote/tests 已完整覆盖(那里的 mock runtime 同时
//! 注册 VotingEngine + InternalVote + JointVote 跑端到端)。本文件只为
//! `migrations::v1::tests` 提供能让 `Pallet::<T>::on_chain_storage_version()` 跑通
//! 的最小 Config,所以全部 trait 用 `()` 默认 impl。

use frame_support::derive_impl;
use frame_support::traits::ConstU32;
use frame_system as system;
use sp_io::TestExternalities;
use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};

type Block = system::mocking::MockBlock<Test>;

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
    pub type JointVote = super;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl system::Config for Test {
    type Block = Block;
    type AccountId = AccountId32;
    type Lookup = IdentityLookup<Self::AccountId>;
}

/// 时间源 stub:迁移测试不依赖时间,常返一个固定值即可。
pub struct StubTime;
impl frame_support::traits::UnixTime for StubTime {
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
    type MaxCleanupStepsPerBlock = ConstU32<3>;
    type MaxCleanupQueueBucketLimit = ConstU32<50>;
    type MaxCleanupScheduleOffset = ConstU32<100>;
    type CleanupKeysPerStep = ConstU32<2>;
    type MaxProposalDataLen = ConstU32<4096>;
    type MaxProposalObjectLen = ConstU32<10_240>;
    type MaxModuleTagLen = ConstU32<32>;
    type MaxManualExecutionAttempts = ConstU32<3>;
    type ExecutionRetryGraceBlocks = frame_support::traits::ConstU64<216>;
    type MaxExecutionRetryDeadlinesPerBlock = ConstU32<128>;
    type MaxPendingRetryExpirationsPerBlock = ConstU32<16>;
    type SfidEligibility = ();
    type PopulationSnapshotVerifier = ();
    type JointVoteResultCallback = ();
    type InternalVoteResultCallback = ();
    type InternalAdminProvider = ();
    type InternalAdminCountProvider = ();
    type MaxAdminsPerInstitution = ConstU32<32>;
    type TimeProvider = StubTime;
    type WeightInfo = ();
    type InternalFinalizer = ();
    type InternalCleanup = ();
    type JointFinalizer = JointVote;
    type JointCleanup = JointVote;
}

impl crate::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

pub fn new_test_ext() -> TestExternalities {
    let t = system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("test storage build");
    TestExternalities::new(t)
}
