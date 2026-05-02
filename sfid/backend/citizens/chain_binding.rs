//! 公民绑定推链 helper。
//!
//! 当前由 admin 后台 `operate/binding.rs::citizen_push_chain_{bind,unbind}` handler 调用。
//!
//! 推链方式严格遵守 SFID PoW 链三件套(`feedback_sfid_pow_chain_recipe`):
//!   ① 显式 nonce(legacy RPC `system_account_next_index`)
//!   ② extrinsic immortal(避免 era 依赖 finalize)
//!   ③ 只等 `InBestBlock`(PoW finalize 显著落后,等不起)
//!
//! ADR:`memory/04-decisions/sfid/2026-04-07-subxt-0.43-pow-chain-quirks.md`。

use sp_core::Pair;
use subxt::{
    config::substrate::{AccountId32, MultiSignature},
    dynamic::{tx, Value},
    OnlineClient, PolkadotConfig,
};

use crate::app_core::chain_runtime::RuntimeBindCredential;
use crate::app_core::chain_url::chain_ws_url;

/// 提交 `SfidSystem::bind_sfid(credential)` extrinsic,返回 tx hash。
pub(crate) async fn submit_bind_sfid_extrinsic(
    credential: &RuntimeBindCredential,
    province_pair: &sp_core::sr25519::Pair,
) -> Result<String, String> {
    let ws_url = chain_ws_url()?;
    let client = OnlineClient::<PolkadotConfig>::from_insecure_url(&ws_url)
        .await
        .map_err(|e| format!("chain ws connect failed: {e}"))?;
    let rpc_client = subxt::backend::rpc::RpcClient::from_insecure_url(ws_url.as_str())
        .await
        .map_err(|e| format!("legacy rpc connect failed: {e}"))?;
    let legacy_rpc = subxt::backend::legacy::LegacyRpcMethods::<PolkadotConfig>::new(rpc_client);

    let signer_account = AccountId32(province_pair.public().0);
    let chain_nonce = legacy_rpc
        .system_account_next_index(&signer_account)
        .await
        .map_err(|e| format!("fetch nonce failed: {e}"))?;

    let binding_id_bytes =
        hex::decode(&credential.binding_id).map_err(|e| format!("binding_id hex decode: {e}"))?;
    let nonce_bytes = credential.bind_nonce.as_bytes().to_vec();
    let sig_bytes =
        hex::decode(&credential.signature).map_err(|e| format!("signature hex decode: {e}"))?;

    let payload = tx(
        "SfidSystem",
        "bind_sfid",
        vec![Value::named_composite([
            ("binding_id", Value::from_bytes(binding_id_bytes)),
            ("bind_nonce", Value::from_bytes(nonce_bytes)),
            ("signature", Value::from_bytes(sig_bytes)),
        ])],
    );

    let params = subxt::config::DefaultExtrinsicParamsBuilder::<PolkadotConfig>::new()
        .immortal()
        .nonce(chain_nonce)
        .build();
    let mut partial_tx = client
        .tx()
        .create_partial(&payload, &signer_account, params)
        .await
        .map_err(|e| format!("build extrinsic failed: {e}"))?;
    let signature = province_pair.sign(&partial_tx.signer_payload()).0;
    let extrinsic = partial_tx
        .sign_with_account_and_signature(&signer_account, &MultiSignature::Sr25519(signature));
    let tx_hash = format!("0x{}", hex::encode(extrinsic.hash().as_ref()));

    let mut submitted = extrinsic
        .submit_and_watch()
        .await
        .map_err(|e| format!("submit_and_watch failed: {e}"))?;
    tokio::time::timeout(std::time::Duration::from_secs(120), async {
        use subxt::tx::TxStatus;
        loop {
            match submitted.next().await {
                Some(Ok(TxStatus::InBestBlock(b))) => {
                    b.wait_for_success()
                        .await
                        .map_err(|e| format!("dispatch failed: {e}"))?;
                    return Ok::<_, String>(());
                }
                Some(Ok(TxStatus::InFinalizedBlock(b))) => {
                    b.wait_for_success()
                        .await
                        .map_err(|e| format!("dispatch failed: {e}"))?;
                    return Ok(());
                }
                Some(Ok(TxStatus::Error { message }))
                | Some(Ok(TxStatus::Invalid { message }))
                | Some(Ok(TxStatus::Dropped { message })) => {
                    return Err(format!("tx pool: {message}"));
                }
                Some(Ok(_)) => continue,
                Some(Err(e)) => return Err(format!("tx watch error: {e}")),
                None => return Err("tx watch stream closed".to_string()),
            }
        }
    })
    .await
    .map_err(|_| "timed out waiting for in-block inclusion".to_string())??;

    Ok(tx_hash)
}

