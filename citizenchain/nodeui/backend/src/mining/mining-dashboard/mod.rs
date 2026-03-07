use crate::{home::home_node, settings::security};
use serde::Serialize;
use serde_json::Value;
use std::hash::Hasher;
use std::{
    collections::{HashMap, VecDeque},
    fs,
    io::{Read, Write},
    net::TcpStream,
    path::PathBuf,
    process::Command,
    sync::{Mutex, OnceLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::AppHandle;

const RPC_ADDR: &str = "127.0.0.1:9944";
const EXPECTED_SS58_PREFIX: u64 = 2027;
const RECENT_RECORD_LIMIT: usize = 20;
const DAY_MS: u64 = 86_400_000;
const MAX_RPC_RESPONSE_BYTES: u64 = 4 * 1024 * 1024;
const RESOURCE_CACHE_TTL_MS: u64 = 5_000;
const INCOME_DAY_KEEP: u64 = 400;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MiningIncome {
    pub total_income: String,
    pub total_fee_income: String,
    pub total_reward_income: String,
    pub today_income: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MiningBlockRecord {
    pub block_height: u64,
    pub timestamp_ms: Option<u64>,
    pub fee: String,
    pub block_reward: String,
    pub author: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResourceUsage {
    pub cpu_percent: Option<f64>,
    pub memory_mb: Option<u64>,
    pub disk_usage_percent: Option<f64>,
    pub node_data_size_mb: Option<u64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MiningDashboard {
    pub income: MiningIncome,
    pub records: Vec<MiningBlockRecord>,
    pub resources: ResourceUsage,
    pub warning: Option<String>,
}

#[derive(Clone, Debug)]
struct CachedBlockRecord {
    block_height: u64,
    timestamp_ms: Option<u64>,
    fee_fen: u128,
    block_reward_fen: u128,
    author: String,
}

#[derive(Clone, Debug, Default)]
struct MiningComputationCache {
    chain_genesis_hash: Option<String>,
    last_processed_height: u64,
    last_processed_hash: Option<String>,
    total_fee_fen: u128,
    total_reward_fen: u128,
    income_by_utc_day: HashMap<u64, u128>,
    recent_records: VecDeque<CachedBlockRecord>,
}

#[derive(Clone, Debug)]
struct ResourceUsageSample {
    sampled_at_ms: u64,
    usage: ResourceUsage,
}

#[derive(Clone, Debug, Default)]
struct RefreshStats {
    fee_query_failures: u64,
    timestamp_query_failures: u64,
}

#[derive(Clone, Debug)]
struct ProcessedBlock {
    height: u64,
    hash: String,
    timestamp_ms: Option<u64>,
    fee_fen: u128,
    fee_query_failures: u32,
    timestamp_query_failed: bool,
    reward_fen: u128,
    author: String,
}

static MINING_CACHE: OnceLock<Mutex<MiningComputationCache>> = OnceLock::new();
static RESOURCE_USAGE_CACHE: OnceLock<Mutex<Option<ResourceUsageSample>>> = OnceLock::new();

fn mining_cache() -> &'static Mutex<MiningComputationCache> {
    MINING_CACHE.get_or_init(|| Mutex::new(MiningComputationCache::default()))
}

fn resource_usage_cache() -> &'static Mutex<Option<ResourceUsageSample>> {
    RESOURCE_USAGE_CACHE.get_or_init(|| Mutex::new(None))
}

fn node_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let data = security::app_data_dir(app)?.join("node-data");
    fs::create_dir_all(&data).map_err(|e| format!("create node data dir failed: {e}"))?;
    Ok(data)
}

fn unix_now_ms() -> Result<u64, String> {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("system time error: {e}"))?
        .as_millis();
    Ok(u64::try_from(ms).unwrap_or(u64::MAX))
}

fn utc_day(ms: u64) -> u64 {
    ms / DAY_MS
}

fn rpc_post(method: &str, params: Value) -> Result<Value, String> {
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params,
    })
    .to_string();

    let req = format!(
        "POST / HTTP/1.1\r\nHost: {RPC_ADDR}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        payload.len(),
        payload
    );

    let addr = RPC_ADDR
        .parse()
        .map_err(|e| format!("parse RPC socket address failed: {e}"))?;
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_millis(600))
        .map_err(|e| format!("RPC 连接失败: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .map_err(|e| format!("set RPC read timeout failed: {e}"))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(3)))
        .map_err(|e| format!("set RPC write timeout failed: {e}"))?;

    stream
        .write_all(req.as_bytes())
        .map_err(|e| format!("RPC 写入失败: {e}"))?;

    let mut response = String::new();
    stream
        .take(MAX_RPC_RESPONSE_BYTES)
        .read_to_string(&mut response)
        .map_err(|e| format!("RPC 读取失败: {e}"))?;

    let Some((header, body)) = response.split_once("\r\n\r\n") else {
        return Err("RPC 响应格式错误：缺少 header/body 分隔符".to_string());
    };
    let status_line = header
        .lines()
        .next()
        .ok_or_else(|| "RPC 响应格式错误：缺少状态行".to_string())?;
    if !status_line.contains(" 200 ") {
        return Err(format!("RPC HTTP 状态异常: {status_line}"));
    }

    let json: Value = serde_json::from_str(body).map_err(|e| format!("RPC JSON 解析失败: {e}"))?;
    if let Some(err) = json.get("error") {
        return Err(format!("RPC 返回错误: {err}"));
    }

    Ok(json.get("result").cloned().unwrap_or(Value::Null))
}

fn ensure_expected_rpc_node() -> Result<(), String> {
    let properties = rpc_post("system_properties", Value::Array(vec![]))?;
    let ss58 = properties
        .get("ss58Format")
        .and_then(|v| {
            if let Some(raw) = v.as_u64() {
                Some(raw)
            } else {
                v.as_str().and_then(|s| s.parse::<u64>().ok())
            }
        })
        .ok_or_else(|| "RPC 节点缺少 ss58Format".to_string())?;
    if ss58 != EXPECTED_SS58_PREFIX {
        return Err(format!("RPC 链前缀不匹配：expected=2027, got={ss58}"));
    }

    let name = rpc_post("system_name", Value::Array(vec![]))?
        .as_str()
        .map(str::trim)
        .unwrap_or("")
        .to_string();
    if name.is_empty() {
        return Err("RPC 节点名称为空".to_string());
    }
    Ok(())
}

fn chain_genesis_hash() -> Result<String, String> {
    rpc_post(
        "chain_getBlockHash",
        Value::Array(vec![Value::String("0x0".to_string())]),
    )?
    .as_str()
    .map(|s| s.to_string())
    .ok_or_else(|| "chain_getBlockHash(0) 返回格式无效".to_string())
}

fn best_block_height() -> Result<u64, String> {
    let header = rpc_post("chain_getHeader", Value::Array(vec![]))?;
    header
        .get("number")
        .and_then(Value::as_str)
        .and_then(hex_to_u64)
        .ok_or_else(|| "chain_getHeader.number 缺失或格式无效".to_string())
}

fn block_hash_by_height(height: u64) -> Result<String, String> {
    rpc_post(
        "chain_getBlockHash",
        Value::Array(vec![Value::String(format!("0x{height:x}"))]),
    )?
    .as_str()
    .map(|s| s.to_string())
    .ok_or_else(|| format!("chain_getBlockHash({height}) 返回格式无效"))
}

fn maybe_block_timestamp_ms(ts_key: &str, block_hash: &str) -> Result<Option<u64>, String> {
    let raw = rpc_post(
        "state_getStorage",
        Value::Array(vec![
            Value::String(ts_key.to_string()),
            Value::String(block_hash.to_string()),
        ]),
    )?;
    Ok(raw.as_str().and_then(scale_u64_from_storage_hex))
}

fn hex_to_u64(hex: &str) -> Option<u64> {
    let trimmed = hex.strip_prefix("0x")?;
    u64::from_str_radix(trimmed, 16).ok()
}

fn hex_to_bytes(hex: &str) -> Option<Vec<u8>> {
    let trimmed = hex.strip_prefix("0x").unwrap_or(hex);
    if trimmed.len() % 2 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(trimmed.len() / 2);
    for i in (0..trimmed.len()).step_by(2) {
        let byte = u8::from_str_radix(&trimmed[i..i + 2], 16).ok()?;
        out.push(byte);
    }
    Some(out)
}

fn scale_u64_from_storage_hex(hex: &str) -> Option<u64> {
    let bytes = hex_to_bytes(hex)?;
    if bytes.len() < 8 {
        return None;
    }
    let mut raw = [0u8; 8];
    raw.copy_from_slice(&bytes[..8]);
    Some(u64::from_le_bytes(raw))
}

fn parse_partial_fee(result: &Value) -> Option<u128> {
    let raw = result.get("partialFee")?;
    if let Some(s) = raw.as_str() {
        return s.parse::<u128>().ok();
    }
    raw.as_u64().map(u128::from)
}

fn twox_128(input: &[u8]) -> [u8; 16] {
    let mut h1 = twox_hash::XxHash64::with_seed(0);
    h1.write(input);
    let mut h2 = twox_hash::XxHash64::with_seed(1);
    h2.write(input);

    let mut out = [0u8; 16];
    out[..8].copy_from_slice(&h1.finish().to_le_bytes());
    out[8..].copy_from_slice(&h2.finish().to_le_bytes());
    out
}

fn timestamp_now_storage_key() -> String {
    let mut key = Vec::with_capacity(32);
    key.extend_from_slice(&twox_128(b"Timestamp"));
    key.extend_from_slice(&twox_128(b"Now"));
    format!("0x{}", hex::encode(key))
}

fn format_2_decimals_fen(amount_fen: u128) -> String {
    let major = amount_fen / 100;
    let minor = amount_fen % 100;
    format!("{major}.{minor:02}")
}

fn block_reward_fen_by_height(height: u64) -> u128 {
    if (1..=9_999_999).contains(&height) {
        999_900
    } else {
        0
    }
}

fn decode_scale_compact_u32_prefix(bytes: &[u8]) -> Option<(usize, usize)> {
    let first = *bytes.first()?;
    match first & 0b11 {
        0b00 => Some((((first >> 2) as usize), 1)),
        0b01 => {
            if bytes.len() < 2 {
                return None;
            }
            let v = u16::from_le_bytes([bytes[0], bytes[1]]) >> 2;
            Some((v as usize, 2))
        }
        0b10 => {
            if bytes.len() < 4 {
                return None;
            }
            let v = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) >> 2;
            Some((v as usize, 4))
        }
        0b11 => {
            let byte_len = ((first >> 2) as usize) + 4;
            if bytes.len() < 1 + byte_len || byte_len > 4 {
                return None;
            }
            let mut v: usize = 0;
            for (i, b) in bytes[1..1 + byte_len].iter().enumerate() {
                v |= (*b as usize) << (8 * i);
            }
            Some((v, 1 + byte_len))
        }
        _ => None,
    }
}

