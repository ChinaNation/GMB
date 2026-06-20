//! 地址管理模块：镇/地址段 API。
//!
//! 行政区唯一源是 SFID 维护的 `china.sqlite`，CPMS 安装包随附其只读拷贝。
//! 运行时只启用安装码对应市公安局的镇与镇下地址段数据。
//! 地址 API 只读，CPMS 不允许保存或维护第二套行政区数据源。
//!
//! 中文注释：`china` 子模块是本模块对 SFID 行政区源的只读适配，仅服务 address 业务。

mod china;

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::{
    authz,
    common::{err, ok, ApiError, ApiResponse},
    AppState,
};

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
    let mut tx = db
        .begin()
        .await
        .map_err(|e| format!("begin address sync tx failed: {e}"))?;
    sync_city_address_by_sfid_in_tx(tx.as_mut(), sfid_number).await?;
    tx.commit()
        .await
        .map_err(|e| format!("commit address sync failed: {e}"))?;
    Ok(())
}

pub(crate) async fn sync_city_address_by_sfid_in_tx(
    conn: &mut sqlx::PgConnection,
    sfid_number: &str,
) -> Result<(), String> {
    let (province_code, city_code, _) = find_install_city(sfid_number)?;
    let towns = china::city_towns_with_address_units(&province_code, &city_code)?;
    replace_city_address(conn, &towns).await
}

pub(crate) fn validate_install_area(
    sfid_number: &str,
    province_name: &str,
    city_name: &str,
) -> Result<(), String> {
    let (province_code, city_code, area) = find_install_city(sfid_number)?;
    if area.province_name != province_name.trim() {
        return Err(format!(
            "install province_name '{}' does not match code {}",
            province_name.trim(),
            province_code
        ));
    }
    if area.city_name != city_name.trim() {
        return Err(format!(
            "install city_name '{}' does not match code {}{}",
            city_name.trim(),
            province_code,
            city_code
        ));
    }
    Ok(())
}

pub(crate) async fn validate_town_address_unit(
    state: &AppState,
    town_code: &str,
    address_unit_id: &str,
) -> Result<String, (StatusCode, Json<ApiError>)> {
    let name: Option<String> = sqlx::query_scalar(
        "SELECT u.address_unit_name
         FROM address_towns t
         JOIN address_units u ON u.town_code = t.town_code
         WHERE t.town_code = $1 AND u.address_unit_id = $2",
    )
    .bind(town_code.trim())
    .bind(address_unit_id.trim())
    .fetch_optional(&state.db)
    .await
    .map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "query address failed",
        )
    })?;

    let Some(name) = name else {
        return Err(err(StatusCode::NOT_FOUND, 3006, "address area not found"));
    };
    Ok(name)
}

pub(crate) fn validate_birth_town(
    province_code: &str,
    city_code: &str,
    town_code: &str,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    if province_code.trim().is_empty() || city_code.trim().is_empty() || town_code.trim().is_empty()
    {
        return Err(err(
            StatusCode::BAD_REQUEST,
            1001,
            "birthplace province, city and town are required",
        ));
    }
    let exists = china::find_town(province_code, city_code, town_code).map_err(|_| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            5001,
            "query birthplace failed",
        )
    })?;
    if !exists {
        return Err(err(
            StatusCode::NOT_FOUND,
            3006,
            "birthplace area not found",
        ));
    }
    Ok(())
}

async fn replace_city_address(
    conn: &mut sqlx::PgConnection,
    towns: &[china::TownArea],
) -> Result<(), String> {
    // 中文注释：行政区唯一来源是 SFID 的 china.sqlite；同步只落当前市运行表，清掉旧城市残留。
    sqlx::query("DELETE FROM address_units")
        .execute(&mut *conn)
        .await
        .map_err(|e| format!("clear address units failed: {e}"))?;
    sqlx::query("DELETE FROM address_towns")
        .execute(&mut *conn)
        .await
        .map_err(|e| format!("clear towns failed: {e}"))?;

    for town in towns {
        sqlx::query("INSERT INTO address_towns (town_code, town_name) VALUES ($1, $2)")
            .bind(&town.code)
            .bind(&town.name)
            .execute(&mut *conn)
            .await
            .map_err(|e| format!("insert town {} failed: {e}", town.code))?;

        for address_unit in &town.address_units {
            sqlx::query(
                "INSERT INTO address_units (address_unit_id, town_code, address_unit_name) VALUES ($1, $2, $3)",
            )
            .bind(&address_unit.id)
            .bind(&town.code)
            .bind(&address_unit.name)
            .execute(&mut *conn)
            .await
            .map_err(|e| format!("insert address unit {} failed: {e}", address_unit.id))?;
        }
    }
    Ok(())
}

