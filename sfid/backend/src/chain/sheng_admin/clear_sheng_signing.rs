//! 清除链上某省 sheng signing pubkey。
//!
//! KEY_ADMIN 替换某省登录管理员时调用,失败仅 warn 不中断主流程
//! (链下 admin 替换已生效,链上残留旧 pubkey 不会卡住业务,新管理员
//! 首次登录 bootstrap 时会自动覆盖)。

use subxt::backend::legacy::LegacyRpcMethods;
use subxt::{OnlineClient, PolkadotConfig};

use crate::chain::key_admins::submit_set_sheng_signing_pubkey_with_client;
use crate::chain::url::chain_ws_url;
use crate::AppState;

/// 调本函数把某省 `ShengSigningPubkey[province]` 清成 None。
///
/// 任何步骤失败都会被吞掉(只 tracing::warn),让 admin 替换流程继续推进。
pub(crate) async fn clear_sheng_signing_pubkey_on_chain(state: &AppState, province: &str) {
    let ws_url = match chain_ws_url() {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(province, error = %e, "resolve ws url failed");
            return;
        }
    };
    let client = match OnlineClient::<PolkadotConfig>::from_insecure_url(ws_url.clone()).await {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(province, error = %e, "chain connect failed");
            return;
        }
    };
    let rpc_client = match subxt::backend::rpc::RpcClient::from_insecure_url(ws_url.as_str()).await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(province, error = %e, "legacy rpc connect failed");
            return;
        }
    };
    let legacy = LegacyRpcMethods::<PolkadotConfig>::new(rpc_client);
    let main_pair = state.sheng_signer_cache.sfid_main_signer();
    if let Err(e) =
        submit_set_sheng_signing_pubkey_with_client(&client, &legacy, &main_pair, province, None)
            .await
    {
        tracing::warn!(province, error = %e, "clear sheng signing pubkey on chain failed");
    }
}
