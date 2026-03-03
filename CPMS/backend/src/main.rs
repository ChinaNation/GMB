use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    fs,
    hash::{Hash, Hasher},
    net::SocketAddr,
    path::{Path as FsPath, PathBuf},
    sync::Arc,
};

use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    routing::{get, post, put},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::{Duration, NaiveDate, Utc};
use schnorrkel::{signing_context, MiniSecretKey, PublicKey, Signature};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

mod initialize;
mod province_codes;
use initialize::{
    bind_super_admin as persist_super_admin_binding, init_super_admin_users,
    initialize_install_data_from_sfid_qr, load_or_init_install_data,
    super_admin_user_id_for_key_id, QrSignKeyRuntime,
};

const TOKEN_EXPIRES_SECONDS: i64 = 30 * 60;
const CHALLENGE_EXPIRES_SECONDS: i64 = 90;
const ARCHIVE_NO_MAX_RETRY: u32 = 20;
const QR_EXPIRES_SECONDS: i64 = 24 * 60 * 60;

#[derive(Clone)]
struct AppState {
    runtime_store_path: PathBuf,
    install: Arc<RwLock<InstallRuntime>>,
    admin_users: Arc<RwLock<HashMap<String, AdminUser>>>,
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    login_challenges: Arc<RwLock<HashMap<String, LoginChallenge>>>,
    qr_login_results: Arc<RwLock<HashMap<String, QrLoginResult>>>,
    archives: Arc<RwLock<HashMap<String, Archive>>>,
    sequence: Arc<RwLock<HashMap<String, u32>>>,
    qr_print_records: Arc<RwLock<Vec<QrPrintRecord>>>,
    audit_logs: Arc<RwLock<Vec<AuditLog>>>,
}

#[derive(Clone)]
struct InstallRuntime {
    file_path: PathBuf,
    site_sfid: Option<String>,
    qr_sign_keys: Vec<QrSignKeyRuntime>,
}

#[derive(Clone, Serialize, Deserialize)]
struct AdminUser {
    user_id: String,
    admin_pubkey: String,
    role: String,
    status: String,
    immutable: bool,
}

#[derive(Clone, Serialize, Deserialize)]
struct Session {
    user_id: String,
    role: String,
    expires_at: i64,
}

#[derive(Clone, Serialize, Deserialize)]
struct LoginChallenge {
    admin_pubkey: String,
    challenge_payload: String,
    session_id: String,
    expire_at: i64,
    consumed: bool,
}

#[derive(Clone, Serialize, Deserialize)]
struct QrLoginResult {
    session_id: String,
    access_token: String,
    expires_in: i64,
    user: SessionUser,
    created_at: i64,
}

#[derive(Clone, Serialize, Deserialize)]
struct Archive {
    archive_id: String,
    archive_no: String,
    province_code: String,
    city_code: String,
    full_name: String,
    birth_date: String,
    gender_code: String,
    height_cm: Option<f32>,
    passport_no: String,
    status: String,
    citizen_status: String,
}

