//! 私人通信全节点 IM 模块。
//!
//! 本模块只承载 IM 通信能力：密文收件箱、设备授权、KeyPackage 池和
//! 私人节点间投递边界。通信全节点只服务本机用户，禁止变成公共中继、
//! 公共 DHT、公共 rendezvous 或第三方消息仓库。

pub(crate) mod binding;
pub(crate) mod commands;
pub(crate) mod direct;
pub(crate) mod endpoint;
pub(crate) mod envelope;
pub(crate) mod keypackage;
pub(crate) mod mailbox;
pub(crate) mod network;
pub(crate) mod policy;
pub(crate) mod rpc;
