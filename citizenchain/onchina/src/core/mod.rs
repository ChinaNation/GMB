/// 中文注释:跨业务复用的链上凭证签名、SCALE payload 与 genesis hash 对齐工具。
pub(crate) mod chain_runtime;
/// 中文注释:链 RPC URL 统一读取入口,业务模块不得直接读环境变量。
pub(crate) mod chain_url;
/// 中文注释:PostgreSQL 连接池和当前结构化 schema 初始化。
pub(crate) mod db;
/// 中文注释:内嵌私有 PostgreSQL 生命周期(onchina 自管;Card 05 零依赖部署)。
pub(crate) mod embedded_pg;
pub(crate) mod http_security;
/// 中文注释:propose_create_institution 裸 SCALE call data 编码器(onchina 唯一真源)。
pub(crate) mod institution_call;
/// 中文注释:QR_V1 协议和链上中国平台签名二维码构造。
#[allow(dead_code)]
pub(crate) mod qr;
/// 中文注释:HTTP API 通用响应、分页和健康检查输出模型。
pub(crate) mod response;
pub(crate) mod runtime_ops;
/// 中文注释:敏感字符串封装,只服务密码学代码短暂读取。
pub(crate) mod secret;
/// 中文注释:onchina 内网 API 自签 TLS(Card 05;rcgen 自签 + rustls)。
pub(crate) mod tls;