#[derive(Clone)]
struct AuthContext {
    user_id: String,
    role: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct AuditLog {
    log_id: String,
    operator_user_id: Option<String>,
    action: String,
    target_type: String,
    target_id: Option<String>,
    result: String,
    detail: serde_json::Value,
    created_at: i64,
}

#[derive(Serialize)]
struct ApiResponse<T>
where
    T: Serialize,
{
    code: i32,
    message: String,
    data: Option<T>,
}

#[derive(Serialize)]
struct ApiError {
    code: i32,
    message: String,
    trace_id: String,
}

#[derive(Deserialize)]
struct IdentifyRequest {
    admin_pubkey: String,
}

#[derive(Serialize)]
struct IdentifyData {
    user_id: String,
    role: String,
    status: String,
}

#[derive(Deserialize)]
struct ChallengeRequest {
    admin_pubkey: String,
}

#[derive(Serialize)]
struct ChallengeData {
    challenge_id: String,
    challenge_payload: String,
    nonce: String,
    expire_at: i64,
}

#[derive(Deserialize)]
struct VerifyRequest {
    challenge_id: String,
    admin_pubkey: String,
    signature: String,
}

#[derive(Serialize)]
struct VerifyData {
    access_token: String,
    expires_in: i64,
    user: SessionUser,
}

#[derive(Deserialize)]
struct QrChallengeRequest {
    origin: Option<String>,
    domain: Option<String>,
    session_id: Option<String>,
}

#[derive(Serialize)]
struct QrChallengeData {
    challenge_id: String,
    challenge_payload: String,
    login_qr_payload: String,
    origin: String,
    domain: String,
    session_id: String,
    nonce: String,
    expire_at: i64,
}

#[derive(Deserialize)]
struct QrCompleteRequest {
    challenge_id: String,
    session_id: String,
    admin_pubkey: String,
    signature: String,
}

#[derive(Deserialize)]
struct QrResultQuery {
    challenge_id: String,
    session_id: String,
}

#[derive(Serialize)]
struct QrResultData {
    status: String,
    message: String,
    access_token: Option<String>,
    expires_in: Option<i64>,
    user: Option<SessionUser>,
}

#[derive(Clone, Serialize, Deserialize)]
struct SessionUser {
    user_id: String,
    role: String,
}

#[derive(Deserialize)]
struct CreateOperatorRequest {
    admin_pubkey: String,
}

#[derive(Deserialize)]
struct UpdateOperatorRequest {
    admin_pubkey: Option<String>,
    status: Option<String>,
}

#[derive(Deserialize)]
struct UpdateOperatorStatusRequest {
    status: String,
}

#[derive(Serialize)]
struct OperatorData {
    user_id: String,
    admin_pubkey: String,
    role: String,
    status: String,
}

#[derive(Deserialize)]
struct CreateArchiveRequest {
    province_code: String,
    city_code: String,
    full_name: String,
    birth_date: String,
    gender_code: String,
    height_cm: Option<f32>,
    passport_no: String,
    citizen_status: Option<String>,
}

#[derive(Serialize)]
struct CreateArchiveData {
    archive_id: String,
    archive_no: String,
    status: String,
    citizen_status: String,
}

#[derive(Deserialize)]
struct UpdateCitizenStatusRequest {
    citizen_status: String,
}

#[derive(Serialize)]
struct UpdateCitizenStatusData {
    archive_id: String,
    archive_no: String,
    citizen_status: String,
    voting_eligible: bool,
}

#[derive(Serialize)]
struct QrGenerateData {
    qr_payload: QrPayload,
    qr_content: String,
}

#[derive(Serialize)]
struct QrPrintData {
    print_id: String,
    archive_id: String,
    archive_no: String,
    citizen_status: String,
    voting_eligible: bool,
    printed_at: i64,
}

#[derive(Serialize)]
struct QrPayload {
    ver: String,
    issuer_id: String,
    site_sfid: String,
    sign_key_id: String,
    archive_no: String,
    citizen_status: String,
    voting_eligible: bool,
    issued_at: i64,
    expire_at: i64,
    qr_id: String,
    sig_alg: String,
    signature: String,
}

#[derive(Serialize)]
struct SiteKeyRegistrationData {
    qr_payload: SiteKeyRegistrationPayload,
    qr_content: String,
}

#[derive(Serialize)]
struct SiteKeyRegistrationPayload {
    ver: String,
    qr_type: String,
    issuer_id: String,
    site_sfid: String,
    keys: Vec<SiteKeyPublicItem>,
    issued_at: i64,
    qr_id: String,
    sig_alg: String,
    sign_key_id: String,
    signature: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct SiteKeyPublicItem {
    key_id: String,
    purpose: String,
    status: String,
    pubkey: String,
}

#[derive(Deserialize)]
struct InstallInitializeRequest {
    sfid_init_qr_content: String,
}

#[derive(Serialize)]
struct InstallInitializeData {
    site_sfid: String,
    super_admin_bind_qrs: Vec<SuperAdminBindQrData>,
}

#[derive(Serialize)]
struct InstallStatusData {
    initialized: bool,
    site_sfid: Option<String>,
    super_admin_bound_count: usize,
    super_admin_bind_qrs: Vec<SuperAdminBindQrData>,
}

#[derive(Clone, Serialize, Deserialize)]
struct SuperAdminBindQrData {
    key_id: String,
    bound: bool,
    qr_payload: SuperAdminBindQrPayload,
    qr_content: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct SuperAdminBindQrPayload {
    ver: String,
    qr_type: String,
    issuer_id: String,
    site_sfid: String,
    sign_key_id: String,
    sign_key_pubkey: String,
    bind_nonce: String,
    issued_at: i64,
}

#[derive(Deserialize)]
struct BindSuperAdminRequest {
    key_id: String,
    admin_pubkey: String,
    bind_nonce: String,
    signature: String,
}

#[derive(Serialize)]
struct BindSuperAdminData {
    user_id: String,
    admin_pubkey: String,
    role: String,
    status: String,
    managed_key_id: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct QrPrintRecord {
    print_id: String,
    archive_id: String,
    archive_no: String,
    citizen_status: String,
    voting_eligible: bool,
    printed_at: i64,
}

#[derive(Deserialize)]
struct ListQuery {
    full_name: Option<String>,
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
struct RuntimeStore {
    admin_users: HashMap<String, AdminUser>,
    sessions: HashMap<String, Session>,
    login_challenges: HashMap<String, LoginChallenge>,
    qr_login_results: HashMap<String, QrLoginResult>,
    archives: HashMap<String, Archive>,
    sequence: HashMap<String, u32>,
    qr_print_records: Vec<QrPrintRecord>,
    audit_logs: Vec<AuditLog>,
}

#[tokio::main]
async fn main() {
    let install = load_or_init_install_data().unwrap_or_else(|reason| panic!("{reason}"));
    if install.was_created {
        println!(
            "cpms-backend install bootstrap initialized at {}",
            install.file_path.display()
        );
    }
    if install.data.is_none() {
        println!("cpms-backend waiting for SFID install qr initialization");
    }

    let super_admins = install
        .data
        .as_ref()
        .map(|d| d.super_admins.clone())
        .unwrap_or_default();
    let site_sfid = install.data.as_ref().map(|d| d.site_sfid.clone());
    let runtime_store_path = PathBuf::from(
        std::env::var("CPMS_RUNTIME_STORE_FILE")
            .unwrap_or_else(|_| "runtime/cpms_runtime_store.json".to_string()),
    );
    let mut runtime_store = match load_runtime_store(&runtime_store_path) {
        Ok(store) => store,
        Err(reason) => {
            eprintln!("failed to load runtime store: {reason}");
            RuntimeStore::default()
        }
    };
    if runtime_store.admin_users.is_empty() {
        runtime_store.admin_users = init_super_admin_users(&super_admins);
    } else {
        for (k, v) in init_super_admin_users(&super_admins) {
            runtime_store.admin_users.entry(k).or_insert(v);
        }
    }

    let state = AppState {
        runtime_store_path,
        install: Arc::new(RwLock::new(InstallRuntime {
            file_path: install.file_path,
            site_sfid,
            qr_sign_keys: install.qr_sign_keys,
        })),
        admin_users: Arc::new(RwLock::new(runtime_store.admin_users)),
        sessions: Arc::new(RwLock::new(runtime_store.sessions)),
        login_challenges: Arc::new(RwLock::new(runtime_store.login_challenges)),
        qr_login_results: Arc::new(RwLock::new(runtime_store.qr_login_results)),
        archives: Arc::new(RwLock::new(runtime_store.archives)),
        sequence: Arc::new(RwLock::new(runtime_store.sequence)),
        qr_print_records: Arc::new(RwLock::new(runtime_store.qr_print_records)),
        audit_logs: Arc::new(RwLock::new(runtime_store.audit_logs)),
    };
    if let Err(reason) = persist_runtime_store(&state).await {
        eprintln!("failed to persist runtime store on startup: {reason}");
    }

    let app = Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/install/status", get(install_status))
        .route("/api/v1/install/initialize", post(initialize_install))
        .route(
            "/api/v1/install/super-admin/bind",
            post(bind_super_admin_from_wuminapp),
        )
        .route("/api/v1/admin/auth/identify", post(auth_identify))
        .route("/api/v1/admin/auth/challenge", post(auth_challenge))
        .route("/api/v1/admin/auth/verify", post(auth_verify))
        .route("/api/v1/admin/auth/qr/challenge", post(auth_qr_challenge))
        .route("/api/v1/admin/auth/qr/complete", post(auth_qr_complete))
        .route("/api/v1/admin/auth/qr/result", get(auth_qr_result))
        .route("/api/v1/admin/auth/logout", post(auth_logout))
        .route(
            "/api/v1/admin/operators",
            get(list_operators).post(create_operator),
        )
        .route(
            "/api/v1/admin/operators/:id",
            put(update_operator).delete(delete_operator),
        )
        .route(
            "/api/v1/admin/operators/:id/status",
            put(update_operator_status),
        )
        .route(
            "/api/v1/admin/site-keys/registration-qr",
            post(generate_site_key_registration_qr),
        )
        .route("/api/v1/archives", post(create_archive).get(list_archives))
        .route("/api/v1/archives/:archive_id", get(get_archive))
        .route(
            "/api/v1/archives/:archive_id/qr/generate",
            post(generate_archive_qr),
        )
        .route(
            "/api/v1/archives/:archive_id/citizen-status",
            put(update_archive_citizen_status),
        )
        .route(
            "/api/v1/archives/:archive_id/qr/print",
            post(print_archive_qr),
        )
        .with_state(state);

    let addr: SocketAddr = std::env::var("CPMS_BIND")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
        .parse()
        .expect("invalid CPMS_BIND");

    println!("cpms-backend listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind failed");
    axum::serve(listener, app).await.expect("server failed");
}

async fn health() -> Json<ApiResponse<serde_json::Value>> {
    Json(ok(serde_json::json!({"status": "ok"})))
}

async fn install_status(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<InstallStatusData>>, (StatusCode, Json<ApiError>)> {
    let (site_sfid, keys) = {
        let install = state.install.read().await;
        (install.site_sfid.clone(), install.qr_sign_keys.clone())
    };
    let users = state.admin_users.read().await;
    let super_admin_bound_count = users.values().filter(|u| u.role == "SUPER_ADMIN").count();
    let bind_qrs = build_super_admin_bind_qrs(site_sfid.clone(), &keys, &users)?;

    Ok(Json(ok(InstallStatusData {
        initialized: site_sfid.is_some() && !keys.is_empty(),
        site_sfid,
        super_admin_bound_count,
        super_admin_bind_qrs: bind_qrs,
    })))
}

async fn initialize_install(
    State(state): State<AppState>,
    Json(req): Json<InstallInitializeRequest>,
) -> Result<Json<ApiResponse<InstallInitializeData>>, (StatusCode, Json<ApiError>)> {
    if req.sfid_init_qr_content.trim().is_empty() {
        return Err(err(
            StatusCode::BAD_REQUEST,
            1001,
            "invalid sfid_init_qr_content",
        ));
    }

    {
        let install = state.install.read().await;
        if install.site_sfid.is_some() {
            return Err(err(
                StatusCode::CONFLICT,
                4001,
                "cpms is already initialized",
            ));
        }
    }

    let initialized = initialize_install_data_from_sfid_qr(&req.sfid_init_qr_content)
        .map_err(|reason| err(StatusCode::BAD_REQUEST, 4002, &reason))?;
    let data = initialized.data.ok_or_else(|| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "missing install data",
        )
    })?;

    {
        // 安装初始化只会写入 site_sfid 与 3 把签名密钥，不触碰业务数据。
        let mut install = state.install.write().await;
        install.file_path = initialized.file_path;
        install.site_sfid = Some(data.site_sfid.clone());
        install.qr_sign_keys = initialized.qr_sign_keys;
    }

    {
        let mut users = state.admin_users.write().await;
        *users = init_super_admin_users(&data.super_admins);
    }

    write_audit(
        &state,
        None,
        "INSTALL_INITIALIZE",
        "CPMS_INSTALL",
        Some(data.site_sfid.clone()),
        "SUCCESS",
        serde_json::json!({}),
    )
    .await?;

    let users = state.admin_users.read().await;
    let keys = {
        let install = state.install.read().await;
        install.qr_sign_keys.clone()
    };
    let bind_qrs = build_super_admin_bind_qrs(Some(data.site_sfid.clone()), &keys, &users)?;

    Ok(Json(ok(InstallInitializeData {
        site_sfid: data.site_sfid,
        super_admin_bind_qrs: bind_qrs,
    })))
}

async fn bind_super_admin_from_wuminapp(
    State(state): State<AppState>,
    Json(req): Json<BindSuperAdminRequest>,
) -> Result<Json<ApiResponse<BindSuperAdminData>>, (StatusCode, Json<ApiError>)> {
    if req.key_id.trim().is_empty()
        || req.admin_pubkey.trim().is_empty()
        || req.bind_nonce.trim().is_empty()
        || req.signature.trim().is_empty()
    {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "invalid bind request"));
    }

