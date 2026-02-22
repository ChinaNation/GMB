use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signer, SigningKey};
use hex::FromHex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    store: Arc<RwLock<Store>>,
    admin_user: String,
    admin_password: String,
    signing_key: SigningKey,
    key_id: String,
    key_version: String,
    key_alg: String,
    public_key_hex: String,
}

#[derive(Default)]
struct Store {
    next_seq: u64,
    pending_by_pubkey: HashMap<String, PendingRequest>,
    bindings_by_pubkey: HashMap<String, BindingRecord>,
    pubkey_by_archive_index: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
struct PendingRequest {
    seq: u64,
    account_pubkey: String,
    requested_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
struct BindingRecord {
    seq: u64,
    account_pubkey: String,
    archive_index: String,
    sfid_code: String,
    sfid_signature: String,
    bound_at: DateTime<Utc>,
    bound_by: String,
}

#[derive(Serialize)]
struct ApiResponse<T: Serialize> {
    code: u32,
    message: String,
    data: T,
}

#[derive(Serialize)]
struct ApiError {
    code: u32,
    message: String,
    trace_id: String,
}

#[derive(Serialize)]
struct HealthData {
    service: &'static str,
    status: &'static str,
    pending_count: usize,
    binding_count: usize,
}

#[derive(Deserialize)]
struct BindRequestInput {
    account_pubkey: String,
}

#[derive(Serialize)]
struct BindRequestOutput {
    account_pubkey: String,
    status: &'static str,
    message: &'static str,
}

#[derive(Deserialize)]
struct AdminQueryInput {
    account_pubkey: String,
}

#[derive(Serialize)]
struct AdminQueryOutput {
    account_pubkey: String,
    found_pending: bool,
    found_binding: bool,
    archive_index: Option<String>,
    sfid_code: Option<String>,
}

#[derive(Deserialize)]
struct AdminBindInput {
    account_pubkey: String,
    archive_index: String,
}

#[derive(Deserialize)]
struct AdminUnbindInput {
    account_pubkey: String,
}

#[derive(Deserialize)]
struct CitizensQuery {
    keyword: Option<String>,
}

#[derive(Serialize)]
struct CitizenRow {
    seq: u64,
    account_pubkey: String,
    archive_index: Option<String>,
    sfid_code: Option<String>,
    is_bound: bool,
}

#[derive(Serialize)]
struct AdminBindOutput {
    account_pubkey: String,
    archive_index: String,
    sfid_code: String,
    proof: SignatureEnvelope,
    status: &'static str,
    message: &'static str,
}

#[derive(Serialize)]
struct AdminAuthOutput {
    ok: bool,
    user: String,
}

#[derive(Deserialize)]
struct BindResultQuery {
    account_pubkey: String,
}

#[derive(Serialize)]
struct BindResultOutput {
    account_pubkey: String,
    is_bound: bool,
    sfid_code: Option<String>,
    sfid_signature: Option<String>,
    message: String,
}

#[derive(Deserialize)]
struct VoteVerifyInput {
    account_pubkey: String,
    proposal_id: Option<u64>,
    challenge: Option<String>,
}

#[derive(Serialize)]
struct VoteVerifyOutput {
    account_pubkey: String,
    is_bound: bool,
    has_vote_eligibility: bool,
    sfid_code: Option<String>,
    vote_token: Option<SignatureEnvelope>,
    message: String,
}

#[derive(Serialize)]
struct SignatureEnvelope {
    key_id: String,
    key_version: String,
    alg: String,
    payload: String,
    signature_hex: String,
}

#[derive(Serialize)]
struct PublicKeyOutput {
    key_id: String,
    key_version: String,
    alg: String,
    public_key_hex: String,
}

#[derive(Serialize)]
struct BindingPayload {
    kind: &'static str,
    version: &'static str,
    account_pubkey: String,
    archive_index: String,
    sfid_code: String,
    issued_at: i64,
}

#[derive(Serialize)]
struct VotePayload {
    kind: &'static str,
    version: &'static str,
    account_pubkey: String,
    sfid_code: String,
    proposal_id: Option<u64>,
    challenge: String,
    iat: i64,
    exp: i64,
    jti: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .compact()
        .init();

