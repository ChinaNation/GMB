//! CID、机构身份与机构账户完整性节点永久策略。
//!
//! 机构唯一主键是 CID；`Institutions[cid_number]` 保存机构身份，
//! `InstitutionAccounts[(cid_number, account_name)]` 保存账户正向真源，
//! `AccountRegisteredCid[account]` 只作反向索引。主账户、费用账户和制度专属账户
//! 都只是协议账户，不承担机构身份、管理员根或生命周期状态。

use std::collections::{BTreeMap, BTreeSet};

use codec::{Decode, Encode};
use sp_core::hashing::blake2_128;

use primitives::account_derive::{
    institution_kind_by_name, institution_protocol_account_name,
    institution_protocol_kind_by_name, InstitutionProtocolAccountKind,
};
use primitives::cid::code::{
    is_fixed_governance_code, is_private_legal_code, is_public_legal_code,
    is_unincorporated_code, InstitutionCode,
};
use primitives::cid::number::parse_cid_number_parts_bytes;
use primitives::core_const::SS58_FORMAT;

const CITIZEN_IDENTITY_PALLET: &[u8] = b"CitizenIdentity";
const PUBLIC_MANAGE_PALLET: &[u8] = b"PublicManage";
const PRIVATE_MANAGE_PALLET: &[u8] = b"PrivateManage";
const CODE_KEY: &[u8] = b":code";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Namespace {
    Public,
    Private,
}

impl Namespace {
    fn pallet(self) -> &'static [u8] {
        match self {
            Self::Public => PUBLIC_MANAGE_PALLET,
            Self::Private => PRIVATE_MANAGE_PALLET,
        }
    }

    fn sibling(self) -> Self {
        match self {
            Self::Public => Self::Private,
            Self::Private => Self::Public,
        }
    }
}

#[derive(Clone, Copy, Debug, Decode, Encode, Eq, PartialEq)]
enum CitizenCidStatus {
    Active,
    Revoked,
}

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
struct CitizenCidRecord {
    registrar_cid_number: Vec<u8>,
    commitment: [u8; 32],
    residence_province_code: Vec<u8>,
    residence_city_code: Vec<u8>,
    status: CitizenCidStatus,
    registered_at: u32,
    revoked_at: Option<u32>,
}

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
struct InstitutionRecord {
    cid_full_name: Vec<u8>,
    cid_short_name: Vec<u8>,
    town_code: Vec<u8>,
    legal_representative_name: Option<Vec<u8>>,
    legal_representative_cid_number: Option<Vec<u8>>,
    legal_representative_account: Option<[u8; 32]>,
    institution_code: InstitutionCode,
    created_at: u32,
}

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
struct RegisteredInstitution {
    cid_number: Vec<u8>,
    account_name: Vec<u8>,
}

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
struct InstitutionAccountRecord {
    address: [u8; 32],
    initial_balance: u128,
    created_at: u32,
}

/// 新模型不再冻结一组“创世保护账户”；永久性由账户类别和 CID 制度约束直接决定。
#[derive(Clone, Debug, Default)]
pub struct GenesisReference;

