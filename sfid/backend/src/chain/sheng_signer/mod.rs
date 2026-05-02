//! 中文注释:SFID → 链上 `ShengSigningPubkey` storage 操作模块(ADR-008 phase45)。
//!
//! ## 职责
//!
//! - 推链 activate 签名公钥(`activation.rs::activate`):某 admin slot 首次登录
//!   后 bootstrap 出的 sr25519 公钥写入链上
//! - 推链 rotate 签名公钥(`rotation.rs::rotate`):同一 admin 换签名密钥
//!
//! ## 推链铁律
//!
//! 全部走 `Pays::No` + immortal + 显式 nonce + 等 InBestBlock,
//! `chain/client.rs::submit_immortal_paysno_mock` 封装。phase45 mock 实现,
//! phase7 切真。

pub(crate) mod activation;
pub(crate) mod handler;
pub(crate) mod rotation;
