#![cfg_attr(not(feature = "std"), no_std)]

pub mod citizen_vote;
pub mod internal_vote;
pub mod joint_vote;

pub use citizen_vote::CiicEligibility;
pub use pallet::*;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::dispatch::DispatchResult;
use scale_info::TypeInfo;
use sp_runtime::DispatchError;

pub type InstitutionPalletId = [u8; 8];

pub const PROPOSAL_KIND_INTERNAL: u8 = 0;
pub const PROPOSAL_KIND_JOINT: u8 = 1;

pub const STAGE_INTERNAL: u8 = 0;
pub const STAGE_JOINT: u8 = 1;
pub const STAGE_CITIZEN: u8 = 2;

pub const STATUS_VOTING: u8 = 0;
pub const STATUS_PASSED: u8 = 1;
pub const STATUS_REJECTED: u8 = 2;

/// 中文注释：事项模块接入联合投票时，统一由投票引擎创建提案并写入人口快照。
pub trait JointVoteEngine<AccountId> {
    fn create_joint_proposal(
        who: AccountId,
        eligible_total: u64,
        snapshot_nonce: &[u8],
        snapshot_signature: &[u8],
    ) -> Result<u64, DispatchError>;
}

/// 中文注释：事项模块接入内部投票时，统一由投票引擎创建提案并返回真实提案ID。
pub trait InternalVoteEngine<AccountId> {
    fn create_internal_proposal(
        who: AccountId,
        org: u8,
        institution: InstitutionPalletId,
    ) -> Result<u64, DispatchError>;
}

impl<AccountId> InternalVoteEngine<AccountId> for () {
    fn create_internal_proposal(
        _who: AccountId,
        _org: u8,
        _institution: InstitutionPalletId,
    ) -> Result<u64, DispatchError> {
        Err(DispatchError::Other("InternalVoteEngineNotConfigured"))
    }
}

/// 中文注释：公民总人口快照验签接口（由 runtime 对接 CIIC 系统）。
pub trait PopulationSnapshotVerifier<AccountId, Nonce, Signature> {
    fn verify_population_snapshot(
        who: &AccountId,
        eligible_total: u64,
        nonce: &Nonce,
        signature: &Signature,
    ) -> bool;
}

impl<AccountId, Nonce, Signature> PopulationSnapshotVerifier<AccountId, Nonce, Signature> for () {
    fn verify_population_snapshot(
        _who: &AccountId,
        _eligible_total: u64,
        _nonce: &Nonce,
        _signature: &Signature,
    ) -> bool {
        false
    }
}

pub trait JointVoteResultCallback {
    fn on_joint_vote_finalized(vote_proposal_id: u64, approved: bool) -> DispatchResult;
}

impl JointVoteResultCallback for () {
    fn on_joint_vote_finalized(_vote_proposal_id: u64, _approved: bool) -> DispatchResult {
        Ok(())
    }
}

/// 中文注释：内部管理员动态提供器（可由其他治理模块提供最新管理员集合）。
pub trait InternalAdminProvider<AccountId> {
    fn is_internal_admin(org: u8, institution: InstitutionPalletId, who: &AccountId) -> bool;
}

