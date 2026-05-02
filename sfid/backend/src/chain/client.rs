//! 中文注释:SFID → 链上 push extrinsic 共享 helper(ADR-008 phase45)。
//!
//! ## 背景
//!
//! ADR-008 决议:SFID 仅 push 4 个 `Pays::No` extrinsic
//! (add_sheng_admin_backup / remove_sheng_admin_backup /
//! activate_sheng_signing_pubkey / rotate_sheng_signing_pubkey)。
//!
//! 推链三件套(见 `feedback_sfid_pow_chain_recipe.md`):
//! 1. 显式 nonce(避免 mortal era 漂移)
//! 2. immortal extrinsic(SFID 推链不绑 mortality)
//! 3. 等 InBestBlock(只确认 block 包含,不等 finalized;PoW 链 finalized 滞后)
//! 4. `Pays::No` 转换为 1010 错误的 fail-fast 友好提示
//!
//! ## 当前状态(phase45)
//!
//! 所有调用 **mock 实现**:函数体只 emit `tracing::warn!("chain push mocked,
//! awaiting Step 2")`,然后返回稳定的 [`MockTxHash`]。等 Step 2 链上 4 个
//! extrinsic 上线后,phase7 子卡只动本文件内部实现,业务方调用点不变。
//!
//! ## 与 `chain/runtime_align.rs` 的关系
//!
//! `runtime_align.rs` 负责 **凭证签名 / SCALE 编码 / genesis_hash 缓存**
//! (chain pull 凭证,SFID main signer)。本文件负责 **chain push 推 extrinsic**
//! (SFID → 链上写入 ShengAdmins / ShengSigningPubkey)。两者职责不重叠;
//! 共享需求(subxt OnlineClient 单例)等 phase7 真正接通时再决定是否合并。

#![allow(dead_code)]

use std::fmt;

/// 推链返回的稳定 TxHash 占位类型(mock 阶段)。
///
/// Phase 7 切真:此类型替换为 `subxt::utils::H256` 或 0x hex `String`,
/// 调用点(handler)不需要重新匹配字段。
#[derive(Debug, Clone)]
pub(crate) struct MockTxHash {
    /// 0x 前缀 + 64 hex(64 字节 hex 字符)。mock 阶段返回常量值,
    /// 仅用于让前端有可观察的"已发送"标识。
    pub(crate) hex: String,
}

impl fmt::Display for MockTxHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.hex)
    }
}

impl MockTxHash {
    pub(crate) fn placeholder() -> Self {
        // 中文注释:固定 32 字节占位 hash,前端可识别"mock 推链产物"。
        Self {
            hex: "0x000000000000000000000000000000000000000000000000000000000000beef".to_string(),
        }
    }
}

/// 推链调用统一错误。
#[derive(Debug)]
pub(crate) enum ChainPushError {
    /// extrinsic 还没上线(Step 2 联调前的占位)。
    NotImplemented(&'static str),
    /// 真实推链阶段:1010 InvalidTransaction(余额 / nonce / fee 校验失败)。
    InvalidTx(String),
    /// 其它链端错误。
    Other(String),
}

impl fmt::Display for ChainPushError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChainPushError::NotImplemented(name) => {
                write!(f, "chain push extrinsic `{name}` not implemented (Step 2 pending)")
            }
            ChainPushError::InvalidTx(s) => write!(f, "invalid tx: {s}"),
            ChainPushError::Other(s) => write!(f, "chain push error: {s}"),
        }
    }
}

impl std::error::Error for ChainPushError {}

/// 中文注释:SFID 推链三件套封装 helper(mock 实现)。
///
/// 真实实现(phase7)行为:
/// 1. 取 subxt `OnlineClient` 单例(若 `runtime_align.rs` 已建,则共用;否则在
///    本文件初始化一次)
/// 2. 用 SFID main signer(`SFID_SIGNING_SEED_HEX` 派生)按 `signer` 入参签名
/// 3. 显式 nonce(`system_account_next_index`)+ immortal era
/// 4. `Pays::No`(由 runtime extrinsic 标注,SFID 端不需要额外操作)
/// 5. `submit_and_watch` → 等 `TxStatus::InBestBlock`
/// 6. 1010 InvalidTransaction → 转 [`ChainPushError::InvalidTx`]
///
/// 当前 mock 行为:emit warn 日志 + 返回稳定 [`MockTxHash`]。
///
/// `extrinsic_label` 用于 warn 日志可观察(grep `chain push mocked` 可定位)。
pub(crate) async fn submit_immortal_paysno_mock(
    extrinsic_label: &'static str,
) -> Result<MockTxHash, ChainPushError> {
    tracing::warn!(
        extrinsic = extrinsic_label,
        "chain push mocked, awaiting Step 2"
    );
    Ok(MockTxHash::placeholder())
}
