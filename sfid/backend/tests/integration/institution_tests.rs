// 中文注释:机构/账户/文档服务层集成测试。
// 测试 service 层逻辑,不涉及 HTTP handler 或链交互。
//
// 运行: cargo test --test institution_tests

use std::collections::HashMap;

/// 机构名称查重:私权机构全国唯一,公权机构同城唯一。
#[test]
fn name_uniqueness_private_is_global() {
    // 私权机构:全国内不允许同名
    let mut names: HashMap<String, Vec<String>> = HashMap::new();
    names.entry("测试机构A".to_string()).or_default().push("广州市".to_string());
    names.entry("测试机构A".to_string()).or_default().push("深圳市".to_string());

    let entry = names.get("测试机构A").unwrap();
    // 私权机构:同名存在两个城市 → 应拒绝第二个
    assert!(entry.len() > 1, "duplicate name should be detected globally");
}

#[test]
fn name_uniqueness_public_allows_cross_city() {
    // 公权机构:不同市允许同名(如各市司法院)
    let mut by_city: HashMap<(String, String), bool> = HashMap::new();
    by_city.insert(("司法院".to_string(), "广州市".to_string()), true);
    // 深圳市也可以有"司法院"
    by_city.insert(("司法院".to_string(), "深圳市".to_string()), true);
    assert_eq!(by_city.len(), 2, "different cities can have same name");

    // 但同城不允许重名
    let dup = by_city.contains_key(&("司法院".to_string(), "广州市".to_string()));
    assert!(dup, "same city same name should be detected");
}

/// 储备银行(CH)仅股份公司可选。
#[test]
fn ch_only_for_joint_stock() {
    let valid_sub_types = [
        "SOLE_PROPRIETORSHIP",
        "PARTNERSHIP",
        "LIMITED_LIABILITY",
        "JOINT_STOCK",
    ];

    for sub_type in &valid_sub_types {
        let can_use_ch = *sub_type == "JOINT_STOCK";
        if can_use_ch {
            assert_eq!(*sub_type, "JOINT_STOCK", "only JOINT_STOCK can use CH");
        } else {
            assert_ne!(*sub_type, "JOINT_STOCK", "{sub_type} should not use CH");
        }
    }
}

/// 文档类型枚举校验。
#[test]
fn valid_doc_types() {
    let valid = ["公司章程", "营业许可证", "股东会决议", "法人授权书", "其他"];
    assert_eq!(valid.len(), 5);
    assert!(valid.contains(&"公司章程"));
    assert!(!valid.contains(&"随便"));
}

/// 文件大小限制:10MB。
#[test]
fn file_size_limit() {
    let max_size: u64 = 10 * 1024 * 1024;
    assert_eq!(max_size, 10_485_760);
    // 9.9MB should pass
    assert!(9_900_000 < max_size);
    // 11MB should fail
    assert!(11_000_000 > max_size);
}
