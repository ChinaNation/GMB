//! CPMS 后端跨模块共享层（与前端 `common/` 对齐）。
//!
//! 放横切低层工具（ss58/限流/编码）与跨模块共享的响应封装、DTO、DB helper；
//! 业务逻辑各自有目录，不进 common。`pub(crate) use` 再导出让消费方统一写
//! `use crate::common::{...}`，与前端从 `common/` 导入一致。
pub mod admin;
pub mod audit;
pub mod encoding;
pub mod rate_limit;
pub mod response;
pub mod ss58;
pub mod types;

pub(crate) use admin::{find_admin_by_pubkey, find_admin_by_user_id};
pub(crate) use audit::write_audit;
pub(crate) use encoding::decode_bytes;
pub(crate) use response::{err, ok, ApiError, ApiResponse};
pub(crate) use types::Archive;
