use clap::{Parser, Subcommand};
use primitives::{
    genesis::GENESIS_ISSUANCE, china::china_cb::CHINA_CB,
    china::china_ch::CHINA_CH,
};
use serde_json::{json, Value};
use sp_core::{blake2_128, twox_128};
use std::{
    error::Error,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

#[derive(Parser, Debug)]
#[command(name = "scripts")]
#[command(about = "GMB 本地脚本工具")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 启动后自动验收创世发行/创立发行
    GenesisAudit(GenesisAuditArgs),
}

#[derive(Parser, Debug)]
struct GenesisAuditArgs {
    /// RPC 地址（例如 http://127.0.0.1:9944）
    #[arg(long, default_value = "http://127.0.0.1:9944")]
    rpc_url: String,

    /// 自动启动本地节点（结束后自动关闭），不传则默认连接现有节点
    #[arg(long, action = clap::ArgAction::SetTrue)]
    start_node: bool,

    /// 启动节点时使用的 RPC 端口
    #[arg(long, default_value_t = 9944)]
    rpc_port: u16,

    /// 等待节点就绪超时时间（秒）
    #[arg(long, default_value_t = 600)]
    timeout_secs: u64,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    match cli.command {
        Commands::GenesisAudit(args) => run_genesis_audit(args)?,
    }
    Ok(())
}

fn run_genesis_audit(args: GenesisAuditArgs) -> Result<(), Box<dyn Error>> {
    // 中文注释：可选自动启动节点，便于一键验收。
    let mut child = if args.start_node {
        Some(spawn_node(args.rpc_port)?)
    } else {
        None
    };

    let audit_result = do_audit(&args.rpc_url, args.timeout_secs);

    // 中文注释：无论校验成功或失败，都确保把脚本拉起的节点关闭。
    if let Some(mut c) = child.take() {
        let _ = c.kill();
        let _ = c.wait();
    }

    audit_result
}

fn spawn_node(rpc_port: u16) -> Result<Child, Box<dyn Error>> {
    let mut cmd = Command::new("cargo");
    let child = cmd
        .arg("run")
        .arg("--manifest-path")
        .arg("citizenchain/Cargo.toml")
        .arg("-p")
        .arg("node")
        .arg("--")
        .arg("--chain")
        .arg("mainnet")
        .arg("--tmp")
        .arg("--rpc-external")
        .arg("--unsafe-rpc-external")
        .arg("--rpc-port")
        .arg(rpc_port.to_string())
        .arg("--rpc-methods")
        .arg("Unsafe")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(child)
}

fn do_audit(rpc_url: &str, timeout_secs: u64) -> Result<(), Box<dyn Error>> {
    wait_rpc_ready(rpc_url, timeout_secs)?;

    let nrc = CHINA_CB.first().ok_or("未找到国储会节点")?;
    let nrc_balance = read_free_balance(rpc_url, &nrc.duoqian_address)?;
    if nrc_balance != GENESIS_ISSUANCE {
        return Err(format!(
            "国储会创世发行不匹配: on-chain={}, expected={}",
            nrc_balance, GENESIS_ISSUANCE
        )
        .into());
    }

    let mut shengbank_sum: u128 = 0;
    for bank in CHINA_CH {
        let onchain = read_free_balance(rpc_url, &bank.keyless_address)?;
        if onchain != bank.stake_amount {
            return Err(format!(
                "省储行 {} 创立发行不匹配: on-chain={}, expected={}",
                bank.shenfen_id, onchain, bank.stake_amount
            )
            .into());
        }
        shengbank_sum = shengbank_sum.saturating_add(onchain);
    }

    let expected_total = GENESIS_ISSUANCE.saturating_add(
        CHINA_CH
            .iter()
            .map(|b| b.stake_amount)
            .fold(0u128, |acc, v| acc.saturating_add(v)),
    );
    let onchain_total = nrc_balance.saturating_add(shengbank_sum);
    if onchain_total != expected_total {
        return Err(format!(
            "总额不匹配: on-chain={}, expected={}",
            onchain_total, expected_total
        )
        .into());
    }

    println!("创世验收通过");
    println!("nrc_genesis_issuance={}", nrc_balance);
    println!("shengbank_stake_sum={}", shengbank_sum);
    println!("total_checked={}", onchain_total);
    Ok(())
}

