// 清算行注册机构多签的链上查询。
//
// 中文注释:
// - 本文件只读 `DuoqianManage::Institutions` / `InstitutionAccounts`。
// - `OffchainTransaction::ClearingBankNodes` 已拆到
//   `offchain_transaction::endpoint`,避免"节点声明"与"机构多签"混在一起。

use codec::{Decode, Encode};
use serde_json::Value;
use sp_core::ConstU32;
use sp_runtime::{AccountId32, BoundedVec};
use std::time::Duration;

use crate::governance::signing::pubkey_to_ss58;
use crate::governance::storage_keys;
use crate::shared::{constants::RPC_RESPONSE_LIMIT_SMALL, rpc};

use crate::offchain::common::types::{
    AccountWithBalance, InstitutionDetail, InstitutionProposalPage,
};

const RPC_REQUEST_TIMEOUT: Duration = Duration::from_secs(3);

fn rpc_post(method: &str, params: Value) -> Result<Value, String> {
    rpc::rpc_post(
        method,
        params,
        RPC_REQUEST_TIMEOUT,
        RPC_RESPONSE_LIMIT_SMALL,
    )
}

/// SCALE 编码 sfid_id 的 `BoundedVec<u8, ConstU32<64>>` 形式(用作 storage key data)。
///
/// 字段编码:`Compact<u32>(len)` + `bytes`。
fn encode_sfid_key_data(sfid_id: &str) -> Result<Vec<u8>, String> {
    let raw = sfid_id.as_bytes();
    if raw.is_empty() || raw.len() > 64 {
        return Err(format!("sfid_id 长度需在 1..=64 字节,实际:{}", raw.len()));
    }
    let bv: BoundedVec<u8, ConstU32<64>> = raw
        .to_vec()
        .try_into()
        .map_err(|_| "sfid_id 超出链上 BoundedVec<u8, 64>".to_string())?;
    Ok(bv.encode())
}

// ─── 机构详情查询(duoqian-manage::Institutions / InstitutionAccounts) ──────

/// 链端 `InstitutionInfo<AdminList, AccountId, BlockNumber, AccountName, A3, SubType, SfidId>`
/// 在节点端的 SCALE 镜像。字段顺序必须与 [`citizenchain/runtime/transaction/duoqian-manage/src/institution/types.rs`]
/// 严格一致(Encode/Decode 派生按声明顺序)。
///
/// runtime 实例化的具体类型:
/// - `AdminList = BoundedVec<AccountId32, MaxAdmins>`(MaxAdmins 由 runtime 设)
/// - `AccountId = AccountId32`
/// - `BlockNumber = u32`(citizenchain runtime)
/// - `AccountName = BoundedVec<u8, ConstU32<128>>`
/// - `A3 = BoundedVec<u8, ConstU32<8>>`
/// - `SubType = BoundedVec<u8, ConstU32<32>>`
/// - `SfidId = BoundedVec<u8, ConstU32<64>>`
#[derive(Decode, Encode)]
struct OnChainInstitution {
    institution_name: BoundedVec<u8, ConstU32<128>>,
    main_address: AccountId32,
    fee_address: AccountId32,
    admin_count: u32,
    threshold: u32,
    duoqian_admins: BoundedVec<AccountId32, ConstU32<64>>,
    creator: AccountId32,
    created_at: u32,
    status: OnChainInstitutionStatus,
    account_count: u32,
    a3: BoundedVec<u8, ConstU32<8>>,
    sub_type: Option<BoundedVec<u8, ConstU32<32>>>,
    parent_sfid_id: Option<BoundedVec<u8, ConstU32<64>>>,
}

/// 与 [`citizenchain/runtime/transaction/duoqian-manage/src/institution/types.rs::InstitutionLifecycleStatus`] 对齐。
#[derive(Decode, Encode, Clone, Copy, PartialEq, Eq, Debug)]
enum OnChainInstitutionStatus {
    Pending,
    Active,
    Closed,
}

impl OnChainInstitutionStatus {
    fn label(self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Active => "Active",
            Self::Closed => "Closed",
        }
    }
}

