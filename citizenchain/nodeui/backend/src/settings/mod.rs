// 设置模块入口，聚合钱包地址、引导节点、投票节点和设备密码子模块。
pub(crate) mod address_utils;
#[path = "bootnodes-address/mod.rs"]
pub mod bootnodes_address;
#[path = "device-password/mod.rs"]
pub(crate) mod device_password;
#[path = "fee-address/mod.rs"]
pub mod fee_address;
#[path = "grandpa-address/mod.rs"]
pub mod grandpa_address;
