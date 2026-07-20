//! 管理端操作权限三档分级(读 / 本地写 / 链上写)。
//!
//! 三档之外一律拒绝;写操作一律 ≥ passkey,不存在只会话的写动作:
//! - Session         只读查询:仅需有效会话(会话已是链上已证管理员);由 `require_admin_any`
//!                   保障,不经 AdminActionType(AdminActionType 全是写动作)。
//! - Passkey         本地写:会话 + WebAuthn passkey 断言;只改 onchina 本地库、不产生 extrinsic。
//! - PasskeyColdSign 链上写:会话 + passkey + 冷钱包对真实链载荷签名。

use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::api_error;
use crate::auth::login::AdminAuthContext;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum AdminOperationAuth {
    /// 只读查询:仅需有效会话(会话已是链上已证管理员)。写动作不属此档。
    Session,
    /// 本地写:会话 + WebAuthn passkey 断言;只改 onchina 本地库、不产生 extrinsic。
    Passkey,
    /// 链上写:会话 + passkey + 冷钱包对真实链载荷签名(signer ∈ 本机构链上 Active 集合)。
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
    CreateCityRegistry,
    /// Tier1 删除一名 Tier2 下级注册局管理员。
    DeleteCityRegistry,
    InstitutionCreate,
    InstitutionUpdate,
    InstitutionCreateAccount,
    InstitutionDeleteAccount,
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
    /// 代表机构表决（管理员按当前机构席位投票）。
    CastRepresentativeVote,
    /// 特别案立法公投。
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
    /// 注册局推送公民身份上链(prepare 生成公民待签载荷 + complete 验签绑定,
    /// 公民上链操作：一次 Passkey 后进入短期业务操作会话，最终链签独立完成。
    CitizenOnchainPush,
}

impl AdminActionType {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::CreateCityRegistry => "CREATE_SUBORDINATE_REGISTRY",
            Self::DeleteCityRegistry => "DELETE_SUBORDINATE_REGISTRY",
            Self::InstitutionCreate => "INSTITUTION_CREATE",
            Self::InstitutionUpdate => "INSTITUTION_UPDATE",
            Self::InstitutionCreateAccount => "INSTITUTION_CREATE_ACCOUNT",
            Self::InstitutionDeleteAccount => "INSTITUTION_DELETE_ACCOUNT",
            Self::InstitutionUploadDocument => "INSTITUTION_UPLOAD_DOCUMENT",
            Self::InstitutionDeleteDocument => "INSTITUTION_DELETE_DOCUMENT",
            Self::NodeBindingUnbind => "NODE_BINDING_UNBIND",
            Self::ProposeEnactLaw => "PROPOSE_ENACT_LAW",
            Self::ProposeAmendLaw => "PROPOSE_AMEND_LAW",
            Self::ProposeRepealLaw => "PROPOSE_REPEAL_LAW",
            Self::CastRepresentativeVote => "CAST_REPRESENTATIVE_VOTE",
            Self::CastReferendumVote => "CAST_REFERENDUM_VOTE",
            Self::ExecutiveSign => "EXECUTIVE_SIGN",
            Self::OverrideSign => "OVERRIDE_SIGN",
            Self::GuardVote => "GUARD_VOTE",
            Self::ProposePersonnel => "PROPOSE_PERSONNEL",
            Self::ProposeBudget => "PROPOSE_BUDGET",
            Self::CitizenOnchainPush => "CITIZEN_ONCHAIN_PUSH",
        }
    }

    /// 动作 → 鉴权档(穷尽 match,新增动作漏标编译失败=默认拒绝)。
    ///
    /// 三档 = 读 / 本地写 / 链上写。AdminActionType 全是写动作,故只落 Passkey / PasskeyColdSign
    /// 两档;只读查询归 Session,由 `require_admin_any` 会话门保障,不经 AdminActionType。
    pub(crate) fn auth_type(&self) -> AdminOperationAuth {
        match self {
            // 本地写(Passkey):只改 onchina 本地库,不产生 extrinsic。
            Self::InstitutionUploadDocument
            | Self::InstitutionDeleteDocument
            | Self::NodeBindingUnbind
            // 最终管理员链签就是该角色唯一钱包签名,创建阶段只额外一次 passkey。
            | Self::CitizenOnchainPush => AdminOperationAuth::Passkey,
            // 链上写(PasskeyColdSign):产生链上交易/凭证、改 Active 集合或高危治理。
            // InstitutionUpdate 改 cid_full_name/法人/所属法人(链上注册凭证签名字段=链上单源),
            //   归链上写;前端本就走冷签。若存在纯本地展示字段,Phase 2/3 再拆出为本地写(Passkey)。
            Self::InstitutionUpdate
            | Self::InstitutionCreate
            | Self::InstitutionCreateAccount
            | Self::CreateCityRegistry
            | Self::DeleteCityRegistry
            | Self::InstitutionDeleteAccount
            // 立法与表决:全部产生链上交易。
            | Self::ProposeEnactLaw
            | Self::ProposeAmendLaw
            | Self::ProposeRepealLaw
            | Self::CastRepresentativeVote
            | Self::CastReferendumVote
            | Self::ExecutiveSign
            | Self::OverrideSign
            | Self::GuardVote
            | Self::ProposePersonnel
            | Self::ProposeBudget => AdminOperationAuth::PasskeyColdSign,
        }
    }

    pub(crate) fn is_governance(&self) -> bool {
        matches!(
            self,
            Self::CreateCityRegistry | Self::DeleteCityRegistry | Self::NodeBindingUnbind
        )
    }

    /// 是否要求 Tier1 创世注册局治理能力。注册局自身管理(增删下级注册局、更新/换届本档)
    /// 归此边界；机构元数据更新与文档上传不在其中——任一辖区管理员可对本辖区机构执行,
    /// 由 `scope` 限定可见域。机构自定义账户增删属机构自管(不经注册局审批),也不在此边界:
    /// 由机构在册管理员直接冷签 propose_close,链端以 `is_institution_admin` 鉴权。
    /// 与鉴权档正交:不依赖 auth_type,故动作在档间迁移不改变此权限边界。
    pub(crate) fn requires_governing_capability(&self) -> bool {
        matches!(
            self,
            Self::CreateCityRegistry | Self::DeleteCityRegistry
        )
    }
}

