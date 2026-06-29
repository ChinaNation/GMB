//! 登录链上集合鉴权 + 会话签发(QR 登录与挑战登录共用)。
//!
//! 中文注释(去中心化鉴权):
//! - 验签证明扫码者持有 `signer_pubkey` 私钥后,membership 真源切到**链上 Active 管理员集合**
//!   (`GenesisAdmins`/`PublicAdmins`/`PrivateAdmins::AdminAccounts`),与直设入口同源;
//!   本地 admins 表降级为元数据 / 省映射缓存。
//! - 机构码由本节点链上身份推导;省/市 scope 统一取节点 `CID_RUNTIME_SCOPE_*`(每节点单省;
//!   Tier1 创世注册局省维度同此,`federal_registry_scope` 本地省映射表已退役,决策③)。
//! - 后台 `revoke_stale_admin_sessions_loop` 周期复查,管理员被链上移除后≤TTL 失效。

use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use crate::auth::repo;
use crate::core::chain_runtime;
use crate::crypto::pubkey::same_admin_account;
use crate::*;

use super::model::{AdminIdentifyOutput, AdminSession};
use super::signature::build_admin_name_from_user;

/// 链上集合鉴权失败分类(映射 HTTP 状态)。
pub(super) enum GateError {
    /// 扫码者不在本机构链上 Active 管理员集合。
    NotOnchainAdmin,
    /// 链节点不可达 / 读取失败(瞬时,允许重试,绝不降级查本地表)。
    ChainUnreachable(String),
    /// 节点机构身份环境变量缺失或非法。
    Config(String),
    /// 本地元数据 / 会话落库失败。
    Db(String),
}

pub(super) fn gate_error_response(err: GateError) -> axum::response::Response {
    use axum::http::StatusCode;
    match err {
        GateError::NotOnchainAdmin => {
            api_error(StatusCode::FORBIDDEN, 2002, "not an on-chain admin")
        }
        GateError::ChainUnreachable(message) => {
            tracing::warn!(error = %message, "chain unreachable during login gate");
            api_error(StatusCode::BAD_GATEWAY, 5002, "chain unreachable")
        }
        GateError::Config(message) => {
            tracing::error!(error = %message, "node institution identity misconfigured");
            api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "node identity misconfigured",
            )
        }
        GateError::Db(message) => {
            tracing::error!(error = %message, "login gate db error");
            api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                5001,
                "login persist failed",
            )
        }
    }
}

