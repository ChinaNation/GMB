//! CPMS Store 边界。
//!
//! 中文注释:CPMS 是离线 DB-first 系统,这里的 Store 只封装 PostgreSQL 操作,
//! 不引入 SFID 那种全局内存状态树。

pub(crate) mod db;

pub(crate) use db::StoreDb;
