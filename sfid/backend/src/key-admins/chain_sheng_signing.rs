//! 中文注释：链上 `sfid_code_auth::set_sheng_signing_pubkey` 提交 helper。
//!
//! 对齐本仓 PoW 链推链三件套(feedback_sfid_pow_chain_recipe)：
//!   ① 显式 nonce(legacy RPC system_accountNextIndex)
//!   ② extrinsic immortal
//!   ③ 只等 InBestBlock
//!
//! 任务卡 `20260409-sfid-sheng-admin-per-province-keyring` Phase 1.B。

use sp_core::Pair;
use subxt::{
    config::substrate::{AccountId32, MultiSignature},
    dynamic::{tx, Value},
    OnlineClient, PolkadotConfig,
};

use crate::key_admins::sheng_signer_cache::ProvinceSigner;

/// 提交 `SfidCodeAuth::set_sheng_signing_pubkey(province, new_pubkey)`。
///
/// `new_pubkey = None` → 清除该省；`Some` → 写入。
///
/// 注意：本 helper 不从 AppState 拿任何东西；调用方必须先解析 ws_url 并构造
/// `OnlineClient` 和 `LegacyRpcMethods`，保持 PoW 链提交三件套一致。
#[allow(dead_code)]
pub(crate) async fn submit_set_sheng_signing_pubkey_with_client(
    client: &OnlineClient<PolkadotConfig>,
    legacy_rpc: &subxt::backend::legacy::LegacyRpcMethods<PolkadotConfig>,
    signer_pair: &ProvinceSigner,
    province: &str,
    new_pubkey: Option<[u8; 32]>,
) -> Result<String, String> {
    let signer_account = AccountId32(signer_pair.public().0);

    let province_val = Value::from_bytes(province.as_bytes().to_vec());
    let pubkey_val = match new_pubkey {
        Some(p) => Value::unnamed_variant("Some", vec![Value::from_bytes(p.to_vec())]),
        None => Value::unnamed_variant("None", Vec::<Value>::new()),
    };
    let payload = tx(
        "SfidCodeAuth",
        "set_sheng_signing_pubkey",
        vec![province_val, pubkey_val],
    );

    let chain_nonce = legacy_rpc
        .system_account_next_index(&signer_account)
        .await
        .map_err(|e| format!("fetch account nonce failed: {e}"))?;
    let params = subxt::config::DefaultExtrinsicParamsBuilder::<PolkadotConfig>::new()
        .immortal()
        .nonce(chain_nonce)
        .build();

    let mut partial_tx = client
        .tx()
        .create_partial(&payload, &signer_account, params)
        .await
        .map_err(|e| format!("build extrinsic failed: {e}"))?;
    let signature = signer_pair.sign(&partial_tx.signer_payload()).0;
    let extrinsic = partial_tx
        .sign_with_account_and_signature(&signer_account, &MultiSignature::Sr25519(signature));
    let tx_hash = format!("0x{}", hex::encode(extrinsic.hash().as_ref()));

    let mut submitted = extrinsic
        .submit_and_watch()
        .await
        .map_err(|e| format!("submit_and_watch failed: {e}"))?;

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
    .map_err(|_| "timed out waiting for in-block inclusion".to_string())?
    .map_err(|e| format!("set_sheng_signing_pubkey submit failed: {e}"))?;

    in_block
        .wait_for_success()
        .await
        .map_err(|e| format!("set_sheng_signing_pubkey included failed: {e}"))?;

    Ok(tx_hash)
}