/// 提交 `SfidSystem::unbind_sfid(target)` extrinsic,返回 tx hash。
pub(crate) async fn submit_unbind_sfid_extrinsic(
    target_pubkey_hex: &str,
    province_pair: &sp_core::sr25519::Pair,
) -> Result<String, String> {
    let target_bytes = hex::decode(target_pubkey_hex.trim_start_matches("0x"))
        .map_err(|e| format!("target pubkey hex decode: {e}"))?;
    if target_bytes.len() != 32 {
        return Err("target pubkey must be 32 bytes".to_string());
    }
    let mut target_arr = [0u8; 32];
    target_arr.copy_from_slice(&target_bytes);

    let ws_url = chain_ws_url()?;
    let client = OnlineClient::<PolkadotConfig>::from_insecure_url(&ws_url)
        .await
        .map_err(|e| format!("chain ws connect failed: {e}"))?;
    let rpc_client = subxt::backend::rpc::RpcClient::from_insecure_url(ws_url.as_str())
        .await
        .map_err(|e| format!("legacy rpc connect failed: {e}"))?;
    let legacy_rpc = subxt::backend::legacy::LegacyRpcMethods::<PolkadotConfig>::new(rpc_client);

    let signer_account = AccountId32(province_pair.public().0);
    let chain_nonce = legacy_rpc
        .system_account_next_index(&signer_account)
        .await
        .map_err(|e| format!("fetch nonce failed: {e}"))?;

    // unbind_sfid(target: AccountId) — call_index 1
    let payload = tx(
        "SfidSystem",
        "unbind_sfid",
        vec![Value::from_bytes(target_arr.to_vec())],
    );

    let params = subxt::config::DefaultExtrinsicParamsBuilder::<PolkadotConfig>::new()
        .immortal()
        .nonce(chain_nonce)
        .build();
    let mut partial_tx = client
        .tx()
        .create_partial(&payload, &signer_account, params)
        .await
        .map_err(|e| format!("build extrinsic failed: {e}"))?;
    let signature = province_pair.sign(&partial_tx.signer_payload()).0;
    let extrinsic = partial_tx
        .sign_with_account_and_signature(&signer_account, &MultiSignature::Sr25519(signature));
    let tx_hash = format!("0x{}", hex::encode(extrinsic.hash().as_ref()));

    let mut submitted = extrinsic
        .submit_and_watch()
        .await
        .map_err(|e| format!("submit_and_watch failed: {e}"))?;
    tokio::time::timeout(std::time::Duration::from_secs(120), async {
        use subxt::tx::TxStatus;
        loop {
            match submitted.next().await {
                Some(Ok(TxStatus::InBestBlock(b))) => {
                    b.wait_for_success()
                        .await
                        .map_err(|e| format!("dispatch failed: {e}"))?;
                    return Ok::<_, String>(());
                }
                Some(Ok(TxStatus::InFinalizedBlock(b))) => {
                    b.wait_for_success()
                        .await
                        .map_err(|e| format!("dispatch failed: {e}"))?;
                    return Ok(());
                }
                Some(Ok(TxStatus::Error { message }))
                | Some(Ok(TxStatus::Invalid { message }))
                | Some(Ok(TxStatus::Dropped { message })) => {
                    return Err(format!("tx pool: {message}"));
                }
                Some(Ok(_)) => continue,
                Some(Err(e)) => return Err(format!("tx watch error: {e}")),
                None => return Err("tx watch stream closed".to_string()),
            }
        }
    })
    .await
    .map_err(|_| "timed out waiting for in-block inclusion".to_string())??;

    Ok(tx_hash)
}
