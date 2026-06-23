pub mod china_cb;
pub mod china_ch;
pub mod china_jc;
pub mod china_jy;
pub mod china_lf;
pub mod china_sf;
pub mod china_zb;
pub mod china_zf;

/// 中文注释:内置机构名称四元组的 runtime 指纹。
///
/// `china_*.rs` 中的 `cid_full_name/cid_short_name/cid_full_name_en/cid_short_name_en`
/// 是链上防改锚点:CID 可以修改自己的投影名称,但这些内置机构名称要在链上生效必须随
/// runtime 升级。本函数把四个名称字段都纳入 runtime API,避免出现绕过内置常量的第二套实现。
pub fn builtin_institution_name_digest() -> [u8; 32] {
    let mut digest = [0x47u8; 32];
    fold_builtin_names(&mut digest, china_zf::CHINA_ZF);
    fold_builtin_names(&mut digest, china_lf::CHINA_LF);
    fold_builtin_names(&mut digest, china_sf::CHINA_SF);
    fold_builtin_names(&mut digest, china_jc::CHINA_JC);
    fold_builtin_names(&mut digest, china_jy::CHINA_JY);
    fold_builtin_names(&mut digest, china_cb::CHINA_CB);
    fold_builtin_names(&mut digest, china_ch::CHINA_CH);
    digest
}

sp_api::decl_runtime_apis! {
    pub trait BuiltinInstitutionNameApi {
        /// 返回当前 runtime 内置机构名称四元组的指纹。
        fn builtin_institution_name_digest() -> [u8; 32];
    }
}

fn fold_builtin_names<T: BuiltinInstitutionName>(digest: &mut [u8; 32], items: &[T]) {
    for item in items {
        fold_name_quad(
            digest,
            item.cid_full_name(),
            item.cid_short_name(),
            item.cid_full_name_en(),
            item.cid_short_name_en(),
        );
    }
}

fn fold_name_quad(
    digest: &mut [u8; 32],
    cid_full_name: &str,
    cid_short_name: &str,
    cid_full_name_en: &str,
    cid_short_name_en: &str,
) {
    fold_bytes(digest, cid_full_name.as_bytes());
    fold_bytes(digest, &[0]);
    fold_bytes(digest, cid_short_name.as_bytes());
    fold_bytes(digest, &[1]);
    fold_bytes(digest, cid_full_name_en.as_bytes());
    fold_bytes(digest, &[2]);
    fold_bytes(digest, cid_short_name_en.as_bytes());
    fold_bytes(digest, &[255]);
}

fn fold_bytes(digest: &mut [u8; 32], bytes: &[u8]) {
    for (index, byte) in bytes.iter().copied().enumerate() {
        let slot = index & 31;
        let mixed = byte.wrapping_add((index as u8).wrapping_mul(17));
        digest[slot] = digest[slot]
            .wrapping_mul(31)
            .wrapping_add(mixed)
            .rotate_left(((index as u32) & 7) + 1);
        digest[(slot * 7 + 3) & 31] ^= mixed.wrapping_mul(13);
    }
}

trait BuiltinInstitutionName {
    fn cid_full_name(&self) -> &'static str;
    fn cid_short_name(&self) -> &'static str;
    fn cid_full_name_en(&self) -> &'static str;
    fn cid_short_name_en(&self) -> &'static str;
}

macro_rules! impl_builtin_institution_name {
    ($ty:path) => {
        impl BuiltinInstitutionName for $ty {
            fn cid_full_name(&self) -> &'static str {
                self.cid_full_name
            }

            fn cid_short_name(&self) -> &'static str {
                self.cid_short_name
            }

            fn cid_full_name_en(&self) -> &'static str {
                self.cid_full_name_en
            }

            fn cid_short_name_en(&self) -> &'static str {
                self.cid_short_name_en
            }
        }
    };
}

