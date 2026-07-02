//! CID 协议与内置常量唯一入口。
//!
//!
//! - `code` 维护国家码、省级行政区码、机构码的唯一真源。
//! - `number` / `generator` / `seed` 维护 CID 号格式、核心生成与确定性种子协议。
//! - `china` 维护链上内置中国机构常量,不是 registry 的 SQLite 行政区运行库。

pub mod china;
pub mod code;
pub mod generator;
pub mod number;
pub mod seed;
