//! Chain specification for CitizenChain.

use gmb_runtime::{genesis_config_presets, WASM_BINARY};
use primitives::core_const::{TOKEN_DECIMALS, TOKEN_SYMBOL};
use sc_chain_spec::NoExtension;
use sc_service::ChainType;

pub type ChainSpec = sc_service::GenericChainSpec<NoExtension>;

fn chain_properties() -> sc_service::Properties {
    let mut properties = sc_service::Properties::new();
    properties.insert("tokenSymbol".into(), TOKEN_SYMBOL.into());
    properties.insert("tokenDecimals".into(), TOKEN_DECIMALS.into());
    // 中文注释：显式声明地址显示前缀，避免工具默认按 42（Substrate Generic）展示。
    properties.insert("ss58Format".into(), 2027.into());
    properties
}

pub fn development_chain_spec() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "WASM binary was not built".to_string())?;
    Ok(ChainSpec::builder(wasm_binary, NoExtension::default())
        .with_name("CitizenChain Dev")
        .with_id("dev")
        .with_chain_type(ChainType::Development)
        // 中文注释：直接注入本地生成的创世补丁，避免依赖 runtime preset 名称解析。
        .with_genesis_config_patch(genesis_config_presets::development_config_genesis())
        .with_properties(chain_properties())
        .build())
}

pub fn local_chain_spec() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "WASM binary was not built".to_string())?;
    Ok(ChainSpec::builder(wasm_binary, NoExtension::default())
        .with_name("CitizenChain Local")
        .with_id("local")
        .with_chain_type(ChainType::Local)
        // 中文注释：直接注入本地生成的创世补丁，避免依赖 runtime preset 名称解析。
        .with_genesis_config_patch(genesis_config_presets::local_config_genesis())
        .with_properties(chain_properties())
        .build())
}

pub fn mainnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "WASM binary was not built".to_string())?;
    Ok(ChainSpec::builder(wasm_binary, NoExtension::default())
        .with_name("CitizenChain")
        .with_id("citizenchain")
        .with_chain_type(ChainType::Live)
        // 中文注释：主网当前沿用 local 创世补丁，后续可替换为独立 mainnet 补丁函数。
        .with_genesis_config_patch(genesis_config_presets::local_config_genesis())
        .with_properties(chain_properties())
        .build())
}
