//! 管理端操作权限三档分级。
//!
//! 所有 CID 管理端操作归入 Session / Passkey / PasskeyColdSign 三档之一,三档之外一律拒绝:
//! - Session         一般操作:仅需有效会话(会话已是链上已证管理员)。
//! - Passkey         重要操作:会话 + WebAuthn passkey 断言。
//! - PasskeyColdSign 特殊操作:会话 + WebAuthn passkey 断言 + 冷钱包扫码签名。

use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::api_error;
use crate::auth::login::AdminAuthContext;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum AdminOperationAuth {
    /// 一般操作:仅需有效会话(会话已是链上已证管理员)。
    Session,
    /// 重要操作:会话 + WebAuthn passkey 断言。
    Passkey,
    /// 特殊操作:会话 + WebAuthn passkey 断言 + 冷钱包扫码签名(signer ∈ 本机构链上 Active 集合)。
    PasskeyColdSign,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum AdminActionType {
    CreateCityRegistry,
    UpdateCityRegistry,
    DeleteCityRegistry,
    UpdateFederalRegistry,
    ReplaceFederalRegistry,
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
            Self::UpdateFederalRegistry => "UPDATE_FEDERAL_REGISTRY",
            Self::ReplaceFederalRegistry => "REPLACE_FEDERAL_REGISTRY",
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

    /// 动作 → 鉴权档(穷尽 match,新增动作漏标编译失败=默认拒绝)。
    pub(crate) fn auth_type(&self) -> AdminOperationAuth {
        match self {
            // 纯本地确认 / 元数据更新 → 仅会话。
            Self::InstitutionUpdate | Self::InstitutionUploadDocument => {
                AdminOperationAuth::Session
            }
            // 改注册局管理元数据,重要但不产链上凭证 → passkey 重要档。
            Self::UpdateCityRegistry | Self::UpdateFederalRegistry => AdminOperationAuth::Passkey,
            // 产生链上交易/凭证、改 Active 集合或高危治理 → passkey + 冷签特殊档。
            Self::InstitutionCreate
            | Self::InstitutionCreateAccount
            | Self::CreateCityRegistry
            | Self::DeleteCityRegistry
            | Self::ReplaceFederalRegistry
            | Self::InstitutionDeleteAccount
            | Self::InstitutionDeregister
            | Self::InstitutionAccountDeregister
            | Self::InstitutionDeleteDocument => AdminOperationAuth::PasskeyColdSign,
        }
    }

    pub(crate) fn is_session(&self) -> bool {
        self.auth_type() == AdminOperationAuth::Session
    }

    pub(crate) fn is_governance(&self) -> bool {
        matches!(
            self,
            Self::CreateCityRegistry
                | Self::DeleteCityRegistry
                | Self::ReplaceFederalRegistry
                | Self::InstitutionDeregister
                | Self::InstitutionAccountDeregister
        )
    }

    /// 是否要求联邦注册局管理员。仅注册局自身管理(增删市注册局、更新/换届联邦注册局)
    /// 与机构注销治理(整机构/单账户)归此边界;机构元数据更新与文档上传不在其中——
    /// 任一辖区管理员可对本辖区机构执行,由 `scope` 限定可见域。与鉴权档正交:不依赖
    /// auth_type,故动作在档间迁移不改变此权限边界。
    pub(crate) fn requires_federal_admin(&self) -> bool {
        matches!(
            self,
            Self::CreateCityRegistry
                | Self::DeleteCityRegistry
                | Self::ReplaceFederalRegistry
                | Self::UpdateCityRegistry
                | Self::UpdateFederalRegistry
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
        "UPDATE_FEDERAL_REGISTRY" => Ok(AdminActionType::UpdateFederalRegistry),
        "REPLACE_FEDERAL_REGISTRY" => Ok(AdminActionType::ReplaceFederalRegistry),
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
    if action_type.requires_federal_admin() && ctx.institution_code != "FRG" {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "federal admin required",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operation_auth_has_exactly_three_tiers() {
        // 三档铁律:Session / Passkey / PasskeyColdSign。新增第四档必须显式改本测试与所有
        // 穷尽 match;三档之外的操作一律拒绝。
        let all = [
            AdminOperationAuth::Session,
            AdminOperationAuth::Passkey,
            AdminOperationAuth::PasskeyColdSign,
        ];
        assert_eq!(all.len(), 3);
        for tier in all {
            // 穷尽 match,无 `_ =>` 兜底;新增变体漏标 → 编译失败。
            let _label = match tier {
                AdminOperationAuth::Session => "SESSION",
                AdminOperationAuth::Passkey => "PASSKEY",
                AdminOperationAuth::PasskeyColdSign => "PASSKEY_COLD_SIGN",
            };
        }
    }

    #[test]
    fn federal_admin_boundary_excludes_institution_update_and_upload() {
        // 机构元数据更新与文档上传由发起管理员的 scope 限定本辖区,不要求联邦注册局管理员。
        assert!(!AdminActionType::InstitutionUpdate.requires_federal_admin());
        assert!(!AdminActionType::InstitutionUploadDocument.requires_federal_admin());
        // 注册局自身管理与机构注销治理仍要求联邦注册局管理员。
        assert!(AdminActionType::CreateCityRegistry.requires_federal_admin());
        assert!(AdminActionType::DeleteCityRegistry.requires_federal_admin());
        assert!(AdminActionType::ReplaceFederalRegistry.requires_federal_admin());
        assert!(AdminActionType::UpdateCityRegistry.requires_federal_admin());
        assert!(AdminActionType::UpdateFederalRegistry.requires_federal_admin());
        assert!(AdminActionType::InstitutionDeregister.requires_federal_admin());
        assert!(AdminActionType::InstitutionAccountDeregister.requires_federal_admin());
        // 普通机构特殊操作(建机构/建账户/删账户/删文档)由 scope 收口,不要求联邦。
        assert!(!AdminActionType::InstitutionCreate.requires_federal_admin());
        assert!(!AdminActionType::InstitutionCreateAccount.requires_federal_admin());
        assert!(!AdminActionType::InstitutionDeleteAccount.requires_federal_admin());
        assert!(!AdminActionType::InstitutionDeleteDocument.requires_federal_admin());
    }
}
