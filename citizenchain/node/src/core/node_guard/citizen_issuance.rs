//! 公民轻节点认证发行节点永久策略。
//!
//! 身份登记 extrinsic 只生成本块待发凭据，runtime 在同块 finalize 实际铸发。节点从
//! finalize 前队列、公民身份 RAW 状态和编译期制度常量独立推导收款账户与金额，再把结果
//! 登记到统一 `FinalizeIssuancePlan`；事件和 runtime metadata 都不作为信任来源。

use std::collections::{BTreeMap, BTreeSet};

use codec::{Decode, Encode};
use sp_core::hashing::{blake2_256, twox_128};
// twox_64 只在测试期的 key 校验(parse_twox_u32_key)使用;运行期键构造已收敛 shared::storage_keys。
#[cfg(test)]
use sp_core::hashing::twox_64;

use primitives::citizen_const::{
    CITIZEN_ISSUANCE_HIGH_REWARD, CITIZEN_ISSUANCE_HIGH_REWARD_COUNT, CITIZEN_ISSUANCE_MAX_COUNT,
    CITIZEN_ISSUANCE_NORMAL_REWARD,
};

use super::FinalizeIssuancePlan;

const CITIZEN_ISSUANCE_PALLET: &[u8] = b"CitizenIssuance";
const CITIZEN_IDENTITY_PALLET: &[u8] = b"CitizenIdentity";

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
struct PendingCertificationReward {
    who: [u8; 32],
    cid_number_hash: [u8; 32],
}

#[derive(Clone, Copy, Debug, Decode, Encode, Eq, PartialEq)]
enum CitizenStatus {
    Normal,
    Revoked,
}

#[derive(Clone, Debug, Decode, Eq, PartialEq)]
struct VotingIdentity {
    cid_number: Vec<u8>,
    _passport_valid_from: u32,
    _passport_valid_until: u32,
    _citizen_status: CitizenStatus,
    _residence_province_code: Vec<u8>,
    _residence_city_code: Vec<u8>,
    _residence_town_code: Vec<u8>,
    _updated_at: u32,
}

