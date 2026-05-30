//! 公民资料库：归属 dangan 模块，负责资料元数据、文件正文存储和硬删除清理。

use std::{
    env,
    path::{Path, PathBuf},
};

use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Multipart, Path as AxumPath, State},
    http::{header, HeaderMap, HeaderValue, Response, StatusCode},
    routing::{delete, get},
    Json, Router,
};
use chrono::Utc;
use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::Row;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::{authz, err, ok, write_audit, ApiError, ApiResponse, AppState};

const MAX_MATERIAL_BYTES: u64 = 100 * 1024 * 1024;

#[derive(Clone, Serialize)]
pub(crate) struct ArchiveMaterial {
    material_id: String,
    archive_id: String,
    material_type: String,
    original_file_name: String,
    mime_type: String,
    file_size: i64,
    sha256: String,
    note: String,
    uploaded_by: String,
    uploaded_at: i64,
}

#[derive(Serialize)]
struct MaterialListData {
    items: Vec<ArchiveMaterial>,
}

#[derive(Serialize)]
struct MaterialUploadData {
    item: ArchiveMaterial,
}

struct TempUpload {
    original_file_name: String,
    stored_file_name: String,
    mime_type: String,
    file_size: i64,
    sha256: String,
    temp_path: PathBuf,
    final_path: PathBuf,
}

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/archives/:archive_id/materials",
            get(list_materials).post(upload_material),
        )
        .route(
            "/api/v1/archives/:archive_id/materials/:material_id/download",
            get(download_material),
        )
        .route(
            "/api/v1/archives/:archive_id/materials/:material_id",
            delete(delete_material),
        )
        .layer(DefaultBodyLimit::max(
            (MAX_MATERIAL_BYTES + 1024 * 1024) as usize,
        ))
}

async fn list_materials(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(archive_id): AxumPath<String>,
) -> Result<Json<ApiResponse<MaterialListData>>, (StatusCode, Json<ApiError>)> {
    authz::require_archive_admin(&state, &headers).await?;
    ensure_archive_exists(&state, &archive_id).await?;
    let items = load_materials(&state, &archive_id).await?;
    Ok(Json(ok(MaterialListData { items })))
}

