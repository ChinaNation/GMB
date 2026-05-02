//! 中文注释:RSA 盲签匿名凭证子模块。
//!
//! 由 `sheng_admins/institutions.rs`(公安局机构 SFID 生成流程)与 `operate/binding.rs`
//! (CPMS 绑定 QR4 校验)调用,启动初始化由 `main.rs` 触发。
//! 历史上放在 `key-admins/` 与 KEY_ADMIN 角色无关,phase23b 子卡(任务卡
//! `20260501-sfid-step1-phase23b-rsa-blind-relocate.md`)将其搬到此处归位。

pub mod rsa_blind;
