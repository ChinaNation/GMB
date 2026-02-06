//! Chain specification for CitizenChain
//! ⚠️ 本文件定义【唯一主链】的创世状态。
//! 本文件不能在主链上线后被修改，否则将构成“换链”。

use sc_service::ChainType;
use sc_chain_spec::ChainSpecExtension;
use serde::{Deserialize, Serialize};
use hex;
use hex_literal::hex;

use sp_runtime::traits::SaturatedConversion;

use citizenchain_runtime::{
    AccountId, Balance, GenesisConfig, SystemConfig,
    BalancesConfig, WASM_BINARY,
};

// -----------------------------------------------------------------------------
// 制度常量（来自 primitives，唯一权威来源）
// -----------------------------------------------------------------------------
use primitives::core_const::{TOKEN_SYMBOL, TOKEN_DECIMALS};
use primitives::genesis::GENESIS_ISSUANCE;
use primitives::shengbank_stakes_const::SHENG_BANK_STAKES;
use primitives::reserve_nodes_const::RESERVE_NODES;

pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {}

fn chain_properties() -> sc_service::Properties {
    let mut properties = sc_service::Properties::new();
    properties.insert("tokenSymbol".into(), TOKEN_SYMBOL.into());
    properties.insert("tokenDecimals".into(), TOKEN_DECIMALS.into());
    properties
}

// -----------------------------------------------------------------------------
// 唯一主链配置（Mainnet）
// -----------------------------------------------------------------------------

