//! 治理主体 ID(48 字节 SubjectId)的统一派生协议。
//!
//! ## 永久 ABI 协议(SubjectKind kind tag + payload)
//!
//! ```text
//! [u8; 48] 布局:
//!   byte[0]:    kind tag(SubjectKind 枚举字节值)
//!     0x01 = Builtin           (内置主体:NRC/PRC/PRB,共 44 条 china_cb/china_ch)
//!     0x02 = SfidInstitution   (SFID 注册机构,任意账户经 sfid_id 派生)
//!     0x03 = PersonalDuoqian   (个人多签,creator+account_name 派生地址)
//!     0xFF = Reserved          (协议升级哨兵)
//!     其他  = 非法,parse 返回 None
//!   byte[1..48]: payload (47B)
//!     Builtin:           shenfen_id 字节(≤47B)右填零
//!     SfidInstitution:   sfid_id 字节(≤47B)右填零
//!     PersonalDuoqian:   32B AccountId + 15B 零填充
//! ```
//!
//! 三类主体永远不会撞 key(kind tag 不同),取代 A 阶段"全部右填零、ASCII 撞 key"
//! 的弱协议。链未上线时(2026-05-06)做协议统一,fresh genesis 即生效。
//!
//! ## 调用层
//!
//! - 通用构造:`build_subject_id(kind, payload)`
//! - 反向解析:`parse_subject_id(id) -> (kind, payload)`
//! - 语义 helper:
//!   - `subject_id_from_account(account)` → PersonalDuoqian
//!   - `subject_id_from_sfid_id(sfid_id)` → SfidInstitution
//!   - `subject_id_from_shenfen_id(shenfen_id)` → Builtin

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::DecodeWithMemTracking;
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

/// 治理主体类型枚举。kind tag 字节值是永久 ABI 协议,不可变更。
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    Copy,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
    PartialEq,
    Eq,
)]
pub enum SubjectKind {
    /// 内置主体:国储会 / 省储会 / 省储行(china::china_cb / china_ch)。
    Builtin = 0x01,
    /// SFID 注册机构(任意机构账户经 sfid_id 派生)。
    SfidInstitution = 0x02,
    /// 个人多签(creator + account_name 派生地址)。
    PersonalDuoqian = 0x03,
}

/// 通用构造 SubjectId(`[u8; 48]`)。
///
/// payload 长度限制 1..=47B;超长或为空返回 None。
pub fn build_subject_id(kind: SubjectKind, payload: &[u8]) -> Option<[u8; 48]> {
    if payload.is_empty() || payload.len() > 47 {
        return None;
    }
    let mut id = [0u8; 48];
    id[0] = kind as u8;
    id[1..1 + payload.len()].copy_from_slice(payload);
    Some(id)
}

/// 反向解析:从 institution_id 取 (kind, payload)。
///
/// payload 已去掉尾部零填充;非法 kind 字节返回 None。
pub fn parse_subject_id(id: &[u8; 48]) -> Option<(SubjectKind, &[u8])> {
    let kind = match id[0] {
        0x01 => SubjectKind::Builtin,
        0x02 => SubjectKind::SfidInstitution,
        0x03 => SubjectKind::PersonalDuoqian,
        _ => return None,
    };
    // 找 payload 实际有效长度(去掉尾部零填充)
    let trimmed_end = id[1..]
        .iter()
        .rposition(|&b| b != 0)
        .map(|p| p + 1)
        .unwrap_or(0);
    Some((kind, &id[1..1 + trimmed_end]))
}

/// 个人多签派生:`PersonalDuoqian` kind + 32B AccountId + 15B 零。
///
/// AccountId encode 后取前 32B(项目内 AccountId32 实测就是 32B)。
/// 32B ≤ 47B,build_subject_id 永远成功;此处 expect 是不变量保证。
pub fn subject_id_from_account<AccountId: Encode>(account: &AccountId) -> [u8; 48] {
    let encoded = account.encode();
    let copy_len = core::cmp::min(encoded.len(), 32);
    build_subject_id(SubjectKind::PersonalDuoqian, &encoded[..copy_len])
        .expect("32B AccountId ≤ 47B payload, infallible")
}

/// SFID 注册机构派生:`SfidInstitution` kind + sfid_id 字节(≤47B)右填零。
///
/// MaxSfidIdLength 在 runtime config 强制 ≤47;BoundedVec 入链已守门。
/// sfid_id 为空或超过 47B 返回 None,调用方应在 ensure! 拦截。
pub fn subject_id_from_sfid_id(sfid_id: &[u8]) -> Option<[u8; 48]> {
    build_subject_id(SubjectKind::SfidInstitution, sfid_id)
}

