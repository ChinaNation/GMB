//!  创世常量=genesis.rs

/// 一、创世宣言
pub const CITIZENS: &str = r#"先有人类后有国家，是公民建立国家，国家是公民的国家，是公民治理国家，而不是国家统治公民，公民没有爱国的义务；国家政权的建立其基本原则是保护公民的生命权、自由权、财产权、反抗压迫权和选举与被选举权不受任何的非法侵犯，当国家政权无法保证这一基本原则时，公民有权有义务推翻这个政权，建立一个以保障公民生命权、自由权、财产权、反抗压迫权和选举与被选举权为基本原则的政权。————《公民宪法》程伟"#;
pub const COUNTRY: &str = r#"中华民族联邦共和国国家名称是基于中华各民族悠久历史与璀璨文化的沉淀，全称为：中华民族联邦共和国，简称为：中华联邦；中华民族联邦共和国是致力于推行“公民主义”的———「公民治理国家（民治）、实现民主共和（民主）、保障公民权利（民权）、建设民生社会（民生）、复兴民族文化（民族）」———联邦制共和国。————《公民宪法》程伟"#;

/// 二、创世人口（单位：个）：1,443,497,378人
pub const GENESIS_CITIZEN_MAX: u64 = 1_443_497_378; // 中共国第7次人口普查的总人口数，作为创世人口数量

/// 三、创世发行（单位：分）：144,349,737,800.00 元 = 14_434_973_780_000 分
pub const GENESIS_ISSUANCE: u128 = 14_434_973_780_000; // 每人100元的创世发行总量，单位为分

/// 三之二、两和基金发行（单位：分）：195,818,501,966.00 元 = 19_581_850_196_600 分。
/// 两和基金 = 历史和解与和平建国基金，创世一次性增发到国储会两和基金账户(NRC_HE_ACCOUNT)，
/// 计入总供应量（独立于创世发行）。金额刻意编码 1958(大跃进)/1850(太平天国)/1966(文革)。
pub const HE_FUND_ISSUANCE: u128 = 19_581_850_196_600; // 195,818,501,966.00 元

use sp_std::vec::Vec;

/// 四、公民宪法 HTML 真源
///
/// `CitizenConstitution.html` 被编入 WASM；修改该 HTML 后必须发布 runtime 升级。
pub const CITIZEN_CONSTITUTION_HTML: &str = include_str!("CitizenConstitution.html");

// 五、公民宪法 Runtime API：runtime 暴露内置宪法 HTML 正文与其 blake2_256 摘要。
sp_api::decl_runtime_apis! {
    pub trait CitizenConstitutionApi {
        /// 返回当前链上 runtime 内置的公民宪法 HTML。
        fn citizen_constitution_html() -> Vec<u8>;

        /// 返回当前链上 runtime 内置公民宪法 HTML 的 blake2_256 摘要。
        fn citizen_constitution_blake2_256() -> [u8; 32];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genesis_issuance_matches_population() {
        // 中文注释：创世发行 = 创世人口 × 每人 100 元 × 100 分/元 = 人口 × 10_000。
        assert_eq!(GENESIS_ISSUANCE, GENESIS_CITIZEN_MAX as u128 * 10_000u128);
    }

    #[test]
    fn he_fund_issuance_matches_whitepaper() {
        // 中文注释：两和基金发行 = 195,818,501,966.00 元 × 100 分/元。
        assert_eq!(HE_FUND_ISSUANCE, 195_818_501_966u128 * 100);
    }
}
