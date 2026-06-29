//! onchina CID 运行态入口。
//!
//! 中文注释:
//! runtime primitives 负责 CID 字节协议与常量;onchina 负责 SQLite 行政区、
//! 当前年份、动态 UUID、数据库查重和管理端 API。业务模块只允许从 `crate::cid`
//! 引用 CID 能力,不得恢复顶层 `china` 或 `number` 模块。

#![allow(unused_imports)]

pub(crate) mod admin;
pub mod category;
pub mod china;
pub mod generator;
pub(crate) mod model;
pub mod seed;

pub use category::{InstitutionCategory, SubjectLegalKind, classify, legal_kind};
pub use generator::{GenerateCidInput, generate_cid_number};
pub(crate) use model::*;
pub use primitives::cid::code;
pub use primitives::cid::code::{AdminLevel, InstitutionCode};
pub use primitives::cid::number::{
    CID_NUMBER_SEGMENT_COUNT, CID_NUMBER_SEGMENT_D4_LEN, CID_NUMBER_SEGMENT_K3P1C1_LEN,
    CID_NUMBER_SEGMENT_N9_LEN, CID_NUMBER_SEGMENT_R5_LEN, CidNumberParts, parse_cid_number_parts,
    validate_cid_number_format,
};
pub use seed::{SeedCidError, dynamic_institution_cid, official_institution_cid};
