//! 地址变更链上 call data 编码器。
//!
//! 中文注释:OnChina 只构造裸 SCALE call data 和 QR 动作码,不在这里提交 extrinsic。

use parity_scale_codec::{Compact, Encode};

use crate::{cid::china::china_sqlite_hash, parse_sr25519_pubkey_bytes};

use super::{
    model::{AddressChainAction, AddressChainCallInput, AddressChainCallOutput},
    version::ADDRESS_CATALOG_VERSION,
};

/// AddressRegistry pallet 在 runtime 中的索引。
pub(crate) const ADDRESS_REGISTRY_PALLET_INDEX: u8 = 35;
pub(crate) const CALL_SET_CATALOG_VERSION: u8 = 0;
pub(crate) const CALL_SET_ADDRESS_NAME: u8 = 1;
pub(crate) const CALL_REMOVE_ADDRESS_NAME: u8 = 2;
pub(crate) const CALL_SET_ADDRESS: u8 = 3;
pub(crate) const CALL_REMOVE_ADDRESS: u8 = 4;

fn push_vec(out: &mut Vec<u8>, value: &[u8]) {
    out.extend(Compact(value.len() as u32).encode());
    out.extend_from_slice(value);
}

fn required_text(value: &Option<String>, field: &str) -> Result<String, String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| format!("{field} is required"))
}

fn parse_h256(raw: &str) -> Result<[u8; 32], String> {
    let normalized = raw.trim().strip_prefix("0x").unwrap_or(raw.trim());
    let bytes = hex::decode(normalized).map_err(|e| format!("catalog_hash hex invalid: {e}"))?;
    if bytes.len() != 32 {
        return Err("catalog_hash must be 32 bytes hex".to_string());
    }
    let mut out = [0_u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn registrar_account_bytes(input: &AddressChainCallInput) -> Result<[u8; 32], String> {
    parse_sr25519_pubkey_bytes(input.registrar_account.trim())
        .ok_or_else(|| "registrar_account is invalid".to_string())
}

fn action_code(call_index: u8) -> u16 {
    crate::core::institution_call::chain_action_code(ADDRESS_REGISTRY_PALLET_INDEX, call_index)
}

fn output(
    call_index: u8,
    call_data: Vec<u8>,
    review_title: &'static str,
) -> AddressChainCallOutput {
    AddressChainCallOutput {
        action: action_code(call_index),
        pallet_index: ADDRESS_REGISTRY_PALLET_INDEX,
        call_index,
        call_data_hex: format!("0x{}", hex::encode(call_data)),
        review_title,
    }
}

pub(crate) fn build_address_chain_call(
    input: &AddressChainCallInput,
) -> Result<AddressChainCallOutput, String> {
    let registrar_account = registrar_account_bytes(input)?;
    let mut out = Vec::new();
    out.push(ADDRESS_REGISTRY_PALLET_INDEX);
    let call_index = match input.action {
        AddressChainAction::SetCatalogVersion => CALL_SET_CATALOG_VERSION,
        AddressChainAction::SetAddressName => CALL_SET_ADDRESS_NAME,
        AddressChainAction::RemoveAddressName => CALL_REMOVE_ADDRESS_NAME,
        AddressChainAction::SetAddress => CALL_SET_ADDRESS,
        AddressChainAction::RemoveAddress => CALL_REMOVE_ADDRESS,
    };
    out.push(call_index);
    out.extend_from_slice(&registrar_account);

    match input.action {
        AddressChainAction::SetCatalogVersion => {
            let catalog_version = input
                .catalog_version
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .unwrap_or(ADDRESS_CATALOG_VERSION);
            let catalog_hash = match input.catalog_hash.as_deref() {
                Some(raw) if !raw.trim().is_empty() => parse_h256(raw)?,
                _ => parse_h256(china_sqlite_hash()?.as_str())?,
            };
            push_vec(&mut out, catalog_version.as_bytes());
            out.extend_from_slice(&catalog_hash);
            Ok(output(call_index, out, "设置地址库版本"))
        }
        AddressChainAction::SetAddressName => {
            let province_code = required_text(&input.province_code, "province_code")?;
            let city_code = required_text(&input.city_code, "city_code")?;
            let town_code = required_text(&input.town_code, "town_code")?;
            let address_name_code = required_text(&input.address_name_code, "address_name_code")?;
            let address_name = required_text(&input.address_name, "address_name")?;
            push_vec(&mut out, province_code.as_bytes());
            push_vec(&mut out, city_code.as_bytes());
            push_vec(&mut out, town_code.as_bytes());
            push_vec(&mut out, address_name_code.as_bytes());
            push_vec(&mut out, address_name.as_bytes());
            Ok(output(call_index, out, "设置镇下地址名称"))
        }
        AddressChainAction::RemoveAddressName => {
            let province_code = required_text(&input.province_code, "province_code")?;
            let city_code = required_text(&input.city_code, "city_code")?;
            let town_code = required_text(&input.town_code, "town_code")?;
            let address_name_code = required_text(&input.address_name_code, "address_name_code")?;
            push_vec(&mut out, province_code.as_bytes());
            push_vec(&mut out, city_code.as_bytes());
            push_vec(&mut out, town_code.as_bytes());
            push_vec(&mut out, address_name_code.as_bytes());
            Ok(output(call_index, out, "删除镇下地址名称"))
        }
        AddressChainAction::SetAddress => {
            let province_code = required_text(&input.province_code, "province_code")?;
            let city_code = required_text(&input.city_code, "city_code")?;
            let town_code = required_text(&input.town_code, "town_code")?;
            let address_name_code = required_text(&input.address_name_code, "address_name_code")?;
            let address_local_no = input
                .address_local_no
                .as_deref()
                .map(str::trim)
                .unwrap_or("");
            let address_detail = input.address_detail.as_deref().map(str::trim).unwrap_or("");
            push_vec(&mut out, province_code.as_bytes());
            push_vec(&mut out, city_code.as_bytes());
            push_vec(&mut out, town_code.as_bytes());
            push_vec(&mut out, address_name_code.as_bytes());
            push_vec(&mut out, address_local_no.as_bytes());
            push_vec(&mut out, address_detail.as_bytes());
            Ok(output(call_index, out, "设置完整地址"))
        }
        AddressChainAction::RemoveAddress => {
            let province_code = required_text(&input.province_code, "province_code")?;
            let city_code = required_text(&input.city_code, "city_code")?;
            let town_code = required_text(&input.town_code, "town_code")?;
            let address_name_code = required_text(&input.address_name_code, "address_name_code")?;
            let address_local_no = input
                .address_local_no
                .as_deref()
                .map(str::trim)
                .unwrap_or("");
            let address_detail = input.address_detail.as_deref().map(str::trim).unwrap_or("");
            push_vec(&mut out, province_code.as_bytes());
            push_vec(&mut out, city_code.as_bytes());
            push_vec(&mut out, town_code.as_bytes());
            push_vec(&mut out, address_name_code.as_bytes());
            push_vec(&mut out, address_local_no.as_bytes());
            push_vec(&mut out, address_detail.as_bytes());
            Ok(output(call_index, out, "删除完整地址"))
        }
    }
}
