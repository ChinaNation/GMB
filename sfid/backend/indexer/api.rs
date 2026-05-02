//! Indexer API 路由：提供钱包交易记录查询接口。

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{api_error, ApiResponse, AppState, StoreBackend};

use super::db;

#[derive(Deserialize)]
pub(crate) struct TxQuery {
    pub limit: Option<i64>,
    pub before_id: Option<i64>,
    pub tx_type: Option<String>,
}

#[derive(Serialize)]
struct TxRecordOutput {
    id: i64,
    block_number: i64,
    tx_type: String,
    direction: &'static str,
    from_address: Option<String>,
    to_address: Option<String>,
    amount_yuan: f64,
    fee_yuan: Option<f64>,
    block_timestamp: Option<String>,
}

#[derive(Serialize)]
struct TxListOutput {
    records: Vec<TxRecordOutput>,
    has_more: bool,
}

/// GET /api/v1/app/wallet/:address/transactions
pub(crate) async fn wallet_transactions(
    State(state): State<AppState>,
    Path(address): Path<String>,
    Query(query): Query<TxQuery>,
) -> impl IntoResponse {
    let address = address.trim().to_string();
    if address.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, 1001, "address is required");
    }

    let limit = query.limit.unwrap_or(20).max(1).min(100);
    // 多查一条以判断 has_more
    let fetch_limit = limit + 1;

    let StoreBackend::Postgres {
        clients,
        next_client_idx,
    } = &state.store.backend
    else {
        return api_error(
            StatusCode::SERVICE_UNAVAILABLE,
            1500,
            "indexer not available",
        );
    };

    let result = StoreBackend::with_postgres_client(clients, next_client_idx, |conn| {
        db::query_tx_records(
            conn,
            &address,
            query.before_id,
            query.tx_type.as_deref(),
            fetch_limit,
        )
    });

    let rows = match result {
        Ok(r) => r,
        Err(err) => {
            tracing::warn!(error = %err, "query tx_records failed");
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1500, "query failed");
        }
    };

    let has_more = rows.len() as i64 > limit;
    let rows_to_return = if has_more {
        &rows[..limit as usize]
    } else {
        &rows
    };

    let records: Vec<TxRecordOutput> = rows_to_return
        .iter()
        .map(|r| {
            let direction =
                determine_direction(&address, r.from_address.as_deref(), r.to_address.as_deref());
            TxRecordOutput {
                id: r.id,
                block_number: r.block_number,
                tx_type: r.tx_type.clone(),
                direction,
                from_address: r.from_address.clone(),
                to_address: r.to_address.clone(),
                amount_yuan: r.amount_fen as f64 / 100.0,
                fee_yuan: r.fee_fen.map(|f| f as f64 / 100.0),
                block_timestamp: r.block_timestamp.map(|ts| ts.to_rfc3339()),
            }
        })
        .collect();

    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: TxListOutput { records, has_more },
    })
    .into_response()
}

/// 判断交易方向：out = 我是付款方，in = 我是收款方。
fn determine_direction(my_address: &str, from: Option<&str>, to: Option<&str>) -> &'static str {
    if from.is_some_and(|f| f == my_address) {
        "out"
    } else if to.is_some_and(|t| t == my_address) {
        "in"
    } else {
        // 无 from/to 的系统事件（如 gov_issuance 汇总事件）
        "info"
    }
}
