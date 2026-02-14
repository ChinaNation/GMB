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

use alloc::vec::Vec;
use alloc::vec;
use sp_genesis_builder::PresetId;

#[cfg(feature = "std")]
use crate::{AccountId, BalancesConfig, RuntimeGenesisConfig};
#[cfg(feature = "std")]
use codec::Decode;
#[cfg(feature = "std")]
use frame_support::build_struct_json_patch;
#[cfg(feature = "std")]
use primitives::{
	genesis::GENESIS_ISSUANCE,
	reserve_nodes_const::RESERVE_NODES,
	shengbank_nodes_const::SHENG_BANK_NODES,
};
#[cfg(feature = "std")]
use serde_json::Value;
#[cfg(feature = "std")]
use sp_genesis_builder::{self};

// Returns the genesis config presets populated with given parameters.
#[cfg(feature = "std")]
fn testnet_genesis(
	endowed_accounts: Vec<AccountId>,
	_root: AccountId,
) -> Value {
	// 中文注释：从统一常量中定位国储会（nrcgch01）交易地址，并解码为链上 AccountId。
	let nrc_account = RESERVE_NODES
		.iter()
		.find(|n| n.pallet_id == "nrcgch01")
		.and_then(|n| AccountId::decode(&mut &n.pallet_address[..]).ok())
		.expect("nrcgch01 pallet_address must decode to AccountId");

	// 中文注释：创世发行总量直接预置到国储会交易地址，单位为“分”。
	let mut genesis_balances: Vec<(AccountId, u128)> = vec![(nrc_account.clone(), GENESIS_ISSUANCE)];

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

	build_struct_json_patch!(RuntimeGenesisConfig {
		balances: BalancesConfig {
			balances: genesis_balances,
		},
	})
}

/// Return the development genesis config.
#[cfg(feature = "std")]
pub fn development_config_genesis() -> Value {
	testnet_genesis(vec![], AccountId::new([0u8; 32]))
}

/// Return the local genesis config preset.
#[cfg(feature = "std")]
pub fn local_config_genesis() -> Value {
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
		sp_genesis_builder::DEV_RUNTIME_PRESET => development_config_genesis(),
		sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET => local_config_genesis(),
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