    let (site_sfid, file_path, keys) = {
        let install = state.install.read().await;
        (
            install.site_sfid.clone(),
            install.file_path.clone(),
            install.qr_sign_keys.clone(),
        )
    };
    let site_sfid =
        site_sfid.ok_or_else(|| err(StatusCode::CONFLICT, 4003, "cpms not initialized"))?;
    if !keys.iter().any(|k| k.key_id == req.key_id) {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "invalid key_id"));
    }

    let expected_nonce = super_admin_bind_nonce(
        &site_sfid,
        &req.key_id,
        keys.iter()
            .find(|k| k.key_id == req.key_id)
            .map(|k| k.pubkey.as_str())
            .unwrap_or(""),
    );
    if req.bind_nonce != expected_nonce {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "invalid bind_nonce"));
    }

    let bind_sign_source =
        super_admin_bind_sign_source(&site_sfid, &req.key_id, &req.admin_pubkey, &req.bind_nonce);
    verify_signature_with_context(
        &req.admin_pubkey,
        &bind_sign_source,
        &req.signature,
        b"CPMS-SUPER-ADMIN-BIND-V1",
    )
    .map_err(|reason| err(StatusCode::UNAUTHORIZED, 2002, reason))?;

    let created = persist_super_admin_binding(&file_path, &req.key_id, &req.admin_pubkey)
        .map_err(|reason| err(StatusCode::CONFLICT, 4004, &reason))?;

    let user = AdminUser {
        user_id: created.user_id.clone(),
        admin_pubkey: created.admin_pubkey.clone(),
        role: "SUPER_ADMIN".to_string(),
        status: "ACTIVE".to_string(),
        immutable: true,
    };
    state
        .admin_users
        .write()
        .await
        .insert(user.user_id.clone(), user.clone());

    write_audit(
        &state,
        Some(user.user_id.clone()),
        "BIND_SUPER_ADMIN",
        "ADMIN_USER",
        Some(user.user_id.clone()),
        "SUCCESS",
        serde_json::json!({
            "managed_key_id": created.managed_key_id,
        }),
    )
    .await?;

    Ok(Json(ok(BindSuperAdminData {
        user_id: user.user_id,
        admin_pubkey: user.admin_pubkey,
        role: user.role,
        status: user.status,
        managed_key_id: created.managed_key_id,
    })))
}

async fn auth_identify(
    State(state): State<AppState>,
    Json(req): Json<IdentifyRequest>,
) -> Result<Json<ApiResponse<IdentifyData>>, (StatusCode, Json<ApiError>)> {
    let admin = find_admin_by_pubkey(&state, &req.admin_pubkey).await?;
    if admin.status != "ACTIVE" {
        write_audit(
            &state,
            None,
            "AUTH_IDENTIFY",
            "ADMIN_USER",
            Some(admin.user_id.clone()),
            "FAILED",
            serde_json::json!({"reason": "inactive"}),
        )
        .await?;
        return Err(err(StatusCode::UNAUTHORIZED, 2002, "admin is not active"));
    }

    write_audit(
        &state,
        Some(admin.user_id.clone()),
        "AUTH_IDENTIFY",
        "ADMIN_USER",
        Some(admin.user_id.clone()),
        "SUCCESS",
        serde_json::json!({}),
    )
    .await?;

    Ok(Json(ok(IdentifyData {
        user_id: admin.user_id,
        role: admin.role,
        status: admin.status,
    })))
}

async fn auth_challenge(
    State(state): State<AppState>,
    Json(req): Json<ChallengeRequest>,
) -> Result<Json<ApiResponse<ChallengeData>>, (StatusCode, Json<ApiError>)> {
    let admin = find_admin_by_pubkey(&state, &req.admin_pubkey).await?;
    if admin.status != "ACTIVE" {
        return Err(err(StatusCode::UNAUTHORIZED, 2002, "admin is not active"));
    }

    let challenge_id = format!("chl_{}", Uuid::new_v4().simple());
    let nonce = Uuid::new_v4().simple().to_string();
    let expire_at = (Utc::now() + Duration::seconds(CHALLENGE_EXPIRES_SECONDS)).timestamp();
    let challenge_payload = format!(
        "cpms-admin-auth-v1|{}|{}|{}|{}",
        challenge_id, req.admin_pubkey, nonce, expire_at
    );

    let challenge = LoginChallenge {
        admin_pubkey: req.admin_pubkey,
        challenge_payload: challenge_payload.clone(),
        session_id: challenge_id.clone(),
        expire_at,
        consumed: false,
    };

    state
        .login_challenges
        .write()
        .await
        .insert(challenge_id.clone(), challenge);
    persist_runtime_store(&state)
        .await
        .map_err(|reason| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, &reason))?;

    write_audit(
        &state,
        Some(admin.user_id.clone()),
        "AUTH_CHALLENGE",
        "LOGIN_CHALLENGE",
        Some(challenge_id.clone()),
        "SUCCESS",
        serde_json::json!({"expire_at": expire_at}),
    )
    .await?;

    Ok(Json(ok(ChallengeData {
        challenge_id,
        challenge_payload,
        nonce,
        expire_at,
    })))
}

async fn auth_verify(
    State(state): State<AppState>,
    Json(req): Json<VerifyRequest>,
) -> Result<Json<ApiResponse<VerifyData>>, (StatusCode, Json<ApiError>)> {
    let admin = find_admin_by_pubkey(&state, &req.admin_pubkey).await?;
    if admin.status != "ACTIVE" {
        return Err(err(StatusCode::UNAUTHORIZED, 2002, "admin is not active"));
    }

    let now_ts = Utc::now().timestamp();
    let challenge_payload = {
        let mut challenges = state.login_challenges.write().await;
        let challenge = challenges
            .get_mut(&req.challenge_id)
            .ok_or_else(|| err(StatusCode::BAD_REQUEST, 2003, "challenge not found"))?;

        if challenge.admin_pubkey != req.admin_pubkey {
            return Err(err(
                StatusCode::BAD_REQUEST,
                2004,
                "challenge pubkey mismatch",
            ));
        }
        if challenge.consumed {
            return Err(err(
                StatusCode::BAD_REQUEST,
                2005,
                "challenge already consumed",
            ));
        }
        if challenge.expire_at < now_ts {
            return Err(err(StatusCode::BAD_REQUEST, 2006, "challenge expired"));
        }

        challenge.consumed = true;
        challenge.challenge_payload.clone()
    };

    if let Err(reason) =
        verify_challenge_signature(&req.admin_pubkey, &challenge_payload, &req.signature)
    {
        write_audit(
            &state,
            Some(admin.user_id.clone()),
            "AUTH_VERIFY",
            "LOGIN_CHALLENGE",
            Some(req.challenge_id.clone()),
            "FAILED",
            serde_json::json!({"reason": reason}),
        )
        .await?;
        return Err(err(
            StatusCode::UNAUTHORIZED,
            2007,
            "signature verify failed",
        ));
    }

    let access_token = format!("atk_{}", Uuid::new_v4().simple());
    let expires_at = (Utc::now() + Duration::seconds(TOKEN_EXPIRES_SECONDS)).timestamp();
    let session = Session {
        user_id: admin.user_id.clone(),
        role: admin.role.clone(),
        expires_at,
    };
    state
        .sessions
        .write()
        .await
        .insert(access_token.clone(), session);

    write_audit(
        &state,
        Some(admin.user_id.clone()),
        "AUTH_VERIFY",
        "SESSION",
        Some(access_token.clone()),
        "SUCCESS",
        serde_json::json!({"challenge_id": req.challenge_id}),
    )
    .await?;

    Ok(Json(ok(VerifyData {
        access_token,
        expires_in: TOKEN_EXPIRES_SECONDS,
        user: SessionUser {
            user_id: admin.user_id,
            role: admin.role,
        },
    })))
}

