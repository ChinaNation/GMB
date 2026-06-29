//! onchina 内网 API 自签 TLS(Card 05)。
//!
//! 中文注释:去中心化每市自治节点对内网提供 HTTPS——首启用 rcgen 生成自签证书并持久化,
//! 之后复用。内网客户端首次信任该自签证书即可(部署文档说明)。身份认证由扫码签名(3b
//! 链上集合鉴权)完成,TLS 只负责传输加密。与 node 的 libp2p WSS 证书相互独立。

use std::path::PathBuf;

use axum_server::tls_rustls::RustlsConfig;

/// 是否启用 HTTPS(桌面/生产安装默认开;本地开发可关走 HTTP)。
pub(crate) fn is_enabled() -> bool {
    std::env::var("CID_ENABLE_TLS")
        .ok()
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            v == "1" || v == "true" || v == "yes"
        })
        .unwrap_or(false)
}

/// 证书持久化目录:`CID_TLS_DIR`(node 传 `base_path/onchina-tls`);兜底 exe 同目录 `tls`。
fn tls_dir() -> PathBuf {
    if let Some(dir) = std::env::var("CID_TLS_DIR")
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

/// 加载已有自签证书;无则用 rcgen 生成(localhost + 127.0.0.1 SAN)并持久化。
pub(crate) async fn load_or_generate_rustls_config() -> Result<RustlsConfig, String> {
    // 中文注释:rustls 0.23 需要进程级 CryptoProvider;幂等安装 ring 实现。
    let _ = rustls::crypto::ring::default_provider().install_default();

    let dir = tls_dir();
    let cert_path = dir.join("onchina-cert.pem");
    let key_path = dir.join("onchina-key.pem");

    if !(cert_path.is_file() && key_path.is_file()) {
        std::fs::create_dir_all(&dir).map_err(|e| format!("create tls dir failed: {e}"))?;
        let certified = rcgen::generate_simple_self_signed(vec![
            "localhost".to_string(),
            "127.0.0.1".to_string(),
        ])
        .map_err(|e| format!("rcgen self-signed failed: {e}"))?;
        std::fs::write(&cert_path, certified.cert.pem())
            .map_err(|e| format!("write tls cert failed: {e}"))?;
        std::fs::write(&key_path, certified.key_pair.serialize_pem())
            .map_err(|e| format!("write tls key failed: {e}"))?;
        tracing::info!(dir = %dir.display(), "onchina self-signed TLS cert generated");
    }

    RustlsConfig::from_pem_file(cert_path, key_path)
        .await
        .map_err(|e| format!("load onchina tls cert failed: {e}"))
}