#[derive(Debug, Eq, PartialEq)]
pub enum GuardError {
    StorageKeyMalformed(&'static str),
    StorageValueDecodeFailed(&'static str),
    InvalidCid,
    InvalidCidNamespace,
    CrossNamespaceDuplicate,
    CitizenCidDeleted,
    CitizenCidIdentityChanged,
    CitizenCidStatusInvalid,
    CitizenCidRevocationHeightInvalid,
    InstitutionDeleted,
    InstitutionIdentityChanged,
    InstitutionLegalRepresentativeInvalid,
    FixedInstitutionCreatedAfterGenesis,
    AccountChanged,
    ProtocolAccountDeleted,
    AccountWithoutInstitution,
    AccountAddressMismatch,
    AccountReverseIndexMissing,
    AccountReverseIndexMismatch,
    RequiredProtocolAccountMissing,
    UnexpectedProtocolAccount,
    SingletonInstitutionMissing,
    SingletonInstitutionIdentityMismatch,
    NonGenesisStateImportForbidden,
}

pub mod storage_key {
    use super::*;

    fn storage_prefix(pallet: &[u8], storage: &[u8]) -> Vec<u8> {
        crate::shared::storage_keys::prefix(pallet, storage)
    }

    fn map_vec(pallet: &[u8], storage: &[u8], value: &[u8]) -> Vec<u8> {
        crate::shared::storage_keys::blake2_map(pallet, storage, &value.to_vec().encode())
    }

    fn map_account(pallet: &[u8], storage: &[u8], account: &[u8; 32]) -> Vec<u8> {
        crate::shared::storage_keys::blake2_map(pallet, storage, account)
    }

    fn double_map_vec(pallet: &[u8], storage: &[u8], first: &[u8], second: &[u8]) -> Vec<u8> {
        crate::shared::storage_keys::blake2_double_map(
            pallet,
            storage,
            &first.to_vec().encode(),
            &second.to_vec().encode(),
        )
    }

    pub fn citizen_registry_prefix() -> Vec<u8> {
        storage_prefix(CITIZEN_IDENTITY_PALLET, b"CidRegistry")
    }

    #[cfg(test)]
    pub fn citizen_registry(cid: &[u8]) -> Vec<u8> {
        map_vec(CITIZEN_IDENTITY_PALLET, b"CidRegistry", cid)
    }

    pub fn institution_prefix(namespace: Namespace) -> Vec<u8> {
        storage_prefix(namespace.pallet(), b"Institutions")
    }

    pub fn institution(namespace: Namespace, cid: &[u8]) -> Vec<u8> {
        map_vec(namespace.pallet(), b"Institutions", cid)
    }

    pub fn institution_account_prefix(namespace: Namespace) -> Vec<u8> {
        storage_prefix(namespace.pallet(), b"InstitutionAccounts")
    }

    pub fn institution_account(namespace: Namespace, cid: &[u8], name: &[u8]) -> Vec<u8> {
        double_map_vec(namespace.pallet(), b"InstitutionAccounts", cid, name)
    }

    pub fn account_registered_prefix(namespace: Namespace) -> Vec<u8> {
        storage_prefix(namespace.pallet(), b"AccountRegisteredCid")
    }

    pub fn account_registered(namespace: Namespace, account: &[u8; 32]) -> Vec<u8> {
        map_account(namespace.pallet(), b"AccountRegisteredCid", account)
    }

    /// 启动、runtime 升级和 block#0 状态检查枚举的全部 CID 规范表。
    pub fn enumerated_prefixes() -> Vec<Vec<u8>> {
        vec![
            citizen_registry_prefix(),
            institution_prefix(Namespace::Public),
            institution_prefix(Namespace::Private),
            institution_account_prefix(Namespace::Public),
            institution_account_prefix(Namespace::Private),
            account_registered_prefix(Namespace::Public),
            account_registered_prefix(Namespace::Private),
        ]
    }

    pub fn relevant_prefixes() -> Vec<Vec<u8>> {
        enumerated_prefixes()
    }
}

fn decode_exact<T: Decode>(raw: &[u8], label: &'static str) -> Result<T, GuardError> {
    let mut input = raw;
    let value = T::decode(&mut input).map_err(|_| GuardError::StorageValueDecodeFailed(label))?;
    if !input.is_empty() {
        return Err(GuardError::StorageValueDecodeFailed(label));
    }
    Ok(value)
}

fn parse_vec_map_key(
    key: &[u8],
    prefix: &[u8],
    label: &'static str,
) -> Result<Vec<u8>, GuardError> {
    if !key.starts_with(prefix) || key.len() < prefix.len() + 17 {
        return Err(GuardError::StorageKeyMalformed(label));
    }
    let hash_at = prefix.len();
    let encoded_at = hash_at + 16;
    let encoded = &key[encoded_at..];
    if blake2_128(encoded) != key[hash_at..encoded_at] {
        return Err(GuardError::StorageKeyMalformed(label));
    }
    decode_exact(encoded, label)
}

fn parse_account_map_key(
    key: &[u8],
    prefix: &[u8],
    label: &'static str,
) -> Result<[u8; 32], GuardError> {
    if !key.starts_with(prefix) || key.len() != prefix.len() + 16 + 32 {
        return Err(GuardError::StorageKeyMalformed(label));
    }
    let hash_at = prefix.len();
    let account_at = hash_at + 16;
    if blake2_128(&key[account_at..]) != key[hash_at..account_at] {
        return Err(GuardError::StorageKeyMalformed(label));
    }
    key[account_at..]
        .try_into()
        .map_err(|_| GuardError::StorageKeyMalformed(label))
}

fn parse_double_vec_key(
    key: &[u8],
    prefix: &[u8],
    label: &'static str,
) -> Result<(Vec<u8>, Vec<u8>), GuardError> {
    if !key.starts_with(prefix) || key.len() < prefix.len() + 34 {
        return Err(GuardError::StorageKeyMalformed(label));
    }
    let first_hash_at = prefix.len();
    let first_encoded_at = first_hash_at + 16;
    let first_encoded = &key[first_encoded_at..];
    let mut input = first_encoded;
    let first = Vec::<u8>::decode(&mut input)
        .map_err(|_| GuardError::StorageKeyMalformed(label))?;
    let first_encoded_len = first_encoded.len() - input.len();
    if blake2_128(&first_encoded[..first_encoded_len])
        != key[first_hash_at..first_encoded_at]
    {
        return Err(GuardError::StorageKeyMalformed(label));
    }
    let second_hash_at = first_encoded_at + first_encoded_len;
    if key.len() < second_hash_at + 17 {
        return Err(GuardError::StorageKeyMalformed(label));
    }
    let second_encoded_at = second_hash_at + 16;
    let second_encoded = &key[second_encoded_at..];
    if blake2_128(second_encoded) != key[second_hash_at..second_encoded_at] {
        return Err(GuardError::StorageKeyMalformed(label));
    }
    let second = decode_exact(second_encoded, label)?;
    Ok((first, second))
}

fn parse_cid(cid: &[u8]) -> Result<InstitutionCode, GuardError> {
    parse_cid_number_parts_bytes(cid)
        .map(|parts| parts.institution)
        .map_err(|_| GuardError::InvalidCid)
}

fn validate_cid_namespace(cid: &[u8], namespace: Namespace) -> Result<InstitutionCode, GuardError> {
    let code = parse_cid(cid)?;
    let valid = match namespace {
        Namespace::Public => is_public_legal_code(&code) || is_fixed_governance_code(&code),
        Namespace::Private => is_private_legal_code(&code) || is_unincorporated_code(&code),
    };
    if !valid {
        return Err(GuardError::InvalidCidNamespace);
    }
    if is_fixed_governance_code(&code)
        && primitives::governance_skeleton::fixed_institution_by_cid(cid)
            .is_none_or(|institution| institution.code != code)
    {
        return Err(GuardError::InstitutionIdentityChanged);
    }
    if primitives::institution_constraints::is_permanent_singleton_code(&code)
        && primitives::institution_constraints::singleton_by_cid(cid)
            .is_none_or(|institution| institution.code != code)
    {
        return Err(GuardError::SingletonInstitutionIdentityMismatch);
    }
    Ok(code)
}

fn validate_citizen_record(
    cid: &[u8],
    record: &CitizenCidRecord,
    block: Option<u32>,
) -> Result<(), GuardError> {
    if parse_cid(cid)? != *b"CTZN" || record.registrar_cid_number.is_empty() {
        return Err(GuardError::InvalidCidNamespace);
    }
    match record.status {
        CitizenCidStatus::Active if record.revoked_at.is_none() => {}
        CitizenCidStatus::Revoked if record.revoked_at.is_some() => {}
        _ => return Err(GuardError::CitizenCidStatusInvalid),
    }
    if let Some(block) = block {
        if record.registered_at > block || record.revoked_at.is_some_and(|at| at > block) {
            return Err(GuardError::CitizenCidRevocationHeightInvalid);
        }
    }
    Ok(())
}

fn validate_institution_record(
    namespace: Namespace,
    cid: &[u8],
    record: &InstitutionRecord,
) -> Result<(), GuardError> {
    if validate_cid_namespace(cid, namespace)? != record.institution_code {
        return Err(GuardError::InstitutionIdentityChanged);
    }
    let representative_fields = [
        record.legal_representative_name.is_some(),
        record.legal_representative_cid_number.is_some(),
        record.legal_representative_account.is_some(),
    ];
    if representative_fields.iter().any(|value| *value)
        && !representative_fields.iter().all(|value| *value)
    {
        return Err(GuardError::InstitutionLegalRepresentativeInvalid);
    }
    Ok(())
}

fn validate_account_record(
    cid: &[u8],
    name: &[u8],
    record: &InstitutionAccountRecord,
) -> Result<(), GuardError> {
    let kind = institution_kind_by_name(cid, name).ok_or(GuardError::AccountAddressMismatch)?;
    if kind.derive(SS58_FORMAT) != record.address {
        return Err(GuardError::AccountAddressMismatch);
    }
    Ok(())
}

fn validate_account_reverse<F>(
    namespace: Namespace,
    cid: &[u8],
    name: &[u8],
    record: &InstitutionAccountRecord,
    read: &F,
) -> Result<(), GuardError>
where
    F: Fn(&[u8]) -> Option<Vec<u8>>,
{
    let reverse_raw = read(&storage_key::account_registered(namespace, &record.address))
        .ok_or(GuardError::AccountReverseIndexMissing)?;
    let reverse: RegisteredInstitution = decode_exact(&reverse_raw, "AccountRegisteredCid")?;
    if reverse.cid_number != cid || reverse.account_name != name {
        return Err(GuardError::AccountReverseIndexMismatch);
    }
    Ok(())
}

fn validate_required_accounts<F>(
    namespace: Namespace,
    cid: &[u8],
    record: &InstitutionRecord,
    read: &F,
) -> Result<(), GuardError>
where
    F: Fn(&[u8]) -> Option<Vec<u8>>,
{
    let required = primitives::institution_constraints::required_protocol_account_kinds(
        record.institution_code,
        cid,
    )
    .ok_or(GuardError::InstitutionIdentityChanged)?;
    for kind in required {
        let name = institution_protocol_account_name(*kind);
        let raw = read(&storage_key::institution_account(namespace, cid, name))
            .ok_or(GuardError::RequiredProtocolAccountMissing)?;
        let account: InstitutionAccountRecord = decode_exact(&raw, "InstitutionAccounts")?;
        validate_account_record(cid, name, &account)?;
        validate_account_reverse(namespace, cid, name, &account, read)?;
    }
    Ok(())
}

fn check_citizen_transition(
    block: u32,
    cid: &[u8],
    parent_raw: Option<Vec<u8>>,
    post_raw: Option<Vec<u8>>,
) -> Result<(), GuardError> {
    let post_raw = post_raw.ok_or(GuardError::CitizenCidDeleted)?;
    let post: CitizenCidRecord = decode_exact(&post_raw, "CidRegistry")?;
    validate_citizen_record(cid, &post, Some(block))?;
    let Some(parent_raw) = parent_raw else {
        if post.registered_at != block
            || (post.status == CitizenCidStatus::Revoked && post.revoked_at != Some(block))
        {
            return Err(GuardError::CitizenCidRevocationHeightInvalid);
        }
        return Ok(());
    };
    let parent: CitizenCidRecord = decode_exact(&parent_raw, "CidRegistry")?;
    validate_citizen_record(cid, &parent, None)?;
    if parent.registrar_cid_number != post.registrar_cid_number
        || parent.commitment != post.commitment
        || parent.residence_province_code != post.residence_province_code
        || parent.residence_city_code != post.residence_city_code
        || parent.registered_at != post.registered_at
    {
        return Err(GuardError::CitizenCidIdentityChanged);
    }
    match (parent.status, post.status) {
        (CitizenCidStatus::Active, CitizenCidStatus::Active) if parent_raw == post_raw => Ok(()),
        (CitizenCidStatus::Active, CitizenCidStatus::Revoked) if post.revoked_at == Some(block) => {
            Ok(())
        }
        (CitizenCidStatus::Revoked, CitizenCidStatus::Revoked) if parent_raw == post_raw => Ok(()),
        _ => Err(GuardError::CitizenCidStatusInvalid),
    }
}

/// 普通区块只对本块触及的 CID、机构和账户执行单调性与正反索引校验。
pub fn check_transition<FParent, FPost>(
    block: u32,
    delta: &BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    parent: FParent,
    post: FPost,
    _reference: &GenesisReference,
) -> Result<(), GuardError>
where
    FParent: Fn(&[u8]) -> Option<Vec<u8>>,
    FPost: Fn(&[u8]) -> Option<Vec<u8>>,
{
    let citizen_prefix = storage_key::citizen_registry_prefix();
    for key in delta.keys().filter(|key| key.starts_with(&citizen_prefix)) {
        let cid = parse_vec_map_key(key, &citizen_prefix, "CidRegistry")?;
        check_citizen_transition(block, &cid, parent(key), post(key))?;
    }

    let mut touched_institutions = BTreeSet::<(u8, Vec<u8>)>::new();
    for namespace in [Namespace::Public, Namespace::Private] {
        let namespace_id = if namespace == Namespace::Public { 0 } else { 1 };
        let institution_prefix = storage_key::institution_prefix(namespace);
        let account_prefix = storage_key::institution_account_prefix(namespace);
        let reverse_prefix = storage_key::account_registered_prefix(namespace);

        for key in delta.keys() {
            if key.starts_with(&institution_prefix) {
                let cid = parse_vec_map_key(key, &institution_prefix, "Institutions")?;
                let post_raw = post(key).ok_or(GuardError::InstitutionDeleted)?;
                let post_record: InstitutionRecord = decode_exact(&post_raw, "Institutions")?;
                validate_institution_record(namespace, &cid, &post_record)?;
                if post(&storage_key::institution(namespace.sibling(), &cid)).is_some() {
                    return Err(GuardError::CrossNamespaceDuplicate);
                }
                if let Some(parent_raw) = parent(key) {
                    let parent_record: InstitutionRecord =
                        decode_exact(&parent_raw, "Institutions")?;
                    if parent_record.institution_code != post_record.institution_code
                        || parent_record.created_at != post_record.created_at
                        || parent_record.town_code != post_record.town_code
                    {
                        return Err(GuardError::InstitutionIdentityChanged);
                    }
                } else if is_fixed_governance_code(&post_record.institution_code)
                    || primitives::institution_constraints::is_permanent_singleton_code(
                        &post_record.institution_code,
                    )
                {
                    return Err(GuardError::FixedInstitutionCreatedAfterGenesis);
                }
                touched_institutions.insert((namespace_id, cid));
            }

            if key.starts_with(&account_prefix) {
                let (cid, name) =
                    parse_double_vec_key(key, &account_prefix, "InstitutionAccounts")?;
                let institution_raw = post(&storage_key::institution(namespace, &cid))
                    .ok_or(GuardError::AccountWithoutInstitution)?;
                let institution: InstitutionRecord =
                    decode_exact(&institution_raw, "Institutions")?;
                validate_institution_record(namespace, &cid, &institution)?;
                match (parent(key), post(key)) {
                    (Some(before), Some(after)) if before == after => {}
                    (Some(_), Some(_)) => return Err(GuardError::AccountChanged),
                    (Some(before), None) => {
                        let account: InstitutionAccountRecord =
                            decode_exact(&before, "InstitutionAccounts")?;
                        let kind = institution_kind_by_name(&cid, &name)
                            .ok_or(GuardError::AccountAddressMismatch)?;
                        if !kind.is_closable_institution_account() {
                            return Err(GuardError::ProtocolAccountDeleted);
                        }
                        if post(&storage_key::account_registered(namespace, &account.address))
                            .is_some()
                        {
                            return Err(GuardError::AccountReverseIndexMismatch);
                        }
                    }
                    (None, Some(after)) => {
                        let account: InstitutionAccountRecord =
                            decode_exact(&after, "InstitutionAccounts")?;
                        validate_account_record(&cid, &name, &account)?;
                        if let Some(protocol_kind) = institution_protocol_kind_by_name(&name) {
                            let required =
                                primitives::institution_constraints::required_protocol_account_kinds(
                                    institution.institution_code,
                                    &cid,
                                )
                                .ok_or(GuardError::InstitutionIdentityChanged)?;
                            if !required.contains(&protocol_kind) {
                                return Err(GuardError::UnexpectedProtocolAccount);
                            }
                        }
                        validate_account_reverse(namespace, &cid, &name, &account, &post)?;
                    }
                    (None, None) => {}
                }
                touched_institutions.insert((namespace_id, cid));
            }

            if key.starts_with(&reverse_prefix) {
                let account = parse_account_map_key(key, &reverse_prefix, "AccountRegisteredCid")?;
                if let Some(raw) = post(key) {
                    let registered: RegisteredInstitution =
                        decode_exact(&raw, "AccountRegisteredCid")?;
                    let forward_raw = post(&storage_key::institution_account(
                        namespace,
                        &registered.cid_number,
                        &registered.account_name,
                    ))
                    .ok_or(GuardError::AccountReverseIndexMismatch)?;
                    let forward: InstitutionAccountRecord =
                        decode_exact(&forward_raw, "InstitutionAccounts")?;
                    if forward.address != account {
                        return Err(GuardError::AccountReverseIndexMismatch);
                    }
                    touched_institutions.insert((namespace_id, registered.cid_number));
                }
            }
        }
    }

    for (namespace_id, cid) in touched_institutions {
        let namespace = if namespace_id == 0 {
            Namespace::Public
        } else {
            Namespace::Private
        };
        let raw = post(&storage_key::institution(namespace, &cid))
            .ok_or(GuardError::AccountWithoutInstitution)?;
        let record: InstitutionRecord = decode_exact(&raw, "Institutions")?;
        validate_required_accounts(namespace, &cid, &record, &post)?;
    }
    Ok(())
}

fn validate_full_state<F>(keys: &[Vec<u8>], read: &F) -> Result<(), GuardError>
where
    F: Fn(&[u8]) -> Option<Vec<u8>>,
{
    let citizen_prefix = storage_key::citizen_registry_prefix();
    for key in keys.iter().filter(|key| key.starts_with(&citizen_prefix)) {
        let Some(raw) = read(key) else { continue };
        let cid = parse_vec_map_key(key, &citizen_prefix, "CidRegistry")?;
        let record: CitizenCidRecord = decode_exact(&raw, "CidRegistry")?;
        validate_citizen_record(&cid, &record, None)?;
    }

    let mut occupied_public = BTreeSet::new();
    let mut occupied_private = BTreeSet::new();
    for namespace in [Namespace::Public, Namespace::Private] {
        let institution_prefix = storage_key::institution_prefix(namespace);
        let account_prefix = storage_key::institution_account_prefix(namespace);
        let reverse_prefix = storage_key::account_registered_prefix(namespace);
        let occupied = if namespace == Namespace::Public {
            &mut occupied_public
        } else {
            &mut occupied_private
        };

        for key in keys.iter().filter(|key| key.starts_with(&institution_prefix)) {
            let Some(raw) = read(key) else { continue };
            let cid = parse_vec_map_key(key, &institution_prefix, "Institutions")?;
            let record: InstitutionRecord = decode_exact(&raw, "Institutions")?;
            validate_institution_record(namespace, &cid, &record)?;
            validate_required_accounts(namespace, &cid, &record, read)?;
            occupied.insert(cid);
        }

        for key in keys.iter().filter(|key| key.starts_with(&account_prefix)) {
            let Some(raw) = read(key) else { continue };
            let (cid, name) =
                parse_double_vec_key(key, &account_prefix, "InstitutionAccounts")?;
            if !occupied.contains(&cid) {
                return Err(GuardError::AccountWithoutInstitution);
            }
            let record: InstitutionAccountRecord = decode_exact(&raw, "InstitutionAccounts")?;
            validate_account_record(&cid, &name, &record)?;
            validate_account_reverse(namespace, &cid, &name, &record, read)?;
            if let Some(protocol_kind) = institution_protocol_kind_by_name(&name) {
                let institution_raw = read(&storage_key::institution(namespace, &cid))
                    .ok_or(GuardError::AccountWithoutInstitution)?;
                let institution: InstitutionRecord =
                    decode_exact(&institution_raw, "Institutions")?;
                let required = primitives::institution_constraints::required_protocol_account_kinds(
                    institution.institution_code,
                    &cid,
                )
                .ok_or(GuardError::InstitutionIdentityChanged)?;
                if !required.contains(&protocol_kind) {
                    return Err(GuardError::UnexpectedProtocolAccount);
                }
            }
        }

        for key in keys.iter().filter(|key| key.starts_with(&reverse_prefix)) {
            let Some(raw) = read(key) else { continue };
            let account = parse_account_map_key(key, &reverse_prefix, "AccountRegisteredCid")?;
            let registered: RegisteredInstitution = decode_exact(&raw, "AccountRegisteredCid")?;
            let forward_raw = read(&storage_key::institution_account(
                namespace,
                &registered.cid_number,
                &registered.account_name,
            ))
            .ok_or(GuardError::AccountReverseIndexMismatch)?;
            let forward: InstitutionAccountRecord = decode_exact(&forward_raw, "InstitutionAccounts")?;
            if forward.address != account {
                return Err(GuardError::AccountReverseIndexMismatch);
            }
        }
    }

    if occupied_public.iter().any(|cid| occupied_private.contains(cid)) {
        return Err(GuardError::CrossNamespaceDuplicate);
    }

    for singleton in primitives::institution_constraints::singleton_institutions() {
        let cid = singleton.cid_number.as_bytes();
        let raw = read(&storage_key::institution(Namespace::Public, cid))
            .ok_or(GuardError::SingletonInstitutionMissing)?;
        let record: InstitutionRecord = decode_exact(&raw, "Institutions")?;
        if record.institution_code != singleton.code {
            return Err(GuardError::SingletonInstitutionIdentityMismatch);
        }
        let main_raw = read(&storage_key::institution_account(
            Namespace::Public,
            cid,
            institution_protocol_account_name(InstitutionProtocolAccountKind::Main),
        ))
        .ok_or(GuardError::SingletonInstitutionMissing)?;
        let main: InstitutionAccountRecord = decode_exact(&main_raw, "InstitutionAccounts")?;
        if main.address != singleton.main_account {
            return Err(GuardError::SingletonInstitutionIdentityMismatch);
        }
    }
    Ok(())
}

impl GenesisReference {
    pub fn from_genesis<F>(keys: &[Vec<u8>], read: F) -> Result<Self, GuardError>
    where
        F: Fn(&[u8]) -> Option<Vec<u8>>,
    {
        validate_full_state(keys, &read)?;
        Ok(Self)
    }
}

pub fn check_full_state<F>(
    keys: &[Vec<u8>],
    read: F,
    _reference: &GenesisReference,
) -> Result<(), GuardError>
where
    F: Fn(&[u8]) -> Option<Vec<u8>>,
{
    validate_full_state(keys, &read)
}

pub fn check_imported_genesis<'a, I>(
    pairs: I,
    reference: &GenesisReference,
) -> Result<(), GuardError>
where
    I: IntoIterator<Item = (&'a Vec<u8>, &'a Vec<u8>)>,
{
    let prefixes = storage_key::relevant_prefixes();
    let map: BTreeMap<Vec<u8>, Vec<u8>> = pairs
        .into_iter()
        .filter(|(key, _)| matches_relevant_prefixes(key, &prefixes))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    let keys: Vec<Vec<u8>> = map.keys().cloned().collect();
    check_full_state(&keys, |key| map.get(key).cloned(), reference)
}

/// CID 历史单调性无法由非创世单快照证明，因此禁止非 block#0 状态导入。
pub fn check_state_import_height(block: u32) -> Result<(), GuardError> {
    if block == 0 {
        Ok(())
    } else {
        Err(GuardError::NonGenesisStateImportForbidden)
    }
}

pub fn needs_full_check(delta: &BTreeMap<Vec<u8>, Option<Vec<u8>>>) -> bool {
    delta.contains_key(CODE_KEY)
}

pub fn is_relevant_key(key: &[u8]) -> bool {
    matches_relevant_prefixes(key, &storage_key::relevant_prefixes())
}

pub fn matches_relevant_prefixes(key: &[u8], prefixes: &[Vec<u8>]) -> bool {
    prefixes.iter().any(|prefix| key.starts_with(prefix))
}