/// 链端 `InstitutionAccountInfo<AccountId, Balance, BlockNumber>` 镜像。
#[derive(Decode, Encode)]
struct OnChainInstitutionAccount {
    address: AccountId32,
    initial_balance: u128,
    status: OnChainInstitutionStatus,
    is_default: bool,
    created_at: u32,
}

/// `frame_system::AccountInfo` 头部 16 字节(nonce/consumers/providers/sufficients
/// 4 个 u32),紧接 16 字节 `data.free` u128。
const ACCOUNT_INFO_HEADER_LEN: usize = 16;
const ACCOUNT_INFO_FREE_LEN: usize = 16;

/// 友好标签:由 a3 + sub_type 推。本地化文案,不参与链上验签。
fn institution_type_label(a3: &[u8], _sub_type: Option<&[u8]>) -> String {
    match a3 {
        b"SFR" => "私法人多签".to_string(),
        b"FFR" => "私非法人多签".to_string(),
        b"GFR" => "公法人多签".to_string(),
        b"GMR" => "公权多签".to_string(),
        b"GAR" => "公安局多签".to_string(),
        _ => String::from_utf8_lossy(a3).to_string(),
    }
}

/// 把"分"格式化成 `xxx.xx`(沿用 chain/balance/handler.rs 同款约定)。
fn format_yuan(min_units: u128) -> String {
    let yuan = min_units / 100;
    let cents = (min_units % 100) as u8;
    format!("{}.{:02}", yuan, cents)
}

/// 用 RPC `state_getStorage` 拉单账户的 free 余额(分)。
/// 不存在 → 返回 0。
fn fetch_account_free_balance(account: &AccountId32) -> Result<u128, String> {
    let mut storage_data = Vec::with_capacity(32);
    let raw: [u8; 32] = (*account).clone().into();
    storage_data.extend_from_slice(&raw);
    // System.Account 用 Blake2_128Concat 哈希器,key = blake2_128(account) ++ account
    let mut hashed = Vec::with_capacity(48);
    hashed.extend_from_slice(&storage_keys::blake2b_128(&raw));
    hashed.extend_from_slice(&raw);
    let key = format!(
        "0x{}{}{}",
        hex::encode(storage_keys::twox_128(b"System")),
        hex::encode(storage_keys::twox_128(b"Account")),
        hex::encode(hashed)
    );
    let result = rpc_post("state_getStorage", Value::Array(vec![Value::String(key)]))?;
    match result {
        Value::Null => Ok(0),
        Value::String(hex_data) => {
            let clean = hex_data.strip_prefix("0x").unwrap_or(&hex_data);
            let bytes =
                hex::decode(clean).map_err(|e| format!("System.Account hex 解码失败:{e}"))?;
            let need = ACCOUNT_INFO_HEADER_LEN + ACCOUNT_INFO_FREE_LEN;
            if bytes.len() < need {
                return Err(format!(
                    "AccountInfo bytes too short: got {}, need >= {need}",
                    bytes.len()
                ));
            }
            let mut buf = [0u8; 16];
            buf.copy_from_slice(
                &bytes[ACCOUNT_INFO_HEADER_LEN..ACCOUNT_INFO_HEADER_LEN + ACCOUNT_INFO_FREE_LEN],
            );
            Ok(u128::from_le_bytes(buf))
        }
        _ => Err("state_getStorage 返回格式无效".to_string()),
    }
}

