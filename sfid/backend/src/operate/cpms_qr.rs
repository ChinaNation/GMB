#[allow(dead_code)]
pub(crate) fn canonical_citizen_qr_text(
    ver: &str,
    issuer_id: &str,
    site_sfid: &str,
    archive_no: &str,
    issued_at: i64,
    expire_at: i64,
    qr_id: &str,
    sig_alg: &str,
    status: &str,
) -> String {
    format!(
        "ver={ver}&issuer_id={issuer_id}&site_sfid={site_sfid}&archive_no={archive_no}&issued_at={issued_at}&expire_at={expire_at}&qr_id={qr_id}&sig_alg={sig_alg}&status={status}"
    )
}

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

#[allow(dead_code)]
pub(crate) fn verify_cpms_qr_signature(pubkeys: &[&str], message: &str, signature: &str) -> bool {
    pubkeys
        .iter()
        .any(|pk| crate::verify_admin_signature(pk, message, signature))
}
