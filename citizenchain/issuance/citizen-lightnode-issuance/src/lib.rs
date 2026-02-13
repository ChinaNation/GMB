#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub trait CiicVerifier<AccountId> {
    fn verify(ciic: &[u8], account: &AccountId) -> bool;
}

impl<AccountId> CiicVerifier<AccountId> for () {
    fn verify(_ciic: &[u8], _account: &AccountId) -> bool {
        false
    }
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        pallet_prelude::*,
        traits::Currency,
        Blake2_128Concat,
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{Hash, SaturatedConversion, Saturating};

    use primitives::citizen_const::{
        CITIZEN_LIGHTNODE_HIGH_REWARD,
        CITIZEN_LIGHTNODE_HIGH_REWARD_COUNT,
        CITIZEN_LIGHTNODE_MAX_COUNT,
        CITIZEN_LIGHTNODE_NORMAL_REWARD,
        CITIZEN_LIGHTNODE_ONE_TIME_ONLY,
    };

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<
            <T as frame_system::Config>::AccountId,
        >>::Balance;

    pub type CiicOf<T> = BoundedVec<u8, <T as Config>::MaxCiicLength>;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>>
            + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Currency: Currency<Self::AccountId>;

        #[pallet::constant]
        type MaxCiicLength: Get<u32>;

        #[pallet::constant]
        type VoteCooldownBlocks: Get<BlockNumberFor<Self>>;