fn wait_rpc_ready(rpc_url: &str, timeout_secs: u64) -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(timeout_secs) {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "chain_getFinalizedHead",
            "params": []
        });
        if rpc_post(rpc_url, &body).is_ok() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(500));
    }
    Err(format!("RPC 未就绪: {} (timeout={}s)", rpc_url, timeout_secs).into())
}

fn read_free_balance(rpc_url: &str, account: &[u8; 32]) -> Result<u128, Box<dyn Error>> {
    let storage_key = system_account_storage_key(account);
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "state_getStorage",
        "params": [format!("0x{}", hex_encode(&storage_key))]
    });
    let resp = rpc_post(rpc_url, &body)?;
    let data_hex = resp
        .get("result")
        .and_then(Value::as_str)
        .ok_or("state_getStorage 返回为空")?;
    let bytes = hex_decode_strip_0x(data_hex)?;
    decode_free_from_account_info(&bytes)
}

fn system_account_storage_key(account: &[u8; 32]) -> Vec<u8> {
    // 中文注释：System.Account 的 storage key 规则：
    // Twox128("System") ++ Twox128("Account") ++ Blake2_128(account) ++ account
    let mut out = Vec::with_capacity(16 + 16 + 16 + 32);
    out.extend_from_slice(&twox_128(b"System"));
    out.extend_from_slice(&twox_128(b"Account"));
    out.extend_from_slice(&blake2_128(account));
    out.extend_from_slice(account);
    out
}

fn decode_free_from_account_info(bytes: &[u8]) -> Result<u128, Box<dyn Error>> {
    // 中文注释：AccountInfo 前 16 字节为 nonce/consumers/providers/sufficients（u32*4），
    // 紧接着 AccountData.free（u128 little-endian）。
    const MIN_LEN: usize = 16 + 16;
    if bytes.len() < MIN_LEN {
        return Err(format!("AccountInfo 长度异常: {}", bytes.len()).into());
    }
    let free_bytes: [u8; 16] = bytes[16..32]
        .try_into()
        .map_err(|_| "free bytes 长度错误")?;
    Ok(u128::from_le_bytes(free_bytes))
}

fn rpc_post(rpc_url: &str, body: &Value) -> Result<Value, Box<dyn Error>> {
    // 中文注释：使用系统 curl，避免新增网络依赖导致离线环境编译失败。
    let output = Command::new("curl")
        .arg("-sS")
        .arg("-H")
        .arg("Content-Type: application/json")
        .arg("-d")
        .arg(body.to_string())
        .arg(rpc_url)
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "curl 调用失败: status={:?}, stderr={}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    let v: Value = serde_json::from_slice(&output.stdout)?;
    if let Some(err) = v.get("error") {
        return Err(format!("RPC error: {}", err).into());
    }
    Ok(v)
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

fn hex_decode_strip_0x(input: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let s = input.strip_prefix("0x").unwrap_or(input);
    if s.len() % 2 != 0 {
        return Err("hex 长度必须为偶数".into());
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    let bytes = s.as_bytes();
    for i in (0..bytes.len()).step_by(2) {
        let hi = from_hex_nibble(bytes[i])?;
        let lo = from_hex_nibble(bytes[i + 1])?;
        out.push((hi << 4) | lo);
    }
    Ok(out)
}

fn from_hex_nibble(c: u8) -> Result<u8, Box<dyn Error>> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err(format!("非法 hex 字符: {}", c as char).into()),
    }
}
