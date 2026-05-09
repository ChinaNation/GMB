// 管理员管理桌面端后端模块。
//
// 本目录只承载 AdminsChange pallet 相关的主体读取、管理员激活、管理员集合变更
// call 编码、QR 签名请求和提交逻辑；治理通用签名/RPC 能力继续复用上层公共模块。

pub mod activation;
pub mod call_data;
pub mod codec;
pub mod commands;
pub mod signing;
pub mod storage;
pub mod subject_id;
pub mod types;
pub mod validation;
