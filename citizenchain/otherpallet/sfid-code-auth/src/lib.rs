#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod weights;

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::weights::Weight;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

#[derive(
    Clone,
    Encode,
    Decode,
    DecodeWithMemTracking,
    Eq,
    PartialEq,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
pub struct BindCredential<Hash, Nonce, Signature, BlockNumber> {
    pub sfid_code_hash: Hash,
    pub nonce: Nonce,
    pub expires_at: BlockNumber,
    pub signature: Signature,
}

pub trait SfidVerifier<AccountId, Hash, Nonce, Signature, BlockNumber> {
    fn verify(
        account: &AccountId,
        credential: &BindCredential<Hash, Nonce, Signature, BlockNumber>,
    ) -> bool;
}

impl<AccountId, Hash, Nonce, Signature, BlockNumber>
    SfidVerifier<AccountId, Hash, Nonce, Signature, BlockNumber> for ()
{
    fn verify(
        _account: &AccountId,
        _credential: &BindCredential<Hash, Nonce, Signature, BlockNumber>,
    ) -> bool {
        false
    }
}

/// 中文注释：公民投票实时验签接口（包含 proposal_id 与 nonce）。
pub trait SfidVoteVerifier<AccountId, Hash, Nonce, Signature> {
    fn verify_vote(
        account: &AccountId,
        sfid_hash: Hash,
        proposal_id: u64,
        nonce: &Nonce,
        signature: &Signature,
    ) -> bool;
}

impl<AccountId, Hash, Nonce, Signature> SfidVoteVerifier<AccountId, Hash, Nonce, Signature> for () {
    fn verify_vote(
        _account: &AccountId,
        _sfid_hash: Hash,
        _proposal_id: u64,
        _nonce: &Nonce,
        _signature: &Signature,
    ) -> bool {
        false
    }
}

/// 中文注释：绑定成功后的钩子，用于让“发行模块”只做发行逻辑。
pub trait OnSfidBound<AccountId, Hash> {
    fn on_sfid_bound(_who: &AccountId, _sfid_hash: Hash) {}
}

impl<AccountId, Hash> OnSfidBound<AccountId, Hash> for () {}

pub trait OnSfidBoundWeight {
    fn on_sfid_bound_weight() -> Weight {
        Weight::zero()
    }
}

impl OnSfidBoundWeight for () {}

/// 中文注释：给投票模块使用的统一资格接口。
pub trait SfidEligibilityProvider<AccountId, Hash> {
    fn is_eligible(sfid_hash: &Hash, who: &AccountId) -> bool;
    fn verify_and_consume_vote_credential(
        sfid_hash: &Hash,
        who: &AccountId,
        proposal_id: u64,
        nonce: &[u8],
        signature: &[u8],
    ) -> bool;

