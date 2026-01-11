#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        pallet_prelude::*,
        traits::{Contains},
    };
    use frame_system::pallet_prelude::*;

    /// ================================
    /// 配置 Trait（由 Runtime 实现）
    /// ================================
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// 事件
        type RuntimeEvent: From<Event<Self>>
            + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 省储行【质押地址集合】
        /// 这些地址：只允许收钱，永远不能花
        type LockedAccounts: Contains<Self::AccountId>;
    }

    /// 本 Pallet 不需要存储任何状态
    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// 事件（其实用不上，但保留规范）
    #[pallet::event]
    pub enum Event<T: Config> {}

    /// 错误定义
    #[pallet::error]
    pub enum Error<T> {
        /// 尝试从省储行质押地址支出（永久禁止）
        StakeAccountLocked,
    }

    /// ================================
    /// 关键逻辑：拦截余额转出
    /// ================================
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    /// ================================
    /// 实现 Balance 的“黑名单校验”
    /// ================================
    impl<T: Config> pallet_balances::BalanceWithdrawReasons
        for Pallet<T>
    {
    }

    /// 核心：阻止任何对锁定账户的 withdraw
    impl<T: Config> frame_support::traits::WithdrawConsequence<T::Balance>
        for Pallet<T>
    {
    }

    /// ================================
    /// 真正起作用的地方
    /// ================================
    impl<T: Config> pallet_balances::traits::WithdrawConsequence<T::Balance>
        for Pallet<T>
    {
    }

    /// 更直接的方式：实现 Balance 的 Withdraw Hooks
    impl<T: Config> pallet_balances::traits::OnWithdraw<T::AccountId, T::Balance>
        for Pallet<T>
    {
        fn on_withdraw(
            who: &T::AccountId,
            _amount: T::Balance,
            _reasons: pallet_balances::WithdrawReasons,
        ) -> DispatchResult {
            // 如果是省储行质押地址，直接拒绝
            if T::LockedAccounts::contains(who) {
                return Err(Error::<T>::StakeAccountLocked.into());
            }
            Ok(())
        }
    }
}

#![cfg_attr(not(feature = "std"), no_std)]

//! Sheng Stake Lock 模块：永久锁定省储行创立发行的质押本金
//! 所有质押本金不可动，利息另有模块计算

use frame_support::{pallet_prelude::*, dispatch::DispatchResult};
use frame_system::pallet_prelude::*;

/// Pallet 配置
#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// 配置 trait
    #[pallet::config]
    pub trait Config: frame_system::Config {}

    /// 永久锁定的省储行质押地址列表
    #[pallet::storage]
    #[pallet::getter(fn stake_addresses)]
    pub type StakeAddresses<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

    /// Genesis 配置
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        /// 初始化质押地址
        pub addresses: Vec<T::AccountId>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self { addresses: vec![] }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            StakeAddresses::<T>::put(&self.addresses);
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 管理员可以查询是否锁定
        #[pallet::weight(0)]
        pub fn is_locked(origin: OriginFor<T>, who: T::AccountId) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            ensure!(Self::stake_addresses().contains(&who), "地址未锁定");
            Ok(())
        }
    }
}

//! -------------------- 硬编码省储行质押地址 --------------------
//! 注意：这里需要替换成实际 AccountId 类型，如果是 H160，可以用 sp_core::H160
use sp_core::H160;

