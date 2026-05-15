# 文档真源与治理机构账户模型收口

## 任务需求

- 将白皮书和公民宪法固定为唯一真源；后续公民宪法已从 docs Markdown 迁入 runtime HTML 真源。
- 桌面端三端安装包必须在构建时内置最新文档，并保留原白皮书/公民宪法显示样式。
- 清理旧 `GMB_WHITEPAPER.md`、`GMB_README.html`、`FRC_README.html`、GitHub raw 文档读取链路。
- wuminapp 治理机构静态写死机构名称、身份 ID 和各制度账户地址。
- wuminapp 治理机构管理员和阈值必须从链上 `AdminsChange::Subjects` 动态读取，不使用创世常量管理员。
- 治理机构账户字段从 `duoqianAddress` 语义收口到主账户、费用账户、安全基金账户、质押账户。

## 影响范围

- `docs/`：文档真源和旧静态 HTML 残留。
- `citizenchain/node/frontend/`：文档生成、Markdown 渲染、样式迁移和构建脚本。
- `citizenchain/node/src/other/other-tabs/`：其他 tab 内容协议从 iframe URL 改为本地文档 key。
- `wuminapp/lib/institution/`：治理机构账户模型、静态注册表、详情页和更多账户内联展示。
- `wuminapp/lib/proposal/transfer/`：治理机构转账来源账户改为主账户。
- `memory/05-modules/`：技术文档同步。

## 执行原则

- 不修改与本任务无关的既有未提交文件。
- 文档正文唯一真源当时保留为白皮书 Markdown 与公民宪法 Markdown；后续公民宪法已迁入 runtime HTML 真源。
- 机构账户地址静态生成；管理员和阈值动态读链。

## 验收项

- 桌面端不再通过 iframe/CDN/raw URL 加载白皮书和公民宪法。
- `npm run build` 自动生成并内置最新文档。
- wuminapp 治理机构路径不再用 `duoqianAddress` 表达内置治理机构主账户。
- 管理员列表与阈值仍从 `AdminsChange::Subjects` 读取。
- 文档、注释、残留扫描同步完成。

## 完成记录

- 已删除旧 `docs/GMB_README.html`、`docs/FRC_README.html`、`docs/index.html`；`memory/00-vision/GMB_WHITEPAPER.md` 保持删除状态。
- 已新增桌面端构建前文档生成脚本；该历史实现后来已调整为白皮书进入本地 bundle、公民宪法从 runtime 读取。
- 已把 other-tabs 协议从 iframe URL 改为本地 document key，前端用 `LocalDocViewer` 渲染并保留原文档显示风格。
- 已从 runtime primitives 生成 87 个治理机构的名称、身份 ID、主账户、费用账户、安全基金账户、质押账户静态表。
- 已新增机构详情顶部信息区与更多账户展示；管理员列表与阈值继续动态读取链上 `AdminsChange::Subjects`。
- 已同步更新 README、Other Tabs 技术文档、wuminapp 治理/转账技术文档和统一命名文件。
- 验证：`npm --prefix citizenchain/node/frontend run build` 通过；`flutter analyze` 通过；`flutter test test/institution/institution_admin_service_test.dart` 通过。
- Rust：`cargo check --manifest-path citizenchain/node/Cargo.toml --bin citizenchain` 被 runtime `build.rs` 按仓库规则拦截，原因是未设置 `WASM_FILE`；这是 CI WASM 门禁，不是本次代码错误。
