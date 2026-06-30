//! # CID 绑定与资格校验模块 (cid-system)
//!
//! 本模块只负责 CID 绑定、解绑和公民投票资格消费。凭证签发身份不再由
//! 本 pallet 维护特殊花名册，而是由 runtime 注入的验签器按
//! `issuer_main_account -> admins 模块::AdminAccounts[issuer_main_account].admins`
//! 判断 `signer_pubkey` 是否为签发机构管理员。

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub use pallet::*;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod weights;

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::ConstU32;
use frame_support::weights::Weight;
use frame_support::BoundedVec;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

/// 中文注释:签发机构 CID 号上限。实际 CID 长度由 primitives 约束;这里避免
/// cid-system 直接依赖业务常量,保持凭证容器自洽。
pub type IssuerCidBoundOuter = BoundedVec<u8, ConstU32<128>>;

/// 中文注释:业务作用域名称上限,用于省/市/镇等作用域字段。
pub type ScopeNameBoundOuter = BoundedVec<u8, ConstU32<64>>;

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
/// 中文注释:绑定凭证结构体。
///
/// 签发身份统一为机构模型:
/// `issuer_cid_number + issuer_main_account + signer_pubkey`。
/// `scope_*` 只表示业务作用域,不再表示签发人身份。
pub struct BindCredential<AccountId, Hash, Nonce, Signature> {
    pub binding_id: Hash,
    pub bind_nonce: Nonce,
    pub issuer_cid_number: IssuerCidBoundOuter,
    pub issuer_main_account: AccountId,
    pub signer_pubkey: [u8; 32],
    pub scope_province_name: ScopeNameBoundOuter,
    pub scope_city_name: ScopeNameBoundOuter,
    pub signature: Signature,
}

/// 中文注释:身份注册局绑定验签接口,由 Runtime 注入具体实现。
pub trait CidVerifier<AccountId, Hash, Nonce, Signature> {
    fn verify(
        account: &AccountId,
        credential: &BindCredential<AccountId, Hash, Nonce, Signature>,
    ) -> bool;
}

impl<AccountId, Hash, Nonce, Signature> CidVerifier<AccountId, Hash, Nonce, Signature> for () {
    fn verify(
        _account: &AccountId,
        _credential: &BindCredential<AccountId, Hash, Nonce, Signature>,
    ) -> bool {
        false
    }
}

/// 中文注释:公民投票实时验签接口。runtime 必须确认 `signer_pubkey` 属于
/// `issuer_main_account` 对应的 admins 集合,再验证签名。
pub trait CidVoteVerifier<AccountId, Hash, Nonce, Signature> {
    fn verify_vote(
        account: &AccountId,
        binding_id: Hash,
        proposal_id: u64,
        nonce: &Nonce,
        signature: &Signature,
        issuer_cid_number: &[u8],
        issuer_main_account: &AccountId,
        signer_pubkey: &[u8; 32],
        scope_province_name: &[u8],
        scope_city_name: &[u8],
    ) -> bool;
}

impl<AccountId, Hash, Nonce, Signature> CidVoteVerifier<AccountId, Hash, Nonce, Signature> for () {
    fn verify_vote(
        _account: &AccountId,
        _binding_id: Hash,
        _proposal_id: u64,
        _nonce: &Nonce,
        _signature: &Signature,
        _issuer_cid_number: &[u8],
        _issuer_main_account: &AccountId,
        _signer_pubkey: &[u8; 32],
        _scope_province_name: &[u8],
        _scope_city_name: &[u8],
    ) -> bool {
        false
    }
}

/// 中文注释:绑定成功后的钩子,用于让发行模块基于 binding_id 做一次性奖励判定。
pub trait OnCidBound<AccountId, Hash> {
    fn on_cid_bound(_who: &AccountId, _binding_id: Hash) {}
}

impl<AccountId, Hash> OnCidBound<AccountId, Hash> for () {}

/// 中文注释:回调 weight 声明接口,供 bind_cid 在申报 weight 时叠加回调预算。
pub trait OnCidBoundWeight {
    fn on_cid_bound_weight() -> Weight {
        Weight::zero()
    }
}

impl OnCidBoundWeight for () {}

