//! DUOQIAN 链接收 SFID 机构信息的链上落点。
//!
//! 本目录只承接 SFID → DUOQIAN 的机构信息交互,例如第 1 步机构备案。
//! 它不能把备案记录直接写入 `duoqian-manage` 的正式机构 storage,
//! 也不能激活机构账户;正式多签机构注册仍由后续多签流程完成。

pub mod filing;
pub mod types;
pub mod validate;

#[cfg(test)]
mod tests;

pub use types::{InstitutionFilingPayload, InstitutionFilingRecord};
