#![cfg_attr(not(feature = "std"), no_std)]
//! 创世机构生命周期模块（genesis-manage）。
//!
//! 中文注释：本模块只管理创世机构本体信息和创世账户封存索引。
//! 创世机构管理员集合仍由 `genesis-admins` 管理；投票、换届与阈值快照仍归投票引擎。

extern crate alloc;

use admin_primitives::AdminAccountQuery;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{pallet_prelude::*, BoundedVec};
use frame_system::pallet_prelude::*;
use primitives::{
    account_derive::{RESERVED_NAME_FEE, RESERVED_NAME_MAIN},
    cid::{
        china::{china_cb::CHINA_CB, china_ch::CHINA_CH, china_zf::CHINA_ZF},
        code::{
            fixed_governance_pass_threshold, institution_code_from_cid_number, InstitutionCode,
        },
    },
};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

pub use pallet::*;
pub mod weights;

/// genesis-manage pallet on-chain storage 版本。
const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

fn decode_account<T: frame_system::Config>(raw: &[u8; 32]) -> Option<T::AccountId> {
    T::AccountId::decode(&mut &raw[..]).ok()
}

/// 创世机构 CID 登记反向索引项：account -> (cid_number, account_name)。
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
pub struct RegisteredGenesisInstitution<CidNumber, AccountName> {
    pub cid_number: CidNumber,
    pub account_name: AccountName,
}

/// 创世机构生命周期状态。
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    Copy,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
)]
pub enum GenesisInstitutionStatus {
    /// 创世即激活；创世机构不可关闭、不可注销。
    Active,
}

/// 创世机构链上最小身份档案。
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
pub struct GenesisInstitutionInfo<BlockNumber, AccountName> {
    pub cid_full_name: AccountName,
    pub cid_short_name: AccountName,
    pub institution_code: InstitutionCode,
    pub created_at: BlockNumber,
    pub status: GenesisInstitutionStatus,
}

/// 创世机构账户信息。
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
pub struct GenesisInstitutionAccountInfo<AccountId, BlockNumber> {
    pub address: AccountId,
    pub status: GenesisInstitutionStatus,
    pub is_default: bool,
    pub created_at: BlockNumber,
}

