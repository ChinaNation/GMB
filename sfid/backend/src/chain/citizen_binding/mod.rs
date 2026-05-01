//! 公民身份绑定 chain 交互能力。
//!
//! 当前形态(过渡):admin 在 SFID 后台走绑定流程后,**SFID 推链** `bind_sfid` /
//! `unbind_sfid` extrinsic。等链端补齐 chain pull 模式(wuminapp 自助拉凭证),
//! 这里的 push 实现就可以删,只剩凭证签发函数。
//!
//! 模块组织:
//! - [push] —— `submit_bind_sfid_extrinsic` / `submit_unbind_sfid_extrinsic`
//!   用省级签名密钥推 extrinsic 上链(PoW 三件套:显式 nonce + immortal + 等 InBestBlock)
//! - 凭证签发(`build_bind_credential` / `build_bind_credential_with_province`)
//!   保留在 [`crate::chain::runtime_align`],与其他凭证类型共享 offline 编码工具
//!
//! handler 入口仍在 [`crate::operate::binding`],本模块只负责"和链交互"的部分。

pub(crate) mod push;

pub(crate) use push::{submit_bind_sfid_extrinsic, submit_unbind_sfid_extrinsic};
