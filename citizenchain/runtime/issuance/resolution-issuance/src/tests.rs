use super::*;
use core::cell::RefCell;
use frame_support::{
    assert_noop, assert_ok, derive_impl,
    traits::{ConstU128, ConstU32},
    BoundedVec,
};
use frame_system as system;
use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage, DispatchError};

type Balance = u128;
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
    pub type Balances = pallet_balances;

    #[runtime::pallet_index(2)]
    pub type VotingEngine = voting_engine;

    #[runtime::pallet_index(8)]
    pub type ResolutionIssuance = super;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl system::Config for Test {
    type Block = Block;
    type AccountId = AccountId32;
    type Lookup = IdentityLookup<Self::AccountId>;
    type AccountData = pallet_balances::AccountData<Balance>;
}

impl pallet_balances::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<10>;
    type AccountStore = System;
    type MaxLocks = ConstU32<0>;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type FreezeIdentifier = RuntimeFreezeReason;
    type MaxFreezes = ConstU32<0>;
    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeFreezeReason = RuntimeFreezeReason;
    type DoneSlashHandler = ();
    type WeightInfo = ();
}

pub struct EnsureNrcAdminForTest;
impl frame_support::traits::EnsureOrigin<RuntimeOrigin> for EnsureNrcAdminForTest {
    type Success = AccountId32;

    fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
        let who = frame_system::EnsureSigned::<AccountId32>::try_origin(o)?;
        if who == AccountId32::new([1u8; 32]) {
            Ok(who)
        } else {
            Err(RuntimeOrigin::from(frame_system::RawOrigin::Signed(who)))
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn try_successful_origin() -> Result<RuntimeOrigin, ()> {
        Ok(RuntimeOrigin::signed(AccountId32::new([1u8; 32])))
    }
}

thread_local! {
    static NEXT_JOINT_ID: RefCell<u64> = const { RefCell::new(100) };
}

pub struct TestJointVoteEngine;
impl voting_engine::JointVoteEngine<AccountId32> for TestJointVoteEngine {
    fn create_joint_proposal(
        _who: AccountId32,
        eligible_total: u64,
        snapshot_nonce: &[u8],
        signature: &[u8],
        province: &[u8],
        _signer_admin_pubkey: &[u8; 32],
    ) -> Result<u64, DispatchError> {
        if eligible_total == 0
            || snapshot_nonce.is_empty()
            || signature.is_empty()
            || province.is_empty()
        {
            return Err(DispatchError::Other("bad snapshot"));
        }
        NEXT_JOINT_ID.with(|id| {
            let mut id = id.borrow_mut();
            let v = *id;
            *id = id.saturating_add(1);
            Ok(v)
        })
    }