impl<AccountId> InternalAdminProvider<AccountId> for () {
    fn is_internal_admin(_org: u8, _institution: InstitutionPalletId, _who: &AccountId) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct Proposal<BlockNumber> {
    /// 提案类型：内部投票/联合投票
    pub kind: u8,
    /// 当前所处投票阶段：内部/联合/公民
    pub stage: u8,
    /// 当前提案状态：投票中/通过/否决
    pub status: u8,
    /// 仅内部投票使用：机构类型（国储会/省储会/省储行）
    pub internal_org: Option<u8>,
    /// 仅内部投票使用：机构 pallet_id（全链唯一）
    pub internal_institution: Option<InstitutionPalletId>,
    /// 本阶段起始区块
    pub start: BlockNumber,
    /// 本阶段截止区块（超过则超时）
    pub end: BlockNumber,
    /// 公民投票阶段的可投票总人数（由外部资格系统给出）
    pub citizen_eligible_total: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct VoteCountU32 {
    /// 赞成票
    pub yes: u32,
    /// 反对票
    pub no: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct VoteCountU64 {
    /// 赞成票
    pub yes: u64,
    /// 反对票
    pub no: u64,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{pallet_prelude::*, Blake2_128Concat};
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        #[pallet::constant]
        type MaxCiicLength: Get<u32>;

        #[pallet::constant]
        type MaxVoteNonceLength: Get<u32>;

        #[pallet::constant]
        type MaxVoteSignatureLength: Get<u32>;

        type CiicEligibility: CiicEligibility<Self::AccountId>;
        type PopulationSnapshotVerifier: PopulationSnapshotVerifier<
            Self::AccountId,
            VoteNonceOf<Self>,
            VoteSignatureOf<Self>,
        >;

        type JointVoteResultCallback: JointVoteResultCallback;
        type InternalAdminProvider: InternalAdminProvider<Self::AccountId>;
    }

    pub type CiicOf<T> = BoundedVec<u8, <T as Config>::MaxCiicLength>;
    pub type VoteNonceOf<T> = BoundedVec<u8, <T as Config>::MaxVoteNonceLength>;
    pub type VoteSignatureOf<T> = BoundedVec<u8, <T as Config>::MaxVoteSignatureLength>;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn next_proposal_id)]
    pub type NextProposalId<T> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn proposals)]
    pub type Proposals<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, Proposal<BlockNumberFor<T>>, OptionQuery>;

    #[pallet::storage]
    pub type InternalVotesByAccount<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        T::AccountId,
        bool,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn internal_tally)]
    pub type InternalTallies<T> = StorageMap<_, Blake2_128Concat, u64, VoteCountU32, ValueQuery>;

    #[pallet::storage]
    pub type JointVotesByInstitution<T> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        InstitutionPalletId,
        bool,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn joint_tally)]
    pub type JointTallies<T> = StorageMap<_, Blake2_128Concat, u64, VoteCountU32, ValueQuery>;

    #[pallet::storage]
    pub type CitizenVotesByCiic<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, u64, Blake2_128Concat, T::Hash, bool, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn citizen_tally)]
    pub type CitizenTallies<T> = StorageMap<_, Blake2_128Concat, u64, VoteCountU64, ValueQuery>;

    /// 中文注释：总人口快照 nonce 防重放（全局维度，防止跨提案重放）。
    #[pallet::storage]
    #[pallet::getter(fn used_population_snapshot_nonce)]
    pub type UsedPopulationSnapshotNonce<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, bool, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ProposalCreated {
            proposal_id: u64,
            kind: u8,
            stage: u8,
            end: BlockNumberFor<T>,
        },
        ProposalAdvancedToCitizen {
            proposal_id: u64,
            citizen_end: BlockNumberFor<T>,
            eligible_total: u64,
        },
        ProposalFinalized {
            proposal_id: u64,
            status: u8,
        },
        InternalVoteCast {
            proposal_id: u64,
            who: T::AccountId,
            approve: bool,
        },
        JointInstitutionVoteCast {
            proposal_id: u64,
            institution: InstitutionPalletId,
            internal_passed: bool,
        },
        CitizenVoteCast {
            proposal_id: u64,
            who: T::AccountId,
            ciic_hash: T::Hash,
            approve: bool,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        ProposalNotFound,
        InvalidProposalKind,
        InvalidProposalStage,
        InvalidProposalStatus,
        InvalidInternalOrg,
        InvalidInstitution,
        NoPermission,
        VoteClosed,
        VoteNotExpired,
        AlreadyVoted,
        CiicNotEligible,
        InvalidCiicVoteCredential,
        EmptyCiic,
        CitizenEligibleTotalNotSet,
        InvalidPopulationSnapshot,
        ProposalAlreadyFinalized,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(T::DbWeight::get().reads_writes(2, 2))]
        pub fn create_internal_proposal(
            origin: OriginFor<T>,
            org: u8,
            institution: InstitutionPalletId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_create_internal_proposal(who, org, institution)?;
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(T::DbWeight::get().reads_writes(0, 0))]
        pub fn create_joint_proposal(
            origin: OriginFor<T>,
            _eligible_total: u64,
            _snapshot_nonce: VoteNonceOf<T>,
            _snapshot_signature: VoteSignatureOf<T>,
        ) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            // 中文注释：联合投票提案只能由事项模块通过 JointVoteEngine trait 创建；
            // 禁止外部直接调用，避免产生“无事项映射”的悬空联合提案。
            Err(Error::<T>::NoPermission.into())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(T::DbWeight::get().reads_writes(4, 4))]
        pub fn internal_vote(
            origin: OriginFor<T>,
            proposal_id: u64,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_internal_vote(who, proposal_id, approve)
        }

        #[pallet::call_index(3)]
        #[pallet::weight(T::DbWeight::get().reads_writes(5, 5))]
        pub fn submit_joint_institution_vote(
            origin: OriginFor<T>,
            proposal_id: u64,
            institution: InstitutionPalletId,
            internal_passed: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_submit_joint_institution_vote(who, proposal_id, institution, internal_passed)
        }

        #[pallet::call_index(4)]
        #[pallet::weight(T::DbWeight::get().reads_writes(4, 4))]
        pub fn citizen_vote(
            origin: OriginFor<T>,
            proposal_id: u64,
            ciic: CiicOf<T>,
            nonce: VoteNonceOf<T>,
            signature: VoteSignatureOf<T>,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_citizen_vote(who, proposal_id, ciic, nonce, signature, approve)
        }

        #[pallet::call_index(5)]
        #[pallet::weight(T::DbWeight::get().reads_writes(2, 2))]
        pub fn finalize_proposal(origin: OriginFor<T>, proposal_id: u64) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            let proposal = Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;

            match proposal.stage {
                STAGE_INTERNAL => {
                    Self::do_finalize_internal_timeout(proposal_id)?;
                }
                STAGE_JOINT => {
                    Self::do_finalize_joint_timeout(proposal_id)?;
                }
                STAGE_CITIZEN => {
                    Self::do_finalize_citizen_timeout(proposal_id)?;
                }
                _ => return Err(Error::<T>::InvalidProposalStage.into()),
            }

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        pub(crate) fn allocate_proposal_id() -> u64 {
            let id = NextProposalId::<T>::get();
            NextProposalId::<T>::put(id.saturating_add(1));
            id
        }

        pub(crate) fn ensure_open_proposal(
            proposal_id: u64,
        ) -> Result<Proposal<BlockNumberFor<T>>, DispatchError> {
            let proposal = Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;

            ensure!(
                proposal.status == STATUS_VOTING,
                Error::<T>::InvalidProposalStatus
            );
            ensure!(
                <frame_system::Pallet<T>>::block_number() <= proposal.end,
                Error::<T>::VoteClosed
            );

            Ok(proposal)
        }

        pub(crate) fn set_status_and_emit(proposal_id: u64, status: u8) -> DispatchResult {
            let proposal_before =
                Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;
            Proposals::<T>::try_mutate(proposal_id, |maybe| -> DispatchResult {
                let proposal = maybe.as_mut().ok_or(Error::<T>::ProposalNotFound)?;
                proposal.status = status;
                Ok(())
            })?;

            Self::deposit_event(Event::<T>::ProposalFinalized {
                proposal_id,
                status,
            });

            if proposal_before.kind == PROPOSAL_KIND_JOINT && status != STATUS_VOTING {
                T::JointVoteResultCallback::on_joint_vote_finalized(
                    proposal_id,
                    status == STATUS_PASSED,
                )?;
            }
            Ok(())
        }
    }
}

