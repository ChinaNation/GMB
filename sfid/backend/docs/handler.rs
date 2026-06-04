//! 机构资料库 HTTP handler
//!
//! 中文注释:本模块只承载机构资料上传、下载、查询和删除;
//! 机构新增归 private,公权确定性目录归 gov,账户归 accounts。
//!
//! ## 当前路由表(admin 端,login 中间件)
//!
//! - GET    /api/v1/institution/:sfid_number/documents        → list_documents
//! - POST   /api/v1/institution/:sfid_number/documents        → upload_document
//! - GET    /api/v1/institution/:sfid_number/documents/:doc_id/download → download_document
//! - DELETE /api/v1/institution/:sfid_number/documents/:doc_id → delete_document

#![allow(dead_code)]

use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::admins::actions::require_admin_security_grant;
use crate::admins::operation_auth::AdminActionType;
use crate::login::require_admin_any;
use crate::models::ApiResponse;
use crate::subjects::http::ensure_institution_visible_to_admin;
use crate::subjects::model::{InstitutionDocument, VALID_DOC_TYPES};
use crate::*;

// ─── 0. 机构名称查重(私权=全国唯一,公权=同城唯一) ──────────────

pub(crate) async fn list_documents(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(sfid_number): Path<String>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let store = match store_read_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let inst = match store.multisig_institutions.get(&sfid_number) {
        Some(v) => v,
        None => return api_error(StatusCode::NOT_FOUND, 1004, "institution not found"),
    };
    if let Err(resp) = ensure_institution_visible_to_admin(inst, &ctx) {
        return resp;
    }
    let mut docs: Vec<&InstitutionDocument> = store
        .institution_documents
        .values()
        .filter(|d| d.sfid_number == sfid_number)
        .collect();
    docs.sort_by(|a, b| b.uploaded_at.cmp(&a.uploaded_at));
    let owned: Vec<InstitutionDocument> = docs.into_iter().cloned().collect();
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: owned,
    })
    .into_response()
}

/// POST /api/v1/institution/:sfid_number/documents — 上传文档(multipart/form-data)
/// 字段: file(文件), doc_type(文档类型)
pub(crate) async fn upload_document(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(sfid_number): Path<String>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    // 校验机构存在 + scope 权限(SHENG 本省,SHI 本市)。
    {
        let store = match store_read_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        let inst = match store.multisig_institutions.get(&sfid_number) {
            Some(v) => v,
            None => return api_error(StatusCode::NOT_FOUND, 1004, "institution not found"),
        };
        if let Err(resp) = ensure_institution_visible_to_admin(inst, &ctx) {
            return resp;
        }
    }

    let mut file_name: Option<String> = None;
    let mut file_data: Option<Vec<u8>> = None;
    let mut doc_type: Option<String> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                file_name = field.file_name().map(|s| s.to_string());
                match field.bytes().await {
                    Ok(bytes) => file_data = Some(bytes.to_vec()),
                    Err(e) => {
                        return api_error(
                            StatusCode::BAD_REQUEST,
                            1001,
                            &format!("读取文件失败: {e}"),
                        );
                    }
                }
            }
            "doc_type" => {
                if let Ok(text) = field.text().await {
                    doc_type = Some(text.trim().to_string());
                }
            }
            _ => {}
        }
    }

    let file_name = match file_name.filter(|s| !s.trim().is_empty()) {
        Some(v) => v,
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "file is required"),
    };
    let file_data = match file_data.filter(|d| !d.is_empty()) {
        Some(v) => v,
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "file is empty"),
    };
    let doc_type = match doc_type.filter(|s| !s.trim().is_empty()) {
        Some(v) if VALID_DOC_TYPES.contains(&v.as_str()) => v,
        _ => {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "doc_type is required and must be one of: 公司章程/营业许可证/股东会决议/法人授权书/其他",
            );
        }
    };

    // 10MB 限制
    if file_data.len() > 10 * 1024 * 1024 {
        return api_error(StatusCode::BAD_REQUEST, 1001, "文件大小不能超过 10MB");
    }
    let grant_payload = serde_json::json!({
        "target": sfid_number.clone(),
        "sfid_number": sfid_number.clone(),
        "file_name": file_name.clone(),
        "doc_type": doc_type.clone(),
        "file_size": file_data.len(),
    });
    if let Err(resp) = require_admin_security_grant(
        &state,
        &headers,
        &ctx,
        AdminActionType::InstitutionUploadDocument,
        sfid_number.as_str(),
        Some(&grant_payload),
    ) {
        return resp;
    }

    // 写文件到 data/documents/{sfid_number}/
    let doc_dir = format!("data/documents/{sfid_number}");
    if let Err(e) = std::fs::create_dir_all(&doc_dir) {
        tracing::error!(error = %e, "create document dir failed");
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "create dir failed");
    }
    let file_ext = std::path::Path::new(&file_name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");
    let stored_name = format!(
        "{}_{}.{}",
        Utc::now().format("%Y%m%d%H%M%S"),
        Uuid::new_v4().as_simple(),
        file_ext
    );
    let stored_path = format!("{doc_dir}/{stored_name}");
    if let Err(e) = std::fs::write(&stored_path, &file_data) {
        tracing::error!(error = %e, "write document file failed");
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "write file failed");
    }

    let file_size = file_data.len() as u64;
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let doc_id = store.next_document_id;
    store.next_document_id += 1;
    let doc = InstitutionDocument {
        id: doc_id,
        sfid_number: sfid_number.clone(),
        file_name,
        doc_type,
        file_size,
        file_path: stored_path,
        uploaded_by: ctx.admin_pubkey.clone(),
        uploaded_at: Utc::now(),
    };
    store
        .institution_documents
        .insert(doc_id.to_string(), doc.clone());
    drop(store);

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: doc,
    })
    .into_response()
}

