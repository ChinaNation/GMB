//! 创世机构与创世公职人员写入。
//!
//! 本文件只服务创世构建：把常量库内置机构写入 `public-manage`，把初始
//! 公职人员写入 `public-admins`。运行期的机构生命周期、管理员更换、
//! 法定代表人与内部投票回调均归对应业务 pallet，不在 genesis 模块承载。

extern crate alloc;

use alloc::vec::Vec;
use codec::Decode;
use frame_support::{pallet_prelude::BoundedVec, traits::Currency};
use frame_system::pallet_prelude::BlockNumberFor;
use primitives::{
    account_derive::{AccountKind, RESERVED_NAME_FEE, RESERVED_NAME_MAIN},
    cid::{
        china::{
            china_cb::CHINA_CB,
            china_ch::CHINA_CH,
            china_jc::CHINA_JC,
            china_jy::CHINA_JY,
            china_lf::CHINA_LF,
            china_sf::{CHINA_SF, NATIONAL_JUDICIAL_YUAN_ADMINS},
            china_zf::{CHINA_ZF, FEDERAL_REGISTRY_ADMINS},
        },
        code::{
            institution_code_from_cid_number, InstitutionCode, ProvinceCode, FRG, NJD,
            PROVINCE_CODE_INFOS,
        },
    },
};
use sp_runtime::traits::Zero;

use admin_primitives::{
    AdminAccount, AdminAccountKind, AdminAccountStatus, AdminProfile, AdminSource,
    ADMIN_ROLE_CHIEF_JUSTICE, ADMIN_ROLE_CONSTITUTION_GUARD, ADMIN_ROLE_DEPUTY_CHIEF_JUSTICE,
    ADMIN_ROLE_JUSTICE,
};
use public_manage::{
    InstitutionAccountInfo, InstitutionInfo, InstitutionLifecycleStatus, RegisteredInstitution,
    RESERVED_NAME_FEE as PUBLIC_RESERVED_NAME_FEE, RESERVED_NAME_MAIN as PUBLIC_RESERVED_NAME_MAIN,
};

const FEDERAL_REGISTRY_PROVINCE_GROUP_SIZE: usize =
    primitives::count_const::FRG_PROVINCE_GROUP_ADMIN_COUNT as usize;
const FEDERAL_REGISTRY_PROVINCE_ACCOUNT_PREFIX: &[u8] = b"GMB:FRG-PROVINCE:";

type PublicBalanceOf<T> = <<T as public_manage::Config>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::Balance;
type PublicCidNumberOf<T> = BoundedVec<u8, <T as public_manage::Config>::MaxCidNumberLength>;
type PublicAccountNameOf<T> = BoundedVec<u8, <T as public_manage::Config>::MaxAccountNameLength>;
type PublicInstitutionInfoOf<T> = InstitutionInfo<BlockNumberFor<T>, PublicAccountNameOf<T>>;
type PublicInstitutionAccountInfoOf<T> = InstitutionAccountInfo<
    <T as frame_system::Config>::AccountId,
    PublicBalanceOf<T>,
    BlockNumberFor<T>,
>;
type PublicRegisteredInstitutionOf<T> =
    RegisteredInstitution<PublicCidNumberOf<T>, PublicAccountNameOf<T>>;
type PublicAdminProfilesOf<T> = BoundedVec<
    AdminProfile<<T as frame_system::Config>::AccountId>,
    <T as public_admins::Config>::MaxAdminsPerInstitution,
>;
type PublicAdminAccountOf<T> = AdminAccount<
    PublicAdminProfilesOf<T>,
    <T as frame_system::Config>::AccountId,
    BlockNumberFor<T>,
>;

fn decode_account<T: frame_system::Config>(raw: &[u8; 32], label: &str) -> T::AccountId {
    T::AccountId::decode(&mut &raw[..])
        .unwrap_or_else(|_| panic!("genesis institution: {} 账户 decode 失败", label))
}

fn bounded_cid<T: public_manage::Config>(cid_number: &'static str) -> PublicCidNumberOf<T> {
    cid_number
        .as_bytes()
        .to_vec()
        .try_into()
        .unwrap_or_else(|_| {
            panic!(
                "genesis institution: cid_number {} 超过 MaxCidNumberLength",
                cid_number
            )
        })
}

