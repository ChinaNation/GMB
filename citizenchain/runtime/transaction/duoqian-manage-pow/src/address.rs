//! 多签账户地址派生入口。
//!
//! `DUOQIAN_V1` 已经把机构主账户、费用账户和自定义账户拆成不同 op_tag。
//! 本文件只保存地址角色语义，真正派生仍由 pallet 读取 runtime 的 SS58 前缀后完成。

/// 机构账户角色保留名：这两个中文字串必须强制走 Role::Main / Role::Fee，
/// 禁止被误当作 Named 命名账户落到 OP_INSTITUTION。
pub const RESERVED_NAME_MAIN: &[u8] = "主账户".as_bytes();
pub const RESERVED_NAME_FEE: &[u8] = "费用账户".as_bytes();

/// SFID 登记机构下的账户角色枚举，决定地址派生走哪个 op_tag：
/// - `Main`：所有机构的主账户，preimage 不含 account_name，走 `OP_MAIN = 0x00`。
/// - `Fee`：所有机构的费用账户，preimage 不含 account_name，走 `OP_FEE = 0x01`。
/// - `Named(account_name)`：SFID 机构自定义命名账户，走 `OP_INSTITUTION = 0x05`。
#[derive(Clone, Copy, Debug)]
pub enum InstitutionAccountRole<'a> {
    Main,
    Fee,
    Named(&'a [u8]),
}