impl<T: pallet::Config> JointVoteEngine<T::AccountId> for pallet::Pallet<T> {
    fn create_joint_proposal(
        who: T::AccountId,
        eligible_total: u64,
        snapshot_nonce: &[u8],
        snapshot_signature: &[u8],
    ) -> Result<u64, DispatchError> {
        let snapshot_nonce: pallet::VoteNonceOf<T> = snapshot_nonce
            .to_vec()
            .try_into()
            .map_err(|_| pallet::Error::<T>::InvalidPopulationSnapshot)?;
        let snapshot_signature: pallet::VoteSignatureOf<T> = snapshot_signature
            .to_vec()
            .try_into()
            .map_err(|_| pallet::Error::<T>::InvalidPopulationSnapshot)?;
        pallet::Pallet::<T>::do_create_joint_proposal(
            who,
            eligible_total,
            snapshot_nonce,
            snapshot_signature,
        )
    }
}

impl<T: pallet::Config> InternalVoteEngine<T::AccountId> for pallet::Pallet<T> {
    fn create_internal_proposal(
        who: T::AccountId,
        org: u8,
        institution: InstitutionPalletId,
    ) -> Result<u64, DispatchError> {
        pallet::Pallet::<T>::do_create_internal_proposal(who, org, institution)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::RefCell;
    use std::collections::BTreeSet;

    use frame_support::{assert_noop, assert_ok, derive_impl, traits::ConstU32};
    use frame_system as system;
    use primitives::reserve_nodes_const::{
        pallet_id_to_bytes as reserve_pallet_id_to_bytes, RESERVE_NODES,
    };
    use primitives::shengbank_nodes_const::{
        pallet_id_to_bytes as shengbank_pallet_id_to_bytes, SHENG_BANK_NODES,
    };
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
        pub type VotingEngineSystem = super;
    }

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl system::Config for Test {
        type Block = Block;
        type AccountId = AccountId32;
        type Lookup = IdentityLookup<Self::AccountId>;
    }