async fn upload_material(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(archive_id): AxumPath<String>,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<MaterialUploadData>>, (StatusCode, Json<ApiError>)> {
    let ctx = authz::require_archive_admin(&state, &headers).await?;
    ensure_archive_active(&state, &archive_id).await?;

    let material_id = format!("mat_{}", Uuid::new_v4().simple());
    let mut material_type = "OTHER".to_string();
    let mut note = String::new();
    let mut upload: Option<TempUpload> = None;

    loop {
        let Some(mut field) = (match multipart.next_field().await {
            Ok(field) => field,
            Err(_) => {
                remove_temp_upload(&upload).await;
                return Err(err(
                    StatusCode::BAD_REQUEST,
                    1001,
                    "material multipart invalid",
                ));
            }
        }) else {
            break;
        };
        let name = field.name().unwrap_or_default().to_string();
        match name.as_str() {
            "material_type" => {
                material_type = match field.text().await {
                    Ok(value) => value,
                    Err(_) => {
                        remove_temp_upload(&upload).await;
                        return Err(err(StatusCode::BAD_REQUEST, 1001, "material type invalid"));
                    }
                };
            }
            "note" => {
                note = match field.text().await {
                    Ok(value) => value.trim().chars().take(200).collect(),
                    Err(_) => {
                        remove_temp_upload(&upload).await;
                        return Err(err(StatusCode::BAD_REQUEST, 1001, "material note invalid"));
                    }
                };
            }
            "file" => {
                if let Some(existing) = upload.as_ref() {
                    let _ = tokio::fs::remove_file(&existing.temp_path).await;
                    return Err(err(
                        StatusCode::BAD_REQUEST,
                        1001,
                        "material file duplicated",
                    ));
                }
                upload = Some(save_upload_field(&archive_id, &material_id, &mut field).await?);
            }
            _ => {}
        }
    }

    let upload =
        upload.ok_or_else(|| err(StatusCode::BAD_REQUEST, 1001, "material file required"))?;
    if let Err(e) = validate_material_type(&material_type) {
        let _ = tokio::fs::remove_file(&upload.temp_path).await;
        return Err(e);
    }
    if let Err(e) = validate_material_mime(&material_type, &upload.mime_type) {
        let _ = tokio::fs::remove_file(&upload.temp_path).await;
        return Err(e);
    }

    let now = Utc::now().timestamp();
    let insert_result = insert_material(
        &state,
        InsertMaterial {
            material_id: material_id.clone(),
            archive_id: archive_id.clone(),
            material_type,
            original_file_name: upload.original_file_name,
            stored_file_name: upload.stored_file_name,
            mime_type: upload.mime_type,
            file_size: upload.file_size,
            sha256: upload.sha256,
            note,
            uploaded_by: ctx.user_id.clone(),
            uploaded_at: now,
        },
    )
    .await;
    let item = match insert_result {
        Ok(item) => item,
        Err(e) => {
            let _ = tokio::fs::remove_file(&upload.temp_path).await;
            return Err(e);
        }
    };

    // 中文注释：先落临时文件再改名，避免上传中断时留下可被下载的半成品。
    if tokio::fs::rename(&upload.temp_path, &upload.final_path)
        .await
        .is_err()
    {
        let _ = sqlx::query("DELETE FROM archive_materials WHERE material_id = $1")
            .bind(&item.material_id)
            .execute(&state.db)
            .await;
        let _ = tokio::fs::remove_file(&upload.temp_path).await;
        return Err(err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "material file store failed",
        ));
    }

    write_audit(
        &state,
        Some(ctx.user_id),
        "ARCHIVE_MATERIAL_UPLOAD",
        "ARCHIVE_MATERIAL",
        Some(material_id),
        "SUCCESS",
        serde_json::json!({
            "archive_id": archive_id,
            "material_type": item.material_type,
            "mime_type": item.mime_type,
            "file_size": item.file_size,
            "sha256": item.sha256,
        }),
    )
    .await?;

    Ok(Json(ok(MaterialUploadData { item })))
}

async fn download_material(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath((archive_id, material_id)): AxumPath<(String, String)>,
) -> Result<Response<Body>, (StatusCode, Json<ApiError>)> {
    let ctx = authz::require_archive_admin(&state, &headers).await?;
    ensure_archive_exists(&state, &archive_id).await?;
    let material = load_material(&state, &archive_id, &material_id).await?;
    let path = material_path(&archive_id, &material.stored_file_name);
    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|_| err(StatusCode::NOT_FOUND, 4041, "material file not found"))?;

    write_audit(
        &state,
        Some(ctx.user_id),
        "ARCHIVE_MATERIAL_DOWNLOAD",
        "ARCHIVE_MATERIAL",
        Some(material_id),
        "SUCCESS",
        serde_json::json!({
            "archive_id": archive_id,
            "file_size": material.file_size,
            "sha256": material.sha256,
        }),
    )
    .await?;

    let disposition = format!(
        "inline; filename=\"{}\"",
        material.original_file_name.replace('"', "")
    );
    let mut response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, material.mime_type)
        .header(header::CONTENT_LENGTH, bytes.len().to_string())
        .body(Body::from(bytes))
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "material response failed",
            )
        })?;
    if let Ok(value) = HeaderValue::from_str(&disposition) {
        response
            .headers_mut()
            .insert(header::CONTENT_DISPOSITION, value);
    }
    Ok(response)
}

