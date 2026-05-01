//! 主备账户轮换推链:`SfidSystem::rotate_sfid_keys(new_backup)`。
//!
//! 由 [`crate::key_admins`] 模块的 `admin_chain_keyring_rotate_commit` handler
//! 在轮换 commit 阶段调用。发起者必须是 `backup_1` 或 `backup_2` 私钥(链上 verifier 校验)。
//!
//! 严格遵守 SFID PoW 链推链三件套(`feedback_sfid_pow_chain_recipe`):
//!   ① 显式 nonce(legacy RPC `system_account_next_index`)
//!   ② extrinsic immortal
//!   ③ 只等 `InBestBlock`
//!
//! ADR `04-decisions/sfid/2026-04-07-subxt-0.43-pow-chain-quirks.md`。

use sp_core::Pair;
use subxt::{
    config::substrate::{AccountId32, MultiSignature},
    dynamic::{tx, Value},
    OnlineClient, PolkadotConfig,
};

use crate::chain::url::chain_ws_url;
use crate::key_admins::chain_keyring::try_load_signing_key_from_seed;
use crate::parse_sr25519_pubkey_bytes;

#[derive(Debug, Clone)]
pub(crate) struct ChainRotateReceipt {
    pub(crate) tx_hash: String,
    pub(crate) block_number: u64,
}

fn parse_account_id32(pubkey: &str) -> Result<[u8; 32], String> {
    parse_sr25519_pubkey_bytes(pubkey).ok_or_else(|| "invalid sr25519 account pubkey".to_string())
}

/// 提交 `SfidSystem::rotate_sfid_keys(new_backup)` extrinsic。
///
/// 发起者(`initiator_pubkey` + `initiator_seed_hex`)必须持有 backup 私钥。
pub(crate) async fn submit_rotate_sfid_keys_extrinsic(
    initiator_pubkey: &str,
    initiator_seed_hex: &str,
    new_backup_pubkey: &str,
) -> Result<ChainRotateReceipt, String> {
    let ws_url =
        chain_ws_url().map_err(|e| format!("rotate_sfid_keys submit failed: {e}"))?;
    let client = OnlineClient::<PolkadotConfig>::from_url(ws_url.clone())
        .await
        .map_err(|e| {
            format!("rotate_sfid_keys submit failed: chain websocket connect failed: {e}")
        })?;
    // ① legacy RPC client,用于显式取 nonce
    let rpc_client = subxt::backend::rpc::RpcClient::from_insecure_url(ws_url.as_str())
        .await
        .map_err(|e| format!("rotate_sfid_keys submit failed: legacy rpc connect failed: {e}"))?;
    let legacy_rpc = subxt::backend::legacy::LegacyRpcMethods::<PolkadotConfig>::new(rpc_client);

    let signer_account = AccountId32(
        parse_account_id32(initiator_pubkey)
            .map_err(|e| format!("rotate_sfid_keys submit failed: {e}"))?,
    );
    let chain_nonce = legacy_rpc
        .system_account_next_index(&signer_account)
        .await
        .map_err(|e| format!("rotate_sfid_keys submit failed: fetch account nonce failed: {e}"))?;
    let new_backup_account = parse_account_id32(new_backup_pubkey)
        .map_err(|e| format!("rotate_sfid_keys submit failed: {e}"))?;
    let payload = tx(
        "SfidSystem",
        "rotate_sfid_keys",
        vec![Value::from_bytes(new_backup_account)],
    );
    // ② immortal + 显式 nonce
    let params = subxt::config::DefaultExtrinsicParamsBuilder::<PolkadotConfig>::new()
        .immortal()
        .nonce(chain_nonce)
        .build();
    let mut partial_tx = client
        .tx()
        .create_partial(&payload, &signer_account, params)
        .await
        .map_err(|e| format!("rotate_sfid_keys submit failed: build extrinsic failed: {e}"))?;
    let signing_key = try_load_signing_key_from_seed(initiator_seed_hex)
        .map_err(|e| format!("rotate_sfid_keys submit failed: {e}"))?;
    let signature = signing_key.sign(&partial_tx.signer_payload()).0;
    let extrinsic = partial_tx
        .sign_with_account_and_signature(&signer_account, &MultiSignature::Sr25519(signature));
    let tx_hash = format!("0x{}", hex::encode(extrinsic.hash().as_ref()));

    let mut submitted = extrinsic
        .submit_and_watch()
        .await
        .map_err(|e| format!("rotate_sfid_keys submit failed: submit_and_watch failed: {e}"))?;
    // ③ 只等 InBestBlock
    let in_block = tokio::time::timeout(std::time::Duration::from_secs(120), async {
        use subxt::tx::TxStatus;
        loop {
            match submitted.next().await {
                Some(Ok(TxStatus::InBestBlock(b))) => return Ok::<_, String>(b),
                Some(Ok(TxStatus::InFinalizedBlock(b))) => return Ok(b),
                Some(Ok(TxStatus::Error { message }))
                | Some(Ok(TxStatus::Invalid { message }))
                | Some(Ok(TxStatus::Dropped { message })) => {
                    return Err(format!("tx pool reported: {message}"));
                }
                Some(Ok(_)) => continue,
                Some(Err(e)) => return Err(format!("tx watch stream error: {e}")),
                None => return Err("tx watch stream closed unexpectedly".to_string()),
            }
        }
    })
    .await
    .map_err(|_| {
        "rotate_sfid_keys submit failed: timed out waiting for in-block inclusion".to_string()
    })?
    .map_err(|e| format!("rotate_sfid_keys submit failed: {e}"))?;
    in_block
        .wait_for_success()
        .await
        .map_err(|e| format!("rotate_sfid_keys included failed: {e}"))?;

    let block = client
        .blocks()
        .at(in_block.block_hash())
        .await
        .map_err(|e| format!("rotate_sfid_keys included failed: fetch block failed: {e}"))?;
    let block_number =
        block.number().to_string().parse::<u64>().map_err(|e| {
            format!("rotate_sfid_keys included failed: parse block number failed: {e}")
        })?;

    Ok(ChainRotateReceipt {
        tx_hash,
        block_number,
    })
}
