//! 运行期启动辅助和显式维护动作。
//!
//! 中文注释:本模块不再维护进程内聚合体。启动只初始化必要结构化数据,
//! 大型确定性目录只在显式命令或接口中对账。

use chrono::Utc;

use crate::admins::federal_registry_admins::{federal_registry_mains, federal_scope_province_name};
use crate::admins::repo;
use crate::crypto::pubkey::normalize_admin_account;
use crate::gov::service::{
    reconcile_gov_catalog_db, GovTargetKind, OfficialReconcileReport, OfficialReconcileScope,
};
use crate::{AdminUser, AppState, RegistryOrgCode};

pub(crate) fn ensure_builtin_federal_registry_admins(state: &AppState) {
    if let Err(err) = state.db.with_client(|conn| {
        for item in federal_registry_mains() {
            let admin_account = normalize_admin_account(item.admin_account)
                .unwrap_or_else(|| item.admin_account.trim().to_ascii_lowercase());
            let province = federal_scope_province_name(item.admin_account)
                .unwrap_or(item.province_name)
                .to_string();
            let existing = repo::get_admin_by_account_conn(conn, admin_account.as_str())?;
            let id = existing
                .as_ref()
                .map(|admin| admin.id)
                .unwrap_or(repo::next_admin_id_conn(conn)?);
            let admin = AdminUser {
                id,
                admin_account: admin_account,
                admin_display_name: format!("{}联邦注册局管理员", item.province_name),
                registry_org_code: RegistryOrgCode::FederalRegistry,
                built_in: true,
                created_by: "SYSTEM".to_string(),
                created_at: Utc::now(),
                updated_at: existing.and_then(|admin| admin.updated_at),
                city_name: String::new(),
            };
            repo::upsert_admin_conn(conn, &admin, Some(province.as_str()))?;
        }
        Ok(())
    }) {
        tracing::error!(error = %err, "ensure builtin federal registry admins failed");
    }
}

pub(crate) fn cleanup_stale_citizen_bind_records(state: &AppState) -> usize {
    let now = Utc::now();
    state
        .db
        .with_client(move |conn| {
            let affected = conn
                .execute(
                    "DELETE FROM citizen_bind_challenges WHERE expires_at < $1",
                    &[&now],
                )
                .map_err(|e| format!("cleanup citizen bind challenges failed: {e}"))?;
            Ok(usize::try_from(affected).unwrap_or(0))
        })
        .unwrap_or(0)
}

pub(crate) fn reconcile_official_institutions_explicit(
    state: &AppState,
    scope: OfficialReconcileScope,
    _force_row_sync: bool,
) -> Result<OfficialReconcileReport, String> {
    reconcile_gov_catalog_db(&state.db, "SYSTEM", scope, GovTargetKind::All)
}

/// 中文注释:审计日志只存"事实"——detail 是结构化 JSON(键小写蛇形,值存系统原值:
/// 代码/布尔/原始字段),不得写展示文案;人话翻译统一归前端操作记录渲染器
/// (OperationRecords 的键名/值映射),文案改版零后端改动且历史行同样适用。
#[allow(clippy::too_many_arguments)]
pub(crate) fn append_audit_log(
    state: &AppState,
    action: &'static str,
    actor_account: &str,
    target_sfid: Option<String>,
    detail: serde_json::Value,
) {
    let actor = actor_account.to_string();
    let action = action.to_string();
    let log_action = action.clone();
    let province_code = target_sfid
        .as_deref()
        .and_then(|sfid| sfid.split('-').next())
        .map(|r5| r5[..2.min(r5.len())].to_string())
        .unwrap_or_else(|| "ZS".to_string());
    let city_code = target_sfid
        .as_deref()
        .and_then(|sfid| sfid.split('-').next())
        .and_then(|r5| (r5.len() >= 5).then(|| r5[2..5].to_string()))
        .filter(|v| !v.is_empty() && v != "000");
    if let Err(err) = state.db.with_client(move |conn| {
        conn.execute(
            "INSERT INTO audit(province_code, city_code, actor, action, target_sfid, detail)
             VALUES ($1, $2, $3, $4, $5, $6)",
            &[
                &province_code,
                &city_code,
                &actor,
                &action,
                &target_sfid,
                &detail,
            ],
        )
        .map_err(|e| format!("insert audit failed: {e}"))?;
        Ok(())
    }) {
        tracing::warn!(action = %log_action, error = %err, "append audit failed");
    }
}
