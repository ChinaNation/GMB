任务需求：
按照既有 `CID_CPMS_V1` 协议口径，先完成第 1 步 CID 系统改造：CID 生成 CPMS 安装码，管理 CPMS 授权，验证 CPMS 档案二维码，并把档案号归入省市。协议名不得新增，机构 CID 字段统一使用 `cid_number`。

所属模块：
- CID

输入文档：
- `memory/00-vision/project-goal.md`
- `memory/00-vision/trust-boundary.md`
- `memory/01-architecture/repo-map.md`
- `memory/03-security/security-rules.md`
- `memory/07-ai/unified-required-reading.md`
- `memory/07-ai/unified-protocols.md`
- `memory/07-ai/unified-naming.md`
- `memory/05-modules/citizencode/CID-CPMS-QR-v1.md`
- `memory/05-modules/citizencode/backend/BACKEND_LAYOUT.md`
- `memory/05-modules/citizencode/frontend/FRONTEND_LAYOUT.md`
- `memory/07-ai/module-definition-of-done/citizencode.md`

必须遵守：
- 继续使用 `CID_CPMS_V1`，不得新增协议名。
- 机构 CID 字段必须使用 `cid_number`，不得新增同义字段名。
- 本阶段只改 CID 系统和相关文档，不改 CPMS 代码。
- CID 不保存原始实名数据。
- 档案二维码明文不得暴露省、市、CPMS 机构号；归属信息只能由 CID 侧验证/解密后落库。
- 改代码后必须更新文档、完善中文注释、清理残留。

输出物：
- CID 后端代码
- CID 前端代码
- 中文注释
- 测试或可执行验证
- 文档更新
- 残留清理

验收标准：
- CID 能生成 `CID_CPMS_V1 / INSTALL` 安装码。
- CID 能按安装授权管理 CPMS 的 `PENDING / ACTIVE / DISABLED / REVOKED` 状态。
- CID 能验证 `CID_CPMS_V1 / ARCHIVE` 档案二维码并按省市归档。
- 非 CID 授权的伪 CPMS 档案二维码不能通过认证。
- 重复档案号被拒绝。
- 文档已同步更新。
- 残留已清理。
- 已对照 CID 模块完成标准。

执行记录：
- 2026-05-16：完成 CID 后端 CPMS 安装授权模型、INSTALL 签发、ARCHIVE 验真、档案导入和公民绑定复用验真入口。
- 2026-05-16：完成 CID 前端 CPMS 授权面板和公安局机构详情页两码方案调整。
- 2026-05-16：删除 CID 侧旧中间注册弹窗、后端旧注册路由、旧盲签依赖和旧残留状态字段。
- 2026-05-16：同步 `CID_CPMS_V1` 协议文档、统一协议登记、CID 后端模块文档和相关边界说明。
- 2026-05-16：按最终精简协议收口 CID INSTALL 生成和 ARCHIVE 验真，删除对安装码旧字段和 `geo_seal` 旧字段的依赖。
- 2026-05-16：严格协议复查后移除 ARCHIVE 旧类型字段别名和安装授权过期状态残留。

验证记录：
- `cd citizencode/backend && cargo fmt && cargo check`
- `cd citizencode/backend && cargo test`
- `cd citizencode/frontend && npm run build`
- `rg` 检查 CID 源码、CID 当前有效文档和 open 任务卡中的旧中间注册链路关键词，结果为零命中。

备注：
- 本阶段按任务边界未改 CPMS 代码；CPMS 侧 INSTALL 消费和 ARCHIVE 签发属于第 2 步。
