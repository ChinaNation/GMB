//! 组装机构管理员集合变更的链上 `federal_set_city_registry_admins` 裸 SCALE call data。
//!
//! 中文注释:把市注册局(CREG)某城市的全量管理员集合(当前集合 ± 本次增删)组装成
//! `AdminProfile` 列表,编码为与链端逐字节对齐的 admin-set call data。
//! - CREG 机构 cid 确定性:`official_institution_cid("CITY", province_code, city_code, "", "CREG", ...)`;
//! - 机构主账户 = `derive_account(creg_cid, "主账户")`(= AdminAccounts 键 / federal_set 的 account);
//! - 逐人实名锚(cid_number + 姓名)复用 `registration_call::resolve_admin_identity_conn`;
//! - 阈值取多数 `⌊n/2⌋+1`(满足链端 `2*threshold > admins_len && threshold <= admins_len`)。
//! onchina 只产 call data,不提交 extrinsic;冷钱包解码核对后冷签 origin 并由 CitizenWallet 提交。

use postgres::Client;

use crate::auth::login::parse_sr25519_pubkey_bytes;
use crate::core::institution_call::{
    encode_admin_set_call, AdminProfileArg, AdminSetCallArgs, AdminSourceTag, ChainCall,
    FEDERAL_SET_CITY_REGISTRY_ADMINS_CALL_INDEX, GENESIS_ADMINS_PALLET_INDEX,
    PROPOSE_ADMIN_SET_CHANGE_CALL_INDEX,
};
use crate::institution::subjects::registration_call::resolve_admin_identity_conn;

/// CREG 机构码(市注册局)。
const CREG_INSTITUTION_CODE: &[u8; 4] = b"CREG";
/// FRG 机构码(联邦注册局)。
const FRG_INSTITUTION_CODE: &[u8; 4] = b"FRG\0";
/// 官方机构 cid 的城市级种子 scope(与 gov 创世播种同值)。
const CITY_SEED_SCOPE: &str = "CITY";
/// CREG 主账户保留名(与链端 `primitives::account_derive` 一致)。
const MAIN_ACCOUNT_NAME: &str = "主账户";

/// 把 hex 公钥解析成 32 字节裸账户。
fn account_hex_to_bytes(account_hex: &str) -> Result<[u8; 32], String> {
    parse_sr25519_pubkey_bytes(account_hex)
        .ok_or_else(|| format!("invalid admin account pubkey: {account_hex}"))
}

/// 按管理员进链账户组装单个 `AdminProfileArg`(实名锚联表派生,职务/任期留空,来源 Registry)。
fn admin_profile_for(
    conn: &mut Client,
    admin_account_hex: &str,
) -> Result<AdminProfileArg, String> {
    let account = account_hex_to_bytes(admin_account_hex)?;
    let identity = resolve_admin_identity_conn(conn, admin_account_hex);
    Ok(AdminProfileArg {
        account,
        admin_cid_number: identity.admin_cid_number,
        name: identity.name,
        // 市注册局管理员无对外职务/任期(注册局层);链上留空。
        title: Vec::new(),
        term_start: 0,
        term_end: 0,
        source: AdminSourceTag::Registry,
    })
}