fn bounded_account_name<T: public_manage::Config>(
    bytes: &[u8],
    label: &str,
    cid_number: &'static str,
) -> PublicAccountNameOf<T> {
    bytes.to_vec().try_into().unwrap_or_else(|_| {
        panic!(
            "genesis institution: cid_number {} {} 超过 MaxAccountNameLength",
            cid_number, label
        )
    })
}

fn bounded_static_name<T: public_manage::Config>(
    value: &'static str,
    label: &str,
    cid_number: &'static str,
) -> PublicAccountNameOf<T> {
    bounded_account_name::<T>(value.as_bytes(), label, cid_number)
}

fn insert_public_account<T: public_manage::Config>(
    cid_number: &'static str,
    cid: &PublicCidNumberOf<T>,
    account_name: PublicAccountNameOf<T>,
    address: T::AccountId,
    is_default: bool,
) {
    public_manage::InstitutionAccounts::<T>::insert(
        cid,
        &account_name,
        PublicInstitutionAccountInfoOf::<T> {
            address: address.clone(),
            initial_balance: PublicBalanceOf::<T>::zero(),
            status: InstitutionLifecycleStatus::Active,
            is_default,
            created_at: BlockNumberFor::<T>::default(),
        },
    );
    public_manage::CidRegisteredAccount::<T>::insert(cid, &account_name, address.clone());
    public_manage::AccountRegisteredCid::<T>::insert(
        address.clone(),
        PublicRegisteredInstitutionOf::<T> {
            cid_number: cid.clone(),
            account_name,
        },
    );
    public_manage::ProtectedGenesisAccounts::<T>::insert(address, ());
    let _ = cid_number;
}

/// 模板派生机构落地(ADR-031 卡3 全量创世直铸):账户由 CID 号确定性派生、
/// 名称为运行态 String;不进 ProtectedGenesisAccounts(市行政区/镇行政区机构后续可治理,
/// 且避免 59 万机构双倍保护条目)。构建期断言号格式合法 + 公权家族。
fn insert_derived_public_institution<T: public_manage::Config>(
    cid_number: &str,
    cid_full_name: &str,
    cid_short_name: &str,
) {
    let parts = primitives::cid::number::parse_cid_number_parts(cid_number)
        .unwrap_or_else(|e| panic!("genesis derived cid {cid_number} 非法: {e}"));
    assert!(
        primitives::cid::code::is_public_legal_code(&parts.institution),
        "genesis derived cid {cid_number} 非公权家族"
    );
    let cid: PublicCidNumberOf<T> = cid_number
        .as_bytes()
        .to_vec()
        .try_into()
        .unwrap_or_else(|_| panic!("genesis derived cid {cid_number} 超过 MaxCidNumberLength"));
    let bounded_name =
        |value: &str| -> PublicAccountNameOf<T> {
            value.as_bytes().to_vec().try_into().unwrap_or_else(|_| {
                panic!("genesis derived name {value} 超过 MaxAccountNameLength")
            })
        };
    public_manage::Institutions::<T>::insert(
        &cid,
        PublicInstitutionInfoOf::<T> {
            cid_full_name: bounded_name(cid_full_name),
            cid_short_name: bounded_name(cid_short_name),
            town_code: BoundedVec::new(),
            institution_code: parts.institution,
            created_at: BlockNumberFor::<T>::default(),
            status: InstitutionLifecycleStatus::Active,
        },
    );
    let bounded_reserved = |value: &[u8]| -> PublicAccountNameOf<T> {
        value
            .to_vec()
            .try_into()
            .expect("reserved account name fits")
    };
    let cid_bytes = cid_number.as_bytes();
    let main = AccountKind::InstitutionMain {
        cid_number: cid_bytes,
    }
    .derive(primitives::core_const::SS58_FORMAT);
    let fee = AccountKind::InstitutionFee {
        cid_number: cid_bytes,
    }
    .derive(primitives::core_const::SS58_FORMAT);
    insert_derived_account::<T>(
        &cid,
        bounded_reserved(RESERVED_NAME_MAIN),
        decode_account::<T>(&main, "派生主账户"),
        true,
    );
    insert_derived_account::<T>(
        &cid,
        bounded_reserved(RESERVED_NAME_FEE),
        decode_account::<T>(&fee, "派生费用账户"),
        false,
    );
}

