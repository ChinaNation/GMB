//! 公权机构全量确定性派生(创世直铸单源,ADR-031 卡3)。
//!
//! 「行政区(`china::area`)× 机构码模板(`official_template`)」纯派生:号由
//! `seed + generator` 确定性生成,名称由模板组装。genesis 落地存储与数量/格式断言
//! 共享本枚举,杜绝逻辑漂移。数量 = 国家两院 2 + 省部门 11×43 + 市级 17×市数 +
//! 镇级 14×镇数,与 china_*.rs 282 常量互不重号。

use alloc::string::String;

use crate::cid::china::area::{for_each_area, AreaItem};
use crate::cid::generator::{generate_cid_number, GenerateCidNumberInput};
use crate::cid::official_template::{
    OfficialOrgTemplate, CITY_TEMPLATES, NATIONAL_ASSEMBLY_TEMPLATES,
    PROVINCE_DEPARTMENT_TEMPLATES, TOWN_TEMPLATES,
};
use crate::cid::seed::official_institution_account_seed;

/// 创世直铸年份(固定,确定性):与 china_*.rs 常量号年份一致。
pub const GENESIS_INSTITUTION_YEAR: &str = "2026";
/// 国家参众议会落点省(与 onchina push_extra_national_targets 一致)。
pub const NATIONAL_ASSEMBLY_HOME_PROVINCE: &str = "中枢省";

/// 用模板 + 行政区确定性派生一个机构号(与 onchina official_institution_cid 同源)。
fn derive_template_cid(
    scope: &str,
    province_code: &str,
    city_code: &str,
    town_code: &str,
    template: &OfficialOrgTemplate,
    province_name: &str,
    city_name: &str,
) -> String {
    let seed = official_institution_account_seed(
        scope,
        province_code,
        city_code,
        town_code,
        template.institution_code,
    );
    generate_cid_number(GenerateCidNumberInput {
        account_pubkey: seed.as_str(),
        p1: "0",
        province_code,
        province_name,
        city_code,
        city_name,
        year: GENESIS_INSTITUTION_YEAR,
        institution: template.institution_code,
    })
    .unwrap_or_else(|e| {
        panic!(
            "genesis template cid 生成失败 code={} scope={scope}: {e}",
            template.institution_code
        )
    })
}

/// 枚举全部派生公权机构,对每个机构调用 `f(cid_number, cid_full_name, cid_short_name)`。
///
/// 遍历顺序与 `AREA_DATA` 字节序一致,确定性。省级部门落省主市、显示名=省名;
/// 国家两院落中枢省主市、显示名为空(模板 full_suffix 已含国名前缀);市级显示名=
/// 市名;镇级 generator 的 city_name 取市名(与 onchina 一致)、显示名=镇名。
pub fn for_each_public_institution<F>(mut f: F)
where
    F: FnMut(&str, &str, &str),
{
    let mut emit = |scope: &str,
                    province_code: &str,
                    city_code: &str,
                    town_code: &str,
                    template: &OfficialOrgTemplate,
                    province_name: &str,
                    city_name: &str,
                    display_area_name: &str| {
        let cid = derive_template_cid(
            scope,
            province_code,
            city_code,
            town_code,
            template,
            province_name,
            city_name,
        );
        f(
            cid.as_str(),
            template.full_name(display_area_name).as_str(),
            template.short_name(display_area_name).as_str(),
        );
    };

    for_each_area(|item| match item {
        AreaItem::Province {
            province_code,
            province_name,
            home_city_code,
            home_city_name,
        } => {
            for template in PROVINCE_DEPARTMENT_TEMPLATES {
                emit(
                    "PROVINCE",
                    province_code,
                    home_city_code,
                    "",
                    template,
                    province_name,
                    home_city_name,
                    province_name,
                );
            }
            if province_name == NATIONAL_ASSEMBLY_HOME_PROVINCE {
                for template in NATIONAL_ASSEMBLY_TEMPLATES {
                    emit(
                        "NATIONAL",
                        province_code,
                        home_city_code,
                        "",
                        template,
                        province_name,
                        home_city_name,
                        "",
                    );
                }
            }
        }
        AreaItem::City(city) => {
            for template in CITY_TEMPLATES {
                emit(
                    "CITY",
                    city.province_code,
                    city.city_code,
                    "",
                    template,
                    city.province_name,
                    city.city_name,
                    city.city_name,
                );
            }
        }
        AreaItem::Town(town) => {
            for template in TOWN_TEMPLATES {
                emit(
                    "TOWN",
                    town.province_code,
                    town.city_code,
                    town.town_code,
                    template,
                    town.province_name,
                    town.city_name,
                    town.town_name,
                );
            }
        }
    });
}

/// 派生机构总数(= 国家两院 2 + 省部门 11×省 + 市级 17×市 + 镇级 14×镇)。
pub fn public_institution_derived_count() -> usize {
    let (provinces, cities, towns) = crate::cid::china::area::area_counts();
    NATIONAL_ASSEMBLY_TEMPLATES.len()
        + PROVINCE_DEPARTMENT_TEMPLATES.len() * provinces as usize
        + CITY_TEMPLATES.len() * cities as usize
        + TOWN_TEMPLATES.len() * towns as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::collections::BTreeSet;
    use alloc::string::ToString;

    #[test]
    fn derived_count_matches_area_and_templates() {
        assert_eq!(
            public_institution_derived_count(),
            596_517,
            "派生机构总数 = 2 + 11×43 + 17×2872 + 14×39087"
        );
    }

    #[test]
    fn every_derived_number_is_valid_public_and_unique() {
        let expected = public_institution_derived_count();
        let mut count = 0usize;
        let mut seen = BTreeSet::<String>::new();
        for_each_public_institution(|cid, full, short| {
            count += 1;
            let parts = crate::cid::number::parse_cid_number_parts(cid)
                .unwrap_or_else(|e| panic!("派生号 {cid} 非法: {e}"));
            assert!(
                crate::cid::code::is_public_legal_code(&parts.institution),
                "派生号 {cid} 非公权家族"
            );
            assert!(!full.is_empty() && !short.is_empty(), "派生名不能为空");
            assert!(seen.insert(cid.to_string()), "派生号 {cid} 重复");
        });
        assert_eq!(count, expected, "枚举数量与推导一致");
    }
}
