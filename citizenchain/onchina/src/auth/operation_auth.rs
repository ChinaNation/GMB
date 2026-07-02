//! 管理端操作权限三档分级。
//!
//! 所有链上中国平台操作归入 Session / Passkey / PasskeyColdSign 三档之一,三档之外一律拒绝:
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

/// 管理端动作类型(Tier 中性命名,决策②)。
///
/// 注册局动作按分层命名——Governing = Tier1 创世注册局自身(本期 = 联邦注册局),
/// Subordinate = 其供给的 Tier2 下级注册局(本期 = 市注册局)。命名与具体机构码解耦,
/// 鉴权边界经 `is_tier1_registry` 谓词裁决,不再字面绑定 FRG/CREG。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum AdminActionType {
    /// Tier1 创世注册局新增一名 Tier2 下级注册局管理员。
    CreateSubordinateRegistry,
    /// Tier1 删除一名 Tier2 下级注册局管理员。
    DeleteSubordinateRegistry,
    /// Tier1 换届本档(创世注册局)自身一名管理员(经省组投票)。
    ReplaceGoverningRegistry,
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
    /// 本节点解除当前机构绑定;解绑后必须重新扫码登录并绑定机构。
    NodeBindingUnbind,
    // ───────── 立法与表决(卡 20260630-onchina-legislation-console-framework)─────────
    // 全部产生链上交易(提交 extrinsic / 改提案状态),归 PasskeyColdSign 特殊档。
    /// 发起立法(新法)法律案。
    ProposeEnactLaw,
    /// 发起修法法律案。
    ProposeAmendLaw,
    /// 发起废法法律案。
    ProposeRepealLaw,
    /// 院内表决(议员/委员对当前院投票)。
    CastHouseVote,
    /// 特别案公民投票。
    CastReferendumVote,
    /// 行政签署 / 否决(总统/省长/市长;另线程接入)。
    ExecutiveSign,
    /// 三人会签救济(院长 + 参议长 + 众议长;另线程接入)。
    OverrideSign,
    /// 护宪大法官终审(修宪;另线程接入)。
    GuardVote,
    /// 发起任免案(政府;Phase 4 接入)。
    ProposePersonnel,
    /// 发起预算案(政府;Phase 4 接入)。
    ProposeBudget,
}

