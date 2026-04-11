use crate::login::{parse_sr25519_pubkey, parse_sr25519_pubkey_bytes};

pub(crate) fn normalize_admin_pubkey(input: &str) -> Option<String> {
    normalize_sr25519_pubkey(input)
}

#[allow(dead_code)]
pub(crate) fn normalize_cpms_pubkey(input: &str) -> Option<String> {
    normalize_sr25519_pubkey(input)
}

pub(crate) fn same_admin_pubkey(left: &str, right: &str) -> bool {
    same_sr25519_pubkey(left, right)
}

#[allow(dead_code)]
pub(crate) fn same_cpms_pubkey(left: &str, right: &str) -> bool {
    same_sr25519_pubkey(left, right)
}

fn normalize_sr25519_pubkey(input: &str) -> Option<String> {
    if let Some(hex_pubkey) = parse_sr25519_pubkey(input) {
        return Some(hex_pubkey);
    }
    if parse_sr25519_pubkey_bytes(input).is_some() {
        return Some(input.trim().to_string());
    }
    None
}

fn same_sr25519_pubkey(left: &str, right: &str) -> bool {
    match (parse_sr25519_pubkey(left), parse_sr25519_pubkey(right)) {
        (Some(l), Some(r)) => l.eq_ignore_ascii_case(r.as_str()),
        _ => left.trim().eq_ignore_ascii_case(right.trim()),
    }
}