/// 派生机构账户落地(不标记 ProtectedGenesisAccounts)。
fn insert_derived_account<T: public_manage::Config>(
    cid: &PublicCidNumberOf<T>,
    account_name: PublicAccountNameOf<T>,
    address: T::AccountId,
    is_default: bool,
) {
    public_manage::InstitutionAccounts::<T>::insert(
        cid,
        &account_name,
        PublicInstitutionAccountInfoOf::<T> {
            address: address.clone(),
            initial_balance: PublicBalanceOf::<T>::zero(),
            status: InstitutionLifecycleStatus::Active,
            is_default,
            created_at: BlockNumberFor::<T>::default(),
        },
    );
    public_manage::CidRegisteredAccount::<T>::insert(cid, &account_name, address.clone());
    public_manage::AccountRegisteredCid::<T>::insert(
        address,
        PublicRegisteredInstitutionOf::<T> {
            cid_number: cid.clone(),
            account_name,
        },
    );
}

fn insert_public_institution<T: public_manage::Config>(
    cid_number: &'static str,
    cid_full_name: &'static str,
    cid_short_name: &'static str,
    main_account: [u8; 32],
    fee_account: [u8; 32],
) {
    let cid = bounded_cid::<T>(cid_number);
    let institution_code = institution_code_from_cid_number(cid_number).unwrap_or_else(|| {
        panic!(
            "genesis institution: cid_number {} 机构码解析失败",
            cid_number
        )
    });
    public_manage::Institutions::<T>::insert(
        &cid,
        PublicInstitutionInfoOf::<T> {
            cid_full_name: bounded_static_name::<T>(cid_full_name, "cid_full_name", cid_number),
            cid_short_name: bounded_static_name::<T>(cid_short_name, "cid_short_name", cid_number),
            town_code: BoundedVec::new(),
            institution_code,
            created_at: BlockNumberFor::<T>::default(),
            status: InstitutionLifecycleStatus::Active,
        },
    );
    insert_public_account::<T>(
        cid_number,
        &cid,
        bounded_account_name::<T>(RESERVED_NAME_MAIN, "主账户名", cid_number),
        decode_account::<T>(&main_account, "主账户"),
        true,
    );
    insert_public_account::<T>(
        cid_number,
        &cid,
        bounded_account_name::<T>(RESERVED_NAME_FEE, "费用账户名", cid_number),
        decode_account::<T>(&fee_account, "费用账户"),
        false,
    );
    assert_eq!(RESERVED_NAME_MAIN, PUBLIC_RESERVED_NAME_MAIN);
    assert_eq!(RESERVED_NAME_FEE, PUBLIC_RESERVED_NAME_FEE);
}

fn bounded_role<T: public_admins::Config>(
    cid_number: &'static str,
    role: &'static [u8],
) -> BoundedVec<u8, frame_support::traits::ConstU32<{ admin_primitives::ADMIN_NAME_MAX_BYTES }>> {
    role.to_vec().try_into().unwrap_or_else(|_| {
        panic!(
            "genesis institution: cid_number {} 管理员职务过长",
            cid_number
        )
    })
}

fn national_judicial_yuan_admin_role(index: usize) -> Option<&'static [u8]> {
    // 护宪席位数单源 primitives::governance_skeleton(与节点骨架守卫 I6 同源;i<7 即原 0..=6)。
    let guard_seats = primitives::governance_skeleton::NJD_CONSTITUTION_GUARD_SEATS as usize;
    match index {
        i if i < guard_seats => Some(ADMIN_ROLE_CONSTITUTION_GUARD),
        7 => Some(ADMIN_ROLE_CHIEF_JUSTICE),
        8..=9 => Some(ADMIN_ROLE_DEPUTY_CHIEF_JUSTICE),
        10..=14 => Some(ADMIN_ROLE_JUSTICE),
        _ => None,
    }
}