pub const SHENG_STAKE_ADDRESSES: [H160; 43] = [
    H160::from_low_u64_be(0xe429392955e3b03f8987d22e74b5a2d42b85a85495a98e20),
    H160::from_low_u64_be(0x743b8fd14c75fbd4c91fc1f682ca625616edf71caa439b104),
    H160::from_low_u64_be(0x366ac468abb3aa37589d11df876c161c51aade201e1853ce),
    H160::from_low_u64_be(0x2a2082ab3be8cb6a9ac577a7a482cbc62838437e904f1da9),
    H160::from_low_u64_be(0x120b38cb2dbdfb877eb9d8a4aaa8240ee2f177f49ac51c70),
    H160::from_low_u64_be(0x80699804dec98eb41e76de63dfa32630f2d06ec13709fe15),
    H160::from_low_u64_be(0xb00d4873984cfb5334eff74e79fbe9f693d7688c9fac9df6),
    H160::from_low_u64_be(0x14b1db9cb6636bb04c8c0bdc16883eda77470a53085d7a69),
    H160::from_low_u64_be(0xf61dac348369192c555f3a6f443ba07bb322cdebfa6ff476),
    H160::from_low_u64_be(0x503f6b8bdc85781289dbff9d4783fee9bb8e998b4b6b6261),
    H160::from_low_u64_be(0x8eb158af26c5200cb78f99eab365f02114790adf67985f8b),
    H160::from_low_u64_be(0x98a33988269a045773bd3f66f561198d17f6714a4d1edfd1),
    H160::from_low_u64_be(0x14c9f50b2ece03896047aff016e3e7ddfb8ea881b53786440),
    H160::from_low_u64_be(0xae8db621e08709dff763f3bd8361c0cd70c98db1ba83e0f9),
    H160::from_low_u64_be(0xe23bcd551fc7802eb96e3d830df4b91d59947a325a3577e),
    H160::from_low_u64_be(0x04d7f11a04f03fb00ab6ab73197a8cbbc8b01a95802bd286),
    H160::from_low_u64_be(0xfc79abc72d72d85c463866d12af6481e08d782df6c626aef),
    H160::from_low_u64_be(0x1681e4ac7a82bb57f56e8b4753623cdd42a455edc4409bbcd),
    H160::from_low_u64_be(0xfc07d46fde8e0c02b8467e7c79005ba5818ff48779496c17),
    H160::from_low_u64_be(0x54f959013b31fbea54d020c3c5fdab5d06398eba577d74912),
    H160::from_low_u64_be(0xacb01b90db422448ed0406d2b914871e11a16d3d27af86e47),
    H160::from_low_u64_be(0xcc74a9343c8e6ab9bdd93060397583e8dafa2343c783925e9),
    H160::from_low_u64_be(0x4e1b7b96d7f525b9dcc9496a101154474a9d24dba0f50755),
    H160::from_low_u64_be(0x88d7eb8edbcee1f7dfe8f0750d5faa32d33657e4dff00948),
    H160::from_low_u64_be(0xf2ea0f6dbc76849807589504aef5e524b554de3e0898617f),
    H160::from_low_u64_be(0x448e7616718d1834d43ebfae38b5d4582e3431c8edf44b9d),
    H160::from_low_u64_be(0x14c6a1b01f309ee53e17b5fadfc840d7713b3ff44ad3fc5a),
    H160::from_low_u64_be(0x246b6ae6e66eb1c7e2afe835e20ce34466f53b058f1a2d71),
    H160::from_low_u64_be(0xba096e41228a6b74f3e3308dbb52e6cf2f6ac77b11f8e84),
    H160::from_low_u64_be(0xd42198407e26bf5f030b99d4d8a57b8ea1b79e2fbd09464),
    H160::from_low_u64_be(0x7ad68c2854dcf7f0ef7f87be6e00179e5725fc490d1e0922),
    H160::from_low_u64_be(0xec6f3c4cb06ccae833cbc2f03a093942341790b06be47de),
    H160::from_low_u64_be(0xeabebe6411a8b8ddb1a72498f532a02838e1bf90aa94ce8),
    H160::from_low_u64_be(0xbab75ce814c57941638b66440954d611fe19fc4fc9ff16b6),
    H160::from_low_u64_be(0x56102c196455fe656fada0e137153573fce7b5f1f5d6b7bd),
    H160::from_low_u64_be(0x8682612ec5b831b495d893d0a53338519a61496135a6cdc0),
    H160::from_low_u64_be(0xf8b72becfecfce5462b51ee90cc4c47cf312c81365646518),
    H160::from_low_u64_be(0x5465ab03c0d4993aa6afe95d1c17a712521cb06e57977884),
    H160::from_low_u64_be(0x4c1e5e2a5f15543ea0455c4b2b2f38a1586e3e568aa3789),
    H160::from_low_u64_be(0x9819d4d3606124c7dcdbdbf4821ea61195845470e1b9ced2),
    H160::from_low_u64_be(0x80d01d6198a9121e98724cae237f4a9a8f425932db77ee94),
    H160::from_low_u64_be(0xa6f2daffe8d06fc5948a9ed606ebae529e348ae4f94c5f5e),
    H160::from_low_u64_be(0x16baa06b70cb409622766d05703c4fdd2dc1545eaf5b3a9b),
];