pub(crate) fn parse_action_type(
    action_type: &str,
) -> Result<AdminActionType, axum::response::Response> {
    match action_type {
        "CREATE_SUBORDINATE_REGISTRY" => Ok(AdminActionType::CreateCityRegistry),
        "DELETE_SUBORDINATE_REGISTRY" => Ok(AdminActionType::DeleteCityRegistry),
        "INSTITUTION_CREATE" => Ok(AdminActionType::InstitutionCreate),
        "INSTITUTION_UPDATE" => Ok(AdminActionType::InstitutionUpdate),
        "INSTITUTION_CREATE_ACCOUNT" => Ok(AdminActionType::InstitutionCreateAccount),
        "INSTITUTION_DELETE_ACCOUNT" => Ok(AdminActionType::InstitutionDeleteAccount),
        "INSTITUTION_UPLOAD_DOCUMENT" => Ok(AdminActionType::InstitutionUploadDocument),
        "INSTITUTION_DELETE_DOCUMENT" => Ok(AdminActionType::InstitutionDeleteDocument),
        "NODE_BINDING_UNBIND" => Ok(AdminActionType::NodeBindingUnbind),
        "PROPOSE_ENACT_LAW" => Ok(AdminActionType::ProposeEnactLaw),
        "PROPOSE_AMEND_LAW" => Ok(AdminActionType::ProposeAmendLaw),
        "PROPOSE_REPEAL_LAW" => Ok(AdminActionType::ProposeRepealLaw),
        "CAST_REPRESENTATIVE_VOTE" => Ok(AdminActionType::CastRepresentativeVote),
        "CAST_REFERENDUM_VOTE" => Ok(AdminActionType::CastReferendumVote),
        "EXECUTIVE_SIGN" => Ok(AdminActionType::ExecutiveSign),
        "OVERRIDE_SIGN" => Ok(AdminActionType::OverrideSign),
        "GUARD_VOTE" => Ok(AdminActionType::GuardVote),
        "PROPOSE_PERSONNEL" => Ok(AdminActionType::ProposePersonnel),
        "PROPOSE_BUDGET" => Ok(AdminActionType::ProposeBudget),
        "CITIZEN_ONCHAIN_PUSH" => Ok(AdminActionType::CitizenOnchainPush),
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
        // 注册局新增/删除下级仍要求 Tier1 创世注册局治理能力。
        assert!(AdminActionType::CreateCityRegistry.requires_governing_capability());
        assert!(AdminActionType::DeleteCityRegistry.requires_governing_capability());
        // 机构自定义账户增删属机构自管(不经注册局审批),不要求治理能力。
        assert!(!AdminActionType::InstitutionCreate.requires_governing_capability());
        assert!(!AdminActionType::InstitutionCreateAccount.requires_governing_capability());
        assert!(!AdminActionType::InstitutionDeleteAccount.requires_governing_capability());
        assert!(!AdminActionType::InstitutionDeleteDocument.requires_governing_capability());
        assert!(!AdminActionType::NodeBindingUnbind.requires_governing_capability());
    }

    #[test]
    fn citizen_onchain_push_uses_one_passkey_and_round_trips() {
        // 最终链签已是管理员唯一钱包签名，操作创建阶段只额外消费一次 Passkey。
        let action = AdminActionType::CitizenOnchainPush;
        assert_eq!(action.auth_type(), AdminOperationAuth::Passkey);
        assert!(!action.requires_governing_capability());
        assert!(!action.is_governance());
        let parsed = parse_action_type(action.as_str()).expect("citizen action parses");
        assert_eq!(parsed, action);
    }

    #[test]
    fn legislation_actions_are_cold_sign_and_round_trip() {
        // 立法与表决动作全部产链上交易,归 PasskeyColdSign;且不属注册局治理能力边界。
        let actions = [
            AdminActionType::ProposeEnactLaw,
            AdminActionType::ProposeAmendLaw,
            AdminActionType::ProposeRepealLaw,
            AdminActionType::CastRepresentativeVote,
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