/// 中文注释:给投票模块使用的统一资格接口。
pub trait CidEligibilityProvider<AccountId, Hash> {
    fn is_eligible(binding_id: &Hash, who: &AccountId) -> bool;
    fn verify_and_consume_vote_credential(
        binding_id: &Hash,
        who: &AccountId,
        proposal_id: u64,
        nonce: &[u8],
        signature: &[u8],
        issuer_cid_number: &[u8],
        issuer_main_account: &AccountId,
        signer_pubkey: &[u8; 32],
        scope_province_name: &[u8],
        scope_city_name: &[u8],
    ) -> bool;

    /// 清理某个提案维度下的投票凭证防重放状态。
    fn cleanup_vote_credentials(_proposal_id: u64) {}
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;
    use frame_support::{pallet_prelude::*, traits::EnsureOrigin, Blake2_128Concat};
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::Hash;

    pub type NonceOf<T> = BoundedVec<u8, <T as Config>::MaxCredentialNonceLength>;
    pub type SignatureOf<T> = BoundedVec<u8, <T as Config>::MaxCredentialSignatureLength>;
    pub type CredentialOf<T> = BindCredential<
        <T as frame_system::Config>::AccountId,
        <T as frame_system::Config>::Hash,
        NonceOf<T>,
        SignatureOf<T>,
    >;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        #[pallet::constant]
        type MaxCredentialNonceLength: Get<u32>;

        #[pallet::constant]
        type MaxCredentialSignatureLength: Get<u32>;

        /// 中文注释:身份注册局绑定验签器(外部接口桥接点)。
        type CidVerifier: CidVerifier<Self::AccountId, Self::Hash, NonceOf<Self>, SignatureOf<Self>>;

        /// 中文注释:公民投票实时验签器。
        type CidVoteVerifier: CidVoteVerifier<
            Self::AccountId,
            Self::Hash,
            NonceOf<Self>,
            SignatureOf<Self>,
        >;

        /// 中文注释:绑定后回调到发行模块发放认证奖励。
        type OnCidBound: OnCidBound<Self::AccountId, Self::Hash> + OnCidBoundWeight;

        /// 中文注释:`unbind_cid` 由治理 origin / Root / 受信任管理员调用。
        type UnbindOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// 权重信息:由 runtime 注入实际 benchmark 结果。
        type WeightInfo: crate::weights::WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// 中文注释:binding_id 到账户的正向映射,保证同一 binding_id 只能绑定一个账户。
    #[pallet::storage]
    #[pallet::getter(fn binding_id_to_account)]
    pub type BindingIdToAccount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, T::AccountId, OptionQuery>;

    /// 中文注释:账户到 binding_id 的反向映射,用于快速查询账户当前绑定的身份标识。
    #[pallet::storage]
    #[pallet::getter(fn account_to_binding_id)]
    pub type AccountToBindingId<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, T::Hash, OptionQuery>;

    /// 中文注释:当前已绑定身份的账户数量,可用于公民投票基数。
    #[pallet::storage]
    #[pallet::getter(fn bound_count)]
    pub type BoundCount<T> = StorageValue<_, u64, ValueQuery>;

    /// 中文注释:已消费的绑定 nonce,防止同一条绑定消息重放。
    #[pallet::storage]
    #[pallet::getter(fn used_bind_nonce)]
    pub type UsedBindNonce<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, bool, ValueQuery>;

