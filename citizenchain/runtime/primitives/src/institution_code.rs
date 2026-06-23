//! 机构码(CID institution_code)链上表示与治理分类 = institution_code.rs
//!
//! 中文注释(铁律):
//! 机构分类全链唯一真源是 CID 号里的**机构码**(backend `number/code.rs` 同一套 86 码)。
//! 链上治理统一用本文件的 `InstitutionCode`([u8; 4] 原始码字节)作机构分类。
//! 所有"用哪种阈值 / 是不是个人多签 / 是不是
//! 机构账户"的治理策略都由本文件的纯函数从机构码派生,绝不另立第二套分类。
//!
//! ## 字节表示
//! 机构码是 3~4 个大写 ASCII 字符。链上统一用 `[u8; 4]`,3 字符码右补 `0`:
//!   - `NRC` → `*b"NRC\0"`   `CGOV` → `*b"CGOV"`   `PMUL` → `*b"PMUL"`
//!
//! ## 治理策略派生
//!   - 固定治理档(NRC/PRC/PRB):制度阈值常量,账户校验走 china 内建表。
//!   - 个人多签(PMUL):动态阈值,管理员来自 personal-manage。
//!   - 机构账户(公权/私权法人):动态阈值,管理员来自 organization-manage。

/// 机构码链上表示:3~4 字符大写 ASCII,3 字符码右补 `0`。
pub type InstitutionCode = [u8; 4];

/// 由字符串机构码构造链上字节表示(3 字符右补 `0`)。
/// 非 std 环境也可用(const fn 仅做长度规整)。
pub const fn code_bytes(s: &str) -> InstitutionCode {
    let b = s.as_bytes();
    let mut out = [0u8; 4];
    let mut i = 0;
    while i < b.len() && i < 4 {
        out[i] = b[i];
        i += 1;
    }
    out
}

/// 从 CID 号(`R5-seg2-N9-D4`)解析机构码。
///
/// 机构码在第二段 seg2:3 字符码布局 = `码(3)+盈利位(1)+校验(1)`;
/// 4 字符码布局 = `码(4)+M1(1)`。靠 seg2 索引 3 区分(数字→3 字符,字母→4 字符,
/// 与 backend `validator.rs` 同规则)。china 内建机构据此从 cid_number 派生自身机构码。
pub fn institution_code_from_cid_number(cid_number: &str) -> Option<InstitutionCode> {
    let seg2 = cid_number.split('-').nth(1)?;
    let b = seg2.as_bytes();
    if b.len() < 4 {
        return None;
    }
    let code_len = if b[3].is_ascii_alphabetic() { 4 } else { 3 };
    let mut out = [0u8; 4];
    let mut i = 0;
    while i < code_len {
        out[i] = b[i];
        i += 1;
    }
    Some(out)
}

// ──────────────────────────────────────────────────────────────────
// 治理相关机构码常量(链上需要直接识别的少数码)
// ──────────────────────────────────────────────────────────────────

/// 国家公民储备委员会(固定治理档)。
pub const NRC: InstitutionCode = *b"NRC\0";
/// 省公民储备委员会(固定治理档)。
pub const PRC: InstitutionCode = *b"PRC\0";
/// 省公民储备银行(固定治理档)。
pub const PRB: InstitutionCode = *b"PRB\0";
/// 个人多签账户(不发号,仅链上/后端分类常量)。
pub const PMUL: InstitutionCode = *b"PMUL";

// ──────────────────────────────────────────────────────────────────
// 机构码分类清单(与 backend number/code.rs 同源的 86 码)
// ──────────────────────────────────────────────────────────────────

/// 公权法人机构码(A 国家级 26 + B 省级 17 + C 市级 17 + D 镇级 10 + 公立大学/学校 2)= 72。
const PUBLIC_LEGAL_CODES: &[InstitutionCode] = &[
    // A 国家级单体(26)
    *b"PRS\0", *b"FSC\0", *b"FIB\0", *b"FSS\0", *b"FPR\0", *b"FRG\0", *b"MFA\0", *b"MDF\0",
    *b"MHS\0", *b"MCW\0", *b"MHU\0", *b"MAG\0", *b"MCM\0", *b"MFT\0", *b"MEN\0", *b"MTR\0",
    *b"NLG\0", *b"NJD\0", *b"NSP\0", *b"FAC\0", *b"FAU\0", *b"FIV\0", *b"NED\0", *b"NRC\0",
    *b"NSN\0", *b"NRP\0", // B 省级类型(17)
    *b"PGV\0", *b"PLG\0", *b"PJD\0", *b"PSP\0", *b"PRC\0", *b"PRB\0", *b"PDF\0", *b"PHS\0",
    *b"PCW\0", *b"PHU\0", *b"PAG\0", *b"PCM\0", *b"PFT\0", *b"PEN\0", *b"PTR\0", *b"PSN\0",
    *b"PRP\0", // C 市级类型(17)
    *b"CGOV", *b"CLEG", *b"CSUP", *b"CJUD", *b"CEDU", *b"CSLF", *b"CDEF", *b"CHSC", *b"CCWF",
    *b"CHUD", *b"CAGR", *b"CCOM", *b"CFIN", *b"CENR", *b"CTRN", *b"CREG", *b"CPOL",
    // D 镇级类型(10)
    *b"TGOV", *b"TCWF", *b"THUD", *b"TAGR", *b"TFIN", *b"TDEF", *b"THSC", *b"TCOM", *b"TENR",
    *b"TTRN", // 公立大学 / 公立学校
    *b"GUN\0", *b"GSCH",
];

