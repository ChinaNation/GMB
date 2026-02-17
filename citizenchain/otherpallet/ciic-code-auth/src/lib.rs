#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
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
pub struct BindCredential<Hash, Nonce, Signature> {
    pub ciic_code_hash: Hash,
    pub nonce: Nonce,
    pub signature: Signature,
}

pub trait CiicVerifier<AccountId, Hash, Nonce, Signature> {
    fn verify(account: &AccountId, credential: &BindCredential<Hash, Nonce, Signature>) -> bool;
}

impl<AccountId, Hash, Nonce, Signature> CiicVerifier<AccountId, Hash, Nonce, Signature> for () {
    fn verify(_account: &AccountId, _credential: &BindCredential<Hash, Nonce, Signature>) -> bool {
        false
    }
}

/// 中文注释：公民投票实时验签接口（包含 proposal_id 与 nonce）。
pub trait CiicVoteVerifier<AccountId, Hash, Nonce, Signature> {
    fn verify_vote(
        account: &AccountId,
        ciic_hash: Hash,
        proposal_id: u64,
        nonce: &Nonce,
        signature: &Signature,
    ) -> bool;
}

impl<AccountId, Hash, Nonce, Signature> CiicVoteVerifier<AccountId, Hash, Nonce, Signature> for () {
    fn verify_vote(
        _account: &AccountId,
        _ciic_hash: Hash,
        _proposal_id: u64,
        _nonce: &Nonce,
        _signature: &Signature,
    ) -> bool {
        false
    }
}

/// 中文注释：绑定成功后的钩子，用于让“发行模块”只做发行逻辑。
pub trait OnCiicBound<AccountId, Hash> {
    fn on_ciic_bound(_who: &AccountId, _ciic_hash: Hash) {}
}

impl<AccountId, Hash> OnCiicBound<AccountId, Hash> for () {}

/// 中文注释：给投票模块使用的统一资格接口。
pub trait CiicEligibilityProvider<AccountId> {
    fn is_eligible(ciic: &[u8], who: &AccountId) -> bool;
    fn verify_and_consume_vote_credential(
        ciic: &[u8],
        who: &AccountId,
        proposal_id: u64,
        nonce: &[u8],
        signature: &[u8],
    ) -> bool;
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{pallet_prelude::*, Blake2_128Concat};
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::Hash;

    pub type CiicOf<T> = BoundedVec<u8, <T as Config>::MaxCiicLength>;
    pub type NonceOf<T> = BoundedVec<u8, <T as Config>::MaxCredentialNonceLength>;
    pub type SignatureOf<T> = BoundedVec<u8, <T as Config>::MaxCredentialSignatureLength>;
    pub type CredentialOf<T> =
        BindCredential<<T as frame_system::Config>::Hash, NonceOf<T>, SignatureOf<T>>;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        #[pallet::constant]
        type MaxCiicLength: Get<u32>;

        #[pallet::constant]
        type MaxCredentialNonceLength: Get<u32>;

        #[pallet::constant]
        type MaxCredentialSignatureLength: Get<u32>;

        /// 中文注释：CIIC 系统签名校验器（外部接口桥接点）。
        type CiicVerifier: CiicVerifier<
            Self::AccountId,
            Self::Hash,
            NonceOf<Self>,
            SignatureOf<Self>,
        >;
        /// 中文注释：公民投票实时验签器。
        type CiicVoteVerifier: CiicVoteVerifier<
            Self::AccountId,
            Self::Hash,
            NonceOf<Self>,
            SignatureOf<Self>,
        >;

