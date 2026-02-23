//! Chain specification for CitizenChain.

use gmb_runtime::{genesis_config_presets, WASM_BINARY};
use primitives::core_const::{CHAIN_ID, CHAIN_NAME, SS58_FORMAT, TOKEN_DECIMALS, TOKEN_SYMBOL};
use primitives::reserve_nodes_const::CHINACB;
use sc_chain_spec::NoExtension;
use sc_service::ChainType;

pub type ChainSpec = sc_service::GenericChainSpec<NoExtension>;

fn chain_properties() -> sc_service::Properties {
    let mut properties = sc_service::Properties::new();
    properties.insert("tokenSymbol".into(), TOKEN_SYMBOL.into());
    properties.insert("tokenDecimals".into(), TOKEN_DECIMALS.into());
    // 中文注释：显式声明地址显示前缀，避免工具默认按 42（Substrate Generic）展示。
    properties.insert("ss58Format".into(), SS58_FORMAT.into());
    properties
}

fn reserve_boot_nodes() -> Result<Vec<sc_network::config::MultiaddrWithPeerId>, String> {
    CHINACB
        .iter()
        .flat_map(|node| node.p2p_bootnodes.iter().copied())
        .map(|addr| {
            addr.parse::<sc_network::config::MultiaddrWithPeerId>()
                .map_err(|e| format!("invalid bootnode `{addr}`: {e}"))
        })
        .collect()
}

pub fn mainnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "WASM binary was not built".to_string())?;
    let boot_nodes = reserve_boot_nodes()?;
    Ok(ChainSpec::builder(wasm_binary, NoExtension::default())
        .with_name(CHAIN_NAME)
        .with_id(CHAIN_ID)
        .with_chain_type(ChainType::Live)
        .with_boot_nodes(boot_nodes)
        // 中文注释：唯一创世来源（主网）。
        .with_genesis_config_patch(genesis_config_presets::mainnet_config_genesis())
        .with_properties(chain_properties())
        .build())
}
