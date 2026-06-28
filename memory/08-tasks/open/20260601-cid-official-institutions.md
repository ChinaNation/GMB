任务需求：
CID 公权机构目录统一按公民宪法、citizenchain 宪法机构常量与 CID 工具行政区划自动生成，CID 工具行政区划是唯一真源；机构唯一身份只使用不可变 `cid_number`，不得新增 `identity_key` 或 `generation_key`。手动新增只保留教育委员会 `JY` 类型学校机构，也就是注册“学校”这个机构本体，不注册学校内部的校教育委员会组织，不设置所属学校 CID。机构状态收口为未注册、已注册、已注销三态，不再保留第四状态。前后端完成后必须更新文档、完善中文注释并清理残留。

所属模块：
CID

输入文档：
- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/unified-required-reading.md
- memory/07-ai/workflow.md
- memory/07-ai/context-loading-order.md
- memory/07-ai/document-boundaries.md
- memory/07-ai/unified-naming.md
- memory/07-ai/unified-protocols.md
- memory/07-ai/module-checklists/citizencode.md
- memory/07-ai/module-definition-of-done/citizencode.md
- memory/05-modules/citizencode/backend/subjects/SUBJECTS_TECHNICAL.md
- memory/05-modules/citizencode/backend/number/NUMBER_TECHNICAL.md
- memory/05-modules/citizencode/backend/store/STORE_TECHNICAL.md
- memory/05-modules/citizencode/frontend/FRONTEND_LAYOUT.md

必须遵守：
- 不可突破 CID 模块边界，不恢复 `backend/src/`、独立 `backend/chain/`、独立 `frontend/chain/` 或独立 `frontend/api/`。
- CID 不保存原始实名数据。
- 机构唯一身份只认 `cid_number`，不得新增第二套机构身份键。
- 普通公权机构不得手动新增；只有教育委员会 `JY` 类型学校机构可以由市注册局机构管理员人工注册。
- 注册局不进入公权机构目录，继续由现有注册局 tab 管理。
- CPOL 市公安局不再保留独立生成逻辑,统一并入普通市级公权机构模板。
- CID 工具行政区划是自动公权机构目录唯一行政区划真源。
- 宪法常量中已有 `cid_number` 的机构直接使用常量值，不重新生成。
- 不涉及投票流程，不得在机构模块实现或复刻投票逻辑。
- 不清楚逻辑时先沟通。

输出物：
- 后端代码
- 前端代码
- 中文注释
- 测试或验证记录
- 文档更新
- 残留清理

实施要点：
- 后端机构状态收口为 `NOT_REGISTERED / REGISTERED / REVOKED_ON_CHAIN`。
- 机构来源字段只保留自动目录来源 `AUTO`；手动学校机构 `source=null / institution_level=null`。
- 教育委员会 `JY` 手动新增语义是学校机构本体，不设置学校级层级，不设置所属学校 CID。
- 普通公权机构手动创建入口后端必须拒绝。
- 前端公权机构新增入口按钮文案为“新增”。
- 公权机构和私权机构新增选项均显示“教育委员会 (JY)”；选择 `JY` 时全称字段显示为“学校全称”。

后续单独任务提醒：
- 公民宪法第 52 条后续需要删除“国家注册局、国家新闻局”，保留“国家安全局、国家情报局和国家人事局”。
- 区块链软件公民宪法 tab 存在缺陷：点击左侧目录中的第 xx 条时，整个页面变成白板；后续应在 citizenchain 任务中处理。

预计修改目录：
- citizenchain/registry/src/gov/：调整公权自动目录和确定性列表接口，涉及代码、中文注释和残留清理。
- citizencode/backend/private/：调整学校机构创建和私权机构创建规则，涉及代码、中文注释和残留清理。
- citizencode/backend/subjects/：调整机构共享模型、状态、详情和链端公开查询，涉及代码、中文注释和残留清理。
- citizencode/backend/number/：调整 CID 生成规则和机构选项说明，允许教育委员会 `JY` 对应学校机构，涉及代码和中文注释。
- citizencode/backend/store/：如共享 Store 序列化字段需要补充，限制在现有模型边界内修改。
- citizencode/frontend/gov/、citizencode/frontend/private/、citizencode/frontend/core/institution/：调整新增入口、选项、状态文案、公共表单和 API 类型，涉及前端代码。
- memory/05-modules/citizencode/：同步技术文档，涉及文档。

