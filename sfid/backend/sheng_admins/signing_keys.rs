//! 中文注释:省管理员本人签名密钥的自动加载与手动生成/更换接口。
//!
//! 这里处理的是 SFID 后端本地签名 seed 的生命周期,不是管理员名册。
//! 主管理员只能管理备用管理员账户,不能调用本接口替备用管理员生成私钥。

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sp_core::{sr25519, Pair};
use zeroize::Zeroizing;

use crate::crypto::pubkey::same_admin_pubkey;
use crate::login::require_sheng_admin;
use crate::sheng_admins::signing_cache::{ShengSigningCache, Sr25519Pair};
use crate::sheng_admins::signing_seed_store::{load_seed, save_seed};
use crate::AppState;

const PAYLOAD_PREFIX: &[u8; 12] = b"GMB_ACTIVATE";
const SHENFEN_ID_LEN: usize = 48;
const TIMESTAMP_LEN: usize = 8;
const NONCE_LEN: usize = 16;
const PAYLOAD_LEN: usize = PAYLOAD_PREFIX.len() + SHENFEN_ID_LEN + TIMESTAMP_LEN + NONCE_LEN;
const TTL_SECONDS: i64 = 300;

/// 签名密钥 bootstrap / replace 失败原因。
#[derive(Debug)]
pub(crate) enum BootstrapError {
    Rng(String),
    Persist(String),
    Load(String),
}

impl std::fmt::Display for BootstrapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BootstrapError::Rng(s) => write!(f, "rng error: {s}"),
            BootstrapError::Persist(s) => write!(f, "persist error: {s}"),
            BootstrapError::Load(s) => write!(f, "load error: {s}"),
        }
    }
}

impl std::error::Error for BootstrapError {}

/// 确保 (province, admin_pubkey) 的签名 Pair 已就绪并载入 cache。
///
/// 中文注释:登录 bootstrap 和手动“生成”共用该函数。若加密 seed 已存在,
/// 只解密加载;若不存在,才随机生成并落盘。
pub(crate) fn ensure_signing_keypair(
    cache: &ShengSigningCache,
    province: &str,
    admin_pubkey: &[u8; 32],
) -> Result<Sr25519Pair, BootstrapError> {
    if let Some(existing) = cache.get(province, admin_pubkey) {
        return Ok(existing);
    }

    let seed = match load_seed(province, admin_pubkey).map_err(BootstrapError::Load)? {
        Some(s) => s,
        None => {
            let mut seed_arr: Zeroizing<[u8; 32]> = Zeroizing::new([0u8; 32]);
            getrandom::getrandom(seed_arr.as_mut_slice())
                .map_err(|e| BootstrapError::Rng(e.to_string()))?;
            save_seed(province, admin_pubkey, &seed_arr).map_err(BootstrapError::Persist)?;
            *seed_arr
        }
    };

    let pair = sr25519::Pair::from_seed(&seed);
    cache.load(province.to_string(), *admin_pubkey, pair.clone());
    Ok(pair)
}

/// 手动更换 (province, admin_pubkey) 的签名 Pair。
///
/// 中文注释:调用方必须先完成登录态和本人扫码签名校验。本函数只覆盖当前
/// 管理员自己的本地加密 seed,不会替其它管理员生成或更换私钥。
pub(crate) fn replace_signing_keypair(
    cache: &ShengSigningCache,
    province: &str,
    admin_pubkey: &[u8; 32],
) -> Result<Sr25519Pair, BootstrapError> {
    let mut seed_arr: Zeroizing<[u8; 32]> = Zeroizing::new([0u8; 32]);
    getrandom::getrandom(seed_arr.as_mut_slice())
        .map_err(|e| BootstrapError::Rng(e.to_string()))?;
    save_seed(province, admin_pubkey, &seed_arr).map_err(BootstrapError::Persist)?;

    let pair = sr25519::Pair::from_seed(&seed_arr);
    cache.load(province.to_string(), *admin_pubkey, pair.clone());
    Ok(pair)
}