    let signing_key = load_signing_key();
    let public_key_hex = hex::encode(signing_key.verifying_key().to_bytes());
    let state = AppState {
        store: Arc::new(RwLock::new(Store::default())),
        admin_user: std::env::var("SFID_ADMIN_USER").unwrap_or_else(|_| "admin".to_string()),
        admin_password: std::env::var("SFID_ADMIN_PASSWORD")
            .unwrap_or_else(|_| "admin123".to_string()),
        signing_key,
        key_id: std::env::var("SFID_KEY_ID").unwrap_or_else(|_| "sfid-master-v1".to_string()),
        key_version: "v1".to_string(),
        key_alg: "ed25519".to_string(),
        public_key_hex,
    };
    seed_demo_record(&state);

    let app = Router::new()
        .route("/", get(root))
        .route("/api/v1/health", get(health))
        .route("/api/v1/attestor/public-key", get(attestor_public_key))
        .route("/api/v1/admin/auth/check", get(admin_auth_check))
        .route("/api/v1/bind/request", post(create_bind_request))
        .route("/api/v1/admin/citizens", get(admin_list_citizens))
        .route("/api/v1/admin/bind/query", get(admin_query_by_pubkey))
        .route("/api/v1/admin/bind/confirm", post(admin_bind_confirm))
        .route("/api/v1/admin/bind/unbind", post(admin_unbind))
        .route("/api/v1/bind/result", get(get_bind_result))
        .route("/api/v1/vote/verify", post(verify_vote_eligibility))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8899));
    info!("sfid-backend listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind sfid backend listener");
    axum::serve(listener, app)
        .await
        .expect("run sfid backend server");
}

async fn root() -> impl IntoResponse {
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: "sfid backend is running",
    })
}

async fn health(State(state): State<AppState>) -> impl IntoResponse {
    let store = state.store.read().expect("store read lock poisoned");
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: HealthData {
            service: "sfid-backend",
            status: "UP",
            pending_count: store.pending_by_pubkey.len(),
            binding_count: store.bindings_by_pubkey.len(),
        },
    })
}

async fn attestor_public_key(State(state): State<AppState>) -> impl IntoResponse {
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: PublicKeyOutput {
            key_id: state.key_id.clone(),
            key_version: state.key_version.clone(),
            alg: state.key_alg.clone(),
            public_key_hex: state.public_key_hex.clone(),
        },
    })
}

async fn admin_auth_check(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let user = match admin_auth(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AdminAuthOutput { ok: true, user },
    })
    .into_response()
}

async fn create_bind_request(
    State(state): State<AppState>,
    Json(input): Json<BindRequestInput>,
) -> impl IntoResponse {
    if input.account_pubkey.trim().is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "account_pubkey is required");
    }

    let mut store = state.store.write().expect("store write lock poisoned");
    let seq = if let Some(existing) = store.pending_by_pubkey.get(&input.account_pubkey) {
        existing.seq
    } else if let Some(existing) = store.bindings_by_pubkey.get(&input.account_pubkey) {
        existing.seq
    } else {
        store.next_seq += 1;
        store.next_seq
    };
    store.pending_by_pubkey.insert(
        input.account_pubkey.clone(),
        PendingRequest {
            seq,
            account_pubkey: input.account_pubkey.clone(),
            requested_at: Utc::now(),
        },
    );

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: BindRequestOutput {
            account_pubkey: input.account_pubkey,
            status: "WAITING_ADMIN",
            message: "binding request received",
        },
    })
    .into_response()
}

