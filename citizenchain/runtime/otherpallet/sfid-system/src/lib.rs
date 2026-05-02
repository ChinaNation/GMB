//! # SFID 绑定与资格校验模块 (sfid-system)
//!
//! 本模块负责四件核心事:
//! - SFID 与链上账户的一对一绑定 / 解绑。
//! - 公民投票资格校验(基于 SFID 绑定关系 + SFID 系统签名凭证)。
//! - 维护按省 3-tier 管理员花名册(`ShengAdmins[Province][Slot]`)。
//! - 维护按省 + admin 二维独立的省级签名公钥(`ShengSigningPubkey[Province][AdminPubkey]`)。
//!
//! 架构边界(ADR-008):
//! - **没有 KEY_ADMIN**:链上 0 prior knowledge of SFID,初始 storage 全空。
//! - **首激活 first-come-first-serve**:任意省 admin 公钥首次调 `activate_sheng_signing_pubkey`
//!   被记录到 `ShengAdmins[Province][Main]`,后续 backup 由 Main 签名授权。
//! - **每省 3 把独立签名密钥**:Main / Backup1 / Backup2 各自一把,业务凭证签发互不共享。
//! - **链 → SFID 单向 pull**;**SFID → 链 4 个 Pays::No extrinsic** (零余额可推链)。
//!
//! 设计边界:
//! - 不保存 SFID 明文,只保存 `binding_id`。
//! - 绑定成功后的奖励发行通过 `OnSfidBound` 回调给上游模块处理。
//! - 投票凭证校验返回 `bool`,不抛 dispatch 错误,不污染治理模块语义。

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub use pallet::*;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod duoqian_info;
pub mod sheng_admins;
pub mod weights;
pub use sheng_admins::payload::{
    ACTIVATE_DOMAIN, ADD_BACKUP_DOMAIN, REMOVE_BACKUP_DOMAIN, ROTATE_DOMAIN,
};
pub use sheng_admins::types::Slot;

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::ConstU32;
use frame_support::weights::Weight;
use frame_support::BoundedVec;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

/// 中文注释:省名上限 64 字节(对应 sfid-system pallet 内 `ProvinceBound` 容量)。
/// 在 trait 层面公开,避免外部业务模块重复定义。
pub type ProvinceBoundOuter = BoundedVec<u8, ConstU32<64>>;

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
/// 中文注释:绑定凭证结构体,封装 binding_id、一次性 nonce、ADR-008 step3 双层
/// (province, signer_admin_pubkey) 字段以及 SFID 系统签名。
/// SCALE 顺序固定:`binding_id → bind_nonce → province → signer_admin_pubkey → signature`,
/// wumin / wuminapp / SFID 后端三处签发流程必须按本顺序序列化。
pub struct BindCredential<Hash, Nonce, Signature> {
    pub binding_id: Hash,
    pub bind_nonce: Nonce,
    /// ADR-008 step3:与 `signer_admin_pubkey` 配合在 `ShengSigningPubkey` 双映射中查派生签名公钥。
    pub province: ProvinceBoundOuter,
    /// ADR-008 step3:签发本绑定凭证的省级 admin 公钥(花名册内三槽之一)。
    pub signer_admin_pubkey: [u8; 32],
    pub signature: Signature,
}

/// 中文注释:SFID 系统绑定验签接口,由 Runtime 注入具体实现(sr25519 验签桥接)。
pub trait SfidVerifier<AccountId, Hash, Nonce, Signature> {
    fn verify(account: &AccountId, credential: &BindCredential<Hash, Nonce, Signature>) -> bool;
}

impl<AccountId, Hash, Nonce, Signature> SfidVerifier<AccountId, Hash, Nonce, Signature> for () {
    fn verify(_account: &AccountId, _credential: &BindCredential<Hash, Nonce, Signature>) -> bool {
        false
    }
}

/// 中文注释:公民投票实时验签接口。ADR-008 step3 起,签名载荷与公钥派生路径
/// 都按 (province, signer_admin_pubkey) 双层匹配,绑定身份标识与提案 ID 同步进 payload。
pub trait SfidVoteVerifier<AccountId, Hash, Nonce, Signature> {
    fn verify_vote(
        account: &AccountId,
        binding_id: Hash,
        proposal_id: u64,
        nonce: &Nonce,
        signature: &Signature,
        province: &[u8],
        signer_admin_pubkey: &[u8; 32],
    ) -> bool;
}

