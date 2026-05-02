/// 中文注释:跨业务复用的链推送 helper,业务入口必须放各模块 `chain_*` 文件。
pub(crate) mod chain_client;
/// 中文注释:跨业务复用的链上凭证签名、SCALE payload 与 genesis hash 对齐工具。
pub(crate) mod chain_runtime;
/// 中文注释:链 RPC URL 统一读取入口,业务模块不得直接读环境变量。
pub(crate) mod chain_url;
pub(crate) mod http_security;
pub(crate) mod runtime_ops;