async fn auth_qr_challenge(
    State(state): State<AppState>,
    Json(req): Json<QrChallengeRequest>,
) -> Result<Json<ApiResponse<QrChallengeData>>, (StatusCode, Json<ApiError>)> {
    let origin = req.origin.unwrap_or_default().trim().to_string();
    if origin.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "origin is required"));
    }
    let session_id = req.session_id.unwrap_or_default().trim().to_string();
    if session_id.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "session_id is required"));
    }
    let domain = extract_domain_from_origin(&origin)
        .or(req.domain)
        .unwrap_or_default();
    if domain.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "domain is required"));
    }

    let challenge_id = format!("chl_{}", Uuid::new_v4().simple());
    let nonce = Uuid::new_v4().simple().to_string();
    let expire_at = (Utc::now() + Duration::seconds(CHALLENGE_EXPIRES_SECONDS)).timestamp();
    let challenge_payload = format!(
        "cpms-admin-qr-login-v1|{}|{}|{}|{}|{}|{}",
        challenge_id, origin, domain, session_id, nonce, expire_at
    );

    let challenge = LoginChallenge {
        admin_pubkey: String::new(),
        challenge_payload: challenge_payload.clone(),
        session_id: session_id.clone(),
        expire_at,
        consumed: false,
    };
    state
        .login_challenges
        .write()
        .await
        .insert(challenge_id.clone(), challenge);
    persist_runtime_store(&state)
        .await
        .map_err(|reason| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, &reason))?;

    let login_qr_payload = serde_json::json!({
        "ver": "1",
        "type": "CPMS_ADMIN_LOGIN",
        "challenge_id": challenge_id,
        "challenge_payload": challenge_payload,
        "session_id": session_id,
        "nonce": nonce,
        "origin": origin,
        "domain": domain,
        "expire_at": expire_at
    })
    .to_string();

    Ok(Json(ok(QrChallengeData {
        challenge_id,
        challenge_payload,
        login_qr_payload,
        origin,
        domain,
        session_id,
        nonce,
        expire_at,
    })))
}

async fn auth_qr_complete(
    State(state): State<AppState>,
    Json(req): Json<QrCompleteRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiError>)> {
    if req.challenge_id.trim().is_empty()
        || req.session_id.trim().is_empty()
        || req.admin_pubkey.trim().is_empty()
        || req.signature.trim().is_empty()
    {
        return Err(err(
            StatusCode::BAD_REQUEST,
            1001,
            "challenge_id, session_id, admin_pubkey, signature are required",
        ));
    }
    let now_ts = Utc::now().timestamp();
    let challenge_payload = {
        let mut challenges = state.login_challenges.write().await;
        let challenge = challenges
            .get_mut(req.challenge_id.trim())
            .ok_or_else(|| err(StatusCode::BAD_REQUEST, 2003, "challenge not found"))?;
        if challenge.consumed {
            return Err(err(
                StatusCode::BAD_REQUEST,
                2005,
                "challenge already consumed",
            ));
        }
        if challenge.expire_at < now_ts {
            return Err(err(StatusCode::BAD_REQUEST, 2006, "challenge expired"));
        }
        if challenge.session_id != req.session_id.trim() {
            return Err(err(
                StatusCode::BAD_REQUEST,
                2004,
                "challenge session mismatch",
            ));
        }
        challenge.consumed = true;
        challenge.admin_pubkey = req.admin_pubkey.trim().to_string();
        challenge.challenge_payload.clone()
    };

    let admin = find_admin_by_pubkey(&state, req.admin_pubkey.trim()).await?;
    if admin.status != "ACTIVE" {
        return Err(err(StatusCode::UNAUTHORIZED, 2002, "admin is not active"));
    }
    if verify_challenge_signature(
        req.admin_pubkey.trim(),
        &challenge_payload,
        req.signature.trim(),
    )
    .is_err()
    {
        return Err(err(
            StatusCode::UNAUTHORIZED,
            2007,
            "signature verify failed",
        ));
    }

    let access_token = format!("atk_{}", Uuid::new_v4().simple());
    let expires_at = (Utc::now() + Duration::seconds(TOKEN_EXPIRES_SECONDS)).timestamp();
    let session = Session {
        user_id: admin.user_id.clone(),
        role: admin.role.clone(),
        expires_at,
    };
    state
        .sessions
        .write()
        .await
        .insert(access_token.clone(), session);
    state.qr_login_results.write().await.insert(
        req.challenge_id.clone(),
        QrLoginResult {
            session_id: req.session_id.trim().to_string(),
            access_token,
            expires_in: TOKEN_EXPIRES_SECONDS,
            user: SessionUser {
                user_id: admin.user_id,
                role: admin.role,
            },
            created_at: now_ts,
        },
    );
    persist_runtime_store(&state)
        .await
        .map_err(|reason| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, &reason))?;

    Ok(Json(ok(serde_json::json!({"status": "SUCCESS"}))))
}

async fn auth_qr_result(
    State(state): State<AppState>,
    Query(query): Query<QrResultQuery>,
) -> Result<Json<ApiResponse<QrResultData>>, (StatusCode, Json<ApiError>)> {
    if query.challenge_id.trim().is_empty() || query.session_id.trim().is_empty() {
        return Err(err(
            StatusCode::BAD_REQUEST,
            1001,
            "challenge_id and session_id are required",
        ));
    }
    let now_ts = Utc::now().timestamp();
    let changed = {
        let mut results = state.qr_login_results.write().await;
        let before = results.len();
        results.retain(|_, v| v.created_at + 3600 > now_ts);
        before != results.len()
    };
    if changed {
        persist_runtime_store(&state)
            .await
            .map_err(|reason| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, &reason))?;
    }

    if let Some(result) = state
        .qr_login_results
        .read()
        .await
        .get(query.challenge_id.trim())
        .cloned()
    {
        if result.session_id != query.session_id.trim() {
            return Err(err(
                StatusCode::BAD_REQUEST,
                2004,
                "challenge session mismatch",
            ));
        }
        return Ok(Json(ok(QrResultData {
            status: "SUCCESS".to_string(),
            message: "login success".to_string(),
            access_token: Some(result.access_token),
            expires_in: Some(result.expires_in),
            user: Some(result.user),
        })));
    }

    let challenge_map = state.login_challenges.read().await;
    let Some(challenge) = challenge_map.get(query.challenge_id.trim()) else {
        return Err(err(StatusCode::BAD_REQUEST, 2003, "challenge not found"));
    };
    if challenge.session_id != query.session_id.trim() {
        return Err(err(
            StatusCode::BAD_REQUEST,
            2004,
            "challenge session mismatch",
        ));
    }
    if challenge.expire_at < now_ts {
        return Ok(Json(ok(QrResultData {
            status: "EXPIRED".to_string(),
            message: "challenge expired".to_string(),
            access_token: None,
            expires_in: None,
            user: None,
        })));
    }

    Ok(Json(ok(QrResultData {
        status: "PENDING".to_string(),
        message: "waiting mobile scan".to_string(),
        access_token: None,
        expires_in: None,
        user: None,
    })))
}

async fn auth_logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiError>)> {
    let token = bearer_token(&headers)?;
    let removed = state.sessions.write().await.remove(&token);
    if removed.is_none() {
        return Err(err(StatusCode::UNAUTHORIZED, 2001, "invalid token"));
    }
    persist_runtime_store(&state)
        .await
        .map_err(|reason| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, &reason))?;

    Ok(Json(ok(serde_json::json!({}))))
}

async fn list_operators(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Vec<OperatorData>>>, (StatusCode, Json<ApiError>)> {
    require_role(&state, &headers, "SUPER_ADMIN").await?;

    let users = state.admin_users.read().await;
    let operators = users
        .values()
        .filter(|u| u.role == "OPERATOR_ADMIN")
        .map(|u| OperatorData {
            user_id: u.user_id.clone(),
            admin_pubkey: u.admin_pubkey.clone(),
            role: u.role.clone(),
            status: u.status.clone(),
        })
        .collect::<Vec<OperatorData>>();

    Ok(Json(ok(operators)))
}

async fn create_operator(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateOperatorRequest>,
) -> Result<Json<ApiResponse<OperatorData>>, (StatusCode, Json<ApiError>)> {
    let ctx = require_role(&state, &headers, "SUPER_ADMIN").await?;
    if req.admin_pubkey.trim().is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "invalid admin_pubkey"));
    }

    if find_admin_by_pubkey(&state, &req.admin_pubkey)
        .await
        .is_ok()
    {
        return Err(err(
            StatusCode::CONFLICT,
            3001,
            "admin_pubkey already exists",
        ));
    }

    let operator = AdminUser {
        user_id: format!("u_operator_{}", Uuid::new_v4().simple()),
        admin_pubkey: req.admin_pubkey,
        role: "OPERATOR_ADMIN".to_string(),
        status: "ACTIVE".to_string(),
        immutable: false,
    };
    state
        .admin_users
        .write()
        .await
        .insert(operator.user_id.clone(), operator.clone());

    write_audit(
        &state,
        Some(ctx.user_id),
        "CREATE_OPERATOR",
        "ADMIN_USER",
        Some(operator.user_id.clone()),
        "SUCCESS",
        serde_json::json!({"role": operator.role}),
    )
    .await?;

    Ok(Json(ok(OperatorData {
        user_id: operator.user_id,
        admin_pubkey: operator.admin_pubkey,
        role: operator.role,
        status: operator.status,
    })))
}