impl_builtin_institution_name!(china_zf::ChinaZf);
impl_builtin_institution_name!(china_lf::ChinaLf);
impl_builtin_institution_name!(china_sf::ChinaSf);
impl_builtin_institution_name!(china_jc::ChinaJc);
impl_builtin_institution_name!(china_jy::ChinaJy);
impl_builtin_institution_name!(china_cb::ChinaCb);
impl_builtin_institution_name!(china_ch::ChinaCh);

/// 创世内置机构预派生地址对拍测试：用唯一派生入口
/// `account_derive::AccountKind::{InstitutionMain,InstitutionFee}.derive` 重新派生
/// 主账户/费用账户地址，断言等于硬编码常量，防止派生协议与创世常量漂移。
#[cfg(test)]
mod derive_consistency_tests {
    use crate::account_derive::AccountKind;
    use crate::core_const::SS58_FORMAT;

    #[test]
    fn china_ch_main_fee_accounts_match_derive_primitive() {
        for n in super::china_ch::CHINA_CH {
            let cid = n.cid_number.as_bytes();
            assert_eq!(
                n.main_account,
                AccountKind::InstitutionMain { cid_number: cid }.derive(SS58_FORMAT),
                "省储行 {} 主账户派生漂移",
                n.cid_full_name
            );
            assert_eq!(
                n.fee_account,
                AccountKind::InstitutionFee { cid_number: cid }.derive(SS58_FORMAT),
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
                AccountKind::InstitutionMain { cid_number: cid }.derive(SS58_FORMAT),
                "储委会 {} 主账户派生漂移",
                n.cid_full_name
            );
            assert_eq!(
                n.fee_account,
                AccountKind::InstitutionFee { cid_number: cid }.derive(SS58_FORMAT),
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
                        AccountKind::InstitutionMain { cid_number: cid }.derive(SS58_FORMAT),
                        "{} {} 主账户派生漂移",
                        $label,
                        n.cid_full_name
                    );
                    assert_eq!(
                        n.fee_account,
                        AccountKind::InstitutionFee { cid_number: cid }.derive(SS58_FORMAT),
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
        check_arr!(super::china_jc::CHINA_JC, "监察");
        check_arr!(super::china_jy::CHINA_JY, "教育");
    }

    #[test]
    fn builtin_institution_name_have_runtime_digest() {
        let digest = super::builtin_institution_name_digest();
        assert_ne!(digest, [0u8; 32], "内置机构名称四元组 runtime 指纹不可为空");
        assert_eq!(
            digest,
            super::builtin_institution_name_digest(),
            "内置机构名称四元组 runtime 指纹必须稳定"
        );
    }

    #[test]
    fn builtin_institution_name_values_are_present() {
        macro_rules! check_arr {
            ($arr:expr, $label:expr) => {
                for n in $arr {
                    assert!(
                        !n.cid_full_name.trim().is_empty(),
                        "{} 缺少 cid_full_name",
                        $label
                    );
                    assert!(
                        !n.cid_short_name.trim().is_empty(),
                        "{} {} 缺少 cid_short_name",
                        $label,
                        n.cid_full_name
                    );
                    assert!(
                        !n.cid_full_name_en.trim().is_empty(),
                        "{} {} 缺少 cid_full_name_en",
                        $label,
                        n.cid_full_name
                    );
                    assert!(
                        !n.cid_short_name_en.trim().is_empty(),
                        "{} {} 缺少 cid_short_name_en",
                        $label,
                        n.cid_full_name
                    );
                }
            };
        }
        check_arr!(super::china_zf::CHINA_ZF, "政府");
        check_arr!(super::china_lf::CHINA_LF, "立法");
        check_arr!(super::china_sf::CHINA_SF, "司法");
        check_arr!(super::china_jc::CHINA_JC, "监察");
        check_arr!(super::china_jy::CHINA_JY, "教育");
        check_arr!(super::china_cb::CHINA_CB, "储委会");
        check_arr!(super::china_ch::CHINA_CH, "储行");
    }
}
