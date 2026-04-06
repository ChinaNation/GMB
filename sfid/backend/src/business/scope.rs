use crate::sfid::province::super_admin_province;
use crate::{AdminRole, BindingRecord, CpmsSiteKeys, MultisigSfidRecord, PendingRequest, Store};

pub(crate) fn province_scope_for_role(
    store: &Store,
    admin_pubkey: &str,
    role: &AdminRole,
) -> Option<String> {
    match role {
        AdminRole::KeyAdmin => None,
        AdminRole::InstitutionAdmin => store
            .super_admin_province_by_pubkey
            .get(admin_pubkey)
            .cloned()
            .or_else(|| super_admin_province(admin_pubkey).map(|v| v.to_string())),
        AdminRole::SystemAdmin => {
            let creator_pubkey = store
                .admin_users_by_pubkey
                .get(admin_pubkey)
                .map(|u| u.created_by.clone())?;
            store
                .super_admin_province_by_pubkey
                .get(&creator_pubkey)
                .cloned()
                .or_else(|| super_admin_province(&creator_pubkey).map(|v| v.to_string()))
        }
    }
}

pub(crate) fn in_scope(binding: &BindingRecord, admin_province: Option<&str>) -> bool {
    match admin_province {
        Some(scope) => binding
            .admin_province
            .as_deref()
            .map(|v| v == scope)
            .unwrap_or(false),
        None => true,
    }
}

pub(crate) fn in_scope_pending(pending: &PendingRequest, admin_province: Option<&str>) -> bool {
    match admin_province {
        Some(scope) => pending
            .admin_province
            .as_deref()
            .map(|v| v == scope)
            .unwrap_or(false),
        None => true,
    }
}

pub(crate) fn in_scope_cpms_site(site: &CpmsSiteKeys, admin_province: Option<&str>) -> bool {
    match admin_province {
        Some(scope) => site.admin_province == scope,
        None => true,
    }
}

pub(crate) fn in_scope_multisig(record: &MultisigSfidRecord, admin_province: Option<&str>) -> bool {
    match admin_province {
        Some(scope) => record.province == scope,
        None => true,
    }
}