impl AdminActionType {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::CreateSubordinateRegistry => "CREATE_SUBORDINATE_REGISTRY",
            Self::DeleteSubordinateRegistry => "DELETE_SUBORDINATE_REGISTRY",
            Self::ReplaceGoverningRegistry => "REPLACE_GOVERNING_REGISTRY",
            Self::InstitutionCreate => "INSTITUTION_CREATE",
            Self::InstitutionUpdate => "INSTITUTION_UPDATE",
            Self::InstitutionCreateAccount => "INSTITUTION_CREATE_ACCOUNT",
            Self::InstitutionDeleteAccount => "INSTITUTION_DELETE_ACCOUNT",
            Self::InstitutionDeregister => "INSTITUTION_DEREGISTER",
            Self::InstitutionAccountDeregister => "INSTITUTION_ACCOUNT_DEREGISTER",
            Self::InstitutionUploadDocument => "INSTITUTION_UPLOAD_DOCUMENT",
            Self::InstitutionDeleteDocument => "INSTITUTION_DELETE_DOCUMENT",
            Self::NodeBindingUnbind => "NODE_BINDING_UNBIND",
            Self::ProposeEnactLaw => "PROPOSE_ENACT_LAW",
            Self::ProposeAmendLaw => "PROPOSE_AMEND_LAW",
            Self::ProposeRepealLaw => "PROPOSE_REPEAL_LAW",
            Self::CastHouseVote => "CAST_HOUSE_VOTE",
            Self::CastReferendumVote => "CAST_REFERENDUM_VOTE",
            Self::ExecutiveSign => "EXECUTIVE_SIGN",
            Self::OverrideSign => "OVERRIDE_SIGN",
            Self::GuardVote => "GUARD_VOTE",
            Self::ProposePersonnel => "PROPOSE_PERSONNEL",
            Self::ProposeBudget => "PROPOSE_BUDGET",
        }
    }

    /// 动作 → 鉴权档(穷尽 match,新增动作漏标编译失败=默认拒绝)。
    pub(crate) fn auth_type(&self) -> AdminOperationAuth {
        match self {
            // 纯本地确认 / 元数据更新 → 仅会话。
            Self::InstitutionUpdate | Self::InstitutionUploadDocument => {
                AdminOperationAuth::Session
            }
            // 产生链上交易/凭证、改 Active 集合或高危治理 → passkey + 冷签特殊档。
            Self::InstitutionCreate
            | Self::InstitutionCreateAccount
            | Self::CreateSubordinateRegistry
            | Self::DeleteSubordinateRegistry
            | Self::ReplaceGoverningRegistry
            | Self::InstitutionDeleteAccount
            | Self::InstitutionDeregister
            | Self::InstitutionAccountDeregister
            | Self::InstitutionDeleteDocument
            | Self::NodeBindingUnbind
            // 立法与表决:全部产生链上交易,归 PasskeyColdSign 特殊档(冷钱包扫码签名)。
            | Self::ProposeEnactLaw
            | Self::ProposeAmendLaw
            | Self::ProposeRepealLaw
            | Self::CastHouseVote
            | Self::CastReferendumVote
            | Self::ExecutiveSign
            | Self::OverrideSign
            | Self::GuardVote
            | Self::ProposePersonnel
            | Self::ProposeBudget => AdminOperationAuth::PasskeyColdSign,
        }
    }

    pub(crate) fn is_session(&self) -> bool {
        self.auth_type() == AdminOperationAuth::Session
    }

    pub(crate) fn is_governance(&self) -> bool {
        matches!(
            self,
            Self::CreateSubordinateRegistry
                | Self::DeleteSubordinateRegistry
                | Self::ReplaceGoverningRegistry
                | Self::InstitutionDeregister
                | Self::InstitutionAccountDeregister
                | Self::NodeBindingUnbind
        )
    }

    /// 是否要求 Tier1 创世注册局治理能力。注册局自身管理(增删下级注册局、更新/换届本档)
    /// 与机构注销治理(整机构/单账户)归此边界;机构元数据更新与文档上传不在其中——
    /// 任一辖区管理员可对本辖区机构执行,由 `scope` 限定可见域。与鉴权档正交:不依赖
    /// auth_type,故动作在档间迁移不改变此权限边界。
    pub(crate) fn requires_governing_capability(&self) -> bool {
        matches!(
            self,
            Self::CreateSubordinateRegistry
                | Self::DeleteSubordinateRegistry
                | Self::ReplaceGoverningRegistry
                | Self::InstitutionDeregister
                | Self::InstitutionAccountDeregister
        )
    }
}