/// 内置主体派生:`Builtin` kind + shenfen_id ASCII 字节(≤47B)右填零。
///
/// 当前 china_cb/china_ch 实测 shenfen_id 长度 33B,远小于 47B。
/// shenfen_id 字符串为空或字节数超过 47 时返回 None。
pub fn subject_id_from_shenfen_id(shenfen_id: &str) -> Option<[u8; 48]> {
    build_subject_id(SubjectKind::Builtin, shenfen_id.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_id_starts_with_0x01() {
        let id = subject_id_from_shenfen_id("GFR-LN001-CB0C-617776487-20260222").unwrap();
        assert_eq!(id[0], 0x01);
        assert_eq!(&id[1..34], b"GFR-LN001-CB0C-617776487-20260222");
        // payload 长度 33B,后续应全零
        assert!(id[34..].iter().all(|&b| b == 0));
    }

    #[test]
    fn sfid_id_starts_with_0x02() {
        let sfid_id = b"CN-110000-XXXX";
        let id = subject_id_from_sfid_id(sfid_id).unwrap();
        assert_eq!(id[0], 0x02);
        assert_eq!(&id[1..1 + sfid_id.len()], sfid_id);
        assert!(id[1 + sfid_id.len()..].iter().all(|&b| b == 0));
    }

    #[test]
    fn personal_duoqian_id_starts_with_0x03() {
        // 32B 假账户
        let account_bytes: [u8; 32] = [0x7Fu8; 32];
        let id = subject_id_from_account(&account_bytes);
        assert_eq!(id[0], 0x03);
        assert_eq!(&id[1..33], &account_bytes);
        // payload 32B 后,剩 15B 零(byte[33..48])
        assert!(id[33..].iter().all(|&b| b == 0));
    }

    #[test]
    fn three_kinds_never_collide() {
        // 三类主体即使 payload 字节内容相同(32B 全 1),institution_id 也因 kind 互斥。
        let same_payload = [0x11u8; 32];
        let a = build_subject_id(SubjectKind::Builtin, &same_payload).unwrap();
        let b = build_subject_id(SubjectKind::SfidInstitution, &same_payload).unwrap();
        let c = subject_id_from_account(&same_payload);
        assert_ne!(a[0], b[0]);
        assert_ne!(b[0], c[0]);
        assert_ne!(a[0], c[0]);
        assert_eq!(a[0], 0x01);
        assert_eq!(b[0], 0x02);
        assert_eq!(c[0], 0x03);
    }

    #[test]
    fn payload_length_47_max() {
        // 边界:47B payload 通过,48B 失败
        let payload_47 = [0xAAu8; 47];
        assert!(build_subject_id(SubjectKind::Builtin, &payload_47).is_some());
        let payload_48 = [0xAAu8; 48];
        assert!(build_subject_id(SubjectKind::Builtin, &payload_48).is_none());
        // 空 payload 拒绝
        assert!(build_subject_id(SubjectKind::Builtin, &[]).is_none());
    }

    #[test]
    fn parse_round_trip() {
        let payload = b"GFR-LN001-CB0C-617776487-20260222";
        let id = build_subject_id(SubjectKind::Builtin, payload).unwrap();
        let (kind, parsed_payload) = parse_subject_id(&id).unwrap();
        assert_eq!(kind, SubjectKind::Builtin);
        assert_eq!(parsed_payload, payload.as_slice());

        let account_bytes: [u8; 32] = [0xBBu8; 32];
        let pid = subject_id_from_account(&account_bytes);
        let (pkind, ppayload) = parse_subject_id(&pid).unwrap();
        assert_eq!(pkind, SubjectKind::PersonalDuoqian);
        assert_eq!(ppayload, &account_bytes);
    }

    #[test]
    fn parse_rejects_invalid_kind() {
        // 非法 kind 字节(0x00 / 0x04 / 0xFF 等保留位)拒绝
        let id_zero = [0u8; 48];
        assert!(parse_subject_id(&id_zero).is_none());
        let mut id_invalid = [0u8; 48];
        id_invalid[0] = 0x05;
        assert!(parse_subject_id(&id_invalid).is_none());
        // 0xFF 是协议升级哨兵,当前未启用,parse 拒绝
        let mut id_reserved = [0u8; 48];
        id_reserved[0] = 0xFF;
        assert!(parse_subject_id(&id_reserved).is_none());
    }
}
