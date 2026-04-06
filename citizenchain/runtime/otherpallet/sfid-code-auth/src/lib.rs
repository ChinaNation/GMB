//! # SFID 绑定与资格校验模块 (sfid-code-auth)
//!
//! 本模块负责三件核心事：
//! - SFID 与链上账户的一对一绑定 / 解绑。
//! - 公民投票资格校验（基于 SFID 绑定关系 + SFID 系统签名凭证）。
//! - 维护 SFID 验签主备账户（主账户验签、备用账户轮换）。
//!
//! 设计边界：
//! - 不保存 SFID 明文，只保存 `binding_id`。
//! - 绑定成功后的奖励发行通过 `OnSfidBound` 回调给上游模块处理。
//! - 投票凭证校验返回 `bool`，不抛 dispatch 错误，不污染治理模块语义。

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
/// 中文注释：绑定凭证结构体，封装 binding_id、一次性 nonce 和 SFID 系统签名。
pub struct BindCredential<Hash, Nonce, Signature> {
    pub binding_id: Hash,
    pub bind_nonce: Nonce,
    pub signature: Signature,
}

/// 中文注释：SFID 系统绑定验签接口，由 Runtime 注入具体实现（sr25519 验签桥接）。
pub trait SfidVerifier<AccountId, Hash, Nonce, Signature> {
    fn verify(account: &AccountId, credential: &BindCredential<Hash, Nonce, Signature>) -> bool;
}

impl<AccountId, Hash, Nonce, Signature> SfidVerifier<AccountId, Hash, Nonce, Signature> for () {
    fn verify(_account: &AccountId, _credential: &BindCredential<Hash, Nonce, Signature>) -> bool {
        false
    }
}

/// 中文注释：公民投票实时验签接口，绑定身份标识与提案 ID 必须一并进入签名载荷。
pub trait SfidVoteVerifier<AccountId, Hash, Nonce, Signature> {
    fn verify_vote(
        account: &AccountId,
        binding_id: Hash,
        proposal_id: u64,
        nonce: &Nonce,
        signature: &Signature,
    ) -> bool;
}

impl<AccountId, Hash, Nonce, Signature> SfidVoteVerifier<AccountId, Hash, Nonce, Signature> for () {
    fn verify_vote(
        _account: &AccountId,
        _binding_id: Hash,
        _proposal_id: u64,
        _nonce: &Nonce,
        _signature: &Signature,
    ) -> bool {
        false
    }
}

/// 中文注释：绑定成功后的钩子，用于让发行模块基于 binding_id 做一次性奖励判定。
pub trait OnSfidBound<AccountId, Hash> {
    fn on_sfid_bound(_who: &AccountId, _binding_id: Hash) {}
}

impl<AccountId, Hash> OnSfidBound<AccountId, Hash> for () {}

/// 中文注释：回调 weight 声明接口，供 bind_sfid 在申报 weight 时叠加回调预算。
pub trait OnSfidBoundWeight {
    fn on_sfid_bound_weight() -> Weight {
        Weight::zero()
    }
}

impl OnSfidBoundWeight for () {}

