//! 公民币区块链的链规范（Chain specification for CitizenChain）；
//! 本文件定义【公民币唯一主链】的创世状态；
//! 本文件不能在主链上线后被修改。

use sc_service::ChainType;
use sc_chain_spec::ChainSpecExtension;
use serde::{Deserialize, Serialize};
use hex;
use sp_runtime::traits::SaturatedConversion;

// Runtime
// -----------------------------------------------------------------------------
use citizenchain_runtime::{AccountId, Balance, GenesisConfig, SystemConfig,BalancesConfig, WASM_BINARY,};

// 制度常量（来自 primitives，常量唯一来源）
// -----------------------------------------------------------------------------
use primitives::core_const::{TOKEN_SYMBOL, TOKEN_DECIMALS};
use primitives::genesis::{GENESIS_ISSUANCE,DECLARATION_OF_CITIZENS,COUNTRY_NAME_INFO,};
use primitives::shengbank_stakes_const::SHENG_BANK_STAKES;
use primitives::reserve_nodes_const::RESERVE_NODES;

// ChainSpec 类型定义
// -----------------------------------------------------------------------------
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {}

// 链属性
// -----------------------------------------------------------------------------
fn chain_properties() -> sc_service::Properties {
    let mut properties = sc_service::Properties::new();

    // 基础代币制度
	// ------------------------------
    properties.insert("tokenSymbol".into(), TOKEN_SYMBOL.into());
    properties.insert("tokenDecimals".into(), TOKEN_DECIMALS.into());

    // 创世宣言
	// ------------------------------
    properties.insert("declarationOfCitizens".into(),DECLARATION_OF_CITIZENS.into(),);
    properties.insert("countryNameInfo".into(),COUNTRY_NAME_INFO.into(),);
    properties
}

// 唯一主链配置（Mainnet）
// -----------------------------------------------------------------------------
pub fn mainnet_config() -> Result<ChainSpec, String> {
    Ok(ChainSpec::from_genesis(
        // 链名
        "CitizenChain",
        // 链 ID
        "citizenchain",
        ChainType::Live,
        || genesis_config(
            // 国储会多签地址（创世发行接收账户）
            get_national_reserve_account(),
        ),

        // ✅ 引导节点：统一从 RESERVE_NODES.p2p_bootnodes 自动生成
        // -----------------------------------------------------------------
        build_bootnodes_from_reserve_nodes(),
        None,
        None,
        None,
        Some(chain_properties()),
        Default::default(),
    ))
}

// 引导节点构造（唯一权威实现）
// -----------------------------------------------------------------------------
fn build_bootnodes_from_reserve_nodes() -> Vec<sc_service::config::Multiaddr> {
    RESERVE_NODES
        .iter()
        .flat_map(|node| node.p2p_bootnodes.iter())
        .map(|addr| {
            addr.parse()
                .expect("invalid p2p bootnode multiaddr in RESERVE_NODES")
        })
        .collect()
}

// 国储会账户解析（制度唯一来源）
// -----------------------------------------------------------------------------
fn get_national_reserve_account() -> AccountId {
    // 制度约定：RESERVE_NODES[0] = 国储会
    let reserve = &RESERVE_NODES[0];

    let raw = hex::decode(
        reserve.pallet_address.trim_start_matches("0x")
    ).expect("invalid national reserve pallet address hex");

    raw.as_slice()
        .try_into()
        .expect("national reserve address must be 32 bytes")
}

// Genesis 构造函数
// -----------------------------------------------------------------------------
fn genesis_config(
    treasury_account: AccountId,
) -> GenesisConfig {

    let mut balances: Vec<(AccountId, Balance)> = Vec::new();

    // 国家级创世发行 → 国储会
    // ---------------------------------------------------------------------
    balances.push((
        treasury_account,
        GENESIS_ISSUANCE.saturated_into::<Balance>(),
    ));

    // 省级省储行创立发行 → 各省永久质押地址（无私钥）
    // ---------------------------------------------------------------------
    for stake in SHENG_BANK_STAKES {
        let raw = hex::decode(
            stake.keyless_address.trim_start_matches("0x")
        ).expect("invalid sheng bank stake address hex");

        let account: AccountId = raw
            .as_slice()
            .try_into()
            .expect("sheng bank stake address must be 32 bytes");

        balances.push((
            account,
            stake.stake_amount.saturated_into::<Balance>(),
        ));
    }

    GenesisConfig {
        system: SystemConfig {
            code: WASM_BINARY.expect("WASM binary was not built").to_vec(),
        },
        balances: BalancesConfig {
            balances,
        },
        fullnode_pow_reward: citizenchain_runtime::FullnodePowRewardConfig {},
        transaction_payment: Default::default(),
    }
}