#[frame_support::pallet]
#[allow(dead_code)] // 中文注释：当前无 extrinsic，事件仅预留给未来创世机构补档迁移审计。
pub mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + votingengine::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 管理员统一查询入口；本模块只读，不保存管理员集合。
        type AdminAccountQuery: admin_primitives::AdminAccountQuery<Self::AccountId>;

        #[pallet::constant]
        type MaxCidNumberLength: Get<u32>;

        #[pallet::constant]
        type MaxAccountNameLength: Get<u32>;

        type WeightInfo: crate::weights::WeightInfo;
    }

    pub type CidNumberOf<T> = BoundedVec<u8, <T as Config>::MaxCidNumberLength>;
    pub type AccountNameOf<T> = BoundedVec<u8, <T as Config>::MaxAccountNameLength>;
    pub type GenesisInstitutionInfoOf<T> =
        GenesisInstitutionInfo<BlockNumberFor<T>, AccountNameOf<T>>;
    pub type GenesisInstitutionAccountInfoOf<T> =
        GenesisInstitutionAccountInfo<<T as frame_system::Config>::AccountId, BlockNumberFor<T>>;
    pub type RegisteredGenesisInstitutionOf<T> =
        RegisteredGenesisInstitution<CidNumberOf<T>, AccountNameOf<T>>;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// 创世机构档案：cid_number -> 机构信息。
    #[pallet::storage]
    pub type GenesisInstitutions<T: Config> =
        StorageMap<_, Blake2_128Concat, CidNumberOf<T>, GenesisInstitutionInfoOf<T>, OptionQuery>;

    /// 创世机构账户索引：(cid_number, account_name) -> 账户信息。
    #[pallet::storage]
    pub type GenesisInstitutionAccounts<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        CidNumberOf<T>,
        Blake2_128Concat,
        AccountNameOf<T>,
        GenesisInstitutionAccountInfoOf<T>,
        OptionQuery,
    >;

    /// 创世机构 CID 账户正向索引：(cid_number, account_name) -> account。
    #[pallet::storage]
    pub type GenesisCidRegisteredAccount<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        CidNumberOf<T>,
        Blake2_128Concat,
        AccountNameOf<T>,
        T::AccountId,
        OptionQuery,
    >;

    /// 创世机构 CID 账户反向索引：account -> (cid_number, account_name)。
    #[pallet::storage]
    pub type GenesisAccountRegisteredCid<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        RegisteredGenesisInstitutionOf<T>,
        OptionQuery,
    >;

    /// 创世初始机构封存表：CID 系统根基，永不可注销关闭。
    #[pallet::storage]
    pub type ProtectedGenesisAccounts<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, (), OptionQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub _phantom: core::marker::PhantomData<T>,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                _phantom: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            for node in CHINA_CB.iter() {
                Pallet::<T>::insert_builtin_institution(
                    node.cid_number,
                    node.cid_full_name,
                    node.cid_short_name,
                    node.main_account,
                    node.fee_account,
                );
            }
            for node in CHINA_CH.iter() {
                Pallet::<T>::insert_builtin_institution(
                    node.cid_number,
                    node.cid_full_name,
                    node.cid_short_name,
                    node.main_account,
                    node.fee_account,
                );
            }
            for node in CHINA_ZF.iter() {
                let Some(institution_code) = institution_code_from_cid_number(node.cid_number)
                else {
                    panic!(
                        "genesis-manage: cid_number {} 机构码解析失败",
                        node.cid_number
                    );
                };
                if institution_code != admin_primitives::FRG {
                    continue;
                }
                Pallet::<T>::insert_builtin_institution(
                    node.cid_number,
                    node.cid_full_name,
                    node.cid_short_name,
                    node.main_account,
                    node.fee_account,
                );
            }
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 创世机构已在创世构建中写入。
        ///
        /// 中文注释：当前创世构建不发事件；该事件预留给未来补档迁移审计。
        GenesisInstitutionLoaded { account: T::AccountId },
    }

    #[pallet::error]
    pub enum Error<T> {}

    impl<T: Config> Pallet<T> {
        /// 账户是否属于创世封存机构。
        pub fn is_genesis_protected(account: &T::AccountId) -> bool {
            ProtectedGenesisAccounts::<T>::contains_key(account)
        }

        /// 账户对应的创世机构码。
        pub fn resolve_institution_code_for_account(
            account: &T::AccountId,
        ) -> Option<InstitutionCode> {
            let registered = GenesisAccountRegisteredCid::<T>::get(account)?;
            GenesisInstitutions::<T>::get(registered.cid_number).map(|info| info.institution_code)
        }

        /// 任意创世机构账户映射到该机构主账户，管理员治理统一绑定主账户。
        pub fn resolve_admin_account_for_account(account: &T::AccountId) -> Option<T::AccountId> {
            let registered = GenesisAccountRegisteredCid::<T>::get(account)?;
            let main_name = Self::bounded_bytes(
                RESERVED_NAME_MAIN,
                "主账户",
                registered.cid_number.as_slice(),
            );
            GenesisCidRegisteredAccount::<T>::get(registered.cid_number, main_name)
        }

        fn bounded_bytes(
            bytes: &[u8],
            label: &str,
            cid_number: &[u8],
        ) -> BoundedVec<u8, <T as Config>::MaxAccountNameLength> {
            bytes.to_vec().try_into().unwrap_or_else(|_| {
                panic!(
                    "genesis-manage: cid_number {:?} {} 超过 MaxAccountNameLength",
                    cid_number, label
                )
            })
        }

        fn bounded_cid(cid_number: &'static str) -> CidNumberOf<T> {
            cid_number
                .as_bytes()
                .to_vec()
                .try_into()
                .unwrap_or_else(|_| {
                    panic!(
                        "genesis-manage: cid_number {} 超过 MaxCidNumberLength",
                        cid_number
                    )
                })
        }

        fn insert_account(
            cid_number: &CidNumberOf<T>,
            account_name: AccountNameOf<T>,
            address: T::AccountId,
            is_default: bool,
        ) {
            let info = GenesisInstitutionAccountInfo {
                address: address.clone(),
                status: GenesisInstitutionStatus::Active,
                is_default,
                created_at: BlockNumberFor::<T>::default(),
            };
            GenesisInstitutionAccounts::<T>::insert(cid_number, &account_name, info);
            GenesisCidRegisteredAccount::<T>::insert(cid_number, &account_name, address.clone());
            GenesisAccountRegisteredCid::<T>::insert(
                address.clone(),
                RegisteredGenesisInstitution {
                    cid_number: cid_number.clone(),
                    account_name,
                },
            );
            ProtectedGenesisAccounts::<T>::insert(address, ());
        }

        fn insert_builtin_institution(
            cid_number: &'static str,
            cid_full_name: &'static str,
            cid_short_name: &'static str,
            main_account: [u8; 32],
            fee_account: [u8; 32],
        ) {
            let cid = Self::bounded_cid(cid_number);
            let Some(institution_code) = institution_code_from_cid_number(cid_number) else {
                panic!("genesis-manage: cid_number {} 机构码解析失败", cid_number);
            };
            let cid_full_name = cid_full_name
                .as_bytes()
                .to_vec()
                .try_into()
                .unwrap_or_else(|_| {
                    panic!(
                        "genesis-manage: cid_number {} cid_full_name 超过 MaxAccountNameLength",
                        cid_number
                    )
                });
            let cid_short_name = cid_short_name
                .as_bytes()
                .to_vec()
                .try_into()
                .unwrap_or_else(|_| {
                    panic!(
                        "genesis-manage: cid_number {} cid_short_name 超过 MaxAccountNameLength",
                        cid_number
                    )
                });
            GenesisInstitutions::<T>::insert(
                &cid,
                GenesisInstitutionInfo {
                    cid_full_name,
                    cid_short_name,
                    institution_code,
                    created_at: BlockNumberFor::<T>::default(),
                    status: GenesisInstitutionStatus::Active,
                },
            );
            let main = decode_account::<T>(&main_account).unwrap_or_else(|| {
                panic!(
                    "genesis-manage: cid_number {} 主账户 decode 失败",
                    cid_number
                )
            });
            let fee = decode_account::<T>(&fee_account).unwrap_or_else(|| {
                panic!(
                    "genesis-manage: cid_number {} 费用账户 decode 失败",
                    cid_number
                )
            });
            Self::insert_account(
                &cid,
                Self::bounded_bytes(RESERVED_NAME_MAIN, "主账户名", cid_number.as_bytes()),
                main,
                true,
            );
            Self::insert_account(
                &cid,
                Self::bounded_bytes(RESERVED_NAME_FEE, "费用账户名", cid_number.as_bytes()),
                fee,
                false,
            );
        }
    }
}

