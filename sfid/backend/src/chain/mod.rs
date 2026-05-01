//! `chain/` 是 SFID 后端 ↔ 区块链 交互能力的统一入口。
//!
//! ## 架构铁律(2026-05-01 立)
//!
//! 1. **链需要 SFID 数据时主动 HTTP pull;SFID 自身独立维护数据,不再读链。**
//! 2. **过渡期保留**:`citizen_binding` / `key_admins` / `sheng_admin` 三处仍含
//!    "SFID 主动推 extrinsic"代码——这是上一阶段的产物,与铁律不符,只是为了
//!    避免一次性破坏 admin 后台业务而暂未删除。每处都标注了"过渡态保留",
//!    后续配套链端 chain pull 路径就绪后再下架。
//!
//! ## 二级目录(7 个业务功能 + 3 个共享文件)
//!
//! - [institution_info] · 链/钱包 pull SFID 机构信息(含清算行)
//! - [joint_vote]       · 联合投票:获取公民人数快照凭证
//! - [citizen_binding]  · 公民身份绑定(过渡:含 admin push extrinsic)
//! - [citizen_vote]     · 公民投票凭证签发
//! - [key_admins]       · key-admins 推链(过渡:rotate / sheng_signing / state 查)
//! - [sheng_admin]      · 省级管理员链上 signing pubkey 清理(过渡)
//! - [balance]          · admin 后台主账户链上余额展示
//!
//! 共享文件:
//! - [url]            · `SFID_CHAIN_WS_URL` 环境变量入口
//! - [runtime_align]  · SCALE 编码 / 域常量 / genesis_hash 缓存,所有凭证签发函数也在此
//!
//! ## 删除清单(本次重构一并下架的"无 caller dead code")
//!
//! - `chain/vote.rs::verify_vote_eligibility`(0 caller)
//! - `chain/voters.rs::chain_voters_count`(与 `joint_vote` 同义重复,0 caller)
//! - `chain/binding.rs::chain_binding_validate / chain_reward_*`(0 caller)
//! - `app_core/http_security.rs::attestor_public_key`(0 caller)
//! - `institutions/handler.rs::sync_institution_chain_state`(0 caller,SFID 不再读链)
//! - `chain/clearing_bank_watcher.rs`(SFID 不再读链)

pub(crate) mod balance;
pub(crate) mod citizen_binding;
pub(crate) mod citizen_vote;
pub(crate) mod institution_info;
pub(crate) mod joint_vote;
pub(crate) mod key_admins;
pub(crate) mod runtime_align;
pub(crate) mod sheng_admin;
pub(crate) mod url;