    fn create_joint_proposal_with_data(
        who: AccountId32,
        eligible_total: u64,
        snapshot_nonce: &[u8],
        signature: &[u8],
        province: &[u8],
        signer_admin_pubkey: &[u8; 32],
        module_tag: &[u8],
        data: Vec<u8>,
    ) -> Result<u64, DispatchError> {
        let proposal_id = Self::create_joint_proposal(
            who,
            eligible_total,
            snapshot_nonce,
            signature,
            province,
            signer_admin_pubkey,
        )?;
        let bounded_data: frame_support::BoundedVec<
            u8,
            <Test as voting_engine::Config>::MaxProposalDataLen,
        > = data
            .try_into()
            .map_err(|_| DispatchError::Other("proposal data too large"))?;
        let owner: frame_support::BoundedVec<u8, <Test as voting_engine::Config>::MaxModuleTagLen> =
            module_tag
                .to_vec()
                .try_into()
                .map_err(|_| DispatchError::Other("module tag too large"))?;
        voting_engine::ProposalData::<Test>::insert(proposal_id, bounded_data);
        voting_engine::ProposalOwner::<Test>::insert(proposal_id, owner);
        Ok(proposal_id)
    }
}

pub struct TestSfidEligibility;
impl voting_engine::SfidEligibility<AccountId32, <Test as frame_system::Config>::Hash>
    for TestSfidEligibility
{
    fn is_eligible(_binding_id: &<Test as frame_system::Config>::Hash, _who: &AccountId32) -> bool {
        true
    }

    fn verify_and_consume_vote_credential(
        _binding_id: &<Test as frame_system::Config>::Hash,
        _who: &AccountId32,
        _proposal_id: u64,
        _nonce: &[u8],
        _signature: &[u8],
        _province: &[u8],
        _signer_admin_pubkey: &[u8; 32],
    ) -> bool {
        true
    }
}

pub struct TestPopulationSnapshotVerifier;
impl
    voting_engine::PopulationSnapshotVerifier<
        AccountId32,
        voting_engine::pallet::VoteNonceOf<Test>,
        voting_engine::pallet::VoteSignatureOf<Test>,
    > for TestPopulationSnapshotVerifier
{
    fn verify_population_snapshot(
        _who: &AccountId32,
        _eligible_total: u64,
        _nonce: &voting_engine::pallet::VoteNonceOf<Test>,
        _signature: &voting_engine::pallet::VoteSignatureOf<Test>,
        _province: &[u8],
        _signer_admin_pubkey: &[u8; 32],
    ) -> bool {
        true
    }
}

pub struct TestTimeProvider;
impl frame_support::traits::UnixTime for TestTimeProvider {
    fn now() -> core::time::Duration {
        core::time::Duration::from_secs(1_782_864_000)
    }
}

pub struct TestInternalThresholdProvider;
impl voting_engine::InternalThresholdProvider for TestInternalThresholdProvider {
    fn pass_threshold(org: u8, _institution: voting_engine::InstitutionPalletId) -> Option<u32> {
        voting_engine::internal_vote::fixed_governance_pass_threshold(org)
    }
}

impl voting_engine::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MaxVoteNonceLength = ConstU32<64>;
    type MaxVoteSignatureLength = ConstU32<64>;
    type MaxAutoFinalizePerBlock = ConstU32<64>;
    type MaxProposalsPerExpiry = ConstU32<128>;
    type MaxInternalProposalMutexBindings = ConstU32<256>;
    type MaxActiveProposals = ConstU32<10>;
    type MaxCleanupStepsPerBlock = ConstU32<8>;
    type CleanupKeysPerStep = ConstU32<64>;
    type MaxProposalDataLen = ConstU32<8192>;
    type MaxProposalObjectLen = ConstU32<{ 10 * 1024 }>;
    type MaxModuleTagLen = ConstU32<32>;
    type MaxManualExecutionAttempts = ConstU32<3>;
    type ExecutionRetryGraceBlocks = frame_support::traits::ConstU64<216>;
    type MaxExecutionRetryDeadlinesPerBlock = ConstU32<128>;
    type SfidEligibility = TestSfidEligibility;
    type PopulationSnapshotVerifier = TestPopulationSnapshotVerifier;
    type JointVoteResultCallback = ();
    type InternalVoteResultCallback = ();
    type InternalAdminProvider = ();
    type InternalThresholdProvider = TestInternalThresholdProvider;
    type InternalAdminCountProvider = ();
    type MaxAdminsPerInstitution = ConstU32<32>;
    type TimeProvider = TestTimeProvider;
    type WeightInfo = ();
}

impl pallet::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type ProposeOrigin = EnsureNrcAdminForTest;
    type RecipientSetOrigin = frame_system::EnsureRoot<AccountId32>;
    type MaintenanceOrigin = frame_system::EnsureRoot<AccountId32>;
    type JointVoteEngine = TestJointVoteEngine;
    type MaxReasonLen = ConstU32<128>;
    type MaxAllocations = ConstU32<64>;
    type MaxSnapshotNonceLength = ConstU32<64>;
    type MaxSnapshotSignatureLength = ConstU32<64>;
    type MaxTotalIssuance = ConstU128<14_434_973_780_000>;
    type MaxSingleIssuance = ConstU128<14_434_973_780_000>;
    type WeightInfo = ();
}

