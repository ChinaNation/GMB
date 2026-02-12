//! Chain specification for CitizenChain.

use gmb_runtime::WASM_BINARY;
use primitives::core_const::{TOKEN_DECIMALS, TOKEN_SYMBOL};
use sc_chain_spec::NoExtension;
use sc_service::ChainType;

pub type ChainSpec = sc_service::GenericChainSpec<NoExtension>;

fn chain_properties() -> sc_service::Properties {
    let mut properties = sc_service::Properties::new();
    properties.insert("tokenSymbol".into(), TOKEN_SYMBOL.into());
    properties.insert("tokenDecimals".into(), TOKEN_DECIMALS.into());
    properties
}

pub fn development_chain_spec() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "WASM binary was not built".to_string())?;
    Ok(ChainSpec::builder(wasm_binary, NoExtension::default())
        .with_name("CitizenChain Dev")
        .with_id("dev")
        .with_chain_type(ChainType::Development)
        .with_genesis_config_preset_name(sp_genesis_builder::DEV_RUNTIME_PRESET)
        .with_properties(chain_properties())
        .build())
}

pub fn local_chain_spec() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "WASM binary was not built".to_string())?;
    Ok(ChainSpec::builder(wasm_binary, NoExtension::default())
        .with_name("CitizenChain Local")
        .with_id("local")
        .with_chain_type(ChainType::Local)
        .with_genesis_config_preset_name(sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET)
        .with_properties(chain_properties())
        .build())
}

pub fn mainnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "WASM binary was not built".to_string())?;
    Ok(ChainSpec::builder(wasm_binary, NoExtension::default())
        .with_name("CitizenChain")
        .with_id("citizenchain")
        .with_chain_type(ChainType::Live)
        .with_genesis_config_preset_name(sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET)
        .with_properties(chain_properties())
        .build())
}