impl<AccountId, Hash, Nonce, Signature> SfidVoteVerifier<AccountId, Hash, Nonce, Signature> for () {
    fn verify_vote(
        _account: &AccountId,
        _binding_id: Hash,
        _proposal_id: u64,
        _nonce: &Nonce,
        _signature: &Signature,
        _province: &[u8],
        _signer_admin_pubkey: &[u8; 32],
    ) -> bool {
        false
    }
}

/// 中文注释:绑定成功后的钩子,用于让发行模块基于 binding_id 做一次性奖励判定。
pub trait OnSfidBound<AccountId, Hash> {
    fn on_sfid_bound(_who: &AccountId, _binding_id: Hash) {}
}

impl<AccountId, Hash> OnSfidBound<AccountId, Hash> for () {}

/// 中文注释:回调 weight 声明接口,供 bind_sfid 在申报 weight 时叠加回调预算。
pub trait OnSfidBoundWeight {
    fn on_sfid_bound_weight() -> Weight {
        Weight::zero()
    }
}

impl OnSfidBoundWeight for () {}

/// 中文注释:给投票模块使用的统一资格接口。
/// ADR-008 step3:`verify_and_consume_vote_credential` 加 (province, signer_admin_pubkey)
/// 双层匹配字段,链上不再保留任何"SFID main 兜底"路径。
pub trait SfidEligibilityProvider<AccountId, Hash> {
    fn is_eligible(binding_id: &Hash, who: &AccountId) -> bool;
    fn verify_and_consume_vote_credential(
        binding_id: &Hash,
        who: &AccountId,
        proposal_id: u64,
        nonce: &[u8],
        signature: &[u8],
        province: &[u8],
        signer_admin_pubkey: &[u8; 32],
    ) -> bool;

    /// 清理某个提案维度下的投票凭证防重放状态。
    fn cleanup_vote_credentials(_proposal_id: u64) {}
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;
    use alloc::vec::Vec;
    use frame_support::{pallet_prelude::*, traits::EnsureOrigin, Blake2_128Concat, Twox64Concat};
    use frame_system::pallet_prelude::*;
    use sp_core::sr25519;
    use sp_io::crypto::sr25519_verify;
    use sp_runtime::traits::Hash;

    pub type NonceOf<T> = BoundedVec<u8, <T as Config>::MaxCredentialNonceLength>;
    pub type SignatureOf<T> = BoundedVec<u8, <T as Config>::MaxCredentialSignatureLength>;
    pub type CredentialOf<T> =
        BindCredential<<T as frame_system::Config>::Hash, NonceOf<T>, SignatureOf<T>>;

    /// 中文注释:省名上限 64 字节 UTF-8(支持多字节中文 / 拉丁拼音)。
    pub type ProvinceBound = BoundedVec<u8, ConstU32<64>>;

    /// 中文注释:Step 2a unsigned extrinsic 的 nonce(由 SFID 后端生成的 32 字节随机数)。
    pub type ShengNonce = [u8; 32];

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        #[pallet::constant]
        type MaxCredentialNonceLength: Get<u32>;

        #[pallet::constant]
        type MaxCredentialSignatureLength: Get<u32>;

        /// 中文注释:SFID 系统绑定验签器(外部接口桥接点)。
        type SfidVerifier: SfidVerifier<
            Self::AccountId,
            Self::Hash,
            NonceOf<Self>,
            SignatureOf<Self>,
        >;

        /// 中文注释:公民投票实时验签器。
        type SfidVoteVerifier: SfidVoteVerifier<
            Self::AccountId,
            Self::Hash,
            NonceOf<Self>,
            SignatureOf<Self>,
        >;

        /// 中文注释:绑定后回调到发行模块发放认证奖励。
        type OnSfidBound: OnSfidBound<Self::AccountId, Self::Hash> + OnSfidBoundWeight;

