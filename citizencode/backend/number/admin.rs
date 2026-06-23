use axum::{extract::State, http::HeaderMap, response::IntoResponse, Json};

use crate::china::provinces;
use crate::*;

// 中文注释:本文件只保留管理端 CID 编码元信息接口;城市列表由 china::admin 提供。

pub(crate) async fn admin_number_meta(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let admin_ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let scoped = admin_ctx.scope_province_name.clone();
    let mut provinces_rows: Vec<CidProvinceItem> = provinces()
        .iter()
        .filter(|p| scoped.as_deref().map(|v| v == p.name).unwrap_or(true))
        .map(|p| CidProvinceItem {
            province_name: p.name.to_string(),
            code: p.code.to_string(),
        })
        .collect();
    provinces_rows.sort_by(|a, b| a.code.cmp(&b.code));
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AdminCidMetaOutput {
            // 机构码选项由 InstitutionCode::ALL 单源派生(86 码,新版 T3/T4);
            // 主体属性已由机构码派生,不再单列(K1 旧输入已删)。
            institution_options: crate::number::InstitutionCode::ALL
                .iter()
                .map(|c| CidOptionItem {
                    label: c.label_zh(),
                    value: c.as_code(),
                })
                .collect(),
            provinces: provinces_rows,
            scoped_province_name: scoped,
        },
    })
    .into_response()
}
