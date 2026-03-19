//! Chain specification for CitizenChain.

use citizenchain::{genesis_config_presets, WASM_BINARY};
use primitives::core_const::{CHAIN_ID, CHAIN_NAME, SS58_FORMAT, TOKEN_DECIMALS, TOKEN_SYMBOL};
use sc_chain_spec::NoExtension;
use sc_service::ChainType;

pub type ChainSpec = sc_service::GenericChainSpec<NoExtension>;

struct ChainSpecBootnode {
    name: &'static str,
    addr: &'static str,
}

const CHAIN_SPEC_BOOTNODES: &[ChainSpecBootnode] = &[
    ChainSpecBootnode { name: "国储会权威节点", addr: "/dns4/nrcgch.wuminapp.com/tcp/30333/p2p/12D3KooWHepcMGD3h9VC1XNWmrac3pXo63RimV5jhTU2nC2TLAyS" },
    ChainSpecBootnode { name: "中枢省权威节点", addr: "/dns4/prczss.wuminapp.com/tcp/30333/p2p/12D3KooWPjWNXvCzPv6PPuiGnF3J5uToW3ySfaB7rKkwUrN2CALv" },
    ChainSpecBootnode { name: "岭南省权威节点", addr: "/dns4/prclns.wuminapp.com/tcp/30333/p2p/12D3KooWD9EpWCRceAQBc5rxq8pMS75ke9ovDyqAF8ZjoVQVD3tt" },
    ChainSpecBootnode { name: "广东省权威节点", addr: "/dns4/prcgds.wuminapp.com/tcp/30333/p2p/12D3KooWJKT8iE9guv4wfem1L9Xd91bNC9CTcLmZyRgUuWkpmEqf" },
    ChainSpecBootnode { name: "广西省权威节点", addr: "/dns4/prcgxs.wuminapp.com/tcp/30333/p2p/12D3KooWAxCE4TpEkDKibtQBzFtEuTAvxrDp1JXabhXPY7tAp9qx" },
    ChainSpecBootnode { name: "福建省权威节点", addr: "/dns4/prcfjs.wuminapp.com/tcp/30333/p2p/12D3KooWJdGUANuEpVCmarfH2gi23GodbbbBBabuw9Eb4raBabt8" },
    ChainSpecBootnode { name: "海南省权威节点", addr: "/dns4/prchns.wuminapp.com/tcp/30333/p2p/12D3KooWEhovD6QmFbZZGBS7pkwKZinfGZPCAKvyEGGDqkja8HDa" },
    ChainSpecBootnode { name: "云南省权威节点", addr: "/dns4/prcyns.wuminapp.com/tcp/30333/p2p/12D3KooWB7kZKwKEPFDo7DToUeFHeyZCJWXUR1wUN1t6uW7mFr2Z" },
    ChainSpecBootnode { name: "贵州省权威节点", addr: "/dns4/prcgzs.wuminapp.com/tcp/30333/p2p/12D3KooWC7t4V1Z2aQWS9HikBdXQgXEaTqeZ5YD78cnxtYBDn31M" },
    ChainSpecBootnode { name: "湖南省权威节点", addr: "/dns4/prchus.wuminapp.com/tcp/30333/p2p/12D3KooWHS6G18ZtqiCGFYxb3CdvXT3Hb3zds8zknuWPCsdkFPPL" },
    ChainSpecBootnode { name: "江西省权威节点", addr: "/dns4/prcjxs.wuminapp.com/tcp/30333/p2p/12D3KooWNpANUi6qmJCJXkMzyAMzjf4nY9wUdkAbwcGRJgikSY13" },
    ChainSpecBootnode { name: "浙江省权威节点", addr: "/dns4/prczjs.wuminapp.com/tcp/30333/p2p/12D3KooWKLAEv8qEicjGX3MF667gqGF8Lf1iEATskv61pRdGaxS4" },
    ChainSpecBootnode { name: "江苏省权威节点", addr: "/dns4/prcjss.wuminapp.com/tcp/30333/p2p/12D3KooWQqjnQ8wLx6qNX94PoJGZgEJkgyCA3G5ck3zetcpuQp7f" },
    ChainSpecBootnode { name: "山东省权威节点", addr: "/dns4/prcsds.wuminapp.com/tcp/30333/p2p/12D3KooWFgD8cFDqherjpiuRkHwHfAcCwaqXcBjTS2G3LkwUBTsq" },
    ChainSpecBootnode { name: "山西省权威节点", addr: "/dns4/prcsxs.wuminapp.com/tcp/30333/p2p/12D3KooWQY3DEaJy9wEBE2bQ9gG1B8XByfVaz839jf1ov75kRmD9" },
    ChainSpecBootnode { name: "河南省权威节点", addr: "/dns4/prches.wuminapp.com/tcp/30333/p2p/12D3KooWSkKBEJ2KZXckFhzLvrqqbhpq4PVKeFuWsxdTF7hfzoGc" },
    ChainSpecBootnode { name: "河北省权威节点", addr: "/dns4/prchbs.wuminapp.com/tcp/30333/p2p/12D3KooWMXQoZ9F6nxMuoC2ZnzxEKAn4z2qPKAugP2CZFEcXDqkT" },
    ChainSpecBootnode { name: "湖北省权威节点", addr: "/dns4/prchis.wuminapp.com/tcp/30333/p2p/12D3KooWS2WYJ9AQ6Y1AKZcKjaHbmCFNkozV7XBBqqDG8kvwsH22" },
    ChainSpecBootnode { name: "陕西省权威节点", addr: "/dns4/prcsis.wuminapp.com/tcp/30333/p2p/12D3KooWNr4EWB1PwBANoU9h2FzZXfS78vxDQynLtft3TDWMQ42p" },
    ChainSpecBootnode { name: "重庆省权威节点", addr: "/dns4/prccqs.wuminapp.com/tcp/30333/p2p/12D3KooWD8qAmRfVPyDn65j8aNLUZ3xKpc4jVVJ2Jdro3LZKJhrY" },
    ChainSpecBootnode { name: "四川省权威节点", addr: "/dns4/prcscs.wuminapp.com/tcp/30333/p2p/12D3KooWR831Zp5wr6AXtwo5f6uoLzig1vTq8GtN8PK7AL3A4t1m" },
    ChainSpecBootnode { name: "甘肃省权威节点", addr: "/dns4/prcgss.wuminapp.com/tcp/30333/p2p/12D3KooWRKEFiEJGBdK6AdkJb6ei5FJiqSAvEkk4NxGnoT9p5MUS" },
    ChainSpecBootnode { name: "北平省权威节点", addr: "/dns4/prcbps.wuminapp.com/tcp/30333/p2p/12D3KooWQZF44Z2U9mT6Q371ULaRLHK9ucTuxPVV8WpaUnw9Q4Ug" },
    ChainSpecBootnode { name: "海滨省权威节点", addr: "/dns4/prchas.wuminapp.com/tcp/30333/p2p/12D3KooWE69n2vS9KqPuXvZPAVRAXwfLcnAfHLz6EDBCD6G8Zqdk" },
    ChainSpecBootnode { name: "松江省权威节点", addr: "/dns4/prcsjs.wuminapp.com/tcp/30333/p2p/12D3KooWRQt9MWd8v1F5b8nNksRgvCk7XmMgntxiv6RX12gkY5Dx" },
    ChainSpecBootnode { name: "龙江省权威节点", addr: "/dns4/prcljs.wuminapp.com/tcp/30333/p2p/12D3KooWGdzag2ekE4JBbcNYNNg3bAJJrqfrZQnsC4uaVavNpmtX" },
    ChainSpecBootnode { name: "吉林省权威节点", addr: "/dns4/prcjls.wuminapp.com/tcp/30333/p2p/12D3KooWHbuz7D91uDpbEPKLpSSKE9ZVqPSsTXFMewBbYAAxJYc2" },
    ChainSpecBootnode { name: "辽宁省权威节点", addr: "/dns4/prclis.wuminapp.com/tcp/30333/p2p/12D3KooWE8RugcDKrBwxobPzGkVxke4WnGJhi74No53EH7zhaziB" },
    ChainSpecBootnode { name: "宁夏省权威节点", addr: "/dns4/prcnxs.wuminapp.com/tcp/30333/p2p/12D3KooWGdFwKQQoZTyGbKHtq6FcEjmXSWJ4MfdebuM37MXXNV1T" },
    ChainSpecBootnode { name: "青海省权威节点", addr: "/dns4/prcqhs.wuminapp.com/tcp/30333/p2p/12D3KooWEL5PTHVD4HEGRcsTxQKWanzW31qSzAGapvwnBsfdTWWS" },
    ChainSpecBootnode { name: "安徽省权威节点", addr: "/dns4/prcahs.wuminapp.com/tcp/30333/p2p/12D3KooWPC96XCXpuuErd8G7bteNhmvkk6NTPjLtccPCiRwLRGSw" },
    ChainSpecBootnode { name: "台湾省权威节点", addr: "/dns4/prctws.wuminapp.com/tcp/30333/p2p/12D3KooWQYc1jQZQyaUQC1snk9DHGmydhMdgtJ9LZZ5pbzTciG2J" },
    ChainSpecBootnode { name: "西藏省权威节点", addr: "/dns4/prcxzs.wuminapp.com/tcp/30333/p2p/12D3KooWNhQUZN2zvX8WTa5SvbyziGvr18qjVNnhygstb8KHQ7Ro" },
    ChainSpecBootnode { name: "新疆省权威节点", addr: "/dns4/prcxjs.wuminapp.com/tcp/30333/p2p/12D3KooWMbsFaTXiGKXqjEFZjuP5Tp7iU4FFvf3MJoSmGRXDVc69" },
    ChainSpecBootnode { name: "西康省权威节点", addr: "/dns4/prcxks.wuminapp.com/tcp/30333/p2p/12D3KooWBczZmptJkbQkX4yx4XP7QXwtJXxZn1We8R4GtbRExUox" },
    ChainSpecBootnode { name: "阿里省权威节点", addr: "/dns4/prcals.wuminapp.com/tcp/30333/p2p/12D3KooWJKCXsrzLVWLuZVTENBLeLG5F9KcLoeGhdp1tjs8qtk2y" },
    ChainSpecBootnode { name: "葱岭省权威节点", addr: "/dns4/prccls.wuminapp.com/tcp/30333/p2p/12D3KooWMU7y4HSkWdKQYQ15xQC9L33TUkfcMfBgYQtkxMDcos9v" },
    ChainSpecBootnode { name: "天山省权威节点", addr: "/dns4/prctss.wuminapp.com/tcp/30333/p2p/12D3KooWG8ZyfEQo7MkkcKqUczQkY1eKVZFvvAeUpz4EPFi8vEoN" },
    ChainSpecBootnode { name: "河西省权威节点", addr: "/dns4/prchxs.wuminapp.com/tcp/30333/p2p/12D3KooWBjDquSWFYAjTy5LWxBqYKC453WT8FpoJTVKK6qGk2G4y" },
    ChainSpecBootnode { name: "昆仑省权威节点", addr: "/dns4/prckls.wuminapp.com/tcp/30333/p2p/12D3KooWSeAo5RUTjTX53NmD8Ncv6fXfnkqd461a6FBEGv8szB8N" },
    ChainSpecBootnode { name: "河套省权威节点", addr: "/dns4/prchts.wuminapp.com/tcp/30333/p2p/12D3KooWFrXygQG5HZ1buBcrGwe7KYQagNu29ippkUAbLUndxt9v" },
    ChainSpecBootnode { name: "热河省权威节点", addr: "/dns4/prcrhs.wuminapp.com/tcp/30333/p2p/12D3KooWBUyRBBAb6QFkJ3obK1bniWNFu4Gk7VoZAAQo7jrQfNCf" },
    ChainSpecBootnode { name: "兴安省权威节点", addr: "/dns4/prcxas.wuminapp.com/tcp/30333/p2p/12D3KooWC4errbqKaeyDZVjpNmpryUAfbLM8h6CjvyAbkYjzgnne" },
    ChainSpecBootnode { name: "合江省权威节点", addr: "/dns4/prchjs.wuminapp.com/tcp/30333/p2p/12D3KooWPciaAo15DT24rXPZK5EUtBdEyotFBhvEdw6d3zBmzVHH" },
];

