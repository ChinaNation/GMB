//! 机构资料库 HTTP handler。
//!
//! 中文注释:资料文件本体存磁盘,元数据只写 `docs` 结构化表。

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
use crate::admins::login::require_admin_any;
use crate::admins::operation_auth::AdminActionType;
use crate::core::response::ApiResponse;
use crate::subjects::http::ensure_institution_visible_to_admin;
use crate::subjects::model::{InstitutionDocument, VALID_DOC_TYPES};
use crate::*;

impl Db {
    fn list_documents_for_subject(
        &self,
        sfid_number: &str,
    ) -> Result<Vec<InstitutionDocument>, String> {
        let sfid_number = sfid_number.trim().to_string();
        self.with_client(move |conn| {
            let rows = conn
                .query(
                    "SELECT id, sfid_number, file_name, doc_type, file_size, file_path,
                            uploaded_by, uploaded_at
                     FROM docs
                     WHERE sfid_number = $1
                     ORDER BY uploaded_at DESC, id DESC",
                    &[&sfid_number],
                )
                .map_err(|e| format!("query documents failed: {e}"))?;
            rows.iter()
                .map(|row| {
                    let id: i64 = row.get(0);
                    let file_size: i64 = row.get(4);
                    Ok(InstitutionDocument {
                        id: u64::try_from(id).unwrap_or(0),
                        sfid_number: row.get(1),
                        file_name: row.get(2),
                        doc_type: row.get(3),
                        file_size: u64::try_from(file_size).unwrap_or(0),
                        file_path: row.get(5),
                        uploaded_by: row.get(6),
                        uploaded_at: row.get(7),
                    })
                })
                .collect()
        })
    }

    fn insert_document(&self, doc: &InstitutionDocument) -> Result<InstitutionDocument, String> {
        let doc = doc.clone();
        self.with_client(move |conn| {
            let (province_code, city_code) = Db::scope_codes_from_sfid(doc.sfid_number.as_str());
            let file_size = i64::try_from(doc.file_size)
                .map_err(|_| "document file size too large".to_string())?;
            let row = conn
                .query_one(
                    "INSERT INTO docs (
                        sfid_number, province_code, city_code, file_name, doc_type, file_size,
                        file_path, uploaded_by, uploaded_at
                     ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                     RETURNING id",
                    &[
                        &doc.sfid_number,
                        &province_code,
                        &city_code,
                        &doc.file_name,
                        &doc.doc_type,
                        &file_size,
                        &doc.file_path,
                        &doc.uploaded_by,
                        &doc.uploaded_at,
                    ],
                )
                .map_err(|e| format!("insert document failed: {e}"))?;
            let id: i64 = row.get(0);
            Ok(InstitutionDocument {
                id: u64::try_from(id).unwrap_or(0),
                ..doc
            })
        })
    }

    fn get_document(
        &self,
        sfid_number: &str,
        doc_id: u64,
    ) -> Result<Option<InstitutionDocument>, String> {
        let sfid_number = sfid_number.trim().to_string();
        let doc_id = i64::try_from(doc_id).map_err(|_| "document id too large".to_string())?;
        self.with_client(move |conn| {
            let row = conn
                .query_opt(
                    "SELECT id, sfid_number, file_name, doc_type, file_size, file_path,
                            uploaded_by, uploaded_at
                     FROM docs
                     WHERE sfid_number = $1 AND id = $2",
                    &[&sfid_number, &doc_id],
                )
                .map_err(|e| format!("query document failed: {e}"))?;
            Ok(row.map(|row| {
                let id: i64 = row.get(0);
                let file_size: i64 = row.get(4);
                InstitutionDocument {
                    id: u64::try_from(id).unwrap_or(0),
                    sfid_number: row.get(1),
                    file_name: row.get(2),
                    doc_type: row.get(3),
                    file_size: u64::try_from(file_size).unwrap_or(0),
                    file_path: row.get(5),
                    uploaded_by: row.get(6),
                    uploaded_at: row.get(7),
                }
            }))
        })
    }

    fn delete_document(&self, sfid_number: &str, doc_id: u64) -> Result<bool, String> {
        let sfid_number = sfid_number.trim().to_string();
        let doc_id = i64::try_from(doc_id).map_err(|_| "document id too large".to_string())?;
        self.with_client(move |conn| {
            let affected = conn
                .execute(
                    "DELETE FROM docs WHERE sfid_number = $1 AND id = $2",
                    &[&sfid_number, &doc_id],
                )
                .map_err(|e| format!("delete document failed: {e}"))?;
            Ok(affected > 0)
        })
    }
}

