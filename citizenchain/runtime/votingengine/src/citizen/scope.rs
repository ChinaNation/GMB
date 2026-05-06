//! Scope 公民快照验签(Phase 3 实现)。
//!
//! 不同签发方各自维护成员名册与签名公钥:
//! - SFID 系统:按行政区(省/市/县)签快照
//! - 教育局系统:辖区教师/学生
//! - 公司系统:股东名册
//! - ...
//!
//! `PopulationSnapshotVerifier` trait 将扩展为多签发方版本:
//! ```ignore
//! pub trait PopulationSnapshotVerifierV2 {
//!     fn verify_scope(scope: &ScopeMeta, proposal_id: u64) -> bool;
//!     fn binding_in_scope(scope: &ScopeMeta, binding_id: &Hash) -> bool;
//!     fn verify_nonce(provider_id: &[u8], nonce: &[u8], signature: &Signature) -> bool;
//! }
//! ```
//!
//! Phase 1 占位 stub。
