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
use crate::{AccountId, BalancesConfig, RuntimeGenesisConfig, SudoConfig};
#[cfg(feature = "std")]
use frame_support::build_struct_json_patch;
#[cfg(feature = "std")]
use serde_json::Value;
#[cfg(feature = "std")]
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
#[cfg(feature = "std")]
use sp_consensus_grandpa::AuthorityId as GrandpaId;
#[cfg(feature = "std")]
use sp_genesis_builder::{self};

// Returns the genesis config presets populated with given parameters.
#[cfg(feature = "std")]
fn testnet_genesis(
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	endowed_accounts: Vec<AccountId>,
	root: AccountId,
) -> Value {
	build_struct_json_patch!(RuntimeGenesisConfig {
		balances: BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 1u128 << 60))
				.collect::<Vec<_>>(),
		},
		aura: pallet_aura::GenesisConfig {
			authorities: initial_authorities.iter().map(|x| x.0.clone()).collect::<Vec<_>>(),
		},
		grandpa: pallet_grandpa::GenesisConfig {
			authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect::<Vec<_>>(),
		},
		sudo: SudoConfig { key: Some(root) },
	})
}

/// Return the development genesis config.
#[cfg(feature = "std")]
pub fn development_config_genesis() -> Value {
	testnet_genesis(vec![], vec![], AccountId::new([0u8; 32]))
}

/// Return the local genesis config preset.
#[cfg(feature = "std")]
pub fn local_config_genesis() -> Value {
	testnet_genesis(vec![], vec![], AccountId::new([0u8; 32]))
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
