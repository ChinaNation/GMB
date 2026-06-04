//! 主体 HTTP handler 共享辅助函数。
//!
//! 中文注释:公权机构、私权机构、账户、资料库和主体详情都会用到同一批
//! HTTP 层辅助逻辑。集中在这里,避免各业务目录复制一份后出现注释和行为漂移。

use axum::http::StatusCode;

use crate::accounts::derive::derive_duoqian_address;
use crate::app_core::runtime_ops::append_audit_log;
use crate::china::province_name_by_code;
use crate::crypto::pubkey::normalize_admin_pubkey;
use crate::login::AdminAuthContext;
use crate::subjects::model::{account_key_to_string, MultisigAccount, MultisigInstitution};
use crate::subjects::service::{ServiceError, DEFAULT_ACCOUNT_NAMES};
use crate::subjects::MultisigChainStatus;
use crate::{api_error, AppState};

pub(crate) const MAX_PROVINCE_CHARS: usize = 100;
pub(crate) const MAX_CITY_CHARS: usize = 100;

pub(crate) fn service_error_to_response(e: ServiceError) -> axum::response::Response {
    let status = match e {
        ServiceError::BadInput(_) => StatusCode::BAD_REQUEST,
        ServiceError::NotFound(_) => StatusCode::NOT_FOUND,
        ServiceError::Conflict(_) => StatusCode::CONFLICT,
    };
    let code = match e {
        ServiceError::BadInput(_) => 1001,
        ServiceError::NotFound(_) => 1004,
        ServiceError::Conflict(_) => 1007,
    };
    api_error(status, code, e.message())
}

pub(crate) fn extract_province_code(sfid: &str) -> String {
    sfid.split('-')
        .nth(1)
        .map(|r5| r5[..2.min(r5.len())].to_string())
        .unwrap_or_default()
}

pub(crate) fn extract_city_code(sfid: &str) -> String {
    // r5 = 省代码 2 字符 + 市代码 3 字符。
    sfid.split('-')
        .nth(1)
        .and_then(|r5| {
            if r5.len() >= 5 {
                Some(r5[2..5].to_string())
            } else {
                None
            }
        })
        .unwrap_or_default()
}

/// 从 sfid_number 解析省代码并映射到省名。
/// 用于 handler 层确定进程内分片缓存 key。
pub(crate) fn resolve_province_from_sfid_number(sfid_number: &str) -> Option<String> {
    let code = extract_province_code(sfid_number);
    if code.is_empty() {
        return None;
    }
    province_name_by_code(&code).map(|n| n.to_string())
}

/// 机构详情、账户和资料库入口必须先确认机构存在,再按管理员 scope 放行。
pub(crate) fn ensure_institution_visible_to_admin(
    inst: &MultisigInstitution,
    ctx: &AdminAuthContext,
) -> Result<(), axum::response::Response> {
    if let Some(ref locked_province) = ctx.admin_province {
        if inst.province != *locked_province {
            return Err(api_error(
                StatusCode::FORBIDDEN,
                1003,
                "province out of scope",
            ));
        }
    }
    if let Some(ref locked_city) = ctx.admin_city {
        if inst.city != *locked_city {
            return Err(api_error(StatusCode::FORBIDDEN, 1003, "city out of scope"));
        }
    }
    Ok(())
}

/// 反查 `created_by` pubkey → (管理员姓名, 角色枚举字符串)。
/// 未命中两者均为 `None`,前端统一显示为“未知”。
pub(crate) fn resolve_created_by(
    state: &AppState,
    created_by: &str,
) -> (Option<String>, Option<String>) {
    let norm = match normalize_admin_pubkey(created_by) {
        Some(v) => v,
        None => return (None, None),
    };
    let store = match state.store.read() {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(error = %e, "resolve_created_by: store read failed");
            return (None, None);
        }
    };
    for user in store.admin_users_by_pubkey.values() {
        let Some(user_norm) = normalize_admin_pubkey(&user.admin_pubkey) else {
            continue;
        };
        if user_norm == norm {
            let role_str = match user.role {
                crate::models::AdminRole::ShengAdmin => "SHENG_ADMIN",
                crate::models::AdminRole::ShiAdmin => "SHI_ADMIN",
            };
            let name_opt = if user.admin_name.trim().is_empty() {
                None
            } else {
                Some(user.admin_name.clone())
            };
            return (name_opt, Some(role_str.to_string()));
        }
    }
    (None, None)
}

