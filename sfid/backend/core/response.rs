//! 中文注释:HTTP API 通用响应 / 错误 / 健康检查输出包装。

use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct ApiResponse<T: Serialize> {
    pub(crate) code: u32,
    pub(crate) message: String,
    pub(crate) data: T,
}

#[derive(Serialize)]
pub(crate) struct PageResult<T: Serialize> {
    pub(crate) items: Vec<T>,
    pub(crate) page_size: usize,
    pub(crate) next_cursor: Option<String>,
    pub(crate) has_more: bool,
}

#[derive(Serialize)]
pub(crate) struct ApiError {
    pub(crate) code: u32,
    /// 中文注释:稳定业务错误码给前端判断逻辑使用;message 只给用户展示。
    pub(crate) error_code: &'static str,
    pub(crate) message: String,
    pub(crate) trace_id: String,
}

#[derive(Serialize)]
pub(crate) struct HealthData {
    pub(crate) service: &'static str,
    pub(crate) status: &'static str,
    pub(crate) checked_at: i64,
}