async fn update_operator(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<UpdateOperatorRequest>,
) -> Result<Json<ApiResponse<OperatorData>>, (StatusCode, Json<ApiError>)> {
    let ctx = require_role(&state, &headers, "SUPER_ADMIN").await?;
    let mut users = state.admin_users.write().await;
    let current = users
        .get(&id)
        .cloned()
        .ok_or_else(|| err(StatusCode::NOT_FOUND, 3002, "operator not found"))?;
    if current.role != "OPERATOR_ADMIN" {
        return Err(err(
            StatusCode::BAD_REQUEST,
            3003,
            "target is not operator admin",
        ));
    }

    if let Some(ref admin_pubkey) = req.admin_pubkey {
        if admin_pubkey.trim().is_empty() {
            return Err(err(StatusCode::BAD_REQUEST, 1001, "invalid admin_pubkey"));
        }
        let duplicated = users
            .values()
            .any(|u| u.user_id != current.user_id && u.admin_pubkey == *admin_pubkey);
        if duplicated {
            return Err(err(
                StatusCode::CONFLICT,
                3001,
                "admin_pubkey already exists",
            ));
        }
    }

    let operator = users
        .get_mut(&id)
        .ok_or_else(|| err(StatusCode::NOT_FOUND, 3002, "operator not found"))?;

    if let Some(admin_pubkey) = req.admin_pubkey {
        operator.admin_pubkey = admin_pubkey;
    }

    if let Some(status) = req.status {
        validate_admin_status(&status)?;
        operator.status = status;
    }

    let updated = operator.clone();
    drop(users);

    write_audit(
        &state,
        Some(ctx.user_id),
        "UPDATE_OPERATOR",
        "ADMIN_USER",
        Some(updated.user_id.clone()),
        "SUCCESS",
        serde_json::json!({"status": updated.status}),
    )
    .await?;

    Ok(Json(ok(OperatorData {
        user_id: updated.user_id,
        admin_pubkey: updated.admin_pubkey,
        role: updated.role,
        status: updated.status,
    })))
}

async fn delete_operator(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiError>)> {
    let ctx = require_role(&state, &headers, "SUPER_ADMIN").await?;
    let mut users = state.admin_users.write().await;
    let user = users
        .get(&id)
        .ok_or_else(|| err(StatusCode::NOT_FOUND, 3002, "operator not found"))?;
    if user.role != "OPERATOR_ADMIN" {
        return Err(err(
            StatusCode::BAD_REQUEST,
            3003,
            "target is not operator admin",
        ));
    }
    users.remove(&id);
    drop(users);

    write_audit(
        &state,
        Some(ctx.user_id),
        "DELETE_OPERATOR",
        "ADMIN_USER",
        Some(id),
        "SUCCESS",
        serde_json::json!({}),
    )
    .await?;

    Ok(Json(ok(serde_json::json!({}))))
}

async fn update_operator_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<UpdateOperatorStatusRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiError>)> {
    let ctx = require_role(&state, &headers, "SUPER_ADMIN").await?;
    validate_admin_status(&req.status)?;

    let mut users = state.admin_users.write().await;
    let operator = users
        .get_mut(&id)
        .ok_or_else(|| err(StatusCode::NOT_FOUND, 3002, "operator not found"))?;
    if operator.role != "OPERATOR_ADMIN" {
        return Err(err(
            StatusCode::BAD_REQUEST,
            3003,
            "target is not operator admin",
        ));
    }

    operator.status = req.status.clone();
    drop(users);

    write_audit(
        &state,
        Some(ctx.user_id),
        "UPDATE_OPERATOR_STATUS",
        "ADMIN_USER",
        Some(id),
        "SUCCESS",
        serde_json::json!({"status": req.status}),
    )
    .await?;

    Ok(Json(ok(serde_json::json!({}))))
}

async fn generate_site_key_registration_qr(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<SiteKeyRegistrationData>>, (StatusCode, Json<ApiError>)> {
    let ctx = require_role(&state, &headers, "SUPER_ADMIN").await?;
    let payload = build_site_key_registration_payload(&state).await?;
    let qr_content = serde_json::to_string(&payload)
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "qr encode failed"))?;

    write_audit(
        &state,
        Some(ctx.user_id),
        "GENERATE_SITE_KEY_REGISTRATION_QR",
        "SITE_KEY_QR",
        Some(payload.qr_id.clone()),
        "SUCCESS",
        serde_json::json!({
            "site_sfid": payload.site_sfid,
            "sign_key_id": payload.sign_key_id
        }),
    )
    .await?;

    Ok(Json(ok(SiteKeyRegistrationData {
        qr_payload: payload,
        qr_content,
    })))
}

async fn create_archive(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateArchiveRequest>,
) -> Result<Json<ApiResponse<CreateArchiveData>>, (StatusCode, Json<ApiError>)> {
    let ctx = require_admin_access(&state, &headers).await?;
    let admin = find_admin_by_user_id(&state, &ctx.user_id).await?;

    if !province_codes::is_valid_province_code(&req.province_code) {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "invalid province_code"));
    }
    if !province_codes::is_valid_city_code_for_province(&req.province_code, &req.city_code) {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "invalid city_code"));
    }
    if req.gender_code != "M" && req.gender_code != "W" {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "invalid gender_code"));
    }
    let _birth_date = NaiveDate::parse_from_str(&req.birth_date, "%Y-%m-%d")
        .map_err(|_| err(StatusCode::BAD_REQUEST, 1001, "invalid birth_date"))?;
    let terminal_id = headers
        .get("x-terminal-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("terminal-000");
    let citizen_status = req.citizen_status.unwrap_or_else(|| "NORMAL".to_string());
    validate_citizen_status(&citizen_status)?;
    // 日期码按“档案号创建时间”写入，确保档案号与状态解耦且保持稳定。
    let created_date_yyyymmdd = Utc::now().format("%Y%m%d").to_string();

    let archive_no = generate_archive_no_with_retry(
        &state,
        &req.province_code,
        &req.city_code,
        &created_date_yyyymmdd,
        terminal_id,
        &admin.admin_pubkey,
    )
    .await?;

    let archive = Archive {
        archive_id: format!("ar_{}", Uuid::new_v4().simple()),
        archive_no: archive_no.clone(),
        province_code: req.province_code,
        city_code: req.city_code,
        full_name: req.full_name,
        birth_date: req.birth_date,
        gender_code: req.gender_code,
        height_cm: req.height_cm,
        passport_no: req.passport_no,
        status: "ACTIVE".to_string(),
        citizen_status,
    };

    state
        .archives
        .write()
        .await
        .insert(archive.archive_id.clone(), archive.clone());

    write_audit(
        &state,
        Some(ctx.user_id),
        "CREATE_ARCHIVE",
        "CITIZEN_ARCHIVE",
        Some(archive.archive_id.clone()),
        "SUCCESS",
        serde_json::json!({}),
    )
    .await?;

    Ok(Json(ok(CreateArchiveData {
        archive_id: archive.archive_id,
        archive_no,
        status: archive.status,
        citizen_status: archive.citizen_status,
    })))
}

async fn update_archive_citizen_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(archive_id): Path<String>,
    Json(req): Json<UpdateCitizenStatusRequest>,
) -> Result<Json<ApiResponse<UpdateCitizenStatusData>>, (StatusCode, Json<ApiError>)> {
    let ctx = require_role(&state, &headers, "SUPER_ADMIN").await?;
    validate_citizen_status(&req.citizen_status)?;

    // 状态更新只影响二维码业务字段，不回写 archive_no。
    let mut archives = state.archives.write().await;
    let archive = archives
        .get_mut(&archive_id)
        .ok_or_else(|| err(StatusCode::NOT_FOUND, 3004, "archive not found"))?;
    let before = archive.citizen_status.clone();
    archive.citizen_status = req.citizen_status.clone();
    let updated = archive.clone();
    drop(archives);

    write_audit(
        &state,
        Some(ctx.user_id),
        "UPDATE_ARCHIVE_CITIZEN_STATUS",
        "CITIZEN_ARCHIVE",
        Some(updated.archive_id.clone()),
        "SUCCESS",
        serde_json::json!({
            "archive_no": updated.archive_no,
            "before_citizen_status": before,
            "after_citizen_status": updated.citizen_status,
            "voting_eligible": updated.citizen_status == "NORMAL"
        }),
    )
    .await?;

    Ok(Json(ok(UpdateCitizenStatusData {
        archive_id: updated.archive_id,
        archive_no: updated.archive_no,
        citizen_status: updated.citizen_status.clone(),
        voting_eligible: updated.citizen_status == "NORMAL",
    })))
}

