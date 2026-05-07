//! 机构多签注销流程指针。
//!
//! 个人多签和机构多签共用同一条关闭逻辑(因 `resolve_admin_subject_for_account`
//! 自动按 storage 命中归属主体),业务体放在 `crate::close::do_propose_close`。
//! 本子模块不再持有独立实现,作为目录边界占位保留。
