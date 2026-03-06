//! 省市代码由编译脚本在编译期从 SFID `sfid` 生成。
//! 默认来源路径：相对当前 crate 的 `../../sfid/backend/src/sfid`。
//! 可通过环境变量覆盖：`CPMS_SFID_DIR=/path/to/sfid/backend/src/sfid`。

include!(concat!(env!("OUT_DIR"), "/sfid_province_city.rs"));
