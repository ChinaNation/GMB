#![cfg_attr(not(feature = "std"), no_std)]

/// Shengbank interest distribution pallet
/// 职责：
/// - 省储行创立发行质押利息支付模块，铸币每年支付43个省储行的质押利息
/// - 根据制度计算利息
/// - 将利息发放到多签账户
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {}

    #[pallet::storage]
    pub type DummyStorage<T> = StorageValue<_, (), OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {}

    #[pallet::error]
    pub enum Error<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {}
}