async fn admin_list_citizens(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<CitizensQuery>,
) -> impl IntoResponse {
    if let Err(resp) = admin_auth(&state, &headers) {
        return resp;
    }

    let keyword = query
        .keyword
        .unwrap_or_default()
        .trim()
        .to_lowercase();

    let store = state.store.read().expect("store read lock poisoned");
    let mut rows: Vec<CitizenRow> = Vec::new();

    for pending in store.pending_by_pubkey.values() {
        if store.bindings_by_pubkey.contains_key(&pending.account_pubkey) {
            continue;
        }
        rows.push(CitizenRow {
            seq: pending.seq,
            account_pubkey: pending.account_pubkey.clone(),
            archive_index: None,
            sfid_code: None,
            is_bound: false,
        });
    }

    for b in store.bindings_by_pubkey.values() {
        rows.push(CitizenRow {
            seq: b.seq,
            account_pubkey: b.account_pubkey.clone(),
            archive_index: Some(b.archive_index.clone()),
            sfid_code: Some(b.sfid_code.clone()),
            is_bound: true,
        });
    }

    rows.sort_by_key(|r| r.seq);

    if !keyword.is_empty() {
        rows.retain(|r| {
            r.account_pubkey.to_lowercase().contains(&keyword)
                || r.archive_index
                    .as_ref()
                    .map(|v| v.to_lowercase().contains(&keyword))
                    .unwrap_or(false)
                || r.sfid_code
                    .as_ref()
                    .map(|v| v.to_lowercase().contains(&keyword))
                    .unwrap_or(false)
        });
    }

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: rows,
    })
    .into_response()
}

async fn admin_query_by_pubkey(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(input): Query<AdminQueryInput>,
) -> impl IntoResponse {
    if let Err(resp) = admin_auth(&state, &headers) {
        return resp;
    }

    if input.account_pubkey.trim().is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "account_pubkey is required");
    }

    let store = state.store.read().expect("store read lock poisoned");
    let pending = store.pending_by_pubkey.get(&input.account_pubkey).is_some();
    let binding = store.bindings_by_pubkey.get(&input.account_pubkey);

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AdminQueryOutput {
            account_pubkey: input.account_pubkey,
            found_pending: pending,
            found_binding: binding.is_some(),
            archive_index: binding.map(|b| b.archive_index.clone()),
            sfid_code: binding.map(|b| b.sfid_code.clone()),
        },
    })
    .into_response()
}

async fn admin_bind_confirm(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<AdminBindInput>,
) -> impl IntoResponse {
    let admin_user = match admin_auth(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    if input.account_pubkey.trim().is_empty() || input.archive_index.trim().is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "invalid request params");
    }

    let mut store = state.store.write().expect("store write lock poisoned");
    if let Some(bound_pubkey) = store.pubkey_by_archive_index.get(&input.archive_index) {
        if bound_pubkey != &input.account_pubkey {
            return api_error(StatusCode::CONFLICT, 3001, "archive_index already bound");
        }
    }
    if let Some(existing) = store.bindings_by_pubkey.get(&input.account_pubkey) {
        if existing.archive_index != input.archive_index {
            return api_error(StatusCode::CONFLICT, 3002, "pubkey already bound to another archive_index");
        }
        let payload = BindingPayload {
            kind: "bind",
            version: "v1",
            account_pubkey: existing.account_pubkey.clone(),
            archive_index: existing.archive_index.clone(),
            sfid_code: existing.sfid_code.clone(),
            issued_at: existing.bound_at.timestamp(),
        };
        let proof = make_signature_envelope(&state, &payload);
        return Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: AdminBindOutput {
                account_pubkey: existing.account_pubkey.clone(),
                archive_index: existing.archive_index.clone(),
                sfid_code: existing.sfid_code.clone(),
                proof,
                status: "BOUND",
                message: "already bound",
            },
        })
        .into_response();
    }

    let sfid_code = deterministic_sfid_code(&state, &input.archive_index, &input.account_pubkey);
    let seq = store
        .pending_by_pubkey
        .get(&input.account_pubkey)
        .map(|p| p.seq)
        .unwrap_or_else(|| {
            store.next_seq += 1;
            store.next_seq
        });
    let bound_at = Utc::now();
    let binding_payload = BindingPayload {
        kind: "bind",
        version: "v1",
        account_pubkey: input.account_pubkey.clone(),
        archive_index: input.archive_index.clone(),
        sfid_code: sfid_code.clone(),
        issued_at: bound_at.timestamp(),
    };
    let proof = make_signature_envelope(&state, &binding_payload);
    let binding = BindingRecord {
        seq,
        account_pubkey: input.account_pubkey.clone(),
        archive_index: input.archive_index.clone(),
        sfid_code: sfid_code.clone(),
        sfid_signature: proof.signature_hex.clone(),
        bound_at,
        bound_by: admin_user,
    };

    store
        .pubkey_by_archive_index
        .insert(input.archive_index.clone(), input.account_pubkey.clone());
    store
        .bindings_by_pubkey
        .insert(input.account_pubkey.clone(), binding);
    store.pending_by_pubkey.remove(&input.account_pubkey);

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AdminBindOutput {
            account_pubkey: input.account_pubkey,
            archive_index: input.archive_index,
            sfid_code,
            proof,
            status: "BOUND",
            message: "bind success",
        },
    })
    .into_response()
}

