// 链上余额查询。
//
// 查询 frame_system 的 `Account` 存储项，返回账户的 free 余额。
//
// 链上 `Account` 是 `StorageMap<Blake2_128Concat, AccountId32, AccountInfo>`，
// 因此 storage key = twox_128("System") + twox_128("Account") + blake2_128(account) + account。
//
// AccountInfo 在 substrate 中的 SCALE 编码布局：
//   nonce: u32        (4 bytes)
//   consumers: u32    (4 bytes)
//   providers: u32    (4 bytes)
//   sufficients: u32  (4 bytes)
//   data: AccountData {
//       free:     u128 (16 bytes)
//       reserved: u128 (16 bytes)
//       frozen:   u128 (16 bytes)
//       flags:    u128 (16 bytes)
//   }
//
// 我们只关心 free，对应偏移 16..32 字节。

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use blake2::{
    digest::{Update, VariableOutput},
    Blake2bVar,
};
use serde::{Deserialize, Serialize};
use std::hash::Hasher;
use twox_hash::XxHash64;

use crate::key_admins;
use crate::{api_error, require_admin_any, ApiResponse, AppState};

/// `frame_system::AccountInfo` 头部 4 个 u32（nonce/consumers/providers/sufficients）共 16 字节，
/// 紧接着是 `data: AccountData`，其中 `free: u128` 是 AccountData 的第一个字段（即整个结构的第 16..32 字节）。
const ACCOUNT_INFO_HEADER_LEN: usize = 16;
const ACCOUNT_INFO_FREE_LEN: usize = 16;

fn twox_128(input: &[u8]) -> [u8; 16] {
    let mut h1 = XxHash64::with_seed(0);
    h1.write(input);
    let mut h2 = XxHash64::with_seed(1);
    h2.write(input);
    let mut out = [0u8; 16];
    out[..8].copy_from_slice(&h1.finish().to_le_bytes());
    out[8..].copy_from_slice(&h2.finish().to_le_bytes());
    out
}

fn blake2_128(input: &[u8]) -> [u8; 16] {
    let mut hasher = Blake2bVar::new(16).expect("blake2b 128-bit");
    hasher.update(input);
    let mut out = [0u8; 16];
    hasher
        .finalize_variable(&mut out)
        .expect("blake2b finalize");
    out
}

fn parse_account_id32(pubkey_hex: &str) -> Result<[u8; 32], String> {
    let trimmed = pubkey_hex.trim();
    let no_prefix = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    let bytes = hex::decode(no_prefix).map_err(|e| format!("invalid pubkey hex: {e}"))?;
    if bytes.len() != 32 {
        return Err(format!("pubkey must be 32 bytes, got {}", bytes.len()));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn build_system_account_storage_key(account: &[u8; 32]) -> String {
    let mut key = Vec::with_capacity(16 + 16 + 16 + 32);
    key.extend_from_slice(&twox_128(b"System"));
    key.extend_from_slice(&twox_128(b"Account"));
    key.extend_from_slice(&blake2_128(account));
    key.extend_from_slice(account);
    format!("0x{}", hex::encode(key))
}

/// 查询账户的链上 free 余额（最小单位：分）。
///
/// 通过 SFID 后端配置的 RPC 节点（`SFID_CHAIN_RPC_URL` / `SFID_CHAIN_WS_URL`）
/// 调用 `state_getStorage` 拉取 `System.Account`，截取 AccountInfo 中的 `data.free`。
///
/// 返回 0 当账户在链上不存在（未上链）。
pub(crate) async fn query_free_balance(account_pubkey_hex: &str) -> Result<u128, String> {
    let account = parse_account_id32(account_pubkey_hex)?;
    let storage_key = build_system_account_storage_key(&account);

    let result = key_admins::call_chain_state_get_storage(&storage_key).await?;
    let Some(raw) = result else {
        return Ok(0);
    };
    let no_prefix = raw.trim_start_matches("0x");
    let bytes = hex::decode(no_prefix).map_err(|e| format!("decode account storage hex: {e}"))?;

    // AccountInfo 头 16 字节（4*u32），紧接着是 AccountData.free（u128, LE, 16 字节）
    let need = ACCOUNT_INFO_HEADER_LEN + ACCOUNT_INFO_FREE_LEN;
    if bytes.len() < need {
        return Err(format!(
            "AccountInfo bytes too short: got {}, need >= {need}",
            bytes.len()
        ));
    }
    let mut buf = [0u8; 16];
    buf.copy_from_slice(&bytes[ACCOUNT_INFO_HEADER_LEN..ACCOUNT_INFO_HEADER_LEN + ACCOUNT_INFO_FREE_LEN]);
    Ok(u128::from_le_bytes(buf))
}

/// 把"分"格式化成 `xxx.xx 元`，遵循链上 TOKEN_DECIMALS=2 的精度。
pub(crate) fn format_yuan(min_units: u128) -> String {
    let yuan = min_units / 100;
    let cents = (min_units % 100) as u8;
    format!("{}.{:02}", yuan, cents)
}

#[derive(Debug, Deserialize)]
pub(crate) struct ChainBalanceQuery {
    pub(crate) account_pubkey: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ChainBalanceOutput {
    /// 32 字节 hex 公钥（与请求一致）
    pub(crate) account_pubkey: String,
    /// 链上 free 余额（最小单位：分）
    pub(crate) balance_min_units: String,
    /// 显示用文本，已按 1 元 = 100 分换算，保留 2 位小数
    pub(crate) balance_text: String,
    /// 单位标签（始终为 "元"）
    pub(crate) unit: &'static str,
}

/// `GET /api/v1/admin/chain/balance?account_pubkey=0x...`
///
/// 任意已登录管理员可查；查询本地全节点的 `System.Account.free`，
/// 返回原始最小单位和按 `xxx.xx` 格式化好的元金额。
pub(crate) async fn admin_query_chain_balance(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ChainBalanceQuery>,
) -> impl IntoResponse {
    if let Err(resp) = require_admin_any(&state, &headers) {
        return resp;
    }
    let pubkey = query.account_pubkey.trim().to_string();
    if pubkey.is_empty() {
        return api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "account_pubkey is required",
        );
    }
    match query_free_balance(pubkey.as_str()).await {
        Ok(min_units) => Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: ChainBalanceOutput {
                account_pubkey: pubkey,
                balance_min_units: min_units.to_string(),
                balance_text: format_yuan(min_units),
                unit: "元",
            },
        })
        .into_response(),
        Err(err) => api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            &format!("query chain balance failed: {err}"),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_yuan_basic() {
        assert_eq!(format_yuan(0), "0.00");
        assert_eq!(format_yuan(1), "0.01");
        assert_eq!(format_yuan(99), "0.99");
        assert_eq!(format_yuan(100), "1.00");
        assert_eq!(format_yuan(123456), "1234.56");
    }

    #[test]
    fn parse_account_id32_strict_32_bytes() {
        let pk = "0x".to_string() + &"ab".repeat(32);
        let bytes = parse_account_id32(&pk).unwrap();
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn build_storage_key_has_expected_prefix() {
        let pk = [0u8; 32];
        let key = build_system_account_storage_key(&pk);
        // System.Account prefix = twox_128("System") + twox_128("Account")
        assert!(key.starts_with("0x"));
        assert_eq!(key.len(), 2 + (16 + 16 + 16 + 32) * 2);
    }
}