        /// 中文注释:`unbind_sfid` 由谁可调用(治理 origin / Root / 受信任管理员)。
        /// ADR-008 后链上不再硬编码 SFID admin pubkey,unbind 不能再用 SfidMainAccount,
        /// 由 runtime 指定 origin(临时:可用 Root / 治理多签)。
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

    /// 中文注释:省管理员 3-tier 花名册。
    /// 索引:`(province, slot)` → admin sr25519 公钥(32 字节)。
    /// 写入路径:`activate_sheng_signing_pubkey`(占 Main)/ `add_sheng_admin_backup`(占 Backup{1,2})。
    /// 删除路径:`remove_sheng_admin_backup`(级联清 ShengSigningPubkey 同 admin 行)。
    #[pallet::storage]
    #[pallet::getter(fn sheng_admins)]
    pub type ShengAdmins<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        ProvinceBound,
        Twox64Concat,
        Slot,
        [u8; 32],
        OptionQuery,
    >;

    /// 中文注释:按 (省, admin 公钥) 二维存储的省级签名公钥。
    /// ADR-008:每省 3 把独立签名密钥(Main / Backup1 / Backup2 各一把),互不共享。
    /// runtime verifier(`duoqian-manage` 等)按 (province, signer_admin_pubkey) 二元组查表验签。
    #[pallet::storage]
    #[pallet::getter(fn sheng_signing_pubkey_storage)]
    pub type ShengSigningPubkey<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        ProvinceBound,
        Blake2_128Concat,
        [u8; 32],
        [u8; 32],
        OptionQuery,
    >;

    /// 中文注释:已使用的 unsigned extrinsic nonce(blake2_256 哈希),防 4 个 SFID 推链 extrinsic 重放。
    /// 使用 `()` 值 + `ValueQuery` 仅为 substrate set-only 语义;clippy 关于 unit 返回值的提示无关紧要。
    #[allow(clippy::unused_unit)]
    #[pallet::storage]
    #[pallet::getter(fn used_sheng_nonce)]
    pub type UsedShengNonce<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, (), ValueQuery>;