/// 给机构写入默认账户(`主账户` / `费用账户`)的未上链本地记录。
///
/// 中文注释:这是机构创建后的 best-effort 补充动作。机构主记录已经成功时,
/// 默认账户写失败只记日志,不回滚机构身份。
pub(crate) async fn insert_default_accounts_best_effort(
    state: &AppState,
    sfid_number: &str,
    province: &str,
    created_by: &str,
) {
    let now = chrono::Utc::now();
    let sfid_owned = sfid_number.to_string();
    let creator_owned = created_by.to_string();
    let write_result = state
        .sharded_store
        .write_province(province, move |shard| {
            for name in DEFAULT_ACCOUNT_NAMES {
                let key = account_key_to_string(&sfid_owned, name);
                let addr = derive_duoqian_address(&sfid_owned, name);
                shard
                    .multisig_accounts
                    .entry(key)
                    .or_insert_with(|| MultisigAccount {
                        sfid_number: sfid_owned.clone(),
                        account_name: (*name).to_string(),
                        duoqian_address: addr,
                        chain_status: MultisigChainStatus::NotOnChain,
                        chain_synced_at: None,
                        chain_tx_hash: None,
                        chain_block_number: None,
                        created_by: creator_owned.clone(),
                        created_at: now,
                    });
            }
        })
        .await;
    if let Err(e) = write_result {
        tracing::warn!(
            sfid = sfid_number,
            error = %e,
            "insert_default_accounts shard write failed; institution create already committed"
        );
        return;
    }
    if let Ok(mut store) = state.store.write() {
        for name in DEFAULT_ACCOUNT_NAMES {
            let key = account_key_to_string(sfid_number, name);
            let addr = derive_duoqian_address(sfid_number, name);
            store
                .multisig_accounts
                .entry(key)
                .or_insert_with(|| MultisigAccount {
                    sfid_number: sfid_number.to_string(),
                    account_name: (*name).to_string(),
                    duoqian_address: addr,
                    chain_status: MultisigChainStatus::NotOnChain,
                    chain_synced_at: None,
                    chain_tx_hash: None,
                    chain_block_number: None,
                    created_by: created_by.to_string(),
                    created_at: now,
                });
        }
    }
    for name in DEFAULT_ACCOUNT_NAMES {
        let account = MultisigAccount {
            sfid_number: sfid_number.to_string(),
            account_name: (*name).to_string(),
            duoqian_address: derive_duoqian_address(sfid_number, name),
            chain_status: MultisigChainStatus::NotOnChain,
            chain_synced_at: None,
            chain_tx_hash: None,
            chain_block_number: None,
            created_by: created_by.to_string(),
            created_at: now,
        };
        if let Err(e) = state.store.upsert_institution_account_row(&account) {
            tracing::warn!(sfid = sfid_number, account = *name, error = %e, "institution account row upsert failed");
        }
    }
}

/// 审计日志 best-effort 写入。失败只记 WARN,不影响已经提交的业务主流程。
#[allow(clippy::too_many_arguments)]
pub(crate) fn append_inst_audit_log_best_effort(
    state: &AppState,
    action: &'static str,
    actor_pubkey: &str,
    target_pubkey: Option<String>,
    target_archive_no: Option<String>,
    result: &'static str,
    detail: String,
) {
    match state.store.write() {
        Ok(mut store) => {
            append_audit_log(
                &mut store,
                action,
                actor_pubkey,
                target_pubkey,
                target_archive_no,
                result,
                detail,
            );
        }
        Err(e) => {
            tracing::warn!(action, error = %e, "append_audit_log failed (main write already committed)");
        }
    }
}