    /// 清理某个提案维度下的投票凭证防重放状态。
    fn cleanup_vote_credentials(_proposal_id: u64) {}
}
#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;
    use frame_support::{pallet_prelude::*, Blake2_128Concat};
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{Hash, Saturating};
    use sp_std::vec::Vec;

    pub type SfidOf<T> = BoundedVec<u8, <T as Config>::MaxSfidLength>;
    pub type NonceOf<T> = BoundedVec<u8, <T as Config>::MaxCredentialNonceLength>;
    pub type SignatureOf<T> = BoundedVec<u8, <T as Config>::MaxCredentialSignatureLength>;
    pub type CredentialOf<T> = BindCredential<
        <T as frame_system::Config>::Hash,
        NonceOf<T>,
        SignatureOf<T>,
        BlockNumberFor<T>,
    >;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        #[pallet::constant]
        type MaxSfidLength: Get<u32>;

        #[pallet::constant]
        type MaxCredentialNonceLength: Get<u32>;

        #[pallet::constant]
        type MaxCredentialSignatureLength: Get<u32>;

        #[pallet::constant]
        type MaxBindCredentialLifetimeBlocks: Get<BlockNumberFor<Self>>;

        #[pallet::constant]
        type MaxCredentialNonceCleanupPerBlock: Get<u32>;

        #[pallet::constant]
        type MaxCredentialNoncesPerExpiryBlock: Get<u32>;

        /// 中文注释：SFID 系统签名校验器（外部接口桥接点）。
        type SfidVerifier: SfidVerifier<
            Self::AccountId,
            Self::Hash,
            NonceOf<Self>,
            SignatureOf<Self>,
            BlockNumberFor<Self>,
        >;
        /// 中文注释：公民投票实时验签器。
        type SfidVoteVerifier: SfidVoteVerifier<
            Self::AccountId,
            Self::Hash,
            NonceOf<Self>,
            SignatureOf<Self>,
        >;

        /// 中文注释：绑定后回调到发行模块发放认证奖励。
        type OnSfidBound: OnSfidBound<Self::AccountId, Self::Hash> + OnSfidBoundWeight;

        /// 权重信息：由 runtime 注入实际 benchmark 结果。
        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn sfid_to_account)]
    pub type SfidToAccount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, T::AccountId, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn account_to_sfid)]
    pub type AccountToSfid<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, T::Hash, OptionQuery>;

    /// 中文注释：当前已绑定 SFID 码的账户数量，可用于公民投票基数。
    #[pallet::storage]
    #[pallet::getter(fn bound_count)]
    pub type BoundCount<T> = StorageValue<_, u64, ValueQuery>;

    /// 中文注释：已使用凭证 nonce（哈希）防重放，value 为该凭证过期区块。
    #[pallet::storage]
    #[pallet::getter(fn used_credential_nonce)]
    pub type UsedCredentialNonce<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, BlockNumberFor<T>, OptionQuery>;

    /// 中文注释：按凭证过期区块索引 nonce 哈希，供按块清理。
    #[pallet::storage]
    #[pallet::getter(fn credential_nonces_by_expiry)]
    pub type CredentialNoncesByExpiry<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BlockNumberFor<T>,
        BoundedVec<T::Hash, T::MaxCredentialNoncesPerExpiryBlock>,
        ValueQuery,
    >;

    /// 中文注释：过期 nonce 清理游标，记录“上个区块未清完”的过期桶。
    #[pallet::storage]
    #[pallet::getter(fn pending_credential_nonce_cleanup_expiry)]
    pub type PendingCredentialNonceCleanupExpiry<T: Config> =
        StorageValue<_, BlockNumberFor<T>, OptionQuery>;

    /// 中文注释：公民投票验签 nonce（哈希）防重放（提案+身份+nonce 三元维度）。
    #[pallet::storage]
    #[pallet::getter(fn used_vote_nonce)]
    pub type UsedVoteNonce<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        (T::Hash, T::Hash),
        bool,
        ValueQuery,
    >;

    /// 中文注释：SFID 当前主账户（用于 SFID 码验签）。
    #[pallet::storage]
    #[pallet::getter(fn sfid_main_account)]
    pub type SfidMainAccount<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

    /// 中文注释：SFID 备用账户1（可发起轮换）。
    #[pallet::storage]
    #[pallet::getter(fn sfid_backup_account_1)]
    pub type SfidBackupAccount1<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

    /// 中文注释：SFID 备用账户2（可发起轮换）。
    #[pallet::storage]
    #[pallet::getter(fn sfid_backup_account_2)]
    pub type SfidBackupAccount2<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub sfid_main_account: Option<T::AccountId>,
        pub sfid_backup_account_1: Option<T::AccountId>,
        pub sfid_backup_account_2: Option<T::AccountId>,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            // 中文注释：默认值不回退常量，要求链规格显式传入三把 SFID 账户。
            Self {
                sfid_main_account: None,
                sfid_backup_account_1: None,
                sfid_backup_account_2: None,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            // 中文注释：默认创世允许不配置 SFID 三把账户（no-op），
            // 但如果配置了任意一个，则必须三把都配置且互不相同。
            if self.sfid_main_account.is_none()
                && self.sfid_backup_account_1.is_none()
                && self.sfid_backup_account_2.is_none()
            {
                return;
            }

            // 中文注释：只要启用 SFID 创世配置，就必须三把完整提供。
            let main = self
                .sfid_main_account
                .clone()
                .expect("SFID genesis requires sfid_main_account");
            let backup_1 = self
                .sfid_backup_account_1
                .clone()
                .expect("SFID genesis requires sfid_backup_account_1");
            let backup_2 = self
                .sfid_backup_account_2
                .clone()
                .expect("SFID genesis requires sfid_backup_account_2");

            assert!(
                main != backup_1 && main != backup_2 && backup_1 != backup_2,
                "SFID genesis keys must be pairwise distinct"
            );

            SfidMainAccount::<T>::put(&main);
            SfidBackupAccount1::<T>::put(&backup_1);
            SfidBackupAccount2::<T>::put(&backup_2);
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        SfidBound {
            who: T::AccountId,
            sfid_hash: T::Hash,
            credential_nonce_hash: T::Hash,
        },
        SfidUnbound {
            who: T::AccountId,
            sfid_hash: T::Hash,
        },
        SfidKeysRotated {
            operator: T::AccountId,
            new_main: T::AccountId,
            backup_1: T::AccountId,
            backup_2: T::AccountId,
        },
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            let max_cleanup = T::MaxCredentialNonceCleanupPerBlock::get() as usize;
            if max_cleanup == 0 {
                return Weight::zero();
            }

            let db_weight = T::DbWeight::get();
            let mut weight = db_weight.reads(1);
            let mut cleanup_budget = max_cleanup;
            let pending_expiry = PendingCredentialNonceCleanupExpiry::<T>::get();

            if let Some(expiry) = pending_expiry {
                if expiry <= n {
                    let (removed, has_remaining, cleanup_weight) =
                        Self::cleanup_credential_nonce_bucket(expiry, cleanup_budget);
                    weight = weight.saturating_add(cleanup_weight);
                    cleanup_budget = cleanup_budget.saturating_sub(removed);
                    if has_remaining {
                        PendingCredentialNonceCleanupExpiry::<T>::put(expiry);
                        weight = weight.saturating_add(db_weight.writes(1));
                        return weight;
                    }

                    PendingCredentialNonceCleanupExpiry::<T>::kill();
                    weight = weight.saturating_add(db_weight.writes(1));
                }
            }

            if cleanup_budget == 0 || pending_expiry == Some(n) {
                return weight;
            }

            let (_removed, has_remaining, cleanup_weight) =
                Self::cleanup_credential_nonce_bucket(n, cleanup_budget);
            weight = weight.saturating_add(cleanup_weight);
            if has_remaining {
                PendingCredentialNonceCleanupExpiry::<T>::put(n);
                weight = weight.saturating_add(db_weight.writes(1));
            }

            weight
        }
    }

    #[pallet::error]
    pub enum Error<T> {
        EmptySfid,
        EmptyCredentialNonce,
        CredentialExpired,
        CredentialExpiryTooFar,
        CredentialNonceExpiryBucketFull,
        InvalidCredentialSfidCodeHash,
        CredentialAlreadyUsed,
        InvalidSfidCredentialSignature,
        SfidAlreadyBoundToAnotherAccount,
        SameSfidAlreadyBound,
        NotBound,
        UnauthorizedSfidOperator,
        DuplicateSfidKey,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 中文注释：使用 SFID 系统签发的一次性凭证绑定钱包。
        #[pallet::call_index(0)]
        #[pallet::weight(
            T::WeightInfo::bind_sfid()
                .saturating_add(T::OnSfidBound::on_sfid_bound_weight())
        )]
        pub fn bind_sfid(
            origin: OriginFor<T>,
            sfid_code: SfidOf<T>,
            credential: CredentialOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(!sfid_code.is_empty(), Error::<T>::EmptySfid);
            ensure!(
                !credential.nonce.is_empty(),
                Error::<T>::EmptyCredentialNonce
            );

            let sfid_hash = T::Hashing::hash(sfid_code.as_slice());
            ensure!(
                credential.sfid_code_hash == sfid_hash,
                Error::<T>::InvalidCredentialSfidCodeHash
            );

            let now = <frame_system::Pallet<T>>::block_number();
            ensure!(credential.expires_at >= now, Error::<T>::CredentialExpired);
            let max_expires_at = now.saturating_add(T::MaxBindCredentialLifetimeBlocks::get());
            ensure!(
                credential.expires_at <= max_expires_at,
                Error::<T>::CredentialExpiryTooFar
            );

            let nonce_hash = T::Hashing::hash(credential.nonce.as_slice());
            ensure!(
                !UsedCredentialNonce::<T>::contains_key(nonce_hash),
                Error::<T>::CredentialAlreadyUsed
            );

            ensure!(
                T::SfidVerifier::verify(&who, &credential),
                Error::<T>::InvalidSfidCredentialSignature
            );

            if let Some(existing_owner) = SfidToAccount::<T>::get(sfid_hash) {
                ensure!(
                    existing_owner == who,
                    Error::<T>::SfidAlreadyBoundToAnotherAccount
                );
                return Err(Error::<T>::SameSfidAlreadyBound.into());
            }

            if let Some(old_sfid_hash) = AccountToSfid::<T>::get(&who) {
                SfidToAccount::<T>::remove(old_sfid_hash);
            } else {
                BoundCount::<T>::mutate(|v| *v = v.saturating_add(1));
            }

            CredentialNoncesByExpiry::<T>::try_mutate(
                credential.expires_at,
                |nonces_by_expiry| -> Result<(), Error<T>> {
                    nonces_by_expiry
                        .try_push(nonce_hash)
                        .map_err(|_| Error::<T>::CredentialNonceExpiryBucketFull)?;
                    Ok(())
                },
            )?;
            SfidToAccount::<T>::insert(sfid_hash, &who);
            AccountToSfid::<T>::insert(&who, sfid_hash);
            UsedCredentialNonce::<T>::insert(nonce_hash, credential.expires_at);

            T::OnSfidBound::on_sfid_bound(&who, sfid_hash);

            Self::deposit_event(Event::<T>::SfidBound {
                who,
                sfid_hash,
                credential_nonce_hash: nonce_hash,
            });
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::unbind_sfid())]
        pub fn unbind_sfid(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let sfid_hash = AccountToSfid::<T>::get(&who).ok_or(Error::<T>::NotBound)?;

            AccountToSfid::<T>::remove(&who);
            SfidToAccount::<T>::remove(sfid_hash);
            BoundCount::<T>::mutate(|v| *v = v.saturating_sub(1));

            Self::deposit_event(Event::<T>::SfidUnbound { who, sfid_hash });
            Ok(())
        }

        /// 中文注释：仅备用账户可发起轮换；发起者升级为主账户，并提交一个新备用账户补位。
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::rotate_sfid_keys())]
        pub fn rotate_sfid_keys(origin: OriginFor<T>, new_backup: T::AccountId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let main = SfidMainAccount::<T>::get().ok_or(Error::<T>::UnauthorizedSfidOperator)?;
            let backup_1 =
                SfidBackupAccount1::<T>::get().ok_or(Error::<T>::UnauthorizedSfidOperator)?;
            let backup_2 =
                SfidBackupAccount2::<T>::get().ok_or(Error::<T>::UnauthorizedSfidOperator)?;
            ensure!(
                who == backup_1 || who == backup_2,
                Error::<T>::UnauthorizedSfidOperator
            );

            let survivor = if who == backup_1 {
                backup_2.clone()
            } else {
                backup_1.clone()
            };

            ensure!(new_backup != main, Error::<T>::DuplicateSfidKey);
            ensure!(new_backup != who, Error::<T>::DuplicateSfidKey);
            ensure!(new_backup != survivor, Error::<T>::DuplicateSfidKey);

            SfidMainAccount::<T>::put(&who);
            SfidBackupAccount1::<T>::put(&survivor);
            SfidBackupAccount2::<T>::put(&new_backup);

            Self::deposit_event(Event::<T>::SfidKeysRotated {
                operator: who.clone(),
                new_main: who,
                backup_1: survivor,
                backup_2: new_backup,
            });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn cleanup_credential_nonce_bucket(
            expiry: BlockNumberFor<T>,
            max_cleanup: usize,
        ) -> (usize, bool, Weight) {
            let db_weight = T::DbWeight::get();
            let mut weight = db_weight.reads_writes(1, 1);
            let mut nonce_hashes: Vec<T::Hash> =
                CredentialNoncesByExpiry::<T>::take(expiry).into_inner();
            if nonce_hashes.is_empty() {
                return (0, false, weight);
            }

            let remove_count = core::cmp::min(max_cleanup, nonce_hashes.len());
            for nonce_hash in nonce_hashes.drain(..remove_count) {
                UsedCredentialNonce::<T>::remove(nonce_hash);
            }
            weight = weight.saturating_add(db_weight.writes(remove_count as u64));

            let has_remaining = !nonce_hashes.is_empty();
            if has_remaining {
                let remaining: BoundedVec<T::Hash, T::MaxCredentialNoncesPerExpiryBlock> =
                    nonce_hashes
                        .try_into()
                        .expect("remaining nonce hashes should stay within bound");
                CredentialNoncesByExpiry::<T>::insert(expiry, remaining);
                weight = weight.saturating_add(db_weight.writes(1));
            }

            (remove_count, has_remaining, weight)
        }

        pub fn is_bound(who: &T::AccountId) -> bool {
            AccountToSfid::<T>::contains_key(who)
        }

        pub fn is_sfid_bound_to(sfid_hash: &T::Hash, who: &T::AccountId) -> bool {
            SfidToAccount::<T>::get(sfid_hash)
                .map(|owner| owner == *who)
                .unwrap_or(false)
        }

        /// 中文注释：当前 SFID 主账户即验签公钥来源；仅当 AccountId 可还原为 32 字节原始公钥时返回。
        pub fn current_sfid_verify_pubkey() -> Option<[u8; 32]> {
            let main = SfidMainAccount::<T>::get()?;
            let encoded = main.encode();
            if encoded.len() != 32 {
                return None;
            }
            let mut raw = [0u8; 32];
            raw.copy_from_slice(encoded.as_slice());
            Some(raw)
        }
    }

    impl<T: Config> crate::SfidEligibilityProvider<T::AccountId, T::Hash> for Pallet<T> {
        fn is_eligible(sfid_hash: &T::Hash, who: &T::AccountId) -> bool {
            Self::is_sfid_bound_to(sfid_hash, who)
        }

        fn verify_and_consume_vote_credential(
            sfid_hash: &T::Hash,
            who: &T::AccountId,
            proposal_id: u64,
            nonce: &[u8],
            signature: &[u8],
        ) -> bool {
            if nonce.is_empty() || signature.is_empty() {
                return false;
            }

            if !Self::is_sfid_bound_to(sfid_hash, who) {
                return false;
            }

            let nonce_hash = T::Hashing::hash(nonce);
            let vote_nonce_key = (sfid_hash.clone(), nonce_hash);
            if UsedVoteNonce::<T>::get(proposal_id, vote_nonce_key.clone()) {
                return false;
            }

            let nonce_bounded: NonceOf<T> = match nonce.to_vec().try_into() {
                Ok(v) => v,
                Err(_) => return false,
            };
            let signature_bounded: SignatureOf<T> = match signature.to_vec().try_into() {
                Ok(v) => v,
                Err(_) => return false,
            };

            if !T::SfidVoteVerifier::verify_vote(
                who,
                sfid_hash.clone(),
                proposal_id,
                &nonce_bounded,
                &signature_bounded,
            ) {
                return false;
            }

            UsedVoteNonce::<T>::insert(proposal_id, vote_nonce_key, true);
            true
        }

        fn cleanup_vote_credentials(proposal_id: u64) {
            let clear_result = UsedVoteNonce::<T>::clear_prefix(proposal_id, u32::MAX, None);
            debug_assert!(
                clear_result.maybe_cursor.is_none(),
                "vote nonces were not fully cleared"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{assert_noop, assert_ok, derive_impl, parameter_types, traits::Hooks};
    use frame_system as system;
    use frame_system::pallet_prelude::BlockNumberFor;
    use sp_runtime::traits::Hash;
    use sp_runtime::{traits::IdentityLookup, BuildStorage};

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
        pub type SfidCodeAuth = super;
    }

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl system::Config for Test {
        type Block = Block;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
    }

    pub struct TestSfidVerifier;
    impl
        SfidVerifier<
            u64,
            <Test as frame_system::Config>::Hash,
            NonceOf<Test>,
            SignatureOf<Test>,
            BlockNumberFor<Test>,
        > for TestSfidVerifier
    {
        fn verify(_account: &u64, credential: &CredentialOf<Test>) -> bool {
            credential.signature.as_slice() == b"bind-ok"
        }
    }

    pub struct TestSfidVoteVerifier;
    impl
        SfidVoteVerifier<
            u64,
            <Test as frame_system::Config>::Hash,
            NonceOf<Test>,
            SignatureOf<Test>,
        > for TestSfidVoteVerifier
    {
        fn verify_vote(
            _account: &u64,
            _sfid_hash: <Test as frame_system::Config>::Hash,
            _proposal_id: u64,
            _nonce: &NonceOf<Test>,
            signature: &SignatureOf<Test>,
        ) -> bool {
            signature.as_slice() == b"vote-ok"
        }
    }

    parameter_types! {
        pub const MaxSfidLength: u32 = 64;
        pub const MaxCredentialNonceLength: u32 = 64;
        pub const MaxCredentialSignatureLength: u32 = 64;
        pub const MaxBindCredentialLifetimeBlocks: u64 = 100;
        pub const MaxCredentialNonceCleanupPerBlock: u32 = 64;
        pub const MaxCredentialNoncesPerExpiryBlock: u32 = 1024;
    }

    impl Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type MaxSfidLength = MaxSfidLength;
        type MaxCredentialNonceLength = MaxCredentialNonceLength;
        type MaxCredentialSignatureLength = MaxCredentialSignatureLength;
        type MaxBindCredentialLifetimeBlocks = MaxBindCredentialLifetimeBlocks;
        type MaxCredentialNonceCleanupPerBlock = MaxCredentialNonceCleanupPerBlock;
        type MaxCredentialNoncesPerExpiryBlock = MaxCredentialNoncesPerExpiryBlock;
        type SfidVerifier = TestSfidVerifier;
        type SfidVoteVerifier = TestSfidVoteVerifier;
        type OnSfidBound = ();
        type WeightInfo = ();
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let mut storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("frame system genesis storage should build");
        GenesisConfig::<Test> {
            sfid_main_account: Some(10),
            sfid_backup_account_1: Some(11),
            sfid_backup_account_2: Some(12),
        }
        .assimilate_storage(&mut storage)
        .expect("sfid genesis should assimilate");
        let mut ext = sp_io::TestExternalities::new(storage);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }

    fn sfid(input: &str) -> SfidOf<Test> {
        input
            .as_bytes()
            .to_vec()
            .try_into()
            .expect("sfid should fit")
    }

    fn nonce(input: &str) -> NonceOf<Test> {
        input
            .as_bytes()
            .to_vec()
            .try_into()
            .expect("nonce should fit")
    }

    fn signature(input: &str) -> SignatureOf<Test> {
        input
            .as_bytes()
            .to_vec()
            .try_into()
            .expect("signature should fit")
    }

    fn credential_with_expiry(
        sfid_plain: &str,
        nonce_plain: &str,
        sig_plain: &str,
        expires_at: BlockNumberFor<Test>,
    ) -> CredentialOf<Test> {
        BindCredential {
            sfid_code_hash: <Test as frame_system::Config>::Hashing::hash(sfid_plain.as_bytes()),
            nonce: nonce(nonce_plain),
            expires_at,
            signature: signature(sig_plain),
        }
    }

    fn credential(sfid_plain: &str, nonce_plain: &str, sig_plain: &str) -> CredentialOf<Test> {
        let expires_at = System::block_number().saturating_add(10);
        credential_with_expiry(sfid_plain, nonce_plain, sig_plain, expires_at)
    }

    fn bind_many_with_expiry(
        account_start: u64,
        count: usize,
        label: &str,
        expires_at: BlockNumberFor<Test>,
    ) -> Vec<<Test as frame_system::Config>::Hash> {
        let mut nonce_hashes = Vec::with_capacity(count);
        for i in 0..count {
            let account = account_start.saturating_add(i as u64);
            let sfid_plain = format!("{label}-sfid-{i}");
            let nonce_plain = format!("{label}-nonce-{i}");
            assert_ok!(SfidCodeAuth::bind_sfid(
                RuntimeOrigin::signed(account),
                sfid(sfid_plain.as_str()),
                credential_with_expiry(
                    sfid_plain.as_str(),
                    nonce_plain.as_str(),
                    "bind-ok",
                    expires_at
                )
            ));
            nonce_hashes.push(<Test as frame_system::Config>::Hashing::hash(
                nonce_plain.as_bytes(),
            ));
        }
        nonce_hashes
    }

    fn count_used_nonce_hashes(nonce_hashes: &[<Test as frame_system::Config>::Hash]) -> usize {
        nonce_hashes
            .iter()
            .filter(|nonce_hash| UsedCredentialNonce::<Test>::contains_key(nonce_hash))
            .count()
    }

    #[test]
    fn bind_sfid_works_and_tracks_binding_count() {
        new_test_ext().execute_with(|| {
            assert_ok!(SfidCodeAuth::bind_sfid(
                RuntimeOrigin::signed(1),
                sfid("sfid-a"),
                credential("sfid-a", "n-a", "bind-ok")
            ));
            assert_eq!(BoundCount::<Test>::get(), 1);
            assert!(SfidCodeAuth::is_bound(&1));
        });
    }

    #[test]
    fn reused_bind_nonce_is_rejected() {
        new_test_ext().execute_with(|| {
            assert_ok!(SfidCodeAuth::bind_sfid(
                RuntimeOrigin::signed(1),
                sfid("sfid-a"),
                credential("sfid-a", "same", "bind-ok")
            ));
            assert_noop!(
                SfidCodeAuth::bind_sfid(
                    RuntimeOrigin::signed(1),
                    sfid("sfid-b"),
                    credential("sfid-b", "same", "bind-ok")
                ),
                Error::<Test>::CredentialAlreadyUsed
            );
        });
    }

    #[test]
    fn same_sfid_cannot_bind_to_another_account() {
        new_test_ext().execute_with(|| {
            assert_ok!(SfidCodeAuth::bind_sfid(
                RuntimeOrigin::signed(1),
                sfid("sfid-same"),
                credential("sfid-same", "n-1", "bind-ok")
            ));
            assert_noop!(
                SfidCodeAuth::bind_sfid(
                    RuntimeOrigin::signed(2),
                    sfid("sfid-same"),
                    credential("sfid-same", "n-2", "bind-ok")
                ),
                Error::<Test>::SfidAlreadyBoundToAnotherAccount
            );
        });
    }

    #[test]
    fn vote_credential_nonce_replay_is_rejected_per_proposal() {
        new_test_ext().execute_with(|| {
            assert_ok!(SfidCodeAuth::bind_sfid(
                RuntimeOrigin::signed(1),
                sfid("sfid-vote"),
                credential("sfid-vote", "bind-n", "bind-ok")
            ));
            let sfid_hash = <Test as frame_system::Config>::Hashing::hash(b"sfid-vote");

            assert!(<Pallet<Test> as SfidEligibilityProvider<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::verify_and_consume_vote_credential(
                &sfid_hash,
                &1,
                100,
                b"vote-nonce",
                b"vote-ok"
            ));
            assert!(!<Pallet<Test> as SfidEligibilityProvider<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::verify_and_consume_vote_credential(
                &sfid_hash,
                &1,
                100,
                b"vote-nonce",
                b"vote-ok"
            ));
            assert!(<Pallet<Test> as SfidEligibilityProvider<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::verify_and_consume_vote_credential(
                &sfid_hash,
                &1,
                101,
                b"vote-nonce",
                b"vote-ok"
            ));

            <Pallet<Test> as SfidEligibilityProvider<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::cleanup_vote_credentials(100);
            assert!(<Pallet<Test> as SfidEligibilityProvider<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::verify_and_consume_vote_credential(
                &sfid_hash,
                &1,
                100,
                b"vote-nonce",
                b"vote-ok"
            ));
        });
    }

    #[test]
    fn rotate_sfid_keys_works_with_backup_operator() {
        new_test_ext().execute_with(|| {
            assert_ok!(SfidCodeAuth::rotate_sfid_keys(
                RuntimeOrigin::signed(11),
                13
            ));
            assert_eq!(SfidCodeAuth::sfid_main_account(), Some(11));
            assert_eq!(SfidCodeAuth::sfid_backup_account_1(), Some(12));
            assert_eq!(SfidCodeAuth::sfid_backup_account_2(), Some(13));
        });
    }

    #[test]
    fn bind_validation_errors_are_enforced() {
        new_test_ext().execute_with(|| {
            let empty_sfid: SfidOf<Test> = Vec::<u8>::new().try_into().expect("bounded");
            assert_noop!(
                SfidCodeAuth::bind_sfid(
                    RuntimeOrigin::signed(1),
                    empty_sfid,
                    credential("sfid-a", "n-a", "bind-ok")
                ),
                Error::<Test>::EmptySfid
            );

            let empty_nonce_cred = BindCredential {
                sfid_code_hash: <Test as frame_system::Config>::Hashing::hash(b"sfid-a"),
                nonce: Vec::<u8>::new().try_into().expect("bounded"),
                expires_at: System::block_number().saturating_add(10),
                signature: signature("bind-ok"),
            };
            assert_noop!(
                SfidCodeAuth::bind_sfid(RuntimeOrigin::signed(1), sfid("sfid-a"), empty_nonce_cred),
                Error::<Test>::EmptyCredentialNonce
            );

            assert_noop!(
                SfidCodeAuth::bind_sfid(
                    RuntimeOrigin::signed(1),
                    sfid("sfid-a"),
                    credential("sfid-b", "n-a", "bind-ok")
                ),
                Error::<Test>::InvalidCredentialSfidCodeHash
            );

            assert_noop!(
                SfidCodeAuth::bind_sfid(
                    RuntimeOrigin::signed(1),
                    sfid("sfid-a"),
                    credential("sfid-a", "n-a", "bad-signature")
                ),
                Error::<Test>::InvalidSfidCredentialSignature
            );
        });
    }

    #[test]
    fn expired_bind_credential_is_rejected() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                SfidCodeAuth::bind_sfid(
                    RuntimeOrigin::signed(1),
                    sfid("sfid-expired"),
                    credential_with_expiry(
                        "sfid-expired",
                        "nonce-expired",
                        "bind-ok",
                        System::block_number().saturating_sub(1)
                    )
                ),
                Error::<Test>::CredentialExpired
            );
        });
    }

    #[test]
    fn bind_credential_with_too_far_expiry_is_rejected() {
        new_test_ext().execute_with(|| {
            let expires_at = System::block_number()
                .saturating_add(MaxBindCredentialLifetimeBlocks::get())
                .saturating_add(1);
            assert_noop!(
                SfidCodeAuth::bind_sfid(
                    RuntimeOrigin::signed(1),
                    sfid("sfid-far"),
                    credential_with_expiry("sfid-far", "nonce-far", "bind-ok", expires_at)
                ),
                Error::<Test>::CredentialExpiryTooFar
            );
        });
    }

    #[test]
    fn used_bind_nonce_is_cleaned_when_expired() {
        new_test_ext().execute_with(|| {
            assert_ok!(SfidCodeAuth::bind_sfid(
                RuntimeOrigin::signed(1),
                sfid("sfid-cleanup-a"),
                credential_with_expiry("sfid-cleanup-a", "same-nonce", "bind-ok", 5)
            ));
            let nonce_hash = <Test as frame_system::Config>::Hashing::hash(b"same-nonce");
            assert!(UsedCredentialNonce::<Test>::contains_key(nonce_hash));

            <Pallet<Test> as Hooks<BlockNumberFor<Test>>>::on_initialize(5);
            assert!(!UsedCredentialNonce::<Test>::contains_key(nonce_hash));

            assert_ok!(SfidCodeAuth::bind_sfid(
                RuntimeOrigin::signed(2),
                sfid("sfid-cleanup-b"),
                credential_with_expiry("sfid-cleanup-b", "same-nonce", "bind-ok", 8)
            ));
        });
    }

    #[test]
    fn cleanup_cursor_continues_expired_bucket_across_blocks() {
        new_test_ext().execute_with(|| {
            let expires_at = 5;
            let max_cleanup = MaxCredentialNonceCleanupPerBlock::get() as usize;
            let total = max_cleanup.saturating_add(5);
            let nonce_hashes = bind_many_with_expiry(100, total, "cursor", expires_at);

            assert_eq!(
                CredentialNoncesByExpiry::<Test>::get(expires_at).len(),
                total
            );
            assert_eq!(count_used_nonce_hashes(&nonce_hashes), total);

            <Pallet<Test> as Hooks<BlockNumberFor<Test>>>::on_initialize(expires_at);
            assert_eq!(
                CredentialNoncesByExpiry::<Test>::get(expires_at).len(),
                total - max_cleanup
            );
            assert_eq!(
                PendingCredentialNonceCleanupExpiry::<Test>::get(),
                Some(expires_at)
            );
            assert_eq!(count_used_nonce_hashes(&nonce_hashes), total - max_cleanup);

            <Pallet<Test> as Hooks<BlockNumberFor<Test>>>::on_initialize(expires_at + 1);
            assert!(CredentialNoncesByExpiry::<Test>::get(expires_at).is_empty());
            assert!(PendingCredentialNonceCleanupExpiry::<Test>::get().is_none());
            assert_eq!(count_used_nonce_hashes(&nonce_hashes), 0);
        });
    }

    #[test]
    fn cleanup_cursor_clears_pending_then_current_expiry_when_budget_remains() {
        new_test_ext().execute_with(|| {
            let max_cleanup = MaxCredentialNonceCleanupPerBlock::get() as usize;
            let pending_expiry = 5;
            let current_expiry = 6;

            let pending_hashes = bind_many_with_expiry(
                1_000,
                max_cleanup.saturating_add(1),
                "pending",
                pending_expiry,
            );
            let current_hashes = bind_many_with_expiry(10_000, 3, "current", current_expiry);

            <Pallet<Test> as Hooks<BlockNumberFor<Test>>>::on_initialize(pending_expiry);
            assert_eq!(
                CredentialNoncesByExpiry::<Test>::get(pending_expiry).len(),
                1
            );
            assert_eq!(
                PendingCredentialNonceCleanupExpiry::<Test>::get(),
                Some(pending_expiry)
            );
            assert_eq!(
                CredentialNoncesByExpiry::<Test>::get(current_expiry).len(),
                3
            );
            assert_eq!(count_used_nonce_hashes(&pending_hashes), 1);
            assert_eq!(count_used_nonce_hashes(&current_hashes), 3);

            <Pallet<Test> as Hooks<BlockNumberFor<Test>>>::on_initialize(current_expiry);
            assert!(CredentialNoncesByExpiry::<Test>::get(pending_expiry).is_empty());
            assert!(CredentialNoncesByExpiry::<Test>::get(current_expiry).is_empty());
            assert!(PendingCredentialNonceCleanupExpiry::<Test>::get().is_none());
            assert_eq!(count_used_nonce_hashes(&pending_hashes), 0);
            assert_eq!(count_used_nonce_hashes(&current_hashes), 0);
        });
    }

    #[test]
    fn same_account_rebind_replaces_old_sfid_without_changing_bound_count() {
        new_test_ext().execute_with(|| {
            assert_ok!(SfidCodeAuth::bind_sfid(
                RuntimeOrigin::signed(1),
                sfid("sfid-old"),
                credential("sfid-old", "n-1", "bind-ok")
            ));
            assert_eq!(BoundCount::<Test>::get(), 1);

            let old_hash = <Test as frame_system::Config>::Hashing::hash(b"sfid-old");
            assert_ok!(SfidCodeAuth::bind_sfid(
                RuntimeOrigin::signed(1),
                sfid("sfid-new"),
                credential("sfid-new", "n-2", "bind-ok")
            ));
            assert_eq!(BoundCount::<Test>::get(), 1);
            assert!(SfidToAccount::<Test>::get(old_hash).is_none());

            let new_hash = <Test as frame_system::Config>::Hashing::hash(b"sfid-new");
            assert_eq!(SfidToAccount::<Test>::get(new_hash), Some(1));
            assert_eq!(AccountToSfid::<Test>::get(1), Some(new_hash));
        });
    }

    #[test]
    fn bind_same_sfid_by_same_account_is_rejected() {
        new_test_ext().execute_with(|| {
            assert_ok!(SfidCodeAuth::bind_sfid(
                RuntimeOrigin::signed(1),
                sfid("sfid-same-self"),
                credential("sfid-same-self", "n-1", "bind-ok")
            ));
            assert_noop!(
                SfidCodeAuth::bind_sfid(
                    RuntimeOrigin::signed(1),
                    sfid("sfid-same-self"),
                    credential("sfid-same-self", "n-2", "bind-ok")
                ),
                Error::<Test>::SameSfidAlreadyBound
            );
        });
    }

    #[test]
    fn unbind_requires_bound_and_decreases_count() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                SfidCodeAuth::unbind_sfid(RuntimeOrigin::signed(1)),
                Error::<Test>::NotBound
            );

            assert_ok!(SfidCodeAuth::bind_sfid(
                RuntimeOrigin::signed(1),
                sfid("sfid-unbind"),
                credential("sfid-unbind", "n-1", "bind-ok")
            ));
            assert_eq!(BoundCount::<Test>::get(), 1);
            assert_ok!(SfidCodeAuth::unbind_sfid(RuntimeOrigin::signed(1)));
            assert_eq!(BoundCount::<Test>::get(), 0);
            assert!(!SfidCodeAuth::is_bound(&1));
        });
    }

    #[test]
    fn rotate_sfid_keys_requires_backup_and_unique_keys() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                SfidCodeAuth::rotate_sfid_keys(RuntimeOrigin::signed(10), 13),
                Error::<Test>::UnauthorizedSfidOperator
            );
            assert_noop!(
                SfidCodeAuth::rotate_sfid_keys(RuntimeOrigin::signed(99), 13),
                Error::<Test>::UnauthorizedSfidOperator
            );
            assert_noop!(
                SfidCodeAuth::rotate_sfid_keys(RuntimeOrigin::signed(11), 10),
                Error::<Test>::DuplicateSfidKey
            );
            assert_noop!(
                SfidCodeAuth::rotate_sfid_keys(RuntimeOrigin::signed(11), 11),
                Error::<Test>::DuplicateSfidKey
            );
            assert_noop!(
                SfidCodeAuth::rotate_sfid_keys(RuntimeOrigin::signed(11), 12),
                Error::<Test>::DuplicateSfidKey
            );
        });
    }

    #[test]
    fn eligibility_and_vote_credential_validation_paths() {
        new_test_ext().execute_with(|| {
            assert_ok!(SfidCodeAuth::bind_sfid(
                RuntimeOrigin::signed(1),
                sfid("sfid-v"),
                credential("sfid-v", "bind-n", "bind-ok")
            ));
            let sfid_hash = <Test as frame_system::Config>::Hashing::hash(b"sfid-v");
            assert!(<Pallet<Test> as SfidEligibilityProvider<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::is_eligible(&sfid_hash, &1));
            assert!(!<Pallet<Test> as SfidEligibilityProvider<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::is_eligible(&sfid_hash, &2));

            assert!(!<Pallet<Test> as SfidEligibilityProvider<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::verify_and_consume_vote_credential(
                &sfid_hash, &1, 1, b"", b"vote-ok"
            ));
            assert!(!<Pallet<Test> as SfidEligibilityProvider<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::verify_and_consume_vote_credential(
                &sfid_hash, &1, 1, b"nonce", b""
            ));
            assert!(!<Pallet<Test> as SfidEligibilityProvider<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::verify_and_consume_vote_credential(
                &sfid_hash, &2, 1, b"nonce", b"vote-ok"
            ));
            assert!(!<Pallet<Test> as SfidEligibilityProvider<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::verify_and_consume_vote_credential(
                &sfid_hash, &1, 1, b"nonce", b"bad"
            ));
        });
    }

    #[test]
    fn current_sfid_verify_pubkey_reads_main_account_encoding() {
        new_test_ext().execute_with(|| {
            assert!(SfidCodeAuth::current_sfid_verify_pubkey().is_none());
        });
    }
}
