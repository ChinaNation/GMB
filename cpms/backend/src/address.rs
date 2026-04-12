//! 地址管理模块：镇/村路的 seed + CRUD API。
//!
//! 内置佛坪市真实镇村数据（对应 sfid 工具 city_codes/01_ZS.rs），
//! 启动时 seed 到 DB（不覆盖超管已修改的数据）。
//! 超级管理员可增删改本市的镇和村/路。

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::{authz, err, ok, ApiError, ApiResponse, AppState};

// ── 内置数据（佛坪市 1 街道 + 6 镇 + 约 35 村） ──

struct SeedTown {
    code: &'static str,
    name: &'static str,
    villages: &'static [(&'static str, &'static str)], // (code, name)
}

// 佛坪市真实镇村数据（来源：国家统计局 2023 版行政区划，统一数据源）
const SEED_TOWNS: &[SeedTown] = &[
    SeedTown { code: "001", name: "袁家庄街道", villages: &[
        ("001", "袁家庄居委会"), ("002", "黄家湾村委会"), ("003", "袁家庄村委会"),
        ("004", "塘湾村委会"), ("005", "东岳殿村委会"), ("006", "王家湾村委会"),
        ("007", "肖家庄村委会"),
    ]},
    SeedTown { code: "002", name: "陈家坝镇", villages: &[
        ("001", "陈家坝村委会"), ("002", "孔家湾村委会"), ("003", "郭家坝村委会"),
        ("004", "金星村委会"), ("005", "三郎沟村委会"),
    ]},
    SeedTown { code: "003", name: "大河坝镇", villages: &[
        ("001", "五四村委会"), ("002", "十亩地村委会"), ("003", "三河口村委会"),
        ("004", "共力村委会"), ("005", "沙坪村委会"), ("006", "高桥村委会"),
        ("007", "水田坪村委会"), ("008", "谭家河村委会"), ("009", "凤凰村委会"),
        ("010", "联合村委会"),
    ]},
    SeedTown { code: "004", name: "西岔河镇", villages: &[
        ("001", "西岔河村委会"), ("002", "三教殿村委会"), ("003", "银厂沟村委会"),
        ("004", "故峪沟村委会"), ("005", "耖家庄村委会"), ("006", "彭家沟村委会"),
    ]},
    SeedTown { code: "005", name: "岳坝镇", villages: &[
        ("001", "岳坝村委会"), ("002", "草林村委会"), ("003", "龙潭村委会"),
        ("004", "大古坪村委会"), ("005", "栗子坝村委会"), ("006", "女儿坝村委会"),
        ("007", "西花村委会"), ("008", "狮子坝村委会"),
    ]},
    SeedTown { code: "006", name: "长角坝镇", villages: &[
        ("001", "两河口村委会"), ("002", "教场坝村委会"), ("003", "沙坝村委会"),
        ("004", "龙草坪村委会"), ("005", "田坝村委会"), ("006", "沙窝村委会"),
    ]},
    SeedTown { code: "007", name: "石墩河镇", villages: &[
        ("001", "石墩河村委会"), ("002", "薅林湾村委会"), ("003", "迴龙寺村委会"),
    ]},
];

/// 启动时 seed 内置镇村数据（INSERT ... ON CONFLICT DO NOTHING，不覆盖超管修改）。
pub(crate) async fn seed_builtin_address(db: &sqlx::PgPool) {
    for town in SEED_TOWNS {
        let _ = sqlx::query(
            "INSERT INTO address_towns (town_code, town_name) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(town.code)
        .bind(town.name)
        .execute(db)
        .await;

        for (vcode, vname) in town.villages {
            let vid = format!("{}-{}", town.code, vcode);
            let _ = sqlx::query(
                "INSERT INTO address_villages (village_id, town_code, village_name) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
            )
            .bind(&vid)
            .bind(town.code)
            .bind(vname)
            .execute(db)
            .await;
        }
    }
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

#[derive(Deserialize)]
struct CreateTownReq {
    town_code: String,
    town_name: String,
}

#[derive(Deserialize)]
struct CreateVillageReq {
    town_code: String,
    village_name: String,
}

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/address/towns", get(list_towns).post(create_town))
        .route("/api/v1/address/towns/:code", delete(delete_town))
        .route("/api/v1/address/villages", get(list_villages).post(create_village))
        .route("/api/v1/address/villages/:id", delete(delete_village))
}

async fn list_towns(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Vec<TownRow>>>, (StatusCode, Json<ApiError>)> {
    authz::require_auth(&state, &headers).await?;
    let rows = sqlx::query("SELECT town_code, town_name FROM address_towns ORDER BY town_code")
        .fetch_all(&state.db)
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "query towns failed"))?;
    let towns: Vec<TownRow> = rows.iter().map(|r| TownRow {
        town_code: r.get("town_code"),
        town_name: r.get("town_name"),
    }).collect();
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
    let villages: Vec<VillageRow> = rows.iter().map(|r| VillageRow {
        village_id: r.get("village_id"),
        town_code: r.get("town_code"),
        village_name: r.get("village_name"),
    }).collect();
    Ok(Json(ok(villages)))
}

async fn create_town(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateTownReq>,
) -> Result<Json<ApiResponse<TownRow>>, (StatusCode, Json<ApiError>)> {
    authz::require_role(&state, &headers, "SUPER_ADMIN").await?;
    let code = req.town_code.trim().to_string();
    let name = req.town_name.trim().to_string();
    if code.is_empty() || name.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "town_code and town_name required"));
    }
    sqlx::query("INSERT INTO address_towns (town_code, town_name) VALUES ($1, $2)")
        .bind(&code)
        .bind(&name)
        .execute(&state.db)
        .await
        .map_err(|_| err(StatusCode::CONFLICT, 3001, "town_code already exists"))?;
    Ok(Json(ok(TownRow { town_code: code, town_name: name })))
}

async fn delete_town(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Path(code): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiError>)> {
    authz::require_role(&state, &headers, "SUPER_ADMIN").await?;
    sqlx::query("DELETE FROM address_towns WHERE town_code = $1")
        .bind(code.trim())
        .execute(&state.db)
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "delete town failed"))?;
    Ok(Json(ok(serde_json::json!({}))))
}

async fn create_village(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateVillageReq>,
) -> Result<Json<ApiResponse<VillageRow>>, (StatusCode, Json<ApiError>)> {
    authz::require_role(&state, &headers, "SUPER_ADMIN").await?;
    let town_code = req.town_code.trim().to_string();
    let name = req.village_name.trim().to_string();
    if town_code.is_empty() || name.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, 1001, "town_code and village_name required"));
    }
    // 自动生成 village_id: town_code + 递增序号
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM address_villages WHERE town_code = $1",
    )
    .bind(&town_code)
    .fetch_one(&state.db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "count villages failed"))?;
    let vid = format!("{}-{:03}", town_code, count + 1);
    sqlx::query("INSERT INTO address_villages (village_id, town_code, village_name) VALUES ($1, $2, $3)")
        .bind(&vid)
        .bind(&town_code)
        .bind(&name)
        .execute(&state.db)
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "insert village failed"))?;
    Ok(Json(ok(VillageRow { village_id: vid, town_code, village_name: name })))
}

async fn delete_village(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiError>)> {
    authz::require_role(&state, &headers, "SUPER_ADMIN").await?;
    sqlx::query("DELETE FROM address_villages WHERE village_id = $1")
        .bind(id.trim())
        .execute(&state.db)
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, 5001, "delete village failed"))?;
    Ok(Json(ok(serde_json::json!({}))))
}