/// 链上查询某机构的多签信息。返回 `None` = 该 sfid_id 尚未创建机构(进入创建流程)。
///
/// 数据来源:
/// 1. `DuoqianManage::Institutions[sfid_id]` 取机构主体
/// 2. `state_getKeysPaged` + `DuoqianManage::InstitutionAccounts[sfid_id, *]` 取账户列表
/// 3. 每个账户的 `System::Account[address].data.free` 取链上余额
pub fn fetch_institution_detail(sfid_id: &str) -> Result<Option<InstitutionDetail>, String> {
    let key_data = encode_sfid_key_data(sfid_id)?;
    let key = storage_keys::map_key("DuoqianManage", "Institutions", &key_data);
    let result = rpc_post("state_getStorage", Value::Array(vec![Value::String(key)]))?;
    let raw = match result {
        Value::Null => return Ok(None),
        Value::String(s) => s,
        _ => return Err("state_getStorage 返回格式无效".to_string()),
    };
    let clean = raw.strip_prefix("0x").unwrap_or(&raw);
    let bytes = hex::decode(clean).map_err(|e| format!("Institutions hex 解码失败:{e}"))?;
    let inst = OnChainInstitution::decode(&mut &bytes[..])
        .map_err(|e| format!("Institutions SCALE 解码失败:{e}"))?;

    // 拉机构下所有账户(InstitutionAccounts[sfid_id, *] 是 DoubleMap)。
    let accounts = fetch_institution_accounts(sfid_id, &inst)?;

    // 主账户 / 费用账户 / 其它账户 分类(用 ss58 字符串比对,避免原始字节做 Eq)。
    let main_addr_bytes: [u8; 32] = inst.main_address.clone().into();
    let fee_addr_bytes: [u8; 32] = inst.fee_address.clone().into();
    let main_addr_ss58 = pubkey_to_ss58(&main_addr_bytes).unwrap_or_default();
    let fee_addr_ss58 = pubkey_to_ss58(&fee_addr_bytes).unwrap_or_default();
    let mut main_account: Option<AccountWithBalance> = None;
    let mut fee_account: Option<AccountWithBalance> = None;
    let mut other_accounts: Vec<AccountWithBalance> = Vec::new();
    for acc in accounts {
        if acc.address_ss58 == main_addr_ss58 {
            main_account = Some(acc);
        } else if acc.address_ss58 == fee_addr_ss58 {
            fee_account = Some(acc);
        } else {
            other_accounts.push(acc);
        }
    }
    let main_account = main_account.unwrap_or_else(|| {
        // 容错:如果 InstitutionAccounts 没显式列主账户(理论上不应该),
        // 就直接用 Institutions.main_address 拉余额拼一条最小记录。
        let bal = fetch_account_free_balance(&inst.main_address).unwrap_or(0);
        AccountWithBalance {
            account_name: "主账户".to_string(),
            address_ss58: pubkey_to_ss58(&main_addr_bytes).unwrap_or_default(),
            balance_min_units: bal.to_string(),
            balance_text: format_yuan(bal),
            is_default: true,
        }
    });
    let fee_account = fee_account.unwrap_or_else(|| {
        let bal = fetch_account_free_balance(&inst.fee_address).unwrap_or(0);
        AccountWithBalance {
            account_name: "费用账户".to_string(),
            address_ss58: pubkey_to_ss58(&fee_addr_bytes).unwrap_or_default(),
            balance_min_units: bal.to_string(),
            balance_text: format_yuan(bal),
            is_default: true,
        }
    });

    let duoqian_admins_ss58 = inst
        .duoqian_admins
        .iter()
        .filter_map(|a| {
            let raw: [u8; 32] = (*a).clone().into();
            pubkey_to_ss58(&raw).ok()
        })
        .collect();
    let creator_bytes: [u8; 32] = inst.creator.clone().into();
    let creator_ss58 = pubkey_to_ss58(&creator_bytes).unwrap_or_default();

    let institution_name = String::from_utf8(inst.institution_name.into_inner())
        .map_err(|_| "institution_name 非 UTF-8".to_string())?;
    let a3_bytes = inst.a3.into_inner();
    let sub_type_bytes = inst.sub_type.map(|v| v.into_inner());
    let label = institution_type_label(&a3_bytes, sub_type_bytes.as_deref());
    let a3 = String::from_utf8(a3_bytes).map_err(|_| "a3 非 UTF-8".to_string())?;
    let sub_type = match sub_type_bytes {
        Some(b) => Some(String::from_utf8(b).map_err(|_| "sub_type 非 UTF-8".to_string())?),
        None => None,
    };
    let parent_sfid_id = match inst.parent_sfid_id {
        Some(v) => Some(
            String::from_utf8(v.into_inner()).map_err(|_| "parent_sfid_id 非 UTF-8".to_string())?,
        ),
        None => None,
    };

    Ok(Some(InstitutionDetail {
        sfid_id: sfid_id.to_string(),
        institution_name,
        institution_type_label: label,
        a3,
        sub_type,
        parent_sfid_id,
        main_account,
        fee_account,
        other_accounts,
        admin_count: inst.admin_count,
        threshold: inst.threshold,
        duoqian_admins_ss58,
        status: inst.status.label().to_string(),
        creator_ss58,
        created_at: inst.created_at as u64,
        account_count: inst.account_count,
    }))
}