fn new_test_ext() -> sp_io::TestExternalities {
    let storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("test storage should build");
    let mut ext: sp_io::TestExternalities = storage.into();
    ext.execute_with(|| {
        System::set_block_number(1);
        NEXT_JOINT_ID.with(|id| *id.borrow_mut() = 100);
        let recipients = reserve_council_accounts();
        let bounded: BoundedVec<AccountId32, ConstU32<64>> =
            recipients.try_into().expect("recipients should fit");
        pallet::AllowedRecipients::<Test>::put(bounded);
    });
    ext
}

fn insert_engine_proposal(proposal_id: u64) {
    insert_engine_proposal_with_status(proposal_id, voting_engine::STATUS_PASSED);
}

fn insert_engine_proposal_with_status(proposal_id: u64, status: u8) {
    voting_engine::pallet::Proposals::<Test>::insert(
        proposal_id,
        voting_engine::Proposal {
            kind: voting_engine::PROPOSAL_KIND_JOINT,
            stage: voting_engine::STAGE_JOINT,
            status,
            internal_org: None,
            internal_institution: None,
            start: 0u64,
            end: 100u64,
            citizen_eligible_total: 10,
        },
    );
}

fn overwrite_proposal_data(
    proposal_id: u64,
    data: crate::proposal::IssuanceProposalData<AccountId32, Balance>,
) {
    let mut encoded = Vec::from(crate::MODULE_TAG);
    encoded.extend_from_slice(&codec::Encode::encode(&data));
    let bounded_data: BoundedVec<u8, <Test as voting_engine::Config>::MaxProposalDataLen> =
        encoded.try_into().expect("proposal data should fit");
    let owner: BoundedVec<u8, <Test as voting_engine::Config>::MaxModuleTagLen> = crate::MODULE_TAG
        .to_vec()
        .try_into()
        .expect("module tag should fit");
    voting_engine::ProposalData::<Test>::insert(proposal_id, bounded_data);
    voting_engine::ProposalOwner::<Test>::insert(proposal_id, owner);
}

fn call_joint_callback(
    proposal_id: u64,
    approved: bool,
) -> Result<voting_engine::ProposalExecutionOutcome, DispatchError> {
    voting_engine::pallet::CallbackExecutionScopes::<Test>::insert(proposal_id, ());
    let result = ResolutionIssuance::on_joint_vote_finalized(proposal_id, approved);
    voting_engine::pallet::CallbackExecutionScopes::<Test>::remove(proposal_id);
    match result {
        Ok(outcome) => {
            if approved {
                voting_engine::pallet::Proposals::<Test>::mutate(proposal_id, |maybe| {
                    if let Some(proposal) = maybe {
                        proposal.status = match outcome {
                            voting_engine::ProposalExecutionOutcome::Executed => {
                                voting_engine::STATUS_EXECUTED
                            }
                            voting_engine::ProposalExecutionOutcome::FatalFailed => {
                                voting_engine::STATUS_EXECUTION_FAILED
                            }
                            _ => proposal.status,
                        };
                    }
                });
            }
            Ok(outcome)
        }
        Err(err) => Err(err),
    }
}

fn reason_ok() -> pallet::ReasonOf<Test> {
    b"issuance".to_vec().try_into().expect("reason should fit")
}

fn nonce_ok() -> pallet::SnapshotNonceOf<Test> {
    b"snap-nonce".to_vec().try_into().expect("nonce should fit")
}

fn sig_ok() -> pallet::SnapshotSignatureOf<Test> {
    b"snap-signature"
        .to_vec()
        .try_into()
        .expect("signature should fit")
}

