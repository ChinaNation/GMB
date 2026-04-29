//! 清算行 offchain 结算引擎子模块。
//!
//! 中文注释:
//! - 本目录只承载清算行节点运行后的结算链路:监听、打包、签名、提交。
//! - 注册清算行、SFID 查询、管理员解密等管理流程放在 `offchain` 根目录同级模块。

pub mod listener;
pub mod packer;
pub mod signer;
pub mod submitter;