async fn list_archives(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiError>)> {
    require_admin_access(&state, &headers).await?;

    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);

    let archives = state.archives.read().await;
    let mut items: Vec<Archive> = archives.values().cloned().collect();

    if let Some(name) = query.full_name {
        items.retain(|a| a.full_name.contains(&name));
    }

    items.sort_by(|a, b| a.archive_id.cmp(&b.archive_id));
    let total = items.len();
    let start = (page - 1) * page_size;
    let end = (start + page_size).min(total);
    let page_items = if start >= total {
        vec![]
    } else {
        items[start..end].to_vec()
    };

    Ok(Json(ok(serde_json::json!({
        "items": page_items,
        "page": page,
        "page_size": page_size,
        "total": total
    }))))
}

async fn get_archive(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(archive_id): Path<String>,
) -> Result<Json<ApiResponse<Archive>>, (StatusCode, Json<ApiError>)> {
    require_admin_access(&state, &headers).await?;

    let archives = state.archives.read().await;
    let archive = archives
        .get(&archive_id)
        .ok_or_else(|| err(StatusCode::NOT_FOUND, 3004, "archive not found"))?
        .clone();

    Ok(Json(ok(archive)))
}

async fn generate_archive_qr(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(archive_id): Path<String>,
) -> Result<Json<ApiResponse<QrGenerateData>>, (StatusCode, Json<ApiError>)> {
    let ctx = require_admin_access(&state, &headers).await?;
    let archive = {
        let archives = state.archives.read().await;
        archives
            .get(&archive_id)
            .cloned()
            .ok_or_else(|| err(StatusCode::NOT_FOUND, 3004, "archive not found"))?
    };
    let qr_payload = build_qr_payload(&state, &archive).await?;
    let qr_content = serde_json::to_string(&qr_payload)
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "qr encode failed"))?;

    write_audit(
        &state,
        Some(ctx.user_id),
        "GENERATE_ARCHIVE_QR",
        "QR",
        Some(qr_payload.qr_id.clone()),
        "SUCCESS",
        serde_json::json!({
            "archive_id": archive_id,
            "archive_no": qr_payload.archive_no,
            "citizen_status": qr_payload.citizen_status,
            "voting_eligible": qr_payload.voting_eligible,
            "sign_key_id": qr_payload.sign_key_id
        }),
    )
    .await?;

    Ok(Json(ok(QrGenerateData {
        qr_payload,
        qr_content,
    })))
}

async fn print_archive_qr(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(archive_id): Path<String>,
) -> Result<Json<ApiResponse<QrPrintData>>, (StatusCode, Json<ApiError>)> {
    let ctx = require_admin_access(&state, &headers).await?;
    let archive = {
        let archives = state.archives.read().await;
        archives
            .get(&archive_id)
            .cloned()
            .ok_or_else(|| err(StatusCode::NOT_FOUND, 3004, "archive not found"))?
    };
    let qr_payload = build_qr_payload(&state, &archive).await?;

    let record = QrPrintRecord {
        print_id: format!("qpr_{}", Uuid::new_v4().simple()),
        archive_id: archive.archive_id,
        archive_no: qr_payload.archive_no.clone(),
        citizen_status: qr_payload.citizen_status.clone(),
        voting_eligible: qr_payload.voting_eligible,
        printed_at: Utc::now().timestamp(),
    };
    state.qr_print_records.write().await.push(record.clone());

    write_audit(
        &state,
        Some(ctx.user_id),
        "PRINT_ARCHIVE_QR",
        "QR_PRINT_RECORD",
        Some(record.print_id.clone()),
        "SUCCESS",
        serde_json::json!({
            "archive_id": record.archive_id,
            "archive_no": record.archive_no,
            "citizen_status": record.citizen_status,
            "voting_eligible": record.voting_eligible,
            "sign_key_id": qr_payload.sign_key_id
        }),
    )
    .await?;

    Ok(Json(ok(QrPrintData {
        print_id: record.print_id,
        archive_id: record.archive_id,
        archive_no: record.archive_no,
        citizen_status: record.citizen_status,
        voting_eligible: record.voting_eligible,
        printed_at: record.printed_at,
    })))
}

async fn require_admin_access(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AuthContext, (StatusCode, Json<ApiError>)> {
    let ctx = require_auth(state, headers).await?;
    if ctx.role != "SUPER_ADMIN" && ctx.role != "OPERATOR_ADMIN" {
        return Err(err(StatusCode::FORBIDDEN, 2008, "permission denied"));
    }
    Ok(ctx)
}

async fn require_role(
    state: &AppState,
    headers: &HeaderMap,
    role: &str,
) -> Result<AuthContext, (StatusCode, Json<ApiError>)> {
    let ctx = require_auth(state, headers).await?;
    if ctx.role != role {
        return Err(err(StatusCode::FORBIDDEN, 2008, "permission denied"));
    }
    Ok(ctx)
}

async fn require_auth(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<AuthContext, (StatusCode, Json<ApiError>)> {
    let token = bearer_token(headers)?;

    let sessions = state.sessions.read().await;
    let session = sessions
        .get(&token)
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, 2001, "invalid token"))?;
    if session.expires_at < Utc::now().timestamp() {
        return Err(err(StatusCode::UNAUTHORIZED, 2009, "token expired"));
    }

    Ok(AuthContext {
        user_id: session.user_id.clone(),
        role: session.role.clone(),
    })
}

fn bearer_token(headers: &HeaderMap) -> Result<String, (StatusCode, Json<ApiError>)> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(ToOwned::to_owned)
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, 2001, "missing bearer token"))
}

fn extract_domain_from_origin(origin: &str) -> Option<String> {
    let trimmed = origin.trim();
    if trimmed.is_empty() {
        return None;
    }
    let no_scheme = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))
        .unwrap_or(trimmed);
    let host_port = no_scheme.split('/').next().unwrap_or("");
    if host_port.is_empty() {
        return None;
    }
    let domain = host_port.split(':').next().unwrap_or("");
    if domain.is_empty() {
        return None;
    }
    Some(domain.to_string())
}

async fn find_admin_by_pubkey(
    state: &AppState,
    admin_pubkey: &str,
) -> Result<AdminUser, (StatusCode, Json<ApiError>)> {
    let users = state.admin_users.read().await;
    users
        .values()
        .find(|u| u.admin_pubkey == admin_pubkey)
        .cloned()
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, 2002, "admin_pubkey not found"))
}

async fn find_admin_by_user_id(
    state: &AppState,
    user_id: &str,
) -> Result<AdminUser, (StatusCode, Json<ApiError>)> {
    let users = state.admin_users.read().await;
    users
        .get(user_id)
        .cloned()
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, 2002, "admin user not found"))
}

async fn generate_archive_no_with_retry(
    state: &AppState,
    province_code: &str,
    city_code: &str,
    created_date_yyyymmdd: &str,
    terminal_id: &str,
    admin_pubkey: &str,
) -> Result<String, (StatusCode, Json<ApiError>)> {
    let seq_key = format!("{}|{}|{}", province_code, city_code, created_date_yyyymmdd);
    let mut nonce = {
        let mut seq = state.sequence.write().await;
        let current = seq.entry(seq_key.clone()).or_insert(1);
        let value = *current;
        *current += 1;
        value
    };

    for _ in 0..ARCHIVE_NO_MAX_RETRY {
        let random9 = generate_random9(terminal_id, admin_pubkey, nonce);
        let check_digit =
            archive_checksum_digit(province_code, city_code, &random9, created_date_yyyymmdd);
        let archive_no = format!(
            "{}{}{}{}{}",
            province_code, city_code, check_digit, random9, created_date_yyyymmdd
        );

        let exists = {
            let archives = state.archives.read().await;
            archives.values().any(|a| a.archive_no == archive_no)
        };
        if !exists {
            return Ok(archive_no);
        }
        // 冲突时通过 nonce 递增重试，直到命中唯一档案号。
        nonce += 1;
    }

    Err(err(
        StatusCode::CONFLICT,
        3005,
        "archive_no conflict, retry exhausted",
    ))
}

fn generate_random9(terminal_id: &str, admin_pubkey: &str, nonce: u32) -> String {
    let ts = Utc::now().timestamp_millis();
    let source = format!("{}|{}|{}|{}", ts, terminal_id, admin_pubkey, nonce);
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    let n = hasher.finish() % 1_000_000_000;
    format!("{:09}", n)
}

fn archive_checksum_digit(
    province_code: &str,
    city_code: &str,
    random9: &str,
    created_date_yyyymmdd: &str,
) -> char {
    // v3: 省2+市3+随机9+创建日期8，和 SFID 保持同算法（BLAKE3 字节和 mod 10）。
    let payload = format!(
        "cpms-archive-v3|{}{}{}{}",
        province_code, city_code, random9, created_date_yyyymmdd
    );
    let digest = blake3::hash(payload.as_bytes());
    let sum: u32 = digest.as_bytes().iter().map(|&b| b as u32).sum();
    let n = (sum % 10) as u8;
    char::from(b'0' + n)
}

