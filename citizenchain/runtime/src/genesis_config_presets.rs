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
    core_const::SS58_FORMAT, genesis::GENESIS_ISSUANCE, reserve_nodes_const::RESERVE_NODES,
    shengbank_nodes_const::SHENG_BANK_NODES,
};
#[cfg(feature = "std")]
use serde_json::{json, Value};
#[cfg(feature = "std")]
use sp_core::crypto::{Ss58AddressFormat, Ss58Codec};
#[cfg(feature = "std")]
use sp_genesis_builder::{self};

#[cfg(feature = "std")]
fn account_to_genesis_ss58(account: &AccountId) -> String {
    // 创世配置地址使用链统一 SS58 前缀（2027）。
    account.to_ss58check_with_version(Ss58AddressFormat::custom(SS58_FORMAT))
}

// Returns the genesis config presets populated with given parameters.
#[cfg(feature = "std")]
fn testnet_genesis(endowed_accounts: Vec<AccountId>, _root: AccountId) -> Value {
    // 中文注释：国储会信息统一从常量数组入口读取。
    let nrc_account = RESERVE_NODES
        .iter()
        .find(|n| n.pallet_id == "nrcgch01")
        .and_then(|n| AccountId::decode(&mut &n.pallet_address[..]).ok())
        .expect("NRC pallet_address must decode to AccountId");

    // 中文注释：创世发行总量直接预置到国储会交易地址，单位为“分”。
    let mut genesis_balances: Vec<(AccountId, u128)> =
        vec![(nrc_account.clone(), GENESIS_ISSUANCE)];

    // 中文注释：省储行创立发行在创世时直接预置到各自 keyless_address（无私钥永久质押地址）。
    genesis_balances.extend(
        SHENG_BANK_NODES
            .iter()
            .map(|bank| (AccountId::new(bank.keyless_address), bank.stake_amount)),
    );

    // 中文注释：开发/测试附加账户继续保留，但避免与国储会地址重复。
    genesis_balances.extend(
        endowed_accounts
            .into_iter()
            .filter(|a| a != &nrc_account)
            .map(|a| (a, 1_000_000_000u128)),
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
    let sfid_main = AccountId::new([
        0xc6, 0xed, 0x4e, 0x83, 0x20, 0x57, 0xd9, 0x3c, 0x08, 0xc3, 0x3e, 0xb9, 0xb0, 0x29, 0x13,
        0x4d, 0x88, 0x13, 0x0f, 0x88, 0xb5, 0x85, 0x7a, 0x9c, 0x8a, 0x74, 0xcd, 0xb2, 0xf6, 0x72,
        0xb1, 0x0a,
    ]);
    let sfid_backup_1 = AccountId::new([
        0x46, 0x9e, 0x28, 0xdc, 0x42, 0xf5, 0xb3, 0x7f, 0xb6, 0x91, 0xcd, 0x01, 0xc9, 0xe8, 0xf8,
        0x06, 0xb2, 0x0a, 0x3c, 0xb2, 0xd7, 0x5c, 0x9f, 0x94, 0x68, 0xe3, 0x29, 0x84, 0xa3, 0x32,
        0x60, 0x56,
    ]);
    let sfid_backup_2 = AccountId::new([
        0x56, 0x70, 0x1c, 0xdf, 0x56, 0x29, 0xb9, 0x6e, 0xc0, 0xd1, 0x1f, 0x41, 0xaf, 0x78, 0x5b,
        0x3a, 0x9e, 0x03, 0xa0, 0xfd, 0x7d, 0xa5, 0x97, 0xce, 0x58, 0x62, 0xbe, 0xe0, 0xe3, 0xe8,
        0x34, 0x65,
    ]);

    // 中文注释：机构创世数据由链规格外部注入；这里默认留空数组。
    let institutions_json: Vec<Value> = Vec::new();

    json!({
        "balances": {
            "balances": balances_json,
        },
        "sfidCodeAuth": {
            "sfidMainAccount": account_to_genesis_ss58(&sfid_main),
            "sfidBackupAccount1": account_to_genesis_ss58(&sfid_backup_1),
            "sfidBackupAccount2": account_to_genesis_ss58(&sfid_backup_2),
        },
        "nationalInstitutionalRegistry": {
            "institutions": institutions_json,
        },
    })
}

/// Return the development genesis config.
#[cfg(feature = "std")]
pub fn mainnet_config_genesis() -> Value {
    testnet_genesis(vec![], AccountId::new([0u8; 32]))
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
            sp_genesis_builder::DEV_RUNTIME_PRESET => mainnet_config_genesis(),
            sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET => mainnet_config_genesis(),
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
        PresetId::from(sp_genesis_builder::DEV_RUNTIME_PRESET),
        PresetId::from(sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET),
    ]
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use primitives::reserve_nodes_const::RESERVE_NODES;

    #[test]
    fn mainnet_genesis_contains_nrc_and_all_shengbank_balances() {
        let patch = mainnet_config_genesis();
        let balances = patch["balances"]["balances"]
            .as_array()
            .expect("balances.balances should be an array");

        // 中文注释：创世应包含 1 个国储会地址 + 43 个省储行 keyless 质押地址。
        assert_eq!(balances.len(), 1 + SHENG_BANK_NODES.len());
    }

    #[test]
    fn mainnet_genesis_issuance_split_is_correct() {
        let patch = mainnet_config_genesis();
        let balances = patch["balances"]["balances"]
            .as_array()
            .expect("balances.balances should be an array");

        let nrc_account = RESERVE_NODES
            .iter()
            .find(|n| n.pallet_id == "nrcgch01")
            .and_then(|n| AccountId::decode(&mut &n.pallet_address[..]).ok())
            .expect("NRC pallet_address must decode to AccountId");
        let nrc_ss58 = account_to_genesis_ss58(&nrc_account);

        let nrc_amount = balances
            .iter()
            .find_map(|entry| {
                let arr = entry.as_array()?;
                let account = arr.first()?.as_str()?;
                if account == nrc_ss58 {
                    arr.get(1)?.as_u64().map(|v| v as u128)
                } else {
                    None
                }
            })
            .expect("NRC balance entry should exist");

        // 中文注释：国储会地址仅承载创世发行，不应与省储行创立发行混淆。
        assert_eq!(nrc_amount, GENESIS_ISSUANCE);

        let total_in_patch: u128 = balances
            .iter()
            .map(|entry| {
                entry
                    .as_array()
                    .and_then(|arr| arr.get(1))
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u128)
                    .expect("each balance amount must be u64-compatible JSON number")
            })
            .sum();
        let total_shengbank_stake: u128 = SHENG_BANK_NODES.iter().map(|n| n.stake_amount).sum();

        // 中文注释：创世总注入 = 国储会创世发行 + 省储行创立发行。
        assert_eq!(total_in_patch, GENESIS_ISSUANCE + total_shengbank_stake);
    }

    #[test]
    fn mainnet_genesis_contains_federal_registry_institutions() {
        let patch = mainnet_config_genesis();
        let institutions = patch["nationalInstitutionalRegistry"]["institutions"]
            .as_array()
            .expect("nationalInstitutionalRegistry.institutions should be an array");
        assert!(institutions.is_empty());
    }
}
