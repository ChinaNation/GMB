//! 管理端操作权限分级。
//!
//! 中文注释:所有 SFID 管理端操作必须归入 LOGIN_STATE / PASSKEY /
//! PASSKEY_CHALLENGE 三类之一。安全动作入口只允许 PASSKEY 与
//! PASSKEY_CHALLENGE,登录态操作必须走各自业务 handler。

use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::admins::login::AdminAuthContext;
use crate::{api_error, AdminRole};

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
    CreateOperator,
    UpdateOperator,
    DeleteOperator,
    CreateShengAdmin,
    UpdateShengAdmin,
    DeleteShengAdmin,
    InstitutionCreate,
    InstitutionUpdate,
    InstitutionCreateAccount,
    InstitutionDeleteAccount,
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
            Self::CreateOperator => "CREATE_OPERATOR",
            Self::UpdateOperator => "UPDATE_OPERATOR",
            Self::DeleteOperator => "DELETE_OPERATOR",
            Self::CreateShengAdmin => "CREATE_FEDERAL_ADMIN",
            Self::UpdateShengAdmin => "UPDATE_FEDERAL_ADMIN",
            Self::DeleteShengAdmin => "DELETE_FEDERAL_ADMIN",
            Self::InstitutionCreate => "INSTITUTION_CREATE",
            Self::InstitutionUpdate => "INSTITUTION_UPDATE",
            Self::InstitutionCreateAccount => "INSTITUTION_CREATE_ACCOUNT",
            Self::InstitutionDeleteAccount => "INSTITUTION_DELETE_ACCOUNT",
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
            Self::CreateOperator => "新增市级管理员",
            Self::UpdateOperator => "编辑市级管理员",
            Self::DeleteOperator => "删除市级管理员",
            Self::CreateShengAdmin => "新增联邦管理员",
            Self::UpdateShengAdmin => "编辑联邦管理员",
            Self::DeleteShengAdmin => "删除联邦管理员",
            Self::InstitutionCreate => "创建机构",
            Self::InstitutionUpdate => "更新机构",
            Self::InstitutionCreateAccount => "新增机构账户",
            Self::InstitutionDeleteAccount => "删除机构账户",
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
            Self::UpdateOperator | Self::UpdateShengAdmin => AdminOperationAuth::LoginState,
            Self::InstitutionCreate
            | Self::InstitutionUpdate
            | Self::InstitutionCreateAccount
            | Self::InstitutionUploadDocument
            | Self::PublicSecurityReconcile
            | Self::CitizenBindCommit
            | Self::CpmsStatusImportConfirm => AdminOperationAuth::Passkey,
            Self::CreateOperator
            | Self::DeleteOperator
            | Self::CreateShengAdmin
            | Self::DeleteShengAdmin
            | Self::InstitutionDeleteAccount
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
            Self::CreateOperator
                | Self::DeleteOperator
                | Self::CreateShengAdmin
                | Self::DeleteShengAdmin
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
        "CREATE_OPERATOR" => Ok(AdminActionType::CreateOperator),
        "UPDATE_OPERATOR" => Ok(AdminActionType::UpdateOperator),
        "DELETE_OPERATOR" => Ok(AdminActionType::DeleteOperator),
        "CREATE_FEDERAL_ADMIN" => Ok(AdminActionType::CreateShengAdmin),
        "UPDATE_FEDERAL_ADMIN" => Ok(AdminActionType::UpdateShengAdmin),
        "DELETE_FEDERAL_ADMIN" => Ok(AdminActionType::DeleteShengAdmin),
        "INSTITUTION_CREATE" => Ok(AdminActionType::InstitutionCreate),
        "INSTITUTION_UPDATE" => Ok(AdminActionType::InstitutionUpdate),
        "INSTITUTION_CREATE_ACCOUNT" => Ok(AdminActionType::InstitutionCreateAccount),
        "INSTITUTION_DELETE_ACCOUNT" => Ok(AdminActionType::InstitutionDeleteAccount),
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
    if ctx.admin_province.is_none() {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "admin province scope missing",
        ));
    }
    if (action_type.is_governance() || action_type.is_cpms() || action_type.is_login_state())
        && ctx.role != AdminRole::ShengAdmin
    {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "sheng admin required",
        ));
    }
    Ok(())
}
