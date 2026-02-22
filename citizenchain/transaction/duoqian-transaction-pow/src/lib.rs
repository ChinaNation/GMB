#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    ensure,
    pallet_prelude::*,
    traits::{Currency, ExistenceRequirement},
    BoundedVec,
};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
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

#[derive(Encode, Decode, DecodeWithMemTracking, Clone, RuntimeDebug, TypeInfo, PartialEq, Eq)]
pub struct DuoqianAccount<PublicKey, AccountId, BlockNumber> {
    pub admin_count: u32,
    pub threshold: u32,
    pub duoqian_admins: Vec<PublicKey>,
    pub creator: AccountId,
    pub created_at: BlockNumber,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Currency: Currency<Self::AccountId>;

        type AdminAuth: DuoqianAdminAuth<Self::AccountId>;

        type AddressValidator: DuoqianAddressValidator<Self::AccountId>;

        #[pallet::constant]
        type MaxAdmins: Get<u32>;

        /// 创建时最低入金（默认应设置为 111 分 = 1.11 元）。
        #[pallet::constant]
        type MinCreateAmount: Get<BalanceOf<Self>>;

        /// 注销时账户最低余额门槛（默认应设置为 111 分 = 1.11 元）。
        #[pallet::constant]
        type MinCloseBalance: Get<BalanceOf<Self>>;
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