fn author_from_pow_digest_logs(logs: &[Value]) -> Option<String> {
    for log in logs {
        let Some(s) = log.as_str() else {
            continue;
        };
        let bytes = hex_to_bytes(s)?;
        if bytes.len() < 6 {
            continue;
        }
        if bytes[0] != 0x06 || &bytes[1..5] != b"pow_" {
            continue;
        }

        let payload = &bytes[5..];
        let Some((payload_len, prefix_len)) = decode_scale_compact_u32_prefix(payload) else {
            continue;
        };
        if payload.len() < prefix_len + payload_len || payload_len < 32 {
            continue;
        }
        let author = &payload[prefix_len..prefix_len + 32];
        return Some(format!("0x{}", hex::encode(author)));
    }
    None
}

fn block_fee_fen(block_hash: &str, extrinsics: &[Value]) -> (u128, u32) {
    let mut total_fee: u128 = 0;
    let mut failures: u32 = 0;

    for xt in extrinsics {
        let Some(xt_hex) = xt.as_str() else {
            failures = failures.saturating_add(1);
            continue;
        };
        let params = Value::Array(vec![
            Value::String(xt_hex.to_string()),
            Value::String(block_hash.to_string()),
        ]);
        let Ok(v) = rpc_post("payment_queryInfo", params) else {
            failures = failures.saturating_add(1);
            continue;
        };
        let Some(fee) = parse_partial_fee(&v) else {
            failures = failures.saturating_add(1);
            continue;
        };
        total_fee = total_fee.saturating_add(fee);
    }

    (total_fee, failures)
}