    impl Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type MaxCiicLength = ConstU32<64>;
        type MaxVoteNonceLength = ConstU32<64>;
        type MaxVoteSignatureLength = ConstU32<64>;
        type CiicEligibility = TestCiicEligibility;
        type PopulationSnapshotVerifier = TestPopulationSnapshotVerifier;
        type JointVoteResultCallback = ();
        type InternalAdminProvider = ();
    }

    thread_local! {
        static USED_VOTE_NONCES: RefCell<BTreeSet<(u64, Vec<u8>, Vec<u8>)>> = RefCell::new(BTreeSet::new());
    }

    pub struct TestCiicEligibility;
    pub struct TestPopulationSnapshotVerifier;

    impl
        PopulationSnapshotVerifier<
            AccountId32,
            pallet::VoteNonceOf<Test>,
            pallet::VoteSignatureOf<Test>,
        > for TestPopulationSnapshotVerifier
    {
        fn verify_population_snapshot(
            _who: &AccountId32,
            eligible_total: u64,
            nonce: &pallet::VoteNonceOf<Test>,
            signature: &pallet::VoteSignatureOf<Test>,
        ) -> bool {
            eligible_total > 0 && !nonce.is_empty() && signature.as_slice() == b"snapshot-ok"
        }
    }

    impl CiicEligibility<AccountId32> for TestCiicEligibility {
        fn is_eligible(ciic: &[u8], who: &AccountId32) -> bool {
            ciic == b"ciic-ok" && who == &nrc_admin(0)
        }

        fn verify_and_consume_vote_credential(
            ciic: &[u8],
            who: &AccountId32,
            proposal_id: u64,
            nonce: &[u8],
            signature: &[u8],
        ) -> bool {
            if !Self::is_eligible(ciic, who) || signature != b"vote-ok" || nonce.is_empty() {
                return false;
            }
            let key = (proposal_id, ciic.to_vec(), nonce.to_vec());
            USED_VOTE_NONCES.with(|set| {
                let mut set = set.borrow_mut();
                if set.contains(&key) {
                    false
                } else {
                    set.insert(key);
                    true
                }
            })
        }
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("frame system genesis storage should build");
        let mut ext = sp_io::TestExternalities::new(storage);
        ext.execute_with(|| {
            USED_VOTE_NONCES.with(|set| set.borrow_mut().clear());
            System::set_block_number(1);
        });
        ext
    }

    fn nrc_pid() -> InstitutionPalletId {
        reserve_pallet_id_to_bytes(RESERVE_NODES[0].pallet_id).expect("nrc id should be 8 bytes")
    }

    fn prc_pid() -> InstitutionPalletId {
        reserve_pallet_id_to_bytes(RESERVE_NODES[1].pallet_id).expect("prc id should be 8 bytes")
    }

    fn prb_pid() -> InstitutionPalletId {
        shengbank_pallet_id_to_bytes(SHENG_BANK_NODES[0].pallet_id)
            .expect("prb id should be 8 bytes")
    }

    fn nrc_admin(index: usize) -> AccountId32 {
        AccountId32::new(RESERVE_NODES[0].admins[index])
    }

    fn nrc_multisig() -> AccountId32 {
        AccountId32::new(RESERVE_NODES[0].pallet_address)
    }

    fn prc_admin(index: usize) -> AccountId32 {
        AccountId32::new(RESERVE_NODES[1].admins[index])
    }

    fn prb_admin(index: usize) -> AccountId32 {
        AccountId32::new(SHENG_BANK_NODES[0].admins[index])
    }

    fn ciic_ok() -> pallet::CiicOf<Test> {
        b"ciic-ok".to_vec().try_into().expect("ciic should fit")
    }

    fn vote_nonce(input: &str) -> pallet::VoteNonceOf<Test> {
        input
            .as_bytes()
            .to_vec()
            .try_into()
            .expect("nonce should fit")
    }

    fn vote_sig_ok() -> pallet::VoteSignatureOf<Test> {
        b"vote-ok"
            .to_vec()
            .try_into()
            .expect("signature should fit")
    }

    fn vote_sig_bad() -> pallet::VoteSignatureOf<Test> {
        b"bad".to_vec().try_into().expect("signature should fit")
    }

    fn snapshot_nonce_ok() -> pallet::VoteNonceOf<Test> {
        b"snap-nonce"
            .to_vec()
            .try_into()
            .expect("snapshot nonce should fit")
    }

    fn snapshot_sig_ok() -> pallet::VoteSignatureOf<Test> {
        b"snapshot-ok"
            .to_vec()
            .try_into()
            .expect("snapshot signature should fit")
    }

    fn insert_citizen_proposal(proposal_id: u64, eligible_total: u64, end: u64) {
        Proposals::<Test>::insert(
            proposal_id,
            Proposal {
                kind: PROPOSAL_KIND_JOINT,
                stage: STAGE_CITIZEN,
                status: STATUS_VOTING,
                internal_org: None,
                internal_institution: None,
                start: System::block_number(),
                end,
                citizen_eligible_total: eligible_total,
            },
        );
    }

    #[test]
    fn internal_proposal_must_be_created_by_same_institution_admin() {
        new_test_ext().execute_with(|| {
            let outsider = AccountId32::new([7u8; 32]);

            assert_noop!(
                VotingEngineSystem::create_internal_proposal(
                    RuntimeOrigin::signed(outsider),
                    internal_vote::ORG_NRC,
                    nrc_pid(),
                ),
                pallet::Error::<Test>::InvalidInstitution
            );

            assert_noop!(
                VotingEngineSystem::create_internal_proposal(
                    RuntimeOrigin::signed(prc_admin(0)),
                    internal_vote::ORG_NRC,
                    nrc_pid(),
                ),
                pallet::Error::<Test>::InvalidInstitution
            );

            assert_ok!(VotingEngineSystem::create_internal_proposal(
                RuntimeOrigin::signed(nrc_admin(0)),
                internal_vote::ORG_NRC,
                nrc_pid(),
            ));
        });
    }

    #[test]
    fn internal_vote_must_be_by_same_institution_admin() {
        new_test_ext().execute_with(|| {
            assert_ok!(VotingEngineSystem::create_internal_proposal(
                RuntimeOrigin::signed(prb_admin(0)),
                internal_vote::ORG_PRB,
                prb_pid(),
            ));

            assert_noop!(
                VotingEngineSystem::internal_vote(RuntimeOrigin::signed(nrc_admin(0)), 0, true,),
                pallet::Error::<Test>::InvalidInstitution
            );

            assert_ok!(VotingEngineSystem::internal_vote(
                RuntimeOrigin::signed(prb_admin(1)),
                0,
                true,
            ));
        });
    }

    #[test]
    fn nrc_internal_vote_passes_at_13_yes_votes() {
        new_test_ext().execute_with(|| {
            assert_ok!(VotingEngineSystem::create_internal_proposal(
                RuntimeOrigin::signed(nrc_admin(0)),
                internal_vote::ORG_NRC,
                nrc_pid(),
            ));

            for i in 0..12 {
                assert_ok!(VotingEngineSystem::internal_vote(
                    RuntimeOrigin::signed(nrc_admin(i)),
                    0,
                    true,
                ));
            }
            assert_eq!(
                VotingEngineSystem::proposals(0)
                    .expect("proposal exists")
                    .status,
                STATUS_VOTING
            );

            assert_ok!(VotingEngineSystem::internal_vote(
                RuntimeOrigin::signed(nrc_admin(12)),
                0,
                true,
            ));
            assert_eq!(
                VotingEngineSystem::proposals(0)
                    .expect("proposal exists")
                    .status,
                STATUS_PASSED
            );
        });
    }

    #[test]
    fn internal_vote_is_rejected_after_timeout() {
        new_test_ext().execute_with(|| {
            assert_ok!(VotingEngineSystem::create_internal_proposal(
                RuntimeOrigin::signed(prc_admin(0)),
                internal_vote::ORG_PRC,
                prc_pid(),
            ));

            let proposal = VotingEngineSystem::proposals(0).expect("proposal exists");
            System::set_block_number(proposal.end + 1);

            assert_ok!(VotingEngineSystem::finalize_proposal(
                RuntimeOrigin::signed(prc_admin(0)),
                0,
            ));
            assert_eq!(
                VotingEngineSystem::proposals(0)
                    .expect("proposal exists")
                    .status,
                STATUS_REJECTED
            );
        });
    }

    #[test]
    fn joint_proposal_must_be_created_by_nrc_admin() {
        new_test_ext().execute_with(|| {
            // 中文注释：外部 extrinsic 入口已禁用，统一要求事项模块通过 trait 创建联合投票提案。
            assert_noop!(
                VotingEngineSystem::create_joint_proposal(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    10,
                    snapshot_nonce_ok(),
                    snapshot_sig_ok()
                ),
                pallet::Error::<Test>::NoPermission
            );

            let outsider = AccountId32::new([9u8; 32]);
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            assert!(
                <VotingEngineSystem as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    outsider,
                    10,
                    nonce.as_slice(),
                    sig.as_slice()
                )
                .is_err()
            );

            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            assert!(
                <VotingEngineSystem as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    prc_admin(0),
                    10,
                    nonce.as_slice(),
                    sig.as_slice()
                )
                .is_err()
            );

            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            assert_ok!(
                <VotingEngineSystem as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    10,
                    nonce.as_slice(),
                    sig.as_slice()
                )
            );
        });
    }

    #[test]
    fn joint_vote_submission_must_be_by_nrc_multisig() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            assert_ok!(
                <VotingEngineSystem as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    10,
                    nonce.as_slice(),
                    sig.as_slice()
                )
            );

            assert_noop!(
                VotingEngineSystem::submit_joint_institution_vote(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    0,
                    nrc_pid(),
                    true
                ),
                pallet::Error::<Test>::NoPermission
            );

            assert_ok!(VotingEngineSystem::submit_joint_institution_vote(
                RuntimeOrigin::signed(nrc_multisig()),
                0,
                nrc_pid(),
                true
            ));
        });
    }

    #[test]
    fn population_snapshot_nonce_cannot_be_reused_across_proposals() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            assert_ok!(
                <VotingEngineSystem as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    10,
                    nonce.as_slice(),
                    sig.as_slice()
                )
            );

            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            assert!(
                <VotingEngineSystem as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    11,
                    nonce.as_slice(),
                    sig.as_slice()
                )
                .is_err()
            );
        });
    }

    #[test]
    fn citizen_vote_rejects_invalid_signature_and_allows_valid_vote() {
        new_test_ext().execute_with(|| {
            insert_citizen_proposal(0, 10, 100);

            assert_noop!(
                VotingEngineSystem::citizen_vote(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    0,
                    ciic_ok(),
                    vote_nonce("n-1"),
                    vote_sig_bad(),
                    true
                ),
                pallet::Error::<Test>::InvalidCiicVoteCredential
            );

            assert_ok!(VotingEngineSystem::citizen_vote(
                RuntimeOrigin::signed(nrc_admin(0)),
                0,
                ciic_ok(),
                vote_nonce("n-2"),
                vote_sig_ok(),
                true
            ));
            assert_eq!(CitizenTallies::<Test>::get(0).yes, 1);
        });
    }

    #[test]
    fn citizen_vote_same_ciic_can_only_vote_once_per_proposal() {
        new_test_ext().execute_with(|| {
            insert_citizen_proposal(0, 10, 100);

            assert_ok!(VotingEngineSystem::citizen_vote(
                RuntimeOrigin::signed(nrc_admin(0)),
                0,
                ciic_ok(),
                vote_nonce("n-1"),
                vote_sig_ok(),
                true
            ));

            assert_noop!(
                VotingEngineSystem::citizen_vote(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    0,
                    ciic_ok(),
                    vote_nonce("n-2"),
                    vote_sig_ok(),
                    false
                ),
                pallet::Error::<Test>::AlreadyVoted
            );
        });
    }

    #[test]
    fn citizen_vote_credential_nonce_is_replay_protected_per_proposal_and_ciic() {
        new_test_ext().execute_with(|| {
            insert_citizen_proposal(0, 10, 100);
            insert_citizen_proposal(1, 10, 100);

            assert_ok!(VotingEngineSystem::citizen_vote(
                RuntimeOrigin::signed(nrc_admin(0)),
                0,
                ciic_ok(),
                vote_nonce("same"),
                vote_sig_ok(),
                true
            ));

            assert_ok!(VotingEngineSystem::citizen_vote(
                RuntimeOrigin::signed(nrc_admin(0)),
                1,
                ciic_ok(),
                vote_nonce("same"),
                vote_sig_ok(),
                true
            ));
        });
    }

    #[test]
    fn citizen_vote_rejects_when_eligible_total_not_set_in_proposal() {
        new_test_ext().execute_with(|| {
            insert_citizen_proposal(0, 0, 100);

            assert_noop!(
                VotingEngineSystem::citizen_vote(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    0,
                    ciic_ok(),
                    vote_nonce("x-1"),
                    vote_sig_ok(),
                    true
                ),
                pallet::Error::<Test>::CitizenEligibleTotalNotSet
            );
        });
    }

    #[test]
    fn citizen_timeout_with_half_or_less_is_rejected() {
        new_test_ext().execute_with(|| {
            insert_citizen_proposal(0, 10, 5);
            CitizenTallies::<Test>::insert(0, VoteCountU64 { yes: 5, no: 0 });
            System::set_block_number(6);

            assert_ok!(VotingEngineSystem::finalize_proposal(
                RuntimeOrigin::signed(nrc_admin(0)),
                0
            ));
            assert_eq!(
                Proposals::<Test>::get(0)
                    .expect("proposal should exist")
                    .status,
                STATUS_REJECTED
            );
        });
    }
}
