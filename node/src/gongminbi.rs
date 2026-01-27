//! chain_spec.rs — 公民币 GMB 专用 Chain Spec
//! 说明：
//! 本文件仅负责链的创世状态配置，不涉及任何共识逻辑。

use sc_service::ChainType;
use serde_json::json;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

// Runtime 中需要使用的类型
pub type AccountPublic = <sp_runtime::MultiSignature as Verify>::Signer;

fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("静态 seed 生成失败")
        .public()
}

fn account_id_from_pub(pubkey_hex: &str) -> sp_runtime::AccountId32 {
    use sp_core::hexdisplay::HexDisplay;
    let bytes = hex::decode(pubkey_hex.trim_start_matches("0x"))
        .expect("公钥 hex 错误");
    sp_runtime::AccountId32::from(bytes.as_slice())
}

/// 公民币链的基础 ChainSpec 入口，直接使用 JSON 中的 bootNodes
pub fn chain_spec() -> sc_service::GenericChainSpec {
    let boot_nodes = vec![
    "/dns4/nrcgch01.wuminbi.com/tcp/30333/p2p/12D3KooWHepcMGD3h9VC1XNWmrac3pXo63RimV5jhTU2nC2TLAyS",
    "/dns4/prczss01.wuminbi.com/tcp/30333/p2p/12D3KooWPjWNXvCzPv6PPuiGnF3J5uToW3ySfaB7rKkwUrN2CALv",
    "/dns4/prclns02.wuminbi.com/tcp/30333/p2p/12D3KooWD9EpWCRceAQBc5rxq8pMS75ke9ovDyqAF8ZjoVQVD3tt",
    "/dns4/prcgds03.wuminbi.com/tcp/30333/p2p/12D3KooWJKT8iE9guv4wfem1L9Xd91bNC9CTcLmZyRgUuWkpmEqf",
    "/dns4/prcgxs04.wuminbi.com/tcp/30333/p2p/12D3KooWAxCE4TpEkDKibtQBzFtEuTAvxrDp1JXabhXPY7tAp9qx",
    "/dns4/prcfjs05.wuminbi.com/tcp/30333/p2p/12D3KooWJdGUANuEpVCmarfH2gi23GodbbbBBabuw9Eb4raBabt8",
    "/dns4/prchns06.wuminbi.com/tcp/30333/p2p/12D3KooWEhovD6QmFbZZGBS7pkwKZinfGZPCAKvyEGGDqkja8HDa",
    "/dns4/prcyns07.wuminbi.com/tcp/30333/p2p/12D3KooWB7kZKwKEPFDo7DToUeFHeyZCJWXUR1wUN1t6uW7mFr2Z",
    "/dns4/prcgzs08.wuminbi.com/tcp/30333/p2p/12D3KooWC7t4V1Z2aQWS9HikBdXQgXEaTqeZ5YD78cnxtYBDn31M",
    "/dns4/prchns09.wuminbi.com/tcp/30333/p2p/12D3KooWHS6G18ZtqiCGFYxb3CdvXT3Hb3zds8zknuWPCsdkFPPL",
    "/dns4/prcjxs10.wuminbi.com/tcp/30333/p2p/12D3KooWNpANUi6qmJCJXkMzyAMzjf4nY9wUdkAbwcGRJgikSY13",
    "/dns4/prczjs11.wuminbi.com/tcp/30333/p2p/12D3KooWKLAEv8qEicjGX3MF667gqGF8Lf1iEATskv61pRdGaxS4",
    "/dns4/prcjss12.wuminbi.com/tcp/30333/p2p/12D3KooWQqjnQ8wLx6qNX94PoJGZgEJkgyCA3G5ck3zetcpuQp7f",
    "/dns4/prcsds13.wuminbi.com/tcp/30333/p2p/12D3KooWFgD8cFDqherjpiuRkHwHfAcCwaqXcBjTS2G3LkwUBTsq",
    "/dns4/prcsxs14.wuminbi.com/tcp/30333/p2p/12D3KooWQY3DEaJy9wEBE2bQ9gG1B8XByfVaz839jf1ov75kRmD9",
    "/dns4/prchns15.wuminbi.com/tcp/30333/p2p/12D3KooWSkKBEJ2KZXckFhzLvrqqbhpq4PVKeFuWsxdTF7hfzoGc",
    "/dns4/prchbs16.wuminbi.com/tcp/30333/p2p/12D3KooWMXQoZ9F6nxMuoC2ZnzxEKAn4z2qPKAugP2CZFEcXDqkT",
    "/dns4/prchbs17.wuminbi.com/tcp/30333/p2p/12D3KooWS2WYJ9AQ6Y1AKZcKjaHbmCFNkozV7XBBqqDG8kvwsH22",
    "/dns4/prcsxs18.wuminbi.com/tcp/30333/p2p/12D3KooWNr4EWB1PwBANoU9h2FzZXfS78vxDQynLtft3TDWMQ42p",
    "/dns4/prccqs19.wuminbi.com/tcp/30333/p2p/12D3KooWD8qAmRfVPyDn65j8aNLUZ3xKpc4jVVJ2Jdro3LZKJhrY",
    "/dns4/prcscs20.wuminbi.com/tcp/30333/p2p/12D3KooWR831Zp5wr6AXtwo5f6uoLzig1vTq8GtN8PK7AL3A4t1m",
    "/dns4/prcscs20.wuminbi.com/tcp/30333/p2p/12D3KooWR63RRCk3PDbyBEY8zdEB7JXKzqGGrTwx1RgeQUmr28ZH",
    "/dns4/prcgss21.wuminbi.com/tcp/30333/p2p/12D3KooWRKEFiEJGBdK6AdkJb6ei5FJiqSAvEkk4NxGnoT9p5MUS",
    "/dns4/prcbps22.wuminbi.com/tcp/30333/p2p/12D3KooWQZF44Z2U9mT6Q371ULaRLHK9ucTuxPVV8WpaUnw9Q4Ug",
    "/dns4/prcbhs23.wuminbi.com/tcp/30333/p2p/12D3KooWE69n2vS9KqPuXvZPAVRAXwfLcnAfHLz6EDBCD6G8Zqdk",
    "/dns4/prcsjs24.wuminbi.com/tcp/30333/p2p/12D3KooWRQt9MWd8v1F5b8nNksRgvCk7XmMgntxiv6RX12gkY5Dx",
    "/dns4/prcljs25.wuminbi.com/tcp/30333/p2p/12D3KooWGdzag2ekE4JBbcNYNNg3bAJJrqfrZQnsC4uaVavNpmtX",
    "/dns4/prcjls26.wuminbi.com/tcp/30333/p2p/12D3KooWHbuz7D91uDpbEPKLpSSKE9ZVqPSsTXFMewBbYAAxJYc2",
    "/dns4/prclns27.wuminbi.com/tcp/30333/p2p/12D3KooWE8RugcDKrBwxobPzGkVxke4WnGJhi74No53EH7zhaziB",
    "/dns4/prcnxs28.wuminbi.com/tcp/30333/p2p/12D3KooWGdFwKQQoZTyGbKHtq6FcEjmXSWJ4MfdebuM37MXXNV1T",
    "/dns4/prcqhs29.wuminbi.com/tcp/30333/p2p/12D3KooWEL5PTHVD4HEGRcsTxQKWanzW31qSzAGapvwnBsfdTWWS",
    "/dns4/prcahs30.wuminbi.com/tcp/30333/p2p/12D3KooWPC96XCXpuuErd8G7bteNhmvkk6NTPjLtccPCiRwLRGSw",
    "/dns4/prctws31.wuminbi.com/tcp/30333/p2p/12D3KooWQYc1jQZQyaUQC1snk9DHGmydhMdgtJ9LZZ5pbzTciG2J",
    "/dns4/prcxzs32.wuminbi.com/tcp/30333/p2p/12D3KooWNhQUZN2zvX8WTa5SvbyziGvr18qjVNnhygstb8KHQ7Ro",
    "/dns4/prcxjs33.wuminbi.com/tcp/30333/p2p/12D3KooWMbsFaTXiGKXqjEFZjuP5Tp7iU4FFvf3MJoSmGRXDVc69",
    "/dns4/prcxks34.wuminbi.com/tcp/30333/p2p/12D3KooWBczZmptJkbQkX4yx4XP7QXwtJXxZn1We8R4GtbRExUox",
    "/dns4/prcals35.wuminbi.com/tcp/30333/p2p/12D3KooWJKCXsrzLVWLuZVTENBLeLG5F9KcLoeGhdp1tjs8qtk2y",
    "/dns4/prccls36.wuminbi.com/tcp/30333/p2p/12D3KooWMU7y4HSkWdKQYQ15xQC9L33TUkfcMfBgYQtkxMDcos9v",
    "/dns4/prctss37.wuminbi.com/tcp/30333/p2p/12D3KooWG8ZyfEQo7MkkcKqUczQkY1eKVZFvvAeUpz4EPFi8vEoN",
    "/dns4/prchxs38.wuminbi.com/tcp/30333/p2p/12D3KooWBjDquSWFYAjTy5LWxBqYKC453WT8FpoJTVKK6qGk2G4y",
    "/dns4/prckls39.wuminbi.com/tcp/30333/p2p/12D3KooWSeAo5RUTjTX53NmD8Ncv6fXfnkqd461a6FBEGv8szB8N",
    "/dns4/prchts40.wuminbi.com/tcp/30333/p2p/12D3KooWFrXygQG5HZ1buBcrGwe7KYQagNu29ippkUAbLUndxt9v",
    "/dns4/prcrhs41.wuminbi.com/tcp/30333/p2p/12D3KooWBUyRBBAb6QFkJ3obK1bniWNFu4Gk7VoZAAQo7jrQfNCf",
    "/dns4/prcxas42.wuminbi.com/tcp/30333/p2p/12D3KooWC4errbqKaeyDZVjpNmpryUAfbLM8h6CjvyAbkYjzgnne",
    "/dns4/prchjs43.wuminbi.com/tcp/30333/p2p/12D3KooWPciaAo15DT24rXPZK5EUtBdEyotFBhvEdw6d3zBmzVHH"];

    sc_service::GenericChainSpec::from_json_bytes(
        build_raw_spec(boot_nodes).to_string().as_bytes()
    ).expect("ChainSpec JSON 构建失败")
}