/// 中文注释：给投票模块使用的统一资格接口。
pub trait SfidEligibilityProvider<AccountId, Hash> {
    fn is_eligible(binding_id: &Hash, who: &AccountId) -> bool;
    fn verify_and_consume_vote_credential(
        binding_id: &Hash,
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
    use sp_runtime::traits::Hash;

    pub type NonceOf<T> = BoundedVec<u8, <T as Config>::MaxCredentialNonceLength>;
    pub type SignatureOf<T> = BoundedVec<u8, <T as Config>::MaxCredentialSignatureLength>;
    pub type CredentialOf<T> =
        BindCredential<<T as frame_system::Config>::Hash, NonceOf<T>, SignatureOf<T>>;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        #[pallet::constant]
        type MaxCredentialNonceLength: Get<u32>;

        #[pallet::constant]
        type MaxCredentialSignatureLength: Get<u32>;

        /// 中文注释：SFID 系统绑定验签器（外部接口桥接点）。
        type SfidVerifier: SfidVerifier<
            Self::AccountId,
            Self::Hash,
            NonceOf<Self>,
            SignatureOf<Self>,
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

    /// 中文注释：binding_id 到账户的正向映射，保证同一 binding_id 只能绑定一个账户。
    #[pallet::storage]
    #[pallet::getter(fn binding_id_to_account)]
    pub type BindingIdToAccount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, T::AccountId, OptionQuery>;

    /// 中文注释：账户到 binding_id 的反向映射，用于快速查询账户当前绑定的身份标识。
    #[pallet::storage]
    #[pallet::getter(fn account_to_binding_id)]
    pub type AccountToBindingId<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, T::Hash, OptionQuery>;

    /// 中文注释：当前已绑定身份的账户数量，可用于公民投票基数。
    #[pallet::storage]
    #[pallet::getter(fn bound_count)]
    pub type BoundCount<T> = StorageValue<_, u64, ValueQuery>;

    /// 中文注释：已消费的绑定 nonce，防止同一条绑定消息重放。
    #[pallet::storage]
    #[pallet::getter(fn used_bind_nonce)]
    pub type UsedBindNonce<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, bool, ValueQuery>;

    /// 中文注释：公民投票验签 nonce（提案 + binding_id + nonce 三元维度）防重放。
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

    /// 中文注释：SFID 当前主账户（用于绑定、投票与人口快照验签）。
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
            if self.sfid_main_account.is_none()
                && self.sfid_backup_account_1.is_none()
                && self.sfid_backup_account_2.is_none()
            {
                return;
            }

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
        /// 中文注释：SFID 绑定成功，记录账户、binding_id 和 nonce 哈希。
        SfidBound {
            who: T::AccountId,
            binding_id: T::Hash,
            bind_nonce_hash: T::Hash,
        },
        /// 中文注释：账户主动解绑 SFID。
        SfidUnbound {
            who: T::AccountId,
            binding_id: T::Hash,
        },
        /// 中文注释：SFID 验签密钥轮换完成，记录操作者和新的三把账户。
        SfidKeysRotated {
            operator: T::AccountId,
            new_main: T::AccountId,
            backup_1: T::AccountId,
            backup_2: T::AccountId,
        },
    }

