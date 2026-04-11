//! 机构/账户链交互层
//!
//! 中文注释:本文件是新 API 调用链端 `DuoqianManagePow.register_sfid_institution`
//! 的**唯一入口**。当前实现复用 `sheng_admins::institutions` 里现有的
//! `submit_register_sfid_institution_extrinsic`,避免把几百行链交互代码重复写一份。
//!
//! 推链时的 PoW 链三件套(显式 nonce + immortal + 只等 InBestBlock)已经固化
//! 在被复用的函数里,见 ADR-005-sfid-subxt-0.43-pow-chain-quirks。

#![allow(dead_code)]

use crate::login::AdminAuthContext;
use crate::sheng_admins::institutions::{
    submit_register_sfid_institution_extrinsic, ChainInstitutionRegisterReceipt,
};
use crate::AppState;

/// 向链提交 `register_sfid_institution(sfid_id, account_name, ...)`。
///
/// 成功返回链上回执(tx_hash + block_number + 派生的 duoqian_address 需要调用方补算)。
/// 失败返回错误字符串,由 handler 包装成 HTTP 500。
///
/// 任务卡 `20260409-sfid-sheng-admin-per-province-keyring` Phase 1.B 步骤 11：
/// 新增 `ctx` 参数，用于在下游解析本省签名 Pair（`resolve_business_signer`）。
pub async fn submit_register_account(
    state: &AppState,
    ctx: &AdminAuthContext,
    sfid_id: &str,
    account_name: &str,
) -> Result<ChainInstitutionRegisterReceipt, String> {
    // 中文注释:链端的 `name` 字段 = sfid 系统的 account_name(见任务卡 2 背景)。
    submit_register_sfid_institution_extrinsic(state, ctx, sfid_id, account_name).await
}
