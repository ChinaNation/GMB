//! 运行期启动辅助和显式维护动作。
//!
//! 公权机构不在这里生成或对账;其唯一真源是链上 PublicManage,
//! 本地只通过 gov::service 同步链投影缓存。

use crate::AppState;

/// 审计日志只存"事实"——detail 是结构化 JSON(键小写蛇形,值存系统原值:
/// 代码/布尔/原始字段),不得写展示文案;人话翻译统一归前端操作记录渲染器
/// (OperationRecords 的键名/值映射),文案改版零后端改动且历史行同样适用。
#[allow(clippy::too_many_arguments)]
pub(crate) fn append_audit_log(
    state: &AppState,
    action: &'static str,
    actor_account_id: &str,
    target_cid: Option<String>,
    detail: serde_json::Value,
) {
    let actor_account_id = crate::crypto::pubkey::normalize_account_id(actor_account_id);
    let action = action.to_string();
    let log_action = action.clone();
    let province_code = target_cid
        .as_deref()
        .and_then(|cid| cid.split('-').next())
        .map(|r5| r5[..2.min(r5.len())].to_string())
        .unwrap_or_else(|| "ZS".to_string());
    let city_code = target_cid
        .as_deref()
        .and_then(|cid| cid.split('-').next())
        .and_then(|r5| (r5.len() >= 5).then(|| r5[2..5].to_string()))
        .filter(|v| !v.is_empty() && v != "000");
    if let Err(err) = state.db.with_client(move |conn| {
        conn.execute(
            "INSERT INTO audit(
                province_code, city_code, actor_account_id, action, target_cid, detail
             )
             VALUES ($1, $2, $3, $4, $5, $6)",
            &[
                &province_code,
                &city_code,
                &actor_account_id,
                &action,
                &target_cid,
                &detail,
            ],
        )
        .map_err(|e| format!("insert audit failed: {e}"))?;
        Ok(())
    }) {
        tracing::warn!(action = %log_action, error = %err, "append audit failed");
    }
}
