use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub code: u32,
    pub message: &'static str,
    pub data: T,
}

#[derive(Serialize)]
pub struct HealthData {
    pub service: &'static str,
    pub version: &'static str,
    pub status: &'static str,
}