    pub type DuoqianAccountOf<T> =
        DuoqianAccount<
            <<T as Config>::AdminAuth as DuoqianAdminAuth<
                <T as frame_system::Config>::AccountId,
            >>::PublicKey,
            <T as frame_system::Config>::AccountId,
            BlockNumberFor<T>,
        >;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// 多签账户配置。key 为 duoqian_address。
    #[pallet::storage]
    #[pallet::getter(fn duoqian_account_of)]
    pub type DuoqianAccounts<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, DuoqianAccountOf<T>, OptionQuery>;

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
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 参数不完整
        IncompleteParameters,
        /// 地址非法
        InvalidAddress,
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
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 创建多签账户：
        /// - 参数必须完整；
        /// - N>=2，M>=ceil(N/2) 且 M<=N；
        /// - duoqian_admins 去重且长度等于 N；
        /// - duoqian_address 必须为本链合法地址且当前未被占用；
        /// - 发起人必须是管理员之一；
        /// - 管理员有效签名数必须 >= M；
        /// - 创建时转入金额必须 >= MinCreateAmount（建议 111 分）。
        #[pallet::call_index(0)]
        #[pallet::weight(T::DbWeight::get().reads_writes(6, 2))]
        pub fn create_duoqian(
            origin: OriginFor<T>,
            duoqian_address: T::AccountId,
            admin_count: u32,
            duoqian_admins: DuoqianAdminsOf<T>,
            threshold: u32,
            amount: BalanceOf<T>,
            approvals: AdminApprovalsOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(!duoqian_admins.is_empty(), Error::<T>::IncompleteParameters);
            ensure!(admin_count > 0, Error::<T>::IncompleteParameters);
            ensure!(threshold > 0, Error::<T>::IncompleteParameters);
            ensure!(
                amount >= T::MinCreateAmount::get(),
                Error::<T>::InsufficientAmount
            );

            ensure!(admin_count >= 2, Error::<T>::InvalidAdminCount);
            ensure!(
                duoqian_admins.len() as u32 == admin_count,
                Error::<T>::AdminCountMismatch
            );

            ensure!(
                T::AddressValidator::is_valid(&duoqian_address),
                Error::<T>::InvalidAddress
            );
            ensure!(
                !DuoqianAccounts::<T>::contains_key(&duoqian_address),
                Error::<T>::AddressAlreadyExists
            );
            ensure!(
                !frame_system::Account::<T>::contains_key(&duoqian_address),
                Error::<T>::AddressAlreadyOnChain
            );

            let min_threshold = admin_count.saturating_add(1) / 2;
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

            let payload = (
                b"DUOQIAN_CREATE_V1".to_vec(),
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
            )
            .map_err(|_| Error::<T>::InsufficientAmount)?;

            DuoqianAccounts::<T>::insert(
                &duoqian_address,
                DuoqianAccount {
                    admin_count,
                    threshold,
                    duoqian_admins: duoqian_admins.to_vec(),
                    creator: who.clone(),
                    created_at: frame_system::Pallet::<T>::block_number(),
                },
            );

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
        #[pallet::weight(T::DbWeight::get().reads_writes(4, 2))]
        pub fn close_duoqian(
            origin: OriginFor<T>,
            duoqian_address: T::AccountId,
            beneficiary: T::AccountId,
            approvals: AdminApprovalsOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                beneficiary != duoqian_address,
                Error::<T>::InvalidBeneficiary
            );

            let account =
                DuoqianAccounts::<T>::get(&duoqian_address).ok_or(Error::<T>::DuoqianNotFound)?;
            let admin_count = account.admin_count;
            let threshold = account.threshold;
            let admins: DuoqianAdminsOf<T> = account
                .duoqian_admins
                .clone()
                .try_into()
                .map_err(|_| Error::<T>::IncompleteParameters)?;

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

            let payload = (
                b"DUOQIAN_CLOSE_V1".to_vec(),
                &duoqian_address,
                &beneficiary,
                admin_count,
                threshold,
                all_balance,
            )
                .encode();
            let signed = Self::count_valid_signatures(&admins, &approvals, &payload)?;
            ensure!(signed >= threshold, Error::<T>::InsufficientSignatures);

            T::Currency::transfer(
                &duoqian_address,
                &beneficiary,
                all_balance,
                ExistenceRequirement::AllowDeath,
            )
            .map_err(|_| Error::<T>::InsufficientAmount)?;

            DuoqianAccounts::<T>::remove(&duoqian_address);

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
        type MaxAdmins = ConstU32<10>;
        type MinCreateAmount = ConstU128<111>;
        type MinCloseBalance = ConstU128<111>;
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

    #[test]
    fn create_duoqian_works_and_locks_config() {
        new_test_ext().execute_with(|| {
            let p1 = pair(1);
            let p2 = pair(2);
            let duoqian = account_of(&pair(9));

            let admins = admins_vec(vec![public_of(&p1), public_of(&p2)]);
            let payload = (
                b"DUOQIAN_CREATE_V1".to_vec(),
                &duoqian,
                2u32,
                &admins,
                1u32,
                111u128,
            )
                .encode();
            let approvals = approvals_vec(vec![AdminApproval {
                public_key: public_of(&p1),
                signature: sign(&p1, &payload),
            }]);

            assert_ok!(Duoqian::create_duoqian(
                RuntimeOrigin::signed(account_of(&p1)),
                duoqian.clone(),
                2,
                admins,
                1,
                111,
                approvals
            ));

            let config = DuoqianAccounts::<Test>::get(&duoqian).expect("must exist");
            assert_eq!(config.admin_count, 2);
            assert_eq!(config.threshold, 1);
            assert_eq!(Balances::free_balance(&duoqian), 111);
        });
    }

    #[test]
    fn create_duoqian_rejects_duplicate_admins() {
        new_test_ext().execute_with(|| {
            let p1 = pair(1);
            let duoqian = account_of(&pair(9));
            let duplicated = public_of(&p1);

            let admins = admins_vec(vec![duplicated, duplicated]);
            let payload = (
                b"DUOQIAN_CREATE_V1".to_vec(),
                &duoqian,
                2u32,
                &admins,
                1u32,
                111u128,
            )
                .encode();
            let approvals = approvals_vec(vec![AdminApproval {
                public_key: duplicated,
                signature: sign(&p1, &payload),
            }]);

            assert_noop!(
                Duoqian::create_duoqian(
                    RuntimeOrigin::signed(account_of(&p1)),
                    duoqian,
                    2,
                    admins,
                    1,
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
            let duoqian = account_of(&pair(9));

            let admins = admins_vec(vec![public_of(&p1), public_of(&p2)]);
            let payload = (
                b"DUOQIAN_CREATE_V1".to_vec(),
                &duoqian,
                2u32,
                &admins,
                0u32,
                111u128,
            )
                .encode();
            let approvals = approvals_vec(vec![AdminApproval {
                public_key: public_of(&p1),
                signature: sign(&p1, &payload),
            }]);

            assert_noop!(
                Duoqian::create_duoqian(
                    RuntimeOrigin::signed(account_of(&p1)),
                    duoqian,
                    2,
                    admins,
                    0,
                    111,
                    approvals
                ),
                Error::<Test>::IncompleteParameters
            );
        });
    }

    #[test]
    fn create_duoqian_requires_half_or_more_signatures() {
        new_test_ext().execute_with(|| {
            let p1 = pair(1);
            let p2 = pair(2);
            let p3 = pair(3);
            let duoqian = account_of(&pair(9));

            let admins = admins_vec(vec![public_of(&p1), public_of(&p2), public_of(&p3)]);
            let payload = (
                b"DUOQIAN_CREATE_V1".to_vec(),
                &duoqian,
                3u32,
                &admins,
                2u32,
                111u128,
            )
                .encode();
            let approvals = approvals_vec(vec![AdminApproval {
                public_key: public_of(&p1),
                signature: sign(&p1, &payload),
            }]);

            assert_noop!(
                Duoqian::create_duoqian(
                    RuntimeOrigin::signed(account_of(&p1)),
                    duoqian,
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
            let duoqian = account_of(&pair(9));
            let beneficiary = account_of(&pair(8));

            // first create: admins p1,p2 threshold 1
            let admins1 = admins_vec(vec![public_of(&p1), public_of(&p2)]);
            let create_payload_1 = (
                b"DUOQIAN_CREATE_V1".to_vec(),
                &duoqian,
                2u32,
                &admins1,
                1u32,
                200u128,
            )
                .encode();
            let approvals_1 = approvals_vec(vec![AdminApproval {
                public_key: public_of(&p1),
                signature: sign(&p1, &create_payload_1),
            }]);
            assert_ok!(Duoqian::create_duoqian(
                RuntimeOrigin::signed(account_of(&p1)),
                duoqian.clone(),
                2,
                admins1,
                1,
                200,
                approvals_1
            ));

            let close_payload = (
                b"DUOQIAN_CLOSE_V1".to_vec(),
                &duoqian,
                &beneficiary,
                2u32,
                1u32,
                200u128,
            )
                .encode();
            let close_approvals = approvals_vec(vec![AdminApproval {
                public_key: public_of(&p2),
                signature: sign(&p2, &close_payload),
            }]);
            assert_ok!(Duoqian::close_duoqian(
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
            let create_payload_2 = (
                b"DUOQIAN_CREATE_V1".to_vec(),
                &duoqian,
                2u32,
                &admins2,
                2u32,
                111u128,
            )
                .encode();
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
            assert_ok!(Duoqian::create_duoqian(
                RuntimeOrigin::signed(account_of(&p3)),
                duoqian.clone(),
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
            let duoqian = account_of(&pair(9));

            let admins = admins_vec(vec![public_of(&p1), public_of(&p2)]);
            let payload = (
                b"DUOQIAN_CREATE_V1".to_vec(),
                &duoqian,
                3u32,
                &admins,
                2u32,
                111u128,
            )
                .encode();
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
                Duoqian::create_duoqian(
                    RuntimeOrigin::signed(account_of(&p1)),
                    duoqian,
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
            let duoqian = account_of(&pair(9));

            let admins = admins_vec(vec![public_of(&p1), public_of(&p2)]);
            let payload = (
                b"DUOQIAN_CREATE_V1".to_vec(),
                &duoqian,
                2u32,
                &admins,
                1u32,
                111u128,
            )
                .encode();
            let approvals = approvals_vec(vec![AdminApproval {
                public_key: public_of(&p1),
                signature: sign(&p1, &payload),
            }]);

            assert_noop!(
                Duoqian::create_duoqian(
                    RuntimeOrigin::signed(account_of(&outsider)),
                    duoqian,
                    2,
                    admins,
                    1,
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
            let duoqian = account_of(&pair(9));

            let admins = admins_vec(vec![public_of(&p1), public_of(&p2)]);
            let payload = (
                b"DUOQIAN_CREATE_V1".to_vec(),
                &duoqian,
                2u32,
                &admins,
                1u32,
                111u128,
            )
                .encode();
            let approvals = approvals_vec(vec![AdminApproval {
                public_key: public_of(&outsider),
                signature: sign(&outsider, &payload),
            }]);

            assert_noop!(
                Duoqian::create_duoqian(
                    RuntimeOrigin::signed(account_of(&p1)),
                    duoqian,
                    2,
                    admins,
                    1,
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
            let duoqian = account_of(&pair(9));

            let admins = admins_vec(vec![public_of(&p1), public_of(&p2)]);
            let payload = (
                b"DUOQIAN_CREATE_V1".to_vec(),
                &duoqian,
                2u32,
                &admins,
                1u32,
                111u128,
            )
                .encode();
            // 使用错误签名者 p2 对 p1 公钥字段造签名，应该失败
            let approvals = approvals_vec(vec![AdminApproval {
                public_key: public_of(&p1),
                signature: sign(&p2, &payload),
            }]);

            assert_noop!(
                Duoqian::create_duoqian(
                    RuntimeOrigin::signed(account_of(&p1)),
                    duoqian,
                    2,
                    admins,
                    1,
                    111,
                    approvals
                ),
                Error::<Test>::InvalidAdminSignature
            );
        });
    }

    #[test]
    fn create_duoqian_rejects_address_already_on_chain() {
        new_test_ext().execute_with(|| {
            let p1 = pair(1);
            let p2 = pair(2);
            let duoqian = account_of(&p3_for_existing_account());
            let _ = Balances::deposit_creating(&duoqian, 50);

            let admins = admins_vec(vec![public_of(&p1), public_of(&p2)]);
            let payload = (
                b"DUOQIAN_CREATE_V1".to_vec(),
                &duoqian,
                2u32,
                &admins,
                1u32,
                111u128,
            )
                .encode();
            let approvals = approvals_vec(vec![AdminApproval {
                public_key: public_of(&p1),
                signature: sign(&p1, &payload),
            }]);

            assert_noop!(
                Duoqian::create_duoqian(
                    RuntimeOrigin::signed(account_of(&p1)),
                    duoqian,
                    2,
                    admins,
                    1,
                    111,
                    approvals
                ),
                Error::<Test>::AddressAlreadyOnChain
            );
        });
    }

    #[test]
    fn close_duoqian_rejects_beneficiary_equal_self() {
        new_test_ext().execute_with(|| {
            let p1 = pair(1);
            let p2 = pair(2);
            let duoqian = account_of(&pair(9));

            let admins = admins_vec(vec![public_of(&p1), public_of(&p2)]);
            let create_payload = (
                b"DUOQIAN_CREATE_V1".to_vec(),
                &duoqian,
                2u32,
                &admins,
                1u32,
                200u128,
            )
                .encode();
            let create_approvals = approvals_vec(vec![AdminApproval {
                public_key: public_of(&p1),
                signature: sign(&p1, &create_payload),
            }]);
            assert_ok!(Duoqian::create_duoqian(
                RuntimeOrigin::signed(account_of(&p1)),
                duoqian.clone(),
                2,
                admins.clone(),
                1,
                200,
                create_approvals
            ));

            let close_payload = (
                b"DUOQIAN_CLOSE_V1".to_vec(),
                &duoqian,
                &duoqian,
                2u32,
                1u32,
                200u128,
            )
                .encode();
            let close_approvals = approvals_vec(vec![AdminApproval {
                public_key: public_of(&p2),
                signature: sign(&p2, &close_payload),
            }]);

            assert_noop!(
                Duoqian::close_duoqian(
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
    fn close_duoqian_rejects_non_admin_submitter() {
        new_test_ext().execute_with(|| {
            let p1 = pair(1);
            let p2 = pair(2);
            let outsider = pair(7);
            let duoqian = account_of(&pair(9));
            let beneficiary = account_of(&pair(8));

            let admins = admins_vec(vec![public_of(&p1), public_of(&p2)]);
            let create_payload = (
                b"DUOQIAN_CREATE_V1".to_vec(),
                &duoqian,
                2u32,
                &admins,
                1u32,
                200u128,
            )
                .encode();
            let create_approvals = approvals_vec(vec![AdminApproval {
                public_key: public_of(&p1),
                signature: sign(&p1, &create_payload),
            }]);
            assert_ok!(Duoqian::create_duoqian(
                RuntimeOrigin::signed(account_of(&p1)),
                duoqian.clone(),
                2,
                admins.clone(),
                1,
                200,
                create_approvals
            ));

            let close_payload = (
                b"DUOQIAN_CLOSE_V1".to_vec(),
                &duoqian,
                &beneficiary,
                2u32,
                1u32,
                200u128,
            )
                .encode();
            let close_approvals = approvals_vec(vec![AdminApproval {
                public_key: public_of(&p1),
                signature: sign(&p1, &close_payload),
            }]);

            assert_noop!(
                Duoqian::close_duoqian(
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
            let duoqian_a = account_of(&pair(9));
            let duoqian_b = account_of(&pair(10));

            let admins = admins_vec(vec![public_of(&p1), public_of(&p2)]);
            let create_payload = (
                b"DUOQIAN_CREATE_V1".to_vec(),
                &duoqian_a,
                2u32,
                &admins,
                1u32,
                300u128,
            )
                .encode();
            let create_approvals = approvals_vec(vec![AdminApproval {
                public_key: public_of(&p1),
                signature: sign(&p1, &create_payload),
            }]);
            assert_ok!(Duoqian::create_duoqian(
                RuntimeOrigin::signed(account_of(&p1)),
                duoqian_a.clone(),
                2,
                admins.clone(),
                1,
                300,
                create_approvals
            ));

            let close_payload = (
                b"DUOQIAN_CLOSE_V1".to_vec(),
                &duoqian_a,
                &duoqian_b,
                2u32,
                1u32,
                300u128,
            )
                .encode();
            let close_approvals = approvals_vec(vec![AdminApproval {
                public_key: public_of(&p2),
                signature: sign(&p2, &close_payload),
            }]);
            assert_ok!(Duoqian::close_duoqian(
                RuntimeOrigin::signed(account_of(&p2)),
                duoqian_a.clone(),
                duoqian_b.clone(),
                close_approvals
            ));

            assert_eq!(Balances::free_balance(&duoqian_a), 0);
            assert_eq!(Balances::free_balance(&duoqian_b), 300);
        });
    }

    fn p3_for_existing_account() -> sr25519::Pair {
        pair(33)
    }
}