/// ADR-008 step3:测试用占位 province + signer_admin_pubkey,
/// 仅在 `TestPopulationSnapshotVerifier` / `TestJointVoteEngine` 内做空字段非空检验,
/// 不参与真实 sr25519 验签(真实验签覆盖留 runtime 层测试)。
fn province_ok() -> frame_support::BoundedVec<u8, frame_support::pallet_prelude::ConstU32<64>> {
    b"liaoning".to_vec().try_into().expect("province should fit")
}

fn signer_admin_pubkey_ok() -> [u8; 32] {
    [7u8; 32]
}

fn reserve_council_accounts() -> Vec<AccountId32> {
    primitives::china::china_cb::CHINA_CB
        .iter()
        .skip(1)
        .map(|n| AccountId32::new(n.main_address))
        .collect()
}

fn allocations_ok(total: Balance) -> pallet::AllocationOf<Test> {
    let recipients = reserve_council_accounts();
    let count = recipients.len() as u128;
    let per = total / count;
    let mut left = total;
    let mut v = Vec::new();
    for (i, recipient) in recipients.into_iter().enumerate() {
        let amount = if i + 1 == count as usize { left } else { per };
        left = left.saturating_sub(amount);
        v.push(crate::proposal::RecipientAmount { recipient, amount });
    }
    v.try_into().expect("allocations should fit")
}

#[test]
fn only_authorized_admin_can_propose() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            ResolutionIssuance::propose_resolution_issuance(
                RuntimeOrigin::signed(AccountId32::new([2u8; 32])),
                reason_ok(),
                4300,
                allocations_ok(4300),
                10,
                nonce_ok(),
                sig_ok(),
                province_ok(),
                signer_admin_pubkey_ok()
            ),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn reject_invalid_allocation_count() {
    new_test_ext().execute_with(|| {
        let one = vec![crate::proposal::RecipientAmount {
            recipient: reserve_council_accounts()[0].clone(),
            amount: 1000,
        }];
        let alloc: pallet::AllocationOf<Test> = one.try_into().expect("should fit");
        assert_noop!(
            ResolutionIssuance::propose_resolution_issuance(
                RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
                reason_ok(),
                1000,
                alloc,
                10,
                nonce_ok(),
                sig_ok(),
                province_ok(),
                signer_admin_pubkey_ok()
            ),
            pallet::Error::<Test>::InvalidAllocationCount
        );
    });
}

#[test]
fn approved_callback_executes_issuance() {
    new_test_ext().execute_with(|| {
        assert_ok!(ResolutionIssuance::propose_resolution_issuance(
            RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
            reason_ok(),
            4300,
            allocations_ok(4300),
            10,
            nonce_ok(),
            sig_ok(),
            province_ok(),
            signer_admin_pubkey_ok()
        ));

        insert_engine_proposal(100);
        assert_ok!(call_joint_callback(100, true));
        assert_eq!(
            voting_engine::pallet::Proposals::<Test>::get(100)
                .expect("engine proposal should exist")
                .status,
            voting_engine::STATUS_EXECUTED
        );
        assert_eq!(pallet::VotingProposalCount::<Test>::get(), 0);
        assert!(pallet::Executed::<Test>::get(100).is_some());
        assert!(pallet::EverExecuted::<Test>::contains_key(100));
        assert_eq!(pallet::TotalIssued::<Test>::get(), 4300);
    });
}

#[test]
fn callback_rejects_non_finalizable_engine_status() {
    new_test_ext().execute_with(|| {
        assert_ok!(ResolutionIssuance::propose_resolution_issuance(
            RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
            reason_ok(),
            4300,
            allocations_ok(4300),
            10,
            nonce_ok(),
            sig_ok(),
            province_ok(),
            signer_admin_pubkey_ok()
        ));

        insert_engine_proposal_with_status(100, voting_engine::STATUS_VOTING);
        assert_noop!(
            call_joint_callback(100, true),
            pallet::Error::<Test>::ProposalNotFinalizable
        );
        assert_eq!(pallet::VotingProposalCount::<Test>::get(), 1);
        assert!(!pallet::Executed::<Test>::contains_key(100));
        assert_eq!(pallet::TotalIssued::<Test>::get(), 0);
    });
}

