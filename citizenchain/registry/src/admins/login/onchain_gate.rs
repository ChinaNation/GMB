//! 登录链上集合鉴权 + 会话签发(QR 登录与挑战登录共用)。
//!
//! 中文注释(3b 去中心化鉴权):
//! - 验签证明扫码者持有 `signer_pubkey` 私钥后,membership 真源切到**链上 Active 管理员集合**
//!   (`GenesisAdmins`/`PublicAdmins::AdminAccounts`),与 3a 直设入口同源;本地 admins 表降级为
//!   元数据 / 联邦省映射缓存。
//! - org(联邦/市)由本节点链上身份推导;省/市 scope:市节点取 `CID_RUNTIME_SCOPE_*`,
//!   联邦保留本地 `federal_registry_scope` 映射(链上不存省映射,锁定决策)。
//! - 后台 `revoke_stale_admin_sessions_loop` 周期复查,管理员被链上移除后≤TTL 失效。

use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use crate::admins::repo;
use crate::core::chain_runtime;
use crate::crypto::pubkey::same_admin_account;
use crate::*;

use super::model::{AdminIdentifyOutput, AdminSession};
use super::signature::{build_admin_name, resolve_scope_city_name};

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

fn node_scope_province() -> Option<String> {
    std::env::var("CID_RUNTIME_SCOPE_PROVINCE_NAME")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn node_scope_city() -> Option<String> {
    std::env::var("CID_RUNTIME_SCOPE_CITY_NAME")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
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
    let onchain_admins =
        chain_runtime::fetch_active_admins_onchain(identity.pallet, &identity.main_account)
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

    let org_code = if identity.is_federal {
        RegistryOrgCode::FederalRegistry
    } else {
        RegistryOrgCode::CityRegistry
    };
    let node_province = node_scope_province();
    let node_city = node_scope_city();
    let pubkey_for_db = normalized.clone();

    // 4) 落本地元数据 + 签发会话(单事务)。
    let result = state
        .db
        .with_client(move |conn| {
            // 已有本地行优先(保留联邦既有省映射 / 既有市行 id);否则按节点身份新建元数据行。
            let existing = repo::get_admin_by_account_conn(conn, pubkey_for_db.as_str())?;
            let admin = match existing {
                Some(mut current) => {
                    // 链上身份与本地登记冲突时,以链上 org 为准(去中心化真源)。
                    current.registry_org_code = org_code.clone();
                    if org_code == RegistryOrgCode::CityRegistry {
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
                    registry_org_code: org_code.clone(),
                    built_in: false,
                    created_by: pubkey_for_db.clone(),
                    created_at: now,
                    updated_at: Some(now),
                    city_name: node_city.clone().unwrap_or_default(),
                },
            };

            // 中文注释:联邦管理员省作用域是创世从 china_zf 一次性 bootstrap 进 federal_registry_scope 的
            // 降级元数据(链上只存成员资格,不存省映射,锁定决策)。这里**只取该映射**,
            // 绝不用节点级 CID_RUNTIME_SCOPE_PROVINCE_NAME 兜底——否则会静默退化成"全国"。
            // 查不到 = 创世 bootstrap 缺失(配置错误),宁可省名为空显式暴露,也不伪造省名。
            let province_for_upsert = if org_code == RegistryOrgCode::FederalRegistry {
                repo::province_scope_for_registry_org_conn(
                    conn,
                    &admin.admin_account,
                    &admin.registry_org_code,
                )?
            } else {
                None
            };
            repo::upsert_admin_conn(conn, &admin, province_for_upsert.as_deref())?;

            // 重新解析 scope(联邦省走本地映射,市走节点配置)。
            let scope_province_name = if org_code == RegistryOrgCode::FederalRegistry {
                repo::province_scope_for_registry_org_conn(
                    conn,
                    &admin.admin_account,
                    &admin.registry_org_code,
                )?
            } else {
                node_province.clone()
            };
            let scope_city_name = if org_code == RegistryOrgCode::CityRegistry {
                node_city.clone()
            } else {
                resolve_scope_city_name(&admin)
            };
            let admin_name = if admin.admin_name.trim().is_empty() {
                build_admin_name(
                    &admin.admin_account,
                    &admin.registry_org_code,
                    scope_province_name.as_deref(),
                )
            } else {
                admin.admin_name.clone()
            };
            let cid_short_name = repo::resolve_home_cid_short_name_conn(
                conn,
                &admin.registry_org_code,
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
                    registry_org_code: org_code.clone(),
                    expire_at,
                    last_active_at: now,
                },
            )?;

            Ok((
                access_token,
                expire_at,
                AdminIdentifyOutput {
                    admin_account: admin.admin_account,
                    registry_org_code: org_code,
                    admin_name,
                    scope_province_name,
                    scope_city_name,
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
    let Some(onchain_admins) =
        chain_runtime::fetch_active_admins_onchain(identity.pallet, &identity.main_account).await?
    else {
        // 账户暂不存在(链未就绪/未配),不冒然清退本地会话。
        return Ok(());
    };
    let org = if identity.is_federal {
        RegistryOrgCode::FederalRegistry
    } else {
        RegistryOrgCode::CityRegistry
    };
    db.with_client(move |conn| {
        let accounts = repo::list_session_admin_accounts_conn(conn, &org)?;
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