async fn delete_material(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath((archive_id, material_id)): AxumPath<(String, String)>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiError>)> {
    let ctx = authz::require_archive_admin(&state, &headers).await?;
    ensure_archive_active(&state, &archive_id).await?;
    let material = load_material(&state, &archive_id, &material_id).await?;
    let now = Utc::now().timestamp();
    let result = sqlx::query(
        "UPDATE archive_materials
         SET deleted_at = $1, deleted_by = $2
         WHERE material_id = $3 AND archive_id = $4 AND deleted_at IS NULL",
    )
    .bind(now)
    .bind(&ctx.user_id)
    .bind(&material_id)
    .bind(&archive_id)
    .execute(&state.db)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "delete material failed",
        )
    })?;

    if result.rows_affected() != 1 {
        return Err(err(StatusCode::NOT_FOUND, 4040, "material not found"));
    }

    write_audit(
        &state,
        Some(ctx.user_id),
        "ARCHIVE_MATERIAL_DELETE",
        "ARCHIVE_MATERIAL",
        Some(material_id),
        "SUCCESS",
        serde_json::json!({
            "archive_id": archive_id,
            "file_size": material.file_size,
            "sha256": material.sha256,
        }),
    )
    .await?;

    Ok(Json(ok(serde_json::json!({"deleted_at": now}))))
}

pub(crate) async fn remove_archive_material_files(archive_id: &str) -> Result<(), String> {
    let dir = material_archive_dir(archive_id);
    if !dir.exists() {
        return Ok(());
    }
    tokio::fs::remove_dir_all(&dir)
        .await
        .map_err(|e| format!("delete archive material files failed: {e}"))
}

async fn remove_temp_upload(upload: &Option<TempUpload>) {
    if let Some(upload) = upload {
        let _ = tokio::fs::remove_file(&upload.temp_path).await;
    }
}

async fn save_upload_field(
    archive_id: &str,
    material_id: &str,
    field: &mut axum::extract::multipart::Field<'_>,
) -> Result<TempUpload, (StatusCode, Json<ApiError>)> {
    // 中文注释：上传流边写边算 SHA-256，数据库只保存元数据和摘要，不保存文件正文。
    let original_file_name = sanitize_original_file_name(field.file_name().unwrap_or("material"));
    let mime_type = field
        .content_type()
        .map(|v| v.to_string())
        .ok_or_else(|| err(StatusCode::BAD_REQUEST, 1001, "material mime invalid"))?;
    let ext = file_extension(&original_file_name);
    let stored_file_name = format!("{material_id}{ext}");
    let dir = material_archive_dir(archive_id);
    tokio::fs::create_dir_all(&dir).await.map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "material dir create failed",
        )
    })?;

    let temp_path = dir.join(format!("{stored_file_name}.tmp"));
    let final_path = dir.join(&stored_file_name);
    let mut file = tokio::fs::File::create(&temp_path).await.map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "material file store failed",
        )
    })?;
    let mut hasher = Sha256::new();
    let mut file_size: u64 = 0;

    loop {
        let chunk = match field.chunk().await {
            Ok(chunk) => chunk,
            Err(_) => {
                let _ = tokio::fs::remove_file(&temp_path).await;
                return Err(err(StatusCode::BAD_REQUEST, 1001, "material file invalid"));
            }
        };
        let Some(chunk) = chunk else {
            break;
        };
        file_size += chunk.len() as u64;
        if file_size > MAX_MATERIAL_BYTES {
            let _ = tokio::fs::remove_file(&temp_path).await;
            return Err(err(
                StatusCode::PAYLOAD_TOO_LARGE,
                1001,
                "material file too large",
            ));
        }
        hasher.update(&chunk);
        if file.write_all(&chunk).await.is_err() {
            let _ = tokio::fs::remove_file(&temp_path).await;
            return Err(err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "material file store failed",
            ));
        }
    }
    if file.flush().await.is_err() {
        let _ = tokio::fs::remove_file(&temp_path).await;
        return Err(err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "material file store failed",
        ));
    }
    if file_size == 0 {
        let _ = tokio::fs::remove_file(&temp_path).await;
        return Err(err(StatusCode::BAD_REQUEST, 1001, "material file empty"));
    }

    Ok(TempUpload {
        original_file_name,
        stored_file_name,
        mime_type,
        file_size: file_size as i64,
        sha256: format!("0x{}", hex::encode(hasher.finalize())),
        temp_path,
        final_path,
    })
}

