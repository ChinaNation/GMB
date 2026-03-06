use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::Utc;
use rand::rngs::OsRng;
use schnorrkel::{signing_context, MiniSecretKey, PublicKey, Signature};
use serde::{Deserialize, Serialize};

use crate::{err, ok, write_audit, AdminUser, ApiError, ApiResponse, AppState};

const FIXED_SUPER_ADMIN_COUNT: usize = 3;
const FIXED_QR_SIGN_KEY_COUNT: usize = 3;

#[derive(Clone)]
pub struct QrSignKeyRuntime {
    pub key_id: String,
    pub purpose: String,
    pub status: String,
    pub pubkey: String,
    pub secret_bytes: Vec<u8>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BootstrapInstallData {
    pub site_sfid: String,
    pub super_admins: Vec<BootstrapSuperAdmin>,
    qr_sign_keys: Vec<BootstrapQrSignKey>,
    version: String,
    created_at: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BootstrapSuperAdmin {
    pub user_id: String,
    pub admin_pubkey: String,
    #[serde(default)]
    pub managed_key_id: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct BootstrapQrSignKey {
    key_id: String,
    purpose: String,
    status: String,
    pubkey: String,
    secret: String,
}

#[derive(Clone)]
pub struct RuntimeInstallData {
    pub was_created: bool,
    pub file_path: PathBuf,
    pub data: Option<BootstrapInstallData>,
    pub qr_sign_keys: Vec<QrSignKeyRuntime>,
}

#[derive(Deserialize)]
struct SfidInstallQrPayload {
    ver: String,
    qr_type: String,
    issuer_id: String,
    site_sfid: String,
    issued_at: i64,
    qr_id: String,
    sig_alg: String,
    signature: String,
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

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/install/status", get(install_status))
        .route("/api/v1/install/initialize", post(initialize_install))
        .route(
            "/api/v1/install/super-admin/bind",
            post(bind_super_admin_from_wuminapp),
        )
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
    crate::verify_signature_with_context(
        &req.admin_pubkey,
        &bind_sign_source,
        &req.signature,
        b"CPMS-SUPER-ADMIN-BIND-V1",
    )
    .map_err(|reason| err(StatusCode::UNAUTHORIZED, 2002, reason))?;

    let created = bind_super_admin(&file_path, &req.key_id, &req.admin_pubkey)
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

pub fn load_or_init_install_data() -> Result<RuntimeInstallData, String> {
    let file_path = install_file_path();
    if !file_path.exists() {
        // 首次启动允许处于“待初始化”状态，等待扫码 SFID 安装二维码后再创建安装文件。
        return Ok(RuntimeInstallData {
            was_created: false,
            file_path,
            data: None,
            qr_sign_keys: Vec::new(),
        });
    }

    let content = fs::read_to_string(&file_path).map_err(|e| {
        format!(
            "read install bootstrap file '{}' failed: {e}",
            file_path.display()
        )
    })?;
    let mut data: BootstrapInstallData = serde_json::from_str(&content).map_err(|e| {
        format!(
            "parse install bootstrap file '{}' failed: {e}",
            file_path.display()
        )
    })?;
    normalize_legacy_super_admins(&mut data);
    validate_install_data(&data)?;
    let qr_sign_keys = runtime_qr_sign_keys(&data.qr_sign_keys)?;

    Ok(RuntimeInstallData {
        was_created: false,
        file_path,
        data: Some(data),
        qr_sign_keys,
    })
}

pub fn initialize_install_data_from_sfid_qr(
    sfid_init_qr_content: &str,
) -> Result<RuntimeInstallData, String> {
    let file_path = install_file_path();
    if file_path.exists() {
        return Err(format!(
            "install bootstrap file '{}' already exists",
            file_path.display()
        ));
    }

    let qr_payload = parse_sfid_install_qr_content(sfid_init_qr_content)?;
    validate_sfid_install_qr(&qr_payload)?;

    let data = create_install_data(qr_payload.site_sfid);
    validate_install_data(&data)?;
    persist_install_data(&file_path, &data)?;
    let qr_sign_keys = runtime_qr_sign_keys(&data.qr_sign_keys)?;

    Ok(RuntimeInstallData {
        was_created: true,
        file_path,
        data: Some(data),
        qr_sign_keys,
    })
}

pub fn bind_super_admin(
    file_path: &Path,
    key_id: &str,
    admin_pubkey: &str,
) -> Result<BootstrapSuperAdmin, String> {
    let content = fs::read_to_string(file_path).map_err(|e| {
        format!(
            "read install bootstrap file '{}' failed: {e}",
            file_path.display()
        )
    })?;
    let mut data: BootstrapInstallData = serde_json::from_str(&content).map_err(|e| {
        format!(
            "parse install bootstrap file '{}' failed: {e}",
            file_path.display()
        )
    })?;
    normalize_legacy_super_admins(&mut data);
    validate_install_data(&data)?;

    if data.super_admins.len() >= FIXED_SUPER_ADMIN_COUNT {
        return Err(format!(
            "super admin count reached {}",
            FIXED_SUPER_ADMIN_COUNT
        ));
    }

    let trimmed_key_id = key_id.trim();
    let trimmed_admin_pubkey = admin_pubkey.trim();
    if trimmed_key_id.is_empty() || trimmed_admin_pubkey.is_empty() {
        return Err("invalid key_id or admin_pubkey".to_string());
    }

    if !data.qr_sign_keys.iter().any(|k| k.key_id == trimmed_key_id) {
        return Err(format!("unknown sign key_id '{}'", trimmed_key_id));
    }

    if data
        .super_admins
        .iter()
        .any(|a| a.managed_key_id == trimmed_key_id)
    {
        return Err(format!(
            "sign key '{}' already bound to super admin",
            trimmed_key_id
        ));
    }

    if data
        .super_admins
        .iter()
        .any(|a| a.admin_pubkey == trimmed_admin_pubkey)
    {
        return Err("admin_pubkey already bound to super admin".to_string());
    }

    let user_id = super_admin_user_id_for_key_id(trimmed_key_id)
        .ok_or_else(|| format!("unsupported sign key_id '{}'", trimmed_key_id))?;

    let created = BootstrapSuperAdmin {
        user_id,
        admin_pubkey: trimmed_admin_pubkey.to_string(),
        managed_key_id: trimmed_key_id.to_string(),
    };

    data.super_admins.push(created.clone());
    validate_install_data(&data)?;
    persist_install_data(file_path, &data)?;

    Ok(created)
}

pub fn init_super_admin_users(super_admins: &[BootstrapSuperAdmin]) -> HashMap<String, AdminUser> {
    let mut users = HashMap::new();
    for super_admin in super_admins {
        let user = AdminUser {
            user_id: super_admin.user_id.clone(),
            admin_pubkey: super_admin.admin_pubkey.clone(),
            role: "SUPER_ADMIN".to_string(),
            status: "ACTIVE".to_string(),
            immutable: true,
        };
        users.insert(user.user_id.clone(), user);
    }
    users
}

pub fn super_admin_user_id_for_key_id(key_id: &str) -> Option<String> {
    match key_id {
        "K1" => Some("u_super_admin_01".to_string()),
        "K2" => Some("u_super_admin_02".to_string()),
        "K3" => Some("u_super_admin_03".to_string()),
        _ => None,
    }
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

fn install_file_path() -> PathBuf {
    std::env::var("CPMS_INSTALL_FILE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("runtime/cpms_install_init.json"))
}

fn create_install_data(site_sfid: String) -> BootstrapInstallData {
    let trimmed_site_sfid = site_sfid.trim().to_string();
    if trimmed_site_sfid.is_empty() {
        panic!("site_sfid from SFID install QR is empty");
    }

    let mut qr_sign_keys = Vec::with_capacity(FIXED_QR_SIGN_KEY_COUNT);
    let key_meta = [
        (
            "K1".to_string(),
            "PRIMARY".to_string(),
            "ACTIVE".to_string(),
        ),
        (
            "K2".to_string(),
            "BACKUP".to_string(),
            "STANDBY".to_string(),
        ),
        (
            "K3".to_string(),
            "EMERGENCY".to_string(),
            "STANDBY".to_string(),
        ),
    ];
    for (key_id, purpose, status) in key_meta {
        let (pubkey, secret) = generate_sr25519_keypair_hex();
        qr_sign_keys.push(BootstrapQrSignKey {
            key_id,
            purpose,
            status,
            pubkey,
            secret,
        });
    }

    BootstrapInstallData {
        version: "2".to_string(),
        created_at: Utc::now().timestamp(),
        site_sfid: trimmed_site_sfid,
        super_admins: Vec::new(),
        qr_sign_keys,
    }
}

fn persist_install_data(file_path: &Path, data: &BootstrapInstallData) -> Result<(), String> {
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "create install bootstrap dir '{}' failed: {e}",
                parent.display()
            )
        })?;
    }
    let json = serde_json::to_string_pretty(data)
        .map_err(|e| format!("encode install bootstrap json failed: {e}"))?;
    fs::write(file_path, json).map_err(|e| {
        format!(
            "write install bootstrap file '{}' failed: {e}",
            file_path.display()
        )
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = fs::Permissions::from_mode(0o600);
        let _ = fs::set_permissions(file_path, permissions);
    }
    Ok(())
}

fn normalize_legacy_super_admins(data: &mut BootstrapInstallData) {
    // 兼容旧安装文件：若缺少 managed_key_id，则按固定账号后缀回填 K1/K2/K3。
    for admin in &mut data.super_admins {
        if !admin.managed_key_id.trim().is_empty() {
            continue;
        }
        admin.managed_key_id = match admin.user_id.as_str() {
            "u_super_admin_01" => "K1".to_string(),
            "u_super_admin_02" => "K2".to_string(),
            "u_super_admin_03" => "K3".to_string(),
            _ => String::new(),
        };
    }
}

fn validate_install_data(data: &BootstrapInstallData) -> Result<(), String> {
    if data.site_sfid.trim().is_empty() {
        return Err("install bootstrap site_sfid is empty".to_string());
    }

    if data.qr_sign_keys.len() != FIXED_QR_SIGN_KEY_COUNT {
        return Err(format!(
            "install bootstrap qr_sign_keys count must be {}",
            FIXED_QR_SIGN_KEY_COUNT
        ));
    }

    if data.super_admins.len() > FIXED_SUPER_ADMIN_COUNT {
        return Err(format!(
            "install bootstrap super_admins count must be <= {}",
            FIXED_SUPER_ADMIN_COUNT
        ));
    }

    let mut key_ids = HashMap::new();
    for key in &data.qr_sign_keys {
        if key.key_id.trim().is_empty()
            || key.pubkey.trim().is_empty()
            || key.secret.trim().is_empty()
            || key.status.trim().is_empty()
            || key.purpose.trim().is_empty()
        {
            return Err("install bootstrap qr_sign_key item is invalid".to_string());
        }
        if key_ids.insert(key.key_id.clone(), ()).is_some() {
            return Err("install bootstrap has duplicated key_id".to_string());
        }
    }

    let mut admin_pubkeys = HashMap::new();
    let mut managed_key_ids = HashMap::new();
    for admin in &data.super_admins {
        if admin.user_id.trim().is_empty()
            || admin.admin_pubkey.trim().is_empty()
            || admin.managed_key_id.trim().is_empty()
        {
            return Err("install bootstrap super_admin item is invalid".to_string());
        }

        if admin_pubkeys
            .insert(admin.admin_pubkey.clone(), ())
            .is_some()
        {
            return Err("install bootstrap has duplicated super admin pubkey".to_string());
        }

        if managed_key_ids
            .insert(admin.managed_key_id.clone(), ())
            .is_some()
        {
            return Err("install bootstrap has duplicated managed_key_id".to_string());
        }

        if !data
            .qr_sign_keys
            .iter()
            .any(|k| k.key_id == admin.managed_key_id)
        {
            return Err("install bootstrap super admin managed_key_id is invalid".to_string());
        }

        let expected_user_id =
            super_admin_user_id_for_key_id(&admin.managed_key_id).ok_or_else(|| {
                "install bootstrap super admin managed_key_id is unsupported".to_string()
            })?;
        if expected_user_id != admin.user_id {
            return Err("install bootstrap super admin user_id/key_id mismatch".to_string());
        }
    }

    if !data
        .qr_sign_keys
        .iter()
        .any(|k| k.status == "ACTIVE" && k.purpose == "PRIMARY")
    {
        return Err("install bootstrap missing active primary qr sign key".to_string());
    }

    Ok(())
}

fn runtime_qr_sign_keys(keys: &[BootstrapQrSignKey]) -> Result<Vec<QrSignKeyRuntime>, String> {
    keys.iter()
        .map(|k| {
            let secret_bytes = decode_bytes(&k.secret)
                .ok_or_else(|| format!("invalid qr sign secret encoding for {}", k.key_id))?;
            if secret_bytes.len() != 32 {
                return Err(format!("invalid qr sign secret length for {}", k.key_id));
            }
            Ok(QrSignKeyRuntime {
                key_id: k.key_id.clone(),
                purpose: k.purpose.clone(),
                status: k.status.clone(),
                pubkey: k.pubkey.clone(),
                secret_bytes,
            })
        })
        .collect::<Result<Vec<QrSignKeyRuntime>, String>>()
}

fn parse_sfid_install_qr_content(content: &str) -> Result<SfidInstallQrPayload, String> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err("sfid_init_qr_content is empty".to_string());
    }

    if let Ok(payload) = serde_json::from_str::<SfidInstallQrPayload>(trimmed) {
        return Ok(payload);
    }

    if let Ok(decoded) = STANDARD.decode(trimmed) {
        if let Ok(decoded_text) = String::from_utf8(decoded) {
            if let Ok(payload) = serde_json::from_str::<SfidInstallQrPayload>(&decoded_text) {
                return Ok(payload);
            }
        }
    }

    Err("invalid sfid_init_qr_content, expected json or base64(json)".to_string())
}

fn validate_sfid_install_qr(payload: &SfidInstallQrPayload) -> Result<(), String> {
    if payload.ver.trim().is_empty()
        || payload.qr_type.trim().is_empty()
        || payload.issuer_id.trim().is_empty()
        || payload.site_sfid.trim().is_empty()
        || payload.sig_alg.trim().is_empty()
        || payload.signature.trim().is_empty()
        || payload.qr_id.trim().is_empty()
    {
        return Err("invalid sfid install qr payload".to_string());
    }

    if payload.qr_type != "SFID_CPMS_INSTALL" {
        return Err(format!(
            "invalid sfid install qr_type '{}', expected SFID_CPMS_INSTALL",
            payload.qr_type
        ));
    }

    let sfid_pubkey = std::env::var("SFID_ROOT_PUBKEY")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| "SFID_ROOT_PUBKEY is required for install qr verification".to_string())?;

