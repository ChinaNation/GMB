//! 公民宪法 HTML 真源。
//!
//! 本文件只保存 runtime 需要暴露的宪法正文常量与 Runtime API 声明。
//! `CitizenConstitution.html` 被编入 WASM；修改该 HTML 后必须发布 runtime 升级。

use sp_std::vec::Vec;

/// 公民宪法完整 HTML。
pub const CITIZEN_CONSTITUTION_HTML: &str = include_str!("CitizenConstitution.html");

sp_api::decl_runtime_apis! {
    pub trait CitizenConstitutionApi {
        /// 返回当前链上 runtime 内置的公民宪法 HTML。
        fn citizen_constitution_html() -> Vec<u8>;

        /// 返回当前链上 runtime 内置公民宪法 HTML 的 blake2_256 摘要。
        fn citizen_constitution_blake2_256() -> [u8; 32];
    }
}
