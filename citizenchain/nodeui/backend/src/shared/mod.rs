// 共享基础层，承载跨页面复用的 RPC、输入校验与本地安全能力。
pub(crate) mod rpc;
pub(crate) mod security;
pub(crate) mod validation;
