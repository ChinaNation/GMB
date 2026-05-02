//! 中文注释:SFID → 链上 ShengAdmins storage 操作模块(ADR-008 phase7 真实推链)。
//!
//! ## 职责
//!
//! - 拉链上 3-tier 名册(`query.rs::fetch_roster`,3 槽 main/backup_1/backup_2)
//! - 推链 add backup 公钥(`add_backup.rs`,call_index=2)
//! - 推链 remove backup 公钥(`remove_backup.rs`,call_index=3)
//! - HTTP handler(`handler.rs`):
//!   - 公开 `GET /api/v1/chain/sheng-admin/list?province=AH`(链反向调,无 session)
//!   - session 触发型 add/remove backup
//!
//! ## 推链铁律
//!
//! 全部 4 个 extrinsic 走 `Pays::No`(SFID main 账户零余额下也能成功),
//! `chain/client.rs::submit_immortal_paysno` 封装显式 nonce + immortal +
//! 等 InBestBlock 三件套。phase7 已切真,通过裸 SCALE 编码 + V4 BARE 包装路径
//! 提交 unsigned extrinsic。
//!
//! ## 1010 错误规避
//!
//! `Pays::No` 标注由 runtime extrinsic 自带,SFID 端不需要额外干预。
//! 如果运行时仍遇到 1010 InvalidTransaction,排查方向:
//! 1. extrinsic dispatch info 的 `pays_fee` 字段是否真的是 `Pays::No`
//! 2. ValidateUnsigned 验签是否通过(payload 顺序 / domain 常量与链端对齐)
//! 3. nonce 是否已被 `UsedShengNonce` storage 收录(防重放)
//! 4. runtime spec_version 是否升过(metadata 必须与连接节点一致)

pub(crate) mod add_backup;
pub(crate) mod handler;
pub(crate) mod query;
pub(crate) mod remove_backup;
