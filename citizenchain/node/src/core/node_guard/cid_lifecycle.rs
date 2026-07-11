//! CID 与机构生命周期节点永久策略。
//!
//! 节点把 `CitizenIdentity::CidRegistry`、公私权 `CidRegisteredAccount/Institutions`
//! 视为唯一规范真源：机构主账户已登记但尚无机构记录表示“占号中”，机构记录 `Active`
//! 表示运行中，`Closed` 表示永久关闭。名称可以依法更新或被新 CID 复用，CID 本身不得删除、
//! 跨公私权重复、换主体或从终态恢复。

use std::collections::{BTreeMap, BTreeSet};

use codec::{Decode, Encode};
use sp_core::hashing::blake2_128;

use primitives::account_derive::RESERVED_NAME_MAIN;
use primitives::cid::code::{
    is_fixed_governance_code, is_private_legal_code, is_public_legal_code, is_unincorporated_code,
    InstitutionCode,
};
use primitives::cid::number::parse_cid_number_parts_bytes;

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
    registrar_account: [u8; 32],
    commitment: [u8; 32],
    residence_province_code: Vec<u8>,
    residence_city_code: Vec<u8>,
    status: CitizenCidStatus,
    registered_at: u32,
    revoked_at: Option<u32>,
}

#[derive(Clone, Copy, Debug, Decode, Encode, Eq, PartialEq)]
enum InstitutionStatus {
    Pending,
    Active,
    Closed,
}

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
struct InstitutionRecord {
    cid_full_name: Vec<u8>,
    cid_short_name: Vec<u8>,
    town_code: Vec<u8>,
    institution_code: InstitutionCode,
    created_at: u32,
    status: InstitutionStatus,
}

#[derive(Clone, Debug, Decode, Eq, PartialEq)]
struct RegisteredInstitution {
    cid_number: Vec<u8>,
    account_name: Vec<u8>,
}

#[derive(Clone, Debug, Decode, Eq, PartialEq)]
struct InstitutionAccountRecord {
    address: [u8; 32],
    _initial_balance: u128,
    status: InstitutionStatus,
    _is_default: bool,
    _created_at: u32,
}

/// block#0 派生的创世封存账户基准。
#[derive(Clone, Debug, Default)]
pub struct GenesisReference {
    protected_accounts: BTreeSet<[u8; 32]>,
    /// `ProtectedGenesisAccounts` 及其三条规范索引必须与创世逐字一致。
    frozen_values: BTreeMap<Vec<u8>, Vec<u8>>,
}

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
    InstitutionStatusInvalid,
    ClosedInstitutionChanged,
    FixedInstitutionNotActive,
    MainReservationMissing,
    MainReservationDeletedBeforeClosed,
    RegistrationChanged,
    ReusedInstitutionCid,
    ProtectedGenesisSetChanged,
    ProtectedGenesisValueChanged,
    ProtectedGenesisIndexMissing,
    ProtectedGenesisAccountNotActive,
    NonGenesisStateImportForbidden,
}

pub mod storage_key {
    use super::*;

    // 以下四个私有 helper 是 `crate::shared::storage_keys` 单源的薄委托:
    // map_vec/double_map_vec 内部 SCALE 编码键(Vec 键带 compact 长度前缀),
    // map_account 传裸 32 字节(AccountId32 无长度前缀)。
    fn storage_prefix(pallet: &[u8], storage: &[u8]) -> Vec<u8> {
        crate::shared::storage_keys::prefix(pallet, storage)
    }

    fn map_vec(pallet: &[u8], storage: &[u8], value: &[u8]) -> Vec<u8> {
        crate::shared::storage_keys::blake2_map(pallet, storage, &value.encode())
    }

    fn map_account(pallet: &[u8], storage: &[u8], account: &[u8; 32]) -> Vec<u8> {
        crate::shared::storage_keys::blake2_map(pallet, storage, account)
    }

