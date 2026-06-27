#![allow(dead_code)]

//! CID 机构码引用层。
//!
//! 中文注释:
//! - 机构码常量、机构码对应 `cid_short_name`、盈利策略和行政层级的唯一真源在
//!   `citizenchain/runtime/primitives/src/code.rs`。
//! - 本文件只保留 CID number 模块需要的薄封装,继续服务 CID 号生成、解析、校验。
//! - 不得在 CID 系统内恢复第二份机构码枚举或第二份机构码数组。

pub use primitives::code::{
    AdminLevel, InstitutionCode, InstitutionCodeInfo, ProfitPolicy, ALL_CODES,
    INSTITUTION_CODE_INFOS, NRC, PMUL, PRB, PRC,
};

/// 全部 92 个机构码,顺序与 primitives 的 `INSTITUTION_CODE_INFOS` 一致。
pub const ALL: [InstitutionCode; 92] = ALL_CODES;

/// 从 3/4 字符机构码或机构实体中文简称解析机构码。
pub fn from_str(value: &str) -> Option<InstitutionCode> {
    primitives::code::institution_code_from_str(value)
}

/// 返回 CID 号里使用的 3 或 4 字符机构码。
pub fn as_code(code: &InstitutionCode) -> &'static str {
    primitives::code::institution_code_text(code).expect("known CID institution code")
}

/// 机构码对应的机构实体中文简称。
pub fn cid_short_name(code: &InstitutionCode) -> &'static str {
    primitives::code::cid_short_name(code).expect("known CID institution code")
}

/// 机构码字符长度(3 = 国家/省部/大学布局,4 = 市镇/私权/个人布局)。
pub fn code_len(code: &InstitutionCode) -> usize {
    primitives::code::institution_code_len(code).expect("known CID institution code")
}

/// 是否为 3 字符码。
pub fn is_three_char(code: &InstitutionCode) -> bool {
    primitives::code::is_three_char_code(code)
}

/// 盈利策略。
pub fn profit_policy(code: &InstitutionCode) -> ProfitPolicy {
    primitives::code::profit_policy(code).expect("known CID institution code")
}

/// 个人主体(公民人/自然人/智能人)。
pub fn is_person(code: &InstitutionCode) -> bool {
    primitives::code::is_person_code(code)
}

/// 非法人(个体经营/无限合伙/非法人组织)。
pub fn is_unincorporated(code: &InstitutionCode) -> bool {
    primitives::code::is_unincorporated_code(code)
}

/// 私法人(有限合伙/股权/股份/公益/协会/私立大学/私立学校)。
pub fn is_private_legal(code: &InstitutionCode) -> bool {
    primitives::code::is_private_legal_code(code)
}

/// 公法人(国家/省部/市镇公权机构、委员会、公立大学/学校)。
pub fn is_public_legal(code: &InstitutionCode) -> bool {
    primitives::code::is_public_legal_code(code)
}

/// 是否教育机构(公私大学/学校)。
pub fn is_education_institution(code: &InstitutionCode) -> bool {
    primitives::code::is_education_institution_code(code)
}

/// 是否基础教育学校(初学/小学/中学),需要 education_type 级别字段。
pub fn requires_education_level(code: &InstitutionCode) -> bool {
    primitives::code::requires_education_level(code)
}

/// 机构码所属行政层级。
pub fn admin_level(code: &InstitutionCode) -> Option<AdminLevel> {
    primitives::code::admin_level(code)
}

/// 是否市公安局(== CPOL)。
pub fn is_city_police(code: &InstitutionCode) -> bool {
    primitives::code::is_city_police_code(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_codes_are_three_or_four_ascii_upper() {
        for code in ALL {
            let text = as_code(&code);
            assert!(
                text.len() == 3 || text.len() == 4,
                "{text} must be 3 or 4 chars"
            );
            assert!(
                text.chars().all(|ch| ch.is_ascii_uppercase()),
                "{text} must be ascii uppercase"
            );
        }
    }

    #[test]
    fn parse_code_and_cid_short_name() {
        assert_eq!(from_str("NRC"), Some(NRC));
        assert_eq!(from_str("国家公民储备委员会"), Some(NRC));
        assert_eq!(from_str("SFGQ"), Some(*b"SFGQ"));
        assert_eq!(from_str("xyz"), None);
    }

    #[test]
    fn profit_policy_and_category_spot_check() {
        assert_eq!(profit_policy(b"SFGQ"), ProfitPolicy::Profit);
        assert_eq!(profit_policy(b"SFGY"), ProfitPolicy::NonProfit);
        assert_eq!(profit_policy(b"SFAS"), ProfitPolicy::Variable);
        assert_eq!(profit_policy(b"SMTP"), ProfitPolicy::Variable);
        assert_eq!(profit_policy(b"UNIN"), ProfitPolicy::InheritParent);

        assert!(is_unincorporated(b"SFGT"));
        assert!(is_private_legal(b"SFGQ"));
        assert!(is_person(b"CTZN"));
        assert!(is_public_legal(&NRC));
        assert!(!is_public_legal(&PMUL) && !is_person(&PMUL));
    }

    #[test]
    fn all_codes_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for code in ALL {
            assert!(
                seen.insert(as_code(&code)),
                "duplicate code {}",
                as_code(&code)
            );
        }
        assert_eq!(seen.len(), 92);
    }
}