验收标准：
- 功能可运行。
- 后端拒绝普通公权机构手动创建。
- 前端公权机构只保留“新增”入口，弹窗机构选项为“教育委员会 (JY)”。
- 私权机构新增显示“教育委员会 (JY)”，选择后名称字段改为“学校名称”。
- 公权机构确定性目录无需精确搜索即可直接展示。
- 确定性列表和机构详情有本地缓存时先显示缓存，再后台刷新。
- 机构状态只有未注册、已注册、已注销三态。
- 代码关键逻辑有中文注释。
- 文档已更新。
- 残留已清理。
- CID 模块完成标准已对照。

执行记录：
- 后端已清理早期教育分类试验字段和临时来源枚举；机构身份仍只使用 `cid_number`。
- 后端已将机构链上状态收口为 `NOT_REGISTERED / REGISTERED / REVOKED_ON_CHAIN`。
- 后端启动时已增加普通自动机构目录对账：国家/省级机构读取 citizenchain `china_*.rs` 常量，市级自治政府/立法会/司法院/监察院/教育委员会读取 CID 行政区划生成。
- 后端普通官方公权机构不走手动创建；仅允许市注册局机构管理员注册教育委员会 `JY` 类型学校机构。
- 后端已新增 `/api/v1/institutions/official` 确定性公权机构列表接口，按登录省/市 scope 直接返回自动目录和手动学校机构。
- 前端公权机构新增按钮已改为“新增”；公权/私权机构选项均显示“教育委员会 (JY)”，选择后名称字段显示“学校名称”。
- 前端确定性公权列表、机构详情和注册局管理员列表已接入本地缓存：先显示缓存，再后台刷新只读查询结果。
- 公权机构确定性列表接口已改为请求路径只读，不再在每次 GET 时触发全量自动目录 reconcile 和批量写库，避免进入某市公权机构列表时被全量对账拖慢。
- 注册局联邦注册局机构管理员详情页已修复首次自动定位所属省时覆盖用户点击页签的问题，只有真实切换省份时才重置到默认市列表。
- 2026-06-27 追加修复：联邦注册局管理员登录态缺少省域时后端直接拒绝，前端不再使用 `ALL/全国` 或伪省名兜底；市注册局 tab 按登录省域加载城市列表。
- 2026-06-27 追加修复：管理员姓名不再由账户公钥回填，upsert 不再用空姓名覆盖真实姓名，已污染的“姓名等于账户”记录在启动迁移中清空。
- 2026-06-27 追加修复：CPOL 市公安局并入 `CITY_TEMPLATES` 普通市级公权机构模板，registry 侧删除公安局专用生成路径和独立生成残留。
- 2026-06-27 追加修复：明确 `InstitutionCategory` 只是前端 tab 展示桶；公法人、私法人、非法人是独立法律主体类型，非法人按父级公/私法人分流。
- 2026-06-27 运行态验收：使用 `/tmp` 临时内嵌 PostgreSQL 播种联邦注册局管理员,确认省域映射 `215/43`、污染姓名 `0`；按 `ZS/001` 单市对账确认 CPOL 作为普通市级公权机构生成；完整 `reconcile-gov --changed-only` + `check-gov --strict` 后启动 registry,`/api/v1/health` 返回 UP。
- 已更新 CID institutions、CID 工具和前端布局文档。
- 已运行 `git diff --check`、`cargo fmt --manifest-path citizencode/backend/Cargo.toml`、`cargo check --manifest-path citizencode/backend/Cargo.toml`、`cargo test --manifest-path citizencode/backend/Cargo.toml` 与 `npm run build`；前端构建产物 `citizencode/frontend/dist/` 已清理。