    /// 中文注释：本模块无需 on_initialize / on_finalize 钩子，所有逻辑由 extrinsic 或内部接口驱动。
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::error]
    pub enum Error<T> {
        /// 中文注释：绑定凭证中 bind_nonce 为空。
        EmptyBindNonce,
        /// 中文注释：该 bind_nonce 已被使用（防重放）。
        BindNonceAlreadyUsed,
        /// 中文注释：SFID 绑定签名验证失败。
        InvalidSfidBindingSignature,
        /// 中文注释：该 binding_id 已被另一个账户绑定。
        BindingIdAlreadyBoundToAnotherAccount,
        /// 中文注释：该账户已绑定到同一 binding_id，无需重复操作。
        SameBindingIdAlreadyBound,
        /// 中文注释：账户当前未绑定 SFID，无法解绑。
        NotBound,
        /// 中文注释：调用者不是 SFID 备用账户，无权发起轮换。
        UnauthorizedSfidOperator,
        /// 中文注释：新备用账户与现有三把账户之一重复。
        DuplicateSfidKey,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 中文注释：使用 SFID 系统签发的绑定消息把钱包和 binding_id 绑定。
        #[pallet::call_index(0)]
        #[pallet::weight(
            T::WeightInfo::bind_sfid()
                .saturating_add(T::OnSfidBound::on_sfid_bound_weight())
        )]
        pub fn bind_sfid(origin: OriginFor<T>, credential: CredentialOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                !credential.bind_nonce.is_empty(),
                Error::<T>::EmptyBindNonce
            );

            let bind_nonce_hash = T::Hashing::hash(credential.bind_nonce.as_slice());
            ensure!(
                !UsedBindNonce::<T>::get(bind_nonce_hash),
                Error::<T>::BindNonceAlreadyUsed
            );
            ensure!(
                T::SfidVerifier::verify(&who, &credential),
                Error::<T>::InvalidSfidBindingSignature
            );

            let binding_id = credential.binding_id;
            if let Some(existing_owner) = BindingIdToAccount::<T>::get(binding_id) {
                ensure!(
                    existing_owner == who,
                    Error::<T>::BindingIdAlreadyBoundToAnotherAccount
                );
                return Err(Error::<T>::SameBindingIdAlreadyBound.into());
            }

            // 中文注释：账户允许换绑到新的 binding_id，但只释放旧映射，不减少当前绑定人数。
            if let Some(old_binding_id) = AccountToBindingId::<T>::get(&who) {
                BindingIdToAccount::<T>::remove(old_binding_id);
            } else {
                BoundCount::<T>::mutate(|v| *v = v.saturating_add(1));
            }

            BindingIdToAccount::<T>::insert(binding_id, &who);
            AccountToBindingId::<T>::insert(&who, binding_id);
            UsedBindNonce::<T>::insert(bind_nonce_hash, true);

            T::OnSfidBound::on_sfid_bound(&who, binding_id);

            Self::deposit_event(Event::<T>::SfidBound {
                who,
                binding_id,
                bind_nonce_hash,
            });
            Ok(())
        }

        /// 中文注释：主动解绑当前账户的 SFID 绑定关系，释放双向映射并减少计数。
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::unbind_sfid())]
        pub fn unbind_sfid(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let binding_id = AccountToBindingId::<T>::get(&who).ok_or(Error::<T>::NotBound)?;

            AccountToBindingId::<T>::remove(&who);
            BindingIdToAccount::<T>::remove(binding_id);
            BoundCount::<T>::mutate(|v| *v = v.saturating_sub(1));

            Self::deposit_event(Event::<T>::SfidUnbound { who, binding_id });
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
        /// 中文注释：查询账户是否已绑定 SFID。
        pub fn is_bound(who: &T::AccountId) -> bool {
            AccountToBindingId::<T>::contains_key(who)
        }

        /// 中文注释：查询指定 binding_id 是否绑定到指定账户。
        pub fn is_binding_id_bound_to(binding_id: &T::Hash, who: &T::AccountId) -> bool {
            BindingIdToAccount::<T>::get(binding_id)
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

    /// 中文注释：实现投票资格接口，供治理模块统一判断公民身份和消费投票凭证。
    impl<T: Config> crate::SfidEligibilityProvider<T::AccountId, T::Hash> for Pallet<T> {
        fn is_eligible(binding_id: &T::Hash, who: &T::AccountId) -> bool {
            Self::is_binding_id_bound_to(binding_id, who)
        }

        fn verify_and_consume_vote_credential(
            binding_id: &T::Hash,
            who: &T::AccountId,
            proposal_id: u64,
            nonce: &[u8],
            signature: &[u8],
        ) -> bool {
            if nonce.is_empty() || signature.is_empty() {
                return false;
            }

            if !Self::is_binding_id_bound_to(binding_id, who) {
                return false;
            }

            let nonce_hash = T::Hashing::hash(nonce);
            let vote_nonce_key = (binding_id.clone(), nonce_hash);
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
                binding_id.clone(),
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
        pub type SfidCodeAuth = super;
    }

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl system::Config for Test {
        type Block = Block;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
    }

    pub struct TestSfidVerifier;
    impl SfidVerifier<u64, <Test as frame_system::Config>::Hash, NonceOf<Test>, SignatureOf<Test>>
        for TestSfidVerifier
    {
        fn verify(_account: &u64, credential: &CredentialOf<Test>) -> bool {
            !credential.bind_nonce.is_empty() && credential.signature.as_slice() == b"bind-ok"
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
            _binding_id: <Test as frame_system::Config>::Hash,
            _proposal_id: u64,
            _nonce: &NonceOf<Test>,
            signature: &SignatureOf<Test>,
        ) -> bool {
            signature.as_slice() == b"vote-ok"
        }
    }

    parameter_types! {
        pub const MaxCredentialNonceLength: u32 = 64;
        pub const MaxCredentialSignatureLength: u32 = 64;
    }

    impl Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type MaxCredentialNonceLength = MaxCredentialNonceLength;
        type MaxCredentialSignatureLength = MaxCredentialSignatureLength;
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

    fn binding_id(seed: &[u8]) -> <Test as frame_system::Config>::Hash {
        <Test as frame_system::Config>::Hashing::hash(seed)
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

    fn bind_credential(seed: &[u8], bind_nonce: &str, sig: &str) -> CredentialOf<Test> {
        BindCredential {
            binding_id: binding_id(seed),
            bind_nonce: nonce(bind_nonce),
            signature: signature(sig),
        }
    }

    #[test]
    fn bind_succeeds_and_tracks_binding_id() {
        new_test_ext().execute_with(|| {
            let credential = bind_credential(b"binding-a", "nonce-a", "bind-ok");

            assert_ok!(SfidCodeAuth::bind_sfid(
                RuntimeOrigin::signed(1),
                credential.clone()
            ));

            assert_eq!(
                BindingIdToAccount::<Test>::get(credential.binding_id),
                Some(1)
            );
            assert_eq!(
                AccountToBindingId::<Test>::get(1),
                Some(credential.binding_id)
            );
            assert_eq!(BoundCount::<Test>::get(), 1);
        });
    }

    #[test]
    fn bind_rejects_reused_bind_nonce() {
        new_test_ext().execute_with(|| {
            let first = bind_credential(b"binding-a", "same-nonce", "bind-ok");
            let second = bind_credential(b"binding-b", "same-nonce", "bind-ok");

            assert_ok!(SfidCodeAuth::bind_sfid(RuntimeOrigin::signed(1), first));
            assert_noop!(
                SfidCodeAuth::bind_sfid(RuntimeOrigin::signed(2), second),
                Error::<Test>::BindNonceAlreadyUsed
            );
        });
    }

    #[test]
    fn bind_allows_account_rebinding_to_new_binding_id() {
        new_test_ext().execute_with(|| {
            let first = bind_credential(b"binding-a", "nonce-a", "bind-ok");
            let second = bind_credential(b"binding-b", "nonce-b", "bind-ok");

            assert_ok!(SfidCodeAuth::bind_sfid(
                RuntimeOrigin::signed(1),
                first.clone()
            ));
            assert_ok!(SfidCodeAuth::bind_sfid(
                RuntimeOrigin::signed(1),
                second.clone()
            ));

            assert!(BindingIdToAccount::<Test>::get(first.binding_id).is_none());
            assert_eq!(BindingIdToAccount::<Test>::get(second.binding_id), Some(1));
            assert_eq!(AccountToBindingId::<Test>::get(1), Some(second.binding_id));
            assert_eq!(BoundCount::<Test>::get(), 1);
        });
    }

    #[test]
    fn vote_credential_is_consumed_once_per_proposal_and_binding_id() {
        new_test_ext().execute_with(|| {
            let credential = bind_credential(b"binding-vote", "bind-nonce", "bind-ok");
            let binding_id = credential.binding_id;
            assert_ok!(SfidCodeAuth::bind_sfid(
                RuntimeOrigin::signed(1),
                credential
            ));

            assert!(<Pallet<Test> as SfidEligibilityProvider<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::is_eligible(&binding_id, &1));
            assert!(<Pallet<Test> as SfidEligibilityProvider<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::verify_and_consume_vote_credential(
                &binding_id,
                &1,
                7,
                b"vote-nonce",
                b"vote-ok"
            ));
            assert!(!<Pallet<Test> as SfidEligibilityProvider<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::verify_and_consume_vote_credential(
                &binding_id,
                &1,
                7,
                b"vote-nonce",
                b"vote-ok"
            ));
        });
    }

    #[test]
    fn vote_nonce_is_scoped_per_proposal_and_cannot_replay_within_same_proposal() {
        new_test_ext().execute_with(|| {
            let credential = bind_credential(b"binding-replay", "bind-nonce", "bind-ok");
            let binding_id = credential.binding_id;
            assert_ok!(SfidCodeAuth::bind_sfid(
                RuntimeOrigin::signed(1),
                credential
            ));

            let proposal_a = 10u64;
            let proposal_b = 20u64;

            // 提案 A 投票成功
            assert!(<Pallet<Test> as SfidEligibilityProvider<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::verify_and_consume_vote_credential(
                &binding_id, &1, proposal_a, b"same-nonce", b"vote-ok"
            ));

            // 同一 nonce 对同一提案重放 → 失败
            assert!(!<Pallet<Test> as SfidEligibilityProvider<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::verify_and_consume_vote_credential(
                &binding_id, &1, proposal_a, b"same-nonce", b"vote-ok"
            ));

            // 同一 nonce 对不同提案 → 成功（nonce 按 proposal_id 分区存储，
            // 生产环境中签名包含 proposal_id 所以无法跨提案重放签名）
            assert!(<Pallet<Test> as SfidEligibilityProvider<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::verify_and_consume_vote_credential(
                &binding_id, &1, proposal_b, b"same-nonce", b"vote-ok"
            ));
        });
    }

    #[test]
    fn rotate_sfid_keys_keeps_two_distinct_backups() {
        new_test_ext().execute_with(|| {
            assert_ok!(SfidCodeAuth::rotate_sfid_keys(
                RuntimeOrigin::signed(11),
                20
            ));
            assert_eq!(SfidMainAccount::<Test>::get(), Some(11));
            assert_eq!(SfidBackupAccount1::<Test>::get(), Some(12));
            assert_eq!(SfidBackupAccount2::<Test>::get(), Some(20));
        });
    }

    #[test]
    fn current_sfid_verify_pubkey_reads_main_account_encoding() {
        new_test_ext().execute_with(|| {
            assert!(SfidCodeAuth::current_sfid_verify_pubkey().is_none());
        });
    }

    // ========================================================================
    // 以下为补充的错误路径和边界测试
    // ========================================================================

    #[test]
    fn bind_rejects_empty_nonce() {
        new_test_ext().execute_with(|| {
            let empty_credential = BindCredential {
                binding_id: binding_id(b"id-empty"),
                bind_nonce: Vec::<u8>::new().try_into().expect("empty vec fits"),
                signature: signature("bind-ok"),
            };
            assert_noop!(
                SfidCodeAuth::bind_sfid(RuntimeOrigin::signed(1), empty_credential),
                Error::<Test>::EmptyBindNonce
            );
        });
    }

    #[test]
    fn bind_rejects_invalid_signature() {
        new_test_ext().execute_with(|| {
            let credential = bind_credential(b"id-badsig", "nonce-badsig", "bad-sig");
            assert_noop!(
                SfidCodeAuth::bind_sfid(RuntimeOrigin::signed(1), credential),
                Error::<Test>::InvalidSfidBindingSignature
            );
        });
    }

    #[test]
    fn bind_rejects_binding_id_owned_by_another_account() {
        new_test_ext().execute_with(|| {
            let credential_1 = bind_credential(b"shared-id", "nonce-1", "bind-ok");
            let credential_2 = bind_credential(b"shared-id", "nonce-2", "bind-ok");

            assert_ok!(SfidCodeAuth::bind_sfid(
                RuntimeOrigin::signed(1),
                credential_1
            ));
            assert_noop!(
                SfidCodeAuth::bind_sfid(RuntimeOrigin::signed(2), credential_2),
                Error::<Test>::BindingIdAlreadyBoundToAnotherAccount
            );
        });
    }

    #[test]
    fn bind_rejects_same_binding_id_already_bound() {
        new_test_ext().execute_with(|| {
            let credential_1 = bind_credential(b"dup-id", "nonce-dup-1", "bind-ok");
            let credential_2 = bind_credential(b"dup-id", "nonce-dup-2", "bind-ok");

            assert_ok!(SfidCodeAuth::bind_sfid(
                RuntimeOrigin::signed(1),
                credential_1
            ));
            assert_noop!(
                SfidCodeAuth::bind_sfid(RuntimeOrigin::signed(1), credential_2),
                Error::<Test>::SameBindingIdAlreadyBound
            );
        });
    }

    #[test]
    fn unbind_rejects_unbound_account() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                SfidCodeAuth::unbind_sfid(RuntimeOrigin::signed(99)),
                Error::<Test>::NotBound
            );
        });
    }

    #[test]
    fn rotate_rejects_main_account_as_operator() {
        new_test_ext().execute_with(|| {
            // main = 10, 主账户不能直接发起轮换
            assert_noop!(
                SfidCodeAuth::rotate_sfid_keys(RuntimeOrigin::signed(10), 20),
                Error::<Test>::UnauthorizedSfidOperator
            );
        });
    }

    #[test]
    fn rotate_from_backup_2_succeeds() {
        new_test_ext().execute_with(|| {
            // backup_2 = 12 发起轮换
            assert_ok!(SfidCodeAuth::rotate_sfid_keys(
                RuntimeOrigin::signed(12),
                20
            ));
            assert_eq!(SfidMainAccount::<Test>::get(), Some(12));
            assert_eq!(SfidBackupAccount1::<Test>::get(), Some(11));
            assert_eq!(SfidBackupAccount2::<Test>::get(), Some(20));
        });
    }

    #[test]
    fn rotate_rejects_duplicate_new_backup() {
        new_test_ext().execute_with(|| {
            // new_backup == main (10)
            assert_noop!(
                SfidCodeAuth::rotate_sfid_keys(RuntimeOrigin::signed(11), 10),
                Error::<Test>::DuplicateSfidKey
            );
            // new_backup == caller (11)
            assert_noop!(
                SfidCodeAuth::rotate_sfid_keys(RuntimeOrigin::signed(11), 11),
                Error::<Test>::DuplicateSfidKey
            );
            // new_backup == survivor (12)
            assert_noop!(
                SfidCodeAuth::rotate_sfid_keys(RuntimeOrigin::signed(11), 12),
                Error::<Test>::DuplicateSfidKey
            );
        });
    }

    #[test]
    fn cleanup_vote_credentials_removes_nonces() {
        new_test_ext().execute_with(|| {
            let credential = bind_credential(b"binding-cleanup", "nonce-cleanup", "bind-ok");
            let bid = credential.binding_id;
            assert_ok!(SfidCodeAuth::bind_sfid(
                RuntimeOrigin::signed(1),
                credential
            ));

            // 消费一个投票 nonce
            assert!(<Pallet<Test> as SfidEligibilityProvider<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::verify_and_consume_vote_credential(
                &bid, &1, 42, b"vote-nonce-c", b"vote-ok"
            ));

            // 清理提案 42 的 nonce
            <Pallet<Test> as SfidEligibilityProvider<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::cleanup_vote_credentials(42);

            // 同一 nonce 应该可以再次使用（已被清理）
            assert!(<Pallet<Test> as SfidEligibilityProvider<
                u64,
                <Test as frame_system::Config>::Hash,
            >>::verify_and_consume_vote_credential(
                &bid, &1, 42, b"vote-nonce-c", b"vote-ok"
            ));
        });
    }
}
