use axum::{extract::State, http::HeaderMap, response::IntoResponse, Json};

use crate::cid::china::provinces;
use crate::cid::code as institution_code;
use crate::*;

// 中文注释:本文件只保留管理端 CID 编码元信息接口;城市列表由 cid::china::admin 提供。

pub(crate) async fn admin_cid_meta(
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
        .filter(|p| {
            scoped
                .as_deref()
                .map(|v| v == p.province_name)
                .unwrap_or(true)
        })
        .map(|p| CidProvinceItem {
            province_name: p.province_name.to_string(),
            province_code: p.province_code.to_string(),
        })
        .collect();
    provinces_rows.sort_by(|a, b| a.province_code.cmp(&b.province_code));
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AdminCidMetaOutput {
            // 机构码选项由 primitives code.rs 单源派生(92 码);
            // 机构类别由机构码派生,不单列。
            institution_options: institution_code::ALL_CODES
                .iter()
                .map(|c| CidInstitutionCodeItem {
                    institution_code: institution_code::institution_code_text(c)
                        .expect("known CID institution code"),
                    cid_short_name: institution_code::cid_short_name(c)
                        .expect("known CID institution code"),
                })
                .collect(),
            provinces: provinces_rows,
            scoped_province_name: scoped,
        },
    })
    .into_response()
}
