//! 中文注释:SFID → 链上 `ShengSigningPubkey` storage 操作模块(ADR-008 phase7 真实推链)。
//!
//! ## 职责
//!
//! - 推链 activate 签名公钥(`activation.rs::activate`,call_index=4):某 admin slot
//!   首次登录后 bootstrap 出的 sr25519 公钥写入链上
//! - 推链 rotate 签名公钥(`rotation.rs::rotate`,call_index=5):同一 admin 换签名密钥
//!
//! ## 推链铁律
//!
//! 全部走 `Pays::No` + immortal + 显式 nonce + 等 InBestBlock,
//! `chain/client.rs::submit_immortal_paysno` 封装。phase7 已切真,通过裸 SCALE
//! 编码 + V4 BARE 包装路径提交 unsigned extrinsic。

pub(crate) mod activation;
pub(crate) mod handler;
pub(crate) mod rotation;