        type CiicVerifier: CiicVerifier<Self::AccountId>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn rewarded_count)]
    pub type RewardedCount<T> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn ciic_to_account)]
    pub type CiicToAccount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, T::AccountId, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn account_to_ciic)]
    pub type AccountToCiic<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, T::Hash, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn reward_claimed)]
    pub type RewardClaimed<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, bool, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn vote_cooldown_until)]
    pub type VoteCooldownUntil<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BlockNumberFor<T>, ValueQuery>;

    /// V1: CIIC 系统尚未上线时，使用链上白名单做人工核验兜底。
    #[pallet::storage]
    #[pallet::getter(fn ciic_allowlisted)]
    pub type CiicAllowlist<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, bool, ValueQuery>;

    /// false: 使用白名单; true: 使用外部验证器（V2）。
    #[pallet::storage]
    #[pallet::getter(fn use_external_ciic_verifier)]
    pub type UseExternalCiicVerifier<T> = StorageValue<_, bool, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        CitizenLightnodeBound {
            who: T::AccountId,
            ciic_hash: T::Hash,
            cooldown_until: BlockNumberFor<T>,
            reward: BalanceOf<T>,
        },
        CitizenLightnodeUnbound {
            who: T::AccountId,
            ciic_hash: T::Hash,
        },
        CiicAllowlistUpdated {
            ciic_hash: T::Hash,
            allowed: bool,
        },
        ExternalCiicVerifierModeUpdated {
            enabled: bool,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        EmptyCiic,
        CiicNotAuthorized,
        CiicAlreadyBoundToAnotherAccount,
        SameCiicAlreadyBound,
        NotBound,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 绑定（或重绑定）CIIC 到当前签名账户，并按制度自动发放认证奖励。
        /// 奖励发放遵守分段、总量封顶与“每个 CIIC 仅一次”规则。
        #[pallet::call_index(0)]
        #[pallet::weight(T::DbWeight::get().reads_writes(8, 8))]
        pub fn bind_ciic(origin: OriginFor<T>, ciic: CiicOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(!ciic.is_empty(), Error::<T>::EmptyCiic);
            let ciic_hash = T::Hashing::hash(ciic.as_slice());

            ensure!(
                Self::is_ciic_authorized(ciic.as_slice(), &ciic_hash, &who),
                Error::<T>::CiicNotAuthorized
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
            }

            CiicToAccount::<T>::insert(ciic_hash, &who);
            AccountToCiic::<T>::insert(&who, ciic_hash);

            let now = <frame_system::Pallet<T>>::block_number();
            let cooldown_until = now.saturating_add(T::VoteCooldownBlocks::get());
            VoteCooldownUntil::<T>::insert(&who, cooldown_until);

            let reward = Self::try_issue_certification_reward(&who, ciic_hash);
            Self::deposit_event(Event::<T>::CitizenLightnodeBound {
                who,
                ciic_hash,
                cooldown_until,
                reward,
            });
            Ok(())
        }

        /// 解绑当前账户的 CIIC。
        #[pallet::call_index(1)]
        #[pallet::weight(T::DbWeight::get().reads_writes(2, 2))]
        pub fn unbind_ciic(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let ciic_hash =
                AccountToCiic::<T>::get(&who).ok_or(Error::<T>::NotBound)?;

            AccountToCiic::<T>::remove(&who);
            CiicToAccount::<T>::remove(ciic_hash);

            Self::deposit_event(Event::<T>::CitizenLightnodeUnbound {
                who,
                ciic_hash,
            });
            Ok(())
        }

        /// V1 模式下，治理方将 CIIC 加入/移出白名单。
        #[pallet::call_index(2)]
        #[pallet::weight(T::DbWeight::get().reads_writes(0, 1))]
        pub fn set_ciic_allowlist(
            origin: OriginFor<T>,
            ciic_hash: T::Hash,
            allowed: bool,
        ) -> DispatchResult {
            ensure_root(origin)?;
            CiicAllowlist::<T>::insert(ciic_hash, allowed);
            Self::deposit_event(Event::<T>::CiicAllowlistUpdated {
                ciic_hash,
                allowed,
            });
            Ok(())
        }

        /// 切换验证模式：false=白名单(V1)，true=外部验证器(V2)。
        #[pallet::call_index(3)]
        #[pallet::weight(T::DbWeight::get().reads_writes(0, 1))]
        pub fn set_external_ciic_verifier_mode(
            origin: OriginFor<T>,
            enabled: bool,
        ) -> DispatchResult {
            ensure_root(origin)?;
            UseExternalCiicVerifier::<T>::put(enabled);
            Self::deposit_event(Event::<T>::ExternalCiicVerifierModeUpdated {
                enabled,
            });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn is_ciic_authorized(
            ciic: &[u8],
            ciic_hash: &T::Hash,
            who: &T::AccountId,
        ) -> bool {
            if UseExternalCiicVerifier::<T>::get() {
                return T::CiicVerifier::verify(ciic, who);
            }
            CiicAllowlist::<T>::get(ciic_hash)
        }

        fn try_issue_certification_reward(
            who: &T::AccountId,
            ciic_hash: T::Hash,
        ) -> BalanceOf<T> {
            if CITIZEN_LIGHTNODE_ONE_TIME_ONLY
                && RewardClaimed::<T>::get(ciic_hash)
            {
                return 0u128.saturated_into();
            }

            let rewarded_count = RewardedCount::<T>::get();
            if rewarded_count >= CITIZEN_LIGHTNODE_MAX_COUNT {
                return 0u128.saturated_into();
            }

            let reward_amount = if rewarded_count < CITIZEN_LIGHTNODE_HIGH_REWARD_COUNT {
                CITIZEN_LIGHTNODE_HIGH_REWARD
            } else {
                CITIZEN_LIGHTNODE_NORMAL_REWARD
            };

            let reward: BalanceOf<T> = reward_amount.saturated_into();
            let _imbalance = T::Currency::deposit_creating(who, reward);

            RewardedCount::<T>::put(rewarded_count.saturating_add(1));
            if CITIZEN_LIGHTNODE_ONE_TIME_ONLY {
                RewardClaimed::<T>::insert(ciic_hash, true);
            }
            reward
        }

        /// 对外查询：账户是否可参与投票。
        pub fn can_vote_now(who: &T::AccountId) -> bool {
            let Some(_) = AccountToCiic::<T>::get(who) else {
                return false;
            };

            let now = <frame_system::Pallet<T>>::block_number();
            now >= VoteCooldownUntil::<T>::get(who)
        }

        /// 对外查询：账户是否存在 CIIC 绑定。
        pub fn is_bound(who: &T::AccountId) -> bool {
            AccountToCiic::<T>::contains_key(who)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{
        assert_noop, assert_ok, derive_impl, parameter_types,
        traits::{ConstU128, ConstU32, VariantCountOf},
    };
    use frame_system as system;
    use primitives::citizen_const::{
        CITIZEN_LIGHTNODE_HIGH_REWARD,
        CITIZEN_LIGHTNODE_HIGH_REWARD_COUNT,
        CITIZEN_LIGHTNODE_MAX_COUNT,
        CITIZEN_LIGHTNODE_NORMAL_REWARD,
    };
    use sp_runtime::{BuildStorage, traits::{Hash, IdentityLookup}};

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
        pub type CitizenLightnodeIssuance = super;
    }

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl system::Config for Test {
        type Block = Block;
        type AccountId = u64;
        type AccountData = pallet_balances::AccountData<u128>;
        type Lookup = IdentityLookup<Self::AccountId>;
    }

    impl pallet_balances::Config for Test {
        type MaxLocks = ConstU32<0>;
        type MaxReserves = ConstU32<0>;
        type ReserveIdentifier = [u8; 8];
        type Balance = u128;
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

    parameter_types! {
        pub const MaxCiicLength: u32 = 64;
        pub const VoteCooldownBlocks: u64 = 35;
    }

    impl Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type Currency = Balances;
        type MaxCiicLength = MaxCiicLength;
        type VoteCooldownBlocks = VoteCooldownBlocks;
        type CiicVerifier = ();
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("frame system genesis storage should build");
        let mut ext = sp_io::TestExternalities::new(storage);
        ext.execute_with(|| {
            System::set_block_number(1);
        });
        ext
    }

    fn ciic(input: &str) -> CiicOf<Test> {
        input.as_bytes().to_vec().try_into().expect("ciic should fit max length")
    }

    fn ciic_hash(input: &str) -> <Test as frame_system::Config>::Hash {
        <Test as frame_system::Config>::Hashing::hash(input.as_bytes())
    }

    #[test]
    fn reward_changes_from_high_to_normal_at_boundary() {
        new_test_ext().execute_with(|| {
            RewardedCount::<Test>::put(CITIZEN_LIGHTNODE_HIGH_REWARD_COUNT - 1);
            assert_ok!(CitizenLightnodeIssuance::set_ciic_allowlist(
                RuntimeOrigin::root(),
                ciic_hash("ciic-a"),
                true
            ));
            assert_ok!(CitizenLightnodeIssuance::set_ciic_allowlist(
                RuntimeOrigin::root(),
                ciic_hash("ciic-b"),
                true
            ));

            assert_ok!(CitizenLightnodeIssuance::bind_ciic(
                RuntimeOrigin::signed(1),
                ciic("ciic-a")
            ));
            assert_ok!(CitizenLightnodeIssuance::unbind_ciic(RuntimeOrigin::signed(1)));
            assert_ok!(CitizenLightnodeIssuance::bind_ciic(
                RuntimeOrigin::signed(1),
                ciic("ciic-b")
            ));

            let balance = Balances::free_balance(1);
            assert_eq!(
                balance,
                CITIZEN_LIGHTNODE_HIGH_REWARD + CITIZEN_LIGHTNODE_NORMAL_REWARD
            );
        });
    }

    #[test]
    fn same_ciic_only_gets_reward_once() {
        new_test_ext().execute_with(|| {
            assert_ok!(CitizenLightnodeIssuance::set_ciic_allowlist(
                RuntimeOrigin::root(),
                ciic_hash("only-once"),
                true
            ));

            assert_ok!(CitizenLightnodeIssuance::bind_ciic(
                RuntimeOrigin::signed(1),
                ciic("only-once")
            ));
            let first = Balances::free_balance(1);
            assert_eq!(first, CITIZEN_LIGHTNODE_HIGH_REWARD);

            assert_ok!(CitizenLightnodeIssuance::unbind_ciic(RuntimeOrigin::signed(1)));
            assert_ok!(CitizenLightnodeIssuance::bind_ciic(
                RuntimeOrigin::signed(1),
                ciic("only-once")
            ));

            let second = Balances::free_balance(1);
            assert_eq!(second, first);
        });
    }

    #[test]
    fn no_reward_after_total_cap_reached() {
        new_test_ext().execute_with(|| {
            RewardedCount::<Test>::put(CITIZEN_LIGHTNODE_MAX_COUNT);
            assert_ok!(CitizenLightnodeIssuance::set_ciic_allowlist(
                RuntimeOrigin::root(),
                ciic_hash("capped"),
                true
            ));

            assert_ok!(CitizenLightnodeIssuance::bind_ciic(
                RuntimeOrigin::signed(1),
                ciic("capped")
            ));
            assert_eq!(Balances::free_balance(1), 0);
            assert_eq!(RewardedCount::<Test>::get(), CITIZEN_LIGHTNODE_MAX_COUNT);
        });
    }

    #[test]
    fn rebinding_starts_new_vote_cooldown() {
        new_test_ext().execute_with(|| {
            assert_ok!(CitizenLightnodeIssuance::set_ciic_allowlist(
                RuntimeOrigin::root(),
                ciic_hash("cool-1"),
                true
            ));
            assert_ok!(CitizenLightnodeIssuance::set_ciic_allowlist(
                RuntimeOrigin::root(),
                ciic_hash("cool-2"),
                true
            ));

            assert_ok!(CitizenLightnodeIssuance::bind_ciic(
                RuntimeOrigin::signed(1),
                ciic("cool-1")
            ));
            let first_cooldown = VoteCooldownUntil::<Test>::get(1);
            assert!(!CitizenLightnodeIssuance::can_vote_now(&1));

            System::set_block_number(10);
            assert_ok!(CitizenLightnodeIssuance::unbind_ciic(RuntimeOrigin::signed(1)));
            assert_ok!(CitizenLightnodeIssuance::bind_ciic(
                RuntimeOrigin::signed(1),
                ciic("cool-2")
            ));
            let second_cooldown = VoteCooldownUntil::<Test>::get(1);
            assert!(second_cooldown > first_cooldown);
            assert!(!CitizenLightnodeIssuance::can_vote_now(&1));

            System::set_block_number(second_cooldown);
            assert!(CitizenLightnodeIssuance::can_vote_now(&1));
        });
    }

    #[test]
    fn empty_ciic_is_rejected() {
        new_test_ext().execute_with(|| {
            let empty: CiicOf<Test> = Vec::<u8>::new()
                .try_into()
                .expect("empty vec should fit bounded ciic");
            assert_noop!(
                CitizenLightnodeIssuance::bind_ciic(
                    RuntimeOrigin::signed(1),
                    empty
                ),
                Error::<Test>::EmptyCiic
            );
        });
    }

    #[test]
    fn same_ciic_cannot_bind_to_another_account() {
        new_test_ext().execute_with(|| {
            assert_ok!(CitizenLightnodeIssuance::set_ciic_allowlist(
                RuntimeOrigin::root(),
                ciic_hash("one-owner"),
                true
            ));

            assert_ok!(CitizenLightnodeIssuance::bind_ciic(
                RuntimeOrigin::signed(1),
                ciic("one-owner")
            ));

            assert_noop!(
                CitizenLightnodeIssuance::bind_ciic(
                    RuntimeOrigin::signed(2),
                    ciic("one-owner")
                ),
                Error::<Test>::CiicAlreadyBoundToAnotherAccount
            );
        });
    }
}
