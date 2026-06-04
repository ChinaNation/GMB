//! 机构账户 HTTP handler
//!
//! 中文注释:本模块只承载机构账户新增、查询和删除;机构新增归 private,
//! 公权确定性目录归 gov,资料库归 docs,主体详情归 subjects::admin。
//!
//! ## 当前路由表(admin 端,login 中间件)
//!
//! - POST   /api/v1/institution/:sfid_number/account/create   → create_account
//! - GET    /api/v1/institution/:sfid_number/accounts         → list_accounts
//! - DELETE /api/v1/institution/:sfid_number/account/:account_name → delete_account
//!
//! ## 链端公开查询
//!
//! 区块链 / 钱包 pull 用的"机构信息查询"5 个 endpoint(无鉴权)已搬到
//! `crate::subjects::chain_duoqian_info::*`,本文件不再持有。

#![allow(dead_code)]

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::Serialize;

use crate::admins::actions::require_admin_security_grant;
use crate::admins::login::require_admin_any;
use crate::admins::operation_auth::AdminActionType;
use crate::core::response::ApiResponse;
use crate::scope::get_visible_scope;
use crate::subjects::http::{
    append_inst_audit_log_best_effort, cache_institution_detail_best_effort,
    read_institution_with_accounts_from_store, resolve_province_from_sfid_number,
    service_error_to_response,
};
use crate::subjects::model::{
    account_key_to_string, CreateAccountInput, CreateAccountOutput, MultisigAccount,
};
use crate::subjects::service::{
    can_delete_account, is_default_account_name, validate_account_name, ServiceError,
};
use crate::subjects::MultisigChainStatus;
use crate::*;

// ─── 0. 机构名称查重(私权=全国唯一,公权=同城唯一) ──────────────

pub(crate) async fn create_account(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(sfid_number): Path<String>,
    Json(input): Json<CreateAccountInput>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let grant_payload = serde_json::json!({
        "target": sfid_number.clone(),
        "sfid_number": sfid_number.clone(),
        "account_name": input.account_name.clone(),
    });
    if let Err(resp) = require_admin_security_grant(
        &state,
        &headers,
        &ctx,
        AdminActionType::InstitutionCreateAccount,
        sfid_number.as_str(),
        Some(&grant_payload),
    ) {
        return resp;
    }
    let scope = get_visible_scope(&ctx);

    let account_name = match validate_account_name(&input.account_name) {
        Ok(v) => v,
        Err(e) => return service_error_to_response(e),
    };

    // 中文注释:从 sfid_number 解析省份后读取进程内分片缓存。
    let province = match resolve_province_from_sfid_number(&sfid_number) {
        Some(p) => p,
        None => {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "cannot resolve province from sfid_number",
            )
        }
    };

    // ── 机构存在 + scope 校验 + 账户名唯一性(从分片读)──
    let sfid_number_r = sfid_number.clone();
    let account_name_r = account_name.clone();
    let read_result = state
        .sharded_store
        .read_province(&province, move |shard| {
            let inst = shard.multisig_institutions.get(&sfid_number_r);
            let inst_clone = inst.cloned();
            let acc_key = account_key_to_string(&sfid_number_r, &account_name_r);
            let acc_exists = shard.multisig_accounts.contains_key(&acc_key);
            (inst_clone, acc_exists)
        })
        .await;
    let (inst_opt, mut acc_exists) = match read_result {
        Ok(v) => v,
        Err(e) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                &format!("shard read: {e}"),
            )
        }
    };
    let inst = match inst_opt {
        Some(i) => i,
        None => match read_institution_with_accounts_from_store(&state, &sfid_number) {
            Ok(Some((inst, accounts))) => {
                acc_exists = accounts
                    .iter()
                    .any(|account| account.account_name == account_name);
                cache_institution_detail_best_effort(&state, &inst, &accounts).await;
                inst
            }
            Ok(None) => return api_error(StatusCode::NOT_FOUND, 1004, "institution not found"),
            Err(e) => {
                tracing::warn!(sfid = %sfid_number, error = %e, "institution fallback read failed");
                return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "store read failed");
            }
        },
    };
    if !scope.includes_province(&inst.province) || !scope.includes_city(&inst.city) {
        return api_error(
            StatusCode::FORBIDDEN,
            1003,
            "institution out of current admin scope",
        );
    }
    if acc_exists {
        return service_error_to_response(ServiceError::Conflict(
            "account_name already exists under this institution",
        ));
    }

    // ── 写 NotOnChain 本地记录,**不触链** ──
    // SFID 只登记账户名称;账户激活必须来自链上机构注册/新增账户交易的同步结果。
    // 地址按 DUOQIAN_V1 本地派生(账户名 = "主账户"/"费用账户"/其他,分别走 0x00/0x01/0x05),
    // 不等链上 receipt;链上同步时会再次断言地址一致。
    let now = Utc::now();
    let account = MultisigAccount {
        sfid_number: sfid_number.clone(),
        account_name: account_name.clone(),
        duoqian_address: crate::accounts::derive::derive_duoqian_address(
            &sfid_number,
            &account_name,
        ),
        chain_status: MultisigChainStatus::NotOnChain,
        chain_synced_at: None,
        chain_tx_hash: None,
        chain_block_number: None,
        created_by: ctx.admin_pubkey.clone(),
        created_at: now,
    };
    let acc_for_shard = account.clone();
    let sfid_number_w = sfid_number.clone();
    let account_name_w = account_name.clone();
    if let Err(e) = state
        .sharded_store
        .write_province(&province, move |shard| {
            let key = account_key_to_string(&sfid_number_w, &account_name_w);
            shard.multisig_accounts.insert(key, acc_for_shard);
        })
        .await
    {
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            &format!("shard write: {e}"),
        );
    }
    if let Err(e) = state.store.upsert_institution_account_row(&account) {
        tracing::error!(error = %e, "institution account row upsert failed");
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "institution account row write failed",
        );
    }

    // 中文注释:同步写模块 Store 快照,保持后台反查与审计读取口径一致。
    {
        let acc_key = account_key_to_string(&sfid_number, &account_name);
        match state.store.write() {
            Ok(mut store) => {
                store.multisig_accounts.insert(acc_key, account.clone());
            }
            Err(e) => {
                tracing::warn!(error = %e, "module store snapshot write failed (account create, shard already committed)");
            }
        }
    }

    append_inst_audit_log_best_effort(
        &state,
        "ACCOUNT_CREATE_NOT_ON_CHAIN",
        &ctx.admin_pubkey,
        Some(sfid_number.clone()),
        None,
        "SUCCESS",
        format!("sfid={} account_name={}", sfid_number, account_name),
    );

    let duoqian_address = account.duoqian_address.clone();
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: CreateAccountOutput {
            sfid_number,
            account_name,
            chain_status: MultisigChainStatus::NotOnChain,
            chain_synced_at: None,
            chain_tx_hash: None,
            chain_block_number: None,
            duoqian_address,
        },
    })
    .into_response()
}