/// 创建 JSON 格式 raw chain spec（对应 chain_spec.json）
fn build_raw_spec(boot_nodes: Vec<String>) -> serde_json::Value {
    json!({
        "name": "gongminbi",
        "id": "88711",
        "protocolId": "gongminbi",
        "chainType": "Live",
        "properties": {
            "tokenSymbol": "GMB",
            "tokenDecimals": 2
        },

        "bootNodes": boot_nodes,

        // 远程 Telemetry 可忽略，留空即可
        "telemetryEndpoints": [],

        // ====== 创世状态 ======
        "genesis": {
            "runtime": {
                // ====== 系统代码（WASM runtime code）======
                "system": {
                    "code": "0x1234..."   // 最终 build wasm 后替换此值
                },

                // ====== 创世宣言 ======
                "genesis_declaration": {
                    "Declaration_of_Citizens": "先有人类后有国家，是公民建立国家，国家是公民的国家，是公民治理国家，而不是国家统治公民，公民没有爱国的义务；国家政权的建立其基本原则是保护公民的生命权、自由权、财产权、反抗压迫权和选举与被选举权不受任何的非法侵犯，当国家政权无法保证这一基本原则时，公民有权有义务推翻这个政权，建立一个以保障公民生命权、自由权、财产权、反抗压迫权和选举与被选举权为基本原则的政权。",
                    "Country_Name_and_Wuminism": "中华民族联邦共和国国家名称是基于中华各民族悠久历史与璀璨文化的沉淀，全称为：中华民族联邦共和国，简称为：中华联邦，或中国及中华民国；中华民族联邦共和国是致力于推行五民主义———公民治理国家（民治）、实现民主共和（民主）、保障公民权利（民权）、建设民生社会（民生）、复兴民族文化（民族）———的联邦制共和国"
                },

                // ====== 创世余额 ======
                // 对应 JSON 中的1个国储会和4个省的pallet账户余额
                "balances": [
                ["0x6d6f646c6e726367636830310000000000000000000000000000000000000000", 144349737800],
                ["0x6d6f646c7072627a737330310000000000000000000000000000000000000000", 109139020000],
                ["0x6d6f646c7072626c6e7330320000000000000000000000000000000000000000", 281570640000],
                ["0x6d6f646c70726267647330330000000000000000000000000000000000000000", 1060128640000],
                ["0x6d6f646c70726267787330340000000000000000000000000000000000000000", 501268040000],
                ["0x6d6f646c707262666a7330350000000000000000000000000000000000000000", 415400860000],
                ["0x6d6f646c707262686e7330360000000000000000000000000000000000000000", 100812320000],
                ["0x6d6f646c707262796e7330370000000000000000000000000000000000000000", 468217660000],
                ["0x6d6f646c707262677a7330380000000000000000000000000000000000000000", 385621480000],
                ["0x6d6f646c707262686e7330390000000000000000000000000000000000000000", 664448640000],
                ["0x6d6f646c7072626a787331300000000000000000000000000000000000000000", 451886350000],
                ["0x6d6f646c7072627a6a7331310000000000000000000000000000000000000000", 645675880000],
                ["0x6d6f646c7072626a737331320000000000000000000000000000000000000000", 847480160000],
                ["0x6d6f646c70726273647331330000000000000000000000000000000000000000", 1015274530000],
                ["0x6d6f646c70726273787331340000000000000000000000000000000000000000", 349156160000],
                ["0x6d6f646c707262686e7331350000000000000000000000000000000000000000", 993655190000],
                ["0x6d6f646c70726268627331360000000000000000000000000000000000000000", 562820210000],
                ["0x6d6f646c70726268627331370000000000000000000000000000000000000000", 545435530000],
                ["0x6d6f646c70726273787331380000000000000000000000000000000000000000", 338241010000],
                ["0x6d6f646c70726263717331390000000000000000000000000000000000000000", 320541590000],
                ["0x6d6f646c70726273637332300000000000000000000000000000000000000000", 803102450000],
                ["0x6d6f646c70726267737332310000000000000000000000000000000000000000", 206174650000],
                ["0x6d6f646c70726262707332320000000000000000000000000000000000000000", 218930950000],
                ["0x6d6f646c70726262687332330000000000000000000000000000000000000000", 247208710000],
                ["0x6d6f646c707262736a7332340000000000000000000000000000000000000000", 248708950000],
                ["0x6d6f646c7072626c6a7332350000000000000000000000000000000000000000", 227803540000],
                ["0x6d6f646c7072626a6c7332360000000000000000000000000000000000000000", 240734530000],
                ["0x6d6f646c7072626c6e7332370000000000000000000000000000000000000000", 425914070000],
                ["0x6d6f646c7072626e787332380000000000000000000000000000000000000000", 72026540000],
                ["0x6d6f646c70726271687332390000000000000000000000000000000000000000", 50305420000],
                ["0x6d6f646c70726261687333300000000000000000000000000000000000000000", 610271710000],
                ["0x6d6f646c70726274777333310000000000000000000000000000000000000000", 235612360000],
                ["0x6d6f646c707262787a7333320000000000000000000000000000000000000000", 27638530000],
                ["0x6d6f646c707262786a7333330000000000000000000000000000000000000000", 98804420000],
                ["0x6d6f646c707262786b7333340000000000000000000000000000000000000000", 45130980000],
                ["0x6d6f646c707262616c7333350000000000000000000000000000000000000000", 26279990000],
                ["0x6d6f646c707262636c7333360000000000000000000000000000000000000000", 78330210000],
                ["0x6d6f646c70726274737333370000000000000000000000000000000000000000", 56341640000],
                ["0x6d6f646c70726268787333380000000000000000000000000000000000000000", 46647270000],
                ["0x6d6f646c7072626b6c7333390000000000000000000000000000000000000000", 8934150000],
                ["0x6d6f646c70726268747334300000000000000000000000000000000000000000", 121107800000],
                ["0x6d6f646c70726272687334310000000000000000000000000000000000000000", 154895620000],
                ["0x6d6f646c70726278617334320000000000000000000000000000000000000000", 39910800000],
                ["0x6d6f646c707262686a7334330000000000000000000000000000000000000000", 87384580000]],],

                // ====== 省储行（ShengBank）节点 ======
                // 对应 JSON 中的43个省储行节点信息
                // ===============================================================

}
                "shengbank_nodes": [
                {
                "mutable": false
                "pallet_id" :"prbzss01",
                "node_name" :"中枢省公民储备银行权益节点",
                "pallet_address": "0x6d6f646c7072627a737330310000000000000000000000000000000000000000",
                "keyless_address": "0x21bc9e12d717e4d55666501fd21f8f3fdfbf98d513d6584424f34162397ac1be",
                "admins_prb": [
                    "0x________",
                    "0x________",
                    "0x________"
                    // …… 共 9 个
                },
                ],

                // ====== 储委会节点 ======
                "reserve_nodes": [
                {
                    "mutable": false
                    "pallet_id": "nrcgch01",
                    "node_name": "国家储备委员会权威节点",
                    "pallet_address": "0x6d6f646c6e726367636830310000000000000000000000000000000000000000",
                    "p2p_address": "/dns4/nrcgch01.wuminbi.com/tcp/30333/p2p/2DeusbNnisuEiDuDEiQ1JGAzZVruPDfTaW7wy26NwU7GTc",
                    "admins_nrc": [
                        "0x________",
                        "0x________",
                        "0x________"
                        // …… 共 19 个
                },
                ],
                ],
            },

            // ====== 扩展字段（可选）======
            "raw": {}
        }
    })
}