    let sign_source = format!(
        "sfid-cpms-install-v1|{}|{}|{}",
        payload.site_sfid, payload.issued_at, payload.qr_id
    );

    verify_sr25519_signature(
        &sfid_pubkey,
        &sign_source,
        &payload.signature,
        b"SFID-CPMS-INSTALL-V1",
    )
}

fn verify_sr25519_signature(
    pubkey: &str,
    payload: &str,
    signature: &str,
    context: &[u8],
) -> Result<(), String> {
    let pubkey_bytes = decode_bytes(pubkey).ok_or_else(|| "invalid pubkey encoding".to_string())?;
    if pubkey_bytes.len() != 32 {
        return Err("invalid pubkey length".to_string());
    }

    let sig_bytes =
        decode_bytes(signature).ok_or_else(|| "invalid signature encoding".to_string())?;
    if sig_bytes.len() != 64 {
        return Err("invalid signature length".to_string());
    }

    let pk = PublicKey::from_bytes(&pubkey_bytes)
        .map_err(|_| "invalid sr25519 public key".to_string())?;
    let sig =
        Signature::from_bytes(&sig_bytes).map_err(|_| "invalid sr25519 signature".to_string())?;
    pk.verify(signing_context(context).bytes(payload.as_bytes()), &sig)
        .map_err(|_| "sr25519 verify failed".to_string())
}

fn generate_sr25519_keypair_hex() -> (String, String) {
    let mini = MiniSecretKey::generate_with(OsRng);
    let secret = mini.to_bytes();
    let keypair = mini.expand_to_keypair(schnorrkel::ExpansionMode::Ed25519);
    (hex::encode(keypair.public.to_bytes()), hex::encode(secret))
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
