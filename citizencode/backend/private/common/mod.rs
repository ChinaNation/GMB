//! 私权机构类型单一来源。
//!
//! 中文注释:身份 ID 格式不变,这里只定义私权机构在 `T2` 机构码上的目标分类。
//! `ZG/TG` 不再用于私权机构,它们只服务公民/自然人/智能人等人类主体来源分类。

use serde::{Deserialize, Serialize};

use crate::private::participants::ParticipantRole;
use crate::subjects::CreateInstitutionInput;

/// 私权机构业务类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum PrivateType {
    Sole,
    Partnership,
    Company,
    Corporation,
    Welfare,
    Association,
}

impl PrivateType {
    pub(crate) fn from_str(value: &str) -> Option<Self> {
        match value.trim() {
            "SOLE" => Some(Self::Sole),
            "PARTNERSHIP" => Some(Self::Partnership),
            "COMPANY" => Some(Self::Company),
            "CORPORATION" => Some(Self::Corporation),
            "WELFARE" => Some(Self::Welfare),
            "ASSOCIATION" => Some(Self::Association),
            _ => None,
        }
    }

    pub(crate) fn as_code(self) -> &'static str {
        match self {
            Self::Sole => "SOLE",
            Self::Partnership => "PARTNERSHIP",
            Self::Company => "COMPANY",
            Self::Corporation => "CORPORATION",
            Self::Welfare => "WELFARE",
            Self::Association => "ASSOCIATION",
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Sole => "个体经营",
            Self::Partnership => "合伙企业",
            Self::Company => "股权公司",
            Self::Corporation => "股份公司",
            Self::Welfare => "公益组织",
            Self::Association => "注册协会",
        }
    }
}

/// 合伙企业内部形态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum PartnershipKind {
    General,
    Limited,
}

impl PartnershipKind {
    pub(crate) fn from_str(value: &str) -> Option<Self> {
        match value.trim() {
            "GENERAL" => Some(Self::General),
            "LIMITED" => Some(Self::Limited),
            _ => None,
        }
    }

    pub(crate) fn as_code(self) -> &'static str {
        match self {
            Self::General => "GENERAL",
            Self::Limited => "LIMITED",
        }
    }
}

/// 创建私权机构时由类型规则锁定的身份字段。
#[derive(Debug, Clone, Copy)]
pub(crate) struct PrivateTypeRule {
    pub(crate) private_type: PrivateType,
    pub(crate) partnership_kind: Option<PartnershipKind>,
    pub(crate) subject_property: &'static str,
    pub(crate) institution_code: &'static str,
    pub(crate) p1: &'static str,
    pub(crate) has_legal_personality: bool,
}

/// 私权机构真实子模块的静态边界描述。
#[derive(Debug, Clone, Copy)]
pub(crate) struct PrivateModuleSpec {
    pub(crate) route_segment: &'static str,
    pub(crate) private_type: PrivateType,
    pub(crate) title: &'static str,
    pub(crate) description: &'static str,
    pub(crate) allowed_participant_roles: &'static [ParticipantRole],
}

/// 把请求强制锁定为某个私权类型。调用方只传业务类型,不信任前端的主体属性和机构码。
pub(crate) fn lock_input_to_rule(input: &mut CreateInstitutionInput, rule: PrivateTypeRule) {
    input.private_type = Some(rule.private_type.as_code().to_string());
    input.partnership_kind = rule.partnership_kind.map(|kind| kind.as_code().to_string());
    input.subject_property = rule.subject_property.to_string();
    input.institution = rule.institution_code.to_string();
    input.p1 = Some(rule.p1.to_string());
    // 中文注释:六类目标私权机构都是独立主体;非法人个体经营/无限合伙也不挂靠所属法人。
    input.parent_cid_number = None;
}

/// 通用非合伙私权类型的规则解析。合伙企业必须显式走 partnership 模块校验。
pub(crate) fn fixed_rule(private_type: PrivateType) -> Result<PrivateTypeRule, &'static str> {
    resolve_private_type_rule(private_type.as_code(), None)
}

/// 模块边界运行期自检。开发期用 debug_assert 暴露空配置,生产期无额外返回成本。
pub(crate) fn assert_module_spec(spec: &PrivateModuleSpec) {
    debug_assert!(!spec.route_segment.is_empty());
    debug_assert_eq!(spec.private_type.label(), spec.title);
    debug_assert!(!spec.description.is_empty());
    debug_assert!(!spec.allowed_participant_roles.is_empty());
    for participant_role in spec.allowed_participant_roles {
        debug_assert!(!participant_role.label().is_empty());
    }
}

/// 按私权类型解析身份字段。调用方不得让前端自带 subject_property / institution_code 覆盖本规则。
pub(crate) fn resolve_private_type_rule(
    private_type: &str,
    partnership_kind: Option<&str>,
) -> Result<PrivateTypeRule, &'static str> {
    let private_type =
        PrivateType::from_str(private_type).ok_or("private_type must be a valid private type")?;
    let rule = match private_type {
        PrivateType::Sole => PrivateTypeRule {
            private_type,
            partnership_kind: None,
            subject_property: "F",
            institution_code: "GT",
            p1: "1",
            has_legal_personality: false,
        },
        PrivateType::Partnership => match partnership_kind
            .and_then(PartnershipKind::from_str)
            .ok_or("partnership_kind must be GENERAL or LIMITED")?
        {
            PartnershipKind::General => PrivateTypeRule {
                private_type,
                partnership_kind: Some(PartnershipKind::General),
                subject_property: "F",
                institution_code: "GP",
                p1: "1",
                has_legal_personality: false,
            },
            PartnershipKind::Limited => PrivateTypeRule {
                private_type,
                partnership_kind: Some(PartnershipKind::Limited),
                subject_property: "S",
                institution_code: "LP",
                p1: "1",
                has_legal_personality: true,
            },
        },
        PrivateType::Company => PrivateTypeRule {
            private_type,
            partnership_kind: None,
            subject_property: "S",
            institution_code: "GQ",
            p1: "1",
            has_legal_personality: true,
        },
        PrivateType::Corporation => PrivateTypeRule {
            private_type,
            partnership_kind: None,
            subject_property: "S",
            institution_code: "GF",
            p1: "1",
            has_legal_personality: true,
        },
        PrivateType::Welfare => PrivateTypeRule {
            private_type,
            partnership_kind: None,
            subject_property: "S",
            institution_code: "GY",
            p1: "0",
            has_legal_personality: true,
        },
        PrivateType::Association => PrivateTypeRule {
            private_type,
            partnership_kind: None,
            subject_property: "S",
            institution_code: "AS",
            p1: "0",
            has_legal_personality: true,
        },
    };
    Ok(rule)
}
