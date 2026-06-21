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
            name: p.name.to_string(),
            code: p.code.to_string(),
        })
        .collect();
    provinces_rows.sort_by(|a, b| a.code.cmp(&b.code));
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: AdminCidMetaOutput {
            subject_property_options: vec![
                CidOptionItem {
                    label: "公民",
                    value: "M",
                },
                CidOptionItem {
                    label: "自然人",
                    value: "Z",
                },
                CidOptionItem {
                    label: "智能人",
                    value: "N",
                },
                CidOptionItem {
                    label: "公法人",
                    value: "G",
                },
                CidOptionItem {
                    label: "私法人",
                    value: "S",
                },
                CidOptionItem {
                    label: "非法人",
                    value: "F",
                },
            ],
            institution_options: vec![
                CidOptionItem {
                    label: "中国",
                    value: "ZG",
                },
                CidOptionItem {
                    label: "政府",
                    value: "ZF",
                },
                CidOptionItem {
                    label: "立法院",
                    value: "LF",
                },
                CidOptionItem {
                    label: "司法院",
                    value: "SF",
                },
                CidOptionItem {
                    label: "监察院",
                    value: "JC",
                },
                CidOptionItem {
                    label: "公民教育委员会",
                    value: "JY",
                },
                CidOptionItem {
                    label: "公民储备委员会",
                    value: "CB",
                },
                CidOptionItem {
                    label: "公民储备银行",
                    value: "CH",
                },
                CidOptionItem {
                    label: "他国",
                    value: "TG",
                },
                CidOptionItem {
                    label: "个体经营",
                    value: "GT",
                },
                CidOptionItem {
                    label: "无限合伙",
                    value: "GP",
                },
                CidOptionItem {
                    label: "有限合伙",
                    value: "LP",
                },
                CidOptionItem {
                    label: "股权公司",
                    value: "GQ",
                },
                CidOptionItem {
                    label: "股份公司",
                    value: "GF",
                },
                CidOptionItem {
                    label: "公益组织",
                    value: "GY",
                },
                CidOptionItem {
                    label: "注册协会",
                    value: "AS",
                },
            ],
            provinces: provinces_rows,
            scoped_province_name: scoped,
        },
    })
    .into_response()
}
