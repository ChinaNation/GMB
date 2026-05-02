//! 公民身份业务模块。
//!
//! 公民端身份绑定凭证签发、链上绑定推送、状态扫码、wuminapp 投票账户登记
//! 等业务统一收口在此目录;phase23d 由 `operate/` 整体迁入。
//! handler / vote 子模块预留空骨架,后续 Phase 在公民身份业务扩展时落子。

pub(crate) mod binding;
pub(crate) mod cpms_qr;
pub(crate) mod handler;
pub(crate) mod status;
pub(crate) mod vote;