    fn double_map_vec(pallet: &[u8], storage: &[u8], first: &[u8], second: &[u8]) -> Vec<u8> {
        crate::shared::storage_keys::blake2_double_map(
            pallet,
            storage,
            &first.encode(),
            &second.encode(),
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

    pub fn registration_prefix(namespace: Namespace) -> Vec<u8> {
        storage_prefix(namespace.pallet(), b"CidRegisteredAccount")
    }

    pub fn registration(namespace: Namespace, cid: &[u8], name: &[u8]) -> Vec<u8> {
        double_map_vec(namespace.pallet(), b"CidRegisteredAccount", cid, name)
    }

    pub fn main_registration(namespace: Namespace, cid: &[u8]) -> Vec<u8> {
        registration(namespace, cid, RESERVED_NAME_MAIN)
    }

    pub fn account_registered(namespace: Namespace, account: &[u8; 32]) -> Vec<u8> {
        map_account(namespace.pallet(), b"AccountRegisteredCid", account)
    }

    fn account_registered_prefix(namespace: Namespace) -> Vec<u8> {
        storage_prefix(namespace.pallet(), b"AccountRegisteredCid")
    }

    pub fn institution_account(namespace: Namespace, cid: &[u8], name: &[u8]) -> Vec<u8> {
        double_map_vec(namespace.pallet(), b"InstitutionAccounts", cid, name)
    }

    fn institution_account_prefix(namespace: Namespace) -> Vec<u8> {
        storage_prefix(namespace.pallet(), b"InstitutionAccounts")
    }

    pub fn protected_prefix() -> Vec<u8> {
        storage_prefix(PUBLIC_MANAGE_PALLET, b"ProtectedGenesisAccounts")
    }

    /// 启动、runtime 升级和 block#0 状态检查需要枚举的规范表。
    pub fn enumerated_prefixes() -> Vec<Vec<u8>> {
        vec![
            citizen_registry_prefix(),
            institution_prefix(Namespace::Public),
            institution_prefix(Namespace::Private),
            registration_prefix(Namespace::Public),
            registration_prefix(Namespace::Private),
            protected_prefix(),
        ]
    }

    /// block#0 导入时还需携带创世封存账户关联索引；这些大表不参与日常启动枚举。
    pub fn imported_support_prefixes() -> Vec<Vec<u8>> {
        vec![
            account_registered_prefix(Namespace::Public),
            institution_account_prefix(Namespace::Public),
        ]
    }

    /// block#0 完整导入态内与 CID 生命周期有关的全部 RAW 表前缀。
    pub fn relevant_prefixes() -> Vec<Vec<u8>> {
        let mut prefixes = enumerated_prefixes();
        prefixes.extend(imported_support_prefixes());
        prefixes
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
    let hash = &key[prefix.len()..prefix.len() + 16];
    let encoded = &key[prefix.len() + 16..];
    if blake2_128(encoded) != hash {
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
    let account: [u8; 32] = key[prefix.len() + 16..]
        .try_into()
        .map_err(|_| GuardError::StorageKeyMalformed(label))?;
    if blake2_128(&account) != key[prefix.len()..prefix.len() + 16] {
        return Err(GuardError::StorageKeyMalformed(label));
    }
    Ok(account)
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
    let mut first_input = &key[first_encoded_at..];
    let first =
        Vec::<u8>::decode(&mut first_input).map_err(|_| GuardError::StorageKeyMalformed(label))?;
    let first_encoded_len = key[first_encoded_at..].len() - first_input.len();
    let first_encoded = &key[first_encoded_at..first_encoded_at + first_encoded_len];
    if blake2_128(first_encoded) != key[first_hash_at..first_encoded_at] {
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
    if valid {
        Ok(code)
    } else {
        Err(GuardError::InvalidCidNamespace)
    }
}

fn decode_institution<F>(
    namespace: Namespace,
    cid: &[u8],
    read: &F,
) -> Result<Option<InstitutionRecord>, GuardError>
where
    F: Fn(&[u8]) -> Option<Vec<u8>>,
{
    read(&storage_key::institution(namespace, cid))
        .map(|raw| decode_exact(&raw, "Institutions"))
        .transpose()
}

fn namespace_occupied<F>(namespace: Namespace, cid: &[u8], read: &F) -> bool
where
    F: Fn(&[u8]) -> Option<Vec<u8>>,
{
    read(&storage_key::institution(namespace, cid)).is_some()
        || read(&storage_key::main_registration(namespace, cid)).is_some()
}

fn validate_citizen_record(
    cid: &[u8],
    record: &CitizenCidRecord,
    block: Option<u32>,
) -> Result<(), GuardError> {
    let code = parse_cid(cid)?;
    if code != *b"CTZN" {
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
    if parent.registrar_account != post.registrar_account
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

fn validate_institution_record(
    namespace: Namespace,
    cid: &[u8],
    record: &InstitutionRecord,
) -> Result<(), GuardError> {
    let code = validate_cid_namespace(cid, namespace)?;
    if code != record.institution_code {
        return Err(GuardError::InstitutionIdentityChanged);
    }
    if is_fixed_governance_code(&code) && record.status != InstitutionStatus::Active {
        return Err(GuardError::FixedInstitutionNotActive);
    }
    Ok(())
}

fn check_institution_transition<FParent, FPost>(
    namespace: Namespace,
    cid: &[u8],
    parent_raw: Option<Vec<u8>>,
    post_raw: Option<Vec<u8>>,
    parent: &FParent,
    post: &FPost,
) -> Result<(), GuardError>
where
    FParent: Fn(&[u8]) -> Option<Vec<u8>>,
    FPost: Fn(&[u8]) -> Option<Vec<u8>>,
{
    let post_raw = post_raw.ok_or(GuardError::InstitutionDeleted)?;
    let post_record: InstitutionRecord = decode_exact(&post_raw, "Institutions")?;
    validate_institution_record(namespace, cid, &post_record)?;
    if namespace_occupied(namespace.sibling(), cid, post) {
        return Err(GuardError::CrossNamespaceDuplicate);
    }
    let Some(parent_raw) = parent_raw else {
        if is_fixed_governance_code(&post_record.institution_code) {
            return Err(GuardError::FixedInstitutionNotActive);
        }
        if post_record.status != InstitutionStatus::Closed
            && post(&storage_key::main_registration(namespace, cid)).is_none()
        {
            return Err(GuardError::MainReservationMissing);
        }
        return Ok(());
    };
    let parent_record: InstitutionRecord = decode_exact(&parent_raw, "Institutions")?;
    validate_institution_record(namespace, cid, &parent_record)?;
    if parent_record.institution_code != post_record.institution_code
        || parent_record.created_at != post_record.created_at
        || parent_record.town_code != post_record.town_code
    {
        return Err(GuardError::InstitutionIdentityChanged);
    }
    match (parent_record.status, post_record.status) {
        (InstitutionStatus::Pending, InstitutionStatus::Pending)
        | (InstitutionStatus::Pending, InstitutionStatus::Active)
        | (InstitutionStatus::Pending, InstitutionStatus::Closed)
        | (InstitutionStatus::Active, InstitutionStatus::Active)
        | (InstitutionStatus::Active, InstitutionStatus::Closed) => Ok(()),
        (InstitutionStatus::Closed, InstitutionStatus::Closed) if parent_raw == post_raw => Ok(()),
        (InstitutionStatus::Closed, InstitutionStatus::Closed) => {
            Err(GuardError::ClosedInstitutionChanged)
        }
        _ => Err(GuardError::InstitutionStatusInvalid),
    }?;
    if post_record.status != InstitutionStatus::Closed
        && post(&storage_key::main_registration(namespace, cid)).is_none()
    {
        return Err(GuardError::MainReservationMissing);
    }
    // 防止被升级后的 runtime 在父状态已有墓碑时重新创建主账户登记。
    if parent_record.status == InstitutionStatus::Closed
        && parent(&storage_key::main_registration(namespace, cid)).is_none()
        && post(&storage_key::main_registration(namespace, cid)).is_some()
    {
        return Err(GuardError::ReusedInstitutionCid);
    }
    Ok(())
}

fn check_registration_transition<FParent, FPost>(
    namespace: Namespace,
    cid: &[u8],
    name: &[u8],
    parent_raw: Option<Vec<u8>>,
    post_raw: Option<Vec<u8>>,
    parent: &FParent,
    post: &FPost,
) -> Result<(), GuardError>
where
    FParent: Fn(&[u8]) -> Option<Vec<u8>>,
    FPost: Fn(&[u8]) -> Option<Vec<u8>>,
{
    validate_cid_namespace(cid, namespace)?;
    match (parent_raw, post_raw) {
        (Some(before), Some(after)) if before == after => {}
        (Some(_), Some(_)) => return Err(GuardError::RegistrationChanged),
        (None, Some(after)) => {
            let _: [u8; 32] = decode_exact(&after, "CidRegisteredAccount")?;
            if let Some(existing) = decode_institution(namespace, cid, parent)? {
                if existing.status == InstitutionStatus::Closed || name == RESERVED_NAME_MAIN {
                    return Err(GuardError::ReusedInstitutionCid);
                }
            }
            if namespace_occupied(namespace.sibling(), cid, post) {
                return Err(GuardError::CrossNamespaceDuplicate);
            }
            if post(&storage_key::main_registration(namespace, cid)).is_none() {
                return Err(GuardError::MainReservationMissing);
            }
        }
        (Some(_), None) if name == RESERVED_NAME_MAIN => {
            let closed = decode_institution(namespace, cid, post)?
                .is_some_and(|record| record.status == InstitutionStatus::Closed);
            if !closed {
                return Err(GuardError::MainReservationDeletedBeforeClosed);
            }
        }
        (Some(_), None) => {}
        (None, None) => {}
    }
    Ok(())
}

/// 普通区块只对本块 delta 中触及的 CID 记录执行父/后状态单调性校验。
pub fn check_transition<FParent, FPost>(
    block: u32,
    delta: &BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    parent: FParent,
    post: FPost,
    reference: &GenesisReference,
) -> Result<(), GuardError>
where
    FParent: Fn(&[u8]) -> Option<Vec<u8>>,
    FPost: Fn(&[u8]) -> Option<Vec<u8>>,
{
    for (key, expected) in &reference.frozen_values {
        if delta.contains_key(key) && post(key) != Some(expected.clone()) {
            return Err(GuardError::ProtectedGenesisValueChanged);
        }
    }

    let protected_prefix = storage_key::protected_prefix();
    let citizen_prefix = storage_key::citizen_registry_prefix();
    for key in delta.keys() {
        if key.starts_with(&protected_prefix) {
            let account =
                parse_account_map_key(key, &protected_prefix, "ProtectedGenesisAccounts")?;
            if !reference.protected_accounts.contains(&account)
                || post(key) != reference.frozen_values.get(key).cloned()
            {
                return Err(GuardError::ProtectedGenesisSetChanged);
            }
        }
        if key.starts_with(&citizen_prefix) {
            let cid = parse_vec_map_key(key, &citizen_prefix, "CidRegistry")?;
            check_citizen_transition(block, &cid, parent(key), post(key))?;
        }
    }

    for namespace in [Namespace::Public, Namespace::Private] {
        let institution_prefix = storage_key::institution_prefix(namespace);
        let registration_prefix = storage_key::registration_prefix(namespace);
        for key in delta.keys() {
            if key.starts_with(&institution_prefix) {
                let cid = parse_vec_map_key(key, &institution_prefix, "Institutions")?;
                check_institution_transition(
                    namespace,
                    &cid,
                    parent(key),
                    post(key),
                    &parent,
                    &post,
                )?;
            }
            if key.starts_with(&registration_prefix) {
                let (cid, name) =
                    parse_double_vec_key(key, &registration_prefix, "CidRegisteredAccount")?;
                check_registration_transition(
                    namespace,
                    &cid,
                    &name,
                    parent(key),
                    post(key),
                    &parent,
                    &post,
                )?;
            }
        }
    }
    Ok(())
}

fn validate_full_state<F>(
    keys: &[Vec<u8>],
    read: &F,
    reference: Option<&GenesisReference>,
) -> Result<(), GuardError>
where
    F: Fn(&[u8]) -> Option<Vec<u8>>,
{
    let mut occupied_public = BTreeSet::new();
    let mut occupied_private = BTreeSet::new();
    let citizen_prefix = storage_key::citizen_registry_prefix();
    for key in keys.iter().filter(|key| key.starts_with(&citizen_prefix)) {
        let Some(raw) = read(key) else { continue };
        let cid = parse_vec_map_key(key, &citizen_prefix, "CidRegistry")?;
        let record: CitizenCidRecord = decode_exact(&raw, "CidRegistry")?;
        validate_citizen_record(&cid, &record, None)?;
    }

    for namespace in [Namespace::Public, Namespace::Private] {
        let institution_prefix = storage_key::institution_prefix(namespace);
        let registration_prefix = storage_key::registration_prefix(namespace);
        let occupied = match namespace {
            Namespace::Public => &mut occupied_public,
            Namespace::Private => &mut occupied_private,
        };
        for key in keys
            .iter()
            .filter(|key| key.starts_with(&institution_prefix))
        {
            let Some(raw) = read(key) else { continue };
            let cid = parse_vec_map_key(key, &institution_prefix, "Institutions")?;
            let record: InstitutionRecord = decode_exact(&raw, "Institutions")?;
            validate_institution_record(namespace, &cid, &record)?;
            if record.status != InstitutionStatus::Closed
                && read(&storage_key::main_registration(namespace, &cid)).is_none()
            {
                return Err(GuardError::MainReservationMissing);
            }
            occupied.insert(cid);
        }
        for key in keys
            .iter()
            .filter(|key| key.starts_with(&registration_prefix))
        {
            let Some(raw) = read(key) else { continue };
            let (cid, _name) =
                parse_double_vec_key(key, &registration_prefix, "CidRegisteredAccount")?;
            validate_cid_namespace(&cid, namespace)?;
            let _: [u8; 32] = decode_exact(&raw, "CidRegisteredAccount")?;
            if read(&storage_key::main_registration(namespace, &cid)).is_none() {
                return Err(GuardError::MainReservationMissing);
            }
            occupied.insert(cid);
        }
    }
    if occupied_public
        .iter()
        .any(|cid| occupied_private.contains(cid))
    {
        return Err(GuardError::CrossNamespaceDuplicate);
    }

    if let Some(reference) = reference {
        let protected_prefix = storage_key::protected_prefix();
        let protected: BTreeSet<[u8; 32]> = keys
            .iter()
            .filter(|key| key.starts_with(&protected_prefix))
            .filter(|key| read(key).is_some())
            .map(|key| parse_account_map_key(key, &protected_prefix, "ProtectedGenesisAccounts"))
            .collect::<Result<_, _>>()?;
        if protected != reference.protected_accounts {
            return Err(GuardError::ProtectedGenesisSetChanged);
        }
        for (key, value) in &reference.frozen_values {
            if read(key) != Some(value.clone()) {
                return Err(GuardError::ProtectedGenesisValueChanged);
            }
        }
    }
    Ok(())
}

impl GenesisReference {
    /// 从 block#0 的规范表和直接 RAW 读取构造创世封存基准。
    pub fn from_genesis<F>(keys: &[Vec<u8>], read: F) -> Result<Self, GuardError>
    where
        F: Fn(&[u8]) -> Option<Vec<u8>>,
    {
        validate_full_state(keys, &read, None)?;
        let protected_prefix = storage_key::protected_prefix();
        let mut reference = Self::default();
        for key in keys.iter().filter(|key| key.starts_with(&protected_prefix)) {
            let Some(value) = read(key) else { continue };
            let account =
                parse_account_map_key(key, &protected_prefix, "ProtectedGenesisAccounts")?;
            reference.protected_accounts.insert(account);
            reference.frozen_values.insert(key.clone(), value);

            let registered_key = storage_key::account_registered(Namespace::Public, &account);
            let registered_raw =
                read(&registered_key).ok_or(GuardError::ProtectedGenesisIndexMissing)?;
            let registered: RegisteredInstitution =
                decode_exact(&registered_raw, "AccountRegisteredCid")?;
            let forward_key = storage_key::registration(
                Namespace::Public,
                &registered.cid_number,
                &registered.account_name,
            );
            let forward_raw = read(&forward_key).ok_or(GuardError::ProtectedGenesisIndexMissing)?;
            let forward_account: [u8; 32] = decode_exact(&forward_raw, "CidRegisteredAccount")?;
            if forward_account != account {
                return Err(GuardError::ProtectedGenesisIndexMissing);
            }
            let account_key = storage_key::institution_account(
                Namespace::Public,
                &registered.cid_number,
                &registered.account_name,
            );
            let account_raw = read(&account_key).ok_or(GuardError::ProtectedGenesisIndexMissing)?;
            let account_record: InstitutionAccountRecord =
                decode_exact(&account_raw, "InstitutionAccounts")?;
            if account_record.address != account
                || account_record.status != InstitutionStatus::Active
            {
                return Err(GuardError::ProtectedGenesisAccountNotActive);
            }
            reference
                .frozen_values
                .insert(registered_key, registered_raw);
            reference.frozen_values.insert(forward_key, forward_raw);
            reference.frozen_values.insert(account_key, account_raw);
        }
        validate_full_state(keys, &read, Some(&reference))?;
        Ok(reference)
    }
}

/// runtime `:code` 变化时枚举规范表并执行完整结构复核。
pub fn check_full_state<F>(
    keys: &[Vec<u8>],
    read: F,
    reference: &GenesisReference,
) -> Result<(), GuardError>
where
    F: Fn(&[u8]) -> Option<Vec<u8>>,
{
    validate_full_state(keys, &read, Some(reference))
}

/// block#0 导入态必须与本节点的创世保护基准一致。
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

/// CID 历史单调性无法由非创世单快照证明，故严格禁止非 block#0 状态导入。
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

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::cid::generator::{generate_cid_number, GenerateCidNumberInput};

    fn cid(tag: &str, institution: &str) -> Vec<u8> {
        generate_cid_number(GenerateCidNumberInput {
            account_pubkey: tag,
            p1: "0",
            province_code: "ZS",
            province_name: "中枢省",
            city_code: "001",
            city_name: "测试市",
            year: "2026",
            institution,
        })
        .unwrap()
        .into_bytes()
    }

    fn citizen_record(status: CitizenCidStatus, registered_at: u32) -> CitizenCidRecord {
        CitizenCidRecord {
            registrar_account: [1; 32],
            commitment: [2; 32],
            residence_province_code: b"ZS".to_vec(),
            residence_city_code: b"001".to_vec(),
            status,
            registered_at,
            revoked_at: (status == CitizenCidStatus::Revoked).then_some(registered_at),
        }
    }

    fn institution_record(cid: &[u8], status: InstitutionStatus, name: &[u8]) -> InstitutionRecord {
        InstitutionRecord {
            cid_full_name: name.to_vec(),
            cid_short_name: name.to_vec(),
            town_code: Vec::new(),
            institution_code: parse_cid(cid).unwrap(),
            created_at: 1,
            status,
        }
    }

    fn map_reader(map: &BTreeMap<Vec<u8>, Vec<u8>>) -> impl Fn(&[u8]) -> Option<Vec<u8>> + '_ {
        |key| map.get(key).cloned()
    }

    #[test]
    fn citizen_active_to_revoked_is_terminal() {
        let cid = cid("citizen-a", "CTZN");
        let key = storage_key::citizen_registry(&cid);
        let mut parent = BTreeMap::new();
        let mut post = BTreeMap::new();
        parent.insert(
            key.clone(),
            citizen_record(CitizenCidStatus::Active, 1).encode(),
        );
        let mut revoked = citizen_record(CitizenCidStatus::Revoked, 1);
        revoked.revoked_at = Some(2);
        post.insert(key.clone(), revoked.encode());
        let delta = BTreeMap::from([(key.clone(), post.get(&key).cloned())]);
        assert_eq!(
            check_transition(
                2,
                &delta,
                map_reader(&parent),
                map_reader(&post),
                &GenesisReference::default(),
            ),
            Ok(())
        );

        let restored = citizen_record(CitizenCidStatus::Active, 1).encode();
        let restored_post = BTreeMap::from([(key.clone(), restored.clone())]);
        let restored_delta = BTreeMap::from([(key, Some(restored))]);
        assert_eq!(
            check_transition(
                3,
                &restored_delta,
                map_reader(&post),
                map_reader(&restored_post),
                &GenesisReference::default(),
            ),
            Err(GuardError::CitizenCidStatusInvalid)
        );
    }

    #[test]
    fn citizen_delete_or_change_identity_is_rejected() {
        let cid = cid("citizen-b", "CTZN");
        let key = storage_key::citizen_registry(&cid);
        let original = citizen_record(CitizenCidStatus::Active, 1);
        let parent = BTreeMap::from([(key.clone(), original.encode())]);
        let delta = BTreeMap::from([(key.clone(), None)]);
        assert_eq!(
            check_transition(
                2,
                &delta,
                map_reader(&parent),
                |_| None,
                &GenesisReference::default(),
            ),
            Err(GuardError::CitizenCidDeleted)
        );

        let mut changed = original;
        changed.commitment = [9; 32];
        let post = BTreeMap::from([(key.clone(), changed.encode())]);
        let delta = BTreeMap::from([(key, post.values().next().cloned())]);
        assert_eq!(
            check_transition(
                2,
                &delta,
                map_reader(&parent),
                map_reader(&post),
                &GenesisReference::default(),
            ),
            Err(GuardError::CitizenCidIdentityChanged)
        );
    }

    #[test]
    fn active_institution_can_rename_then_close_but_not_reopen() {
        let cid = cid("public-a", "CGOV");
        let institution_key = storage_key::institution(Namespace::Public, &cid);
        let main_key = storage_key::main_registration(Namespace::Public, &cid);
        let main = [3u8; 32].encode();
        let parent = BTreeMap::from([
            (
                institution_key.clone(),
                institution_record(&cid, InstitutionStatus::Active, b"old").encode(),
            ),
            (main_key.clone(), main.clone()),
        ]);
        let renamed = BTreeMap::from([
            (
                institution_key.clone(),
                institution_record(&cid, InstitutionStatus::Active, b"new").encode(),
            ),
            (main_key.clone(), main),
        ]);
        let delta = BTreeMap::from([(
            institution_key.clone(),
            renamed.get(&institution_key).cloned(),
        )]);
        assert_eq!(
            check_transition(
                2,
                &delta,
                map_reader(&parent),
                map_reader(&renamed),
                &GenesisReference::default(),
            ),
            Ok(())
        );

        let closed = BTreeMap::from([(
            institution_key.clone(),
            institution_record(&cid, InstitutionStatus::Closed, b"new").encode(),
        )]);
        let close_delta = BTreeMap::from([
            (
                institution_key.clone(),
                closed.get(&institution_key).cloned(),
            ),
            (main_key.clone(), None),
        ]);
        assert_eq!(
            check_transition(
                3,
                &close_delta,
                map_reader(&renamed),
                map_reader(&closed),
                &GenesisReference::default(),
            ),
            Ok(())
        );

        let reopened = BTreeMap::from([
            (
                institution_key.clone(),
                institution_record(&cid, InstitutionStatus::Active, b"new").encode(),
            ),
            (main_key, [4u8; 32].encode()),
        ]);
        let reopen_delta = BTreeMap::from([(
            institution_key.clone(),
            reopened.get(&institution_key).cloned(),
        )]);
        assert_eq!(
            check_transition(
                4,
                &reopen_delta,
                map_reader(&closed),
                map_reader(&reopened),
                &GenesisReference::default(),
            ),
            Err(GuardError::InstitutionStatusInvalid)
        );
    }

    #[test]
    fn same_name_with_new_cid_is_allowed_but_cross_namespace_duplicate_is_not() {
        let old_cid = cid("public-old", "CGOV");
        let new_cid = cid("public-new", "CGOV");
        let old_key = storage_key::institution(Namespace::Public, &old_cid);
        let new_key = storage_key::institution(Namespace::Public, &new_cid);
        let new_main = storage_key::main_registration(Namespace::Public, &new_cid);
        let parent = BTreeMap::from([(
            old_key,
            institution_record(&old_cid, InstitutionStatus::Closed, b"same").encode(),
        )]);
        let mut post = parent.clone();
        post.insert(
            new_key.clone(),
            institution_record(&new_cid, InstitutionStatus::Active, b"same").encode(),
        );
        post.insert(new_main, [5u8; 32].encode());
        let delta = BTreeMap::from([(new_key.clone(), post.get(&new_key).cloned())]);
        assert_eq!(
            check_transition(
                2,
                &delta,
                map_reader(&parent),
                map_reader(&post),
                &GenesisReference::default(),
            ),
            Ok(())
        );

        let private_key = storage_key::institution(Namespace::Private, &new_cid);
        post.insert(
            private_key.clone(),
            institution_record(&new_cid, InstitutionStatus::Active, b"same").encode(),
        );
        let delta = BTreeMap::from([(private_key, post.values().next().cloned())]);
        assert!(matches!(
            check_transition(
                2,
                &delta,
                map_reader(&parent),
                map_reader(&post),
                &GenesisReference::default(),
            ),
            Err(GuardError::InvalidCidNamespace | GuardError::CrossNamespaceDuplicate)
        ));
    }

    #[test]
    fn main_reservation_cannot_disappear_before_closed() {
        let cid = cid("private-a", "SFLP");
        let main_key = storage_key::main_registration(Namespace::Private, &cid);
        let parent = BTreeMap::from([(main_key.clone(), [7u8; 32].encode())]);
        let delta = BTreeMap::from([(main_key, None)]);
        assert_eq!(
            check_transition(
                2,
                &delta,
                map_reader(&parent),
                |_| None,
                &GenesisReference::default(),
            ),
            Err(GuardError::MainReservationDeletedBeforeClosed)
        );
    }

    #[test]
    fn main_reservation_without_institution_is_valid_pending_state() {
        let cid = cid("private-pending", "SFLP");
        let main_key = storage_key::main_registration(Namespace::Private, &cid);
        let post = BTreeMap::from([(main_key.clone(), [8u8; 32].encode())]);
        let delta = BTreeMap::from([(main_key, post.values().next().cloned())]);
        assert_eq!(
            check_transition(
                1,
                &delta,
                |_| None,
                map_reader(&post),
                &GenesisReference::default(),
            ),
            Ok(())
        );
    }

    #[test]
    fn fixed_governance_institution_cannot_leave_active() {
        let cid = cid("fixed-nrc", "NRC");
        let pending = institution_record(&cid, InstitutionStatus::Pending, b"NRC");
        assert_eq!(
            validate_institution_record(Namespace::Public, &cid, &pending),
            Err(GuardError::FixedInstitutionNotActive)
        );
        let closed = institution_record(&cid, InstitutionStatus::Closed, b"NRC");
        assert_eq!(
            validate_institution_record(Namespace::Public, &cid, &closed),
            Err(GuardError::FixedInstitutionNotActive)
        );
    }

    #[test]
    fn non_genesis_state_import_is_forbidden() {
        assert_eq!(check_state_import_height(0), Ok(()));
        assert_eq!(
            check_state_import_height(1),
            Err(GuardError::NonGenesisStateImportForbidden)
        );
    }

    #[test]
    fn real_runtime_genesis_satisfies_cid_lifecycle_reference() {
        use sp_runtime::BuildStorage;
        let storage = citizenchain::RuntimeGenesisConfig::default()
            .build_storage()
            .expect("build runtime genesis storage");
        let top = storage.top;
        let keys: Vec<Vec<u8>> = top
            .keys()
            .filter(|key| is_relevant_key(key))
            .cloned()
            .collect();
        let reference = GenesisReference::from_genesis(&keys, |key| top.get(key).cloned())
            .expect("真实 runtime 创世必须满足 CID 永久规则");
        assert!(!reference.protected_accounts.is_empty());
        assert!(!reference.frozen_values.is_empty());
        assert_eq!(check_imported_genesis(top.iter(), &reference), Ok(()));

        let frozen_key = reference.frozen_values.keys().next().unwrap().clone();
        let delta = BTreeMap::from([(frozen_key.clone(), None)]);
        assert_eq!(
            check_transition(
                1,
                &delta,
                |key| top.get(key).cloned(),
                |key| (key != frozen_key.as_slice())
                    .then(|| top.get(key).cloned())
                    .flatten(),
                &reference,
            ),
            Err(GuardError::ProtectedGenesisValueChanged)
        );
    }

    #[test]
    fn malformed_cid_key_and_trailing_record_are_rejected() {
        let number = cid("malformed-cid-key", "CTZN");
        let record = citizen_record(CitizenCidStatus::Active, 1).encode();
        let mut malformed = storage_key::citizen_registry(&number);
        malformed[storage_key::citizen_registry_prefix().len()] ^= 1;
        let malformed_delta = BTreeMap::from([(malformed.clone(), Some(record.clone()))]);
        let malformed_post = BTreeMap::from([(malformed, record.clone())]);
        assert_eq!(
            check_transition(
                1,
                &malformed_delta,
                |_| None,
                |key| malformed_post.get(key).cloned(),
                &GenesisReference::default(),
            ),
            Err(GuardError::StorageKeyMalformed("CidRegistry"))
        );

        let key = storage_key::citizen_registry(&number);
        let mut trailing = record;
        trailing.push(0xff);
        let trailing_delta = BTreeMap::from([(key.clone(), Some(trailing.clone()))]);
        let trailing_post = BTreeMap::from([(key, trailing)]);
        assert_eq!(
            check_transition(
                1,
                &trailing_delta,
                |_| None,
                |key| trailing_post.get(key).cloned(),
                &GenesisReference::default(),
            ),
            Err(GuardError::StorageValueDecodeFailed("CidRegistry"))
        );
    }
}
