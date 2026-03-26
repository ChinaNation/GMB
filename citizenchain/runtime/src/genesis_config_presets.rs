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
use hex_literal::hex;
#[cfg(feature = "std")]
use primitives::{
    china::china_cb::CHINA_CB,
    china::china_ch::CHINA_CH,
    core_const::SS58_FORMAT,
    genesis::{CITIZENS, COUNTRY, GENESIS_CITIZEN_MAX, GENESIS_ISSUANCE},
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
fn grandpa_key_hex_to_genesis_ss58(hex_key: &str) -> String {
    let bytes = hex::decode(hex_key).expect("grandpa key hex must decode");
    let authority = sp_consensus_grandpa::AuthorityId::from_slice(&bytes)
        .expect("grandpa authority id must decode from 32 bytes");
    authority.to_ss58check_with_version(Ss58AddressFormat::custom(SS58_FORMAT))
}

#[cfg(feature = "std")]
const GRANDPA_AUTHORITY_KEYS_HEX: &[&str] = &[
    // 中文注释：44 个最终性验证节点公钥（1 国储会 + 43 省储会），固定在创世配置。
    "3719c39cf92462da2e22a7dfa760f463c801dd86a27a4151d24935e42692e5b5",
    "14803cd63d5ad6c76e0141f730f18f2c4a30ecfaa3631681e490cb5e25ca0958",
    "8d9de5d1b44d39f9567b76a5348d68a497d06b73215d5e89bfdf4f6a6b2f36aa",
    "febdaa3b1c416bffc6ba1e13f799b5295c4644a0d695fe6da7bff3cf6754a903",
    "669d64a0c7c7ba5be629580d8898f9003105b13d18372392cd85562b2782b233",
    "8860948348df8b4efe240829671408476643dbf8d2e07d0bca8f3ea6271c5c51",
    "2d6cfe57d3212c066260cb568645ed0d442a632b24031930b97ca3242111e021",
    "26498679676d181b1346964f5815b94bf766bd284d529ce8e5625547af1fbab3",
    "1c03da677a1ae2e9b4907dc72016ea65adc3d20711f93b8e96f2aa75468072c0",
    "4b9547092662fba8d70658eee8c999c5f5044b2574054a3ce62b2ed19b5645d1",
    "782c1fc618af8c3e1eb225f1b068eb35dfc54e3db2c8067a0ddb55308d62a3d4",
    "a2a7643951e81f6189834bd5c70ee27eddd402ac7c4ef749f40955042b4bb43e",
    "a3819330ccac67a93f09679f81e919e5cc2eedf7166e69a62d402be6ec39de2a",
    "ecbfcf8a0c4e9954f0d8ea71d85b91ad0142b5c5821a74fab99a041be4192948",
    "63e4ac90760504c650773312df9c4ce2faec662c544aeec9c6fcad41654a78c8",
    "e9227e239d61c3ba1463ac0b2206cecf35ebe73a48c7a702a76aeac20327c653",
    "7ce0745ab3270b66570b4b2fa7ef0ea2fd4df5065488c05282cd8ed8c938c199",
    "b10a24b974dabe5f008973007ea3fb67c4af2c7ae77b71f01e0249ac366daab8",
    "14cd5c8e07f738cc3719b4e66c33d81fa3a4baf6f6e50ae1d48d4e2ffb8e47b2",
    "599aa6bb3cf9d48d99599de3c5948308e32dbb427f49346cdab95fe82863b747",
    "26daeb08619448a235d712daa63bf81b97710f684801d22c847624e070e4f600",
    "fa9e7becc8a5984cf15c50ca83e3918adc79521281e2d75ebfcccb7d3e5be8f3",
    "e0dfc5bde6f32d6d41648006c69f6489dea06c37259c57b716ff025fa4fa31af",
    "015829c6f8588903ecbf17207b5fad6d5e766f7512325b13850b43ddd2886305",
    "1527123566372a8082a2d1b62f2dbd00ae3c56f2f71b51b8228178400774c3f3",
    "db5d32816bddb7a0e1d0d17d3863923eb32f76eed87604d0d410a74900195b87",
    "9ffdedc39ab82f7766022a3b53149ffb0f2611567961c0570880adab1a01af88",
    "8133cf439c9142737b34655c48658135d7f946c9d60615a80ed14bad4fec1141",
    "94a7d8f2f0b613dbb1509cc31ffa832188af9f547a0ce25ffe4be56a1cb10e07",
    "6c5ee4dd2a7b9f82a2f04e9eb3bf5ba970adce79cffa662d2dc84adc4ce42492",
    "903f1fcba7a60aeca0221c0fe9dad52b28437775732360a318bc07326638dd39",
    "2f0fcd64f31a318077484148bdf3db0d95de25860764985214356dc3371d598e",
    "eaf447ce1635e7165e9588b6d2864ddf31e2a33ccff69f7a46d69ee2317b4a52",
    "004de5bc4fa8fe5cdce4c1cb00ddce6db55ce5926b8189741cb3cd43d8155f99",
    "ad9ea30f1f967672f8d1b7aabbd8443d860dd2af2e9c9fd9b6a0358fe1690d20",
    "e1dfa8bc752665d0becd9287b28415ced4193371f7cde21e244a8f5b6a1a1a48",
    "ad5fa47ad61097e5dbe077066f34e3cdba31ca0194c184fe556a2f62c4c18172",
    "ae47a89d45e0649ac98aa832eb7d6ae10d6111b30cf64eb118c428c24a00081d",
    "82842c89fe9ce2eca5df47ad92d787e255c0b0dd3ec18947b801077b57743520",
    "4684cd4740972bca1df018cce973771df605cf404c3762c1976f52f9700db391",
    "87f64d53701b846b019341f356e738a87d6c20b244bf2e2d89351717b385a8f3",
    "e7ef1b4ae92e95e9c8b3fb5856e86ea77f596e82c75af20802d61106ca26a25d",
    "0bde5599cdd158c196a45025689be5166a0e4e0ef9e932523040debed85f8b59",
    "a69514c16012e39f3bc49941afa58871aeb46bbfb7825bd296133bad9cd0db9a",
];

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
    // 中文注释：国储会信息统一从常量数组入口读取。
    let nrc_account = CHINA_CB
        .first()
        .and_then(|n| AccountId::decode(&mut &n.duoqian_address[..]).ok())
        .expect("NRC pallet_address must decode to AccountId");

    // 中文注释：创世发行全额存入国储会多签地址。
    let mut genesis_balances: Vec<(AccountId, u128)> =
        vec![(nrc_account.clone(), GENESIS_ISSUANCE)];

    // 中文注释：省储行创立发行在创世时直接预置到各自 keyless_address（无私钥永久质押地址）。
    genesis_balances.extend(
        CHINA_CH
            .iter()
            .map(|bank| (AccountId::new(bank.keyless_address), bank.stake_amount)),
    );

    // 中文注释：创世账户统一输出为链 SS58 地址（前缀 2027）。
    let balances_json: Vec<Value> = genesis_balances
        .into_iter()
        .map(|(account, amount)| {
            let account_ss58 = account_to_genesis_ss58(&account);
            json!([account_ss58, amount])
        })
        .collect();

    // 中文注释：SFID 三把创世账户固定为已确认公钥；不依赖 primitives 常量命名。
    let sfid_main = AccountId::new(hex!(
        "14e4f684453a0ccf9ebb3113d05ae1da934b7f7b2dbd3b9dcdf4138357ab1607"
    ));
    let sfid_backup_1 = AccountId::new(hex!(
        "9084bbff7d86275a50a3f460a435ce4d89c49e659df30a52bce67d9c7e614303"
    ));
    let sfid_backup_2 = AccountId::new(hex!(
        "502a1021f41e025c8c86cb5f486ae9cb83fb8cadd9db29d2dde354baa650f73a"
    ));

    // 中文注释：决议发行合法收款账户改为链上存储初始化，后续可由治理动态更新。
    let issuance_allowed_recipients_json: Vec<Value> = CHINA_CB
        .iter()
        .skip(1)
        .map(|n| {
            let account = AccountId::decode(&mut &n.duoqian_address[..])
                .expect("PRC duoqian_address must decode to AccountId");
            Value::String(account_to_genesis_ss58(&account))
        })
        .collect();

    // 中文注释：正式链开发期 GRANDPA 只使用国储会（NRC）的第 1 把密钥，单节点即可 finalize。
    // 切换到运行期时通过 SwitchToProduction migration 扩展到全部 44 个权威。
    let grandpa_authorities_json: Vec<Value> = vec![
        json!([grandpa_key_hex_to_genesis_ss58(GRANDPA_AUTHORITY_KEYS_HEX[0]), 1]),
    ];

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
        "sfidCodeAuth".into(),
        json!({
            "sfidMainAccount": account_to_genesis_ss58(&sfid_main),
            "sfidBackupAccount1": account_to_genesis_ss58(&sfid_backup_1),
            "sfidBackupAccount2": account_to_genesis_ss58(&sfid_backup_2),
        }),
    );
    root.insert(
        "resolutionIssuanceGov".into(),
        json!({
            "allowedRecipients": issuance_allowed_recipients_json,
        }),
    );

    // 中文注释：创世常量写入 genesis-pallet 链上存储。
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
    vec![
        PresetId::from(sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET),
    ]
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use crate::RuntimeGenesisConfig;
    use ed25519_dalek::VerifyingKey;
    use primitives::china::china_cb::CHINA_CB;
    use std::collections::BTreeSet;

    #[test]
    fn genesis_contains_nrc_and_all_shengbank_balances() {
        let patch = genesis_config();
        let balances = patch["balances"]["balances"]
            .as_array()
            .expect("balances.balances should be an array");

        // 中文注释：创世包含 1 个国储会地址 + 43 个省储行 keyless 质押地址。
        assert_eq!(balances.len(), 1 + CHINA_CH.len());
    }

    #[test]
    fn genesis_issuance_goes_entirely_to_nrc() {
        let patch = genesis_config();
        let balances = patch["balances"]["balances"]
            .as_array()
            .expect("balances.balances should be an array");

        let nrc_account = CHINA_CB
            .first()
            .and_then(|n| AccountId::decode(&mut &n.duoqian_address[..]).ok())
            .expect("NRC pallet_address must decode to AccountId");
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

        // 中文注释：创世发行全额存入国储会。
        assert_eq!(nrc_amount, GENESIS_ISSUANCE);

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
        let total_shengbank_stake: u128 = CHINA_CH.iter().map(|n| n.stake_amount).sum();

        // 中文注释：创世总注入 = 创世发行 + 省储行创立发行。
        assert_eq!(total_in_patch, GENESIS_ISSUANCE + total_shengbank_stake);
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
    fn grandpa_authority_keys_are_unique_valid_hex_and_32_bytes() {
        let mut uniq = BTreeSet::new();
        for key in GRANDPA_AUTHORITY_KEYS_HEX {
            assert_eq!(key.len(), 64, "grandpa key hex must be 64 chars");
            let bytes = hex::decode(key).expect("grandpa key must be valid hex");
            assert_eq!(bytes.len(), 32, "grandpa pubkey must be 32 bytes");
            let mut pubkey = [0u8; 32];
            pubkey.copy_from_slice(&bytes);
            VerifyingKey::from_bytes(&pubkey).expect("grandpa key must be valid ed25519 point");
            assert!(uniq.insert(bytes), "grandpa pubkey must be unique");
        }
        assert_eq!(uniq.len(), 44, "must contain exactly 44 grandpa keys");
    }

    #[test]
    fn grandpa_keys_match_china_cb_grandpa_keys() {
        assert_eq!(
            GRANDPA_AUTHORITY_KEYS_HEX.len(),
            CHINA_CB.len(),
            "grandpa key list length must match CHINA_CB length"
        );
        for (i, node) in CHINA_CB.iter().enumerate() {
            let expected = hex::encode(node.grandpa_key);
            assert_eq!(
                GRANDPA_AUTHORITY_KEYS_HEX[i], expected,
                "grandpa key at index {i} must match CHINA_CB.grandpa_key"
            );
        }
    }

    #[test]
    fn china_cb_grandpa_keys_are_valid_ed25519_pubkeys() {
        for node in CHINA_CB {
            VerifyingKey::from_bytes(&node.grandpa_key)
                .expect("CHINA_CB.grandpa_key must be valid ed25519 point");
        }
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

        for account in patch["resolutionIssuanceGov"]["allowedRecipients"]
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

        for field in [
            "sfidMainAccount",
            "sfidBackupAccount1",
            "sfidBackupAccount2",
        ] {
            let account = patch["sfidCodeAuth"][field].clone();
            let parsed: Result<AccountId, _> = serde_json::from_value(account.clone());
            assert!(
                parsed.is_ok(),
                "sfid account should deserialize: field={field} value={account:?} err={:?}",
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

        let resolution_issuance: Result<resolution_issuance_gov::GenesisConfig<crate::Runtime>, _> =
            serde_json::from_value(patch["resolutionIssuanceGov"].clone());
        assert!(
            resolution_issuance.is_ok(),
            "resolutionIssuanceGov should deserialize: {:?}",
            resolution_issuance.err()
        );

        let sfid: Result<sfid_code_auth::GenesisConfig<crate::Runtime>, _> =
            serde_json::from_value(patch["sfidCodeAuth"].clone());
        assert!(
            sfid.is_ok(),
            "sfidCodeAuth should deserialize: {:?}",
            sfid.err()
        );
    }
}
