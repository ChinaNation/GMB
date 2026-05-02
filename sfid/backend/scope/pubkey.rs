//! sr25519 pubkey 规范化与等值比较
//!
//! 中文注释:本文件由 Phase 23c 从 `business/pubkey.rs` 物理搬迁而来。
//! 内部统一 0x 小写 hex,前端展示 SS58(prefix=2027),禁止混用——
//! 见 feedback_pubkey_format_rule.md。

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
