//! 启动期 / admin 后台同步:从链上 SfidSystem storage 读三把账户公钥。
//!
//! 上层调用方 [`crate::key_admins`] 用本函数把链上结果回写本地 `chain_keyring_state`,
//! 与 admin 后台显示的"主/备账户"信息保持一致。

use crate::key_admins::chain_keyring::ChainKeyringState;
use serde_json::json;
use std::hash::Hasher;
use twox_hash::XxHash64;

use super::state_query::chain_rpc_call;

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

fn parse_chain_account_storage(raw: Option<&str>, field: &str) -> Result<String, String> {
    let Some(raw) = raw else {
        return Err(format!("chain storage `{field}` is empty"));
    };
    let bytes = hex::decode(raw.trim_start_matches("0x"))
        .map_err(|_| format!("chain storage `{field}` is not valid hex"))?;
    let account = match bytes.len() {
        32 => bytes,
        33 if bytes.first().copied() == Some(1) => bytes[1..33].to_vec(),
        _ => {
            return Err(format!(
                "chain storage `{field}` has unexpected AccountId encoding length {}",
                bytes.len()
            ))
        }
    };
    Ok(format!("0x{}", hex::encode(account)))
}

/// 同步读链上 `SfidMainAccount` / `SfidBackupAccount1` / `SfidBackupAccount2`,
/// 组装成 `ChainKeyringState`。
pub(crate) async fn fetch_chain_keyring_from_chain() -> Result<ChainKeyringState, String> {
    async fn fetch_pubkey(storage_name: &str) -> Result<String, String> {
        let storage_key = format!(
            "0x{}{}",
            hex::encode(twox_128(b"SfidSystem")),
            hex::encode(twox_128(storage_name.as_bytes()))
        );
        let raw = chain_rpc_call("state_getStorage", json!([storage_key])).await?;
        parse_chain_account_storage(raw.as_str(), storage_name)
    }

    let main_pubkey = fetch_pubkey("SfidMainAccount").await?;
    let backup_a_pubkey = fetch_pubkey("SfidBackupAccount1").await?;
    let backup_b_pubkey = fetch_pubkey("SfidBackupAccount2").await?;
    Ok(ChainKeyringState::new(
        main_pubkey,
        backup_a_pubkey,
        backup_b_pubkey,
    ))
}