// ─── 3. 列出机构(按 scope 过滤)──────────────────────────────────

pub(crate) async fn list_accounts(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(sfid_number): Path<String>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let scope = get_visible_scope(&ctx);

    // 中文注释:从 sfid_number 解析省份后读取进程内分片缓存。
    let province = match resolve_province_from_sfid_number(&sfid_number) {
        Some(p) => p,
        None => {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "cannot resolve province from sfid_number",
            )
        }
    };
    let sfid_number_r = sfid_number.clone();
    let read_result = state
        .sharded_store
        .read_province(&province, move |shard| {
            let inst = shard.multisig_institutions.get(&sfid_number_r).cloned();
            let accounts: Vec<MultisigAccount> = shard
                .multisig_accounts
                .values()
                .filter(|a| a.sfid_number == sfid_number_r)
                .cloned()
                .collect();
            (inst, accounts)
        })
        .await;
    let (inst_opt, accounts) = match read_result {
        Ok(v) => v,
        Err(e) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                &format!("shard read: {e}"),
            )
        }
    };
    let (inst, accounts) = match inst_opt {
        Some(i) => (i, accounts),
        None => match read_institution_with_accounts_from_store(&state, &sfid_number) {
            Ok(Some((inst, accounts))) => {
                cache_institution_detail_best_effort(&state, &inst, &accounts).await;
                (inst, accounts)
            }
            Ok(None) => return api_error(StatusCode::NOT_FOUND, 1004, "institution not found"),
            Err(e) => {
                tracing::warn!(sfid = %sfid_number, error = %e, "institution fallback read failed");
                return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "store read failed");
            }
        },
    };
    if !scope.includes_province(&inst.province) || !scope.includes_city(&inst.city) {
        return api_error(StatusCode::FORBIDDEN, 1003, "out of admin scope");
    }
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: accounts,
    })
    .into_response()
}

// ─── 5b/扫码支付/清算行候选搜索 已搬 subjects/chain_duoqian_info.rs ────────
//
// 5 个 endpoint 全部整合到 subjects/chain_duoqian_info.rs:
//   - app_search_institutions / app_get_institution / app_list_accounts
//   - app_search_clearing_banks / app_search_eligible_clearing_banks
//
// 历史 sync_institution_chain_state(POST /app/institutions/:sfid_number/chain-sync)
// 0 caller,与 SFID 不再读链铁律冲突,2026-05-01 一并下架。
//
// 调用入口现走 `crate::subjects::chain_duoqian_info::*` 重新导出。

// ─── 6. 删除账户(软删,不触链)──────────────────────────────────