async fn build_qr_payload(
    state: &AppState,
    archive: &Archive,
) -> Result<QrPayload, (StatusCode, Json<ApiError>)> {
    let (site_sfid, sign_key) = active_qr_sign_key(state).await?;
    let issued_at = Utc::now().timestamp();
    let expire_at = issued_at + QR_EXPIRES_SECONDS;
    let qr_id = format!("qr_{}", Uuid::new_v4().simple());
    let voting_eligible = archive.citizen_status == "NORMAL";
    // 投票资格由 citizen_status 直接映射，避免 SFID 二次推导。
    let sign_source = format!(
        "cpms-qr-v1|{}|{}|{}|{}|{}|{}|{}",
        &site_sfid,
        sign_key.key_id,
        archive.archive_no,
        archive.citizen_status,
        voting_eligible,
        issued_at,
        qr_id
    );
    let signature = sign_qr_payload_with_secret(&sign_key.secret_bytes, &sign_source)?;

    Ok(QrPayload {
        ver: "1".to_string(),
        issuer_id: "cpms".to_string(),
        site_sfid: site_sfid.clone(),
        sign_key_id: sign_key.key_id,
        archive_no: archive.archive_no.clone(),
        citizen_status: archive.citizen_status.clone(),
        voting_eligible,
        issued_at,
        expire_at,
        qr_id,
        sig_alg: "sr25519".to_string(),
        signature,
    })
}

async fn build_site_key_registration_payload(
    state: &AppState,
) -> Result<SiteKeyRegistrationPayload, (StatusCode, Json<ApiError>)> {
    let (site_sfid, keys_runtime) = install_snapshot(state).await?;
    let sign_key = keys_runtime
        .iter()
        .find(|k| k.status == "ACTIVE")
        .cloned()
        .ok_or_else(|| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5002,
                "missing active qr sign key",
            )
        })?;
    let issued_at = Utc::now().timestamp();
    let qr_id = format!("qr_{}", Uuid::new_v4().simple());
    let keys: Vec<SiteKeyPublicItem> = keys_runtime
        .iter()
        .map(|key| SiteKeyPublicItem {
            key_id: key.key_id.clone(),
            purpose: key.purpose.clone(),
            status: key.status.clone(),
            pubkey: key.pubkey.clone(),
        })
        .collect();
    // 机构公钥登记二维码使用固定顺序拼接，避免跨系统验签串不一致。
    let key_summary = keys
        .iter()
        .map(|k| format!("{}:{}:{}", k.key_id, k.purpose, k.pubkey))
        .collect::<Vec<String>>()
        .join("|");
    let sign_source = format!(
        "cpms-site-key-register-v1|{}|{}|{}|{}",
        &site_sfid, key_summary, issued_at, qr_id
    );
    let signature = sign_qr_payload_with_secret(&sign_key.secret_bytes, &sign_source)?;

    Ok(SiteKeyRegistrationPayload {
        ver: "1".to_string(),
        qr_type: "CPMS_SITE_KEYS_REGISTER".to_string(),
        issuer_id: "cpms".to_string(),
        site_sfid: site_sfid.clone(),
        keys,
        issued_at,
        qr_id,
        sig_alg: "sr25519".to_string(),
        sign_key_id: sign_key.key_id,
        signature,
    })
}

async fn install_snapshot(
    state: &AppState,
) -> Result<(String, Vec<QrSignKeyRuntime>), (StatusCode, Json<ApiError>)> {
    let install = state.install.read().await;
    let site_sfid = install
        .site_sfid
        .clone()
        .ok_or_else(|| err(StatusCode::CONFLICT, 4003, "cpms not initialized"))?;
    if install.qr_sign_keys.is_empty() {
        return Err(err(
            StatusCode::CONFLICT,
            4005,
            "missing qr sign keys after initialization",
        ));
    }
    Ok((site_sfid, install.qr_sign_keys.clone()))
}

async fn active_qr_sign_key(
    state: &AppState,
) -> Result<(String, QrSignKeyRuntime), (StatusCode, Json<ApiError>)> {
    // 当前版本以状态为 ACTIVE 的密钥作为签发密钥（默认 K1 主密钥）。
    let (site_sfid, keys) = install_snapshot(state).await?;
    let sign_key = keys
        .iter()
        .find(|k| k.status == "ACTIVE")
        .cloned()
        .ok_or_else(|| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5002,
                "missing active qr sign key",
            )
        })?;
    Ok((site_sfid, sign_key))
}

fn build_super_admin_bind_qrs(
    site_sfid: Option<String>,
    keys: &[QrSignKeyRuntime],
    users: &HashMap<String, AdminUser>,
) -> Result<Vec<SuperAdminBindQrData>, (StatusCode, Json<ApiError>)> {
    let Some(site_sfid) = site_sfid else {
        return Ok(Vec::new());
    };

    keys.iter()
        .map(|key| {
            let expected_user_id =
                super_admin_user_id_for_key_id(&key.key_id).ok_or_else(|| {
                    err(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        5001,
                        "invalid fixed sign key id",
                    )
                })?;
            let bound = users
                .get(&expected_user_id)
                .map(|u| u.role == "SUPER_ADMIN")
                .unwrap_or(false);
            let bind_nonce = super_admin_bind_nonce(&site_sfid, &key.key_id, &key.pubkey);
            let qr_payload = SuperAdminBindQrPayload {
                ver: "1".to_string(),
                qr_type: "CPMS_SUPER_ADMIN_BIND".to_string(),
                issuer_id: "cpms".to_string(),
                site_sfid: site_sfid.clone(),
                sign_key_id: key.key_id.clone(),
                sign_key_pubkey: key.pubkey.clone(),
                bind_nonce,
                issued_at: Utc::now().timestamp(),
            };
            let qr_content = serde_json::to_string(&qr_payload)
                .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "qr encode failed"))?;
            Ok(SuperAdminBindQrData {
                key_id: key.key_id.clone(),
                bound,
                qr_payload,
                qr_content,
            })
        })
        .collect()
}

fn super_admin_bind_nonce(site_sfid: &str, key_id: &str, sign_key_pubkey: &str) -> String {
    // 绑定 nonce 按站点+密钥固定生成，重启后可重复展示并保持一致。
    let source = format!(
        "cpms-super-admin-bind-nonce-v1|{}|{}|{}",
        site_sfid, key_id, sign_key_pubkey
    );
    let digest = blake3::hash(source.as_bytes());
    hex::encode(&digest.as_bytes()[..16])
}

fn super_admin_bind_sign_source(
    site_sfid: &str,
    key_id: &str,
    admin_pubkey: &str,
    bind_nonce: &str,
) -> String {
    format!(
        "cpms-super-admin-bind-v1|{}|{}|{}|{}",
        site_sfid, key_id, admin_pubkey, bind_nonce
    )
}

fn validate_citizen_status(status: &str) -> Result<(), (StatusCode, Json<ApiError>)> {
    match status {
        "NORMAL" | "ABNORMAL" => Ok(()),
        _ => Err(err(StatusCode::BAD_REQUEST, 1001, "invalid citizen_status")),
    }
}

fn sign_qr_payload_with_secret(
    secret_bytes: &[u8],
    payload: &str,
) -> Result<String, (StatusCode, Json<ApiError>)> {
    if secret_bytes.len() != 32 {
        return Err(err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5003,
            "invalid qr sign secret length",
        ));
    }

    let mini = MiniSecretKey::from_bytes(&secret_bytes).map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5003,
            "invalid qr sign secret key",
        )
    })?;
    let keypair = mini.expand_to_keypair(schnorrkel::ExpansionMode::Ed25519);
    let sig = keypair.sign(signing_context(b"CPMS-QR-SIGN-V1").bytes(payload.as_bytes()));
    Ok(hex::encode(sig.to_bytes()))
}

fn validate_admin_status(status: &str) -> Result<(), (StatusCode, Json<ApiError>)> {
    match status {
        "ACTIVE" | "DISABLED" => Ok(()),
        _ => Err(err(StatusCode::BAD_REQUEST, 1001, "invalid status")),
    }
}

fn verify_challenge_signature(
    admin_pubkey: &str,
    challenge_payload: &str,
    signature: &str,
) -> Result<(), &'static str> {
    verify_signature_with_context(
        admin_pubkey,
        challenge_payload,
        signature,
        b"CPMS-ADMIN-AUTH-V1",
    )
}