/// 用 `state_getKeysPaged` 列出 `InstitutionAccounts[sfid_id, *]` 全部账户名,
/// 然后逐个 `state_getStorage` 拉取账户内容并查链上余额。
fn fetch_institution_accounts(
    sfid_id: &str,
    inst: &OnChainInstitution,
) -> Result<Vec<AccountWithBalance>, String> {
    // 第一层 key 哈希器是 Blake2_128Concat,完整 storage key 前缀 =
    //   twox_128("DuoqianManage") ++ twox_128("InstitutionAccounts")
    //   ++ blake2_128(sfid_id_bytes) ++ sfid_id_bytes(BoundedVec 编码)
    let sfid_key = encode_sfid_key_data(sfid_id)?;
    let mut sfid_prefix_data = Vec::with_capacity(16 + sfid_key.len());
    sfid_prefix_data.extend_from_slice(&storage_keys::blake2b_128(&sfid_key));
    sfid_prefix_data.extend_from_slice(&sfid_key);
    let pallet = storage_keys::twox_128(b"DuoqianManage");
    let storage = storage_keys::twox_128(b"InstitutionAccounts");
    let prefix_hex = format!(
        "0x{}{}{}",
        hex::encode(pallet),
        hex::encode(storage),
        hex::encode(&sfid_prefix_data)
    );

    const PAGE: u32 = 100;
    let mut keys: Vec<String> = Vec::new();
    let mut start_key: Option<String> = None;
    loop {
        let mut params = vec![
            Value::String(prefix_hex.clone()),
            Value::Number(serde_json::Number::from(PAGE)),
        ];
        if let Some(s) = start_key.as_ref() {
            params.push(Value::String(s.clone()));
        }
        let result = rpc_post("state_getKeysPaged", Value::Array(params))?;
        let arr = result
            .as_array()
            .ok_or_else(|| "state_getKeysPaged 返回非数组".to_string())?;
        let n = arr.len();
        for k in arr {
            if let Some(s) = k.as_str() {
                keys.push(s.to_string());
            }
        }
        if n < PAGE as usize {
            break;
        }
        start_key = arr.last().and_then(|v| v.as_str().map(|s| s.to_string()));
        if start_key.is_none() {
            break;
        }
    }

    let mut out: Vec<AccountWithBalance> = Vec::with_capacity(keys.len());
    for key in keys {
        // 解 account_name:key 末尾段 = blake2_128(name_bytes_compact) ++ name_bytes_compact
        // name_bytes_compact = BoundedVec<u8>::encode = Compact(len) ++ bytes
        // 我们直接拉 storage 值,然后从 key 反推 account_name(便于和 inst 对账)。
        let value = rpc_post(
            "state_getStorage",
            Value::Array(vec![Value::String(key.clone())]),
        )?;
        let value_hex = match value {
            Value::Null => continue,
            Value::String(s) => s,
            _ => continue,
        };
        let clean = value_hex.strip_prefix("0x").unwrap_or(&value_hex);
        let bytes = hex::decode(clean).map_err(|e| format!("InstitutionAccounts hex:{e}"))?;
        let acc = match OnChainInstitutionAccount::decode(&mut &bytes[..]) {
            Ok(v) => v,
            Err(e) => {
                // 中文注释:单条解码失败不应阻止整页;打 stderr 让运维看见,继续下一条。
                eprintln!(
                    "[clearing-bank] InstitutionAccounts SCALE decode failed key={} err={}",
                    key, e
                );
                continue;
            }
        };
        let acc_name = decode_account_name_from_key(&key, &prefix_hex).unwrap_or_default();
        let acc_addr_bytes: [u8; 32] = acc.address.clone().into();
        let bal = fetch_account_free_balance(&acc.address).unwrap_or(0);
        out.push(AccountWithBalance {
            account_name: acc_name,
            address_ss58: pubkey_to_ss58(&acc_addr_bytes).unwrap_or_default(),
            balance_min_units: bal.to_string(),
            balance_text: format_yuan(bal),
            is_default: acc.is_default,
        });
    }

    // 给 unused 字段一个明确的 borrow,避免 dead_code 警告(future-proof:inst 字段未来可能用到)
    let _ = inst;
    Ok(out)
}