pub(crate) async fn delete_account(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((sfid_number, account_name)): Path<(String, String)>,
) -> impl IntoResponse {
    let ctx = match require_admin_any(&state, &headers) {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let scope = get_visible_scope(&ctx);

    // 默认账户(主账户 / 费用账户)保护:禁止删除
    // 这两个账户每家机构都自动生成,绑定业务语义(主账户 = Role::Main, 费用账户 = Role::Fee),
    // 不允许从 sfid 系统层面移除;只有删除整个 SFID 时才随机构一起消失。
    if is_default_account_name(&account_name) {
        return api_error(
            StatusCode::CONFLICT,
            1007,
            "默认账户(主账户/费用账户)不可删除",
        );
    }

    // 中文注释:从 sfid_number 解析省份后操作进程内分片缓存。
    let province = match resolve_province_from_sfid_number(&sfid_number) {
        Some(p) => p,
        None => {
            return api_error(
                StatusCode::BAD_REQUEST,
                1001,
                "cannot resolve province from sfid_number",
            )
        }
    };

    // 先读:校验机构存在 + scope + 账户状态。SFID 不能删除仍在链上的账户名称。
    let sfid_number_r = sfid_number.clone();
    let account_name_r = account_name.clone();
    let read_result = state
        .sharded_store
        .read_province(&province, move |shard| {
            let inst = shard.multisig_institutions.get(&sfid_number_r).cloned();
            let account = shard
                .multisig_accounts
                .get(&account_key_to_string(&sfid_number_r, &account_name_r))
                .cloned();
            (inst, account)
        })
        .await;
    let fallback_loaded = matches!(&read_result, Ok((None, _)));
    let (inst, account) = match read_result {
        Ok((Some(i), Some(a))) => (i, a),
        Ok((None, _)) => match read_institution_with_accounts_from_store(&state, &sfid_number) {
            Ok(Some((inst, accounts))) => {
                let account = accounts
                    .iter()
                    .find(|item| item.account_name == account_name)
                    .cloned();
                cache_institution_detail_best_effort(&state, &inst, &accounts).await;
                match account {
                    Some(account) => (inst, account),
                    None => return api_error(StatusCode::NOT_FOUND, 1004, "account not found"),
                }
            }
            Ok(None) => return api_error(StatusCode::NOT_FOUND, 1004, "institution not found"),
            Err(e) => {
                tracing::warn!(sfid = %sfid_number, error = %e, "institution fallback read failed");
                return api_error(StatusCode::INTERNAL_SERVER_ERROR, 1004, "store read failed");
            }
        },
        Ok((Some(_), None)) => return api_error(StatusCode::NOT_FOUND, 1004, "account not found"),
        Err(e) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                &format!("shard read: {e}"),
            )
        }
    };
    if !scope.includes_province(&inst.province) || !scope.includes_city(&inst.city) {
        return api_error(StatusCode::FORBIDDEN, 1003, "out of admin scope");
    }
    if !can_delete_account(&account) {
        return api_error(
            StatusCode::CONFLICT,
            1007,
            "账户仍在链上或处于上链中,不能在 SFID 系统删除",
        );
    }
    let grant_payload = serde_json::json!({
        "target": sfid_number.clone(),
        "sfid_number": sfid_number.clone(),
        "account_name": account_name.clone(),
    });
    if let Err(resp) = require_admin_security_grant(
        &state,
        &headers,
        &ctx,
        AdminActionType::InstitutionDeleteAccount,
        sfid_number.as_str(),
        Some(&grant_payload),
    ) {
        return resp;
    }

    // 写:删除账户
    let sfid_number_w = sfid_number.clone();
    let account_name_w = account_name.clone();
    let remove_result = state
        .sharded_store
        .write_province(&province, move |shard| {
            let key = account_key_to_string(&sfid_number_w, &account_name_w);
            shard.multisig_accounts.remove(&key)
        })
        .await;
    match remove_result {
        Ok(None) if !fallback_loaded => {
            return api_error(StatusCode::NOT_FOUND, 1004, "account not found")
        }
        Ok(None) => {}
        Err(e) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                1004,
                &format!("shard write: {e}"),
            )
        }
        Ok(Some(_)) => {}
    }

    // 同步写模块 Store 快照,供审计与管理员反查读取同一份账户快照。
    {
        let acc_key = account_key_to_string(&sfid_number, &account_name);
        match state.store.write() {
            Ok(mut store) => {
                store.multisig_accounts.remove(&acc_key);
            }
            Err(e) => {
                tracing::warn!(error = %e, "module store snapshot write failed (account delete, shard already committed)");
            }
        }
    }
    if let Err(e) = state
        .store
        .delete_institution_account_row(&sfid_number, &account_name)
    {
        tracing::error!(sfid = %sfid_number, account = %account_name, error = %e, "institution account row delete failed");
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            1004,
            "institution account row delete failed",
        );
    }

    append_inst_audit_log_best_effort(
        &state,
        "ACCOUNT_DELETE",
        &ctx.admin_pubkey,
        Some(sfid_number.clone()),
        None,
        "SUCCESS",
        format!("sfid={} account={}", sfid_number, account_name),
    );
    #[derive(Serialize)]
    struct Ok {
        deleted: bool,
    }
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data: Ok { deleted: true },
    })
    .into_response()
}