fn build_admin_account<T, F>(
    cid_number: &'static str,
    institution_code: InstitutionCode,
    raw_admins: &[[u8; 32]],
    role_for_index: F,
) -> PublicAdminAccountOf<T>
where
    T: public_admins::Config,
    F: Fn(usize) -> Option<&'static [u8]>,
{
    let admins: Vec<AdminProfile<T::AccountId>> = raw_admins
        .iter()
        .enumerate()
        .map(|(index, raw)| {
            let role_name = role_for_index(index)
                .map(|role| bounded_role::<T>(cid_number, role))
                .unwrap_or_else(BoundedVec::new);
            AdminProfile {
                admin_account: decode_account::<T>(raw, "管理员"),
                admin_cid_number: BoundedVec::new(),
                admin_name: BoundedVec::new(),
                role_code: Default::default(),
                role_name,
                term_start: 0,
                term_end: 0,
                admin_source: AdminSource::Genesis,
                admin_source_ref: Default::default(),
            }
        })
        .collect();
    let bounded: PublicAdminProfilesOf<T> = admins.try_into().unwrap_or_else(|_| {
        panic!(
            "genesis institution: cid_number {} 管理员数量超过 MaxAdminsPerInstitution",
            cid_number
        )
    });
    let creator = bounded
        .first()
        .map(|p| p.admin_account.clone())
        .unwrap_or_else(|| {
            panic!(
                "genesis institution: cid_number {} 内置机构必须至少 1 个管理员",
                cid_number
            )
        });
    PublicAdminAccountOf::<T> {
        cid_number: cid_number
            .as_bytes()
            .to_vec()
            .try_into()
            .unwrap_or_else(|_| {
                panic!(
                    "genesis institution: cid_number {} 超过管理员集合 CID 长度上限",
                    cid_number
                )
            }),
        institution_code,
        kind: AdminAccountKind::PublicInstitution,
        admins: bounded,
        creator,
        created_at: BlockNumberFor::<T>::default(),
        updated_at: BlockNumberFor::<T>::default(),
        status: AdminAccountStatus::Active,
    }
}

fn federal_registry_province_group_account<T: frame_system::Config>(
    province_code: ProvinceCode,
) -> T::AccountId {
    let mut payload = Vec::with_capacity(
        FEDERAL_REGISTRY_PROVINCE_ACCOUNT_PREFIX
            .len()
            .saturating_add(province_code.len()),
    );
    payload.extend_from_slice(FEDERAL_REGISTRY_PROVINCE_ACCOUNT_PREFIX);
    payload.extend_from_slice(&province_code);
    let raw = sp_io::hashing::blake2_256(&payload);
    T::AccountId::decode(&mut &raw[..])
        .unwrap_or_else(|_| panic!("genesis institution: FRG 省行政区组账户 decode 失败"))
}

fn insert_fixed_admins<T, F>(
    main_account: [u8; 32],
    cid_number: &'static str,
    institution_code: InstitutionCode,
    raw_admins: &[[u8; 32]],
    role_for_index: F,
) where
    T: public_admins::Config,
    F: Fn(usize) -> Option<&'static [u8]>,
{
    let account = decode_account::<T>(&main_account, "固定治理机构主账户");
    public_admins::AdminAccounts::<T>::insert(
        account,
        build_admin_account::<T, _>(cid_number, institution_code, raw_admins, role_for_index),
    );
}

/// 创世写入内置公权机构和创世公职人员。
/// 创世直铸国家/省/市公权机构(ADR-031 v3):纯枚举(primitives 单源)
/// → 落地存储;账户由 CID 号确定性派生,与 296 常量互不重号。
fn build_template_institutions<T: public_manage::Config>() {
    primitives::cid::official_derive::for_each_public_institution(|cid, full, short| {
        insert_derived_public_institution::<T>(cid, full, short);
    });
}

