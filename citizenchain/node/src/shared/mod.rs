// 共享基础层，承载跨页面复用的常量、RPC、输入校验、本地安全能力与 keystore 操作。
pub(crate) mod constants;
pub(crate) mod keystore;
pub(crate) mod rpc;
pub(crate) mod security;
pub(crate) mod sfid_config;
pub(crate) mod validation;