/// 从完整 storage key 反推第二层 key(account_name 字节)。
fn decode_account_name_from_key(full_key_hex: &str, sfid_prefix_hex: &str) -> Option<String> {
    let full = full_key_hex.strip_prefix("0x").unwrap_or(full_key_hex);
    let prefix = sfid_prefix_hex
        .strip_prefix("0x")
        .unwrap_or(sfid_prefix_hex);
    if !full.starts_with(prefix) {
        return None;
    }
    // 第二层后缀:blake2_128(account_name_compact) ++ account_name_compact
    let after = &full[prefix.len()..];
    if after.len() < 32 {
        return None;
    }
    let after_bytes = hex::decode(&after[32..]).ok()?;
    if after_bytes.is_empty() {
        return None;
    }
    // Compact<u32>(len) 解码,与 clearing_bank_watcher.rs 的实现一致(单字节模式占多数)
    let first = after_bytes[0];
    let (len, consumed) = match first & 0x03 {
        0 => ((first >> 2) as usize, 1usize),
        1 => {
            if after_bytes.len() < 2 {
                return None;
            }
            let v = ((first as u16) | ((after_bytes[1] as u16) << 8)) >> 2;
            (v as usize, 2)
        }
        2 => {
            if after_bytes.len() < 4 {
                return None;
            }
            let v = (first as u32)
                | ((after_bytes[1] as u32) << 8)
                | ((after_bytes[2] as u32) << 16)
                | ((after_bytes[3] as u32) << 24);
            ((v >> 2) as usize, 4)
        }
        _ => return None,
    };
    if after_bytes.len() < consumed + len {
        return None;
    }
    String::from_utf8(after_bytes[consumed..consumed + len].to_vec()).ok()
}

/// 机构提案列表分页。
///
/// 当前阶段返回空列表占位。提案存储在 `voting-engine::Proposals[id]`,
/// 按 sfid_id 过滤需要扫描全表 + 反查 ProposalMeta.institution_hex,
/// 实现略显重,放 follow-up 任务卡(本任务卡 §8 风险表)。
///
/// 前端 UI 展示"暂无提案"行兜底,未来填充时无需改 UI 结构。
pub fn fetch_institution_proposals(
    _sfid_id: &str,
    _start_id: u64,
    _page_size: u32,
) -> Result<InstitutionProposalPage, String> {
    Ok(InstitutionProposalPage {
        items: Vec::new(),
        has_more: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_sfid_key_data_round_trip() {
        let raw = "GFR-LN001-CB0C-617776487-20260222";
        let encoded = encode_sfid_key_data(raw).unwrap();
        // Compact<u32> 长度前缀(单字节模式 raw.len() < 64)+ raw 字节
        assert_eq!(encoded[0], (raw.len() as u8) << 2);
        assert_eq!(&encoded[1..], raw.as_bytes());
    }

    #[test]
    fn empty_sfid_rejected() {
        let err = encode_sfid_key_data("").unwrap_err();
        assert!(err.contains("长度"));
    }

    #[test]
    fn over_long_sfid_rejected() {
        let s = "a".repeat(65);
        let err = encode_sfid_key_data(&s).unwrap_err();
        assert!(err.contains("长度"));
    }
}