    // ADR-008 Step 2c:GenesisConfig 已彻底删除。
    // 链上 0 prior knowledge of SFID,创世 storage 全空,
    // ShengAdmins / ShengSigningPubkey 走 first-come-first-serve activation 写入。

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 中文注释:SFID 绑定成功,记录账户、binding_id 和 nonce 哈希。
        SfidBound {
            who: T::AccountId,
            binding_id: T::Hash,
            bind_nonce_hash: T::Hash,
        },
        /// 中文注释:管理员代为解绑用户 SFID,记录管理员、被解绑用户和 binding_id。
        SfidUnbound {
            admin: T::AccountId,
            who: T::AccountId,
            binding_id: T::Hash,
        },
        /// 中文注释:省管理员 backup 槽被占用(Main 已 activate 后由 Main 签名授权写入)。
        ShengAdminBackupAdded {
            province: ProvinceBound,
            slot: Slot,
            pubkey: [u8; 32],
        },
        /// 中文注释:省管理员 backup 槽被清空(级联清同 admin 行 ShengSigningPubkey)。
        ShengAdminBackupRemoved {
            province: ProvinceBound,
            slot: Slot,
            pubkey: [u8; 32],
        },
        /// 中文注释:省级签名公钥首次激活(写入 ShengSigningPubkey;若 Main 槽空则同时占 Main)。
        ShengSigningActivated {
            province: ProvinceBound,
            admin_pubkey: [u8; 32],
            signing_pubkey: [u8; 32],
        },
        /// 中文注释:省级签名公钥轮换(替换同 admin 行的现有 signing pubkey)。
        ShengSigningRotated {
            province: ProvinceBound,
            admin_pubkey: [u8; 32],
            old_signing_pubkey: [u8; 32],
            new_signing_pubkey: [u8; 32],
        },
    }

    /// 中文注释:本模块无需 on_initialize / on_finalize 钩子。
    /// ADR-008 Step 2c:on_runtime_upgrade 仅 log 提示;开发期老 storage 数据由
    /// fresh genesis 重启清理(`feedback_chain_in_dev.md` 允许),不在 hook 内做
    /// 显式 kill_prefix(已删字段无 storage_alias 引用,残留 entry 无人读取也无害)。
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> Weight {
            // 中文注释:ADR-008 Step 2c。开发期裸升级路径,实际 storage 清理由
            // chain 重启 + fresh genesis 完成;提示文本归 sheng_admins/migration.rs 维护。
            crate::sheng_admins::migration::log_legacy_storage_cleanup()
        }
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 中文注释:绑定凭证中 bind_nonce 为空。
        EmptyBindNonce,
        /// 中文注释:该 bind_nonce 已被使用(防重放)。
        BindNonceAlreadyUsed,
        /// 中文注释:SFID 绑定签名验证失败。
        InvalidSfidBindingSignature,
        /// 中文注释:该 binding_id 已被另一个账户绑定。
        BindingIdAlreadyBoundToAnotherAccount,
        /// 中文注释:该账户已绑定到同一 binding_id,无需重复操作。
        SameBindingIdAlreadyBound,
        /// 中文注释:账户当前未绑定 SFID,无法解绑。
        NotBound,
        /// 中文注释:Main 槽已被占用,activate 走 first-come-first-serve 失败。
        Sheng3TierMainAlreadyActivated,
        /// 中文注释:admin_pubkey 不在 ShengAdmins[province][\*] 任何槽中。
        Sheng3TierAdminNotInRoster,
        /// 中文注释:目标 backup 槽已被占用(必须先 remove)。
        Sheng3TierSlotOccupied,
        /// 中文注释:该 admin 当前没有有效签名公钥,不允许 rotate。
        Sheng3TierSigningNotActivated,
        /// 中文注释:sr25519 验签失败。
        Sheng3TierSignatureInvalid,
        /// 中文注释:nonce 已使用(防重放)。
        Sheng3TierNonceUsed,
        /// 中文注释:省名长度超限(>64 字节)。
        Sheng3TierProvinceTooLong,
        /// 中文注释:add_backup / remove_backup 只接受 Backup1 / Backup2,不能传 Main。
        Sheng3TierInvalidSlotForBackup,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 中文注释:使用 SFID 系统签发的绑定消息把钱包和 binding_id 绑定。
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

            // 中文注释:账户允许换绑到新的 binding_id,但只释放旧映射,不减少当前绑定人数。
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

        /// 中文注释:管理员代为解绑指定用户的 SFID 绑定关系。
        /// ADR-008 后改为由 `T::UnbindOrigin`(治理 / Root / 受信任管理员)鉴权,
        /// 不再依赖已删除的 SfidMainAccount / 省级签名账户。
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::unbind_sfid())]
        pub fn unbind_sfid(origin: OriginFor<T>, target: T::AccountId) -> DispatchResult {
            T::UnbindOrigin::ensure_origin(origin)?;

            // 移除 target 的绑定映射
            let binding_id = AccountToBindingId::<T>::get(&target).ok_or(Error::<T>::NotBound)?;
            AccountToBindingId::<T>::remove(&target);
            BindingIdToAccount::<T>::remove(binding_id);
            BoundCount::<T>::mutate(|v| *v = v.saturating_sub(1));

            Self::deposit_event(Event::<T>::SfidUnbound {
                admin: target.clone(),
                who: target,
                binding_id,
            });
            Ok(())
        }

        /// 中文注释:由本省 Main 签名授权,在 Backup1 / Backup2 槽写入新 admin 公钥。
        /// - origin = unsigned;鉴权全靠 sr25519 验签 + ValidateUnsigned 防重放。
        /// - 调用前提:本省 ShengAdmins[Main] 必须已激活(否则没有 Main 私钥能签名)。
        /// - slot ∈ {Backup1, Backup2};对应槽必须为空。
        #[pallet::call_index(2)]
        #[pallet::weight((T::WeightInfo::add_sheng_admin_backup(), DispatchClass::Normal, Pays::No))]
        pub fn add_sheng_admin_backup(
            origin: OriginFor<T>,
            province: Vec<u8>,
            slot: Slot,
            new_pubkey: [u8; 32],
            nonce: ShengNonce,
            sig: [u8; 64],
        ) -> DispatchResult {
            ensure_none(origin)?;

            ensure!(
                matches!(slot, Slot::Backup1 | Slot::Backup2),
                Error::<T>::Sheng3TierInvalidSlotForBackup
            );

            let bounded = Self::bounded_province(&province)?;

            let main_pubkey = ShengAdmins::<T>::get(&bounded, Slot::Main)
                .ok_or(Error::<T>::Sheng3TierAdminNotInRoster)?;

            ensure!(
                ShengAdmins::<T>::get(&bounded, slot).is_none(),
                Error::<T>::Sheng3TierSlotOccupied
            );

            // 中文注释:nonce 已用过 → 防重放(ValidateUnsigned 已查一次,这里二次保险)。
            let nonce_hash = T::Hashing::hash(&nonce);
            ensure!(
                !UsedShengNonce::<T>::contains_key(nonce_hash),
                Error::<T>::Sheng3TierNonceUsed
            );

            let payload = crate::sheng_admins::payload::add_backup_payload(
                &province,
                slot,
                &new_pubkey,
                &nonce,
            );
            ensure!(
                Self::verify_sr25519(&main_pubkey, &sig, &payload),
                Error::<T>::Sheng3TierSignatureInvalid
            );

            UsedShengNonce::<T>::insert(nonce_hash, ());
            ShengAdmins::<T>::insert(&bounded, slot, new_pubkey);

            Self::deposit_event(Event::<T>::ShengAdminBackupAdded {
                province: bounded,
                slot,
                pubkey: new_pubkey,
            });
            Ok(())
        }

        /// 中文注释:由本省 Main 签名授权,清空 Backup1 / Backup2 槽。
        /// - 级联效应:同时清 ShengSigningPubkey[(province, removed_admin_pubkey)]。
        #[pallet::call_index(3)]
        #[pallet::weight((T::WeightInfo::remove_sheng_admin_backup(), DispatchClass::Normal, Pays::No))]
        pub fn remove_sheng_admin_backup(
            origin: OriginFor<T>,
            province: Vec<u8>,
            slot: Slot,
            nonce: ShengNonce,
            sig: [u8; 64],
        ) -> DispatchResult {
            ensure_none(origin)?;

            ensure!(
                matches!(slot, Slot::Backup1 | Slot::Backup2),
                Error::<T>::Sheng3TierInvalidSlotForBackup
            );

            let bounded = Self::bounded_province(&province)?;

            let main_pubkey = ShengAdmins::<T>::get(&bounded, Slot::Main)
                .ok_or(Error::<T>::Sheng3TierAdminNotInRoster)?;

            let removed_pubkey = ShengAdmins::<T>::get(&bounded, slot)
                .ok_or(Error::<T>::Sheng3TierAdminNotInRoster)?;

            let nonce_hash = T::Hashing::hash(&nonce);
            ensure!(
                !UsedShengNonce::<T>::contains_key(nonce_hash),
                Error::<T>::Sheng3TierNonceUsed
            );

            let payload =
                crate::sheng_admins::payload::remove_backup_payload(&province, slot, &nonce);
            ensure!(
                Self::verify_sr25519(&main_pubkey, &sig, &payload),
                Error::<T>::Sheng3TierSignatureInvalid
            );

            UsedShengNonce::<T>::insert(nonce_hash, ());
            ShengAdmins::<T>::remove(&bounded, slot);
            ShengSigningPubkey::<T>::remove(&bounded, removed_pubkey);

            Self::deposit_event(Event::<T>::ShengAdminBackupRemoved {
                province: bounded,
                slot,
                pubkey: removed_pubkey,
            });
            Ok(())
        }

        /// 中文注释:首次激活省级签名公钥。
        /// 鉴权:sig 由 admin_pubkey 私钥签发。
        /// 业务:
        /// - 若 Main 槽空 → first-come-first-serve 占 Main 槽,同时写 ShengSigningPubkey。
        /// - 若 Main 槽已被占 → admin_pubkey 必须 ∈ ShengAdmins[province][\*],写 ShengSigningPubkey。
        /// - 若该 admin 已有 signing pubkey → 用 `rotate_sheng_signing_pubkey` 而非本入口。
        #[pallet::call_index(4)]
        #[pallet::weight((T::WeightInfo::activate_sheng_signing_pubkey(), DispatchClass::Normal, Pays::No))]
        pub fn activate_sheng_signing_pubkey(
            origin: OriginFor<T>,
            province: Vec<u8>,
            admin_pubkey: [u8; 32],
            signing_pubkey: [u8; 32],
            nonce: ShengNonce,
            sig: [u8; 64],
        ) -> DispatchResult {
            ensure_none(origin)?;

            let bounded = Self::bounded_province(&province)?;

            let nonce_hash = T::Hashing::hash(&nonce);
            ensure!(
                !UsedShengNonce::<T>::contains_key(nonce_hash),
                Error::<T>::Sheng3TierNonceUsed
            );

            let payload = crate::sheng_admins::payload::activate_payload(
                &province,
                &admin_pubkey,
                &signing_pubkey,
                &nonce,
            );
            ensure!(
                Self::verify_sr25519(&admin_pubkey, &sig, &payload),
                Error::<T>::Sheng3TierSignatureInvalid
            );

            let main_existing = ShengAdmins::<T>::get(&bounded, Slot::Main);
            let occupies_main = match main_existing {
                None => true,
                Some(_) => {
                    // 中文注释:Main 已占,本次只允许花名册内 admin 调用。
                    let in_roster = ShengAdmins::<T>::get(&bounded, Slot::Main).as_ref()
                        == Some(&admin_pubkey)
                        || ShengAdmins::<T>::get(&bounded, Slot::Backup1).as_ref()
                            == Some(&admin_pubkey)
                        || ShengAdmins::<T>::get(&bounded, Slot::Backup2).as_ref()
                            == Some(&admin_pubkey);
                    ensure!(in_roster, Error::<T>::Sheng3TierAdminNotInRoster);
                    false
                }
            };

            UsedShengNonce::<T>::insert(nonce_hash, ());

            if occupies_main {
                ShengAdmins::<T>::insert(&bounded, Slot::Main, admin_pubkey);
            }
            ShengSigningPubkey::<T>::insert(&bounded, admin_pubkey, signing_pubkey);

            Self::deposit_event(Event::<T>::ShengSigningActivated {
                province: bounded,
                admin_pubkey,
                signing_pubkey,
            });
            Ok(())
        }

        /// 中文注释:轮换某 admin 的省级签名公钥。
        /// 鉴权:sig 由 admin_pubkey 私钥签发。
        /// 前提:admin_pubkey ∈ ShengAdmins[province][\*] 且 ShengSigningPubkey[(province, admin)] 已存在。
        #[pallet::call_index(5)]
        #[pallet::weight((T::WeightInfo::rotate_sheng_signing_pubkey(), DispatchClass::Normal, Pays::No))]
        pub fn rotate_sheng_signing_pubkey(
            origin: OriginFor<T>,
            province: Vec<u8>,
            admin_pubkey: [u8; 32],
            new_signing_pubkey: [u8; 32],
            nonce: ShengNonce,
            sig: [u8; 64],
        ) -> DispatchResult {
            ensure_none(origin)?;

            let bounded = Self::bounded_province(&province)?;

            let nonce_hash = T::Hashing::hash(&nonce);
            ensure!(
                !UsedShengNonce::<T>::contains_key(nonce_hash),
                Error::<T>::Sheng3TierNonceUsed
            );

            // admin 必须在花名册中
            ensure!(
                Self::is_sheng_admin(&province, &admin_pubkey).is_some(),
                Error::<T>::Sheng3TierAdminNotInRoster
            );

            let old_signing_pubkey = ShengSigningPubkey::<T>::get(&bounded, admin_pubkey)
                .ok_or(Error::<T>::Sheng3TierSigningNotActivated)?;

            let payload = crate::sheng_admins::payload::rotate_payload(
                &province,
                &admin_pubkey,
                &new_signing_pubkey,
                &nonce,
            );
            ensure!(
                Self::verify_sr25519(&admin_pubkey, &sig, &payload),
                Error::<T>::Sheng3TierSignatureInvalid
            );

            UsedShengNonce::<T>::insert(nonce_hash, ());
            ShengSigningPubkey::<T>::insert(&bounded, admin_pubkey, new_signing_pubkey);

            Self::deposit_event(Event::<T>::ShengSigningRotated {
                province: bounded,
                admin_pubkey,
                old_signing_pubkey,
                new_signing_pubkey,
            });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// 中文注释:查询账户是否已绑定 SFID。
        pub fn is_bound(who: &T::AccountId) -> bool {
            AccountToBindingId::<T>::contains_key(who)
        }

        /// 中文注释:查询指定 binding_id 是否绑定到指定账户。
        pub fn is_binding_id_bound_to(binding_id: &T::Hash, who: &T::AccountId) -> bool {
            BindingIdToAccount::<T>::get(binding_id)
                .map(|owner| owner == *who)
                .unwrap_or(false)
        }

        /// 中文注释:按 (province, admin_pubkey) 二元组读省级签名公钥。
        /// runtime verifier(`duoqian-manage` 等)按本入口验签。
        pub fn sheng_signing_pubkey_for_admin(
            province: &[u8],
            admin_pubkey: &[u8; 32],
        ) -> Option<[u8; 32]> {
            let bounded = ProvinceBound::try_from(province.to_vec()).ok()?;
            ShengSigningPubkey::<T>::get(&bounded, admin_pubkey)
        }

        /// 中文注释:判断 pubkey 是否在某省花名册中,返回所占槽位。
        pub fn is_sheng_admin(province: &[u8], pubkey: &[u8; 32]) -> Option<Slot> {
            let bounded = ProvinceBound::try_from(province.to_vec()).ok()?;
            [Slot::Main, Slot::Backup1, Slot::Backup2]
                .into_iter()
                .find(|slot| ShengAdmins::<T>::get(&bounded, *slot).as_ref() == Some(pubkey))
        }

        /// 中文注释:判断 pubkey 是否为某省 Main(用于业务判定 + 治理快速路径)。
        pub fn is_sheng_main(province: &[u8], pubkey: &[u8; 32]) -> bool {
            let bounded = match ProvinceBound::try_from(province.to_vec()) {
                Ok(v) => v,
                Err(_) => return false,
            };
            ShengAdmins::<T>::get(&bounded, Slot::Main).as_ref() == Some(pubkey)
        }

        // ---- Helpers (内部) ----

        fn bounded_province(province: &[u8]) -> Result<ProvinceBound, Error<T>> {
            ProvinceBound::try_from(province.to_vec())
                .map_err(|_| Error::<T>::Sheng3TierProvinceTooLong)
        }

        fn verify_sr25519(public: &[u8; 32], sig: &[u8; 64], msg: &[u8; 32]) -> bool {
            let pk = sr25519::Public::from_raw(*public);
            let signature = sr25519::Signature::from_raw(*sig);
            sr25519_verify(&signature, msg, &pk)
        }
    }

    /// 中文注释:实现投票资格接口,供治理模块统一判断公民身份和消费投票凭证。
    /// ADR-008 step3:消费凭证时把 (province, signer_admin_pubkey) 透传到 verifier,
    /// runtime verifier 会按 `ShengSigningPubkey` 双映射查派生签名公钥并验签。
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
            province: &[u8],
            signer_admin_pubkey: &[u8; 32],
        ) -> bool {
            if nonce.is_empty() || signature.is_empty() || province.is_empty() {
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

            if !T::SfidVoteVerifier::verify_vote(
                who,
                *binding_id,
                proposal_id,
                &nonce_bounded,
                &signature_bounded,
                province,
                signer_admin_pubkey,
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

    /// 中文注释:Step 2a unsigned extrinsic 校验入口。
    /// 4 个 SFID 推链 extrinsic 全部走 ensure_none + sr25519 验签;TxPool 提交前会先调
    /// `validate_unsigned`,对应防重放 + 验签 + priority/longevity。
    /// 进入 dispatch 前 `pre_dispatch` 再做一次原子查重以防同区块多笔同 nonce。
    #[pallet::validate_unsigned]
    impl<T: Config> ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;

        fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
            let (nonce, payload, signer_pubkey, sig, tag_seed) =
                match Self::extract_unsigned_parts(call) {
                    Some(v) => v,
                    None => return InvalidTransaction::Call.into(),
                };

            let nonce_hash = T::Hashing::hash(&nonce);
            if UsedShengNonce::<T>::contains_key(nonce_hash) {
                return InvalidTransaction::Stale.into();
            }

            if !Self::verify_sr25519(&signer_pubkey, &sig, &payload) {
                return InvalidTransaction::BadProof.into();
            }

            // 中文注释:tag = (b"sfid-system-step2a", tag_seed) 保证同 nonce 在 pool 内只允许一笔。
            ValidTransaction::with_tag_prefix("SfidSystemStep2a")
                .priority(TransactionPriority::MAX / 2)
                .and_provides((tag_seed, nonce))
                .longevity(64)
                .propagate(true)
                .build()
        }

        fn pre_dispatch(call: &Self::Call) -> Result<(), TransactionValidityError> {
            let (nonce, payload, signer_pubkey, sig, _tag_seed) =
                Self::extract_unsigned_parts(call).ok_or(InvalidTransaction::Call)?;

            let nonce_hash = T::Hashing::hash(&nonce);
            if UsedShengNonce::<T>::contains_key(nonce_hash) {
                return Err(InvalidTransaction::Stale.into());
            }

            if !Self::verify_sr25519(&signer_pubkey, &sig, &payload) {
                return Err(InvalidTransaction::BadProof.into());
            }

            Ok(())
        }
    }

    /// 中文注释:Step 2a unsigned extrinsic 拆解结果。
    /// 字段:(nonce, payload_hash, 签名公钥, sig, tag_seed)。
    pub type UnsignedParts = (ShengNonce, [u8; 32], [u8; 32], [u8; 64], &'static [u8]);

    impl<T: Config> Pallet<T> {
        /// 中文注释:把 unsigned extrinsic 拆出 (nonce, payload_hash, 签名公钥, sig, tag_seed)。
        /// - tag_seed 用于 ValidateUnsigned 的 tx pool tag 区分(避免同 nonce 跨 extrinsic 复用)。
        /// - 返回 None 表示不是本模块 4 个 unsigned extrinsic 之一(InvalidTransaction::Call)。
        fn extract_unsigned_parts(call: &Call<T>) -> Option<UnsignedParts> {
            match call {
                Call::add_sheng_admin_backup {
                    province,
                    slot,
                    new_pubkey,
                    nonce,
                    sig,
                } => {
                    let bounded = ProvinceBound::try_from(province.clone()).ok()?;
                    let signer = ShengAdmins::<T>::get(&bounded, Slot::Main)?;
                    let payload = crate::sheng_admins::payload::add_backup_payload(
                        province, *slot, new_pubkey, nonce,
                    );
                    Some((*nonce, payload, signer, *sig, b"add_backup"))
                }
                Call::remove_sheng_admin_backup {
                    province,
                    slot,
                    nonce,
                    sig,
                } => {
                    let bounded = ProvinceBound::try_from(province.clone()).ok()?;
                    let signer = ShengAdmins::<T>::get(&bounded, Slot::Main)?;
                    let payload =
                        crate::sheng_admins::payload::remove_backup_payload(province, *slot, nonce);
                    Some((*nonce, payload, signer, *sig, b"remove_backup"))
                }
                Call::activate_sheng_signing_pubkey {
                    province,
                    admin_pubkey,
                    signing_pubkey,
                    nonce,
                    sig,
                } => {
                    let payload = crate::sheng_admins::payload::activate_payload(
                        province,
                        admin_pubkey,
                        signing_pubkey,
                        nonce,
                    );
                    Some((*nonce, payload, *admin_pubkey, *sig, b"activate"))
                }
                Call::rotate_sheng_signing_pubkey {
                    province,
                    admin_pubkey,
                    new_signing_pubkey,
                    nonce,
                    sig,
                } => {
                    let payload = crate::sheng_admins::payload::rotate_payload(
                        province,
                        admin_pubkey,
                        new_signing_pubkey,
                        nonce,
                    );
                    Some((*nonce, payload, *admin_pubkey, *sig, b"rotate"))
                }
                _ => None,
            }
        }
    }
}

#[cfg(test)]
mod tests;
