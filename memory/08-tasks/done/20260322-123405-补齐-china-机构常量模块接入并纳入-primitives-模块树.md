任务需求：
补齐 `citizenchain/runtime/primitives/china/` 目录下尚未接入模块树的机构常量文件，使其可以被后续业务正式引用，并纳入 `mod.rs` 导出。

所属模块：
- citizenchain/runtime/primitives
- memory/05-modules/citizenchain/runtime/primitives

输入文档：
- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/05-modules/citizenchain/runtime/primitives/README.md
- AGENTS.md

必须遵守：
- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通

输出物：
- china 目录模块树修复
- 缺失结构体与依赖补齐
- 文档更新
- 编译检查

验收标准：
- `china_zf`、`china_lf`、`china_jc`、`china_jy`、`china_sf` 被正式导出
- `primitives` crate 编译通过
- 文档已更新
- 任务卡已回写