struct InsertMaterial {
    material_id: String,
    archive_id: String,
    material_type: String,
    original_file_name: String,
    stored_file_name: String,
    mime_type: String,
    file_size: i64,
    sha256: String,
    note: String,
    uploaded_by: String,
    uploaded_at: i64,
}

async fn insert_material(
    state: &AppState,
    material: InsertMaterial,
) -> Result<ArchiveMaterial, (StatusCode, Json<ApiError>)> {
    let row = sqlx::query(
        "INSERT INTO archive_materials
         (material_id, archive_id, material_type, original_file_name, stored_file_name,
          mime_type, file_size, sha256, note, uploaded_by, uploaded_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
         RETURNING material_id, archive_id, material_type, original_file_name, mime_type,
                   file_size, sha256, note, uploaded_by, uploaded_at",
    )
    .bind(&material.material_id)
    .bind(&material.archive_id)
    .bind(&material.material_type)
    .bind(&material.original_file_name)
    .bind(&material.stored_file_name)
    .bind(&material.mime_type)
    .bind(material.file_size)
    .bind(&material.sha256)
    .bind(&material.note)
    .bind(&material.uploaded_by)
    .bind(material.uploaded_at)
    .fetch_one(&state.db)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "insert material failed",
        )
    })?;
    Ok(row_to_material(row))
}

async fn load_materials(
    state: &AppState,
    archive_id: &str,
) -> Result<Vec<ArchiveMaterial>, (StatusCode, Json<ApiError>)> {
    let rows = sqlx::query(
        "SELECT material_id, archive_id, material_type, original_file_name, mime_type,
                file_size, sha256, note, uploaded_by, uploaded_at
         FROM archive_materials
         WHERE archive_id = $1 AND deleted_at IS NULL
         ORDER BY uploaded_at DESC, material_id DESC",
    )
    .bind(archive_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "query materials failed",
        )
    })?;
    Ok(rows.into_iter().map(row_to_material).collect())
}

struct StoredMaterial {
    original_file_name: String,
    stored_file_name: String,
    mime_type: String,
    file_size: i64,
    sha256: String,
}

async fn load_material(
    state: &AppState,
    archive_id: &str,
    material_id: &str,
) -> Result<StoredMaterial, (StatusCode, Json<ApiError>)> {
    let row = sqlx::query(
        "SELECT original_file_name, stored_file_name, mime_type, file_size, sha256
         FROM archive_materials
         WHERE material_id = $1 AND archive_id = $2 AND deleted_at IS NULL",
    )
    .bind(material_id)
    .bind(archive_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "query material failed",
        )
    })?
    .ok_or_else(|| err(StatusCode::NOT_FOUND, 4040, "material not found"))?;

    Ok(StoredMaterial {
        original_file_name: row.get("original_file_name"),
        stored_file_name: row.get("stored_file_name"),
        mime_type: row.get("mime_type"),
        file_size: row.get("file_size"),
        sha256: row.get("sha256"),
    })
}

fn row_to_material(row: sqlx::postgres::PgRow) -> ArchiveMaterial {
    ArchiveMaterial {
        material_id: row.get("material_id"),
        archive_id: row.get("archive_id"),
        material_type: row.get("material_type"),
        original_file_name: row.get("original_file_name"),
        mime_type: row.get("mime_type"),
        file_size: row.get("file_size"),
        sha256: row.get("sha256"),
        note: row.get("note"),
        uploaded_by: row.get("uploaded_by"),
        uploaded_at: row.get("uploaded_at"),
    }
}

