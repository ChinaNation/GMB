//! 注册协会私权机构。
//!
//! 中文注释:注册协会固定为 `S + AS`,具有法人资格且非营利,由会员、理事和监事构成。

use axum::{
    Json,
    extract::{Query, State},
    http::HeaderMap,
    response::Response,
};
use serde::Serialize;

use crate::AppState;
use crate::domains::private::common::{
    PrivateModuleSpec, PrivateType, assert_module_spec, fixed_rule, lock_input_to_rule,
};
use crate::domains::private::participants::{ASSOCIATION_ROLES, ParticipantRole};
use crate::institution::subjects::CreateInstitutionInput;
use crate::institution::subjects::registration::{self, ListInstitutionQuery};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AssociationProfile {
    pub(crate) identity_code: &'static str,
    pub(crate) p1: &'static str,
    pub(crate) has_legal_personality: bool,
    pub(crate) member_role: ParticipantRole,
}

pub(crate) const PROFILE: AssociationProfile = AssociationProfile {
    identity_code: "SFAS",
    p1: "0",
    has_legal_personality: true,
    member_role: ParticipantRole::Member,
};

pub(crate) const SPEC: PrivateModuleSpec = PrivateModuleSpec {
    route_segment: "association",
    private_type: PrivateType::Association,
    title: "注册协会",
    description: "具有法人资格的协会类组织,管理会员、理事和协会宗旨。",
    allowed_participant_roles: ASSOCIATION_ROLES,
};

fn lock_input(input: &mut CreateInstitutionInput) -> Result<(), &'static str> {
    assert_module_spec(&SPEC);
    debug_assert_eq!(PROFILE.identity_code, "SFAS");
    debug_assert_eq!(PROFILE.p1, "0");
    debug_assert!(PROFILE.has_legal_personality);
    debug_assert_eq!(PROFILE.member_role, ParticipantRole::Member);
    let rule = fixed_rule(SPEC.private_type)?;
    lock_input_to_rule(input, rule);
    Ok(())
}

pub(crate) async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(mut input): Json<CreateInstitutionInput>,
) -> Response {
    if let Err(msg) = lock_input(&mut input) {
        return crate::api_error(axum::http::StatusCode::BAD_REQUEST, 1001, msg);
    }
    registration::create_private_institution(state, headers, input).await
}

pub(crate) async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(mut query): Query<ListInstitutionQuery>,
) -> Response {
    query.category = Some("PRIVATE_INSTITUTION".to_string());
    query.private_type = Some(SPEC.private_type.as_code().to_string());
    registration::list_private_institutions(state, headers, query).await
}
