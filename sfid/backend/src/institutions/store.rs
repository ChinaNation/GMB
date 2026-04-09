//! 机构/账户 store 读写层
//!
//! 中文注释:数据存在 `Store` 里(内存 + runtime_cache_entries 持久化),
//! 两个 HashMap:
//!
//! - `Store::multisig_institutions: HashMap<sfid_id, MultisigInstitution>`
//! - `Store::multisig_accounts: HashMap<"sfid_id|account_name", MultisigAccount>`
//!
//! 持久化通过 `runtime_cache_entries` 表的 payload JSON(见 main.rs 的
//! load_store_postgres / persist_store_postgres)。本模块**只**操作 in-memory Store。

#![allow(dead_code)]

use std::collections::HashMap;

use crate::institutions::model::{
    account_key_to_string, MultisigAccount, MultisigInstitution,
};
use crate::models::Store;

// ─── 机构 ────────────────────────────────────────────────────────

pub fn get_institution<'a>(store: &'a Store, sfid_id: &str) -> Option<&'a MultisigInstitution> {
    store.multisig_institutions.get(sfid_id)
}

pub fn insert_institution(store: &mut Store, inst: MultisigInstitution) {
    store
        .multisig_institutions
        .insert(inst.sfid_id.clone(), inst);
}

pub fn contains_institution(store: &Store, sfid_id: &str) -> bool {
    store.multisig_institutions.contains_key(sfid_id)
}

pub fn remove_institution(store: &mut Store, sfid_id: &str) -> Option<MultisigInstitution> {
    store.multisig_institutions.remove(sfid_id)
}

pub fn all_institutions(store: &Store) -> Vec<MultisigInstitution> {
    store.multisig_institutions.values().cloned().collect()
}

// ─── 账户 ────────────────────────────────────────────────────────

pub fn get_account<'a>(
    store: &'a Store,
    sfid_id: &str,
    account_name: &str,
) -> Option<&'a MultisigAccount> {
    let key = account_key_to_string(sfid_id, account_name);
    store.multisig_accounts.get(&key)
}

pub fn insert_account(store: &mut Store, account: MultisigAccount) {
    let key = account_key_to_string(&account.sfid_id, &account.account_name);
    store.multisig_accounts.insert(key, account);
}

pub fn update_account_chain<F>(
    store: &mut Store,
    sfid_id: &str,
    account_name: &str,
    f: F,
) -> bool
where
    F: FnOnce(&mut MultisigAccount),
{
    let key = account_key_to_string(sfid_id, account_name);
    if let Some(acc) = store.multisig_accounts.get_mut(&key) {
        f(acc);
        true
    } else {
        false
    }
}

pub fn remove_account(
    store: &mut Store,
    sfid_id: &str,
    account_name: &str,
) -> Option<MultisigAccount> {
    let key = account_key_to_string(sfid_id, account_name);
    store.multisig_accounts.remove(&key)
}

pub fn contains_account(store: &Store, sfid_id: &str, account_name: &str) -> bool {
    let key = account_key_to_string(sfid_id, account_name);
    store.multisig_accounts.contains_key(&key)
}

/// 列出机构下所有账户。
pub fn accounts_of_institution(store: &Store, sfid_id: &str) -> Vec<MultisigAccount> {
    store
        .multisig_accounts
        .values()
        .filter(|a| a.sfid_id == sfid_id)
        .cloned()
        .collect()
}

/// 统计机构下账户数量(用于 list 返回的 account_count 字段)。
pub fn count_accounts_of_institution(store: &Store, sfid_id: &str) -> usize {
    store
        .multisig_accounts
        .values()
        .filter(|a| a.sfid_id == sfid_id)
        .count()
}

/// 返回所有账户(用于迁移等场景,慎用)。
pub fn all_accounts(store: &Store) -> Vec<MultisigAccount> {
    store.multisig_accounts.values().cloned().collect()
}

/// 按 HashMap<String, _> 构造 accounts 侧引用(给 filter_map_by_scope 用,
/// 但由于 account 没有 province/city 字段,通常通过 institution 先过滤)。
pub fn accounts_map(store: &Store) -> &HashMap<String, MultisigAccount> {
    &store.multisig_accounts
}
