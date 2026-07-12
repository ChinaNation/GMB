// 钱包 JSON 持久化。
//
// 冷钱包仅保存 SS58 地址和公钥，不存储任何私钥或助记词。
// 签名通过 QR 码协议由外部离线设备完成。

use crate::shared::security;
use serde::{Deserialize, Serialize};
use std::{fs, io::ErrorKind, path::PathBuf};
use tauri::AppHandle;

/// 钱包类型。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum WalletKind {
    /// 本机 powr 矿工密钥派生的热钱包，不写入 cold-wallets.json。
    MinerHot,
    /// 用户手动添加的冷钱包，只保存地址和公钥。
    Cold,
}

impl Default for WalletKind {
    fn default() -> Self {
        Self::Cold
    }
}

fn default_deletable() -> bool {
    true
}

/// 单个钱包条目。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColdWallet {
    pub id: String,
    pub name: String,
    /// 前端签名路径选择：冷钱包走 QR，矿工热钱包走本地 powr 签名。
    #[serde(default)]
    pub kind: WalletKind,
    /// 是否允许从钱包管理列表删除。
    #[serde(default = "default_deletable")]
    pub deletable: bool,
    /// SS58 地址（prefix 2027）。
    pub address: String,
    /// 从 SS58 解出的 32 字节公钥（64 位 hex，无 0x 前缀）。
    pub pubkey_hex: String,
    pub created_at: u64,
}

/// 钱包列表 + 当前激活钱包 ID。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletStore {
    pub wallets: Vec<ColdWallet>,
    pub active_id: Option<String>,
}

impl Default for WalletStore {
    fn default() -> Self {
        Self {
            wallets: Vec::new(),
            active_id: None,
        }
    }
}

fn store_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(security::app_data_dir(app)?.join("cold-wallets.json"))
}

pub fn load(app: &AppHandle) -> Result<WalletStore, String> {
    let path = store_path(app)?;
    let raw = match fs::read_to_string(&path) {
        Ok(v) => v,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(WalletStore::default()),
        Err(e) => return Err(format!("读取钱包文件失败: {e}")),
    };
    serde_json::from_str(&raw).map_err(|e| format!("解析钱包文件失败: {e}"))
}

pub fn save(app: &AppHandle, store: &WalletStore) -> Result<(), String> {
    let raw =
        serde_json::to_string_pretty(store).map_err(|e| format!("序列化钱包数据失败: {e}"))?;
    security::write_text_atomic(&store_path(app)?, &format!("{raw}\n"))
        .map_err(|e| format!("写入钱包文件失败: {e}"))
}
