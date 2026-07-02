//! onchina 内网 API 机构私有 CA TLS(Card 05)。
//!
//! 每个机构节点首启生成自己的私有根 CA,再用该 CA 签发 `onchina.local`
//! 服务证书。员工电脑只下载并信任 CA 公钥证书;CA 私钥永不通过 HTTP 暴露。

use std::fs;
use std::path::PathBuf;

use axum_server::tls_rustls::RustlsConfig;
use base64::Engine as _;
use rcgen::{
    date_time_ymd, BasicConstraints, Certificate, CertificateParams, DnType,
    ExtendedKeyUsagePurpose, IsCa, KeyPair, KeyUsagePurpose,
};
use sha2::{Digest, Sha256};
use time::{Duration, OffsetDateTime};

const ONCHINA_TLS_HOST: &str = "onchina.local";
const ORG_CA_CERT_FILE: &str = "onchina-org-root-ca.crt";
const ORG_CA_KEY_FILE: &str = "onchina-org-root-ca.key";
const SERVER_CERT_FILE: &str = "onchina-server.crt";
const SERVER_KEY_FILE: &str = "onchina-server.key";
const HOST_MARKER_FILE: &str = "onchina-cert-host.txt";
const PROFILE_MARKER_FILE: &str = "onchina-cert-profile.txt";
const TLS_PROFILE_VERSION: &str = "onchina-ca-v2-ca2036-server397d";
const ORG_CA_COMMON_NAME: &str = "OnChina Organization Root CA";
const SERVER_COMMON_NAME: &str = "onchina.local";
const ORG_CA_VALID_UNTIL: &str = "2036-01-01T00:00:00Z";
const SERVER_VALID_DAYS: i64 = 397;

#[derive(Clone, Debug)]
pub(crate) struct CaCertificateInfo {
    pub(crate) filename: &'static str,
    pub(crate) sha256: String,
    pub(crate) subject: &'static str,
    pub(crate) valid_until: String,
}

/// 是否启用 HTTPS(桌面/生产安装默认开;本地开发脚本同样开启 HTTPS)。
pub(crate) fn is_enabled() -> bool {
    std::env::var("ONCHINA_ENABLE_TLS")
        .ok()
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            v == "1" || v == "true" || v == "yes"
        })
        .unwrap_or(false)
}

/// 证书持久化目录:`ONCHINA_TLS_DIR`(node 传 `base_path/onchina-tls`);兜底 exe 同目录 `tls`。
pub(crate) fn tls_dir() -> PathBuf {
    if let Some(dir) = std::env::var("ONCHINA_TLS_DIR")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
    {
        return PathBuf::from(dir);
    }
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("tls")
}

fn ca_cert_path() -> PathBuf {
    tls_dir().join(ORG_CA_CERT_FILE)
}

fn ca_key_path() -> PathBuf {
    tls_dir().join(ORG_CA_KEY_FILE)
}

fn server_cert_path() -> PathBuf {
    tls_dir().join(SERVER_CERT_FILE)
}

fn server_key_path() -> PathBuf {
    tls_dir().join(SERVER_KEY_FILE)
}

fn host_marker_path() -> PathBuf {
    tls_dir().join(HOST_MARKER_FILE)
}

fn profile_marker_path() -> PathBuf {
    tls_dir().join(PROFILE_MARKER_FILE)
}

fn org_ca_params() -> CertificateParams {
    let mut params = CertificateParams::default();
    params.not_before = date_time_ymd(2026, 1, 1);
    params.not_after = date_time_ymd(2036, 1, 1);
    params
        .distinguished_name
        .push(DnType::OrganizationName, "OnChina");
    params
        .distinguished_name
        .push(DnType::CommonName, ORG_CA_COMMON_NAME);
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
    params
}

fn server_params() -> Result<CertificateParams, String> {
    let mut params = CertificateParams::new(vec![ONCHINA_TLS_HOST.to_string()])
        .map_err(|e| format!("build onchina server cert params failed: {e}"))?;
    let now = OffsetDateTime::now_utc();
    params.not_before = now - Duration::days(1);
    params.not_after = now + Duration::days(SERVER_VALID_DAYS);
    params
        .distinguished_name
        .push(DnType::OrganizationName, "OnChina");
    params
        .distinguished_name
        .push(DnType::CommonName, SERVER_COMMON_NAME);
    params.is_ca = IsCa::ExplicitNoCa;
    params.key_usages = vec![
        KeyUsagePurpose::DigitalSignature,
        KeyUsagePurpose::KeyEncipherment,
    ];
    params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    Ok(params)
}

fn write_secret_file(path: PathBuf, content: &str) -> Result<(), String> {
    fs::write(&path, content).map_err(|e| format!("write {} failed: {e}", path.display()))?;
    restrict_secret_file(&path)?;
    Ok(())
}

#[cfg(unix)]
fn restrict_secret_file(path: &std::path::Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)
        .map_err(|e| format!("read {} metadata failed: {e}", path.display()))?
        .permissions();
    perms.set_mode(0o600);
    fs::set_permissions(path, perms)
        .map_err(|e| format!("restrict {} permissions failed: {e}", path.display()))
}

#[cfg(not(unix))]
fn restrict_secret_file(_path: &std::path::Path) -> Result<(), String> {
    Ok(())
}

fn cert_host_matches() -> bool {
    fs::read_to_string(host_marker_path())
        .ok()
        .is_some_and(|value| value.trim() == ONCHINA_TLS_HOST)
}