async fn ensure_archive_exists(
    state: &AppState,
    archive_id: &str,
) -> Result<String, (StatusCode, Json<ApiError>)> {
    sqlx::query_scalar::<_, String>("SELECT status FROM archives WHERE archive_id = $1")
        .bind(archive_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "query archive failed",
            )
        })?
        .ok_or_else(|| err(StatusCode::NOT_FOUND, 4040, "archive not found"))
}

async fn ensure_archive_active(
    state: &AppState,
    archive_id: &str,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    let status = ensure_archive_exists(state, archive_id).await?;
    if status != "ACTIVE" {
        return Err(err(StatusCode::CONFLICT, 3016, "archive already deleted"));
    }
    Ok(())
}

fn validate_material_type(value: &str) -> Result<(), (StatusCode, Json<ApiError>)> {
    match value {
        "PHOTO" | "BIRTH_CERTIFICATE" | "COPY" | "VIDEO" | "OTHER" => Ok(()),
        _ => Err(err(StatusCode::BAD_REQUEST, 1001, "material type invalid")),
    }
}

fn validate_material_mime(
    material_type: &str,
    mime_type: &str,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    let allowed = match material_type {
        "PHOTO" => matches!(mime_type, "image/jpeg" | "image/png" | "image/webp"),
        "VIDEO" => matches!(mime_type, "video/mp4" | "video/quicktime" | "video/webm"),
        "BIRTH_CERTIFICATE" | "COPY" | "OTHER" => matches!(
            mime_type,
            "image/jpeg"
                | "image/png"
                | "image/webp"
                | "application/pdf"
                | "video/mp4"
                | "video/quicktime"
                | "video/webm"
        ),
        _ => false,
    };
    allowed
        .then_some(())
        .ok_or_else(|| err(StatusCode::BAD_REQUEST, 1001, "material mime invalid"))
}

fn material_root() -> PathBuf {
    // 中文注释：部署可把资料正文放到专用数据盘；默认目录用于本机开发和单机部署。
    env::var("CPMS_MATERIALS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("data/archive-materials"))
}

fn material_archive_dir(archive_id: &str) -> PathBuf {
    material_root().join(safe_segment(archive_id))
}

fn material_path(archive_id: &str, stored_file_name: &str) -> PathBuf {
    material_archive_dir(archive_id).join(safe_segment(stored_file_name))
}

fn sanitize_original_file_name(value: &str) -> String {
    Path::new(value)
        .file_name()
        .and_then(|v| v.to_str())
        .map(|v| v.trim().chars().take(120).collect::<String>())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "material".to_string())
}

fn file_extension(file_name: &str) -> String {
    Path::new(file_name)
        .extension()
        .and_then(|v| v.to_str())
        .map(|v| {
            let clean: String = v
                .chars()
                .filter(|c| c.is_ascii_alphanumeric())
                .take(12)
                .collect();
            if clean.is_empty() {
                String::new()
            } else {
                format!(".{}", clean.to_ascii_lowercase())
            }
        })
        .unwrap_or_default()
}

fn safe_segment(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.') {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        file_extension, safe_segment, sanitize_original_file_name, validate_material_mime,
    };

    #[test]
    fn material_mime_validation_matches_type() {
        assert!(validate_material_mime("PHOTO", "image/jpeg").is_ok());
        assert!(validate_material_mime("VIDEO", "video/mp4").is_ok());
        assert!(validate_material_mime("BIRTH_CERTIFICATE", "application/pdf").is_ok());
        assert!(validate_material_mime("PHOTO", "application/pdf").is_err());
        assert!(validate_material_mime("VIDEO", "image/png").is_err());
    }

    #[test]
    fn material_file_names_are_kept_inside_archive_dir() {
        assert_eq!(sanitize_original_file_name("../birth.pdf"), "birth.pdf");
        assert_eq!(file_extension("report.final.PDF"), ".pdf");
        assert_eq!(safe_segment("arc_1/../../x"), "arc_1_.._.._x");
    }
}