#[test]
fn callback_requires_voting_engine_scope() {
    new_test_ext().execute_with(|| {
        assert_ok!(ResolutionIssuance::propose_resolution_issuance(
            RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
            reason_ok(),
            4300,
            allocations_ok(4300),
            10,
            nonce_ok(),
            sig_ok(),
            province_ok(),
            signer_admin_pubkey_ok()
        ));

        insert_engine_proposal(100);
        assert_noop!(
            ResolutionIssuance::on_joint_vote_finalized(100, true),
            pallet::Error::<Test>::ProposalNotFinalizable
        );
        assert_eq!(pallet::VotingProposalCount::<Test>::get(), 1);
        assert!(!pallet::Executed::<Test>::contains_key(100));
    });
}

#[test]
fn second_callback_after_executed_is_rejected() {
    new_test_ext().execute_with(|| {
        assert_ok!(ResolutionIssuance::propose_resolution_issuance(
            RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
            reason_ok(),
            4300,
            allocations_ok(4300),
            10,
            nonce_ok(),
            sig_ok(),
            province_ok(),
            signer_admin_pubkey_ok()
        ));

        insert_engine_proposal(100);
        assert_ok!(call_joint_callback(100, true));
        assert_noop!(
            call_joint_callback(100, true),
            pallet::Error::<Test>::ProposalNotFinalizable
        );
        assert_eq!(
            voting_engine::pallet::Proposals::<Test>::get(100)
                .expect("engine proposal should exist")
                .status,
            voting_engine::STATUS_EXECUTED
        );
        assert_eq!(pallet::VotingProposalCount::<Test>::get(), 0);
        assert_eq!(pallet::TotalIssued::<Test>::get(), 4300);
    });
}

#[test]
fn rejected_callback_does_not_issue() {
    new_test_ext().execute_with(|| {
        assert_ok!(ResolutionIssuance::propose_resolution_issuance(
            RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
            reason_ok(),
            4300,
            allocations_ok(4300),
            10,
            nonce_ok(),
            sig_ok(),
            province_ok(),
            signer_admin_pubkey_ok()
        ));

        insert_engine_proposal_with_status(100, voting_engine::STATUS_REJECTED);
        assert_ok!(call_joint_callback(100, false));
        assert_eq!(pallet::VotingProposalCount::<Test>::get(), 0);
        assert!(!pallet::Executed::<Test>::contains_key(100));
        assert_eq!(pallet::TotalIssued::<Test>::get(), 0);
    });
}

#[test]
fn callback_rejects_corrupted_reason_with_reason_too_long() {
    new_test_ext().execute_with(|| {
        assert_ok!(ResolutionIssuance::propose_resolution_issuance(
            RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
            reason_ok(),
            4300,
            allocations_ok(4300),
            10,
            nonce_ok(),
            sig_ok(),
            province_ok(),
            signer_admin_pubkey_ok()
        ));

        overwrite_proposal_data(
            100,
            crate::proposal::IssuanceProposalData {
                proposer: AccountId32::new([1u8; 32]),
                reason: vec![b'x'; 129],
                total_amount: 4300,
                allocations: allocations_ok(4300).to_vec(),
            },
        );
        insert_engine_proposal(100);
        assert_noop!(
            call_joint_callback(100, true),
            pallet::Error::<Test>::ReasonTooLong
        );
        assert_eq!(pallet::VotingProposalCount::<Test>::get(), 1);
        assert!(!pallet::Executed::<Test>::contains_key(100));
        assert_eq!(pallet::TotalIssued::<Test>::get(), 0);
    });
}

