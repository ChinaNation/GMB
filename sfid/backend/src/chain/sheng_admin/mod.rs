//! 中文注释:SFID → 链上 ShengAdmins storage 操作模块(ADR-008 phase45)。
//!
//! ## 职责
//!
//! - 拉链上 3-tier 名册(`query.rs::fetch_roster`,3 槽 main/backup_1/backup_2)
//! - 推链 add backup 公钥(`add_backup.rs`)
//! - 推链 remove backup 公钥(`remove_backup.rs`)
//! - HTTP handler(`handler.rs`):
//!   - 公开 `GET /api/v1/chain/sheng-admin/list?province=AH`(链反向调,无 session)
//!   - session 触发型 add/remove backup
//!
//! ## 推链铁律
//!
//! 全部 4 个 extrinsic 走 `Pays::No`(SFID main 账户零余额下也能成功),
//! `chain/client.rs::submit_immortal_paysno_mock` 封装显式 nonce + immortal +
//! 等 InBestBlock 三件套。phase45 阶段 mock 实现,phase7 切真。
//!
//! ## 1010 错误规避
//!
//! `Pays::No` 标注由 runtime extrinsic 自带,SFID 端不需要额外干预。
//! 如果 phase7 切真后仍遇到 1010,排查方向:
//! 1. extrinsic dispatch info 的 `pays_fee` 字段是否真的是 `Pays::No`
//! 2. SFID main signer 是否在链上已有 account_nonce(零余额账户首次推链时
//!    nonce=0,需要 `system_account_next_index` 主动查)
//! 3. runtime spec_version 是否升过(签名 era / metadata 必须匹配)

pub(crate) mod add_backup;
pub(crate) mod handler;
pub(crate) mod query;
pub(crate) mod remove_backup;