/// 已验签的 pubkey 经链上集合鉴权后落本地元数据并签发会话。
///
/// 返回 `(access_token, expire_at, AdminIdentifyOutput)`;调用方按各自登录流(QR / 挑战)回包。
pub(super) async fn issue_session_after_onchain_gate(
    state: &AppState,
    verified_pubkey: &str,
    now: DateTime<Utc>,
) -> Result<(String, DateTime<Utc>, AdminIdentifyOutput), GateError> {
    // 1) 本节点链上机构身份。
    let identity = chain_runtime::node_institution_identity().map_err(GateError::Config)?;

    // 2) 读链上 Active 管理员集合(账户不存在/非 Active → 拒绝)。
    let onchain_admins = chain_runtime::fetch_active_admins_onchain(&identity)
        .await
        .map_err(GateError::ChainUnreachable)?
        .ok_or(GateError::NotOnchainAdmin)?;

    // 3) membership 比对(统一规整成 0x 小写 hex 后逐一比较)。
    let normalized = chain_runtime::normalize_account_pubkey(verified_pubkey)
        .ok_or(GateError::NotOnchainAdmin)?;
    if !onchain_admins
        .iter()
        .any(|admin| same_admin_account(admin, normalized.as_str()))
    {
        return Err(GateError::NotOnchainAdmin);
    }

    let institution_code = chain_runtime::institution_code_label(&identity.institution_code);
    let is_federal_registry = chain_runtime::is_tier1_registry(&institution_code);
    let node_province = chain_runtime::node_scope_province();
    let node_city = chain_runtime::node_scope_city();
    let node_town = chain_runtime::node_scope_town();
    let pubkey_for_db = normalized.clone();

    // 纵深校验:非 Tier1 创世注册局机构的省/市/镇作用域来自节点 CID_RUNTIME_SCOPE_* env;在此 env→会话
    // 唯一写入边界核对其落在 china.sqlite 行政区真源内(省存在、市属省、镇属市,逐级要求上级齐备),
    // 不一致即节点配置错误——拒登录显式暴露,绝不带错位作用域签发会话。Tier1 创世注册局省级、无市镇维度,
    // 其省码已在 node_institution_identity 经 PROVINCE_CODE_INFOS 校验,不在此列。
    if !is_federal_registry {
        if let Some(province) = node_province.as_deref() {
            if crate::cid::china::province_code_by_name(province).is_none() {
                return Err(GateError::Config(format!(
                    "CID_RUNTIME_SCOPE_PROVINCE_NAME '{province}' not found in china.sqlite"
                )));
            }
            if let Some(city) = node_city.as_deref() {
                if crate::cid::china::city_code_by_name(province, city).is_none() {
                    return Err(GateError::Config(format!(
                        "CID_RUNTIME_SCOPE_CITY_NAME '{city}' not within province '{province}' in china.sqlite"
                    )));
                }
                if let Some(town) = node_town.as_deref() {
                    if crate::cid::china::town_code_by_name(province, city, town).is_none() {
                        return Err(GateError::Config(format!(
                            "CID_RUNTIME_SCOPE_TOWN_NAME '{town}' not within '{province}/{city}' in china.sqlite"
                        )));
                    }
                }
            } else if node_town.is_some() {
                return Err(GateError::Config(
                    "CID_RUNTIME_SCOPE_TOWN_NAME set without CID_RUNTIME_SCOPE_CITY_NAME"
                        .to_string(),
                ));
            }
        } else if node_city.is_some() || node_town.is_some() {
            return Err(GateError::Config(
                "CID_RUNTIME_SCOPE_CITY_NAME/TOWN_NAME set without CID_RUNTIME_SCOPE_PROVINCE_NAME"
                    .to_string(),
            ));
        }
    }

    // 4) 落本地元数据 + 签发会话(单事务)。
    let result = state
        .db
        .with_client(move |conn| {
            // 已有本地行优先(保留既有省映射 / 既有市行 id);否则按节点身份新建元数据行。
            let existing = repo::get_admin_by_account_conn(conn, pubkey_for_db.as_str())?;
            let admin = match existing {
                Some(mut current) => {
                    // 链上身份与本地登记冲突时,以链上机构码为准(去中心化真源)。
                    current.institution_code = institution_code.clone();
                    if !is_federal_registry {
                        if let Some(city) = node_city.clone() {
                            current.city_name = city;
                        }
                    }
                    current.updated_at = Some(now);
                    current
                }
                None => AdminUser {
                    id: repo::next_admin_id_conn(conn)?,
                    admin_account: pubkey_for_db.clone(),
                    admin_name: String::new(),
                    institution_code: institution_code.clone(),
                    built_in: false,
                    created_by: pubkey_for_db.clone(),
                    created_at: now,
                    updated_at: Some(now),
                    city_name: node_city.clone().unwrap_or_default(),
                },
            };

            // 中文注释:省映射不再随管理员落本地表(federal_registry_scope 已退役,决策③)。
            // Tier1 创世注册局管理员省作用域统一取节点 env(每节点单省;省码已在
            // `node_institution_identity` 经 PROVINCE_CODE_INFOS 校验)。本步只维护 admins 缓存本身。
            repo::upsert_admin_conn(conn, &admin)?;

            // 解析 scope:与会话重建(guards)共用 derive_admin_scope_conn 单一来源,口径一致。
            let (scope_province_name, scope_city_name, scope_town_name) =
                repo::derive_admin_scope_conn(conn, &admin.admin_account, &admin.institution_code)?;
            let admin_name = build_admin_name_from_user(&admin, scope_province_name.as_deref());
            let cid_short_name = repo::resolve_home_cid_short_name_conn(
                conn,
                &admin.institution_code,
                scope_province_name.as_deref(),
                scope_city_name.as_deref(),
            )?;

            let access_token = Uuid::new_v4().to_string();
            let expire_at = now + Duration::hours(8);
            repo::insert_admin_session_conn(
                conn,
                &AdminSession {
                    token: access_token.clone(),
                    admin_account: admin.admin_account.clone(),
                    institution_code: institution_code.clone(),
                    expire_at,
                    last_active_at: now,
                },
            )?;

            let admin_level = chain_runtime::admin_level_label_for(&institution_code);
            let capabilities = crate::platform::capability::capabilities_for(&institution_code);
            Ok((
                access_token,
                expire_at,
                AdminIdentifyOutput {
                    admin_account: admin.admin_account,
                    institution_code: institution_code.clone(),
                    admin_level,
                    capabilities,
                    admin_name,
                    scope_province_name,
                    scope_city_name,
                    scope_town_name,
                    cid_short_name,
                },
            ))
        })
        .map_err(GateError::Db)?;

    Ok(result)
}

/// 后台周期复查:把已不在本机构链上 Active 集合的管理员的会话清退。
///
/// 中文注释:管理员"失效即时生效"靠此扫描(默认 45s,`CID_ADMIN_ONCHAIN_REVOKE_SECONDS` 可调)。
/// 链不可达时跳过本轮(绝不因瞬时断链批量清退);账户不存在(None)亦跳过(链未就绪保守处理)。
pub(crate) async fn revoke_stale_admin_sessions_loop(db: Db) {
    let interval_secs = std::env::var("CID_ADMIN_ONCHAIN_REVOKE_SECONDS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(45);
    let mut ticker = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
    loop {
        ticker.tick().await;
        if let Err(err) = revoke_stale_admin_sessions_once(&db).await {
            tracing::warn!(error = %err, "on-chain admin session revocation sweep failed");
        }
    }
}

async fn revoke_stale_admin_sessions_once(db: &Db) -> Result<(), String> {
    let identity = chain_runtime::node_institution_identity()?;
    let Some(onchain_admins) = chain_runtime::fetch_active_admins_onchain(&identity).await? else {
        // 账户暂不存在(链未就绪/未配),不冒然清退本地会话。
        return Ok(());
    };
    let institution_code = chain_runtime::institution_code_label(&identity.institution_code);
    db.with_client(move |conn| {
        let accounts = repo::list_session_admin_accounts_conn(conn, &institution_code)?;
        for account in accounts {
            let still_admin = onchain_admins
                .iter()
                .any(|admin| same_admin_account(admin, account.as_str()));
            if !still_admin {
                let removed = repo::delete_admin_sessions_for_account_conn(conn, account.as_str())?;
                if removed > 0 {
                    tracing::info!(
                        admin_account = %account,
                        sessions = removed,
                        "revoked sessions for admin no longer on-chain"
                    );
                }
            }
        }
        Ok(())
    })
}