impl<T: pallet::Config> entity_primitives::InstitutionCidQuery<pallet::CidNumberOf<T>>
    for pallet::Pallet<T>
{
    fn cid_exists(cid_number: &pallet::CidNumberOf<T>) -> bool {
        pallet::GenesisInstitutions::<T>::contains_key(cid_number)
            || pallet::GenesisCidRegisteredAccount::<T>::iter_prefix(cid_number)
                .next()
                .is_some()
    }
}

impl<T: pallet::Config> entity_primitives::InstitutionMultisigQuery<T::AccountId>
    for pallet::Pallet<T>
{
    fn lookup_org(addr: &T::AccountId) -> Option<InstitutionCode> {
        pallet::Pallet::<T>::resolve_institution_code_for_account(addr)
    }

    fn lookup_admin_config(
        addr: &T::AccountId,
    ) -> Option<primitives::multisig::MultisigConfigSnapshot<T::AccountId>> {
        let institution_code = Self::lookup_org(addr)?;
        if institution_code == admin_primitives::FRG {
            // 中文注释：FRG 的代码级固定阈值 3 只用于省级 5 人组内部投票；
            // 创世机构主账户聚合 215 名管理员仅用于身份/特权校验,不得暴露成 215/3 多签配置。
            return None;
        }
        let account = pallet::Pallet::<T>::resolve_admin_account_for_account(addr)?;
        let admins =
            T::AdminAccountQuery::active_account_admins(institution_code, account.clone())?;
        let threshold = fixed_governance_pass_threshold(&institution_code)?;
        let admins_len = admins.len() as u32;
        Some(primitives::multisig::MultisigConfigSnapshot {
            admins,
            admins_len,
            threshold,
        })
    }

    fn is_active(addr: &T::AccountId) -> bool {
        let Some(registered) = pallet::GenesisAccountRegisteredCid::<T>::get(addr) else {
            return false;
        };
        matches!(
            pallet::GenesisInstitutionAccounts::<T>::get(
                &registered.cid_number,
                &registered.account_name,
            )
            .map(|a| a.status),
            Some(GenesisInstitutionStatus::Active)
        )
    }
}