fn process_block(height: u64, ts_key: &str) -> Result<ProcessedBlock, String> {
    let block_hash = block_hash_by_height(height)?;
    let block = rpc_post(
        "chain_getBlock",
        Value::Array(vec![Value::String(block_hash.clone())]),
    )
    .map_err(|e| format!("load block {height} failed: {e}"))?;

    let extrinsics = block
        .get("block")
        .and_then(|b| b.get("extrinsics"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let (fee_fen, fee_query_failures) = block_fee_fen(&block_hash, &extrinsics);

    let (timestamp_ms, timestamp_query_failed) = match maybe_block_timestamp_ms(ts_key, &block_hash)
    {
        Ok(v) => (v, false),
        Err(_) => (None, true),
    };

    let logs = block
        .get("block")
        .and_then(|b| b.get("header"))
        .and_then(|h| h.get("digest"))
        .and_then(|d| d.get("logs"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let author = author_from_pow_digest_logs(&logs).unwrap_or_else(|| "未知".to_string());

    Ok(ProcessedBlock {
        height,
        hash: block_hash,
        timestamp_ms,
        fee_fen,
        fee_query_failures,
        timestamp_query_failed,
        reward_fen: block_reward_fen_by_height(height),
        author,
    })
}

fn reset_cache_for_chain(cache: &mut MiningComputationCache, chain_hash: String) {
    *cache = MiningComputationCache::default();
    cache.chain_genesis_hash = Some(chain_hash);
}

fn prune_old_income_days(cache: &mut MiningComputationCache, today_utc: u64) {
    let min_day = today_utc.saturating_sub(INCOME_DAY_KEEP);
    cache.income_by_utc_day.retain(|day, _| *day >= min_day);
}

fn refresh_cache(best_height: u64, today_utc: u64) -> Result<RefreshStats, String> {
    let mut working = {
        let cache = mining_cache()
            .lock()
            .map_err(|_| "acquire mining cache failed".to_string())?;
        cache.clone()
    };

    let chain_hash = chain_genesis_hash()?;
    if working.chain_genesis_hash.as_deref() != Some(chain_hash.as_str()) {
        reset_cache_for_chain(&mut working, chain_hash.clone());
    }

    if working.last_processed_height > best_height {
        reset_cache_for_chain(&mut working, chain_hash.clone());
    }

    if working.last_processed_height > 0 {
        let current_last_hash = block_hash_by_height(working.last_processed_height)?;
        if working.last_processed_hash.as_deref() != Some(current_last_hash.as_str()) {
            reset_cache_for_chain(&mut working, chain_hash.clone());
        }
    }

    let ts_key = timestamp_now_storage_key();
    let mut stats = RefreshStats::default();

    for n in (working.last_processed_height + 1)..=best_height {
        let block = process_block(n, &ts_key)?;
        working.total_fee_fen = working.total_fee_fen.saturating_add(block.fee_fen);
        working.total_reward_fen = working.total_reward_fen.saturating_add(block.reward_fen);

        if let Some(ms) = block.timestamp_ms {
            let day = utc_day(ms);
            let income = block.fee_fen.saturating_add(block.reward_fen);
            let entry = working.income_by_utc_day.entry(day).or_insert(0);
            *entry = entry.saturating_add(income);
        }

        stats.fee_query_failures = stats
            .fee_query_failures
            .saturating_add(block.fee_query_failures as u64);
        if block.timestamp_query_failed {
            stats.timestamp_query_failures = stats.timestamp_query_failures.saturating_add(1);
        }

        working.recent_records.push_front(CachedBlockRecord {
            block_height: block.height,
            timestamp_ms: block.timestamp_ms,
            fee_fen: block.fee_fen,
            block_reward_fen: block.reward_fen,
            author: block.author,
        });
        while working.recent_records.len() > RECENT_RECORD_LIMIT {
            let _ = working.recent_records.pop_back();
        }

        working.last_processed_height = block.height;
        working.last_processed_hash = Some(block.hash);
    }

    prune_old_income_days(&mut working, today_utc);

    let mut cache = mining_cache()
        .lock()
        .map_err(|_| "acquire mining cache failed".to_string())?;
    *cache = working;
    Ok(stats)
}

fn dashboard_from_cache(
    cache: &MiningComputationCache,
    resources: ResourceUsage,
    warning: Option<String>,
    today_utc: u64,
) -> MiningDashboard {
    let today_income_fen = cache
        .income_by_utc_day
        .get(&today_utc)
        .copied()
        .unwrap_or(0);
    let records: Vec<MiningBlockRecord> = cache
        .recent_records
        .iter()
        .map(|row| MiningBlockRecord {
            block_height: row.block_height,
            timestamp_ms: row.timestamp_ms,
            fee: format_2_decimals_fen(row.fee_fen),
            block_reward: format_2_decimals_fen(row.block_reward_fen),
            author: row.author.clone(),
        })
        .collect();

    let total_income_fen = cache.total_fee_fen.saturating_add(cache.total_reward_fen);

    MiningDashboard {
        income: MiningIncome {
            total_income: format_2_decimals_fen(total_income_fen),
            total_fee_income: format_2_decimals_fen(cache.total_fee_fen),
            total_reward_income: format_2_decimals_fen(cache.total_reward_fen),
            today_income: format_2_decimals_fen(today_income_fen),
        },
        records,
        resources,
        warning,
    }
}

fn empty_dashboard(resources: ResourceUsage, warning: Option<String>) -> MiningDashboard {
    MiningDashboard {
        income: MiningIncome {
            total_income: "0.00".to_string(),
            total_fee_income: "0.00".to_string(),
            total_reward_income: "0.00".to_string(),
            today_income: "0.00".to_string(),
        },
        records: vec![],
        resources,
        warning,
    }
}

fn warning_from_stats(stats: &RefreshStats) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();
    if stats.fee_query_failures > 0 {
        parts.push(format!("{} 笔交易手续费估算失败", stats.fee_query_failures));
    }
    if stats.timestamp_query_failures > 0 {
        parts.push(format!(
            "{} 个区块时间戳读取失败",
            stats.timestamp_query_failures
        ));
    }
    if parts.is_empty() {
        None
    } else {
        Some(format!("统计数据部分不完整：{}", parts.join("；")))
    }
}

fn collect_resource_usage(app: &AppHandle) -> ResourceUsage {
    let mut cpu_percent = None;
    let mut memory_mb = None;
    let mut disk_usage_percent = None;
    let mut node_data_size_mb = None;

    if let Ok(status) = home_node::current_status(app) {
        if let Some(pid) = status.pid {
            if let Ok(out) = Command::new("ps")
                .args(["-p", &pid.to_string(), "-o", "%cpu=,rss="])
                .output()
            {
                if out.status.success() {
                    let line = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        cpu_percent = parts[0].parse::<f64>().ok();
                        memory_mb = parts[1]
                            .parse::<u64>()
                            .ok()
                            .map(|kb| kb.saturating_add(1023) / 1024);
                    }
                }
            }
        }
    }

    if cpu_percent.is_none() || memory_mb.is_none() {
        #[cfg(target_os = "macos")]
        {
            if let Ok(out) = Command::new("top").args(["-l", "1", "-n", "0"]).output() {
                if out.status.success() {
                    let text = String::from_utf8_lossy(&out.stdout);
                    for line in text.lines() {
                        if cpu_percent.is_none() && line.contains("CPU usage:") {
                            if let Some(idle_idx) = line.find("% idle") {
                                let prefix = &line[..idle_idx];
                                let token = prefix
                                    .split_whitespace()
                                    .last()
                                    .unwrap_or("")
                                    .trim_end_matches('%');
                                if let Ok(idle) = token.parse::<f64>() {
                                    cpu_percent = Some((100.0 - idle).max(0.0));
                                }
                            }
                        }
                        if memory_mb.is_none()
                            && line.contains("PhysMem:")
                            && line.contains(" used")
                        {
                            let used_token = line
                                .split("PhysMem:")
                                .nth(1)
                                .map(str::trim)
                                .and_then(|s| s.split_whitespace().next())
                                .unwrap_or("");
                            if !used_token.is_empty() {
                                let (num_str, unit) =
                                    used_token.split_at(used_token.len().saturating_sub(1));
                                if let Ok(n) = num_str.parse::<f64>() {
                                    memory_mb = match unit {
                                        "T" => Some((n * 1024.0 * 1024.0).round() as u64),
                                        "G" => Some((n * 1024.0).round() as u64),
                                        "M" => Some(n.round() as u64),
                                        "K" => Some((n / 1024.0).round() as u64),
                                        _ => None,
                                    };
                                }
                            }
                        }
                        if cpu_percent.is_some() && memory_mb.is_some() {
                            break;
                        }
                    }
                }
            }
        }
    }

    if let Ok(data_dir) = node_data_dir(app) {
        if let Ok(out) = Command::new("du")
            .args(["-sk", &data_dir.display().to_string()])
            .output()
        {
            if out.status.success() {
                let line = String::from_utf8_lossy(&out.stdout);
                if let Some(first) = line.split_whitespace().next() {
                    node_data_size_mb = first
                        .parse::<u64>()
                        .ok()
                        .map(|kb| kb.saturating_add(1023) / 1024);
                }
            }
        }

        if let Ok(out) = Command::new("df")
            .args(["-k", &data_dir.display().to_string()])
            .output()
        {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                if let Some(line) = text.lines().nth(1) {
                    for part in line.split_whitespace() {
                        if let Some(v) = part.strip_suffix('%') {
                            if let Ok(p) = v.parse::<f64>() {
                                disk_usage_percent = Some(p);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    ResourceUsage {
        cpu_percent,
        memory_mb,
        disk_usage_percent,
        node_data_size_mb,
    }
}

fn resource_usage(app: &AppHandle) -> ResourceUsage {
    let now_ms = unix_now_ms().unwrap_or(0);
    let cache = resource_usage_cache();
    if let Ok(mut guard) = cache.lock() {
        if let Some(sample) = guard.as_ref() {
            if now_ms.saturating_sub(sample.sampled_at_ms) <= RESOURCE_CACHE_TTL_MS {
                return sample.usage.clone();
            }
        }
        let usage = collect_resource_usage(app);
        *guard = Some(ResourceUsageSample {
            sampled_at_ms: now_ms,
            usage: usage.clone(),
        });
        return usage;
    }
    collect_resource_usage(app)
}

#[tauri::command]
pub fn get_mining_dashboard(app: AppHandle) -> Result<MiningDashboard, String> {
    let resources = resource_usage(&app);
    let today_utc = utc_day(unix_now_ms().unwrap_or(0));

    if let Err(err) = ensure_expected_rpc_node() {
        let warning = format!("挖矿统计不可用：{err}");
        eprintln!("{warning}");
        return Ok(empty_dashboard(resources, Some(warning)));
    }

    let best_height = match best_block_height() {
        Ok(v) => v,
        Err(err) => {
            let warning = format!("读取最新区块高度失败：{err}");
            eprintln!("{warning}");
            return Ok(empty_dashboard(resources, Some(warning)));
        }
    };

    let warning_from_refresh = match refresh_cache(best_height, today_utc) {
        Ok(stats) => warning_from_stats(&stats),
        Err(err) => {
            let warning = format!("刷新挖矿统计失败，返回最近缓存：{err}");
            eprintln!("{warning}");
            let cache = mining_cache()
                .lock()
                .map_err(|_| "acquire mining cache failed".to_string())?;
            return Ok(dashboard_from_cache(
                &cache,
                resources,
                Some(warning),
                today_utc,
            ));
        }
    };

    let cache = mining_cache()
        .lock()
        .map_err(|_| "acquire mining cache failed".to_string())?;
    Ok(dashboard_from_cache(
        &cache,
        resources,
        warning_from_refresh,
        today_utc,
    ))
}
