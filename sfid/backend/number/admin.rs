use axum::{extract::State, http::HeaderMap, response::IntoResponse, Json};

use crate::china::provinces;
use crate::*;

// 中文注释:本文件只保留管理端 SFID 编码元信息接口;城市列表由 china::admin 提供。

pub(crate) async fn admin_number_meta(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let admin_ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let scoped = admin_ctx.admin_province.clone();
    let mut provinces_rows: Vec<SfidProvinceItem> = provinces()
        .iter()
        .filter(|p| scoped.as_deref().map(|v| v == p.name).unwrap_or(true))
        .map(|p| SfidProvinceItem {
            name: p.name.to_string(),
            code: p.code.to_string(),
        })
        .collect();
    provinces_rows.sort_by(|a, b| a.code.cmp(&b.code));
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AdminSfidMetaOutput {
            a3_options: vec![
                SfidOptionItem {
                    label: "公民人",
                    value: "GMR",
                },
                SfidOptionItem {
                    label: "自然人",
                    value: "ZRR",
                },
                SfidOptionItem {
                    label: "智能人",
                    value: "ZNR",
                },
                SfidOptionItem {
                    label: "公法人",
                    value: "GFR",
                },
                SfidOptionItem {
                    label: "私法人",
                    value: "SFR",
                },
                SfidOptionItem {
                    label: "非法人",
                    value: "FFR",
                },
            ],
            institution_options: vec![
                SfidOptionItem {
                    label: "中国",
                    value: "ZG",
                },
                SfidOptionItem {
                    label: "政府",
                    value: "ZF",
                },
                SfidOptionItem {
                    label: "立法院",
                    value: "LF",
                },
                SfidOptionItem {
                    label: "司法院",
                    value: "SF",
                },
                SfidOptionItem {
                    label: "监察院",
                    value: "JC",
                },
                SfidOptionItem {
                    label: "公民教育委员会",
                    value: "JY",
                },
                SfidOptionItem {
                    label: "公民储备委员会",
                    value: "CB",
                },
                SfidOptionItem {
                    label: "公民储备银行",
                    value: "CH",
                },
                SfidOptionItem {
                    label: "他国",
                    value: "TG",
                },
            ],
            provinces: provinces_rows,
            scoped_province: scoped,
        },
    })
    .into_response()
}