fn chain_properties() -> sc_service::Properties {
    let mut properties = sc_service::Properties::new();
    properties.insert("tokenSymbol".into(), TOKEN_SYMBOL.into());
    properties.insert("tokenDecimals".into(), TOKEN_DECIMALS.into());
    // 中文注释：显式声明地址显示前缀。
    properties.insert("ss58Format".into(), SS58_FORMAT.into());
    properties
}

fn reserve_boot_nodes() -> Result<Vec<sc_network::config::MultiaddrWithPeerId>, String> {
    // 中文注释：chain spec 的 bootNodes 统一在本文件内定义为单一来源。
    CHAIN_SPEC_BOOTNODES
        .iter()
        .map(|node| {
            node.addr
                .parse::<sc_network::config::MultiaddrWithPeerId>()
                .map_err(|e| format!("invalid bootnode `{} ({})`: {e}", node.name, node.addr))
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

/// 开发链配置：单节点、无 bootnodes、快速出块（需 `dev-chain` feature 编译）。
pub fn dev_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "WASM binary was not built".to_string())?;
    Ok(ChainSpec::builder(wasm_binary, NoExtension::default())
        .with_name(&format!("{} (Dev)", CHAIN_NAME))
        .with_id("dev")
        .with_chain_type(ChainType::Development)
        // 无 bootnodes，单机运行
        .with_genesis_config_patch(genesis_config_presets::mainnet_config_genesis())
        .with_properties(chain_properties())
        .build())
}