pub fn mainnet_config() -> Result<ChainSpec, String> {
    Ok(ChainSpec::from_genesis(
        // ✅ 链名（给人看的）
        "CitizenChain",
        // ✅ 链 ID（给机器看的，上线后不可再改）
        "citizenchain",
        ChainType::Live,
        || genesis_config(
            // ✅ 国储会多签地址（来自制度常量）
            get_national_reserve_account(),
        ),
        vec![
            "/dns4/nrcgch01.wuminbi.com/tcp/30333/p2p/12D3KooWHepcMGD3h9VC1XNWmrac3pXo63RimV5jhTU2nC2TLAyS".parse().unwrap(),
            "/dns4/prczss01.wuminbi.com/tcp/30333/p2p/12D3KooWPjWNXvCzPv6PPuiGnF3J5uToW3ySfaB7rKkwUrN2CALv".parse().unwrap(),
            "/dns4/prclns02.wuminbi.com/tcp/30333/p2p/12D3KooWD9EpWCRceAQBc5rxq8pMS75ke9ovDyqAF8ZjoVQVD3tt".parse().unwrap(),
            "/dns4/prcgds03.wuminbi.com/tcp/30333/p2p/12D3KooWJKT8iE9guv4wfem1L9Xd91bNC9CTcLmZyRgUuWkpmEqf".parse().unwrap(),
            "/dns4/prcgxs04.wuminbi.com/tcp/30333/p2p/12D3KooWAxCE4TpEkDKibtQBzFtEuTAvxrDp1JXabhXPY7tAp9qx".parse().unwrap(),
            "/dns4/prcfjs05.wuminbi.com/tcp/30333/p2p/12D3KooWJdGUANuEpVCmarfH2gi23GodbbbBBabuw9Eb4raBabt8".parse().unwrap(),
            "/dns4/prchns06.wuminbi.com/tcp/30333/p2p/12D3KooWEhovD6QmFbZZGBS7pkwKZinfGZPCAKvyEGGDqkja8HDa".parse().unwrap(),
            "/dns4/prcyns07.wuminbi.com/tcp/30333/p2p/12D3KooWB7kZKwKEPFDo7DToUeFHeyZCJWXUR1wUN1t6uW7mFr2Z".parse().unwrap(),
            "/dns4/prcgzs08.wuminbi.com/tcp/30333/p2p/12D3KooWC7t4V1Z2aQWS9HikBdXQgXEaTqeZ5YD78cnxtYBDn31M".parse().unwrap(),
            "/dns4/prchns09.wuminbi.com/tcp/30333/p2p/12D3KooWHS6G18ZtqiCGFYxb3CdvXT3Hb3zds8zknuWPCsdkFPPL".parse().unwrap(),
            "/dns4/prcjxs10.wuminbi.com/tcp/30333/p2p/12D3KooWNpANUi6qmJCJXkMzyAMzjf4nY9wUdkAbwcGRJgikSY13".parse().unwrap(),
            "/dns4/prczjs11.wuminbi.com/tcp/30333/p2p/12D3KooWKLAEv8qEicjGX3MF667gqGF8Lf1iEATskv61pRdGaxS4".parse().unwrap(),
            "/dns4/prcjss12.wuminbi.com/tcp/30333/p2p/12D3KooWQqjnQ8wLx6qNX94PoJGZgEJkgyCA3G5ck3zetcpuQp7f".parse().unwrap(),
            "/dns4/prcsds13.wuminbi.com/tcp/30333/p2p/12D3KooWFgD8cFDqherjpiuRkHwHfAcCwaqXcBjTS2G3LkwUBTsq".parse().unwrap(),
            "/dns4/prcsxs14.wuminbi.com/tcp/30333/p2p/12D3KooWQY3DEaJy9wEBE2bQ9gG1B8XByfVaz839jf1ov75kRmD9".parse().unwrap(),
            "/dns4/prchns15.wuminbi.com/tcp/30333/p2p/12D3KooWSkKBEJ2KZXckFhzLvrqqbhpq4PVKeFuWsxdTF7hfzoGc".parse().unwrap(),
            "/dns4/prchbs16.wuminbi.com/tcp/30333/p2p/12D3KooWMXQoZ9F6nxMuoC2ZnzxEKAn4z2qPKAugP2CZFEcXDqkT".parse().unwrap(),
            "/dns4/prchbs17.wuminbi.com/tcp/30333/p2p/12D3KooWS2WYJ9AQ6Y1AKZcKjaHbmCFNkozV7XBBqqDG8kvwsH22".parse().unwrap(),
            "/dns4/prcsxs18.wuminbi.com/tcp/30333/p2p/12D3KooWNr4EWB1PwBANoU9h2FzZXfS78vxDQynLtft3TDWMQ42p".parse().unwrap(),
            "/dns4/prccqs19.wuminbi.com/tcp/30333/p2p/12D3KooWD8qAmRfVPyDn65j8aNLUZ3xKpc4jVVJ2Jdro3LZKJhrY".parse().unwrap(),
            "/dns4/prcscs20.wuminbi.com/tcp/30333/p2p/12D3KooWR63RRCk3PDbyBEY8zdEB7JXKzqGGrTwx1RgeQUmr28ZH".parse().unwrap(),
            "/dns4/prcgss21.wuminbi.com/tcp/30333/p2p/12D3KooWRKEFiEJGBdK6AdkJb6ei5FJiqSAvEkk4NxGnoT9p5MUS".parse().unwrap(),
            "/dns4/prcbps22.wuminbi.com/tcp/30333/p2p/12D3KooWQZF44Z2U9mT6Q371ULaRLHK9ucTuxPVV8WpaUnw9Q4Ug".parse().unwrap(),
            "/dns4/prcbhs23.wuminbi.com/tcp/30333/p2p/12D3KooWE69n2vS9KqPuXvZPAVRAXwfLcnAfHLz6EDBCD6G8Zqdk".parse().unwrap(),
            "/dns4/prcsjs24.wuminbi.com/tcp/30333/p2p/12D3KooWRQt9MWd8v1F5b8nNksRgvCk7XmMgntxiv6RX12gkY5Dx".parse().unwrap(),
            "/dns4/prcljs25.wuminbi.com/tcp/30333/p2p/12D3KooWGdzag2ekE4JBbcNYNNg3bAJJrqfrZQnsC4uaVavNpmtX".parse().unwrap(),
            "/dns4/prcjls26.wuminbi.com/tcp/30333/p2p/12D3KooWHbuz7D91uDpbEPKLpSSKE9ZVqPSsTXFMewBbYAAxJYc2".parse().unwrap(),
            "/dns4/prclns27.wuminbi.com/tcp/30333/p2p/12D3KooWE8RugcDKrBwxobPzGkVxke4WnGJhi74No53EH7zhaziB".parse().unwrap(),
            "/dns4/prcnxs28.wuminbi.com/tcp/30333/p2p/12D3KooWGdFwKQQoZTyGbKHtq6FcEjmXSWJ4MfdebuM37MXXNV1T".parse().unwrap(),
            "/dns4/prcqhs29.wuminbi.com/tcp/30333/p2p/12D3KooWEL5PTHVD4HEGRcsTxQKWanzW31qSzAGapvwnBsfdTWWS".parse().unwrap(),
            "/dns4/prcahs30.wuminbi.com/tcp/30333/p2p/12D3KooWPC96XCXpuuErd8G7bteNhmvkk6NTPjLtccPCiRwLRGSw".parse().unwrap(),
            "/dns4/prctws31.wuminbi.com/tcp/30333/p2p/12D3KooWQYc1jQZQyaUQC1snk9DHGmydhMdgtJ9LZZ5pbzTciG2J".parse().unwrap(),
            "/dns4/prcxzs32.wuminbi.com/tcp/30333/p2p/12D3KooWNhQUZN2zvX8WTa5SvbyziGvr18qjVNnhygstb8KHQ7Ro".parse().unwrap(),
            "/dns4/prcxjs33.wuminbi.com/tcp/30333/p2p/12D3KooWMbsFaTXiGKXqjEFZjuP5Tp7iU4FFvf3MJoSmGRXDVc69".parse().unwrap(),
            "/dns4/prcxks34.wuminbi.com/tcp/30333/p2p/12D3KooWBczZmptJkbQkX4yx4XP7QXwtJXxZn1We8R4GtbRExUox".parse().unwrap(),
            "/dns4/prcals35.wuminbi.com/tcp/30333/p2p/12D3KooWJKCXsrzLVWLuZVTENBLeLG5F9KcLoeGhdp1tjs8qtk2y".parse().unwrap(),
            "/dns4/prccls36.wuminbi.com/tcp/30333/p2p/12D3KooWMU7y4HSkWdKQYQ15xQC9L33TUkfcMfBgYQtkxMDcos9v".parse().unwrap(),
            "/dns4/prctss37.wuminbi.com/tcp/30333/p2p/12D3KooWG8ZyfEQo7MkkcKqUczQkY1eKVZFvvAeUpz4EPFi8vEoN".parse().unwrap(),
            "/dns4/prchxs38.wuminbi.com/tcp/30333/p2p/12D3KooWBjDquSWFYAjTy5LWxBqYKC453WT8FpoJTVKK6qGk2G4y".parse().unwrap(),
            "/dns4/prckls39.wuminbi.com/tcp/30333/p2p/12D3KooWSeAo5RUTjTX53NmD8Ncv6fXfnkqd461a6FBEGv8szB8N".parse().unwrap(),
            "/dns4/prchts40.wuminbi.com/tcp/30333/p2p/12D3KooWFrXygQG5HZ1buBcrGwe7KYQagNu29ippkUAbLUndxt9v".parse().unwrap(),
            "/dns4/prcrhs41.wuminbi.com/tcp/30333/p2p/12D3KooWBUyRBBAb6QFkJ3obK1bniWNFu4Gk7VoZAAQo7jrQfNCf".parse().unwrap(),
            "/dns4/prcxas42.wuminbi.com/tcp/30333/p2p/12D3KooWC4errbqKaeyDZVjpNmpryUAfbLM8h6CjvyAbkYjzgnne".parse().unwrap(),
            "/dns4/prchjs43.wuminbi.com/tcp/30333/p2p/12D3KooWPciaAo15DT24rXPZK5EUtBdEyotFBhvEdw6d3zBmzVHH".parse().unwrap(),
        ],
        None, None, None,
        Some(chain_properties()),
        Default::default(),
    ))
}

// -----------------------------------------------------------------------------
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

// -----------------------------------------------------------------------------
// Genesis 构造函数（最终版）
// -----------------------------------------------------------------------------

fn genesis_config(
    treasury_account: AccountId,
) -> GenesisConfig {

    let mut balances: Vec<(AccountId, Balance)> = Vec::new();

    // 国家级创世发行 → 国储会
    balances.push((
        treasury_account,
        GENESIS_ISSUANCE.saturated_into::<Balance>(),
    ));

    // 省储行创立发行 → 各省永久质押地址
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