pub(crate) async fn list_documents(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(sfid_number): Path<String>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let Some((inst, _)) = (match state.db.get_institution_with_accounts(&sfid_number) {
        Ok(v) => v,
        Err(err) => {
            tracing::error!(error = %err, "query institution failed");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "institution query failed",
            );
        }
    }) else {
        return api_error(StatusCode::NOT_FOUND, 1004, "institution not found");
    };
    if let Err(resp) = ensure_institution_visible_to_admin(&inst, &ctx) {
        return resp;
    }
    let docs = match state.db.list_documents_for_subject(&sfid_number) {
        Ok(v) => v,
        Err(err) => {
            tracing::error!(error = %err, "list documents failed");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "documents query failed",
            );
        }
    };
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: docs,
    })
    .into_response()
}

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
    let Some((inst, _)) = (match state.db.get_institution_with_accounts(&sfid_number) {
        Ok(v) => v,
        Err(err) => {
            tracing::error!(error = %err, "query institution failed");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "institution query failed",
            );
        }
    }) else {
        return api_error(StatusCode::NOT_FOUND, 1004, "institution not found");
    };
    if let Err(resp) = ensure_institution_visible_to_admin(&inst, &ctx) {
        return resp;
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
                        let message = format!("读取文件失败: {e}");
                        return api_error(StatusCode::BAD_REQUEST, 1001, message.as_str());
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
    let doc_type = match doc_type.filter(|s| VALID_DOC_TYPES.contains(&s.as_str())) {
        Some(v) => v,
        None => return api_error(StatusCode::BAD_REQUEST, 1001, "invalid doc_type"),
    };
    if file_data.len() > 10 * 1024 * 1024 {
        return api_error(StatusCode::BAD_REQUEST, 1001, "文件大小不能超过 10MB");
    }
    let grant_payload = serde_json::json!({
        "target": sfid_number.clone(),
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
    let doc = InstitutionDocument {
        id: 0,
        sfid_number: sfid_number.clone(),
        file_name,
        doc_type,
        file_size: file_data.len() as u64,
        file_path: stored_path,
        uploaded_by: ctx.admin_pubkey.clone(),
        uploaded_at: Utc::now(),
    };
    let doc = match state.db.insert_document(&doc) {
        Ok(v) => v,
        Err(err) => {
            tracing::error!(error = %err, "insert document failed");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "document write failed",
            );
        }
    };
    crate::core::runtime_ops::append_audit_log(
        &state,
        "INSTITUTION_DOCUMENT_UPLOAD",
        &ctx.admin_pubkey,
        Some(sfid_number.clone()),
        serde_json::json!({
            "sfid_number": sfid_number.clone(),
            "doc_id": doc.id,
            "file_name": doc.file_name.clone(),
            "doc_type": doc.doc_type.clone(),
            "file_size": doc.file_size,
        }),
    );
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: doc,
    })
    .into_response()
}

pub(crate) async fn download_document(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((sfid_number, doc_id)): Path<(String, u64)>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let Some((inst, _)) = (match state.db.get_institution_with_accounts(&sfid_number) {
        Ok(v) => v,
        Err(err) => {
            tracing::error!(error = %err, "query institution failed");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "institution query failed",
            );
        }
    }) else {
        return api_error(StatusCode::NOT_FOUND, 1004, "institution not found");
    };
    if let Err(resp) = ensure_institution_visible_to_admin(&inst, &ctx) {
        return resp;
    }
    let doc = match state.db.get_document(&sfid_number, doc_id) {
        Ok(Some(v)) => v,
        Ok(None) => return api_error(StatusCode::NOT_FOUND, 1004, "document not found"),
        Err(err) => {
            tracing::error!(error = %err, "query document failed");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "document query failed",
            );
        }
    };
    let data = match std::fs::read(&doc.file_path) {
        Ok(v) => v,
        Err(_) => return api_error(StatusCode::NOT_FOUND, 1004, "file not found"),
    };
    crate::core::runtime_ops::append_audit_log(
        &state,
        "INSTITUTION_DOCUMENT_DOWNLOAD",
        &ctx.admin_pubkey,
        Some(sfid_number.clone()),
        serde_json::json!({
            "sfid_number": sfid_number.clone(),
            "doc_id": doc.id,
            "file_name": doc.file_name.clone(),
            "doc_type": doc.doc_type.clone(),
            "file_size": doc.file_size,
        }),
    );
    let safe_name = doc
        .file_name
        .bytes()
        .map(|b: u8| {
            if b.is_ascii_alphanumeric() || b == b'.' || b == b'-' || b == b'_' {
                b as char
            } else {
                '_'
            }
        })
        .collect::<String>();
    (
        [
            (header::CONTENT_TYPE, "application/octet-stream".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{safe_name}\""),
            ),
        ],
        Body::from(data),
    )
        .into_response()
}

pub(crate) async fn delete_document(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((sfid_number, doc_id)): Path<(String, u64)>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let Some((inst, _)) = (match state.db.get_institution_with_accounts(&sfid_number) {
        Ok(v) => v,
        Err(err) => {
            tracing::error!(error = %err, "query institution failed");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "institution query failed",
            );
        }
    }) else {
        return api_error(StatusCode::NOT_FOUND, 1004, "institution not found");
    };
    if let Err(resp) = ensure_institution_visible_to_admin(&inst, &ctx) {
        return resp;
    }
    let doc = match state.db.get_document(&sfid_number, doc_id) {
        Ok(Some(v)) => v,
        Ok(None) => return api_error(StatusCode::NOT_FOUND, 1004, "document not found"),
        Err(err) => {
            tracing::error!(error = %err, "query document failed");
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                "document query failed",
            );
        }
    };
    let grant_payload = serde_json::json!({
        "target": sfid_number.clone(),
        "doc_id": doc_id,
        "file_name": doc.file_name.clone(),
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
    if let Err(err) = state.db.delete_document(&sfid_number, doc_id) {
        tracing::error!(error = %err, "delete document failed");
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "document delete failed",
        );
    }
    let _ = std::fs::remove_file(&doc.file_path);
    crate::core::runtime_ops::append_audit_log(
        &state,
        "INSTITUTION_DOCUMENT_DELETE",
        &ctx.admin_pubkey,
        Some(sfid_number.clone()),
        serde_json::json!({
            "sfid_number": sfid_number.clone(),
            "doc_id": doc.id,
            "file_name": doc.file_name.clone(),
            "doc_type": doc.doc_type.clone(),
            "file_size": doc.file_size,
        }),
    );
    #[derive(serde::Serialize)]
    struct DeleteOutput {
        deleted: bool,
    }
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: DeleteOutput { deleted: true },
    })
    .into_response()
}
