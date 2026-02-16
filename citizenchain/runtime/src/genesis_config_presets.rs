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

    json!({
        "balances": {
            "balances": balances_json,
        }
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
