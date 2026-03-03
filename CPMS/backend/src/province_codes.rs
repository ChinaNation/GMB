//! 省市代码由编译脚本在编译期从 SFID `sfid-tool` 生成。
//! 默认来源路径：相对当前 crate 的 `../../SFID/backend/src/sfid-tool`。
//! 可通过环境变量覆盖：`CPMS_SFID_TOOL_DIR=/path/to/SFID/backend/src/sfid-tool`。

include!(concat!(env!("OUT_DIR"), "/sfid_province_city.rs"));
