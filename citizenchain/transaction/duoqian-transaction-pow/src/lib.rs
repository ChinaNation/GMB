#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    ensure,
    pallet_prelude::*,
    traits::{Currency, ExistenceRequirement, ReservableCurrency},
    weights::Weight,
    BoundedVec,
};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::traits::Zero;
use sp_std::{collections::btree_set::BTreeSet, prelude::*};

type BalanceOf<T> =
    <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// 多签管理员认证抽象：由 runtime 对接具体公钥/签名算法。
pub trait DuoqianAdminAuth<AccountId> {
    type PublicKey: Parameter + Member + MaxEncodedLen + Ord + Clone;
    type Signature: Parameter + Member + MaxEncodedLen + Clone;

    fn is_valid_public_key(public_key: &Self::PublicKey) -> bool;
    fn public_key_to_account(public_key: &Self::PublicKey) -> Option<AccountId>;
    fn verify_signature(
        public_key: &Self::PublicKey,
        payload: &[u8],
        signature: &Self::Signature,
    ) -> bool;
}

/// 账户地址合法性抽象：用于校验 duoqian_address 是否为本链合法哈希地址。
pub trait DuoqianAddressValidator<AccountId> {
    fn is_valid(address: &AccountId) -> bool;
}

impl<AccountId> DuoqianAddressValidator<AccountId> for () {
    fn is_valid(_address: &AccountId) -> bool {
        true
    }
}

/// 保留地址校验抽象：用于拦截制度保留地址被 duoqian 抢注册。
pub trait DuoqianReservedAddressChecker<AccountId> {
    fn is_reserved(address: &AccountId) -> bool;
}

impl<AccountId> DuoqianReservedAddressChecker<AccountId> for () {
    fn is_reserved(_address: &AccountId) -> bool {
        false
    }
}

/// 转出源地址保护：用于禁止制度保留地址作为资金转出源。
pub trait ProtectedSourceChecker<AccountId> {
    fn is_protected(address: &AccountId) -> bool;
}

impl<AccountId> ProtectedSourceChecker<AccountId> for () {
    fn is_protected(_address: &AccountId) -> bool {
        false
    }
}

/// SFID 机构登记操作员权限校验：仅 SFID 系统授权账户可登记机构ID。
pub trait SfidRegistryOperator<AccountId> {
    fn can_register(operator: &AccountId) -> bool;
}

impl<AccountId> SfidRegistryOperator<AccountId> for () {
    fn can_register(_operator: &AccountId) -> bool {
        false
    }
}

pub trait WeightInfo {
    fn register_sfid_institution() -> Weight;
    fn create_duoqian(approval_count: u32) -> Weight;
    fn close_duoqian(approval_count: u32) -> Weight;
}

impl WeightInfo for () {
    fn register_sfid_institution() -> Weight {
        Weight::from_parts(40_000_000, 1_024)
    }

    fn create_duoqian(approval_count: u32) -> Weight {
        Weight::from_parts(120_000_000, 4_096).saturating_add(
            Weight::from_parts(25_000_000, 256).saturating_mul(approval_count as u64),
        )
    }

    fn close_duoqian(approval_count: u32) -> Weight {
        Weight::from_parts(95_000_000, 3_072).saturating_add(
            Weight::from_parts(25_000_000, 256).saturating_mul(approval_count as u64),
        )
    }
}

#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
)]
pub struct AdminApproval<PublicKey, Signature> {
    pub public_key: PublicKey,
    pub signature: Signature,
}

#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
)]
pub struct DuoqianAccount<AdminList, AccountId, BlockNumber> {
    pub admin_count: u32,
    pub threshold: u32,
    pub duoqian_admins: AdminList,
    pub creator: AccountId,
    pub created_at: BlockNumber,
}

#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
)]
pub struct RegisteredInstitution<SfidId> {
    pub sfid_id: SfidId,
    pub nonce: u64,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        type AdminAuth: DuoqianAdminAuth<Self::AccountId>;

        type AddressValidator: DuoqianAddressValidator<Self::AccountId>;
        type ReservedAddressChecker: DuoqianReservedAddressChecker<Self::AccountId>;
        type ProtectedSourceChecker: ProtectedSourceChecker<Self::AccountId>;
        type SfidRegistryOperator: SfidRegistryOperator<Self::AccountId>;

        #[pallet::constant]
        type MaxAdmins: Get<u32>;

        #[pallet::constant]
        type MaxSfidIdLength: Get<u32>;

        /// 创建时最低入金（默认应设置为 111 分 = 1.11 元）。
        #[pallet::constant]
        type MinCreateAmount: Get<BalanceOf<Self>>;

        /// 注销时账户最低余额门槛（默认应设置为 111 分 = 1.11 元）。
        #[pallet::constant]
        type MinCloseBalance: Get<BalanceOf<Self>>;

