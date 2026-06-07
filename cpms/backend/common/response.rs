//! 统一 HTTP 响应与错误封装（≈ 前端 `common/http.ts`）。

use axum::{http::StatusCode, Json};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub(crate) struct ApiResponse<T>
where
    T: Serialize,
{
    code: i32,
    message: String,
    data: Option<T>,
}

#[derive(Serialize)]
pub(crate) struct ApiError {
    code: i32,
    /// 中文注释:稳定业务错误码供前端判断;数字 code 只保留为内部分类编号。
    error_code: &'static str,
    message: String,
    trace_id: String,
}

pub(crate) fn ok<T: Serialize>(data: T) -> ApiResponse<T> {
    ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: Some(data),
    }
}

pub(crate) fn err(status: StatusCode, code: i32, message: &str) -> (StatusCode, Json<ApiError>) {
    (
        status,
        Json(ApiError {
            code,
            error_code: cpms_error_code(status, message),
            message: message.to_string(),
            trace_id: Uuid::new_v4().to_string(),
        }),
    )
}

fn cpms_error_code(status: StatusCode, message: &str) -> &'static str {
    // 中文注释:CPMS 是离线实名系统,错误码只描述本机认证、档案、签发和审计状态。
    match message {
        "missing session cookie" => "CPMS_AUTH_MISSING_SESSION",
        "invalid session" => "CPMS_AUTH_INVALID_SESSION",
        "session expired" => "CPMS_AUTH_SESSION_EXPIRED",
        "permission denied" => "CPMS_AUTH_PERMISSION_DENIED",
        "admin_pubkey not found" => "CPMS_AUTH_ADMIN_NOT_FOUND",
        "admin user not found" => "CPMS_AUTH_ADMIN_NOT_FOUND",
        "challenge not found" => "CPMS_AUTH_CHALLENGE_NOT_FOUND",
        "challenge already consumed" => "CPMS_AUTH_CHALLENGE_CONSUMED",
        "challenge expired" => "CPMS_AUTH_CHALLENGE_EXPIRED",
        "challenge pubkey mismatch" | "challenge session mismatch" => {
            "CPMS_AUTH_CHALLENGE_MISMATCH"
        }
        "signature verify failed" => "CPMS_AUTH_SIGNATURE_VERIFY_FAILED",
        "archive not found" => "CPMS_INTAKE_ARCHIVE_NOT_FOUND",
        "address area not found" => "CPMS_INTAKE_ADDRESS_AREA_NOT_FOUND",
        "archive_no conflict, retry exhausted" => "CPMS_INTAKE_ARCHIVE_DUPLICATED",
        "passport_no conflict, retry exhausted" => "CPMS_INTAKE_PASSPORT_DUPLICATED",
        "passport_no capacity exhausted" => "CPMS_INTAKE_PASSPORT_CAPACITY_EXHAUSTED",
        "invalid passport province_code" | "invalid passport city_code" => {
            "CPMS_INTAKE_PASSPORT_AREA_INVALID"
        }
        "invalid citizen_status" => "CPMS_INTAKE_CITIZEN_STATUS_INVALID",
        "annual status export required" => "CPMS_ANNUAL_STATUS_EXPORT_REQUIRED",
        "annual status export not required" => "CPMS_ANNUAL_STATUS_EXPORT_NOT_REQUIRED",
        "qr encode failed" => "CPMS_ISSUE_QR_GENERATE_FAILED",
        "save print record failed" => "CPMS_AUDIT_WRITE_FAILED",
        "archive wallet required" => "CPMS_ARCHIVE_WALLET_REQUIRED",
        "invalid wallet_address" => "CPMS_ARCHIVE_WALLET_ADDRESS_INVALID",
        "wallet already bound" => "CPMS_ARCHIVE_WALLET_ALREADY_BOUND",
        "archive already deleted" => "CPMS_ARCHIVE_ALREADY_DELETED",
        "delete challenge not found" => "CPMS_ARCHIVE_DELETE_CHALLENGE_NOT_FOUND",
        "delete challenge already consumed" => "CPMS_ARCHIVE_DELETE_CHALLENGE_CONSUMED",
        "delete challenge expired" => "CPMS_ARCHIVE_DELETE_CHALLENGE_EXPIRED",
        "delete challenge mismatch" => "CPMS_ARCHIVE_DELETE_CHALLENGE_MISMATCH",
        "delete signer mismatch" => "CPMS_ARCHIVE_DELETE_SIGNER_MISMATCH",
        "delete signature verify failed" => "CPMS_ARCHIVE_DELETE_SIGNATURE_INVALID",
        "delete payload hash mismatch" => "CPMS_ARCHIVE_DELETE_PAYLOAD_HASH_MISMATCH",
        "material not found" => "CPMS_ARCHIVE_MATERIAL_NOT_FOUND",
        "material file not found" => "CPMS_ARCHIVE_MATERIAL_FILE_NOT_FOUND",
        "material type invalid" => "CPMS_ARCHIVE_MATERIAL_TYPE_INVALID",
        "material mime invalid" => "CPMS_ARCHIVE_MATERIAL_MIME_INVALID",
        "material file required" => "CPMS_ARCHIVE_MATERIAL_FILE_REQUIRED",
        "material file too large" => "CPMS_ARCHIVE_MATERIAL_FILE_TOO_LARGE",
        "material file empty" => "CPMS_ARCHIVE_MATERIAL_FILE_EMPTY",
        "invalid admin role" => "CPMS_ADMIN_ROLE_INVALID",
        "admin name required" => "CPMS_ADMIN_NAME_REQUIRED",
        "admin name too long" => "CPMS_ADMIN_NAME_TOO_LONG",
        "super admin limit reached" => "CPMS_ADMIN_SUPER_ADMIN_LIMIT_REACHED",
        "initial super admin cannot be deleted" => "CPMS_ADMIN_INITIAL_SUPER_ADMIN_IMMUTABLE",
        "admin not found" => "CPMS_ADMIN_NOT_FOUND",
        "admin_pubkey already exists" => "CPMS_ADMIN_PUBKEY_DUPLICATED",
        "too many requests" => "CPMS_RATE_LIMITED",
        _ if status == StatusCode::UNAUTHORIZED => "CPMS_AUTH_UNAUTHORIZED",
        _ if status == StatusCode::FORBIDDEN => "CPMS_AUTH_FORBIDDEN",
        _ if status == StatusCode::BAD_REQUEST => "CPMS_REQUEST_INVALID",
        _ if status == StatusCode::NOT_FOUND => "CPMS_RESOURCE_NOT_FOUND",
        _ if status == StatusCode::CONFLICT => "CPMS_RESOURCE_CONFLICT",
        _ if status == StatusCode::GONE => "CPMS_RESOURCE_EXPIRED",
        _ if status == StatusCode::UNPROCESSABLE_ENTITY => "CPMS_BUSINESS_VALIDATION_FAILED",
        _ if status == StatusCode::SERVICE_UNAVAILABLE => "CPMS_SERVICE_UNAVAILABLE",
        _ if status == StatusCode::LOCKED => "CPMS_RESOURCE_LOCKED",
        _ if status == StatusCode::TOO_MANY_REQUESTS => "CPMS_RATE_LIMITED",
        _ => "CPMS_INTERNAL_ERROR",
    }
}
