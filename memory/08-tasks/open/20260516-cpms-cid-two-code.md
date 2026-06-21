任务需求：
按照既有 `CID_CPMS_V1` 协议口径完成第 2 步 CPMS 系统改造：CPMS 只消费 CID 签发的 INSTALL 安装码，安装后直接签发公民档案号二维码 ARCHIVE。协议名不得新增，机构 CID 字段统一使用 `cid_number`。本阶段数据库不用迁移，旧数据允许删除，直接按目标状态调整基准结构。

所属模块：
- CPMS

输入文档：
- `memory/00-vision/project-goal.md`
- `memory/00-vision/trust-boundary.md`
- `memory/01-architecture/repo-map.md`
- `memory/03-security/security-rules.md`
- `memory/07-ai/unified-required-reading.md`
- `memory/07-ai/unified-protocols.md`
- `memory/07-ai/unified-naming.md`
- `memory/05-modules/citizencode/CID-CPMS-QR-v1.md`
- `memory/01-architecture/citizenpassport/CPMS_TECHNICAL.md`
- `memory/05-modules/citizenpassport/backend/initialize/INITIALIZE_TECHNICAL.md`
- `memory/05-modules/citizenpassport/backend/dangan/DANGAN_TECHNICAL.md`

必须遵守：
- 继续使用 `CID_CPMS_V1`，不得新增协议名。
- 机构 CID 字段必须使用 `cid_number`，不得新增同义字段名。
- CPMS 只保留 INSTALL 安装码和 ARCHIVE 档案码两类二维码，不保留旧中间注册流程。
- 档案号必须全局唯一，不得暴露省、市、CPMS 机构号。
- 档案二维码明文不得暴露省、市、CPMS 机构号；归属信息只能放入加密 `geo_seal`，由 CID 验证/解密。
- 本阶段不做数据库迁移兼容，旧库可清空后按新基准结构启动。
- 改代码后必须更新文档、完善中文注释、清理残留。

输出物：
- CPMS 后端代码
- CPMS 前端代码
- CPMS 数据库基准结构
- 中文注释
- 测试或可执行验证
- 文档更新
- 残留清理

验收标准：
- CPMS 能扫码/粘贴 `CID_CPMS_V1 / INSTALL` 安装码完成离线初始化。
- CPMS 安装后自动具备 ARCHIVE 签发能力，不再生成中间注册码。
- CPMS 生成的档案号全局唯一，且明文看不出省、市、CPMS 机构号。
- CPMS 生成的 `CID_CPMS_V1 / ARCHIVE` 二维码能被 CID 第 1 步实现验证。
- 其他 CPMS 或外部人员无法从 ARCHIVE 明文字段看出签发城市。
- 旧中间注册流程残留已清理。
- 文档已同步更新。

执行记录：
- 2026-05-16：完成 CPMS 后端 INSTALL 初始化、ARCHIVE 签发密钥、档案号生成、`geo_seal` 加密和 ARCHIVE 签名改造。
- 2026-05-16：按清库重建口径更新 PostgreSQL 基准结构，删除旧中间注册迁移文件，不做旧数据迁移兼容。
- 2026-05-16：完成 CPMS 前端初始化页、系统设置页、档案创建页、档案详情页两码方案同步。
- 2026-05-16：同步 CPMS 架构文档、模块文档、仓库内 CPMS 技术说明和启动脚本。
- 2026-05-16：按目标协议去掉档案号固定示例前缀，档案号格式调整为 `<26位Base32>-<2位Base32校验>`。
- 2026-05-16：按最终精简协议收口 CPMS INSTALL 消费和 ARCHIVE `geo_seal` 生成，安装码不再消费重复省市码、安装 ID、城市令牌、时间字段和机构名称。
- 2026-05-16：撤销 CPMS 外置公钥验签设计，恢复离线 INSTALL 初始化；档案码真实性由 CID 在 ARCHIVE 验真阶段闭环确认。

验证记录：
- `cd citizenpassport/backend && cargo fmt && cargo check`
- `cd citizenpassport/backend && cargo fmt && cargo test`
- `cd citizenpassport/frontend && npm run build`
- `rg` 检查 CPMS 源码、脚本、当前有效文档和本任务卡中的旧中间注册链路关键词，结果为零命中。
- 2026-05-16：`cd citizenpassport/backend && cargo fmt && cargo check && cargo test` 通过；`rg` 检查当前有效 CID/CPMS 代码和文档中的档案号旧前缀，结果为零命中。
