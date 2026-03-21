#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use curve25519_dalek::edwards::CompressedEdwardsY;
use frame_support::{
    ensure,
    pallet_prelude::*,
    traits::{GetStorageVersion, StorageVersion},
    weights::Weight,
    Blake2_128Concat,
};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_consensus_grandpa::AuthorityId as GrandpaAuthorityId;
use sp_core::ed25519;
use primitives::china::china_cb::{shenfen_id_to_fixed48 as reserve_pallet_id_to_bytes, CHINA_CB};
use voting_engine_system::{
    internal_vote::{ORG_NRC, ORG_PRC},
    InstitutionPalletId, STATUS_EXECUTED, STATUS_PASSED, STATUS_REJECTED,
};

pub use pallet::*;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod weights;

const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

#[derive(
    Clone, Debug, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen,
)]
pub struct GrandpaKeyReplacementAction {
    pub institution: InstitutionPalletId,
    pub old_key: [u8; 32],
    pub new_key: [u8; 32],
}

fn nrc_pallet_id_bytes() -> Option<InstitutionPalletId> {
    CHINA_CB
        .first()
        .and_then(|n| reserve_pallet_id_to_bytes(n.shenfen_id))
}

fn institution_org(institution: InstitutionPalletId) -> Option<u8> {
    if Some(institution) == nrc_pallet_id_bytes() {
        return Some(ORG_NRC);
    }

    if CHINA_CB
        .iter()
        .skip(1)
        .filter_map(|n| reserve_pallet_id_to_bytes(n.shenfen_id))
        .any(|pid| pid == institution)
    {
        return Some(ORG_PRC);
    }

    None
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;
    use sp_runtime::DispatchError;
    use sp_std::vec::Vec;
    use voting_engine_system::{InternalAdminProvider, InternalVoteEngine};

    #[pallet::config]
    pub trait Config:
        frame_system::Config + voting_engine_system::Config + pallet_grandpa::Config
    {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        #[pallet::constant]
        type GrandpaChangeDelay: Get<BlockNumberFor<Self>>;

        /// 中文注释：内部投票引擎（返回真实 proposal_id，避免猜测 next_proposal_id）。
        type InternalVoteEngine: voting_engine_system::InternalVoteEngine<Self::AccountId>;

        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn current_grandpa_key)]
    pub type CurrentGrandpaKeys<T: Config> =
        StorageMap<_, Blake2_128Concat, InstitutionPalletId, [u8; 32], OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn key_owner)]
    pub type GrandpaKeyOwnerByKey<T: Config> =
        StorageMap<_, Blake2_128Concat, [u8; 32], InstitutionPalletId, OptionQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub _phantom: core::marker::PhantomData<T>,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                _phantom: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            // 中文注释：初始 GRANDPA 公钥与 CHINA_CB 的机构地址一一对应（1 国储会 + 43 省储会）。
            for node in CHINA_CB.iter() {
                let Some(institution) = reserve_pallet_id_to_bytes(node.shenfen_id) else {
                    continue;
                };
                assert!(
                    !GrandpaKeyOwnerByKey::<T>::contains_key(node.grandpa_key),
                    "duplicated initial grandpa key in CHINA_CB"
                );
                CurrentGrandpaKeys::<T>::insert(institution, node.grandpa_key);
                GrandpaKeyOwnerByKey::<T>::insert(node.grandpa_key, institution);
            }
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> Weight {
            let onchain = Pallet::<T>::on_chain_storage_version();
            if onchain < 2 {
                let mut reads: u64 = 1;
                let mut writes: u64 = 1;
                for (inst, key) in CurrentGrandpaKeys::<T>::iter() {
                    reads = reads.saturating_add(1);
                    GrandpaKeyOwnerByKey::<T>::insert(key, inst);
                    writes = writes.saturating_add(1);
                }
                STORAGE_VERSION.put::<Pallet<T>>();
                return T::DbWeight::get().reads_writes(reads, writes);
            }
            Weight::zero()
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 已发起 GRANDPA 密钥替换提案（并已在投票引擎创建内部提案）
        GrandpaKeyReplacementProposed {
            proposal_id: u64,
            org: u8,
            institution: InstitutionPalletId,
            proposer: T::AccountId,
            old_key: [u8; 32],
            new_key: [u8; 32],
        },
        /// GRANDPA 密钥替换提案已提交一票
        GrandpaKeyVoteSubmitted {
            proposal_id: u64,
            who: T::AccountId,
            approve: bool,
        },
        /// 提案达到通过状态但自动执行失败（投票不回滚）
        GrandpaKeyExecutionFailed { proposal_id: u64 },
        /// GRANDPA 密钥替换已完成并已调度 GRANDPA authority set 变更
        GrandpaKeyReplaced {
            proposal_id: u64,
            institution: InstitutionPalletId,
            old_key: [u8; 32],
            new_key: [u8; 32],
        },
        /// 已通过但不可执行的提案被取消
        FailedProposalCancelled {
            proposal_id: u64,
            institution: InstitutionPalletId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        InvalidInstitution,
        InstitutionOrgMismatch,
        UnsupportedOrg,
        UnauthorizedAdmin,
        ProposalActionNotFound,
        ProposalNotPassed,
        PassedProposalCannotBeCancelled,
        CurrentGrandpaKeyNotFound,
        NewKeyIsZero,
        InvalidEd25519Key,
        NewKeyUnchanged,
        NewKeyAlreadyUsed,
        OldAuthorityNotFound,
        GrandpaChangePending,
        ProposalStillExecutable,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 发起“GRANDPA 密钥替换”内部投票提案（仅支持国储会/省储会）。
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::propose_replace_grandpa_key())]
        pub fn propose_replace_grandpa_key(
            origin: OriginFor<T>,
            institution: InstitutionPalletId,
            new_key: [u8; 32],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(new_key != [0u8; 32], Error::<T>::NewKeyIsZero);
            let point = CompressedEdwardsY(new_key)
                .decompress()
                .ok_or(Error::<T>::InvalidEd25519Key)?;
            // 中文注释：仅”能解压”为曲线点还不够，small-order 弱公钥可能导致 GRANDPA 签名安全性失真。
            ensure!(!point.is_small_order(), Error::<T>::InvalidEd25519Key);

            let actual_org = institution_org(institution).ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                matches!(actual_org, ORG_NRC | ORG_PRC),
                Error::<T>::UnsupportedOrg
            );
            ensure!(
                Self::is_internal_admin(actual_org, institution, &who),
                Error::<T>::UnauthorizedAdmin
            );

            let old_key = CurrentGrandpaKeys::<T>::get(institution)
                .ok_or(Error::<T>::CurrentGrandpaKeyNotFound)?;
            ensure!(new_key != old_key, Error::<T>::NewKeyUnchanged);
            ensure!(
                !Self::is_key_used_by_other_institution(institution, &new_key),
                Error::<T>::NewKeyAlreadyUsed
            );

            let action = GrandpaKeyReplacementAction {
                institution,
                old_key,
                new_key,
            };

            let proposal_id = T::InternalVoteEngine::create_internal_proposal(
                who.clone(),
                actual_org,
                institution,
            )?;

            let data = action.encode();
            voting_engine_system::Pallet::<T>::store_proposal_data(proposal_id, data)?;
            voting_engine_system::Pallet::<T>::store_proposal_meta(proposal_id, frame_system::Pallet::<T>::block_number());

            Self::deposit_event(Event::<T>::GrandpaKeyReplacementProposed {
                proposal_id,
                org: actual_org,
                institution,
                proposer: who,
                old_key,
                new_key,
            });
            Ok(())
        }

        /// 对“GRANDPA 密钥替换”提案投票，达到阈值通过后自动执行替换。
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::vote_replace_grandpa_key())]
        pub fn vote_replace_grandpa_key(
            origin: OriginFor<T>,
            proposal_id: u64,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let action = Self::decode_action(proposal_id)?;
            let org = institution_org(action.institution).ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                Self::is_internal_admin(org, action.institution, &who),
                Error::<T>::UnauthorizedAdmin
            );

            T::InternalVoteEngine::cast_internal_vote(who.clone(), proposal_id, approve)?;

            Self::deposit_event(Event::<T>::GrandpaKeyVoteSubmitted {
                proposal_id,
                who,
                approve,
            });

            if let Some(proposal) = voting_engine_system::Pallet::<T>::proposals(proposal_id) {
                if proposal.status == STATUS_PASSED {
                    if Self::try_execute_from_action(proposal_id, action).is_err() {
                        Self::deposit_event(Event::<T>::GrandpaKeyExecutionFailed { proposal_id });
                    }
                }
            }

            Ok(())
        }

        /// 手动执行已通过的密钥替换提案。
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::execute_replace_grandpa_key())]
        pub fn execute_replace_grandpa_key(
            origin: OriginFor<T>,
            proposal_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let action = Self::decode_action(proposal_id)?;
            let org = institution_org(action.institution).ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                Self::is_internal_admin(org, action.institution, &who),
                Error::<T>::UnauthorizedAdmin
            );
            Self::try_execute_from_action(proposal_id, action)
        }

        /// 清理”已通过但确定无法执行”的提案。
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::cancel_failed_replace_grandpa_key())]
        pub fn cancel_failed_replace_grandpa_key(
            origin: OriginFor<T>,
            proposal_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let action = Self::decode_action(proposal_id)?;
            let org = institution_org(action.institution).ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                Self::is_internal_admin(org, action.institution, &who),
                Error::<T>::UnauthorizedAdmin
            );

            let proposal = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            ensure!(
                proposal.status == STATUS_PASSED,
                Error::<T>::ProposalNotPassed
            );
            // 中文注释：这里只允许清理”确定已经执行不了”的通过提案；
            // 若只是 GRANDPA 仍有 pending change，则属于暂时不可执行，应该等待后重试。
            match Self::validate_action(&action) {
                Ok(_) => return Err(Error::<T>::ProposalStillExecutable.into()),
                Err(Error::<T>::GrandpaChangePending) => {
                    return Err(Error::<T>::GrandpaChangePending.into())
                }
                Err(_) => {}
            }

            Self::deposit_event(Event::<T>::FailedProposalCancelled {
                proposal_id,
                institution: action.institution,
            });
            // 标记为已取消，防止重复取消或重复执行
            voting_engine_system::Pallet::<T>::set_status_and_emit(proposal_id, STATUS_REJECTED)?;
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn is_internal_admin(
            org: u8,
            institution: InstitutionPalletId,
            who: &T::AccountId,
        ) -> bool {
            <T as voting_engine_system::Config>::InternalAdminProvider::is_internal_admin(
                org,
                institution,
                who,
            )
        }

        fn is_key_used_by_other_institution(
            institution: InstitutionPalletId,
            key: &[u8; 32],
        ) -> bool {
            GrandpaKeyOwnerByKey::<T>::get(*key)
                .map(|owner| owner != institution)
                .unwrap_or(false)
        }

        fn decode_action(proposal_id: u64) -> Result<GrandpaKeyReplacementAction, DispatchError> {
            let data = voting_engine_system::Pallet::<T>::get_proposal_data(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            GrandpaKeyReplacementAction::decode(&mut &data[..])
                .map_err(|_| Error::<T>::ProposalActionNotFound.into())
        }

        fn try_execute_from_action(
            proposal_id: u64,
            action: GrandpaKeyReplacementAction,
        ) -> DispatchResult {
            let proposal = voting_engine_system::Pallet::<T>::proposals(proposal_id)
                .ok_or(Error::<T>::ProposalActionNotFound)?;
            ensure!(
                proposal.status == STATUS_PASSED,
                Error::<T>::ProposalNotPassed
            );

            let next_authorities = Self::validate_action(&action)?;

            pallet_grandpa::Pallet::<T>::schedule_change(
                next_authorities,
                T::GrandpaChangeDelay::get(),
                None,
            )?;

            // 中文注释：GRANDPA 接受调度后，链上“当前治理认可的目标 key”立即切到新值；
            // 真正 authority set 生效仍由 pallet-grandpa 在 delay 结束时完成。
            CurrentGrandpaKeys::<T>::insert(action.institution, action.new_key);
            GrandpaKeyOwnerByKey::<T>::remove(action.old_key);
            GrandpaKeyOwnerByKey::<T>::insert(action.new_key, action.institution);

            Self::deposit_event(Event::<T>::GrandpaKeyReplaced {
                proposal_id,
                institution: action.institution,
                old_key: action.old_key,
                new_key: action.new_key,
            });
            // 标记为已执行，防止双重执行
            voting_engine_system::Pallet::<T>::set_status_and_emit(proposal_id, STATUS_EXECUTED)?;
            Ok(())
        }

        fn validate_action(
            action: &GrandpaKeyReplacementAction,
        ) -> Result<Vec<(GrandpaAuthorityId, u64)>, Error<T>> {
            ensure!(
                pallet_grandpa::Pallet::<T>::pending_change().is_none(),
                Error::<T>::GrandpaChangePending
            );

            let old_authority = GrandpaAuthorityId::from(ed25519::Public::from_raw(action.old_key));
            let new_authority = GrandpaAuthorityId::from(ed25519::Public::from_raw(action.new_key));

            let mut found = false;
            // 中文注释：仅替换目标机构对应的一把 key，其余 authority 与权重原样保留。
            let next_authorities: Vec<(GrandpaAuthorityId, u64)> =
                pallet_grandpa::Pallet::<T>::grandpa_authorities()
                    .into_iter()
                    .map(|(authority, weight)| {
                        if authority == old_authority {
                            found = true;
                            (new_authority.clone(), weight)
                        } else {
                            (authority, weight)
                        }
                    })
                    .collect();

            ensure!(found, Error::<T>::OldAuthorityNotFound);
            let mut uniq = sp_std::collections::btree_set::BTreeSet::new();
            ensure!(
                next_authorities
                    .iter()
                    .all(|(authority, _)| uniq.insert(authority.encode())),
                Error::<T>::NewKeyAlreadyUsed
            );

            Ok(next_authorities)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{
        assert_noop, assert_ok, derive_impl, parameter_types,
        traits::{ConstU32, Hooks},
    };
    use frame_system as system;
    use primitives::china::china_cb::CHINA_CB;
    use sp_core::{Pair, Void};
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
        pub type Grandpa = pallet_grandpa;

        #[runtime::pallet_index(2)]
        pub type VotingEngineSystem = voting_engine_system;

        #[runtime::pallet_index(3)]
        pub type GrandpaKeyGov = super;
    }

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl system::Config for Test {
        type Block = Block;
        type AccountId = AccountId32;
        type Lookup = IdentityLookup<Self::AccountId>;
    }

    parameter_types! {
        pub const MaxGrandpaAuthorities: u32 = 64;
        pub const MaxGrandpaNominators: u32 = 0;
        pub const MaxSetIdSessionEntries: u64 = 16;
        pub const GrandpaChangeDelay: u64 = 30;
    }

    impl pallet_grandpa::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type WeightInfo = ();
        type MaxAuthorities = MaxGrandpaAuthorities;
        type MaxNominators = MaxGrandpaNominators;
        type MaxSetIdSessionEntries = MaxSetIdSessionEntries;
        type KeyOwnerProof = Void;
        type EquivocationReportSystem = ();
    }

    pub struct TestSfidEligibility;
    pub struct TestPopulationSnapshotVerifier;
    pub struct TestInternalAdminProvider;

    impl voting_engine_system::SfidEligibility<AccountId32, <Test as frame_system::Config>::Hash>
        for TestSfidEligibility
    {
        fn is_eligible(
            _sfid_hash: &<Test as frame_system::Config>::Hash,
            _who: &AccountId32,
        ) -> bool {
            false
        }

        fn verify_and_consume_vote_credential(
            _sfid_hash: &<Test as frame_system::Config>::Hash,
            _who: &AccountId32,
            _proposal_id: u64,
            _nonce: &[u8],
            _signature: &[u8],
        ) -> bool {
            false
        }

        fn cleanup_vote_credentials(_proposal_id: u64) {}
    }

    impl
        voting_engine_system::PopulationSnapshotVerifier<
            AccountId32,
            voting_engine_system::pallet::VoteNonceOf<Test>,
            voting_engine_system::pallet::VoteSignatureOf<Test>,
        > for TestPopulationSnapshotVerifier
    {
        fn verify_population_snapshot(
            _who: &AccountId32,
            _eligible_total: u64,
            _nonce: &voting_engine_system::pallet::VoteNonceOf<Test>,
            _signature: &voting_engine_system::pallet::VoteSignatureOf<Test>,
        ) -> bool {
            true
        }
    }

    impl voting_engine_system::InternalAdminProvider<AccountId32> for TestInternalAdminProvider {
        fn is_internal_admin(org: u8, institution: InstitutionPalletId, who: &AccountId32) -> bool {
            let mut who_raw = [0u8; 32];
            who_raw.copy_from_slice(who.as_ref());
            match org {
                ORG_NRC | ORG_PRC => CHINA_CB
                    .iter()
                    .find(|node| reserve_pallet_id_to_bytes(node.shenfen_id) == Some(institution))
                    .map(|node| node.admins.iter().any(|admin| *admin == who_raw))
                    .unwrap_or(false),
                _ => false,
            }
        }
    }

    pub struct TestTimeProvider;
    impl frame_support::traits::UnixTime for TestTimeProvider {
        fn now() -> core::time::Duration {
            core::time::Duration::from_secs(1_782_864_000) // 2026-07-01
        }
    }

    impl voting_engine_system::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type MaxVoteNonceLength = ConstU32<64>;
        type MaxVoteSignatureLength = ConstU32<64>;
        type MaxAutoFinalizePerBlock = ConstU32<64>;
        type MaxProposalsPerExpiry = ConstU32<128>;
        type MaxCleanupStepsPerBlock = ConstU32<8>;
        type CleanupKeysPerStep = ConstU32<64>;
        type MaxJointDecisionApprovals = ConstU32<32>;
        type MaxProposalDataLen = ConstU32<256>;
        type SfidEligibility = TestSfidEligibility;
        type PopulationSnapshotVerifier = TestPopulationSnapshotVerifier;
        type JointVoteResultCallback = ();
        type InternalAdminProvider = TestInternalAdminProvider;
        type InternalThresholdProvider = ();
        type JointInstitutionDecisionVerifier = ();
        type TimeProvider = TestTimeProvider;
        type WeightInfo = ();
    }

    impl Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type GrandpaChangeDelay = GrandpaChangeDelay;
        type InternalVoteEngine = voting_engine_system::Pallet<Test>;
        type WeightInfo = ();
    }

    fn grandpa_authorities() -> sp_consensus_grandpa::AuthorityList {
        vec![
            (
                GrandpaAuthorityId::from(ed25519::Public::from_raw(CHINA_CB[0].grandpa_key)),
                1,
            ),
            (
                GrandpaAuthorityId::from(ed25519::Public::from_raw(CHINA_CB[1].grandpa_key)),
                1,
            ),
        ]
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let mut storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("test storage should build");
        pallet_grandpa::GenesisConfig::<Test> {
            authorities: grandpa_authorities(),
            _config: Default::default(),
        }
        .assimilate_storage(&mut storage)
        .expect("grandpa genesis should assimilate");
        GenesisConfig::<Test>::default()
            .assimilate_storage(&mut storage)
            .expect("grandpa-key-gov genesis should assimilate");

        let mut ext = sp_io::TestExternalities::new(storage);
        ext.execute_with(|| {
            System::set_block_number(1);
        });
        ext
    }

    fn cb_admin(node_index: usize, admin_index: usize) -> AccountId32 {
        AccountId32::new(CHINA_CB[node_index].admins[admin_index])
    }

    fn cb_pallet_id(node_index: usize) -> InstitutionPalletId {
        reserve_pallet_id_to_bytes(CHINA_CB[node_index].shenfen_id)
            .expect("institution should map to pallet id")
    }

    fn prc_admin(index: usize) -> AccountId32 {
        cb_admin(1, index)
    }

    fn prc_pallet_id() -> InstitutionPalletId {
        cb_pallet_id(1)
    }

    fn valid_public_key(seed: u8) -> [u8; 32] {
        let mut seed_bytes = [0u8; 32];
        seed_bytes[0] = seed;
        ed25519::Pair::from_seed(&seed_bytes).public().0
    }

    fn identity_public_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        key[0] = 1;
        key
    }

    fn authority_id_from_key(key: [u8; 32]) -> GrandpaAuthorityId {
        GrandpaAuthorityId::from(ed25519::Public::from_raw(key))
    }

    fn pass_prc_proposal(node_index: usize, proposal_id: u64) {
        for admin_index in 0..6 {
            assert_ok!(GrandpaKeyGov::vote_replace_grandpa_key(
                RuntimeOrigin::signed(cb_admin(node_index, admin_index)),
                proposal_id,
                true,
            ));
        }
    }

    fn finalize_grandpa_at(block: u64) {
        System::set_block_number(block);
        <Grandpa as Hooks<u64>>::on_finalize(block);
    }

    /// 获取最近一次 create_internal_proposal 分配的 proposal_id。
    fn last_proposal_id() -> u64 {
        voting_engine_system::Pallet::<Test>::next_proposal_id().saturating_sub(1)
    }

    #[test]
    fn weak_small_order_new_key_is_rejected() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                GrandpaKeyGov::propose_replace_grandpa_key(
                    RuntimeOrigin::signed(prc_admin(0)),
                    prc_pallet_id(),
                    identity_public_key()
                ),
                Error::<Test>::InvalidEd25519Key
            );
        });
    }

    #[test]
    fn passed_proposal_executes_and_cleans_up_state() {
        new_test_ext().execute_with(|| {
            let institution = prc_pallet_id();
            let old_key = CurrentGrandpaKeys::<Test>::get(institution)
                .expect("institution should have an initial key");
            let new_key = valid_public_key(31);

            assert_ok!(GrandpaKeyGov::propose_replace_grandpa_key(
                RuntimeOrigin::signed(prc_admin(0)),
                institution,
                new_key,
            ));
            let pid = last_proposal_id();

            pass_prc_proposal(1, pid);

            let pending_change = Grandpa::pending_change().expect("change should be scheduled");
            assert_eq!(pending_change.scheduled_at, 1);
            assert_eq!(pending_change.delay, GrandpaChangeDelay::get());
            assert!(pending_change
                .next_authorities
                .iter()
                .any(|(authority, _)| *authority == authority_id_from_key(new_key)));

            assert_eq!(CurrentGrandpaKeys::<Test>::get(institution), Some(new_key));
            assert!(GrandpaKeyOwnerByKey::<Test>::get(old_key).is_none());
            assert_eq!(
                GrandpaKeyOwnerByKey::<Test>::get(new_key),
                Some(institution)
            );
            assert!(System::events().iter().any(|record| {
                matches!(
                    &record.event,
                    RuntimeEvent::GrandpaKeyGov(Event::<Test>::GrandpaKeyReplaced {
                        proposal_id,
                        institution: inst,
                        old_key: replaced_old_key,
                        new_key: replaced_new_key,
                    }) if *proposal_id == pid
                        && *inst == institution
                        && *replaced_old_key == old_key
                        && *replaced_new_key == new_key
                )
            }));
        });
    }

    #[test]
    fn passed_proposal_can_be_manually_executed_after_pending_change_clears() {
        new_test_ext().execute_with(|| {
            let institution = prc_pallet_id();
            let old_key = CurrentGrandpaKeys::<Test>::get(institution)
                .expect("institution should have an initial key");
            let new_key = valid_public_key(41);

            assert_ok!(GrandpaKeyGov::propose_replace_grandpa_key(
                RuntimeOrigin::signed(prc_admin(0)),
                institution,
                new_key,
            ));
            let pid = last_proposal_id();
            assert_ok!(Grandpa::schedule_change(
                grandpa_authorities(),
                GrandpaChangeDelay::get(),
                None,
            ));

            pass_prc_proposal(1, pid);

            assert_eq!(
                voting_engine_system::Pallet::<Test>::proposals(pid)
                    .expect("passed proposal should remain for retries")
                    .status,
                STATUS_PASSED
            );
            assert_eq!(CurrentGrandpaKeys::<Test>::get(institution), Some(old_key));
            assert!(voting_engine_system::Pallet::<Test>::get_proposal_data(pid).is_some());
            assert!(System::events().iter().any(|record| {
                matches!(
                    &record.event,
                    RuntimeEvent::GrandpaKeyGov(Event::<Test>::GrandpaKeyExecutionFailed {
                        proposal_id
                    }) if *proposal_id == pid
                )
            }));

            finalize_grandpa_at(1 + GrandpaChangeDelay::get());
            assert!(Grandpa::pending_change().is_none());

            assert_ok!(GrandpaKeyGov::execute_replace_grandpa_key(
                RuntimeOrigin::signed(prc_admin(0)),
                pid,
            ));

            assert_eq!(CurrentGrandpaKeys::<Test>::get(institution), Some(new_key));
            assert!(GrandpaKeyOwnerByKey::<Test>::get(old_key).is_none());
            assert_eq!(
                GrandpaKeyOwnerByKey::<Test>::get(new_key),
                Some(institution)
            );
            assert!(Grandpa::pending_change().is_some());
        });
    }

    #[test]
    fn cancel_failed_replace_grandpa_key_cleans_up_passed_but_invalid_proposal() {
        new_test_ext().execute_with(|| {
            let institution = prc_pallet_id();
            let old_key = CurrentGrandpaKeys::<Test>::get(institution)
                .expect("institution should have an initial key");
            let new_key = valid_public_key(51);
            let replacement_authority = valid_public_key(52);

            assert_ok!(GrandpaKeyGov::propose_replace_grandpa_key(
                RuntimeOrigin::signed(prc_admin(0)),
                institution,
                new_key,
            ));
            let pid = last_proposal_id();
            assert_ok!(Grandpa::schedule_change(
                vec![
                    (authority_id_from_key(CHINA_CB[0].grandpa_key), 1),
                    (authority_id_from_key(replacement_authority), 1),
                ],
                GrandpaChangeDelay::get(),
                None,
            ));

            pass_prc_proposal(1, pid);

            assert_eq!(
                voting_engine_system::Pallet::<Test>::proposals(pid)
                    .expect("passed proposal should remain for cleanup")
                    .status,
                STATUS_PASSED
            );
            finalize_grandpa_at(1 + GrandpaChangeDelay::get());

            assert_eq!(CurrentGrandpaKeys::<Test>::get(institution), Some(old_key));
            assert_eq!(
                Grandpa::grandpa_authorities(),
                vec![
                    (authority_id_from_key(CHINA_CB[0].grandpa_key), 1),
                    (authority_id_from_key(replacement_authority), 1),
                ]
            );

            assert_ok!(GrandpaKeyGov::cancel_failed_replace_grandpa_key(
                RuntimeOrigin::signed(prc_admin(0)),
                pid,
            ));

            assert!(System::events().iter().any(|record| {
                matches!(
                    &record.event,
                    RuntimeEvent::GrandpaKeyGov(Event::<Test>::FailedProposalCancelled {
                        proposal_id,
                        institution: inst,
                    }) if *proposal_id == pid && *inst == institution
                )
            }));
        });
    }

    #[test]
    fn cancel_failed_replace_grandpa_key_rejects_temporarily_blocked_proposal() {
        new_test_ext().execute_with(|| {
            let institution = prc_pallet_id();
            let old_key = CurrentGrandpaKeys::<Test>::get(institution)
                .expect("institution should have an initial key");
            let new_key = valid_public_key(71);

            assert_ok!(GrandpaKeyGov::propose_replace_grandpa_key(
                RuntimeOrigin::signed(prc_admin(0)),
                institution,
                new_key,
            ));
            let pid = last_proposal_id();
            assert_ok!(Grandpa::schedule_change(
                grandpa_authorities(),
                GrandpaChangeDelay::get(),
                None,
            ));

            pass_prc_proposal(1, pid);

            assert_noop!(
                GrandpaKeyGov::cancel_failed_replace_grandpa_key(
                    RuntimeOrigin::signed(prc_admin(0)),
                    pid,
                ),
                Error::<Test>::GrandpaChangePending
            );

            assert_eq!(CurrentGrandpaKeys::<Test>::get(institution), Some(old_key));
            assert!(voting_engine_system::Pallet::<Test>::get_proposal_data(pid).is_some());
            assert_eq!(
                voting_engine_system::Pallet::<Test>::proposals(pid)
                    .expect("passed proposal should remain active")
                    .status,
                STATUS_PASSED
            );
        });
    }
}
