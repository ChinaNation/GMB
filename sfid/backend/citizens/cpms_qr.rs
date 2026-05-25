//! CPMS 站点公民状态扫码 QR canonical 文本拼装工具。
//!
//! 旧 CPMS QR 签名链路已废弃(SFID-CPMS QR v1 走 archive/verify 验真端点);
//! 此处仅保留状态扫码仍需复用的 canonical 文本生成函数,供 `status.rs` 复用。

pub(crate) fn canonical_status_qr_text(
    ver: &str,
    issuer_id: &str,
    site_sfid: &str,
    archive_no: &str,
    status: &str,
    issued_at: i64,
    expire_at: i64,
    qr_id: &str,
    sig_alg: &str,
) -> String {
    format!(
        "ver={ver}&issuer_id={issuer_id}&site_sfid={site_sfid}&archive_no={archive_no}&status={status}&issued_at={issued_at}&expire_at={expire_at}&qr_id={qr_id}&sig_alg={sig_alg}"
    )
}