/// 私权法人机构码(有限合伙/股权/股份/公益/注册协会 + 私立大学/学校)= 7。
const PRIVATE_LEGAL_CODES: &[InstitutionCode] = &[
    *b"SFLP", *b"SFGQ", *b"SFGF", *b"SFGY", *b"SFAS", *b"SUN\0", *b"SFSC",
];

/// 非法人机构码(个体经营/无限合伙/非法人组织)= 3。
const UNINCORPORATED_CODES: &[InstitutionCode] = &[*b"SFGT", *b"SFGP", *b"UNIN"];

/// 个人主体机构码(公民人/自然人/智能人)= 3。
const PERSON_CODES: &[InstitutionCode] = &[*b"CTZN", *b"NATP", *b"SMTP"];

// ──────────────────────────────────────────────────────────────────
// 治理策略派生(纯函数,链上唯一分类来源)
// ──────────────────────────────────────────────────────────────────

/// 是否为固定治理档机构码(国储会/省储会/省储行)。
/// 这三类阈值是永久治理常量,账户合法性走 china 内建表,不读动态阈值。
pub fn is_fixed_governance_code(code: &InstitutionCode) -> bool {
    matches!(*code, NRC | PRC | PRB)
}

/// 固定治理档机构码的制度阈值(国储会 13 / 省储会 6 / 省储行 6)。
pub fn fixed_governance_pass_threshold(code: &InstitutionCode) -> Option<u32> {
    use crate::count_const::{
        NRC_INTERNAL_THRESHOLD, PRB_INTERNAL_THRESHOLD, PRC_INTERNAL_THRESHOLD,
    };
    match *code {
        NRC => Some(NRC_INTERNAL_THRESHOLD),
        PRC => Some(PRC_INTERNAL_THRESHOLD),
        PRB => Some(PRB_INTERNAL_THRESHOLD),
        _ => None,
    }
}

/// 是否为个人多签账户机构码(PMUL)。管理员来自 personal-manage。
pub fn is_personal_code(code: &InstitutionCode) -> bool {
    *code == PMUL
}

/// 是否为公权法人机构码。
pub fn is_public_legal_code(code: &InstitutionCode) -> bool {
    PUBLIC_LEGAL_CODES.contains(code)
}

/// 是否为私权法人机构码。
pub fn is_private_legal_code(code: &InstitutionCode) -> bool {
    PRIVATE_LEGAL_CODES.contains(code)
}

/// 是否为机构账户机构码(公权/私权/非法人法人实体,经 organization-manage 注册多签)。
/// 个人/个人多签不算机构账户;
/// 固定治理档(NRC/PRC/PRB)是 china 内建创世账户,走固定治理路径,也不算 organization-manage 机构账户。
pub fn is_institution_code(code: &InstitutionCode) -> bool {
    !is_fixed_governance_code(code)
        && (is_public_legal_code(code)
            || is_private_legal_code(code)
            || UNINCORPORATED_CODES.contains(code))
}

/// 是否为个人主体机构码(公民人/自然人/智能人)。
pub fn is_person_code(code: &InstitutionCode) -> bool {
    PERSON_CODES.contains(code)
}

/// 是否为注册多签动态阈值账户机构码(个人多签 或 机构账户)。
/// 固定治理档不在内。
pub fn is_registered_multisig_code(code: &InstitutionCode) -> bool {
    is_personal_code(code) || is_institution_code(code)
}

/// 是否为内部投票支持的治理机构码(固定治理档 或 注册多签账户)。
pub fn is_valid_governance_code(code: &InstitutionCode) -> bool {
    is_fixed_governance_code(code) || is_registered_multisig_code(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_governance_thresholds_match_constants() {
        assert_eq!(fixed_governance_pass_threshold(&NRC), Some(13));
        assert_eq!(fixed_governance_pass_threshold(&PRC), Some(6));
        assert_eq!(fixed_governance_pass_threshold(&PRB), Some(6));
        assert_eq!(fixed_governance_pass_threshold(&PMUL), None);
        assert_eq!(fixed_governance_pass_threshold(b"CGOV"), None);
    }

    #[test]
    fn classification_buckets_are_disjoint_and_complete() {
        // 固定治理档 ⊂ 公权法人,但不算"注册多签动态账户"。
        assert!(is_fixed_governance_code(&NRC));
        assert!(!is_registered_multisig_code(&NRC));
        // 个人多签:动态、个人、非机构。
        assert!(is_personal_code(&PMUL));
        assert!(is_registered_multisig_code(&PMUL));
        assert!(!is_institution_code(&PMUL));
        // 机构账户:公权 + 私权。
        assert!(is_institution_code(b"CGOV"));
        assert!(is_institution_code(b"SFLP"));
        assert!(is_registered_multisig_code(b"CGOV"));
        // 个人主体不是机构账户也不是个人多签。
        assert!(is_person_code(b"CTZN"));
        assert!(!is_institution_code(b"CTZN"));
        assert!(!is_personal_code(b"CTZN"));
    }

    #[test]
    fn public_legal_table_has_72_codes() {
        assert_eq!(PUBLIC_LEGAL_CODES.len(), 72);
        assert_eq!(PRIVATE_LEGAL_CODES.len(), 7);
        assert_eq!(UNINCORPORATED_CODES.len(), 3);
        assert_eq!(PERSON_CODES.len(), 3);
    }

    #[test]
    fn code_bytes_pads_three_char() {
        assert_eq!(code_bytes("NRC"), *b"NRC\0");
        assert_eq!(code_bytes("CGOV"), *b"CGOV");
    }
}