async fn admin_unbind(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<AdminUnbindInput>,
) -> impl IntoResponse {
    if let Err(resp) = admin_auth(&state, &headers) {
        return resp;
    }
    if input.account_pubkey.trim().is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "account_pubkey is required");
    }
    let mut store = state.store.write().expect("store write lock poisoned");
    let Some(binding) = store.bindings_by_pubkey.remove(&input.account_pubkey) else {
        return api_error(StatusCode::NOT_FOUND, 3005, "binding not found");
    };
    store.pubkey_by_archive_index.remove(&binding.archive_index);
    store.pending_by_pubkey.remove(&input.account_pubkey);
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: "unbind success and citizen removed",
    })
    .into_response()
}

async fn get_bind_result(
    State(state): State<AppState>,
    Query(query): Query<BindResultQuery>,
) -> impl IntoResponse {
    if query.account_pubkey.trim().is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "account_pubkey is required");
    }

    let store = state.store.read().expect("store read lock poisoned");
    if let Some(binding) = store.bindings_by_pubkey.get(&query.account_pubkey) {
        Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: BindResultOutput {
                account_pubkey: query.account_pubkey,
                is_bound: true,
                sfid_code: Some(binding.sfid_code.clone()),
                sfid_signature: Some(binding.sfid_signature.clone()),
                message: "sfid bind success".to_string(),
            },
        })
        .into_response()
    } else {
        Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: BindResultOutput {
                account_pubkey: query.account_pubkey,
                is_bound: false,
                sfid_code: None,
                sfid_signature: None,
                message: "not bound yet".to_string(),
            },
        })
        .into_response()
    }
}

async fn verify_vote_eligibility(
    State(state): State<AppState>,
    Json(input): Json<VoteVerifyInput>,
) -> impl IntoResponse {
    if input.account_pubkey.trim().is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "account_pubkey is required");
    }

    let store = state.store.read().expect("store read lock poisoned");
    if let Some(binding) = store.bindings_by_pubkey.get(&input.account_pubkey) {
        let iat = Utc::now().timestamp();
        let challenge = input
            .challenge
            .clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let vote_payload = VotePayload {
            kind: "vote",
            version: "v1",
            account_pubkey: input.account_pubkey.clone(),
            sfid_code: binding.sfid_code.clone(),
            proposal_id: input.proposal_id,
            challenge,
            iat,
            exp: iat + 60,
            jti: Uuid::new_v4().to_string(),
        };
        let vote_token = make_signature_envelope(&state, &vote_payload);
        Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: VoteVerifyOutput {
                account_pubkey: input.account_pubkey,
                is_bound: true,
                has_vote_eligibility: true,
                sfid_code: Some(binding.sfid_code.clone()),
                vote_token: Some(vote_token),
                message: "pubkey bound and vote eligible".to_string(),
            },
        })
        .into_response()
    } else {
        Json(ApiResponse {
            code: 0,
            message: "ok".to_string(),
            data: VoteVerifyOutput {
                account_pubkey: input.account_pubkey,
                is_bound: false,
                has_vote_eligibility: false,
                sfid_code: None,
                vote_token: None,
                message: "pubkey not bound, no vote eligibility".to_string(),
            },
        })
        .into_response()
    }
}

fn admin_auth(state: &AppState, headers: &HeaderMap) -> Result<String, axum::response::Response> {
    let user = headers
        .get("x-admin-user")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .trim()
        .to_string();
    let password = headers
        .get("x-admin-password")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .trim()
        .to_string();

    if user.is_empty() || password.is_empty() {
        return Err(api_error(
            StatusCode::UNAUTHORIZED,
            2001,
            "admin auth required",
        ));
    }
    if user != state.admin_user || password != state.admin_password {
        return Err(api_error(StatusCode::FORBIDDEN, 2002, "admin auth invalid"));
    }
    Ok(user)
}

