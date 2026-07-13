// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use alloc::vec;
use alloc::vec::Vec;
use sp_genesis_builder::PresetId;

#[cfg(feature = "std")]
use crate::AccountId;
#[cfg(feature = "std")]
use codec::Decode;
#[cfg(feature = "std")]
use primitives::{
    cid::china::china_cb::{CHINA_CB, NRC_HE_ACCOUNT},
    cid::china::china_ch::CHINA_CH,
    core_const::SS58_FORMAT,
    genesis::{CITIZENS, COUNTRY, GENESIS_CITIZEN_MAX, GENESIS_ISSUANCE, HE_FUND_ISSUANCE},
};
#[cfg(feature = "std")]
use serde_json::{json, Value};
#[cfg(feature = "std")]
use sp_core::crypto::{Ss58AddressFormat, Ss58Codec};
#[cfg(feature = "std")]
use sp_core::ByteArray;
#[cfg(feature = "std")]
use sp_genesis_builder::{self};

#[cfg(feature = "std")]
fn account_to_genesis_ss58(account: &AccountId) -> String {
    // 创世配置地址使用链统一 SS58 前缀（2027）。
    account.to_ss58check_with_version(Ss58AddressFormat::custom(SS58_FORMAT))
}

#[cfg(feature = "std")]
fn grandpa_key_to_genesis_ss58(key: &[u8; 32]) -> String {
    let authority = sp_consensus_grandpa::AuthorityId::from_slice(key)
        .expect("grandpa authority id must decode from 32 bytes");
    authority.to_ss58check_with_version(Ss58AddressFormat::custom(SS58_FORMAT))
}

#[cfg(all(feature = "std", test))]
fn json_amount_to_u128(v: &Value) -> Option<u128> {
    if let Some(value) = v.as_u64() {
        return Some(value as u128);
    }
    v.as_str().and_then(|s| s.parse::<u128>().ok())
}

// Returns the genesis config presets.
#[cfg(feature = "std")]
fn build_genesis() -> Value {
    // 国家储委会信息统一从常量数组入口读取。
    let nrc_account = CHINA_CB
        .first()
        .and_then(|n| AccountId::decode(&mut &n.main_account[..]).ok())
        .expect("NRC main_account must decode to AccountId");

    // 每位国家储委会管理员创世预置 1000 万元（单位：分）。
    let admin_each: u128 = 1_000_000_000; // 1000万元 = 10亿分
    let nrc_admins = &CHINA_CB
        .first()
        .expect("CHINA_CB must have NRC entry")
        .admins;
    let admin_total: u128 = admin_each * nrc_admins.len() as u128;

    // 国家储委会多签账户 = 创世发行总量 - 管理员预置总额，总量不变。
    let mut genesis_balances: Vec<(AccountId, u128)> =
        vec![(nrc_account.clone(), GENESIS_ISSUANCE - admin_total)];

    // 19 位管理员各自获得创世余额。
    genesis_balances.extend(nrc_admins.iter().map(|key| {
        let account = AccountId::new(*key);
        (account, admin_each)
    }));

    // 省储行创立发行在创世时直接预置到各自 stake_account（无私钥永久质押地址）。
    genesis_balances.extend(
        CHINA_CH
            .iter()
            .map(|bank| (AccountId::new(bank.stake_account), bank.stake_amount)),
    );

    // 两和基金创世一次性发行到国家储委会两和基金账户（无私钥派生地址 NRC_HE_ACCOUNT），
    // 作为独立增发计入总供应量，国家储委会通过内部投票管理该基金。
    genesis_balances.push((AccountId::new(NRC_HE_ACCOUNT), HE_FUND_ISSUANCE));

    // 创世账户统一输出为链 SS58 地址（前缀 2027）。
    let balances_json: Vec<Value> = genesis_balances
        .into_iter()
        .map(|(account, amount)| {
            let account_ss58 = account_to_genesis_ss58(&account);
            json!([account_ss58, amount])
        })
        .collect();

    // 决议发行合法收款账户改为链上存储初始化，后续可由治理动态更新。
    let issuance_allowed_recipients_json: Vec<Value> = CHINA_CB
        .iter()
        .skip(1)
        .map(|n| {
            let account = AccountId::decode(&mut &n.main_account[..])
                .expect("PRC main_account must decode to AccountId");
            Value::String(account_to_genesis_ss58(&account))
        })
        .collect();

    // 正式链开发期 GRANDPA 只使用国家储委会（NRC）的第 1 把密钥，单节点即可 finalize。
    // 切换到运行期时通过 SwitchToProduction migration 扩展到全部 44 个权威。
    let grandpa_authorities_json: Vec<Value> = vec![json!([
        grandpa_key_to_genesis_ss58(&CHINA_CB[0].grandpa_key),
        1
    ])];

    let mut genesis = serde_json::to_value(crate::RuntimeGenesisConfig::default())
        .expect("default runtime genesis config should serialize");

    let root = genesis
        .as_object_mut()
        .expect("runtime genesis config should serialize to a JSON object");

    root.insert(
        "balances".into(),
        json!({
            "balances": balances_json,
        }),
    );
    root.insert(
        "grandpa".into(),
        json!({
            "authorities": grandpa_authorities_json,
        }),
    );
    root.insert(
        "resolutionIssuance".into(),
        json!({
            "allowedRecipients": issuance_allowed_recipients_json,
        }),
    );

    // 创世常量写入 genesis-pallet 链上存储。
    let citizens_bytes: Vec<u8> = CITIZENS.as_bytes().to_vec();
    let country_bytes: Vec<u8> = COUNTRY.as_bytes().to_vec();
    root.insert(
        "genesisPallet".into(),
        json!({
            "citizensDeclaration": citizens_bytes,
            "countryDeclaration": country_bytes,
            "citizenMax": GENESIS_CITIZEN_MAX,
        }),
    );

    genesis
}

