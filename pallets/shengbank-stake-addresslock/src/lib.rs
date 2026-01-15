#![cfg_attr(not(feature = "std"), no_std)]

/// Shengbank account control pallet
/// 职责：
/// - 省储行创立发行永久质押锁定模块，锁定43个省储行单签名账户地址
/// - 提供权限判断接口给其他 Pallet
pub use pallet::*;

#![cfg_attr(not(feature = "std"), no_std)] 
// 如果不是在 std 环境下（链上运行时），使用 no_std
// Substrate runtime 是在 WebAssembly 中运行的，所以必须 no_std

use frame_support::{pallet_prelude::*}; 
use frame_system::pallet_prelude::*; 
use sp_std::vec::Vec; // 使用 Substrate 的标准 Vec 类型

// 这里定义一个 pallet 模块
#[frame_support::pallet]
pub mod pallet {
    use super::*;

    // Pallet 结构体，Substrate 所有 pallet 都必须有这个结构体
    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // 配置 trait，Runtime 会实现它
    #[pallet::config]
    pub trait Config: frame_system::Config {}

    // -----------------------
    // 存储项 Storage
    // -----------------------
    #[pallet::storage]
    #[pallet::getter(fn stake_addresses)]
    /// 存储 43 个省储行质押地址
    /// 这些地址的公民币**永久锁定**，不能转出
    pub type StakeAddresses<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

    // -----------------------
    // 创世配置 Genesis
    // -----------------------
    #[pallet::genesis_config]
    /// 用来在链初始化时写入质押地址
    pub struct GenesisConfig<T: Config> {
        pub stake_addresses: Vec<T::AccountId>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                stake_addresses: vec![], // 默认空
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            // 把创世配置写入 Storage
            <StakeAddresses<T>>::put(&self.stake_addresses);
        }
    }

    // -----------------------
    // Call 接口（可选）
    // -----------------------
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 检查某个账户是否在锁定名单中
        #[pallet::weight(0)]
        pub fn is_locked(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
            let _who = ensure_signed(origin)?; // 确保是外部用户调用
            ensure!(
                <StakeAddresses<T>>::get().contains(&account),
                "该账户资金已永久质押，不能转出"
            );
            Ok(())
        }
    }
}