/// 解析安装码所属省市代码并从 china.sqlite 还原省市名称。
fn find_install_city(sfid_number: &str) -> Result<(String, String, china::CityArea), String> {
    let (province_code, city_code) = parse_sfid_area_codes(sfid_number)?;
    let area = china::find_city(province_code, city_code)?.ok_or_else(|| {
        format!("install city not found in china source: {province_code}{city_code}")
    })?;
    Ok((province_code.to_string(), city_code.to_string(), area))
}

fn parse_sfid_area_codes(sfid_number: &str) -> Result<(&str, &str), String> {
    let r5 = sfid_number
        .trim()
        .split('-')
        .next()
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
struct ProvinceRow {
    province_code: String,
    province_name: String,
}

#[derive(Serialize)]
struct CityRow {
    city_code: String,
    city_name: String,
}

#[derive(Serialize)]
struct AddressUnitRow {
    address_unit_id: String,
    town_code: String,
    address_unit_name: String,
}

#[derive(Deserialize)]
struct CitiesQuery {
    province_code: String,
}

#[derive(Deserialize)]
struct BirthTownsQuery {
    province_code: String,
    city_code: String,
}

#[derive(Deserialize)]
struct AddressUnitQuery {
    town_code: String,
}

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/address/china/provinces", get(list_china_provinces))
        .route("/api/v1/address/china/cities", get(list_china_cities))
        .route("/api/v1/address/china/towns", get(list_china_towns))
        .route("/api/v1/address/towns", get(list_towns))
        .route("/api/v1/address/units", get(list_address_units))
}

async fn list_china_provinces(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Vec<ProvinceRow>>>, (StatusCode, Json<ApiError>)> {
    authz::require_auth(&state, &headers).await?;
    let rows = china::provinces()
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "query provinces failed",
            )
        })?
        .into_iter()
        .map(|p| ProvinceRow {
            province_code: p.code,
            province_name: p.name,
        })
        .collect();
    Ok(Json(ok(rows)))
}

async fn list_china_cities(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<CitiesQuery>,
) -> Result<Json<ApiResponse<Vec<CityRow>>>, (StatusCode, Json<ApiError>)> {
    authz::require_auth(&state, &headers).await?;
    let rows = china::cities(q.province_code.as_str())
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "query cities failed",
            )
        })?
        .into_iter()
        .map(|c| CityRow {
            city_code: c.code,
            city_name: c.name,
        })
        .collect();
    Ok(Json(ok(rows)))
}

async fn list_china_towns(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<BirthTownsQuery>,
) -> Result<Json<ApiResponse<Vec<TownRow>>>, (StatusCode, Json<ApiError>)> {
    authz::require_auth(&state, &headers).await?;
    let rows = china::towns(q.province_code.as_str(), q.city_code.as_str())
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "query towns failed",
            )
        })?
        .into_iter()
        .map(|t| TownRow {
            town_code: t.code,
            town_name: t.name,
        })
        .collect();
    Ok(Json(ok(rows)))
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

async fn list_address_units(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<AddressUnitQuery>,
) -> Result<Json<ApiResponse<Vec<AddressUnitRow>>>, (StatusCode, Json<ApiError>)> {
    authz::require_auth(&state, &headers).await?;
    let rows = sqlx::query(
        "SELECT address_unit_id, town_code, address_unit_name FROM address_units WHERE town_code = $1 ORDER BY address_unit_id",
    )
    .bind(q.town_code.trim())
    .fetch_all(&state.db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "query address units failed"))?;
    let address_units: Vec<AddressUnitRow> = rows
        .iter()
        .map(|r| AddressUnitRow {
            address_unit_id: r.get("address_unit_id"),
            town_code: r.get("town_code"),
            address_unit_name: r.get("address_unit_name"),
        })
        .collect();
    Ok(Json(ok(address_units)))
}