/// 返回 citizenchain 创世配置。
#[cfg(feature = "std")]
pub fn genesis_config() -> Value {
    build_genesis()
}

/// Provides the JSON representation of predefined genesis config for given `id`.
pub fn get_preset(id: &PresetId) -> Option<Vec<u8>> {
    #[cfg(not(feature = "std"))]
    {
        let _ = id;
        return None;
    }

    #[cfg(feature = "std")]
    {
        let patch = match id.as_ref() {
            sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET => genesis_config(),
            _ => return None,
        };
        Some(
            serde_json::to_string(&patch)
                .expect("serialization to json is expected to work. qed.")
                .into_bytes(),
        )
    }
}

/// List of supported presets.
pub fn preset_names() -> Vec<PresetId> {
    vec![PresetId::from(
        sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET,
    )]
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use crate::RuntimeGenesisConfig;
    use ed25519_dalek::VerifyingKey;
    use primitives::cid::china::china_cb::CHINA_CB;
    use std::collections::BTreeSet;

    #[test]
    fn genesis_contains_nrc_and_all_provincialbank_balances() {
        let patch = genesis_config();
        let balances = patch["balances"]["balances"]
            .as_array()
            .expect("balances.balances should be an array");

        // 创世包含 1 个国家储委会多签账户 + 19 个 NRC 管理员 + 43 个省储行 stake 质押地址
        // + 1 个国家储委会两和基金账户。
        let nrc_admins_len = CHINA_CB.first().map(|n| n.admins.len()).unwrap_or(0);
        assert_eq!(balances.len(), 1 + nrc_admins_len + CHINA_CH.len() + 1);

        // 每家省储行的创立发行必须逐户精确进入无私钥 stake_account，不能改发主账户或汇总账户。
        for bank in CHINA_CH {
            let stake_ss58 = account_to_genesis_ss58(&AccountId::new(bank.stake_account));
            let amount = balances
                .iter()
                .find_map(|entry| {
                    let fields = entry.as_array()?;
                    (fields.first()?.as_str()? == stake_ss58)
                        .then(|| fields.get(1).and_then(json_amount_to_u128))
                        .flatten()
                })
                .expect("每家省储行 stake_account 都必须有创立发行余额");
            assert_eq!(amount, bank.stake_amount);
        }
    }

    #[test]
    fn genesis_issuance_goes_entirely_to_nrc() {
        let patch = genesis_config();
        let balances = patch["balances"]["balances"]
            .as_array()
            .expect("balances.balances should be an array");

        let nrc_account = CHINA_CB
            .first()
            .and_then(|n| AccountId::decode(&mut &n.main_account[..]).ok())
            .expect("NRC main_account must decode to AccountId");
        let nrc_ss58 = account_to_genesis_ss58(&nrc_account);

        let nrc_amount = balances
            .iter()
            .find_map(|entry| {
                let arr = entry.as_array()?;
                let account = arr.first()?.as_str()?;
                if account == nrc_ss58 {
                    arr.get(1).and_then(json_amount_to_u128)
                } else {
                    None
                }
            })
            .expect("NRC balance entry should exist");

        // 创世发行分配到国家储委会多签账户 = 总发行量 - NRC 管理员预置总额。
        let admin_each: u128 = 1_000_000_000;
        let nrc_admins_len = CHINA_CB
            .first()
            .map(|n| n.admins.len() as u128)
            .unwrap_or(0);
        let expected_nrc = GENESIS_ISSUANCE - admin_each * nrc_admins_len;
        assert_eq!(nrc_amount, expected_nrc);

        let total_in_patch: u128 = balances
            .iter()
            .map(|entry| {
                entry
                    .as_array()
                    .and_then(|arr| arr.get(1))
                    .and_then(json_amount_to_u128)
                    .expect("each balance amount must be u64 number or u128 string")
            })
            .sum();
        let total_provincialbank_stake: u128 = CHINA_CH.iter().map(|n| n.stake_amount).sum();

        // 创世总注入 = 创世发行 + 省储行创立发行 + 两和基金发行。
        assert_eq!(
            total_in_patch,
            GENESIS_ISSUANCE + total_provincialbank_stake + HE_FUND_ISSUANCE
        );
    }

    #[test]
    fn genesis_omits_national_institutional_registry_without_runtime_pallet() {
        let patch = genesis_config();
        assert!(
            patch.get("nationalInstitutionalRegistry").is_none(),
            "nationalInstitutionalRegistry should be absent until the runtime pallet is wired into genesis"
        );
    }

    #[test]
    fn china_cb_grandpa_keys_are_valid_unique_ed25519_pubkeys() {
        let mut uniq = BTreeSet::new();
        for node in CHINA_CB {
            VerifyingKey::from_bytes(&node.grandpa_key)
                .expect("CHINA_CB.grandpa_key must be valid ed25519 point");
            assert!(
                uniq.insert(node.grandpa_key),
                "CHINA_CB.grandpa_key must be unique"
            );
        }
        assert_eq!(uniq.len(), 44, "must contain exactly 44 grandpa keys");
    }

    #[test]
    fn genesis_json_deserializes_into_runtime_genesis_config() {
        let patch = genesis_config();
        let parsed: Result<RuntimeGenesisConfig, _> = serde_json::from_value(patch);
        assert!(
            parsed.is_ok(),
            "runtime genesis json should deserialize: {:?}",
            parsed.err()
        );
    }

    #[test]
    fn genesis_account_strings_deserialize_individually() {
        let patch = genesis_config();

        for entry in patch["balances"]["balances"]
            .as_array()
            .expect("balances should be an array")
        {
            let account = entry[0].clone();
            let parsed: Result<AccountId, _> = serde_json::from_value(account.clone());
            assert!(
                parsed.is_ok(),
                "balance account should deserialize: value={account:?} err={:?}",
                parsed.err()
            );
        }

        for account in patch["resolutionIssuance"]["allowedRecipients"]
            .as_array()
            .expect("allowedRecipients should be an array")
        {
            let parsed: Result<AccountId, _> = serde_json::from_value(account.clone());
            assert!(
                parsed.is_ok(),
                "allowed recipient should deserialize: value={account:?} err={:?}",
                parsed.err()
            );
        }
    }

    #[test]
    fn genesis_top_level_sections_deserialize_individually() {
        let patch = genesis_config();

        let balances: Result<pallet_balances::GenesisConfig<crate::Runtime>, _> =
            serde_json::from_value(patch["balances"].clone());
        assert!(
            balances.is_ok(),
            "balances should deserialize: {:?}",
            balances.err()
        );

        let grandpa: Result<pallet_grandpa::GenesisConfig<crate::Runtime>, _> =
            serde_json::from_value(patch["grandpa"].clone());
        assert!(
            grandpa.is_ok(),
            "grandpa should deserialize: {:?}",
            grandpa.err()
        );

        let resolution_issuance: Result<resolution_issuance::GenesisConfig<crate::Runtime>, _> =
            serde_json::from_value(patch["resolutionIssuance"].clone());
        assert!(
            resolution_issuance.is_ok(),
            "resolutionIssuance should deserialize: {:?}",
            resolution_issuance.err()
        );
    }
}
