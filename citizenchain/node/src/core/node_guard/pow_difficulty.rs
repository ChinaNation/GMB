//! PoW 动态难度与版本化参数的节点独立守卫。
//!
//! runtime 负责执行，NodeGuard 直接读取 RAW storage 并复算；可治理参数不在节点重复写死，
//! 但参数版本、升级原子性、公式版本和每块状态转换由节点永久 fail-closed。

use std::collections::BTreeMap;

use codec::{Decode, Encode};
use pow_difficulty::{DifficultyAdjustmentAudit, PendingPowDifficultyParams, PowDifficultyParams};
use sp_core::hashing::{blake2_256, twox_128};

const SUPPORTED_ALGORITHM_VERSION: u16 = 1;

#[derive(Debug, PartialEq, Eq)]
pub enum GuardError {
    Missing(&'static str),
    Malformed(&'static str),
    InvalidParams,
    UnsupportedAlgorithm(u16),
    UnknownStorageKey(Vec<u8>),
    UnauthorizedParamsChange,
    PendingTransitionInvalid,
    DifficultyTransitionInvalid,
    WindowTransitionInvalid,
    AdjustmentAuditInvalid,
    RuntimeUpgradeAuditInvalid,
}

#[derive(Debug, Decode, PartialEq, Eq)]
enum UpgradeExecutionPath {
    JointVote,
    DeveloperDirect,
}

#[derive(Debug, Decode, PartialEq, Eq)]
struct RuntimeUpgradeAudit {
    proposal_id: Option<u64>,
    execution_path: UpgradeExecutionPath,
    code_hash: [u8; 32],
    old_pow_params_hash: [u8; 32],
    new_pow_params_hash: [u8; 32],
    executed_at: u32,
    activate_at: u32,
    developer: Option<[u8; 32]>,
}

pub mod storage_key {
    use super::*;

    fn value(pallet: &[u8], item: &[u8]) -> Vec<u8> {
        [twox_128(pallet).as_slice(), twox_128(item).as_slice()].concat()
    }

    pub fn pallet_prefix() -> [u8; 16] {
        twox_128(b"PowDifficulty")
    }

    pub fn current_difficulty() -> Vec<u8> {
        value(b"PowDifficulty", b"CurrentDifficulty")
    }

    pub fn window_start_ms() -> Vec<u8> {
        value(b"PowDifficulty", b"WindowStartMs")
    }

    pub fn window_start_block() -> Vec<u8> {
        value(b"PowDifficulty", b"WindowStartBlock")
    }

    pub fn active_params() -> Vec<u8> {
        value(b"PowDifficulty", b"ActiveParams")
    }

    pub fn pending_params() -> Vec<u8> {
        value(b"PowDifficulty", b"PendingParams")
    }

    pub fn last_adjustment() -> Vec<u8> {
        value(b"PowDifficulty", b"LastAdjustment")
    }

    pub fn storage_version() -> Vec<u8> {
        value(b"PowDifficulty", b":__STORAGE_VERSION__:")
    }

    pub fn timestamp_now() -> Vec<u8> {
        value(b"Timestamp", b"Now")
    }

    pub fn runtime_upgrade_audit() -> Vec<u8> {
        value(b"RuntimeUpgrade", b"LastRuntimeUpgradeAudit")
    }
}

fn decode_exact<T: Decode>(raw: Option<Vec<u8>>, label: &'static str) -> Result<T, GuardError> {
    let raw = raw.ok_or(GuardError::Missing(label))?;
    let mut input = raw.as_slice();
    let value = T::decode(&mut input).map_err(|_| GuardError::Malformed(label))?;
    if !input.is_empty() {
        return Err(GuardError::Malformed(label));
    }
    Ok(value)
}

fn decode_optional<T: Decode>(
    raw: Option<Vec<u8>>,
    label: &'static str,
) -> Result<Option<T>, GuardError> {
    raw.map(|raw| {
        let mut input = raw.as_slice();
        let value = T::decode(&mut input).map_err(|_| GuardError::Malformed(label))?;
        if !input.is_empty() {
            return Err(GuardError::Malformed(label));
        }
        Ok(value)
    })
    .transpose()
}

fn validate_params(params: &PowDifficultyParams) -> Result<(), GuardError> {
    params.validate().map_err(|_| GuardError::InvalidParams)?;
    if params.algorithm_version != SUPPORTED_ALGORITHM_VERSION {
        return Err(GuardError::UnsupportedAlgorithm(params.algorithm_version));
    }
    Ok(())
}

fn params_hash(params: &PowDifficultyParams) -> [u8; 32] {
    blake2_256(&params.encode())
}

fn same_values_except_version(a: &PowDifficultyParams, b: &PowDifficultyParams) -> bool {
    a.algorithm_version == b.algorithm_version
        && a.target_block_time_ms == b.target_block_time_ms
        && a.adjustment_interval == b.adjustment_interval
        && a.max_adjust_up_factor == b.max_adjust_up_factor
        && a.max_adjust_down_divisor == b.max_adjust_down_divisor
}

fn expected_difficulty(
    old: u64,
    params: &PowDifficultyParams,
    actual_window_ms: u64,
) -> Result<u64, GuardError> {
    if old == 0 {
        return Err(GuardError::DifficultyTransitionInvalid);
    }
    let target = params.target_window_ms().ok_or(GuardError::InvalidParams)?;
    let raw = (old as u128)
        .checked_mul(target as u128)
        .ok_or(GuardError::DifficultyTransitionInvalid)?
        / actual_window_ms.max(1) as u128;
    let max = old.saturating_mul(params.max_adjust_up_factor);
    let min = (old / params.max_adjust_down_divisor).max(1);
    Ok(u64::try_from(raw).unwrap_or(u64::MAX).clamp(min, max))
}

fn known_key(key: &[u8]) -> bool {
    [
        storage_key::current_difficulty(),
        storage_key::window_start_ms(),
        storage_key::window_start_block(),
        storage_key::active_params(),
        storage_key::pending_params(),
        storage_key::last_adjustment(),
        storage_key::storage_version(),
    ]
    .iter()
    .any(|known| known.as_slice() == key)
}

pub fn check_genesis<F>(read: F) -> Result<(), GuardError>
where
    F: Fn(&[u8]) -> Option<Vec<u8>>,
{
    let params: PowDifficultyParams =
        decode_exact(read(&storage_key::active_params()), "ActiveParams")?;
    validate_params(&params)?;
    let difficulty: u64 = decode_exact(
        read(&storage_key::current_difficulty()),
        "CurrentDifficulty",
    )?;
    if difficulty == 0 {
        return Err(GuardError::DifficultyTransitionInvalid);
    }
    let storage_version: u16 =
        decode_exact(read(&storage_key::storage_version()), "StorageVersion")?;
    if storage_version != 1 {
        return Err(GuardError::Malformed("StorageVersion"));
    }
    for (key, label) in [
        (storage_key::pending_params(), "PendingParams"),
        (storage_key::window_start_ms(), "WindowStartMs"),
        (storage_key::window_start_block(), "WindowStartBlock"),
        (storage_key::last_adjustment(), "LastAdjustment"),
    ] {
        if read(&key).is_some() {
            return Err(GuardError::Malformed(label));
        }
    }
    Ok(())
}

pub fn check_transition<FParent, FPost>(
    block: u32,
    post_delta: &BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    read_parent: FParent,
    read_post: FPost,
) -> Result<(), GuardError>
where
    FParent: Fn(&[u8]) -> Option<Vec<u8>>,
    FPost: Fn(&[u8]) -> Option<Vec<u8>>,
{
    let prefix = storage_key::pallet_prefix();
    for key in post_delta.keys().filter(|key| key.starts_with(&prefix)) {
        if !known_key(key) {
            return Err(GuardError::UnknownStorageKey(key.clone()));
        }
    }

    let parent_params: PowDifficultyParams =
        decode_exact(read_parent(&storage_key::active_params()), "ActiveParams")?;
    let post_params: PowDifficultyParams =
        decode_exact(read_post(&storage_key::active_params()), "ActiveParams")?;
    validate_params(&parent_params)?;
    validate_params(&post_params)?;

    let parent_pending: Option<PendingPowDifficultyParams> =
        decode_optional(read_parent(&storage_key::pending_params()), "PendingParams")?;
    let post_pending: Option<PendingPowDifficultyParams> =
        decode_optional(read_post(&storage_key::pending_params()), "PendingParams")?;
    let code_changed = post_delta.contains_key(sp_storage::well_known_keys::CODE);

    if code_changed {
        if parent_pending.is_some() || post_params != parent_params {
            return Err(GuardError::PendingTransitionInvalid);
        }
        match post_pending {
            Some(pending) => {
                validate_params(&pending.params)?;
                if pending.activate_at != block.saturating_add(1)
                    || pending.params.params_version
                        != parent_params.params_version.saturating_add(1)
                    || same_values_except_version(&pending.params, &parent_params)
                {
                    return Err(GuardError::PendingTransitionInvalid);
                }
            }
            None => {}
        }
        let audit: RuntimeUpgradeAudit = decode_exact(
            read_post(&storage_key::runtime_upgrade_audit()),
            "LastRuntimeUpgradeAudit",
        )?;
        let code = read_post(sp_storage::well_known_keys::CODE)
            .ok_or(GuardError::RuntimeUpgradeAuditInvalid)?;
        if audit.code_hash != blake2_256(&code)
            || audit.old_pow_params_hash != params_hash(&parent_params)
            || audit.new_pow_params_hash
                != params_hash(&post_pending.map(|p| p.params).unwrap_or(post_params))
            || audit.executed_at != block
            || audit.activate_at != block.saturating_add(1)
        {
            return Err(GuardError::RuntimeUpgradeAuditInvalid);
        }
    }

    if parent_pending != post_pending
        && !code_changed
        && parent_pending.map(|p| p.activate_at) != Some(block)
    {
        return Err(GuardError::UnauthorizedParamsChange);
    }

    let parent_difficulty: u64 = decode_exact(
        read_parent(&storage_key::current_difficulty()),
        "CurrentDifficulty",
    )?;
    let post_difficulty: u64 = decode_exact(
        read_post(&storage_key::current_difficulty()),
        "CurrentDifficulty",
    )?;
    if parent_difficulty == 0 || post_difficulty == 0 {
        return Err(GuardError::DifficultyTransitionInvalid);
    }

    let post_now: u64 = decode_exact(read_post(&storage_key::timestamp_now()), "Timestamp::Now")?;
    let parent_start_ms: Option<u64> = decode_optional(
        read_parent(&storage_key::window_start_ms()),
        "WindowStartMs",
    )?;
    let post_start_ms: Option<u64> =
        decode_optional(read_post(&storage_key::window_start_ms()), "WindowStartMs")?;
    let parent_start_block: Option<u32> = decode_optional(
        read_parent(&storage_key::window_start_block()),
        "WindowStartBlock",
    )?;
    let post_start_block: Option<u32> = decode_optional(
        read_post(&storage_key::window_start_block()),
        "WindowStartBlock",
    )?;
    let parent_audit: Option<DifficultyAdjustmentAudit> = decode_optional(
        read_parent(&storage_key::last_adjustment()),
        "LastAdjustment",
    )?;
    let post_audit: Option<DifficultyAdjustmentAudit> =
        decode_optional(read_post(&storage_key::last_adjustment()), "LastAdjustment")?;

    if parent_pending.map(|p| p.activate_at) == Some(block) {
        let activated = parent_pending.ok_or(GuardError::PendingTransitionInvalid)?;
        if post_params != activated.params
            || post_pending.is_some()
            || post_difficulty != parent_difficulty
            || post_start_block != Some(block)
            || post_start_ms != Some(post_now)
            || post_audit != parent_audit
        {
            return Err(GuardError::PendingTransitionInvalid);
        }
        return Ok(());
    }

    if post_params != parent_params {
        return Err(GuardError::UnauthorizedParamsChange);
    }

    match (parent_start_block, parent_start_ms) {
        (None, None) => {
            if post_start_block != Some(block)
                || post_start_ms != Some(post_now)
                || post_difficulty != parent_difficulty
                || post_audit != parent_audit
            {
                return Err(GuardError::WindowTransitionInvalid);
            }
        }
        (Some(start_block), Some(start_ms)) => {
            let elapsed = block
                .checked_sub(start_block)
                .ok_or(GuardError::WindowTransitionInvalid)?;
            if elapsed < parent_params.adjustment_interval {
                if post_start_block != Some(start_block)
                    || post_start_ms != Some(start_ms)
                    || post_difficulty != parent_difficulty
                    || post_audit != parent_audit
                {
                    return Err(GuardError::DifficultyTransitionInvalid);
                }
            } else if elapsed == parent_params.adjustment_interval {
                let actual = post_now.saturating_sub(start_ms).max(1);
                let expected = expected_difficulty(parent_difficulty, &parent_params, actual)?;
                let expected_audit = DifficultyAdjustmentAudit {
                    block,
                    params_version: parent_params.params_version,
                    old_difficulty: parent_difficulty,
                    new_difficulty: expected,
                    window_start_block: start_block,
                    actual_window_ms: actual,
                };
                if post_difficulty != expected
                    || post_start_block != Some(block)
                    || post_start_ms != Some(post_now)
                    || post_audit != Some(expected_audit)
                {
                    return Err(GuardError::AdjustmentAuditInvalid);
                }
            } else {
                return Err(GuardError::WindowTransitionInvalid);
            }
        }
        _ => return Err(GuardError::WindowTransitionInvalid),
    }
    Ok(())
}

pub fn check_imported_genesis<'a, I>(entries: I) -> Result<(), GuardError>
where
    I: IntoIterator<Item = (&'a Vec<u8>, &'a Vec<u8>)>,
{
    let state: BTreeMap<Vec<u8>, Vec<u8>> = entries
        .into_iter()
        .filter(|(key, _)| key.starts_with(&storage_key::pallet_prefix()))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    for key in state.keys() {
        if !known_key(key) {
            return Err(GuardError::UnknownStorageKey(key.clone()));
        }
    }
    check_genesis(|key| state.get(key).cloned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put<T: Encode>(state: &mut BTreeMap<Vec<u8>, Vec<u8>>, key: Vec<u8>, value: T) {
        state.insert(key, value.encode());
    }

    fn params_for_test() -> PowDifficultyParams {
        let mut params = PowDifficultyParams::genesis_default();
        params.target_block_time_ms = 1_000;
        params.adjustment_interval = 10;
        params
    }

    #[test]
    fn dynamic_formula_uses_active_params() {
        let params = params_for_test();
        assert_eq!(expected_difficulty(100, &params, 5_000), Ok(200));
        assert_eq!(expected_difficulty(100, &params, 40_000), Ok(25));
    }

    #[test]
    fn unknown_algorithm_fails_closed() {
        let mut params = PowDifficultyParams::genesis_default();
        params.algorithm_version = 2;
        assert_eq!(
            validate_params(&params),
            Err(GuardError::UnsupportedAlgorithm(2))
        );
    }

    #[test]
    fn adjustment_transition_is_independently_recomputed() {
        let params = params_for_test();
        let mut parent = BTreeMap::new();
        put(&mut parent, storage_key::active_params(), params);
        put(&mut parent, storage_key::current_difficulty(), 100u64);
        put(&mut parent, storage_key::window_start_block(), 1u32);
        put(&mut parent, storage_key::window_start_ms(), 1_000u64);

        let mut post = parent.clone();
        put(&mut post, storage_key::timestamp_now(), 6_000u64);
        put(&mut post, storage_key::current_difficulty(), 200u64);
        put(&mut post, storage_key::window_start_block(), 11u32);
        put(&mut post, storage_key::window_start_ms(), 6_000u64);
        put(
            &mut post,
            storage_key::last_adjustment(),
            DifficultyAdjustmentAudit {
                block: 11,
                params_version: params.params_version,
                old_difficulty: 100,
                new_difficulty: 200,
                window_start_block: 1,
                actual_window_ms: 5_000,
            },
        );
        let delta = post
            .iter()
            .filter(|(key, value)| parent.get(*key) != Some(*value))
            .map(|(key, value)| (key.clone(), Some(value.clone())))
            .collect();

        assert_eq!(
            check_transition(
                11,
                &delta,
                |key| parent.get(key).cloned(),
                |key| post.get(key).cloned(),
            ),
            Ok(())
        );
    }

    #[test]
    fn params_cannot_change_without_runtime_upgrade() {
        let params = params_for_test();
        let mut next = params;
        next.params_version += 1;
        next.adjustment_interval = 20;
        let mut parent = BTreeMap::new();
        put(&mut parent, storage_key::active_params(), params);
        put(&mut parent, storage_key::current_difficulty(), 100u64);
        put(&mut parent, storage_key::window_start_block(), 1u32);
        put(&mut parent, storage_key::window_start_ms(), 1_000u64);
        let mut post = parent.clone();
        put(&mut post, storage_key::timestamp_now(), 2_000u64);
        put(&mut post, storage_key::active_params(), next);
        let delta = BTreeMap::from([(storage_key::active_params(), Some(next.encode()))]);

        assert_eq!(
            check_transition(
                2,
                &delta,
                |key| parent.get(key).cloned(),
                |key| post.get(key).cloned(),
            ),
            Err(GuardError::UnauthorizedParamsChange)
        );
    }

    #[test]
    fn activation_resets_window_but_preserves_difficulty() {
        let params = params_for_test();
        let mut next = params;
        next.params_version += 1;
        next.adjustment_interval = 20;
        let pending = PendingPowDifficultyParams {
            params: next,
            activate_at: 2,
        };
        let mut parent = BTreeMap::new();
        put(&mut parent, storage_key::active_params(), params);
        put(&mut parent, storage_key::pending_params(), pending);
        put(&mut parent, storage_key::current_difficulty(), 100u64);
        put(&mut parent, storage_key::window_start_block(), 1u32);
        put(&mut parent, storage_key::window_start_ms(), 1_000u64);
        let mut post = parent.clone();
        post.remove(&storage_key::pending_params());
        put(&mut post, storage_key::active_params(), next);
        put(&mut post, storage_key::timestamp_now(), 2_000u64);
        put(&mut post, storage_key::window_start_block(), 2u32);
        put(&mut post, storage_key::window_start_ms(), 2_000u64);
        let delta = BTreeMap::from([
            (storage_key::active_params(), Some(next.encode())),
            (storage_key::pending_params(), None),
            (storage_key::window_start_block(), Some(2u32.encode())),
            (storage_key::window_start_ms(), Some(2_000u64.encode())),
        ]);

        assert_eq!(
            check_transition(
                2,
                &delta,
                |key| parent.get(key).cloned(),
                |key| post.get(key).cloned(),
            ),
            Ok(())
        );
    }
}