pub fn build<T>()
where
    T: public_manage::Config + public_admins::Config,
{
    for node in CHINA_CB.iter() {
        insert_public_institution::<T>(
            node.cid_number,
            node.cid_full_name,
            node.cid_short_name,
            node.main_account,
            node.fee_account,
        );
        let institution_code = institution_code_from_cid_number(node.cid_number)
            .expect("china_cb cid_number must encode institution code");
        insert_fixed_admins::<T, _>(
            node.main_account,
            node.cid_number,
            institution_code,
            node.admins,
            |_| None,
        );
    }

    for node in CHINA_CH.iter() {
        insert_public_institution::<T>(
            node.cid_number,
            node.cid_full_name,
            node.cid_short_name,
            node.main_account,
            node.fee_account,
        );
        let institution_code = institution_code_from_cid_number(node.cid_number)
            .expect("china_ch cid_number must encode institution code");
        insert_fixed_admins::<T, _>(
            node.main_account,
            node.cid_number,
            institution_code,
            node.admins,
            |_| None,
        );
    }

    // 常量数组全量直铸(ADR-031 卡3):ZF/JC/SF/LF/JY 逐节点写入机构+双账户,
    // 与上方 CB/CH 合计 296;创世不带管理员(NJD/FRG 特例在下方单独写入)。
    for node in CHINA_ZF.iter() {
        insert_public_institution::<T>(
            node.cid_number,
            node.cid_full_name,
            node.cid_short_name,
            node.main_account,
            node.fee_account,
        );
    }
    for node in CHINA_JC.iter() {
        insert_public_institution::<T>(
            node.cid_number,
            node.cid_full_name,
            node.cid_short_name,
            node.main_account,
            node.fee_account,
        );
    }
    for node in CHINA_SF.iter() {
        insert_public_institution::<T>(
            node.cid_number,
            node.cid_full_name,
            node.cid_short_name,
            node.main_account,
            node.fee_account,
        );
    }
    for node in CHINA_LF.iter() {
        insert_public_institution::<T>(
            node.cid_number,
            node.cid_full_name,
            node.cid_short_name,
            node.main_account,
            node.fee_account,
        );
    }
    for node in CHINA_JY.iter() {
        insert_public_institution::<T>(
            node.cid_number,
            node.cid_full_name,
            node.cid_short_name,
            node.main_account,
            node.fee_account,
        );
    }

    // NJD 创世大法官/宪法守护管理员特例。
    let njd_node = CHINA_SF
        .iter()
        .find(|node| institution_code_from_cid_number(node.cid_number) == Some(NJD))
        .expect("china_sf must include NJD");
    insert_fixed_admins::<T, _>(
        njd_node.main_account,
        njd_node.cid_number,
        NJD,
        NATIONAL_JUDICIAL_YUAN_ADMINS,
        national_judicial_yuan_admin_role,
    );

    // FRG 省行政区 5 人组管理员特例。
    let frg_node = CHINA_ZF
        .iter()
        .find(|node| institution_code_from_cid_number(node.cid_number) == Some(FRG))
        .expect("china_zf must include FRG");
    assert!(
        FEDERAL_REGISTRY_ADMINS.len()
            == PROVINCE_CODE_INFOS.len() * FEDERAL_REGISTRY_PROVINCE_GROUP_SIZE,
        "genesis institution: FRG 管理员数量必须等于省数 * 5 人"
    );
    for (index, province) in PROVINCE_CODE_INFOS.iter().enumerate() {
        let group_account = federal_registry_province_group_account::<T>(province.province_code);
        let start = index * FEDERAL_REGISTRY_PROVINCE_GROUP_SIZE;
        let end = start + FEDERAL_REGISTRY_PROVINCE_GROUP_SIZE;
        let account = build_admin_account::<T, _>(
            frg_node.cid_number,
            FRG,
            &FEDERAL_REGISTRY_ADMINS[start..end],
            |_| None,
        );
        public_admins::FederalRegistryProvinceGroups::<T>::insert(province.province_code, account);
        public_admins::FederalRegistryProvinceGroupAccounts::<T>::insert(
            group_account,
            province.province_code,
        );
    }

    // 创世直铸当前国家/省/市公权机构(ADR-031 v3):常量 296 + 派生 49,297。
    build_template_institutions::<T>();
}