#[derive(Debug, Eq, PartialEq)]
pub enum GuardError {
    #[cfg(test)]
    StorageKeyMalformed(&'static str),
    StorageValueDecodeFailed(&'static str),
    GenesisStateNotEmpty(Vec<u8>),
    PendingCountChangedOutsideExtrinsic,
    PendingCountExceedsExtrinsics,
    PendingQueueGap,
    PendingQueueUnexpectedKey,
    PendingQueueNotCleared,
    PendingDuplicate,
    PendingMarkerMissing,
    PermanentMarkerChangedBeforeFinalize,
    PermanentMarkerMissingAfterFinalize,
    RewardedCountChangedBeforeFinalize,
    RewardedCountInvalid,
    RewardLimitExceeded,
    FirstIdentityMissing,
    IdentityHashMismatch,
    IdentityReverseIndexMismatch,
    IdentityAlreadyExisted,
    RewardAlreadyClaimed,
    FinalizeIssuanceOverflow,
}

pub mod storage_key {
    use super::*;

    // `crate::shared::storage_keys` 单源的薄委托;blake2_map_raw 传已编码键(裸哈希入参)。
    fn storage_prefix(pallet: &[u8], storage: &[u8]) -> Vec<u8> {
        crate::shared::storage_keys::prefix(pallet, storage)
    }

    fn blake2_map_raw(pallet: &[u8], storage: &[u8], encoded_key: &[u8]) -> Vec<u8> {
        crate::shared::storage_keys::blake2_map(pallet, storage, encoded_key)
    }

    pub fn pallet_prefix() -> [u8; 16] {
        twox_128(CITIZEN_ISSUANCE_PALLET)
    }

    pub fn rewarded_count() -> Vec<u8> {
        storage_prefix(CITIZEN_ISSUANCE_PALLET, b"RewardedCount")
    }

    pub fn identity_claimed(hash: &[u8; 32]) -> Vec<u8> {
        blake2_map_raw(CITIZEN_ISSUANCE_PALLET, b"IdentityRewardClaimed", hash)
    }

    pub fn account_rewarded(account: &[u8; 32]) -> Vec<u8> {
        blake2_map_raw(CITIZEN_ISSUANCE_PALLET, b"AccountRewarded", account)
    }

    pub fn pending_count() -> Vec<u8> {
        storage_prefix(CITIZEN_ISSUANCE_PALLET, b"PendingRewardCount")
    }

    pub fn storage_version() -> Vec<u8> {
        let mut key = pallet_prefix().to_vec();
        key.extend_from_slice(&twox_128(b":__STORAGE_VERSION__:"));
        key
    }

    pub fn pending_reward(index: u32) -> Vec<u8> {
        crate::shared::storage_keys::twox64_map(
            CITIZEN_ISSUANCE_PALLET,
            b"PendingRewards",
            &index.encode(),
        )
    }

    #[cfg(test)]
    pub fn pending_reward_prefix() -> Vec<u8> {
        storage_prefix(CITIZEN_ISSUANCE_PALLET, b"PendingRewards")
    }

    pub fn pending_identity(hash: &[u8; 32]) -> Vec<u8> {
        blake2_map_raw(
            CITIZEN_ISSUANCE_PALLET,
            b"PendingIdentityRewardClaimed",
            hash,
        )
    }

    pub fn pending_account(account: &[u8; 32]) -> Vec<u8> {
        blake2_map_raw(CITIZEN_ISSUANCE_PALLET, b"PendingAccountRewarded", account)
    }

    pub fn voting_identity(account: &[u8; 32]) -> Vec<u8> {
        blake2_map_raw(CITIZEN_IDENTITY_PALLET, b"VotingIdentityByAccount", account)
    }

    pub fn account_by_cid(cid: &[u8]) -> Vec<u8> {
        blake2_map_raw(CITIZEN_IDENTITY_PALLET, b"AccountByCid", &cid.encode())
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

fn decode_or_zero<T: Decode + Default>(
    raw: Option<Vec<u8>>,
    label: &'static str,
) -> Result<T, GuardError> {
    match raw {
        Some(raw) => decode_exact(&raw, label),
        None => Ok(T::default()),
    }
}

fn unit_present<F>(read: &F, key: &[u8], label: &'static str) -> Result<bool, GuardError>
where
    F: Fn(&[u8]) -> Option<Vec<u8>>,
{
    let Some(raw) = read(key) else {
        return Ok(false);
    };
    let _: () = decode_exact(&raw, label)?;
    Ok(true)
}

#[cfg(test)]
fn parse_twox_u32_key(key: &[u8], prefix: &[u8]) -> Result<u32, GuardError> {
    if !key.starts_with(prefix) || key.len() != prefix.len() + 12 {
        return Err(GuardError::StorageKeyMalformed("PendingRewards"));
    }
    let hash_at = prefix.len();
    let encoded_at = hash_at + 8;
    let encoded = &key[encoded_at..];
    if twox_64(encoded) != key[hash_at..encoded_at] {
        return Err(GuardError::StorageKeyMalformed("PendingRewards"));
    }
    decode_exact(encoded, "PendingRewards key")
}

fn reward_amount_at(index: u64) -> u128 {
    if index < CITIZEN_ISSUANCE_HIGH_REWARD_COUNT {
        CITIZEN_ISSUANCE_HIGH_REWARD
    } else {
        CITIZEN_ISSUANCE_NORMAL_REWARD
    }
}

/// 校验普通区块的待发队列、身份依据、永久防重状态，并登记精确 finalize 发行计划。
pub fn check_transition<FParent, FPre, FPost>(
    extrinsic_count: usize,
    pre_delta: &BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    post_delta: &BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    parent: &FParent,
    pre: &FPre,
    post: &FPost,
    issuance_plan: &mut FinalizeIssuancePlan,
) -> Result<(), GuardError>
where
    FParent: Fn(&[u8]) -> Option<Vec<u8>>,
    FPre: Fn(&[u8]) -> Option<Vec<u8>>,
    FPost: Fn(&[u8]) -> Option<Vec<u8>>,
{
    let pending_count_key = storage_key::pending_count();
    if parent(&pending_count_key).is_some() {
        return Err(GuardError::PendingCountChangedOutsideExtrinsic);
    }
    let pending_count: u32 = decode_or_zero(pre(&pending_count_key), "PendingRewardCount")?;
    if usize::try_from(pending_count).unwrap_or(usize::MAX) > extrinsic_count {
        return Err(GuardError::PendingCountExceedsExtrinsics);
    }
    if post(&pending_count_key).is_some() {
        return Err(GuardError::PendingQueueNotCleared);
    }

    let parent_count: u64 =
        decode_or_zero(parent(&storage_key::rewarded_count()), "RewardedCount")?;
    let pre_count: u64 = decode_or_zero(pre(&storage_key::rewarded_count()), "RewardedCount")?;
    if pre_count != parent_count {
        return Err(GuardError::RewardedCountChangedBeforeFinalize);
    }
    let expected_after = parent_count
        .checked_add(u64::from(pending_count))
        .ok_or(GuardError::RewardLimitExceeded)?;
    if expected_after > CITIZEN_ISSUANCE_MAX_COUNT {
        return Err(GuardError::RewardLimitExceeded);
    }
    let post_count: u64 = decode_or_zero(post(&storage_key::rewarded_count()), "RewardedCount")?;
    if post_count != expected_after {
        return Err(GuardError::RewardedCountInvalid);
    }

    let mut expected_pre_keys = BTreeSet::from([pending_count_key.clone()]);
    // Overlay 可能把“本块插入后删除”保留为显式 deletion delta；只要后状态确实为空即可接受。
    let mut expected_post_keys = BTreeSet::from([pending_count_key]);
    if pending_count > 0 || post_delta.contains_key(&storage_key::rewarded_count()) {
        expected_post_keys.insert(storage_key::rewarded_count());
    }
    let mut accounts = BTreeSet::new();
    let mut identities = BTreeSet::new();
    for index in 0..pending_count {
        let pending_key = storage_key::pending_reward(index);
        let pending: PendingCertificationReward = pre(&pending_key)
            .ok_or(GuardError::PendingQueueGap)
            .and_then(|raw| decode_exact(&raw, "PendingRewards"))?;
        if !accounts.insert(pending.who) || !identities.insert(pending.cid_number_hash) {
            return Err(GuardError::PendingDuplicate);
        }

        let pending_identity_key = storage_key::pending_identity(&pending.cid_number_hash);
        let pending_account_key = storage_key::pending_account(&pending.who);
        if !unit_present(pre, &pending_identity_key, "PendingIdentityRewardClaimed")?
            || !unit_present(pre, &pending_account_key, "PendingAccountRewarded")?
        {
            return Err(GuardError::PendingMarkerMissing);
        }
        if post(&pending_key).is_some()
            || post(&pending_identity_key).is_some()
            || post(&pending_account_key).is_some()
        {
            return Err(GuardError::PendingQueueNotCleared);
        }

        let voting_key = storage_key::voting_identity(&pending.who);
        if parent(&voting_key).is_some() {
            return Err(GuardError::IdentityAlreadyExisted);
        }
        let identity: VotingIdentity = pre(&voting_key)
            .ok_or(GuardError::FirstIdentityMissing)
            .and_then(|raw| decode_exact(&raw, "VotingIdentityByAccount"))?;
        if blake2_256(&identity.cid_number) != pending.cid_number_hash {
            return Err(GuardError::IdentityHashMismatch);
        }
        let reverse: [u8; 32] = pre(&storage_key::account_by_cid(&identity.cid_number))
            .ok_or(GuardError::IdentityReverseIndexMismatch)
            .and_then(|raw| decode_exact(&raw, "AccountByCid"))?;
        if reverse != pending.who {
            return Err(GuardError::IdentityReverseIndexMismatch);
        }

        let claimed_key = storage_key::identity_claimed(&pending.cid_number_hash);
        let rewarded_key = storage_key::account_rewarded(&pending.who);
        if unit_present(parent, &claimed_key, "IdentityRewardClaimed")?
            || unit_present(parent, &rewarded_key, "AccountRewarded")?
        {
            return Err(GuardError::RewardAlreadyClaimed);
        }
        if unit_present(pre, &claimed_key, "IdentityRewardClaimed")?
            || unit_present(pre, &rewarded_key, "AccountRewarded")?
        {
            return Err(GuardError::PermanentMarkerChangedBeforeFinalize);
        }
        if !unit_present(post, &claimed_key, "IdentityRewardClaimed")?
            || !unit_present(post, &rewarded_key, "AccountRewarded")?
        {
            return Err(GuardError::PermanentMarkerMissingAfterFinalize);
        }

        expected_pre_keys.insert(pending_key);
        expected_pre_keys.insert(pending_identity_key.clone());
        expected_pre_keys.insert(pending_account_key.clone());
        expected_post_keys.insert(storage_key::pending_reward(index));
        expected_post_keys.insert(pending_identity_key);
        expected_post_keys.insert(pending_account_key);
        expected_post_keys.insert(claimed_key);
        expected_post_keys.insert(rewarded_key);

        let reward = reward_amount_at(parent_count + u64::from(index));
        issuance_plan
            .add(pending.who, reward)
            .map_err(|_| GuardError::FinalizeIssuanceOverflow)?;
    }

    let pallet_prefix = storage_key::pallet_prefix();
    for key in pre_delta
        .keys()
        .filter(|key| key.starts_with(&pallet_prefix))
    {
        if !expected_pre_keys.contains(key) {
            return Err(GuardError::PendingQueueUnexpectedKey);
        }
    }
    for key in post_delta
        .keys()
        .filter(|key| key.starts_with(&pallet_prefix))
    {
        if !expected_post_keys.contains(key) {
            return Err(GuardError::PendingQueueUnexpectedKey);
        }
    }
    Ok(())
}

/// block#0 的公民认证发行、永久防重与临时队列必须处于规范空状态。
///
/// FRAME 创世构建器会写入当前 pallet 存储版本，也可能把 `ValueQuery` 计数显式编码为零；
/// 零与 key 不存在语义相同。因此只允许存储版本 0 和两个计数的精确 SCALE 零值，任何
/// 防重标记、队列项、非零计数或未知 key 均拒绝。
pub fn check_genesis_key_values<'a, I>(pairs: I) -> Result<(), GuardError>
where
    I: IntoIterator<Item = (&'a Vec<u8>, &'a Vec<u8>)>,
{
    let rewarded_count = storage_key::rewarded_count();
    let pending_count = storage_key::pending_count();
    let storage_version = storage_key::storage_version();
    for (key, value) in pairs {
        let canonical_empty = if key == &rewarded_count {
            decode_exact::<u64>(value, "RewardedCount")? == 0
        } else if key == &pending_count {
            decode_exact::<u32>(value, "PendingRewardCount")? == 0
        } else if key == &storage_version {
            decode_exact::<u16>(value, "StorageVersion")? == 0
        } else {
            false
        };
        if canonical_empty {
            continue;
        }
        return Err(GuardError::GenesisStateNotEmpty(key.clone()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put<T: Encode>(map: &mut BTreeMap<Vec<u8>, Vec<u8>>, key: Vec<u8>, value: T) {
        map.insert(key, value.encode());
    }

    fn voting_identity(cid: &[u8]) -> Vec<u8> {
        (
            cid.to_vec(),
            20260101u32,
            20360101u32,
            CitizenStatus::Normal,
            b"GD".to_vec(),
            b"001".to_vec(),
            Vec::<u8>::new(),
            1u32,
        )
            .encode()
    }

    fn valid_transition() -> (
        BTreeMap<Vec<u8>, Vec<u8>>,
        BTreeMap<Vec<u8>, Vec<u8>>,
        BTreeMap<Vec<u8>, Vec<u8>>,
        BTreeMap<Vec<u8>, Option<Vec<u8>>>,
        BTreeMap<Vec<u8>, Option<Vec<u8>>>,
        [u8; 32],
    ) {
        let parent = BTreeMap::new();
        let mut pre = BTreeMap::new();
        let mut post = BTreeMap::new();
        let account = [7u8; 32];
        let cid = b"GD001-CTZN1-TEST".to_vec();
        let hash = blake2_256(&cid);
        put(&mut pre, storage_key::pending_count(), 1u32);
        put(
            &mut pre,
            storage_key::pending_reward(0),
            PendingCertificationReward {
                who: account,
                cid_number_hash: hash,
            },
        );
        put(&mut pre, storage_key::pending_identity(&hash), ());
        put(&mut pre, storage_key::pending_account(&account), ());
        pre.insert(
            storage_key::voting_identity(&account),
            voting_identity(&cid),
        );
        put(&mut pre, storage_key::account_by_cid(&cid), account);
        put(&mut post, storage_key::rewarded_count(), 1u64);
        put(&mut post, storage_key::identity_claimed(&hash), ());
        put(&mut post, storage_key::account_rewarded(&account), ());
        post.insert(
            storage_key::voting_identity(&account),
            voting_identity(&cid),
        );
        put(&mut post, storage_key::account_by_cid(&cid), account);
        let pre_delta = pre
            .iter()
            .map(|(key, value)| (key.clone(), Some(value.clone())))
            .collect();
        let post_delta = post
            .iter()
            .filter(|(key, _)| key.starts_with(&storage_key::pallet_prefix()))
            .map(|(key, value)| (key.clone(), Some(value.clone())))
            .collect();
        (parent, pre, post, pre_delta, post_delta, account)
    }

    #[test]
    fn valid_pending_reward_registers_exact_finalize_plan() {
        let (parent, pre, post, pre_delta, post_delta, account) = valid_transition();
        let mut plan = FinalizeIssuancePlan::default();
        assert_eq!(
            check_transition(
                2,
                &pre_delta,
                &post_delta,
                &|key| parent.get(key).cloned(),
                &|key| pre.get(key).cloned(),
                &|key| post.get(key).cloned(),
                &mut plan,
            ),
            Ok(())
        );
        assert_eq!(
            plan.accounts.get(&account),
            Some(&CITIZEN_ISSUANCE_HIGH_REWARD)
        );
        assert_eq!(plan.total, CITIZEN_ISSUANCE_HIGH_REWARD);
    }

    #[test]
    fn queue_gap_wrong_identity_and_residual_pending_are_rejected() {
        let (parent, mut pre, post, pre_delta, post_delta, _) = valid_transition();
        pre.remove(&storage_key::pending_reward(0));
        assert_eq!(
            check_transition(
                2,
                &pre_delta,
                &post_delta,
                &|key| parent.get(key).cloned(),
                &|key| pre.get(key).cloned(),
                &|key| post.get(key).cloned(),
                &mut FinalizeIssuancePlan::default(),
            ),
            Err(GuardError::PendingQueueGap)
        );

        let (parent, pre, mut post, pre_delta, post_delta, _) = valid_transition();
        put(&mut post, storage_key::pending_count(), 1u32);
        assert_eq!(
            check_transition(
                2,
                &pre_delta,
                &post_delta,
                &|key| parent.get(key).cloned(),
                &|key| pre.get(key).cloned(),
                &|key| post.get(key).cloned(),
                &mut FinalizeIssuancePlan::default(),
            ),
            Err(GuardError::PendingQueueNotCleared)
        );
    }

    #[test]
    fn genesis_must_have_no_claim_or_pending_state() {
        let empty = BTreeMap::<Vec<u8>, Vec<u8>>::new();
        assert_eq!(check_genesis_key_values(empty.iter()), Ok(()));
        let mut canonical_zero = BTreeMap::new();
        put(&mut canonical_zero, storage_key::rewarded_count(), 0u64);
        put(&mut canonical_zero, storage_key::pending_count(), 0u32);
        put(&mut canonical_zero, storage_key::storage_version(), 0u16);
        assert_eq!(check_genesis_key_values(canonical_zero.iter()), Ok(()));
        let mut bad = BTreeMap::new();
        put(&mut bad, storage_key::pending_count(), 1u32);
        assert_eq!(
            check_genesis_key_values(bad.iter()),
            Err(GuardError::GenesisStateNotEmpty(
                storage_key::pending_count()
            ))
        );
    }

    #[test]
    fn twox_pending_key_parser_rejects_tampering() {
        let prefix = storage_key::pending_reward_prefix();
        let key = storage_key::pending_reward(9);
        assert_eq!(parse_twox_u32_key(&key, &prefix), Ok(9));
        let mut bad = key;
        bad[prefix.len()] ^= 1;
        assert_eq!(
            parse_twox_u32_key(&bad, &prefix),
            Err(GuardError::StorageKeyMalformed("PendingRewards"))
        );
    }

    #[test]
    fn malicious_identity_claim_and_unknown_delta_are_rejected() {
        let (mut parent, pre, post, pre_delta, post_delta, account) = valid_transition();
        parent.insert(
            storage_key::voting_identity(&account),
            voting_identity(b"GD001-CTZN1-OLD"),
        );
        assert_eq!(
            check_transition(
                2,
                &pre_delta,
                &post_delta,
                &|key| parent.get(key).cloned(),
                &|key| pre.get(key).cloned(),
                &|key| post.get(key).cloned(),
                &mut FinalizeIssuancePlan::default(),
            ),
            Err(GuardError::IdentityAlreadyExisted)
        );

        let (parent, mut pre, post, pre_delta, post_delta, account) = valid_transition();
        pre.insert(
            storage_key::voting_identity(&account),
            voting_identity(b"GD001-CTZN1-TAMPERED"),
        );
        assert_eq!(
            check_transition(
                2,
                &pre_delta,
                &post_delta,
                &|key| parent.get(key).cloned(),
                &|key| pre.get(key).cloned(),
                &|key| post.get(key).cloned(),
                &mut FinalizeIssuancePlan::default(),
            ),
            Err(GuardError::IdentityHashMismatch)
        );

        let (parent, pre, post, mut pre_delta, post_delta, _) = valid_transition();
        let mut unknown = storage_key::pallet_prefix().to_vec();
        unknown.extend_from_slice(b"shadow-audit");
        pre_delta.insert(unknown, Some(vec![1]));
        assert_eq!(
            check_transition(
                2,
                &pre_delta,
                &post_delta,
                &|key| parent.get(key).cloned(),
                &|key| pre.get(key).cloned(),
                &|key| post.get(key).cloned(),
                &mut FinalizeIssuancePlan::default(),
            ),
            Err(GuardError::PendingQueueUnexpectedKey)
        );
    }

    #[test]
    fn genesis_rejects_trailing_zero_and_unknown_keys() {
        let mut trailing = BTreeMap::new();
        let mut zero = 0u64.encode();
        zero.push(0xff);
        trailing.insert(storage_key::rewarded_count(), zero);
        assert_eq!(
            check_genesis_key_values(trailing.iter()),
            Err(GuardError::StorageValueDecodeFailed("RewardedCount"))
        );

        let mut unknown = BTreeMap::new();
        let mut key = storage_key::pallet_prefix().to_vec();
        key.extend_from_slice(b"unknown");
        unknown.insert(key.clone(), Vec::new());
        assert_eq!(
            check_genesis_key_values(unknown.iter()),
            Err(GuardError::GenesisStateNotEmpty(key))
        );
    }
}