/// 从 sr25519 Pair 渲染 0x 小写签名公钥。
pub(crate) fn pair_signing_pubkey_hex(pair: &Sr25519Pair) -> String {
    format!("0x{}", hex::encode(pair.public().0))
}

/// 把签名密钥的可展示元数据写回 legacy store 与 GlobalShard。
///
/// 中文注释:签名 seed 真私钥只保存在 `signing_seed_store.rs` 的加密文件中,
/// 本函数只写回页面展示需要的签名公钥和生成时间。
pub(crate) fn record_signing_metadata(
    state: &AppState,
    admin_pubkey_hex: &str,
    signing_pubkey_hex: &str,
    generated_at: DateTime<Utc>,
    replace_created_at: bool,
) {
    if let Ok(mut store) = state.store.write() {
        if let Some((_key, user)) = store
            .admin_users_by_pubkey
            .iter_mut()
            .find(|(key, _)| same_admin_pubkey(key.as_str(), admin_pubkey_hex))
        {
            user.signing_pubkey = Some(signing_pubkey_hex.to_string());
            if replace_created_at || user.signing_created_at.is_none() {
                user.signing_created_at = Some(generated_at);
            }
            user.updated_at = Some(generated_at);
        }
    }

    let sharded_store = state.sharded_store.clone();
    let admin_key = admin_pubkey_hex.to_string();
    let signing_pubkey = signing_pubkey_hex.to_string();
    tokio::task::spawn(async move {
        let result = sharded_store
            .write_global(|global| {
                if let Some((_key, user)) = global
                    .global_admins
                    .iter_mut()
                    .find(|(key, _)| same_admin_pubkey(key.as_str(), admin_key.as_str()))
                {
                    user.signing_pubkey = Some(signing_pubkey.clone());
                    if replace_created_at || user.signing_created_at.is_none() {
                        user.signing_created_at = Some(generated_at);
                    }
                    user.updated_at = Some(generated_at);
                }
            })
            .await;
        if let Err(err) = result {
            tracing::warn!(error = %err, "record sheng signing metadata to global shard failed");
        }
    });
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum SigningOperation {
    Generate,
    Replace,
}

impl SigningOperation {
    fn marker(self, province: &str) -> String {
        let op = match self {
            SigningOperation::Generate => "GEN",
            SigningOperation::Replace => "REP",
        };
        format!("SHENG:{op}:{province}")
    }

    fn display_summary(self, province: &str) -> String {
        match self {
            SigningOperation::Generate => format!("生成{province}省管理员签名密钥"),
            SigningOperation::Replace => format!("更换{province}省管理员签名密钥"),
        }
    }

    fn result_label(self) -> &'static str {
        match self {
            SigningOperation::Generate => "GENERATED",
            SigningOperation::Replace => "REPLACED",
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct PrepareInput {
    pub(crate) operation: SigningOperation,
}

#[derive(Debug, Serialize)]
pub(crate) struct DisplayField {
    pub(crate) key: &'static str,
    pub(crate) label: &'static str,
    pub(crate) value: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct PrepareOutput {
    pub(crate) operation: SigningOperation,
    pub(crate) request_id: String,
    pub(crate) payload_hex: String,
    pub(crate) expires_at: i64,
    pub(crate) display_action: &'static str,
    pub(crate) display_summary: String,
    pub(crate) display_fields: Vec<DisplayField>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SubmitInput {
    pub(crate) operation: SigningOperation,
    pub(crate) payload_hex: String,
    pub(crate) signature: String,
    #[serde(default)]
    pub(crate) signer_pubkey: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct SubmitOutput {
    pub(crate) ok: bool,
    pub(crate) operation_result: &'static str,
    pub(crate) signing_pubkey: String,
    pub(crate) signing_created_at: String,
}

/// `POST /api/v1/admin/sheng-signer/prepare`
///
/// 中文注释:生成给钱包弹窗签名的 payload。为复用现有 wumin 解码器,这里使用
/// 已支持的 `GMB_ACTIVATE` 84 字节结构,display.action 固定为 `activate_admin`。
pub(crate) async fn prepare(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<PrepareInput>,
) -> impl IntoResponse {
    let ctx = match require_sheng_admin(&state, &headers) {
        Ok(ctx) => ctx,
        Err(resp) => return resp,
    };
    let Some(province) = ctx.admin_province.clone() else {
        return crate::api_error(StatusCode::FORBIDDEN, 1003, "admin province scope missing");
    };

    let marker = input.operation.marker(province.as_str());
    let marker_bytes = marker.as_bytes();
    if marker_bytes.len() > SHENFEN_ID_LEN {
        return crate::api_error(StatusCode::BAD_REQUEST, 1001, "signing marker too long");
    }

    let now = Utc::now().timestamp();
    let mut nonce = [0u8; NONCE_LEN];
    if let Err(err) = getrandom::getrandom(&mut nonce) {
        tracing::warn!(error = %err, "generate sheng signer nonce failed");
        return crate::api_error(StatusCode::INTERNAL_SERVER_ERROR, 5001, "rng failed");
    }

    let mut payload = Vec::with_capacity(PAYLOAD_LEN);
    payload.extend_from_slice(PAYLOAD_PREFIX);
    let mut marker_padded = [0u8; SHENFEN_ID_LEN];
    marker_padded[..marker_bytes.len()].copy_from_slice(marker_bytes);
    payload.extend_from_slice(&marker_padded);
    payload.extend_from_slice(&(now as u64).to_le_bytes());
    payload.extend_from_slice(&nonce);

    Json(crate::ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: PrepareOutput {
            operation: input.operation,
            request_id: format!("sheng-signer-{}", uuid::Uuid::new_v4()),
            payload_hex: format!("0x{}", hex::encode(payload)),
            expires_at: now + TTL_SECONDS,
            display_action: "activate_admin",
            display_summary: input.operation.display_summary(province.as_str()),
            display_fields: vec![DisplayField {
                key: "shenfen_id",
                label: "省管理员操作",
                value: marker,
            }],
        },
    })
    .into_response()
}

/// `POST /api/v1/admin/sheng-signer/submit`
///
/// 中文注释:校验当前管理员本人签名后,才生成或更换本人的签名 seed。
pub(crate) async fn submit(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<SubmitInput>,
) -> impl IntoResponse {
    let ctx = match require_sheng_admin(&state, &headers) {
        Ok(ctx) => ctx,
        Err(resp) => return resp,
    };
    let Some(province) = ctx.admin_province.clone() else {
        return crate::api_error(StatusCode::FORBIDDEN, 1003, "admin province scope missing");
    };
    let Some(admin_pubkey) = crate::login::parse_sr25519_pubkey_bytes(ctx.admin_pubkey.as_str())
    else {
        return crate::api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "admin_pubkey must be 0x + 64 hex",
        );
    };

    if let Some(signer_pubkey) = input.signer_pubkey.as_deref() {
        if !same_admin_pubkey(signer_pubkey, ctx.admin_pubkey.as_str()) {
            return crate::api_error(
                StatusCode::FORBIDDEN,
                1003,
                "签名账户必须是当前登录省管理员本人",
            );
        }
    }

    let payload = match parse_payload_hex(input.payload_hex.as_str()) {
        Ok(v) => v,
        Err(msg) => return crate::api_error(StatusCode::BAD_REQUEST, 1001, msg.as_str()),
    };
    let marker = input.operation.marker(province.as_str());
    if let Err(msg) = validate_payload(&payload, marker.as_str()) {
        return crate::api_error(StatusCode::BAD_REQUEST, 1001, msg.as_str());
    }
    let nonce_key = format!(
        "sheng-signer:{}:{}",
        input.operation.result_label(),
        hex::encode(&payload[PAYLOAD_LEN - NONCE_LEN..])
    );
    {
        let mut store = match crate::store_write_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        store
            .chain_nonce_seen
            .retain(|_, seen_at| *seen_at + chrono::Duration::seconds(TTL_SECONDS) > Utc::now());
        if store.chain_nonce_seen.contains_key(&nonce_key) {
            return crate::api_error(StatusCode::CONFLICT, 1005, "签名请求已使用");
        }
        store.chain_nonce_seen.insert(nonce_key, Utc::now());
    }

    if !verify_payload_signature(&admin_pubkey, input.signature.as_str(), &payload) {
        return crate::api_error(StatusCode::FORBIDDEN, 1003, "签名校验失败");
    }

    let pair = match input.operation {
        SigningOperation::Generate => ensure_signing_keypair(
            state.sheng_admin_signing_cache.as_ref(),
            province.as_str(),
            &admin_pubkey,
        ),
        SigningOperation::Replace => replace_signing_keypair(
            state.sheng_admin_signing_cache.as_ref(),
            province.as_str(),
            &admin_pubkey,
        ),
    };
    let pair = match pair {
        Ok(v) => v,
        Err(err) => {
            tracing::warn!(province = %province, error = %err, "sheng signer manual operation failed");
            return crate::api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "签名密钥生成失败,请检查 SFID_MASTER_KEK_HEX 或存储目录权限",
            );
        }
    };

    let generated_at = Utc::now();
    let signing_pubkey = pair_signing_pubkey_hex(&pair);
    record_signing_metadata(
        &state,
        ctx.admin_pubkey.as_str(),
        signing_pubkey.as_str(),
        generated_at,
        input.operation == SigningOperation::Replace,
    );

    Json(crate::ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: SubmitOutput {
            ok: true,
            operation_result: input.operation.result_label(),
            signing_pubkey,
            signing_created_at: generated_at.to_rfc3339(),
        },
    })
    .into_response()
}

fn parse_payload_hex(payload_hex: &str) -> Result<Vec<u8>, String> {
    let trimmed = payload_hex.trim();
    let raw = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .ok_or_else(|| "payload_hex must start with 0x".to_string())?;
    let bytes = hex::decode(raw).map_err(|_| "payload_hex must be valid hex".to_string())?;
    if bytes.len() != PAYLOAD_LEN {
        return Err("payload length invalid".to_string());
    }
    Ok(bytes)
}

fn validate_payload(payload: &[u8], expected_marker: &str) -> Result<(), String> {
    if payload.len() != PAYLOAD_LEN || &payload[..PAYLOAD_PREFIX.len()] != PAYLOAD_PREFIX {
        return Err("payload prefix invalid".to_string());
    }
    let marker_bytes = &payload[PAYLOAD_PREFIX.len()..PAYLOAD_PREFIX.len() + SHENFEN_ID_LEN];
    let marker_end = marker_bytes
        .iter()
        .position(|b| *b == 0)
        .unwrap_or(marker_bytes.len());
    let marker = std::str::from_utf8(&marker_bytes[..marker_end])
        .map_err(|_| "payload marker invalid utf8".to_string())?;
    if marker != expected_marker {
        return Err("payload marker mismatch".to_string());
    }
    let ts_start = PAYLOAD_PREFIX.len() + SHENFEN_ID_LEN;
    let timestamp = u64::from_le_bytes(
        payload[ts_start..ts_start + TIMESTAMP_LEN]
            .try_into()
            .map_err(|_| "payload timestamp invalid".to_string())?,
    ) as i64;
    let now = Utc::now().timestamp();
    if timestamp > now + 30 || timestamp + TTL_SECONDS < now {
        return Err("签名请求已过期".to_string());
    }
    Ok(())
}

fn verify_payload_signature(admin_pubkey: &[u8; 32], signature_hex: &str, payload: &[u8]) -> bool {
    let raw_sig = signature_hex
        .trim()
        .strip_prefix("0x")
        .or_else(|| signature_hex.trim().strip_prefix("0X"))
        .unwrap_or_else(|| signature_hex.trim());
    let sig_vec = match hex::decode(raw_sig) {
        Ok(v) if v.len() == 64 => v,
        _ => return false,
    };
    let sig_arr: [u8; 64] = match sig_vec.as_slice().try_into() {
        Ok(v) => v,
        Err(_) => return false,
    };
    let signature = sr25519::Signature::from_raw(sig_arr);
    let public = sr25519::Public::from_raw(*admin_pubkey);
    sr25519::Pair::verify(&signature, payload, &public)
}
