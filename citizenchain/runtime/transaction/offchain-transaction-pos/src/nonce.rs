//! 扫码支付清算体系 Step 1 新增:L3 支付 nonce 辅助。
//!
//! 中文注释:
//! - 每个 L3 账户有一个单调递增的 nonce,防止清算行或中间人重放已签名的意图。
//! - 链上权威存储 `L3PaymentNonce`(在 lib.rs 定义)。
//! - 本模块只提供"消费下一个 nonce"的辅助函数,Step 2 批次清算里调用。
//! - Step 1 本身未暴露调用 nonce 的 extrinsic,但结构先行,避免 Step 2 返工。

use crate::{Config, Error, L3PaymentNonce};

/// 消费 L3 的下一个 nonce。
///
/// 语义:`submitted_nonce` 必须严格等于链上当前值 + 1,否则拒绝。
/// 成功时把 `L3PaymentNonce[payer]` 更新为 `submitted_nonce`。
pub fn consume_nonce<T: Config>(
    payer: &T::AccountId,
    submitted_nonce: u64,
) -> Result<(), Error<T>> {
    let current = L3PaymentNonce::<T>::get(payer);
    let expected = current.checked_add(1).ok_or(Error::<T>::L3NonceOverflow)?;
    if submitted_nonce != expected {
        return Err(Error::<T>::InvalidL3Nonce);
    }
    L3PaymentNonce::<T>::insert(payer, submitted_nonce);
    Ok(())
}

/// 查询 L3 的"下一个应该使用的 nonce"(供 wuminapp RPC 查询)。
///
/// wuminapp 在签名前先问清算行节点拿这个值,避免本地 nonce 与链上错位。
pub fn next_nonce<T: Config>(payer: &T::AccountId) -> u64 {
    L3PaymentNonce::<T>::get(payer).saturating_add(1)
}
