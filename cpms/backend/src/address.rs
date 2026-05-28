//! 地址管理模块：镇/村路 API。
//!
//! CPMS 编译期直接引用 SFID 工具行政区唯一源，发行版只内置编译后的只读数据。
//! 运行时只启用安装码对应市公安局的镇/村路数据。
//! 地址 API 只读，CPMS 不允许保存或维护第二套行政区数据源。

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::{authz, err, ok, sfid_tool_province::PROVINCES, ApiError, ApiResponse, AppState};

pub(crate) async fn sync_installed_city_address(db: &sqlx::PgPool) -> Result<(), String> {
    let sfid_number: Option<String> =
        sqlx::query_scalar("SELECT sfid_number FROM system_install WHERE id = 1")
            .fetch_optional(db)
            .await
            .map_err(|e| format!("query install failed: {e}"))?
            .flatten();
    let Some(sfid_number) = sfid_number.filter(|v| !v.trim().is_empty()) else {
        return Ok(());
    };
    sync_city_address_by_sfid(db, &sfid_number).await
}

pub(crate) async fn sync_city_address_by_sfid(
    db: &sqlx::PgPool,
    sfid_number: &str,
) -> Result<(), String> {
    let (province_code, city_code) = parse_sfid_area_codes(sfid_number)?;
    let city = PROVINCES
        .iter()
        .find(|p| p.code == province_code)
        .and_then(|p| p.cities.iter().find(|c| c.code == city_code))
        .ok_or_else(|| {
            format!("install city not found in embedded sfid tool: {province_code}{city_code}")
        })?;

    let mut tx = db
        .begin()
        .await
        .map_err(|e| format!("begin address sync tx failed: {e}"))?;
    // 中文注释：行政区唯一来源是 SFID 工具；启动同步只落当前市运行表，清掉旧城市/旧硬编码残留。
    sqlx::query("DELETE FROM address_villages")
        .execute(tx.as_mut())
        .await
        .map_err(|e| format!("clear villages failed: {e}"))?;
    sqlx::query("DELETE FROM address_towns")
        .execute(tx.as_mut())
        .await
        .map_err(|e| format!("clear towns failed: {e}"))?;

    for town in city.towns {
        sqlx::query("INSERT INTO address_towns (town_code, town_name) VALUES ($1, $2)")
            .bind(town.code)
            .bind(town.name)
            .execute(tx.as_mut())
            .await
            .map_err(|e| format!("insert town {} failed: {e}", town.code))?;

        for village in town.villages {
            let village_id = format!("{}-{}", town.code, village.code);
            sqlx::query(
                "INSERT INTO address_villages (village_id, town_code, village_name) VALUES ($1, $2, $3)",
            )
            .bind(&village_id)
            .bind(town.code)
            .bind(village.name)
            .execute(tx.as_mut())
            .await
            .map_err(|e| format!("insert village {} failed: {e}", village_id))?;
        }
    }

    tx.commit()
        .await
        .map_err(|e| format!("commit address sync failed: {e}"))?;
    Ok(())
}

fn parse_sfid_area_codes(sfid_number: &str) -> Result<(&str, &str), String> {
    let r5 = sfid_number
        .trim()
        .split('-')
        .nth(1)
        .ok_or_else(|| "install sfid r5 segment missing".to_string())?;
    if r5.len() != 5 {
        return Err("install sfid r5 segment invalid".to_string());
    }
    let (province_code, city_code) = r5.split_at(2);
    if city_code == "000" {
        return Err("cpms install sfid must bind to a city public security bureau".to_string());
    }
    Ok((province_code, city_code))
}

// ── API ──

#[derive(Serialize)]
struct TownRow {
    town_code: String,
    town_name: String,
}

#[derive(Serialize)]
struct VillageRow {
    village_id: String,
    town_code: String,
    village_name: String,
}

#[derive(Deserialize)]
struct VillageQuery {
    town_code: String,
}

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/address/towns", get(list_towns))
        .route("/api/v1/address/villages", get(list_villages))
}

async fn list_towns(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Vec<TownRow>>>, (StatusCode, Json<ApiError>)> {
    authz::require_auth(&state, &headers).await?;
    let rows = sqlx::query("SELECT town_code, town_name FROM address_towns ORDER BY town_code")
        .fetch_all(&state.db)
        .await
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "query towns failed",
            )
        })?;
    let towns: Vec<TownRow> = rows
        .iter()
        .map(|r| TownRow {
            town_code: r.get("town_code"),
            town_name: r.get("town_name"),
        })
        .collect();
    Ok(Json(ok(towns)))
}

async fn list_villages(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<VillageQuery>,
) -> Result<Json<ApiResponse<Vec<VillageRow>>>, (StatusCode, Json<ApiError>)> {
    authz::require_auth(&state, &headers).await?;
    let rows = sqlx::query(
        "SELECT village_id, town_code, village_name FROM address_villages WHERE town_code = $1 ORDER BY village_id",
    )
    .bind(q.town_code.trim())
    .fetch_all(&state.db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "query villages failed"))?;
    let villages: Vec<VillageRow> = rows
        .iter()
        .map(|r| VillageRow {
            village_id: r.get("village_id"),
            town_code: r.get("town_code"),
            village_name: r.get("village_name"),
        })
        .collect();
    Ok(Json(ok(villages)))
}
