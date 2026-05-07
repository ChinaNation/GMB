//! 冻结 chainspec 加载入口。
//!
//! 铁律(memory/feedback_chainspec_frozen.md):
//! - chainspec 在主网创世后**永久冻结**,JSON 字节作为版本库内的固定资产
//! - 二进制启动一律 `from_json_bytes` 加载这份字节,不再 `with_genesis_config_patch`
//!   现编创世,确保任何平台、任何 commit 编出来的 binary genesis_hash 全网一致
//! - runtime 升级走链上 `setCode`(governance/runtime-upgrade),**绝不动这份 JSON**
//! - 不要再加 dev / staging / 多套 chainspec —— 只有 `citizenchain.raw.json` 一份
//!
//! 该 JSON 由主网在线权威节点 `export-chain-spec --raw` 一次性导出
//! (导出时间 2026-05-06,源:nrcgch.crcfrcn.com)。

use sc_chain_spec::NoExtension;

pub type ChainSpec = sc_service::GenericChainSpec<NoExtension>;

// 主网冻结 chainspec(raw)。文件路径相对本文件:
// citizenchain/node/src/core/chain_spec.rs → ../../chainspecs/citizenchain.raw.json
const CHAIN_SPEC_RAW: &[u8] = include_bytes!("../../chainspecs/citizenchain.raw.json");

pub fn chain_config() -> Result<ChainSpec, String> {
    ChainSpec::from_json_bytes(CHAIN_SPEC_RAW.to_vec())
        .map_err(|e| format!("加载冻结 chainspec 失败: {e}"))
}