        type WeightInfo: WeightInfo;
    }

    pub type AdminApprovalOf<T> =
        AdminApproval<
            <<T as Config>::AdminAuth as DuoqianAdminAuth<
                <T as frame_system::Config>::AccountId,
            >>::PublicKey,
            <<T as Config>::AdminAuth as DuoqianAdminAuth<
                <T as frame_system::Config>::AccountId,
            >>::Signature,
        >;

    pub type AdminApprovalsOf<T> = BoundedVec<AdminApprovalOf<T>, <T as Config>::MaxAdmins>;

    pub type DuoqianAdminsOf<T> =
        BoundedVec<
            <<T as Config>::AdminAuth as DuoqianAdminAuth<
                <T as frame_system::Config>::AccountId,
            >>::PublicKey,
            <T as Config>::MaxAdmins,
        >;

    pub type DuoqianAccountOf<T> = DuoqianAccount<
        DuoqianAdminsOf<T>,
        <T as frame_system::Config>::AccountId,
        BlockNumberFor<T>,
    >;

    pub type SfidIdOf<T> = BoundedVec<u8, <T as Config>::MaxSfidIdLength>;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// 多签账户配置。key 为 duoqian_address。
    #[pallet::storage]
    #[pallet::getter(fn duoqian_account_of)]
    pub type DuoqianAccounts<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, DuoqianAccountOf<T>, OptionQuery>;

    /// SFID 机构登记：sfid_id -> duoqian_address（由 blake3 派生）
    #[pallet::storage]
    #[pallet::getter(fn sfid_registered_address)]
    pub type SfidRegisteredAddress<T: Config> =
        StorageMap<_, Blake2_128Concat, SfidIdOf<T>, T::AccountId, OptionQuery>;

    /// SFID 机构登记反向索引：duoqian_address -> { sfid_id, nonce }
    #[pallet::storage]
    #[pallet::getter(fn address_registered_sfid)]
    pub type AddressRegisteredSfid<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, RegisteredInstitution<SfidIdOf<T>>, OptionQuery>;

    /// 持久化的链域哈希（固定为 genesis hash），用于签名域隔离。
    #[pallet::storage]
    #[pallet::getter(fn chain_domain_hash)]
    pub type ChainDomainHash<T: Config> = StorageValue<_, T::Hash, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 多签账户创建成功。
        DuoqianCreated {
            duoqian_address: T::AccountId,
            creator: T::AccountId,
            admin_count: u32,
            threshold: u32,
            amount: BalanceOf<T>,
        },
        /// 多签账户注销成功。
        DuoqianClosed {
            duoqian_address: T::AccountId,
            submitter: T::AccountId,
            beneficiary: T::AccountId,
            amount: BalanceOf<T>,
        },
        SfidInstitutionRegistered {
            sfid_id: SfidIdOf<T>,
            duoqian_address: T::AccountId,
            operator: T::AccountId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 参数不完整
        IncompleteParameters,
        /// 地址非法
        InvalidAddress,
        /// 地址为制度保留地址，不允许注册
        AddressReserved,
        /// 地址已存在（已初始化）
        AddressAlreadyExists,
        /// 链上已存在同地址账户
        AddressAlreadyOnChain,
        /// 公钥重复
        DuplicatePublicKey,
        /// 阈值不合法
        InvalidThreshold,
        /// 金额不足
        InsufficientAmount,
        /// 手续费不足（由交易支付系统返回）
        InsufficientFee,
        /// 签名不足
        InsufficientSignatures,
        /// 权限不足
        PermissionDenied,
        /// 管理员数量不合法（必须 >=2）
        InvalidAdminCount,
        /// 管理员数量与列表长度不一致
        AdminCountMismatch,
        /// 管理员公钥格式非法
        InvalidAdminPublicKey,
        /// 管理员签名非法
        InvalidAdminSignature,
        /// 多签账户不存在
        DuoqianNotFound,
        /// 注销收款地址非法（不允许等于 duoqian_address）
        InvalidBeneficiary,
        /// 资金转出源地址受保护，不允许转出
        ProtectedSource,
        /// SFID机构未登记，不允许创建
        InstitutionNotRegistered,
        /// SFID机构登记操作无权限
        UnauthorizedSfidRegistrar,
        /// SFID ID 重复登记
        SfidAlreadyRegistered,
        /// SFID ID 为空
        EmptySfidId,
        /// 无法将派生地址转换为账户ID
        DerivedAddressDecodeFailed,
        /// 管理员签名已过期
        SignatureExpired,
        /// 链域哈希暂不可用（等待初始化）
        ChainDomainHashUnavailable,
        /// 账户仍有保留余额，不允许注销
        ReservedBalanceRemaining,
        /// nonce 已耗尽
        NonceOverflow,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // NOTE: `call_index` values are the on-chain ABI and must remain stable.
        // Function declaration order is kept compatible with existing deployments.
        /// SFID 系统登记机构：
        /// - 仅 SFID 系统授权账户可调用；
        /// - 地址按 blake3("DUOQIAN_SFID_V1" || sfid_id) 固定派生；
        /// - 同一 sfid_id 只能登记一次。
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::register_sfid_institution())]
        pub fn register_sfid_institution(
            origin: OriginFor<T>,
            sfid_id: SfidIdOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::ensure_chain_domain_hash_initialized()?;
            ensure!(!sfid_id.is_empty(), Error::<T>::EmptySfidId);
            ensure!(
                T::SfidRegistryOperator::can_register(&who),
                Error::<T>::UnauthorizedSfidRegistrar
            );
            ensure!(
                !SfidRegisteredAddress::<T>::contains_key(&sfid_id),
                Error::<T>::SfidAlreadyRegistered
            );

            let duoqian_address = Self::derive_duoqian_address_from_sfid_id(sfid_id.as_slice())?;
            ensure!(
                !AddressRegisteredSfid::<T>::contains_key(&duoqian_address),
                Error::<T>::AddressAlreadyExists
            );
            ensure!(
                !T::ReservedAddressChecker::is_reserved(&duoqian_address),
                Error::<T>::AddressReserved
            );
            ensure!(
                T::AddressValidator::is_valid(&duoqian_address),
                Error::<T>::InvalidAddress
            );

            SfidRegisteredAddress::<T>::insert(&sfid_id, &duoqian_address);
            AddressRegisteredSfid::<T>::insert(
                &duoqian_address,
                RegisteredInstitution {
                    sfid_id: sfid_id.clone(),
                    nonce: 0,
                },
            );
            Self::deposit_event(Event::<T>::SfidInstitutionRegistered {
                sfid_id,
                duoqian_address,
                operator: who,
            });
            Ok(())
        }

        /// 创建多签账户：
        /// - 参数必须完整；
        /// - N>=2，M>=ceil(N/2) 且 M<=N；
        /// - duoqian_admins 去重且长度等于 N；
        /// - sfid_id 必须已完成机构登记，duoqian_address 由链上登记映射解析；
        /// - 发起人必须是管理员之一；
        /// - 管理员有效签名数必须 >= M；
        /// - 创建时转入金额必须 >= MinCreateAmount（建议 111 分）。
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::create_duoqian(approvals.len() as u32))]
        pub fn create_duoqian(
            origin: OriginFor<T>,
            sfid_id: SfidIdOf<T>,
            admin_count: u32,
            duoqian_admins: DuoqianAdminsOf<T>,
            threshold: u32,
            amount: BalanceOf<T>,
            expires_at: BlockNumberFor<T>,
            approvals: AdminApprovalsOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                !T::ProtectedSourceChecker::is_protected(&who),
                Error::<T>::ProtectedSource
            );
            let now = frame_system::Pallet::<T>::block_number();
            ensure!(now <= expires_at, Error::<T>::SignatureExpired);

            ensure!(!duoqian_admins.is_empty(), Error::<T>::IncompleteParameters);
            ensure!(
                amount >= T::MinCreateAmount::get(),
                Error::<T>::InsufficientAmount
            );

            ensure!(admin_count >= 2, Error::<T>::InvalidAdminCount);
            ensure!(
                duoqian_admins.len() as u32 == admin_count,
                Error::<T>::AdminCountMismatch
            );

            let duoqian_address =
                SfidRegisteredAddress::<T>::get(&sfid_id).ok_or(Error::<T>::InstitutionNotRegistered)?;
            let mut registered = AddressRegisteredSfid::<T>::get(&duoqian_address)
                .ok_or(Error::<T>::InstitutionNotRegistered)?;
            ensure!(registered.sfid_id == sfid_id, Error::<T>::InstitutionNotRegistered);

            ensure!(
                !T::ReservedAddressChecker::is_reserved(&duoqian_address),
                Error::<T>::AddressReserved
            );
            ensure!(
                T::AddressValidator::is_valid(&duoqian_address),
                Error::<T>::InvalidAddress
            );
            ensure!(
                !DuoqianAccounts::<T>::contains_key(&duoqian_address),
                Error::<T>::AddressAlreadyExists
            );

            let min_threshold = core::cmp::max(2, admin_count.saturating_add(1) / 2);
            ensure!(
                threshold >= min_threshold && threshold <= admin_count,
                Error::<T>::InvalidThreshold
            );

            Self::ensure_unique_and_valid_admins(&duoqian_admins)?;

            let caller_is_admin = duoqian_admins.iter().any(|pk| {
                T::AdminAuth::public_key_to_account(pk)
                    .map(|acc| acc == who)
                    .unwrap_or(false)
            });
            ensure!(caller_is_admin, Error::<T>::PermissionDenied);

            Self::ensure_chain_domain_hash_initialized()?;
            let nonce = registered.nonce;
            ensure!(nonce < u64::MAX, Error::<T>::NonceOverflow);
            let payload = (
                b"DUOQIAN_CREATE_V2".to_vec(),
                Self::signature_domain_hash_value()?,
                nonce,
                expires_at,
                &sfid_id,
                &duoqian_address,
                admin_count,
                &duoqian_admins,
                threshold,
                amount,
            )
                .encode();
            let signed = Self::count_valid_signatures(&duoqian_admins, &approvals, &payload)?;
            ensure!(signed >= threshold, Error::<T>::InsufficientSignatures);

            T::Currency::transfer(
                &who,
                &duoqian_address,
                amount,
                ExistenceRequirement::KeepAlive,
            )?;

            DuoqianAccounts::<T>::insert(
                &duoqian_address,
                DuoqianAccount {
                    admin_count,
                    threshold,
                    duoqian_admins: duoqian_admins.clone(),
                    creator: who.clone(),
                    created_at: frame_system::Pallet::<T>::block_number(),
                },
            );
            registered.nonce = nonce.saturating_add(1);
            AddressRegisteredSfid::<T>::insert(&duoqian_address, registered);

            Self::deposit_event(Event::<T>::DuoqianCreated {
                duoqian_address,
                creator: who,
                admin_count,
                threshold,
                amount,
            });

            Ok(())
        }

        /// 注销多签账户：
        /// - 任意管理员可发起，但签名数仍需 >= M；
        /// - 账户余额必须 >= MinCloseBalance（建议 111 分）；
        /// - 将该多签账户余额一次性转至 beneficiary；
        /// - 余额清零后删除配置；
        /// - 删除后可按新管理员配置重新创建。
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::close_duoqian(approvals.len() as u32))]
        pub fn close_duoqian(
            origin: OriginFor<T>,
            duoqian_address: T::AccountId,
            beneficiary: T::AccountId,
            min_balance: BalanceOf<T>,
            expires_at: BlockNumberFor<T>,
            approvals: AdminApprovalsOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let now = frame_system::Pallet::<T>::block_number();
            ensure!(now <= expires_at, Error::<T>::SignatureExpired);
            ensure!(
                !T::ProtectedSourceChecker::is_protected(&duoqian_address),
                Error::<T>::ProtectedSource
            );
            ensure!(
                beneficiary != duoqian_address,
                Error::<T>::InvalidBeneficiary
            );
            ensure!(
                !T::ReservedAddressChecker::is_reserved(&beneficiary),
                Error::<T>::InvalidBeneficiary
            );

            let account =
                DuoqianAccounts::<T>::get(&duoqian_address).ok_or(Error::<T>::DuoqianNotFound)?;
            let admin_count = account.admin_count;
            let threshold = account.threshold;
            let admins: DuoqianAdminsOf<T> = account.duoqian_admins.clone();

            let caller_is_admin = admins.iter().any(|pk| {
                T::AdminAuth::public_key_to_account(pk)
                    .map(|acc| acc == who)
                    .unwrap_or(false)
            });
            ensure!(caller_is_admin, Error::<T>::PermissionDenied);

            let all_balance = T::Currency::free_balance(&duoqian_address);
            ensure!(
                all_balance >= T::MinCloseBalance::get(),
                Error::<T>::InsufficientAmount
            );
            ensure!(all_balance >= min_balance, Error::<T>::InsufficientAmount);
            ensure!(
                T::Currency::reserved_balance(&duoqian_address).is_zero(),
                Error::<T>::ReservedBalanceRemaining
            );

            Self::ensure_chain_domain_hash_initialized()?;
            let mut registered = AddressRegisteredSfid::<T>::get(&duoqian_address)
                .ok_or(Error::<T>::InstitutionNotRegistered)?;
            let nonce = registered.nonce;
            ensure!(nonce < u64::MAX, Error::<T>::NonceOverflow);
            let payload = (
                b"DUOQIAN_CLOSE_V2".to_vec(),
                Self::signature_domain_hash_value()?,
                nonce,
                expires_at,
                &duoqian_address,
                &beneficiary,
                admin_count,
                threshold,
                min_balance,
            )
                .encode();
            let signed = Self::count_valid_signatures(&admins, &approvals, &payload)?;
            ensure!(signed >= threshold, Error::<T>::InsufficientSignatures);

            T::Currency::transfer(
                &duoqian_address,
                &beneficiary,
                all_balance,
                ExistenceRequirement::AllowDeath,
            )?;

            DuoqianAccounts::<T>::remove(&duoqian_address);
            registered.nonce = nonce.saturating_add(1);
            AddressRegisteredSfid::<T>::insert(&duoqian_address, registered);

            Self::deposit_event(Event::<T>::DuoqianClosed {
                duoqian_address,
                submitter: who,
                beneficiary,
                amount: all_balance,
            });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn ensure_chain_domain_hash_initialized() -> DispatchResult {
            if ChainDomainHash::<T>::get().is_some() {
                return Ok(());
            }
            let genesis_hash = frame_system::Pallet::<T>::block_hash(BlockNumberFor::<T>::zero());
            ensure!(
                genesis_hash != T::Hash::default(),
                Error::<T>::ChainDomainHashUnavailable
            );
            ChainDomainHash::<T>::put(genesis_hash);
            Ok(())
        }

        fn signature_domain_hash_value() -> Result<T::Hash, DispatchError> {
            ChainDomainHash::<T>::get()
                .ok_or(Error::<T>::ChainDomainHashUnavailable.into())
        }

        pub fn derive_duoqian_address_from_sfid_id(
            sfid_id: &[u8],
        ) -> Result<T::AccountId, DispatchError> {
            let mut input = b"DUOQIAN_SFID_V1".to_vec();
            input.extend_from_slice(sfid_id);
            let digest = blake3::hash(input.as_slice());
            T::AccountId::decode(&mut &digest.as_bytes()[..])
                .map_err(|_| Error::<T>::DerivedAddressDecodeFailed.into())
        }

        fn ensure_unique_and_valid_admins(
            admins: &DuoqianAdminsOf<T>,
        ) -> Result<(), DispatchError> {
            let mut seen = BTreeSet::new();
            for pk in admins.iter() {
                ensure!(
                    T::AdminAuth::is_valid_public_key(pk),
                    Error::<T>::InvalidAdminPublicKey
                );
                ensure!(seen.insert(pk.clone()), Error::<T>::DuplicatePublicKey);
            }
            Ok(())
        }

        fn count_valid_signatures(
            admins: &DuoqianAdminsOf<T>,
            approvals: &AdminApprovalsOf<T>,
            payload: &[u8],
        ) -> Result<u32, DispatchError> {
            ensure!(!approvals.is_empty(), Error::<T>::IncompleteParameters);

            let admin_set: BTreeSet<_> = admins.iter().cloned().collect();
            let mut approved_signers = BTreeSet::new();

            for approval in approvals.iter() {
                ensure!(
                    admin_set.contains(&approval.public_key),
                    Error::<T>::PermissionDenied
                );
                ensure!(
                    T::AdminAuth::verify_signature(
                        &approval.public_key,
                        payload,
                        &approval.signature
                    ),
                    Error::<T>::InvalidAdminSignature
                );
                approved_signers.insert(approval.public_key.clone());
            }

            Ok(approved_signers.len() as u32)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{
        assert_noop, assert_ok, derive_impl,
        traits::{ConstU128, ConstU32, VariantCountOf},
    };
    use frame_system as system;
    use sp_core::{sr25519, Pair};
    use sp_runtime::{
        traits::{IdentifyAccount, IdentityLookup, Verify},
        AccountId32, BuildStorage, MultiSignature, MultiSigner,
    };

    type Block = frame_system::mocking::MockBlock<Test>;
    type Balance = u128;

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
        pub type Duoqian = pallet;
    }

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl system::Config for Test {
        type Block = Block;
        type AccountId = AccountId32;
        type Lookup = IdentityLookup<Self::AccountId>;
        type AccountData = pallet_balances::AccountData<Balance>;
        type Nonce = u64;
    }

    impl pallet_balances::Config for Test {
        type MaxLocks = ConstU32<0>;
        type MaxReserves = ConstU32<0>;
        type ReserveIdentifier = [u8; 8];
        type Balance = Balance;
        type RuntimeEvent = RuntimeEvent;
        type DustRemoval = ();
        type ExistentialDeposit = ConstU128<1>;
        type AccountStore = System;
        type WeightInfo = ();
        type FreezeIdentifier = RuntimeFreezeReason;
        type MaxFreezes = VariantCountOf<RuntimeFreezeReason>;
        type RuntimeHoldReason = RuntimeHoldReason;
        type RuntimeFreezeReason = RuntimeFreezeReason;
        type DoneSlashHandler = ();
    }

    pub struct TestAddressValidator;
    impl DuoqianAddressValidator<AccountId32> for TestAddressValidator {
        fn is_valid(address: &AccountId32) -> bool {
            address != &AccountId32::new([0u8; 32])
        }
    }

    pub struct TestReservedAddressChecker;
    impl DuoqianReservedAddressChecker<AccountId32> for TestReservedAddressChecker {
        fn is_reserved(address: &AccountId32) -> bool {
            *address == AccountId32::new([0xAA; 32])
        }
    }

    pub struct TestSfidRegistryOperator;
    impl SfidRegistryOperator<AccountId32> for TestSfidRegistryOperator {
        fn can_register(operator: &AccountId32) -> bool {
            *operator == AccountId32::new([0x55; 32])
        }
    }

    pub struct TestAdminAuth;
    impl DuoqianAdminAuth<AccountId32> for TestAdminAuth {
        type PublicKey = [u8; 32];
        type Signature = [u8; 64];

        fn is_valid_public_key(public_key: &Self::PublicKey) -> bool {
            public_key.iter().any(|b| *b != 0)
        }

        fn public_key_to_account(public_key: &Self::PublicKey) -> Option<AccountId32> {
            let signer = MultiSigner::from(sr25519::Public::from_raw(*public_key));
            Some(<MultiSigner as IdentifyAccount>::into_account(signer))
        }

        fn verify_signature(
            public_key: &Self::PublicKey,
            payload: &[u8],
            signature: &Self::Signature,
        ) -> bool {
            let signer = MultiSigner::from(sr25519::Public::from_raw(*public_key));
            let sig = MultiSignature::from(sr25519::Signature::from_raw(*signature));
            <MultiSignature as Verify>::verify(&sig, payload, &signer.into_account())
        }
    }

    impl Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type Currency = Balances;
        type AdminAuth = TestAdminAuth;
        type AddressValidator = TestAddressValidator;
        type ReservedAddressChecker = TestReservedAddressChecker;
        type ProtectedSourceChecker = ();
        type SfidRegistryOperator = TestSfidRegistryOperator;
        type MaxAdmins = ConstU32<10>;
        type MaxSfidIdLength = ConstU32<96>;
        type MinCreateAmount = ConstU128<111>;
        type MinCloseBalance = ConstU128<111>;
        type WeightInfo = ();
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let mut storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("system genesis build should succeed");

        let p1 = pair(1);
        let p2 = pair(2);
        let p3 = pair(3);

        pallet_balances::GenesisConfig::<Test> {
            balances: vec![
                (account_of(&p1), 10_000),
                (account_of(&p2), 10_000),
                (account_of(&p3), 10_000),
            ],
            dev_accounts: None,
        }
        .assimilate_storage(&mut storage)
        .expect("balances genesis build should succeed");

        sp_io::TestExternalities::new(storage)
    }

    fn pair(seed: u8) -> sr25519::Pair {
        sr25519::Pair::from_seed(&[seed; 32])
    }

    fn public_of(pair: &sr25519::Pair) -> [u8; 32] {
        pair.public().0
    }

    fn account_of(pair: &sr25519::Pair) -> AccountId32 {
        let signer = MultiSigner::from(pair.public());
        <MultiSigner as IdentifyAccount>::into_account(signer)
    }

    fn sign(pair: &sr25519::Pair, payload: &[u8]) -> [u8; 64] {
        pair.sign(payload).0
    }

    fn admins_vec(admins: Vec<[u8; 32]>) -> DuoqianAdminsOf<Test> {
        admins.try_into().expect("admins length within bound")
    }

    fn approvals_vec(approvals: Vec<AdminApproval<[u8; 32], [u8; 64]>>) -> AdminApprovalsOf<Test> {
        approvals.try_into().expect("approvals length within bound")
    }

    fn register_sfid_and_get_address(tag: &str) -> (SfidIdOf<Test>, AccountId32) {
        let sfid: SfidIdOf<Test> = format!("GFR-LN001-CB0C-{}-20260222", tag)
            .as_bytes()
            .to_vec()
            .try_into()
            .expect("sfid id should fit");
        assert_ok!(Duoqian::register_sfid_institution(
            RuntimeOrigin::signed(AccountId32::new([0x55; 32])),
            sfid.clone()
        ));
        let duoqian_address = Duoqian::sfid_registered_address(sfid.clone()).expect("sfid should be registered");
        (sfid, duoqian_address)
    }

    const DEFAULT_EXPIRES_AT: u64 = 1_000;
    const DEFAULT_MIN_CLOSE_BALANCE: u128 = 111;

    fn create_payload(
        sfid: &SfidIdOf<Test>,
        duoqian: &AccountId32,
        admin_count: u32,
        admins: &DuoqianAdminsOf<Test>,
        threshold: u32,
        amount: u128,
        expires_at: u64,
    ) -> Vec<u8> {
        let nonce = Duoqian::address_registered_sfid(duoqian)
            .map(|r| r.nonce)
            .unwrap_or(0);
        let domain = Duoqian::chain_domain_hash().unwrap_or_else(|| System::block_hash(0));
        (
            b"DUOQIAN_CREATE_V2".to_vec(),
            domain,
            nonce,
            expires_at,
            sfid,
            duoqian,
            admin_count,
            admins,
            threshold,
            amount,
        )
            .encode()
    }

    fn close_payload(
        duoqian: &AccountId32,
        beneficiary: &AccountId32,
        admin_count: u32,
        threshold: u32,
        min_balance: u128,
        expires_at: u64,
    ) -> Vec<u8> {
        let nonce = Duoqian::address_registered_sfid(duoqian)
            .map(|r| r.nonce)
            .unwrap_or(0);
        let domain = Duoqian::chain_domain_hash().unwrap_or_else(|| System::block_hash(0));
        (
            b"DUOQIAN_CLOSE_V2".to_vec(),
            domain,
            nonce,
            expires_at,
            duoqian,
            beneficiary,
            admin_count,
            threshold,
            min_balance,
        )
            .encode()
    }

    fn call_create(
        origin: RuntimeOrigin,
        sfid_id: SfidIdOf<Test>,
        admin_count: u32,
        duoqian_admins: DuoqianAdminsOf<Test>,
        threshold: u32,
        amount: u128,
        approvals: AdminApprovalsOf<Test>,
    ) -> DispatchResult {
        Duoqian::create_duoqian(
            origin,
            sfid_id,
            admin_count,
            duoqian_admins,
            threshold,
            amount,
            DEFAULT_EXPIRES_AT,
            approvals,
        )
    }

    fn call_close(
        origin: RuntimeOrigin,
        duoqian_address: AccountId32,
        beneficiary: AccountId32,
        approvals: AdminApprovalsOf<Test>,
    ) -> DispatchResult {
        Duoqian::close_duoqian(
            origin,
            duoqian_address,
            beneficiary,
            DEFAULT_MIN_CLOSE_BALANCE,
            DEFAULT_EXPIRES_AT,
            approvals,
        )
    }

    #[test]
    fn create_duoqian_works_and_locks_config() {
        new_test_ext().execute_with(|| {
            let p1 = pair(1);
            let p2 = pair(2);
            let (sfid, duoqian) = register_sfid_and_get_address("create-ok");

            let admins = admins_vec(vec![public_of(&p1), public_of(&p2)]);
            let payload = create_payload(&sfid, &duoqian, 2u32, &admins, 2u32, 111u128, DEFAULT_EXPIRES_AT);
            let approvals = approvals_vec(vec![
                AdminApproval {
                    public_key: public_of(&p1),
                    signature: sign(&p1, &payload),
                },
                AdminApproval {
                    public_key: public_of(&p2),
                    signature: sign(&p2, &payload),
                },
            ]);

            assert_ok!(call_create(
                RuntimeOrigin::signed(account_of(&p1)),
                sfid.clone(),
                2,
                admins,
                2,
                111,
                approvals
            ));

            let config = DuoqianAccounts::<Test>::get(&duoqian).expect("must exist");
            assert_eq!(config.admin_count, 2);
            assert_eq!(config.threshold, 2);
            assert_eq!(Balances::free_balance(&duoqian), 111);
        });
    }

    #[test]
    fn create_duoqian_rejects_duplicate_admins() {
        new_test_ext().execute_with(|| {
            let p1 = pair(1);
            let (sfid, duoqian) = register_sfid_and_get_address("dup-admin");
            let duplicated = public_of(&p1);

            let admins = admins_vec(vec![duplicated, duplicated]);
            let payload = create_payload(&sfid, &duoqian, 2u32, &admins, 2u32, 111u128, DEFAULT_EXPIRES_AT);
            let approvals = approvals_vec(vec![AdminApproval {
                public_key: duplicated,
                signature: sign(&p1, &payload),
            }]);

            assert_noop!(
                call_create(
                    RuntimeOrigin::signed(account_of(&p1)),
                    sfid,
                    2,
                    admins,
                2,
                    111,
                approvals
            ),
                Error::<Test>::DuplicatePublicKey
            );
        });
    }

    #[test]
    fn create_duoqian_rejects_invalid_threshold() {
        new_test_ext().execute_with(|| {
            let p1 = pair(1);
            let p2 = pair(2);
            let (sfid, duoqian) = register_sfid_and_get_address("threshold");

            let admins = admins_vec(vec![public_of(&p1), public_of(&p2)]);
            let payload = create_payload(&sfid, &duoqian, 2u32, &admins, 0u32, 111u128, DEFAULT_EXPIRES_AT);
            let approvals = approvals_vec(vec![AdminApproval {
                public_key: public_of(&p1),
                signature: sign(&p1, &payload),
            }]);

            assert_noop!(
                call_create(
                    RuntimeOrigin::signed(account_of(&p1)),
                    sfid,
                    2,
                    admins,
                    0,
                    111,
                approvals
            ),
                Error::<Test>::InvalidThreshold
            );
        });
    }

    #[test]
    fn create_duoqian_requires_half_or_more_signatures() {
        new_test_ext().execute_with(|| {
            let p1 = pair(1);
            let p2 = pair(2);
            let p3 = pair(3);
            let (sfid, duoqian) = register_sfid_and_get_address("half-sign");

            let admins = admins_vec(vec![public_of(&p1), public_of(&p2), public_of(&p3)]);
            let payload = create_payload(&sfid, &duoqian, 3u32, &admins, 2u32, 111u128, DEFAULT_EXPIRES_AT);
            let approvals = approvals_vec(vec![AdminApproval {
                public_key: public_of(&p1),
                signature: sign(&p1, &payload),
            }]);

            assert_noop!(
                call_create(
                    RuntimeOrigin::signed(account_of(&p1)),
                    sfid,
                    3,
                    admins,
                    2,
                    111,
                approvals
            ),
                Error::<Test>::InsufficientSignatures
            );
        });
    }

    #[test]
    fn close_duoqian_works_and_allows_recreate_with_new_admins() {
        new_test_ext().execute_with(|| {
            let p1 = pair(1);
            let p2 = pair(2);
            let p3 = pair(3);
            let p4 = pair(4);
            let (sfid, duoqian) = register_sfid_and_get_address("close-recreate");
            let beneficiary = account_of(&pair(8));

            // first create: admins p1,p2 threshold 1
            let admins1 = admins_vec(vec![public_of(&p1), public_of(&p2)]);
            let create_payload_1 = create_payload(&sfid, &duoqian, 2u32, &admins1, 2u32, 200u128, DEFAULT_EXPIRES_AT);
            let approvals_1 = approvals_vec(vec![
                AdminApproval {
                    public_key: public_of(&p1),
                    signature: sign(&p1, &create_payload_1),
                },
                AdminApproval {
                    public_key: public_of(&p2),
                    signature: sign(&p2, &create_payload_1),
                },
            ]);
            assert_ok!(call_create(
                RuntimeOrigin::signed(account_of(&p1)),
                sfid.clone(),
                2,
                admins1,
                2,
                200,
                approvals_1
            ));

            let close_payload = close_payload(
                &duoqian,
                &beneficiary,
                2u32,
                2u32,
                DEFAULT_MIN_CLOSE_BALANCE,
                DEFAULT_EXPIRES_AT,
            );
            let close_approvals = approvals_vec(vec![
                AdminApproval {
                    public_key: public_of(&p1),
                    signature: sign(&p1, &close_payload),
                },
                AdminApproval {
                    public_key: public_of(&p2),
                    signature: sign(&p2, &close_payload),
                },
            ]);
            assert_ok!(call_close(
                RuntimeOrigin::signed(account_of(&p2)),
                duoqian.clone(),
                beneficiary.clone(),
                close_approvals
            ));

            assert!(!DuoqianAccounts::<Test>::contains_key(&duoqian));
            assert_eq!(Balances::free_balance(&duoqian), 0);
            assert_eq!(Balances::free_balance(&beneficiary), 200);

            // recreate same address with different admins + threshold
            let admins2 = admins_vec(vec![public_of(&p3), public_of(&p4)]);
            let create_payload_2 = create_payload(&sfid, &duoqian, 2u32, &admins2, 2u32, 111u128, DEFAULT_EXPIRES_AT);
            let approvals_2 = approvals_vec(vec![
                AdminApproval {
                    public_key: public_of(&p3),
                    signature: sign(&p3, &create_payload_2),
                },
                AdminApproval {
                    public_key: public_of(&p4),
                    signature: sign(&p4, &create_payload_2),
                },
            ]);
            assert_ok!(call_create(
                RuntimeOrigin::signed(account_of(&p3)),
                sfid.clone(),
                2,
                admins2,
                2,
                111,
                approvals_2
            ));

            let config = DuoqianAccounts::<Test>::get(&duoqian).expect("recreate must succeed");
            assert_eq!(config.admin_count, 2);
            assert_eq!(config.threshold, 2);
            assert_eq!(config.duoqian_admins, vec![public_of(&p3), public_of(&p4)]);
        });
    }

    #[test]
    fn create_duoqian_rejects_admin_count_mismatch() {
        new_test_ext().execute_with(|| {
            let p1 = pair(1);
            let p2 = pair(2);
            let (sfid, duoqian) = register_sfid_and_get_address("count-mismatch");

            let admins = admins_vec(vec![public_of(&p1), public_of(&p2)]);
            let payload = create_payload(&sfid, &duoqian, 3u32, &admins, 2u32, 111u128, DEFAULT_EXPIRES_AT);
            let approvals = approvals_vec(vec![
                AdminApproval {
                    public_key: public_of(&p1),
                    signature: sign(&p1, &payload),
                },
                AdminApproval {
                    public_key: public_of(&p2),
                    signature: sign(&p2, &payload),
                },
            ]);

            assert_noop!(
                call_create(
                    RuntimeOrigin::signed(account_of(&p1)),
                    sfid,
                    3,
                    admins,
                    2,
                    111,
                approvals
            ),
                Error::<Test>::AdminCountMismatch
            );
        });
    }

    #[test]
    fn create_duoqian_rejects_non_admin_submitter() {
        new_test_ext().execute_with(|| {
            let p1 = pair(1);
            let p2 = pair(2);
            let outsider = pair(7);
            let (sfid, duoqian) = register_sfid_and_get_address("non-admin-submit");

            let admins = admins_vec(vec![public_of(&p1), public_of(&p2)]);
            let payload = create_payload(&sfid, &duoqian, 2u32, &admins, 2u32, 111u128, DEFAULT_EXPIRES_AT);
            let approvals = approvals_vec(vec![
                AdminApproval {
                    public_key: public_of(&p1),
                    signature: sign(&p1, &payload),
                },
                AdminApproval {
                    public_key: public_of(&p2),
                    signature: sign(&p2, &payload),
                },
            ]);

            assert_noop!(
                call_create(
                    RuntimeOrigin::signed(account_of(&outsider)),
                    sfid,
                    2,
                    admins,
                2,
                    111,
                approvals
            ),
                Error::<Test>::PermissionDenied
            );
        });
    }

    #[test]
    fn create_duoqian_rejects_non_admin_approval() {
        new_test_ext().execute_with(|| {
            let p1 = pair(1);
            let p2 = pair(2);
            let outsider = pair(7);
            let (sfid, duoqian) = register_sfid_and_get_address("non-admin-approval");

            let admins = admins_vec(vec![public_of(&p1), public_of(&p2)]);
            let payload = create_payload(&sfid, &duoqian, 2u32, &admins, 2u32, 111u128, DEFAULT_EXPIRES_AT);
            let approvals = approvals_vec(vec![AdminApproval {
                public_key: public_of(&outsider),
                signature: sign(&outsider, &payload),
            }]);

            assert_noop!(
                call_create(
                    RuntimeOrigin::signed(account_of(&p1)),
                    sfid,
                    2,
                    admins,
                2,
                    111,
                approvals
            ),
                Error::<Test>::PermissionDenied
            );
        });
    }

    #[test]
    fn create_duoqian_rejects_invalid_signature() {
        new_test_ext().execute_with(|| {
            let p1 = pair(1);
            let p2 = pair(2);
            let (sfid, duoqian) = register_sfid_and_get_address("invalid-sig");

            let admins = admins_vec(vec![public_of(&p1), public_of(&p2)]);
            let payload = create_payload(&sfid, &duoqian, 2u32, &admins, 2u32, 111u128, DEFAULT_EXPIRES_AT);
            // 使用错误签名者 p2 对 p1 公钥字段造签名，应该失败
            let approvals = approvals_vec(vec![AdminApproval {
                public_key: public_of(&p1),
                signature: sign(&p2, &payload),
            }]);

            assert_noop!(
                call_create(
                    RuntimeOrigin::signed(account_of(&p1)),
                    sfid,
                    2,
                    admins,
                2,
                    111,
                approvals
            ),
                Error::<Test>::InvalidAdminSignature
            );
        });
    }

    #[test]
    fn create_duoqian_allows_preexisting_system_account_for_registered_sfid() {
        new_test_ext().execute_with(|| {
            let p1 = pair(1);
            let p2 = pair(2);
            let (sfid, duoqian) = register_sfid_and_get_address("exists-onchain");
            let _ = Balances::deposit_creating(&duoqian, 50);

            let admins = admins_vec(vec![public_of(&p1), public_of(&p2)]);
            let payload = create_payload(&sfid, &duoqian, 2u32, &admins, 2u32, 111u128, DEFAULT_EXPIRES_AT);
            let approvals = approvals_vec(vec![
                AdminApproval {
                    public_key: public_of(&p1),
                    signature: sign(&p1, &payload),
                },
                AdminApproval {
                    public_key: public_of(&p2),
                    signature: sign(&p2, &payload),
                },
            ]);

            assert_ok!(call_create(
                RuntimeOrigin::signed(account_of(&p1)),
                sfid,
                2,
                admins,
                2,
                111,
                approvals
            ));
            assert_eq!(Balances::free_balance(&duoqian), 161);
        });
    }

    #[test]
    fn create_duoqian_rejects_reserved_address() {
        new_test_ext().execute_with(|| {
            let p1 = pair(1);
            let p2 = pair(2);
            let sfid: SfidIdOf<Test> = b"GFR-LN001-CB0C-reserved-20260222"
                .to_vec()
                .try_into()
                .expect("sfid id should fit");
            let duoqian = AccountId32::new([0xAA; 32]);
            SfidRegisteredAddress::<Test>::insert(&sfid, &duoqian);
            AddressRegisteredSfid::<Test>::insert(
                &duoqian,
                RegisteredInstitution {
                    sfid_id: sfid.clone(),
                    nonce: 0,
                },
            );

            let admins = admins_vec(vec![public_of(&p1), public_of(&p2)]);
            let payload = create_payload(&sfid, &duoqian, 2u32, &admins, 2u32, 111u128, DEFAULT_EXPIRES_AT);
            let approvals = approvals_vec(vec![AdminApproval {
                public_key: public_of(&p1),
                signature: sign(&p1, &payload),
            }]);

            assert_noop!(
                call_create(
                    RuntimeOrigin::signed(account_of(&p1)),
                    sfid,
                    2,
                    admins,
                2,
                    111,
                approvals
            ),
                Error::<Test>::AddressReserved
            );
        });
    }

    #[test]
    fn close_duoqian_rejects_beneficiary_equal_self() {
        new_test_ext().execute_with(|| {
            let p1 = pair(1);
            let p2 = pair(2);
            let (sfid, duoqian) = register_sfid_and_get_address("close-self");

            let admins = admins_vec(vec![public_of(&p1), public_of(&p2)]);
            let create_payload = create_payload(&sfid, &duoqian, 2u32, &admins, 2u32, 200u128, DEFAULT_EXPIRES_AT);
            let create_approvals = approvals_vec(vec![
                AdminApproval {
                    public_key: public_of(&p1),
                    signature: sign(&p1, &create_payload),
                },
                AdminApproval {
                    public_key: public_of(&p2),
                    signature: sign(&p2, &create_payload),
                },
            ]);
            assert_ok!(call_create(
                RuntimeOrigin::signed(account_of(&p1)),
                sfid.clone(),
                2,
                admins.clone(),
                2,
                200,
                create_approvals
            ));

            let close_payload = close_payload(&duoqian, &duoqian, 2u32, 2u32, 200u128, DEFAULT_EXPIRES_AT);
            let close_approvals = approvals_vec(vec![AdminApproval {
                public_key: public_of(&p2),
                signature: sign(&p2, &close_payload),
            }]);

            assert_noop!(
                call_close(
                    RuntimeOrigin::signed(account_of(&p2)),
                    duoqian.clone(),
                    duoqian.clone(),
                    close_approvals
                ),
                Error::<Test>::InvalidBeneficiary
            );
        });
    }

    #[test]
    fn close_duoqian_rejects_reserved_beneficiary() {
        new_test_ext().execute_with(|| {
            let p1 = pair(1);
            let p2 = pair(2);
            let (sfid, duoqian) = register_sfid_and_get_address("close-reserved-beneficiary");
            let beneficiary = AccountId32::new([0xAA; 32]);

            let admins = admins_vec(vec![public_of(&p1), public_of(&p2)]);
            let create_payload = create_payload(
                &sfid,
                &duoqian,
                2u32,
                &admins,
                2u32,
                200u128,
                DEFAULT_EXPIRES_AT,
            );
            let create_approvals = approvals_vec(vec![
                AdminApproval {
                    public_key: public_of(&p1),
                    signature: sign(&p1, &create_payload),
                },
                AdminApproval {
                    public_key: public_of(&p2),
                    signature: sign(&p2, &create_payload),
                },
            ]);
            assert_ok!(call_create(
                RuntimeOrigin::signed(account_of(&p1)),
                sfid.clone(),
                2,
                admins.clone(),
                2,
                200,
                create_approvals
            ));

            let close_payload = close_payload(
                &duoqian,
                &beneficiary,
                2u32,
                2u32,
                DEFAULT_MIN_CLOSE_BALANCE,
                DEFAULT_EXPIRES_AT,
            );
            let close_approvals = approvals_vec(vec![
                AdminApproval {
                    public_key: public_of(&p1),
                    signature: sign(&p1, &close_payload),
                },
                AdminApproval {
                    public_key: public_of(&p2),
                    signature: sign(&p2, &close_payload),
                },
            ]);

            assert_noop!(
                call_close(
                    RuntimeOrigin::signed(account_of(&p1)),
                    duoqian,
                    beneficiary,
                    close_approvals
                ),
                Error::<Test>::InvalidBeneficiary
            );
        });
    }

    #[test]
    fn close_duoqian_rejects_non_admin_submitter() {
        new_test_ext().execute_with(|| {
            let p1 = pair(1);
            let p2 = pair(2);
            let outsider = pair(7);
            let (sfid, duoqian) = register_sfid_and_get_address("close-nonadmin");
            let beneficiary = account_of(&pair(8));

            let admins = admins_vec(vec![public_of(&p1), public_of(&p2)]);
            let create_payload = create_payload(&sfid, &duoqian, 2u32, &admins, 2u32, 200u128, DEFAULT_EXPIRES_AT);
            let create_approvals = approvals_vec(vec![
                AdminApproval {
                    public_key: public_of(&p1),
                    signature: sign(&p1, &create_payload),
                },
                AdminApproval {
                    public_key: public_of(&p2),
                    signature: sign(&p2, &create_payload),
                },
            ]);
            assert_ok!(call_create(
                RuntimeOrigin::signed(account_of(&p1)),
                sfid.clone(),
                2,
                admins.clone(),
                2,
                200,
                create_approvals
            ));

            let close_payload = close_payload(
                &duoqian,
                &beneficiary,
                2u32,
                2u32,
                DEFAULT_MIN_CLOSE_BALANCE,
                DEFAULT_EXPIRES_AT,
            );
            let close_approvals = approvals_vec(vec![AdminApproval {
                public_key: public_of(&p1),
                signature: sign(&p1, &close_payload),
            }]);

            assert_noop!(
                call_close(
                    RuntimeOrigin::signed(account_of(&outsider)),
                    duoqian.clone(),
                    beneficiary.clone(),
                    close_approvals
                ),
                Error::<Test>::PermissionDenied
            );
        });
    }

    #[test]
    fn close_duoqian_allows_transfer_to_another_duoqian_address() {
        new_test_ext().execute_with(|| {
            let p1 = pair(1);
            let p2 = pair(2);
            let (sfid_a, duoqian_a) = register_sfid_and_get_address("close-to-other");
            let duoqian_b = account_of(&pair(10));

            let admins = admins_vec(vec![public_of(&p1), public_of(&p2)]);
            let create_payload = create_payload(&sfid_a, &duoqian_a, 2u32, &admins, 2u32, 300u128, DEFAULT_EXPIRES_AT);
            let create_approvals = approvals_vec(vec![
                AdminApproval {
                    public_key: public_of(&p1),
                    signature: sign(&p1, &create_payload),
                },
                AdminApproval {
                    public_key: public_of(&p2),
                    signature: sign(&p2, &create_payload),
                },
            ]);
            assert_ok!(call_create(
                RuntimeOrigin::signed(account_of(&p1)),
                sfid_a.clone(),
                2,
                admins.clone(),
                2,
                300,
                create_approvals
            ));

            let close_payload = close_payload(
                &duoqian_a,
                &duoqian_b,
                2u32,
                2u32,
                DEFAULT_MIN_CLOSE_BALANCE,
                DEFAULT_EXPIRES_AT,
            );
            let close_approvals = approvals_vec(vec![
                AdminApproval {
                    public_key: public_of(&p1),
                    signature: sign(&p1, &close_payload),
                },
                AdminApproval {
                    public_key: public_of(&p2),
                    signature: sign(&p2, &close_payload),
                },
            ]);
            assert_ok!(call_close(
                RuntimeOrigin::signed(account_of(&p2)),
                duoqian_a.clone(),
                duoqian_b.clone(),
                close_approvals
            ));

            assert_eq!(Balances::free_balance(&duoqian_a), 0);
            assert_eq!(Balances::free_balance(&duoqian_b), 300);
        });
    }

    #[test]
    fn old_create_signatures_cannot_be_replayed_after_close() {
        new_test_ext().execute_with(|| {
            let p1 = pair(1);
            let p2 = pair(2);
            let (sfid, duoqian) = register_sfid_and_get_address("replay-create");
            let beneficiary = account_of(&pair(9));
            let admins = admins_vec(vec![public_of(&p1), public_of(&p2)]);

            let create_payload_1 =
                create_payload(&sfid, &duoqian, 2u32, &admins, 2u32, 200u128, DEFAULT_EXPIRES_AT);
            let create_approvals_1 = approvals_vec(vec![
                AdminApproval {
                    public_key: public_of(&p1),
                    signature: sign(&p1, &create_payload_1),
                },
                AdminApproval {
                    public_key: public_of(&p2),
                    signature: sign(&p2, &create_payload_1),
                },
            ]);
            assert_ok!(call_create(
                RuntimeOrigin::signed(account_of(&p1)),
                sfid.clone(),
                2,
                admins.clone(),
                2,
                200,
                create_approvals_1
            ));

            let close_payload_1 = close_payload(
                &duoqian,
                &beneficiary,
                2u32,
                2u32,
                DEFAULT_MIN_CLOSE_BALANCE,
                DEFAULT_EXPIRES_AT,
            );
            let close_approvals_1 = approvals_vec(vec![
                AdminApproval {
                    public_key: public_of(&p1),
                    signature: sign(&p1, &close_payload_1),
                },
                AdminApproval {
                    public_key: public_of(&p2),
                    signature: sign(&p2, &close_payload_1),
                },
            ]);
            assert_ok!(call_close(
                RuntimeOrigin::signed(account_of(&p1)),
                duoqian.clone(),
                beneficiary,
                close_approvals_1
            ));

            // 重放旧 create 签名（nonce=0）必须失败
            let replay_old_create = approvals_vec(vec![
                AdminApproval {
                    public_key: public_of(&p1),
                    signature: sign(&p1, &create_payload_1),
                },
                AdminApproval {
                    public_key: public_of(&p2),
                    signature: sign(&p2, &create_payload_1),
                },
            ]);
            assert_noop!(
                call_create(
                    RuntimeOrigin::signed(account_of(&p1)),
                    sfid,
                    2,
                    admins,
                    2,
                    200,
                    replay_old_create
                ),
                Error::<Test>::InvalidAdminSignature
            );
        });
    }

    #[test]
    fn old_close_signatures_cannot_be_replayed_after_recreate() {
        new_test_ext().execute_with(|| {
            let p1 = pair(1);
            let p2 = pair(2);
            let p3 = pair(3);
            let p4 = pair(4);
            let (sfid, duoqian) = register_sfid_and_get_address("replay-close");
            let beneficiary = account_of(&pair(8));

            let admins1 = admins_vec(vec![public_of(&p1), public_of(&p2)]);
            let create_payload_1 =
                create_payload(&sfid, &duoqian, 2u32, &admins1, 2u32, 200u128, DEFAULT_EXPIRES_AT);
            let create_approvals_1 = approvals_vec(vec![
                AdminApproval {
                    public_key: public_of(&p1),
                    signature: sign(&p1, &create_payload_1),
                },
                AdminApproval {
                    public_key: public_of(&p2),
                    signature: sign(&p2, &create_payload_1),
                },
            ]);
            assert_ok!(call_create(
                RuntimeOrigin::signed(account_of(&p1)),
                sfid.clone(),
                2,
                admins1.clone(),
                2,
                200,
                create_approvals_1
            ));

            let close_payload_1 = close_payload(
                &duoqian,
                &beneficiary,
                2u32,
                2u32,
                DEFAULT_MIN_CLOSE_BALANCE,
                DEFAULT_EXPIRES_AT,
            );
            let close_approvals_1 = approvals_vec(vec![
                AdminApproval {
                    public_key: public_of(&p1),
                    signature: sign(&p1, &close_payload_1),
                },
                AdminApproval {
                    public_key: public_of(&p2),
                    signature: sign(&p2, &close_payload_1),
                },
            ]);
            assert_ok!(call_close(
                RuntimeOrigin::signed(account_of(&p1)),
                duoqian.clone(),
                beneficiary.clone(),
                close_approvals_1
            ));

            // 重建后再次尝试重放旧 close 签名，必须失败（nonce 已变化）
            let admins2 = admins_vec(vec![public_of(&p3), public_of(&p4)]);
            let create_payload_2 =
                create_payload(&sfid, &duoqian, 2u32, &admins2, 2u32, 111u128, DEFAULT_EXPIRES_AT);
            let create_approvals_2 = approvals_vec(vec![
                AdminApproval {
                    public_key: public_of(&p3),
                    signature: sign(&p3, &create_payload_2),
                },
                AdminApproval {
                    public_key: public_of(&p4),
                    signature: sign(&p4, &create_payload_2),
                },
            ]);
            assert_ok!(call_create(
                RuntimeOrigin::signed(account_of(&p3)),
                sfid,
                2,
                admins2,
                2,
                111,
                create_approvals_2
            ));

            let replay_old_close = approvals_vec(vec![
                AdminApproval {
                    public_key: public_of(&p1),
                    signature: sign(&p1, &close_payload_1),
                },
                AdminApproval {
                    public_key: public_of(&p2),
                    signature: sign(&p2, &close_payload_1),
                },
            ]);
            assert_noop!(
                call_close(
                    RuntimeOrigin::signed(account_of(&p3)),
                    duoqian,
                    beneficiary,
                    replay_old_close
                ),
                Error::<Test>::PermissionDenied
            );
        });
    }

    #[test]
    fn create_duoqian_requires_registered_sfid_address() {
        new_test_ext().execute_with(|| {
            let p1 = pair(1);
            let p2 = pair(2);
            let sfid: SfidIdOf<Test> = b"GFR-LN001-CB0C-unregistered-20260222"
                .to_vec()
                .try_into()
                .expect("sfid id should fit");
            let duoqian = account_of(&pair(9));
            let admins = admins_vec(vec![public_of(&p1), public_of(&p2)]);
            let payload = create_payload(&sfid, &duoqian, 2u32, &admins, 2u32, 111u128, DEFAULT_EXPIRES_AT);
            let approvals = approvals_vec(vec![AdminApproval {
                public_key: public_of(&p1),
                signature: sign(&p1, &payload),
            }]);

            assert_noop!(
                call_create(
                    RuntimeOrigin::signed(account_of(&p1)),
                    sfid,
                    2,
                    admins,
                2,
                    111,
                approvals
            ),
                Error::<Test>::InstitutionNotRegistered
            );
        });
    }

    #[test]
    fn create_rejects_expired_signatures() {
        new_test_ext().execute_with(|| {
            let p1 = pair(1);
            let p2 = pair(2);
            let (sfid, duoqian) = register_sfid_and_get_address("expired-sig");
            let admins = admins_vec(vec![public_of(&p1), public_of(&p2)]);

            let payload = create_payload(&sfid, &duoqian, 2u32, &admins, 2u32, 111u128, DEFAULT_EXPIRES_AT);
            let approvals = approvals_vec(vec![
                AdminApproval {
                    public_key: public_of(&p1),
                    signature: sign(&p1, &payload),
                },
                AdminApproval {
                    public_key: public_of(&p2),
                    signature: sign(&p2, &payload),
                },
            ]);

            System::set_block_number(DEFAULT_EXPIRES_AT + 1);
            assert_noop!(
                call_create(
                    RuntimeOrigin::signed(account_of(&p1)),
                    sfid,
                    2,
                    admins,
                    2,
                    111,
                    approvals
                ),
                Error::<Test>::SignatureExpired
            );
        });
    }

    #[test]
    fn create_rejects_nonce_overflow() {
        new_test_ext().execute_with(|| {
            let p1 = pair(1);
            let p2 = pair(2);
            let (sfid, duoqian) = register_sfid_and_get_address("nonce-overflow");
            AddressRegisteredSfid::<Test>::mutate(&duoqian, |entry| {
                if let Some(e) = entry {
                    e.nonce = u64::MAX;
                }
            });
            let admins = admins_vec(vec![public_of(&p1), public_of(&p2)]);

            let payload = create_payload(&sfid, &duoqian, 2u32, &admins, 2u32, 111u128, DEFAULT_EXPIRES_AT);
            let approvals = approvals_vec(vec![
                AdminApproval {
                    public_key: public_of(&p1),
                    signature: sign(&p1, &payload),
                },
                AdminApproval {
                    public_key: public_of(&p2),
                    signature: sign(&p2, &payload),
                },
            ]);

            assert_noop!(
                call_create(
                    RuntimeOrigin::signed(account_of(&p1)),
                    sfid,
                    2,
                    admins,
                    2,
                    111,
                    approvals
                ),
                Error::<Test>::NonceOverflow
            );
        });
    }

    #[test]
    fn register_sfid_institution_derives_blake3_address_and_blocks_duplicate_sfid() {
        new_test_ext().execute_with(|| {
            let sfid: SfidIdOf<Test> = b"GFR-LN001-CB0C-617776487-20260222"
                .to_vec()
                .try_into()
                .expect("fit");
            assert_ok!(Duoqian::register_sfid_institution(
                RuntimeOrigin::signed(AccountId32::new([0x55; 32])),
                sfid.clone()
            ));
            let expected = Duoqian::derive_duoqian_address_from_sfid_id(sfid.as_slice())
                .expect("must derive");
            assert_eq!(Duoqian::sfid_registered_address(sfid.clone()), Some(expected));

            assert_noop!(
                Duoqian::register_sfid_institution(
                    RuntimeOrigin::signed(AccountId32::new([0x55; 32])),
                    sfid
                ),
                Error::<Test>::SfidAlreadyRegistered
            );
        });
    }
}