// -----------------------
// 43 个省储行质押地址硬编码（在 Runtime 创世时初始化）
pub fn get_default_stake_addresses() -> Vec<[u8; 32]> {
    vec![
        hex_literal::hex!("63e2690b28414c681977c19d0d612625c7901e6b04458da4176ba4874fd2113f"),
        hex_literal::hex!("e429392955e3b03f8987d22e74b5a2d42b85a85495a98e2091060f46cc302136"),
        hex_literal::hex!("743b8fd14c75fbd4c91fc1f682ca625616edf71caa439b1044a81c17d9772839"),
        hex_literal::hex!("366ac468abb3aa37589d11df876c161c51aade201e1853ceeb57488a755a110d"),
        hex_literal::hex!("2a2082ab3be8cb6a9ac577a7a482cbc62838437e904f1da940041e66d00d443c"),
        hex_literal::hex!("120b38cb2dbdfb877eb9d8a4aaa8240ee2f177f49ac51c702680ac441fbbf63b"),
        hex_literal::hex!("80699804dec98eb41e76de63dfa32630f2d06ec13709fe151efc0e5652b2b22c"),
        hex_literal::hex!("b00d4873984cfb5334eff74e79fbe9f693d7688c9fac9df6892b48a1db3e4664"),
        hex_literal::hex!("14b1db9cb6636bb04c8c0bdc16883eda77470a53085d7a69edab49bbdc44850c"),
        hex_literal::hex!("f61dac348369192c555f3a6f443ba07bb322cdebfa6ff47696ad6cb8bbd52a1f"),
        hex_literal::hex!("503f6b8bdc85781289dbff9d4783fee9bb8e998b4b6b6261c426d7109d7bda49"),
        hex_literal::hex!("8eb158af26c5200cb78f99eab365f02114790adf67985f8b93c77158af869914"),
        hex_literal::hex!("98a33988269a045773bd3f66f561198d17f6714a4d1edfd14ee5566f090a750d"),
        hex_literal::hex!("14c9f50b2ece03896047aff016e3e7ddfb8ea881b537864408283f819893df70"),
        hex_literal::hex!("ae8db621e08709dff763f3bd8361c0cd70c98db1ba83e0f9034b23bc6f5dc06c"),
        hex_literal::hex!("e23bcd551fc7802eb96e3d830df4b91d59947a325a3577e934fbbcb428f0c93f"),
        hex_literal::hex!("04d7f11a04f03fb00ab6ab73197a8cbbc8b01a95802bd2865b5a08f0cb6ee247"),
        hex_literal::hex!("fc79abc72d72d85c463866d12af6481e08d782df6c626aef5f69f28a4a39c631"),
        hex_literal::hex!("1681e4ac7a82bb57f56e8b4753623cdd42a455edc4409bbcd9258d7884cc043b"),
        hex_literal::hex!("fc07d46fde8e0c02b8467e7c79005ba5818ff48779496c17ccdc30f8c0f2ae56"),
        hex_literal::hex!("54f959013b31fbea54d020c3c5fdab5d06398eba577d749128c4527f02908055"),
        hex_literal::hex!("acb01b90db422448ed0406d2b914871e11a16d3d27af86e4706c31a70389d725"),
        hex_literal::hex!("cc74a9343c8e6ab9bdd93060397583e8dafa2343c783925e9d8f2f507494dc70"),
        hex_literal::hex!("4e1b7b96d7f525b9dcc9496a101154474a9d24dba0f50755c9bc066f00300368"),
        hex_literal::hex!("88d7eb8edbcee1f7dfe8f0750d5faa32d33657e4dff00948fa7b2afc63a7d105"),
        hex_literal::hex!("f2ea0f6dbc76849807589504aef5e524b554de3e0898617f38cbc29d2d08d050"),
        hex_literal::hex!("448e7616718d1834d43ebfae38b5d4582e3431c8edf44b9ddd227df15a14bd00"),
        hex_literal::hex!("14c6a1b01f309ee53e17b5fadfc840d7713b3ff44ad3fc5ad5656e8b071f092f"),
        hex_literal::hex!("246b6ae6e66eb1c7e2afe835e20ce34466f53b058f1a2d714ca4505bf04f3032"),
        hex_literal::hex!("ba096e41228a6b74f3e3308dbb52e6cf2f6ac77b11f8e844c8d82f2a90947919"),
        hex_literal::hex!("d42198407e26bf5f030b99d4d8a57b8ea1b79e2fbd0946495b082d241bd4b719"),
        hex_literal::hex!("7ad68c2854dcf7f0ef7f87be6e00179e5725fc490d1e0922c1bab431d9580175"),
        hex_literal::hex!("ec6f3c4cb06ccae833cbc2f03a093942341790b06be47dee4d3ef8d243e77118"),
        hex_literal::hex!("eabebe6411a8b8ddb1a72498f532a02838e1bf90aa94ce8c798aaa7e197dd901"),
        hex_literal::hex!("bab75ce814c57941638b66440954d611fe19fc4fc9ff16b698566faece719453"),
        hex_literal::hex!("56102c196455fe656fada0e137153573fce7b5f1f5d6b7bd856135c9fa7d601f"),
        hex_literal::hex!("8682612ec5b831b495d893d0a53338519a61496135a6cdc07471444fce4d4f7c"),
        hex_literal::hex!("f8b72becfecfce5462b51ee90cc4c47cf312c81365646518a8a7ab1f96089a19"),
        hex_literal::hex!("5465ab03c0d4993aa6afe95d1c17a712521cb06e57977884517c6b8b70f20e5f"),
        hex_literal::hex!("4c1e5e2a5f15543ea0455c4b2b2f38a1586e3e568aa37894a4ffa76032c33632"),
        hex_literal::hex!("9819d4d3606124c7dcdbdbf4821ea61195845470e1b9ced2d11e06bd60bdcd08"),
        hex_literal::hex!("80d01d6198a9121e98724cae237f4a9a8f425932db77ee94026408eab50a2548"),
        hex_literal::hex!("a6f2daffe8d06fc5948a9ed606ebae529e348ae4f94c5f5e69867caffcee084a"),
        hex_literal::hex!("16baa06b70cb409622766d05703c4fdd2dc1545eaf5b3a9bcf65c3fbe8413077"),
    ]
}