#[test]
fn clear_executed_does_not_allow_replay() {
    new_test_ext().execute_with(|| {
        assert_ok!(ResolutionIssuance::propose_resolution_issuance(
            RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
            reason_ok(),
            4300,
            allocations_ok(4300),
            10,
            nonce_ok(),
            sig_ok(),
            province_ok(),
            signer_admin_pubkey_ok()
        ));
        insert_engine_proposal(100);
        assert_ok!(call_joint_callback(100, true));
        assert_ok!(ResolutionIssuance::clear_executed(
            RuntimeOrigin::root(),
            100
        ));
        assert!(!pallet::Executed::<Test>::contains_key(100));
        assert!(pallet::EverExecuted::<Test>::contains_key(100));

        assert_noop!(
            pallet::Pallet::<Test>::execute_approved_issuance(
                100,
                &reason_ok(),
                4300,
                &allocations_ok(4300)
            ),
            pallet::Error::<Test>::AlreadyExecuted
        );
    });
}

#[test]
fn pause_blocks_approved_execution() {
    new_test_ext().execute_with(|| {
        assert_ok!(ResolutionIssuance::propose_resolution_issuance(
            RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
            reason_ok(),
            4300,
            allocations_ok(4300),
            10,
            nonce_ok(),
            sig_ok(),
            province_ok(),
            signer_admin_pubkey_ok()
        ));
        assert_ok!(ResolutionIssuance::set_paused(RuntimeOrigin::root(), true));
        insert_engine_proposal(100);
        assert_ok!(call_joint_callback(100, true));
        assert_eq!(
            voting_engine::pallet::Proposals::<Test>::get(100)
                .expect("engine proposal should exist")
                .status,
            voting_engine::STATUS_EXECUTION_FAILED
        );
        assert_eq!(pallet::VotingProposalCount::<Test>::get(), 0);
        assert!(!pallet::Executed::<Test>::contains_key(100));
        assert_eq!(pallet::TotalIssued::<Test>::get(), 0);
    });
}

#[test]
fn set_allowed_recipients_rejected_when_voting_exists() {
    new_test_ext().execute_with(|| {
        assert_ok!(ResolutionIssuance::propose_resolution_issuance(
            RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
            reason_ok(),
            4300,
            allocations_ok(4300),
            10,
            nonce_ok(),
            sig_ok(),
            province_ok(),
            signer_admin_pubkey_ok()
        ));
        let recipients: BoundedVec<AccountId32, ConstU32<64>> = reserve_council_accounts()
            .try_into()
            .expect("recipients should fit");
        assert_noop!(
            ResolutionIssuance::set_allowed_recipients(RuntimeOrigin::root(), recipients),
            pallet::Error::<Test>::ActiveVotingProposalsExist
        );
    });
}

#[test]
fn issuance_event_comes_from_unified_pallet() {
    new_test_ext().execute_with(|| {
        assert_ok!(ResolutionIssuance::propose_resolution_issuance(
            RuntimeOrigin::signed(AccountId32::new([1u8; 32])),
            reason_ok(),
            4300,
            allocations_ok(4300),
            10,
            nonce_ok(),
            sig_ok(),
            province_ok(),
            signer_admin_pubkey_ok()
        ));
        insert_engine_proposal(100);
        assert_ok!(call_joint_callback(100, true));

        assert!(frame_system::Pallet::<Test>::events().iter().any(|record| {
            matches!(
                &record.event,
                RuntimeEvent::ResolutionIssuance(
                    pallet::Event::<Test>::ResolutionIssuanceExecuted {
                        proposal_id: 100,
                        ..
                    }
                )
            )
        }));
    });
}

#[test]
fn clear_executed_requires_existing_key() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            ResolutionIssuance::clear_executed(RuntimeOrigin::root(), 99),
            pallet::Error::<Test>::NotExecuted
        );
    });
}

#[test]
fn set_paused_same_state_is_rejected() {
    new_test_ext().execute_with(|| {
        assert_ok!(ResolutionIssuance::set_paused(RuntimeOrigin::root(), true));
        assert_noop!(
            ResolutionIssuance::set_paused(RuntimeOrigin::root(), true),
            pallet::Error::<Test>::AlreadyInState
        );
    });
}