fn cert_profile_matches() -> bool {
    fs::read_to_string(profile_marker_path())
        .ok()
        .is_some_and(|value| value.trim() == TLS_PROFILE_VERSION)
}

fn ca_material_exists() -> bool {
    ca_cert_path().is_file() && ca_key_path().is_file()
}

fn server_material_exists() -> bool {
    server_cert_path().is_file() && server_key_path().is_file()
}

fn load_ca_key_pair() -> Result<KeyPair, String> {
    let pem = fs::read_to_string(ca_key_path()).map_err(|e| format!("read CA key failed: {e}"))?;
    KeyPair::from_pem(pem.as_str()).map_err(|e| format!("parse CA key failed: {e}"))
}

fn write_cert_markers() -> Result<(), String> {
    fs::write(host_marker_path(), ONCHINA_TLS_HOST)
        .map_err(|e| format!("write tls host marker failed: {e}"))?;
    fs::write(profile_marker_path(), TLS_PROFILE_VERSION)
        .map_err(|e| format!("write tls profile marker failed: {e}"))?;
    Ok(())
}

fn generate_ca_material() -> Result<(Certificate, KeyPair), String> {
    let ca_key = KeyPair::generate().map_err(|e| format!("generate org CA key failed: {e}"))?;
    let ca_cert = org_ca_params()
        .self_signed(&ca_key)
        .map_err(|e| format!("generate org CA cert failed: {e}"))?;
    fs::write(ca_cert_path(), ca_cert.pem()).map_err(|e| format!("write CA cert failed: {e}"))?;
    write_secret_file(ca_key_path(), ca_key.serialize_pem().as_str())?;
    Ok((ca_cert, ca_key))
}

fn load_or_generate_ca(force_regenerate: bool) -> Result<(Certificate, KeyPair, bool), String> {
    if !force_regenerate && ca_material_exists() {
        let ca_key = load_ca_key_pair()?;
        let ca_cert = org_ca_params()
            .self_signed(&ca_key)
            .map_err(|e| format!("rebuild org CA cert failed: {e}"))?;
        return Ok((ca_cert, ca_key, false));
    }
    generate_ca_material().map(|(ca_cert, ca_key)| (ca_cert, ca_key, true))
}

fn generate_server_material(ca_cert: &Certificate, ca_key: &KeyPair) -> Result<(), String> {
    let server_key =
        KeyPair::generate().map_err(|e| format!("generate onchina server key failed: {e}"))?;
    let server_cert = server_params()?
        .signed_by(&server_key, ca_cert, ca_key)
        .map_err(|e| format!("sign onchina server cert failed: {e}"))?;
    fs::write(server_cert_path(), server_cert.pem())
        .map_err(|e| format!("write server cert failed: {e}"))?;
    write_secret_file(server_key_path(), server_key.serialize_pem().as_str())?;
    Ok(())
}

fn ensure_certificate_material(refresh_server: bool) -> Result<(), String> {
    let dir = tls_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("create tls dir failed: {e}"))?;
    let profile_matches = cert_profile_matches();
    let (ca_cert, ca_key, ca_regenerated) = load_or_generate_ca(!profile_matches)?;
    if refresh_server
        || ca_regenerated
        || !(server_material_exists() && cert_host_matches() && profile_matches)
    {
        generate_server_material(&ca_cert, &ca_key)?;
        write_cert_markers()?;
        tracing::info!(
            dir = %dir.display(),
            host = ONCHINA_TLS_HOST,
            profile = TLS_PROFILE_VERSION,
            server_valid_days = SERVER_VALID_DAYS,
            "onchina organization CA TLS cert generated"
        );
    }
    Ok(())
}

/// 读取机构 CA 公钥证书,用于员工浏览器下载并安装到受信任根证书。
pub(crate) fn organization_ca_certificate_pem() -> Result<String, String> {
    ensure_certificate_material(false)?;
    fs::read_to_string(ca_cert_path()).map_err(|e| format!("read CA cert failed: {e}"))
}

fn certificate_pem_to_der(pem: &str) -> Result<Vec<u8>, String> {
    let body = pem
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("-----"))
        .collect::<String>();
    base64::engine::general_purpose::STANDARD
        .decode(body.as_bytes())
        .map_err(|e| format!("decode CA certificate PEM failed: {e}"))
}

pub(crate) fn organization_ca_certificate_info() -> Result<CaCertificateInfo, String> {
    let pem = organization_ca_certificate_pem()?;
    let der = certificate_pem_to_der(pem.as_str())?;
    let sha256 = hex::encode(Sha256::digest(der.as_slice()));
    Ok(CaCertificateInfo {
        filename: ORG_CA_CERT_FILE,
        sha256,
        subject: ORG_CA_COMMON_NAME,
        valid_until: ORG_CA_VALID_UNTIL.to_string(),
    })
}

/// 加载已有机构 CA 签发证书;无则生成机构 CA + onchina.local 服务证书。
pub(crate) async fn load_or_generate_rustls_config() -> Result<RustlsConfig, String> {
    // rustls 0.23 需要进程级 CryptoProvider;幂等安装 ring 实现。
    let _ = rustls::crypto::ring::default_provider().install_default();

    ensure_certificate_material(true)?;

    RustlsConfig::from_pem_file(server_cert_path(), server_key_path())
        .await
        .map_err(|e| format!("load onchina tls cert failed: {e}"))
}