/// GET /api/v1/institution/:sfid_number/documents/:doc_id/download — 下载文档
pub(crate) async fn download_document(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((sfid_number, doc_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let store = match store_read_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let inst = match store.multisig_institutions.get(&sfid_number) {
        Some(v) => v,
        None => return api_error(StatusCode::NOT_FOUND, 1004, "institution not found"),
    };
    if let Err(resp) = ensure_institution_visible_to_admin(inst, &ctx) {
        return resp;
    }
    let doc = match store.institution_documents.get(&doc_id) {
        Some(d) if d.sfid_number == sfid_number => d.clone(),
        _ => return api_error(StatusCode::NOT_FOUND, 1004, "document not found"),
    };
    drop(store);

    let file_bytes = match std::fs::read(&doc.file_path) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error = %e, path = %doc.file_path, "read document file failed");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "read file failed");
        }
    };
    // RFC 5987 percent-encode 文件名
    let encoded_name: String = doc
        .file_name
        .bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b == b'.' || b == b'-' || b == b'_' {
                String::from(b as char)
            } else {
                format!("%{b:02X}")
            }
        })
        .collect();
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/octet-stream".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename*=UTF-8''{encoded_name}"),
            ),
        ],
        Body::from(file_bytes),
    )
        .into_response()
}

/// DELETE /api/v1/institution/:sfid_number/documents/:doc_id — 删除文档
pub(crate) async fn delete_document(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((sfid_number, doc_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    // 中文注释:删除前先校验机构存在、scope 和文档归属,再消费一次性安全授权。
    {
        let store = match store_read_or_500(&state) {
            Ok(v) => v,
            Err(resp) => return resp,
        };
        let inst = match store.multisig_institutions.get(&sfid_number) {
            Some(v) => v,
            None => return api_error(StatusCode::NOT_FOUND, 1004, "institution not found"),
        };
        if let Err(resp) = ensure_institution_visible_to_admin(inst, &ctx) {
            return resp;
        }
        let doc_exists = store
            .institution_documents
            .get(&doc_id)
            .map(|d| d.sfid_number == sfid_number)
            .unwrap_or(false);
        if !doc_exists {
            return api_error(StatusCode::NOT_FOUND, 1004, "document not found");
        }
    }
    let grant_payload = serde_json::json!({
        "target": sfid_number.clone(),
        "sfid_number": sfid_number.clone(),
        "doc_id": doc_id.clone(),
    });
    if let Err(resp) = require_admin_security_grant(
        &state,
        &headers,
        &ctx,
        AdminActionType::InstitutionDeleteDocument,
        sfid_number.as_str(),
        Some(&grant_payload),
    ) {
        return resp;
    }
    let mut store = match store_write_or_500(&state) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let doc = match store.institution_documents.get(&doc_id) {
        Some(d) if d.sfid_number == sfid_number => d.clone(),
        _ => return api_error(StatusCode::NOT_FOUND, 1004, "document not found"),
    };
    store.institution_documents.remove(&doc_id);
    drop(store);

    // 尝试删除文件(best-effort)
    if let Err(e) = std::fs::remove_file(&doc.file_path) {
        tracing::warn!(error = %e, path = %doc.file_path, "remove document file failed");
    }

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: "deleted",
    })
    .into_response()
}
