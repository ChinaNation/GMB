//! 链余额查询请求/响应 DTO。

use serde::{Deserialize, Serialize};

/// 查询参数:32 字节公钥 hex(允许 0x 前缀)。
#[derive(Debug, Deserialize)]
pub(crate) struct ChainBalanceQuery {
    pub(crate) account_pubkey: String,
}

/// 响应:原始最小单位(分) + 友好元字符串。
#[derive(Debug, Serialize)]
pub(crate) struct ChainBalanceOutput {
    /// 32 字节公钥 hex(与请求一致)。
    pub(crate) account_pubkey: String,
    /// 链上 free 余额(最小单位:分)。
    pub(crate) balance_min_units: String,
    /// 显示用文本,1 元 = 100 分,保留两位小数。
    pub(crate) balance_text: String,
    /// 单位标签(始终为 "元")。
    pub(crate) unit: &'static str,
}