fn verify_signature_with_context(
    admin_pubkey: &str,
    payload: &str,
    signature: &str,
    context: &[u8],
) -> Result<(), &'static str> {
    let pubkey_bytes = decode_bytes(admin_pubkey).ok_or("invalid admin_pubkey encoding")?;
    if pubkey_bytes.len() != 32 {
        return Err("invalid admin_pubkey length");
    }
    let sig_bytes = decode_bytes(signature).ok_or("invalid signature encoding")?;
    if sig_bytes.len() != 64 {
        return Err("invalid signature length");
    }

    let pk = PublicKey::from_bytes(&pubkey_bytes).map_err(|_| "invalid sr25519 public key")?;
    let sig = Signature::from_bytes(&sig_bytes).map_err(|_| "invalid sr25519 signature")?;

    let ctx = signing_context(context);
    pk.verify(ctx.bytes(payload.as_bytes()), &sig)
        .map_err(|_| "sr25519 verify failed")
}

fn decode_bytes(input: &str) -> Option<Vec<u8>> {
    let trimmed = input.trim();

    let hex_raw = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    if let Ok(v) = hex::decode(hex_raw) {
        return Some(v);
    }

    if let Ok(v) = STANDARD.decode(trimmed) {
        return Some(v);
    }

    None
}

fn load_runtime_store(path: &FsPath) -> Result<RuntimeStore, String> {
    if !path.exists() {
        return Ok(RuntimeStore::default());
    }
    let raw = fs::read(path)
        .map_err(|e| format!("read runtime store {} failed: {e}", path.display()))?;
    if raw.is_empty() {
        return Ok(RuntimeStore::default());
    }
    serde_json::from_slice::<RuntimeStore>(&raw)
        .map_err(|e| format!("parse runtime store {} failed: {e}", path.display()))
}

async fn snapshot_runtime_store(state: &AppState) -> RuntimeStore {
    RuntimeStore {
        admin_users: state.admin_users.read().await.clone(),
        sessions: state.sessions.read().await.clone(),
        login_challenges: state.login_challenges.read().await.clone(),
        qr_login_results: state.qr_login_results.read().await.clone(),
        archives: state.archives.read().await.clone(),
        sequence: state.sequence.read().await.clone(),
        qr_print_records: state.qr_print_records.read().await.clone(),
        audit_logs: state.audit_logs.read().await.clone(),
    }
}

fn persist_runtime_store_to_path(path: &FsPath, store: &RuntimeStore) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("runtime store path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|e| format!("create runtime store dir {} failed: {e}", parent.display()))?;
    let bytes = serde_json::to_vec_pretty(store)
        .map_err(|e| format!("serialize runtime store failed: {e}"))?;
    let tmp_path = parent.join(format!(
        ".{}.tmp",
        path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("cpms_runtime_store.json")
    ));
    fs::write(&tmp_path, bytes)
        .map_err(|e| format!("write runtime store tmp {} failed: {e}", tmp_path.display()))?;
    fs::rename(&tmp_path, path).map_err(|e| {
        format!(
            "rename runtime store {} -> {} failed: {e}",
            tmp_path.display(),
            path.display()
        )
    })?;
    Ok(())
}

async fn persist_runtime_store(state: &AppState) -> Result<(), String> {
    let snapshot = snapshot_runtime_store(state).await;
    persist_runtime_store_to_path(&state.runtime_store_path, &snapshot)
}

async fn write_audit(
    state: &AppState,
    operator_user_id: Option<String>,
    action: &str,
    target_type: &str,
    target_id: Option<String>,
    result: &str,
    detail: serde_json::Value,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    let log = AuditLog {
        log_id: format!("log_{}", Uuid::new_v4().simple()),
        operator_user_id,
        action: action.to_string(),
        target_type: target_type.to_string(),
        target_id,
        result: result.to_string(),
        detail,
        created_at: Utc::now().timestamp(),
    };
    state.audit_logs.write().await.push(log);
    persist_runtime_store(state)
        .await
        .map_err(|reason| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, &reason))?;
    Ok(())
}

fn ok<T: Serialize>(data: T) -> ApiResponse<T> {
    ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: Some(data),
    }
}

fn err(status: StatusCode, code: i32, message: &str) -> (StatusCode, Json<ApiError>) {
    (
        status,
        Json(ApiError {
            code,
            message: message.to_string(),
            trace_id: Uuid::new_v4().to_string(),
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        archive_checksum_digit, sign_qr_payload_with_secret, validate_citizen_status,
        verify_challenge_signature,
    };
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use schnorrkel::{signing_context, ExpansionMode, MiniSecretKey, PublicKey, Signature};

    #[test]
    fn verify_signature_accepts_hex_inputs() {
        let payload = "cpms-admin-auth-v1|chl_x|pub_x|nonce_x|1234567890";
        let (pubkey_hex, sig_hex) = build_signed_payload(payload);
        assert!(verify_challenge_signature(&pubkey_hex, payload, &sig_hex).is_ok());
    }

    #[test]
    fn verify_signature_accepts_base64_inputs() {
        let payload = "cpms-admin-auth-v1|chl_y|pub_y|nonce_y|1234567890";
        let (pubkey_hex, sig_hex) = build_signed_payload(payload);
        let pubkey_raw = hex::decode(pubkey_hex).expect("hex pubkey decode");
        let sig_raw = hex::decode(sig_hex).expect("hex signature decode");
        let pubkey_b64 = STANDARD.encode(pubkey_raw);
        let sig_b64 = STANDARD.encode(sig_raw);
        assert!(verify_challenge_signature(&pubkey_b64, payload, &sig_b64).is_ok());
    }

    #[test]
    fn verify_signature_rejects_tampered_payload() {
        let payload = "cpms-admin-auth-v1|chl_z|pub_z|nonce_z|1234567890";
        let (pubkey_hex, sig_hex) = build_signed_payload(payload);
        let tampered = "cpms-admin-auth-v1|chl_z|pub_z|nonce_z|1234567891";
        let result = verify_challenge_signature(&pubkey_hex, tampered, &sig_hex);
        assert!(result.is_err());
    }

    #[test]
    fn verify_signature_rejects_invalid_encoding() {
        let payload = "cpms-admin-auth-v1|chl_w|pub_w|nonce_w|1234567890";
        let result = verify_challenge_signature("not-a-key", payload, "not-a-signature");
        assert!(result.is_err());
    }

    #[test]
    fn citizen_status_validation_works() {
        assert!(validate_citizen_status("NORMAL").is_ok());
        assert!(validate_citizen_status("ABNORMAL").is_ok());
        assert!(validate_citizen_status("UNKNOWN").is_err());
    }

    #[test]
    fn qr_signature_can_be_verified() {
        let payload = "cpms-qr-v1|site|archive_no|NORMAL|true|1700000000|qr_1";
        let secret = [9u8; 32];
        let sig_hex = match sign_qr_payload_with_secret(&secret, payload) {
            Ok(v) => v,
            Err(_) => panic!("sign failed"),
        };
        let sig_bytes = hex::decode(sig_hex).expect("decode signature");
        let sig = Signature::from_bytes(&sig_bytes).expect("signature bytes");

        let mini = MiniSecretKey::from_bytes(&secret).expect("mini secret key");
        let keypair = mini.expand_to_keypair(ExpansionMode::Ed25519);
        let pk = PublicKey::from_bytes(&keypair.public.to_bytes()).expect("public key bytes");
        let verify_result = pk.verify(
            signing_context(b"CPMS-QR-SIGN-V1").bytes(payload.as_bytes()),
            &sig,
        );
        assert!(verify_result.is_ok());
    }

    #[test]
    fn archive_no_v3_format_is_stable() {
        let province = "GD";
        let city = "001";
        let random9 = "123456789";
        let created_date = "20260227";
        let check = archive_checksum_digit(province, city, random9, created_date);
        let archive_no = format!("{}{}{}{}{}", province, city, check, random9, created_date);
        assert_eq!(archive_no.len(), 23);
        assert!(archive_no.starts_with("GD001"));
        assert!(archive_no.ends_with("20260227"));
    }

    fn build_signed_payload(payload: &str) -> (String, String) {
        let mini = MiniSecretKey::from_bytes(&[7u8; 32]).expect("mini secret key");
        let keypair = mini.expand_to_keypair(ExpansionMode::Ed25519);
        let sig = keypair.sign(signing_context(b"CPMS-ADMIN-AUTH-V1").bytes(payload.as_bytes()));
        (
            hex::encode(keypair.public.to_bytes()),
            hex::encode(sig.to_bytes()),
        )
    }
}
