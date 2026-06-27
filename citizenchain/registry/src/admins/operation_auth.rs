//! 管理端操作权限分级。
//!
//! 中文注释(3c):所有 CID 管理端操作归入 LOGIN_STATE / SCAN_SIGN 两类之一。
//! 安全动作入口(prepare/commit)只允许 SCAN_SIGN;登录态操作走各自业务 handler。

use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::admins::login::AdminAuthContext;
use crate::{api_error, RegistryOrgCode};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum AdminOperationAuth {
    /// 仅需有效会话(会话已是链上已证管理员)。
    LoginState,
    /// 会话 + 冷钱包扫码签名(signer 还须 ∈ 本机构链上 Active 集合)。
    ScanSign,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum AdminActionType {
    CreateCityRegistry,
    UpdateCityRegistry,
    DeleteCityRegistry,
    CreateFederalRegistry,
    UpdateFederalRegistry,
    DeleteFederalRegistry,
    InstitutionCreate,
    InstitutionUpdate,
    InstitutionCreateAccount,
    InstitutionDeleteAccount,
    /// 注销整个机构(关主账户=级联关全部账户);签发整机构 scope 注销凭证。
    InstitutionDeregister,
    /// 注销机构单个非主账户;签发单账户 scope 注销凭证。
    InstitutionAccountDeregister,
    InstitutionUploadDocument,
    InstitutionDeleteDocument,
}

impl AdminActionType {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::CreateCityRegistry => "CREATE_CITY_REGISTRY",
            Self::UpdateCityRegistry => "UPDATE_CITY_REGISTRY",
            Self::DeleteCityRegistry => "DELETE_CITY_REGISTRY",
            Self::CreateFederalRegistry => "CREATE_FEDERAL_REGISTRY",
            Self::UpdateFederalRegistry => "UPDATE_FEDERAL_REGISTRY",
            Self::DeleteFederalRegistry => "DELETE_FEDERAL_REGISTRY",
            Self::InstitutionCreate => "INSTITUTION_CREATE",
            Self::InstitutionUpdate => "INSTITUTION_UPDATE",
            Self::InstitutionCreateAccount => "INSTITUTION_CREATE_ACCOUNT",
            Self::InstitutionDeleteAccount => "INSTITUTION_DELETE_ACCOUNT",
            Self::InstitutionDeregister => "INSTITUTION_DEREGISTER",
            Self::InstitutionAccountDeregister => "INSTITUTION_ACCOUNT_DEREGISTER",
            Self::InstitutionUploadDocument => "INSTITUTION_UPLOAD_DOCUMENT",
            Self::InstitutionDeleteDocument => "INSTITUTION_DELETE_DOCUMENT",
        }
    }

    pub(crate) fn auth_type(&self) -> AdminOperationAuth {
        match self {
            // 中文注释(3c 决策 C):纯本地确认 / 元数据更新 → 仅会话(已链上已证)。
            Self::UpdateCityRegistry
            | Self::UpdateFederalRegistry
            | Self::InstitutionUpdate
            | Self::InstitutionUploadDocument => AdminOperationAuth::LoginState,
            // 产生链上交易/凭证、改注册权威或高危治理 → 冷钱包扫码签名。
            Self::InstitutionCreate
            | Self::InstitutionCreateAccount
            | Self::CreateCityRegistry
            | Self::DeleteCityRegistry
            | Self::CreateFederalRegistry
            | Self::DeleteFederalRegistry
            | Self::InstitutionDeleteAccount
            | Self::InstitutionDeregister
            | Self::InstitutionAccountDeregister
            | Self::InstitutionDeleteDocument => AdminOperationAuth::ScanSign,
        }
    }

    pub(crate) fn is_login_state(&self) -> bool {
        self.auth_type() == AdminOperationAuth::LoginState
    }

    pub(crate) fn is_governance(&self) -> bool {
        matches!(
            self,
            Self::CreateCityRegistry
                | Self::DeleteCityRegistry
                | Self::CreateFederalRegistry
                | Self::DeleteFederalRegistry
                | Self::InstitutionDeregister
                | Self::InstitutionAccountDeregister
        )
    }
}

pub(crate) fn parse_action_type(
    action_type: &str,
) -> Result<AdminActionType, axum::response::Response> {
    match action_type {
        "CREATE_CITY_REGISTRY" => Ok(AdminActionType::CreateCityRegistry),
        "UPDATE_CITY_REGISTRY" => Ok(AdminActionType::UpdateCityRegistry),
        "DELETE_CITY_REGISTRY" => Ok(AdminActionType::DeleteCityRegistry),
        "CREATE_FEDERAL_REGISTRY" => Ok(AdminActionType::CreateFederalRegistry),
        "UPDATE_FEDERAL_REGISTRY" => Ok(AdminActionType::UpdateFederalRegistry),
        "DELETE_FEDERAL_REGISTRY" => Ok(AdminActionType::DeleteFederalRegistry),
        "INSTITUTION_CREATE" => Ok(AdminActionType::InstitutionCreate),
        "INSTITUTION_UPDATE" => Ok(AdminActionType::InstitutionUpdate),
        "INSTITUTION_CREATE_ACCOUNT" => Ok(AdminActionType::InstitutionCreateAccount),
        "INSTITUTION_DELETE_ACCOUNT" => Ok(AdminActionType::InstitutionDeleteAccount),
        "INSTITUTION_DEREGISTER" => Ok(AdminActionType::InstitutionDeregister),
        "INSTITUTION_ACCOUNT_DEREGISTER" => Ok(AdminActionType::InstitutionAccountDeregister),
        "INSTITUTION_UPLOAD_DOCUMENT" => Ok(AdminActionType::InstitutionUploadDocument),
        "INSTITUTION_DELETE_DOCUMENT" => Ok(AdminActionType::InstitutionDeleteDocument),
        _ => Err(api_error(
            StatusCode::BAD_REQUEST,
            1001,
            "unknown action_type",
        )),
    }
}

pub(crate) fn ensure_action_role_allowed(
    ctx: &AdminAuthContext,
    action_type: &AdminActionType,
) -> Result<(), axum::response::Response> {
    if ctx.scope_province_name.is_none() {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "admin province scope missing",
        ));
    }
    if (action_type.is_governance() || action_type.is_login_state())
        && ctx.registry_org_code != RegistryOrgCode::FederalRegistry
    {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "federal admin required",
        ));
    }
    Ok(())
}