pub(crate) fn parse_action_type(
    action_type: &str,
) -> Result<AdminActionType, axum::response::Response> {
    match action_type {
        "CREATE_SUBORDINATE_REGISTRY" => Ok(AdminActionType::CreateSubordinateRegistry),
        "DELETE_SUBORDINATE_REGISTRY" => Ok(AdminActionType::DeleteSubordinateRegistry),
        "REPLACE_GOVERNING_REGISTRY" => Ok(AdminActionType::ReplaceGoverningRegistry),
        "INSTITUTION_CREATE" => Ok(AdminActionType::InstitutionCreate),
        "INSTITUTION_UPDATE" => Ok(AdminActionType::InstitutionUpdate),
        "INSTITUTION_CREATE_ACCOUNT" => Ok(AdminActionType::InstitutionCreateAccount),
        "INSTITUTION_DELETE_ACCOUNT" => Ok(AdminActionType::InstitutionDeleteAccount),
        "INSTITUTION_DEREGISTER" => Ok(AdminActionType::InstitutionDeregister),
        "INSTITUTION_ACCOUNT_DEREGISTER" => Ok(AdminActionType::InstitutionAccountDeregister),
        "INSTITUTION_UPLOAD_DOCUMENT" => Ok(AdminActionType::InstitutionUploadDocument),
        "INSTITUTION_DELETE_DOCUMENT" => Ok(AdminActionType::InstitutionDeleteDocument),
        "NODE_BINDING_UNBIND" => Ok(AdminActionType::NodeBindingUnbind),
        "PROPOSE_ENACT_LAW" => Ok(AdminActionType::ProposeEnactLaw),
        "PROPOSE_AMEND_LAW" => Ok(AdminActionType::ProposeAmendLaw),
        "PROPOSE_REPEAL_LAW" => Ok(AdminActionType::ProposeRepealLaw),
        "CAST_HOUSE_VOTE" => Ok(AdminActionType::CastHouseVote),
        "CAST_REFERENDUM_VOTE" => Ok(AdminActionType::CastReferendumVote),
        "EXECUTIVE_SIGN" => Ok(AdminActionType::ExecutiveSign),
        "OVERRIDE_SIGN" => Ok(AdminActionType::OverrideSign),
        "GUARD_VOTE" => Ok(AdminActionType::GuardVote),
        "PROPOSE_PERSONNEL" => Ok(AdminActionType::ProposePersonnel),
        "PROPOSE_BUDGET" => Ok(AdminActionType::ProposeBudget),
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
    if action_type.requires_governing_capability()
        && !crate::core::chain_runtime::is_tier1_registry(&ctx.institution_code)
    {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            1003,
            "governing registry admin required",
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
    fn governing_capability_boundary_excludes_institution_update_and_upload() {
        // 机构元数据更新与文档上传由发起管理员的 scope 限定本辖区,不要求 Tier1 创世注册局治理能力。
        assert!(!AdminActionType::InstitutionUpdate.requires_governing_capability());
        assert!(!AdminActionType::InstitutionUploadDocument.requires_governing_capability());
        // 注册局新增/删除下级、换届本档与机构注销治理仍要求 Tier1 创世注册局治理能力。
        assert!(AdminActionType::CreateSubordinateRegistry.requires_governing_capability());
        assert!(AdminActionType::DeleteSubordinateRegistry.requires_governing_capability());
        assert!(AdminActionType::ReplaceGoverningRegistry.requires_governing_capability());
        assert!(AdminActionType::InstitutionDeregister.requires_governing_capability());
        assert!(AdminActionType::InstitutionAccountDeregister.requires_governing_capability());
        // 普通机构特殊操作(建机构/建账户/删账户/删文档)由 scope 收口,不要求治理能力。
        assert!(!AdminActionType::InstitutionCreate.requires_governing_capability());
        assert!(!AdminActionType::InstitutionCreateAccount.requires_governing_capability());
        assert!(!AdminActionType::InstitutionDeleteAccount.requires_governing_capability());
        assert!(!AdminActionType::InstitutionDeleteDocument.requires_governing_capability());
        assert!(!AdminActionType::NodeBindingUnbind.requires_governing_capability());
    }

    #[test]
    fn legislation_actions_are_cold_sign_and_round_trip() {
        // 立法与表决动作全部产链上交易,归 PasskeyColdSign;且不属注册局治理能力边界。
        let actions = [
            AdminActionType::ProposeEnactLaw,
            AdminActionType::ProposeAmendLaw,
            AdminActionType::ProposeRepealLaw,
            AdminActionType::CastHouseVote,
            AdminActionType::CastReferendumVote,
            AdminActionType::ExecutiveSign,
            AdminActionType::OverrideSign,
            AdminActionType::GuardVote,
            AdminActionType::ProposePersonnel,
            AdminActionType::ProposeBudget,
        ];
        for action in actions {
            assert_eq!(action.auth_type(), AdminOperationAuth::PasskeyColdSign);
            assert!(!action.requires_governing_capability());
            // as_str ↔ parse_action_type 逐字往返一致。
            let parsed = parse_action_type(action.as_str()).expect("legislation action parses");
            assert_eq!(parsed, action);
        }
    }
}