/// 组装 CREG 某城市「全量管理员集合 ± 本次变更」的 federal_set 裸 call data。
///
/// `province_name`/`city_name`/`city_code`:已由调用方按 created_by 的联邦作用域校验解析;
/// `current_admin_accounts`:该城市当前 CREG 管理员进链账户(hex)列表;
/// `add`:true=把 `delta_admin_account` 并入集合(创建),false=从集合移除(删除)。
pub(crate) fn build_city_registry_admin_set_call_data(
    conn: &mut Client,
    province_name: &str,
    city_name: &str,
    city_code: &str,
    current_admin_accounts: &[String],
    delta_admin_account: &str,
    add: bool,
) -> Result<ChainCall, String> {
    let Some(province_code) = crate::cid::china::province_code_by_name(province_name) else {
        return Err(format!("province_code not found for {province_name}"));
    };
    // CREG 机构 cid 确定性派生(与生成时同一生成器,exists_fn 恒 false 仅取确定性结果)。
    let creg_cid = crate::cid::official_institution_cid::<std::convert::Infallible>(
        CITY_SEED_SCOPE,
        province_code,
        city_code,
        "",
        std::str::from_utf8(CREG_INSTITUTION_CODE).expect("CREG ascii"),
        province_name,
        city_name,
        |_| Ok(false),
    )
    .map_err(|e| format!("resolve CREG cid failed: {e:?}"))?;
    let Some(main_account_hex) =
        crate::institution::accounts::derive::derive_account(creg_cid.as_str(), MAIN_ACCOUNT_NAME)
    else {
        return Err(format!("derive CREG main_account failed for {creg_cid}"));
    };
    let account = account_hex_to_bytes(main_account_hex.as_str())?;

    // 构造目标账户集合(去重,按 add/remove 合并 delta)。
    let mut target_accounts: Vec<String> = Vec::new();
    for acc in current_admin_accounts {
        if !target_accounts.iter().any(|a| a.eq_ignore_ascii_case(acc)) {
            target_accounts.push(acc.clone());
        }
    }
    let delta_in = target_accounts
        .iter()
        .any(|a| a.eq_ignore_ascii_case(delta_admin_account));
    if add {
        if !delta_in {
            target_accounts.push(delta_admin_account.to_string());
        }
    } else {
        target_accounts.retain(|a| !a.eq_ignore_ascii_case(delta_admin_account));
    }
    if target_accounts.is_empty() {
        return Err("city registry admin set cannot be empty".to_string());
    }

    let mut admins: Vec<AdminProfileArg> = Vec::with_capacity(target_accounts.len());
    for acc in &target_accounts {
        admins.push(admin_profile_for(conn, acc)?);
    }
    // 阈值:多数 ⌊n/2⌋+1,满足链端 2*threshold>admins_len && threshold<=admins_len。
    let threshold = (admins.len() as u32) / 2 + 1;

    Ok(encode_admin_set_call(&AdminSetCallArgs {
        pallet_index: GENESIS_ADMINS_PALLET_INDEX,
        call_index: FEDERAL_SET_CITY_REGISTRY_ADMINS_CALL_INDEX,
        institution_code: *CREG_INSTITUTION_CODE,
        account,
        admins,
        threshold,
    }))
}

/// 组装联邦注册局(FRG)「全量管理员集合,old→new 替换」的 `propose_admin_set_change` 裸 call data。
///
/// 中文注释:FRG 在 GenesisAdmins(创世),替换自身管理员走 genesis `propose_admin_set_change`
/// (call 0,FRG 管理员内部投票),非联邦特权(特权只用于 FRG 向下设 CREG)。
/// `current_admin_accounts`:当前全部 FRG 管理员进链账户;把 `old` 换成 `new` 后重建全集。
pub(crate) fn build_federal_registry_admin_set_call_data(
    conn: &mut Client,
    current_admin_accounts: &[String],
    old_admin_account: &str,
    new_admin_account: &str,
) -> Result<ChainCall, String> {
    let Some(frg_cid) = crate::domains::gov::service::federal_registry_cid_number() else {
        return Err("FRG cid not found in genesis seed".to_string());
    };
    let Some(main_account_hex) =
        crate::institution::accounts::derive::derive_account(frg_cid, MAIN_ACCOUNT_NAME)
    else {
        return Err(format!("derive FRG main_account failed for {frg_cid}"));
    };
    let account = account_hex_to_bytes(main_account_hex.as_str())?;

    // 全量集合 old→new 替换(去重),确保 new 在集合内。
    let mut target_accounts: Vec<String> = Vec::new();
    for acc in current_admin_accounts {
        let mapped = if acc.eq_ignore_ascii_case(old_admin_account) {
            new_admin_account
        } else {
            acc.as_str()
        };
        if !target_accounts
            .iter()
            .any(|a| a.eq_ignore_ascii_case(mapped))
        {
            target_accounts.push(mapped.to_string());
        }
    }
    if !target_accounts
        .iter()
        .any(|a| a.eq_ignore_ascii_case(new_admin_account))
    {
        target_accounts.push(new_admin_account.to_string());
    }
    if target_accounts.is_empty() {
        return Err("federal registry admin set cannot be empty".to_string());
    }

    let mut admins: Vec<AdminProfileArg> = Vec::with_capacity(target_accounts.len());
    for acc in &target_accounts {
        admins.push(admin_profile_for(conn, acc)?);
    }
    let threshold = (admins.len() as u32) / 2 + 1;

    Ok(encode_admin_set_call(&AdminSetCallArgs {
        pallet_index: GENESIS_ADMINS_PALLET_INDEX,
        call_index: PROPOSE_ADMIN_SET_CHANGE_CALL_INDEX,
        institution_code: *FRG_INSTITUTION_CODE,
        account,
        admins,
        threshold,
    }))
}
