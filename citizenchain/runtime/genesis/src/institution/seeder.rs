//! 创世机构、固定岗位任职与管理员钱包集合写入。
//!
//! 本文件只服务创世构建：机构、岗位和任职写入 `public-manage`，由任职钱包
//! 去重得到的管理员集合写入 `public-admins`。运行期机构生命周期、管理员更换、
//! 法定代表人任命与内部投票回调均归对应业务 pallet，不在 genesis 模块承载。

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
        code::{institution_code_from_cid_number, InstitutionCode, FRG, NJD},
    },
};
use sp_runtime::traits::Zero;

use admin_primitives::{AdminAccountStatus, InstitutionAdminAccount};
use public_manage::{
    InstitutionAccountInfo, InstitutionAdminAssignment, InstitutionAssignmentSource,
    InstitutionAssignmentStatus, InstitutionInfo, InstitutionLifecycleStatus, InstitutionRole,
    InstitutionRoleStatus, RegisteredInstitution, RESERVED_NAME_FEE as PUBLIC_RESERVED_NAME_FEE,
    RESERVED_NAME_MAIN as PUBLIC_RESERVED_NAME_MAIN,
};

use super::fixed_roles;

type PublicBalanceOf<T> = <<T as public_manage::Config>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::Balance;
type PublicCidNumberOf<T> = BoundedVec<u8, <T as public_manage::Config>::MaxCidNumberLength>;
type PublicAccountNameOf<T> = BoundedVec<u8, <T as public_manage::Config>::MaxAccountNameLength>;
type PublicInstitutionInfoOf<T> = InstitutionInfo<
    BlockNumberFor<T>,
    PublicAccountNameOf<T>,
    PublicCidNumberOf<T>,
    <T as frame_system::Config>::AccountId,
>;
type PublicInstitutionAccountInfoOf<T> = InstitutionAccountInfo<
    <T as frame_system::Config>::AccountId,
    PublicBalanceOf<T>,
    BlockNumberFor<T>,
>;
type PublicRegisteredInstitutionOf<T> =
    RegisteredInstitution<PublicCidNumberOf<T>, PublicAccountNameOf<T>>;
type PublicAdminsOf<T> = BoundedVec<
    <T as frame_system::Config>::AccountId,
    <T as public_admins::Config>::MaxAdminsPerInstitution,
>;
type PublicAdminAccountOf<T> = InstitutionAdminAccount<PublicAdminsOf<T>>;

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
            // 法定代表人不是创世必填项：创世三字段全空，后续依法任命时原子写入。
            // 禁止用首位管理员、机构主账户或其它钱包占位。
            legal_representative_name: None,
            legal_representative_cid_number: None,
            legal_representative_account: None,
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
            // 固定创世机构同样允许尚未任命法定代表人；三字段必须保持全空。
            legal_representative_name: None,
            legal_representative_cid_number: None,
            legal_representative_account: None,
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

fn insert_fixed_admins<T>(
    main_account: [u8; 32],
    cid_number: &'static str,
    institution_code: InstitutionCode,
    raw_admins: &[[u8; 32]],
) where
    T: public_manage::Config + public_admins::Config,
{
    fixed_roles::assert_fixed_admin_count(institution_code, raw_admins.len());
    let cid = bounded_cid::<T>(cid_number);
    let mut roles: Vec<public_manage::institution::role::InstitutionRoleOf<T>> = Vec::new();
    let mut assignments: Vec<public_manage::institution::role::InstitutionAdminAssignmentOf<T>> =
        Vec::new();
    let mut admin_accounts: Vec<T::AccountId> = Vec::new();

    for (index, raw) in raw_admins.iter().enumerate() {
        let (role_code_raw, role_name_raw) =
            fixed_roles::role_for_fixed_admin(institution_code, index);
        let role_code: public_manage::institution::role::RoleCodeOf = role_code_raw
            .try_into()
            .unwrap_or_else(|_| panic!("genesis institution: {} 岗位代码过长", cid_number));
        let role_name = bounded_account_name::<T>(&role_name_raw, "岗位名称", cid_number);
        if !roles.iter().any(|role| role.role_code == role_code) {
            roles.push(InstitutionRole {
                cid_number: cid.clone(),
                role_code: role_code.clone(),
                role_name,
                term_required: false,
                role_status: InstitutionRoleStatus::Active,
            });
        }
        let admin_account = decode_account::<T>(raw, "管理员");
        assignments.push(InstitutionAdminAssignment {
            cid_number: cid.clone(),
            admin_account: admin_account.clone(),
            role_code,
            term_start: 0,
            term_end: 0,
            assignment_source: InstitutionAssignmentSource::Genesis,
            assignment_source_ref: Default::default(),
            assignment_status: InstitutionAssignmentStatus::Active,
        });
        if !admin_accounts.contains(&admin_account) {
            admin_accounts.push(admin_account);
        }
    }

    let roles: public_manage::institution::role::InstitutionRolesOf<T> = roles
        .try_into()
        .unwrap_or_else(|_| panic!("genesis institution: {} 岗位数量超限", cid_number));
    let assignments: public_manage::institution::role::InstitutionAdminAssignmentsOf<T> =
        assignments
            .try_into()
            .unwrap_or_else(|_| panic!("genesis institution: {} 任职数量超限", cid_number));
    public_manage::Pallet::<T>::store_genesis_roles_and_assignments(&cid, &roles, &assignments)
        .unwrap_or_else(|_| panic!("genesis institution: {} 岗位任职写入失败", cid_number));

    assert_eq!(
        admin_accounts.len(),
        raw_admins.len(),
        "genesis institution: 固定岗位钱包常量不得重复"
    );

    let admins: PublicAdminsOf<T> = admin_accounts.try_into().unwrap_or_else(|_| {
        panic!(
            "genesis institution: cid_number {} 管理员数量超过 MaxAdminsPerInstitution",
            cid_number
        )
    });
    let account = decode_account::<T>(&main_account, "固定治理机构主账户");
    let admin_account = PublicAdminAccountOf::<T> {
        cid_number: cid_number
            .as_bytes()
            .to_vec()
            .try_into()
            .unwrap_or_else(|_| panic!("genesis institution: {} 管理员CID过长", cid_number)),
        institution_code,
        admins,
        status: AdminAccountStatus::Active,
    };
    public_admins::AdminAccounts::<T>::insert(account, admin_account);
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
        insert_fixed_admins::<T>(
            node.main_account,
            node.cid_number,
            institution_code,
            node.admins,
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
        insert_fixed_admins::<T>(
            node.main_account,
            node.cid_number,
            institution_code,
            node.admins,
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
    insert_fixed_admins::<T>(
        njd_node.main_account,
        njd_node.cid_number,
        NJD,
        NATIONAL_JUDICIAL_YUAN_ADMINS,
    );

    // FRG 是一个机构、215 名管理员、43 个省专员岗位；省级分组由 entity 任职表达。
    let frg_node = CHINA_ZF
        .iter()
        .find(|node| institution_code_from_cid_number(node.cid_number) == Some(FRG))
        .expect("china_zf must include FRG");
    insert_fixed_admins::<T>(
        frg_node.main_account,
        frg_node.cid_number,
        FRG,
        FEDERAL_REGISTRY_ADMINS,
    );

    // 创世直铸当前国家/省/市公权机构(ADR-031 v3):常量 296 + 派生 49,297。
    build_template_institutions::<T>();
}