    /// 中文注释:公民投票验签 nonce(提案 + binding_id + nonce 三元维度)防重放。
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

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 中文注释:CID 绑定成功,记录账户、binding_id 和 nonce 哈希。
        CidBound {
            who: T::AccountId,
            binding_id: T::Hash,
            bind_nonce_hash: T::Hash,
        },
        /// 中文注释:受治理 origin 授权解绑用户 CID,记录被解绑用户和 binding_id。
        CidUnbound {
            who: T::AccountId,
            binding_id: T::Hash,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 中文注释:绑定凭证中 bind_nonce 为空。
        EmptyBindNonce,
        /// 中文注释:该 bind_nonce 已被使用(防重放)。
        BindNonceAlreadyUsed,
        /// 中文注释:CID 绑定签名验证失败。
        InvalidCidBindingSignature,
        /// 中文注释:该 binding_id 已被另一个账户绑定。
        BindingIdAlreadyBoundToAnotherAccount,
        /// 中文注释:该账户已绑定到同一 binding_id,无需重复操作。
        SameBindingIdAlreadyBound,
        /// 中文注释:账户当前未绑定 CID,无法解绑。
        NotBound,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 中文注释:使用身份注册局签发的绑定消息把钱包和 binding_id 绑定。
        #[pallet::call_index(0)]
        #[pallet::weight(
            T::WeightInfo::bind_cid()
                .saturating_add(T::OnCidBound::on_cid_bound_weight())
        )]
        pub fn bind_cid(origin: OriginFor<T>, credential: CredentialOf<T>) -> DispatchResult {
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
                T::CidVerifier::verify(&who, &credential),
                Error::<T>::InvalidCidBindingSignature
            );

            let binding_id = credential.binding_id;
            if let Some(existing_owner) = BindingIdToAccount::<T>::get(binding_id) {
                ensure!(
                    existing_owner == who,
                    Error::<T>::BindingIdAlreadyBoundToAnotherAccount
                );
                return Err(Error::<T>::SameBindingIdAlreadyBound.into());
            }

            // 中文注释:账户允许换绑到新的 binding_id,但只释放旧映射,不减少当前绑定人数。
            if let Some(old_binding_id) = AccountToBindingId::<T>::get(&who) {
                BindingIdToAccount::<T>::remove(old_binding_id);
            } else {
                BoundCount::<T>::mutate(|v| *v = v.saturating_add(1));
            }

            BindingIdToAccount::<T>::insert(binding_id, &who);
            AccountToBindingId::<T>::insert(&who, binding_id);
            UsedBindNonce::<T>::insert(bind_nonce_hash, true);

            T::OnCidBound::on_cid_bound(&who, binding_id);

            Self::deposit_event(Event::<T>::CidBound {
                who,
                binding_id,
                bind_nonce_hash,
            });
            Ok(())
        }

        /// 中文注释:由治理 origin 解绑指定用户的 CID 绑定关系。
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::unbind_cid())]
        pub fn unbind_cid(origin: OriginFor<T>, target: T::AccountId) -> DispatchResult {
            T::UnbindOrigin::ensure_origin(origin)?;

            let binding_id = AccountToBindingId::<T>::get(&target).ok_or(Error::<T>::NotBound)?;
            AccountToBindingId::<T>::remove(&target);
            BindingIdToAccount::<T>::remove(binding_id);
            BoundCount::<T>::mutate(|v| *v = v.saturating_sub(1));

            Self::deposit_event(Event::<T>::CidUnbound {
                who: target,
                binding_id,
            });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// 中文注释:查询账户是否已绑定 CID。
        pub fn is_bound(who: &T::AccountId) -> bool {
            AccountToBindingId::<T>::contains_key(who)
        }

        /// 中文注释:查询指定 binding_id 是否绑定到指定账户。
        pub fn is_binding_id_bound_to(binding_id: &T::Hash, who: &T::AccountId) -> bool {
            BindingIdToAccount::<T>::get(binding_id)
                .map(|owner| owner == *who)
                .unwrap_or(false)
        }
    }

    /// 中文注释:实现投票资格接口,供治理模块统一判断公民身份和消费投票凭证。
    impl<T: Config> crate::CidEligibilityProvider<T::AccountId, T::Hash> for Pallet<T> {
        fn is_eligible(binding_id: &T::Hash, who: &T::AccountId) -> bool {
            Self::is_binding_id_bound_to(binding_id, who)
        }

        fn verify_and_consume_vote_credential(
            binding_id: &T::Hash,
            who: &T::AccountId,
            proposal_id: u64,
            nonce: &[u8],
            signature: &[u8],
            issuer_cid_number: &[u8],
            issuer_main_account: &T::AccountId,
            signer_pubkey: &[u8; 32],
            scope_province_name: &[u8],
            scope_city_name: &[u8],
        ) -> bool {
            if nonce.is_empty() || signature.is_empty() || issuer_cid_number.is_empty() {
                return false;
            }

            if !Self::is_binding_id_bound_to(binding_id, who) {
                return false;
            }

            let nonce_hash = T::Hashing::hash(nonce);
            let vote_nonce_key = (*binding_id, nonce_hash);
            if UsedVoteNonce::<T>::get(proposal_id, vote_nonce_key) {
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

            if !T::CidVoteVerifier::verify_vote(
                who,
                *binding_id,
                proposal_id,
                &nonce_bounded,
                &signature_bounded,
                issuer_cid_number,
                issuer_main_account,
                signer_pubkey,
                scope_province_name,
                scope_city_name,
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
mod tests;
