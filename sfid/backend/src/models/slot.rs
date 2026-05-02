//! 中文注释:省管理员槽位枚举 `Slot { Main, Backup1, Backup2 }`。
//! 真正定义在 `crate::sfid::province::Slot`,这里只做 facade 再导出,
//! 让 `crate::models::Slot` 路径与其它领域类型保持一致(Phase 23 progress 写入)。

#[allow(unused_imports)]
pub(crate) use crate::sfid::province::Slot;
