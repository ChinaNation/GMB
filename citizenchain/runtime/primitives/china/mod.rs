pub mod china_cb;
pub mod china_ch;
pub mod china_jc;
pub mod china_jy;
pub mod china_lf;
pub mod china_sf;
pub mod china_zb;
pub mod china_zf;

/// 创世内置机构预派生地址对拍测试：用唯一派生原语
/// `core_const::derive_account` 重新派生主账户/费用账户地址，
/// 断言等于硬编码常量，防止派生协议与创世常量漂移。
#[cfg(test)]
mod derive_consistency_tests {
    use crate::core_const::{derive_account, OP_FEE, OP_MAIN, SS58_FORMAT};

    #[test]
    fn china_ch_main_fee_accounts_match_derive_primitive() {
        for n in super::china_ch::CHINA_CH {
            let cid = n.cid_number.as_bytes();
            assert_eq!(
                n.main_account,
                derive_account(OP_MAIN, SS58_FORMAT, cid),
                "省储行 {} 主账户派生漂移",
                n.cid_full_name
            );
            assert_eq!(
                n.fee_account,
                derive_account(OP_FEE, SS58_FORMAT, cid),
                "省储行 {} 费用账户派生漂移",
                n.cid_full_name
            );
        }
    }

    #[test]
    fn china_cb_main_fee_accounts_match_derive_primitive() {
        for n in super::china_cb::CHINA_CB {
            let cid = n.cid_number.as_bytes();
            assert_eq!(
                n.main_account,
                derive_account(OP_MAIN, SS58_FORMAT, cid),
                "储委会 {} 主账户派生漂移",
                n.cid_full_name
            );
            assert_eq!(
                n.fee_account,
                derive_account(OP_FEE, SS58_FORMAT, cid),
                "储委会 {} 费用账户派生漂移",
                n.cid_full_name
            );
        }
    }

    #[test]
    fn china_other_institutions_main_fee_match_derive_primitive() {
        macro_rules! check_arr {
            ($arr:expr, $label:expr) => {
                for n in $arr {
                    let cid = n.cid_number.as_bytes();
                    assert_eq!(
                        n.main_account,
                        derive_account(OP_MAIN, SS58_FORMAT, cid),
                        "{} {} 主账户派生漂移",
                        $label,
                        n.cid_full_name
                    );
                    assert_eq!(
                        n.fee_account,
                        derive_account(OP_FEE, SS58_FORMAT, cid),
                        "{} {} 费用账户派生漂移",
                        $label,
                        n.cid_full_name
                    );
                }
            };
        }
        check_arr!(super::china_zf::CHINA_ZF, "政府");
        check_arr!(super::china_lf::CHINA_LF, "立法");
        check_arr!(super::china_sf::CHINA_SF, "司法");
        check_arr!(super::china_jc::CHINA_JC, "检察");
        check_arr!(super::china_jy::CHINA_JY, "教育");
    }
}
