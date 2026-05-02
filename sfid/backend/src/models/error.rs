//! 中文注释:HTTP API 通用响应 / 错误 / 健康检查输出包装。

use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct ApiResponse<T: Serialize> {
    pub(crate) code: u32,
    pub(crate) message: String,
    pub(crate) data: T,
}

#[derive(Serialize)]
pub(crate) struct ApiError {
    pub(crate) code: u32,
    pub(crate) message: String,
    pub(crate) trace_id: String,
}

#[derive(Serialize)]
pub(crate) struct HealthData {
    pub(crate) service: &'static str,
    pub(crate) status: &'static str,
    pub(crate) checked_at: i64,
}
