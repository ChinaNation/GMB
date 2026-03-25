# Runtime Primitives 目录说明

本目录用于承载 CitizenChain runtime 内部的常量、基础类型和运行时组织层内容。

说明：

- 原仓库根目录 `primitives/` Rust crate 已迁入本目录
- runtime、node 与仓库工具脚本统一依赖本目录中的 `primitives` crate
- 制度保留地址、链常量、创世相关基础类型都应继续在本目录维护
- `china/` 目录下的机构常量文件必须显式纳入 `china/mod.rs` 模块树，避免目录中存在未参与编译的残留文件
- `china/` 目录下各机构的多签管理员字段统一命名为 `duoqian_admins`，避免 `admins` 与 `duoqian_admins` 并存造成脚本与 runtime 接线歧义