fn api_error(status: StatusCode, code: u32, message: &str) -> axum::response::Response {
    (
        status,
        Json(ApiError {
            code,
            message: message.to_string(),
            trace_id: Uuid::new_v4().to_string(),
        }),
    )
        .into_response()
}

fn seed_demo_record(state: &AppState) {
    let mut store = state.store.write().expect("store write lock poisoned");
    if !store.bindings_by_pubkey.is_empty() || !store.pending_by_pubkey.is_empty() {
        return;
    }
    let total = 50_u64;
    let bound_total = 30_u64;
    let now = Utc::now();

    for seq in 1..=total {
        let pubkey = format!("0xDEMO_PUBKEY_{seq:04}");
        if seq <= bound_total {
            let archive = format!("CIV-DEMO-{seq:04}");
            let sfid = deterministic_sfid_code(state, &archive, &pubkey);
            let binding_payload = BindingPayload {
                kind: "bind",
                version: "v1",
                account_pubkey: pubkey.clone(),
                archive_index: archive.clone(),
                sfid_code: sfid.clone(),
                issued_at: now.timestamp(),
            };
            let proof = make_signature_envelope(state, &binding_payload);
            store.bindings_by_pubkey.insert(
                pubkey.clone(),
                BindingRecord {
                    seq,
                    account_pubkey: pubkey.clone(),
                    archive_index: archive.clone(),
                    sfid_code: sfid,
                    sfid_signature: proof.signature_hex,
                    bound_at: now,
                    bound_by: "system-seed".to_string(),
                },
            );
            store.pubkey_by_archive_index.insert(archive, pubkey);
        } else {
            store.pending_by_pubkey.insert(
                pubkey.clone(),
                PendingRequest {
                    seq,
                    account_pubkey: pubkey,
                    requested_at: now,
                },
            );
        }
    }
    store.next_seq = total;
}

fn deterministic_sfid_code(state: &AppState, archive_index: &str, account_pubkey: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"sfid-code-v1|");
    hasher.update(state.signing_key.to_bytes());
    hasher.update(b"|");
    hasher.update(archive_index.as_bytes());
    hasher.update(b"|");
    hasher.update(account_pubkey.as_bytes());
    let digest = hasher.finalize();

    let core = hex::encode_upper(&digest[..12]);
    let checksum = digest
        .iter()
        .fold(0_u32, |acc, b| (acc + u32::from(*b)) % 10_u32);
    format!("SFID-{core}{checksum}")
}

fn make_signature_envelope<T: Serialize>(state: &AppState, payload: &T) -> SignatureEnvelope {
    let payload_text = serde_json::to_string(payload).expect("serialize payload");
    let signature = state.signing_key.sign(payload_text.as_bytes());
    SignatureEnvelope {
        key_id: state.key_id.clone(),
        key_version: state.key_version.clone(),
        alg: state.key_alg.clone(),
        payload: payload_text,
        signature_hex: hex::encode(signature.to_bytes()),
    }
}

fn load_signing_key() -> SigningKey {
    let raw = std::env::var("SFID_SIGNING_SEED_HEX").unwrap_or_else(|_| {
        // Dev fallback seed. Replace with secure offline-generated seed in production.
        "sfid-dev-master-seed-v1".to_string()
    });
    let seed = decode_seed_to_32(raw);
    SigningKey::from_bytes(&seed)
}

fn decode_seed_to_32(raw: String) -> [u8; 32] {
    let trimmed = raw.trim();
    if trimmed.len() == 64 {
        if let Ok(bytes) = Vec::from_hex(trimmed) {
            let mut out = [0_u8; 32];
            out.copy_from_slice(&bytes[..32]);
            return out;
        }
    }

    let mut hasher = Sha256::new();
    hasher.update(trimmed.as_bytes());
    let digest = hasher.finalize();
    let mut out = [0_u8; 32];
    out.copy_from_slice(&digest[..32]);
    out
}