        /// 中文注释：绑定后回调到发行模块发放认证奖励。
        type OnCiicBound: OnCiicBound<Self::AccountId, Self::Hash>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn ciic_to_account)]
    pub type CiicToAccount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, T::AccountId, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn account_to_ciic)]
    pub type AccountToCiic<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, T::Hash, OptionQuery>;

    /// 中文注释：当前已绑定 CIIC 的账户数量，可用于公民投票基数。
    #[pallet::storage]
    #[pallet::getter(fn bound_count)]
    pub type BoundCount<T> = StorageValue<_, u64, ValueQuery>;

    /// 中文注释：已使用凭证 nonce（哈希）防重放。
    #[pallet::storage]
    #[pallet::getter(fn used_credential_nonce)]
    pub type UsedCredentialNonce<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, bool, ValueQuery>;

    /// 中文注释：公民投票验签 nonce（哈希）防重放（提案+身份+nonce 三元维度）。
    #[pallet::storage]
    #[pallet::getter(fn used_vote_nonce)]
    pub type UsedVoteNonce<T: Config> =
        StorageMap<_, Blake2_128Concat, (u64, T::Hash, T::Hash), bool, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        CiicBound {
            who: T::AccountId,
            ciic_hash: T::Hash,
            credential_nonce_hash: T::Hash,
        },
        CiicUnbound {
            who: T::AccountId,
            ciic_hash: T::Hash,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        EmptyCiic,
        EmptyCredentialNonce,
        InvalidCredentialCiicCodeHash,
        CredentialAlreadyUsed,
        InvalidCiicCredentialSignature,
        CiicAlreadyBoundToAnotherAccount,
        SameCiicAlreadyBound,
        NotBound,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 中文注释：使用 CIIC 系统签发的一次性凭证绑定钱包。
        #[pallet::call_index(0)]
        #[pallet::weight(T::DbWeight::get().reads_writes(6, 6))]
        pub fn bind_ciic(
            origin: OriginFor<T>,
            ciic_code: CiicOf<T>,
            credential: CredentialOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(!ciic_code.is_empty(), Error::<T>::EmptyCiic);
            ensure!(
                !credential.nonce.is_empty(),
                Error::<T>::EmptyCredentialNonce
            );

            let ciic_hash = T::Hashing::hash(ciic_code.as_slice());
            ensure!(
                credential.ciic_code_hash == ciic_hash,
                Error::<T>::InvalidCredentialCiicCodeHash
            );

            let nonce_hash = T::Hashing::hash(credential.nonce.as_slice());
            ensure!(
                !UsedCredentialNonce::<T>::get(nonce_hash),
                Error::<T>::CredentialAlreadyUsed
            );

            ensure!(
                T::CiicVerifier::verify(&who, &credential),
                Error::<T>::InvalidCiicCredentialSignature
            );

            if let Some(existing_owner) = CiicToAccount::<T>::get(ciic_hash) {
                ensure!(
                    existing_owner == who,
                    Error::<T>::CiicAlreadyBoundToAnotherAccount
                );
                return Err(Error::<T>::SameCiicAlreadyBound.into());
            }

            if let Some(old_ciic_hash) = AccountToCiic::<T>::get(&who) {
                CiicToAccount::<T>::remove(old_ciic_hash);
            } else {
                BoundCount::<T>::mutate(|v| *v = v.saturating_add(1));
            }

            CiicToAccount::<T>::insert(ciic_hash, &who);
            AccountToCiic::<T>::insert(&who, ciic_hash);
            UsedCredentialNonce::<T>::insert(nonce_hash, true);

            T::OnCiicBound::on_ciic_bound(&who, ciic_hash);

            Self::deposit_event(Event::<T>::CiicBound {
                who,
                ciic_hash,
                credential_nonce_hash: nonce_hash,
            });
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(T::DbWeight::get().reads_writes(2, 2))]
        pub fn unbind_ciic(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let ciic_hash = AccountToCiic::<T>::get(&who).ok_or(Error::<T>::NotBound)?;

            AccountToCiic::<T>::remove(&who);
            CiicToAccount::<T>::remove(ciic_hash);
            BoundCount::<T>::mutate(|v| *v = v.saturating_sub(1));

            Self::deposit_event(Event::<T>::CiicUnbound { who, ciic_hash });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn is_bound(who: &T::AccountId) -> bool {
            AccountToCiic::<T>::contains_key(who)
        }

        pub fn is_ciic_bound_to(ciic_hash: T::Hash, who: &T::AccountId) -> bool {
            CiicToAccount::<T>::get(ciic_hash)
                .map(|owner| owner == *who)
                .unwrap_or(false)
        }
    }

    impl<T: Config> crate::CiicEligibilityProvider<T::AccountId> for Pallet<T> {
        fn is_eligible(ciic: &[u8], who: &T::AccountId) -> bool {
            let ciic_hash = T::Hashing::hash(ciic);
            Self::is_ciic_bound_to(ciic_hash, who)
        }

        fn verify_and_consume_vote_credential(
            ciic: &[u8],
            who: &T::AccountId,
            proposal_id: u64,
            nonce: &[u8],
            signature: &[u8],
        ) -> bool {
            if nonce.is_empty() || signature.is_empty() {
                return false;
            }

            let ciic_hash = T::Hashing::hash(ciic);
            if !Self::is_ciic_bound_to(ciic_hash, who) {
                return false;
            }

            let nonce_hash = T::Hashing::hash(nonce);
            let vote_nonce_key = (proposal_id, ciic_hash, nonce_hash);
            if UsedVoteNonce::<T>::get(vote_nonce_key) {
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

            if !T::CiicVoteVerifier::verify_vote(
                who,
                ciic_hash,
                proposal_id,
                &nonce_bounded,
                &signature_bounded,
            ) {
                return false;
            }

            UsedVoteNonce::<T>::insert(vote_nonce_key, true);
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{assert_noop, assert_ok, derive_impl, parameter_types};
    use frame_system as system;
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
        pub type CiicCodeAuth = super;
    }

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl system::Config for Test {
        type Block = Block;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
    }

    pub struct TestCiicVerifier;
    impl CiicVerifier<u64, <Test as frame_system::Config>::Hash, NonceOf<Test>, SignatureOf<Test>>
        for TestCiicVerifier
    {
        fn verify(_account: &u64, credential: &CredentialOf<Test>) -> bool {
            credential.signature.as_slice() == b"bind-ok"
        }
    }

    pub struct TestCiicVoteVerifier;
    impl
        CiicVoteVerifier<
            u64,
            <Test as frame_system::Config>::Hash,
            NonceOf<Test>,
            SignatureOf<Test>,
        > for TestCiicVoteVerifier
    {
        fn verify_vote(
            _account: &u64,
            _ciic_hash: <Test as frame_system::Config>::Hash,
            _proposal_id: u64,
            _nonce: &NonceOf<Test>,
            signature: &SignatureOf<Test>,
        ) -> bool {
            signature.as_slice() == b"vote-ok"
        }
    }

    parameter_types! {
        pub const MaxCiicLength: u32 = 64;
        pub const MaxCredentialNonceLength: u32 = 64;
        pub const MaxCredentialSignatureLength: u32 = 64;
    }

    impl Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type MaxCiicLength = MaxCiicLength;
        type MaxCredentialNonceLength = MaxCredentialNonceLength;
        type MaxCredentialSignatureLength = MaxCredentialSignatureLength;
        type CiicVerifier = TestCiicVerifier;
        type CiicVoteVerifier = TestCiicVoteVerifier;
        type OnCiicBound = ();
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("frame system genesis storage should build");
        let mut ext = sp_io::TestExternalities::new(storage);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }

    fn ciic(input: &str) -> CiicOf<Test> {
        input
            .as_bytes()
            .to_vec()
            .try_into()
            .expect("ciic should fit")
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

    fn credential(ciic_plain: &str, nonce_plain: &str, sig_plain: &str) -> CredentialOf<Test> {
        BindCredential {
            ciic_code_hash: <Test as frame_system::Config>::Hashing::hash(ciic_plain.as_bytes()),
            nonce: nonce(nonce_plain),
            signature: signature(sig_plain),
        }
    }

    #[test]
    fn bind_ciic_works_and_tracks_binding_count() {
        new_test_ext().execute_with(|| {
            assert_ok!(CiicCodeAuth::bind_ciic(
                RuntimeOrigin::signed(1),
                ciic("ciic-a"),
                credential("ciic-a", "n-a", "bind-ok")
            ));
            assert_eq!(BoundCount::<Test>::get(), 1);
            assert!(CiicCodeAuth::is_bound(&1));
        });
    }

    #[test]
    fn reused_bind_nonce_is_rejected() {
        new_test_ext().execute_with(|| {
            assert_ok!(CiicCodeAuth::bind_ciic(
                RuntimeOrigin::signed(1),
                ciic("ciic-a"),
                credential("ciic-a", "same", "bind-ok")
            ));
            assert_noop!(
                CiicCodeAuth::bind_ciic(
                    RuntimeOrigin::signed(1),
                    ciic("ciic-b"),
                    credential("ciic-b", "same", "bind-ok")
                ),
                Error::<Test>::CredentialAlreadyUsed
            );
        });
    }

    #[test]
    fn same_ciic_cannot_bind_to_another_account() {
        new_test_ext().execute_with(|| {
            assert_ok!(CiicCodeAuth::bind_ciic(
                RuntimeOrigin::signed(1),
                ciic("ciic-same"),
                credential("ciic-same", "n-1", "bind-ok")
            ));
            assert_noop!(
                CiicCodeAuth::bind_ciic(
                    RuntimeOrigin::signed(2),
                    ciic("ciic-same"),
                    credential("ciic-same", "n-2", "bind-ok")
                ),
                Error::<Test>::CiicAlreadyBoundToAnotherAccount
            );
        });
    }

    #[test]
    fn vote_credential_nonce_replay_is_rejected_per_proposal() {
        new_test_ext().execute_with(|| {
            assert_ok!(CiicCodeAuth::bind_ciic(
                RuntimeOrigin::signed(1),
                ciic("ciic-vote"),
                credential("ciic-vote", "bind-n", "bind-ok")
            ));

            assert!(
                <Pallet<Test> as CiicEligibilityProvider<u64>>::verify_and_consume_vote_credential(
                    b"ciic-vote",
                    &1,
                    100,
                    b"vote-nonce",
                    b"vote-ok"
                )
            );
            assert!(
                !<Pallet<Test> as CiicEligibilityProvider<u64>>::verify_and_consume_vote_credential(
                    b"ciic-vote",
                    &1,
                    100,
                    b"vote-nonce",
                    b"vote-ok"
                )
            );
            assert!(
                <Pallet<Test> as CiicEligibilityProvider<u64>>::verify_and_consume_vote_credential(
                    b"ciic-vote",
                    &1,
                    101,
                    b"vote-nonce",
                    b"vote-ok"
                )
            );
        });
    }
}
