//! 管理端操作权限分级。
//!
//! 中文注释:所有 CID 管理端操作必须归入 LOGIN_STATE / PASSKEY /
//! PASSKEY_CHALLENGE 三类之一。安全动作入口只允许 PASSKEY 与
//! PASSKEY_CHALLENGE,登录态操作必须走各自业务 handler。

use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::admins::login::AdminAuthContext;
use crate::{api_error, RegistryOrgCode};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum AdminOperationAuth {
    LoginState,
    Passkey,
    PasskeyChallenge,
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
    PublicSecurityReconcile,
    CitizenBindCommit,
    CpmsStatusImportConfirm,
    CpmsIssueInstallCode,
    CpmsRevokeInstallToken,
    CpmsReissueInstallToken,
    CpmsDisableKeys,
    CpmsEnableKeys,
    CpmsRevokeKeys,
    CpmsDeleteKeys,
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
            Self::PublicSecurityReconcile => "PUBLIC_SECURITY_RECONCILE",
            Self::CitizenBindCommit => "CITIZEN_BIND_COMMIT",
            Self::CpmsStatusImportConfirm => "CPMS_STATUS_IMPORT_CONFIRM",
            Self::CpmsIssueInstallCode => "CPMS_ISSUE_INSTALL_CODE",
            Self::CpmsRevokeInstallToken => "CPMS_REVOKE_INSTALL_TOKEN",
            Self::CpmsReissueInstallToken => "CPMS_REISSUE_INSTALL_TOKEN",
            Self::CpmsDisableKeys => "CPMS_DISABLE_KEYS",
            Self::CpmsEnableKeys => "CPMS_ENABLE_KEYS",
            Self::CpmsRevokeKeys => "CPMS_REVOKE_KEYS",
            Self::CpmsDeleteKeys => "CPMS_DELETE_KEYS",
        }
    }

    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::CreateCityRegistry => "新增市注册局管理员",
            Self::UpdateCityRegistry => "编辑市注册局管理员",
            Self::DeleteCityRegistry => "删除市注册局管理员",
            Self::CreateFederalRegistry => "新增联邦注册局管理员",
            Self::UpdateFederalRegistry => "编辑联邦注册局管理员",
            Self::DeleteFederalRegistry => "删除联邦注册局管理员",
            Self::InstitutionCreate => "创建机构",
            Self::InstitutionUpdate => "更新机构",
            Self::InstitutionCreateAccount => "新增机构账户",
            Self::InstitutionDeleteAccount => "删除机构账户",
            Self::InstitutionDeregister => "注销机构",
            Self::InstitutionAccountDeregister => "注销机构账户",
            Self::InstitutionUploadDocument => "上传机构文档",
            Self::InstitutionDeleteDocument => "删除机构文档",
            Self::PublicSecurityReconcile => "公安局机构对账",
            Self::CitizenBindCommit => "确认电子护照绑定",
            Self::CpmsStatusImportConfirm => "导入 CPMS 年度报告",
            Self::CpmsIssueInstallCode => "签发 CPMS 安装码",
            Self::CpmsRevokeInstallToken => "作废 CPMS 安装令牌",
            Self::CpmsReissueInstallToken => "重新签发 CPMS 安装码",
            Self::CpmsDisableKeys => "禁用 CPMS 授权",
            Self::CpmsEnableKeys => "启用 CPMS 授权",
            Self::CpmsRevokeKeys => "吊销 CPMS 授权",
            Self::CpmsDeleteKeys => "删除 CPMS 授权",
        }
    }

    pub(crate) fn auth_type(&self) -> AdminOperationAuth {
        match self {
            Self::UpdateCityRegistry | Self::UpdateFederalRegistry => {
                AdminOperationAuth::LoginState
            }
            Self::InstitutionCreate
            | Self::InstitutionUpdate
            | Self::InstitutionCreateAccount
            | Self::InstitutionUploadDocument
            | Self::PublicSecurityReconcile
            | Self::CitizenBindCommit
            | Self::CpmsStatusImportConfirm => AdminOperationAuth::Passkey,
            Self::CreateCityRegistry
            | Self::DeleteCityRegistry
            | Self::CreateFederalRegistry
            | Self::DeleteFederalRegistry
            | Self::InstitutionDeleteAccount
            | Self::InstitutionDeregister
            | Self::InstitutionAccountDeregister
            | Self::InstitutionDeleteDocument
            | Self::CpmsIssueInstallCode
            | Self::CpmsRevokeInstallToken
            | Self::CpmsReissueInstallToken
            | Self::CpmsDisableKeys
            | Self::CpmsEnableKeys
            | Self::CpmsRevokeKeys
            | Self::CpmsDeleteKeys => AdminOperationAuth::PasskeyChallenge,
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

    pub(crate) fn is_cpms(&self) -> bool {
        matches!(
            self,
            Self::CpmsIssueInstallCode
                | Self::CpmsRevokeInstallToken
                | Self::CpmsReissueInstallToken
                | Self::CpmsDisableKeys
                | Self::CpmsEnableKeys
                | Self::CpmsRevokeKeys
                | Self::CpmsDeleteKeys
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
        "PUBLIC_SECURITY_RECONCILE" => Ok(AdminActionType::PublicSecurityReconcile),
        "CITIZEN_BIND_COMMIT" => Ok(AdminActionType::CitizenBindCommit),
        "CPMS_STATUS_IMPORT_CONFIRM" => Ok(AdminActionType::CpmsStatusImportConfirm),
        "CPMS_ISSUE_INSTALL_CODE" => Ok(AdminActionType::CpmsIssueInstallCode),
        "CPMS_REVOKE_INSTALL_TOKEN" => Ok(AdminActionType::CpmsRevokeInstallToken),
        "CPMS_REISSUE_INSTALL_TOKEN" => Ok(AdminActionType::CpmsReissueInstallToken),
        "CPMS_DISABLE_KEYS" => Ok(AdminActionType::CpmsDisableKeys),
        "CPMS_ENABLE_KEYS" => Ok(AdminActionType::CpmsEnableKeys),
        "CPMS_REVOKE_KEYS" => Ok(AdminActionType::CpmsRevokeKeys),
        "CPMS_DELETE_KEYS" => Ok(AdminActionType::CpmsDeleteKeys),
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
    if (action_type.is_governance() || action_type.is_cpms() || action_type.is_login_state())
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
