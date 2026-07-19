# 任务卡：OnChina 机构工作台框架统一

## 当前状态

已执行完成，等待人工复核后可归档。

## 任务背景

OnChina 链上中国定位为所有机构共用的操作台。注册局、司法院、立法院、行政机关、学校、公益组织、公司等机构，都通过相同的节点端加浏览器方式进入本机构后台。

登录入口使用公民钱包扫码。钱包账户在链上的机构管理员模块中属于哪个机构，就进入哪个机构的工作台；如果同一钱包同时属于多个机构管理员，则登录时选择机构后进入对应工作台。

目前注册局功能和 UI 已经比较完善，本任务不改造注册局业务 UI。当前需要补齐的是统一机构工作台框架、目录结构、文件命名、字段命名、注释、文档和旧注册局根语义残留清理。

## 已确认设计

- 统一使用 `workspace` 表达机构工作台。
- 原 `registry` 注册局只是 `workspace` 中的一类机构工作台。
- 不再把 OnChina 前端根 UI 设计成注册局专用后台。
- 公权机构已经统一在创世阶段上链；OnChina 不负责生成既有公权机构，只负责读取链上机构和管理员唯一真源，并投影到本地展示与登录态。
- 非注册局机构不复用注册局功能 UI。
- 机构工作台按三类页面组织：
  - 操作：本机构可发起或执行的链上操作，例如提案、投票、变更管理员等。
  - 显示：本机构的链上资料、管理员、权限、身份信息。
  - 记录：本机构所有操作、投票、登录、管理员变更等记录。

## 建议模块

- `citizenchain/onchina/src/workspace/`
  - 后端机构工作台类型、权限清单、机构分类与登录态输出。
- `citizenchain/onchina/frontend/workspace/`
  - 前端机构工作台壳、路由、通用页面分类和机构专属 UI 挂载点。
- `memory/01-architecture/onchina/`
  - OnChina 总体架构文档更新。
- `memory/05-modules/citizenchain/onchina/`
  - OnChina 前后端模块文档更新。
- `memory/07-ai/`
  - 字段命名和接口协议文档补齐。

## 预计修改目录

- `citizenchain/onchina/src/workspace/`
  - 用途：新增后端机构工作台框架，生成 `workspace` 登录态清单。
  - 边界：只描述工作台类型和可见入口，不保存管理员授权真源，不承载业务 handler。
  - 类型：代码新增。
- `citizenchain/onchina/src/auth/`
  - 用途：登录、扫码登录轮询、登录态检查和身份识别返回 `workspace`。
  - 边界：只改登录态 DTO 和会话组装，不新增授权分支。
  - 类型：代码修改。
- `citizenchain/onchina/src/platform/`
  - 用途：清理能力位注释，把旧 tab 口径统一成工作台入口口径。
  - 边界：能力位仍是声明式渲染单源，安全边界仍在 handler、scope 和链上 active admins。
  - 类型：注释清理。
- `citizenchain/onchina/frontend/workspace/`
  - 用途：新增前端工作台路由、通用壳、注册局挂载层、司法院工作台和通用机构工作台。
  - 边界：注册局业务 UI 只挂载不重构；非注册局机构不复用注册局 tab。
  - 类型：代码新增。
- `citizenchain/onchina/frontend/auth/`
  - 用途：前端登录态类型和 API 接收 `workspace`。
  - 边界：只扩展登录态，不新增本地权限真源。
  - 类型：代码修改。
- `memory/01-architecture/`
  - 用途：更新 OnChina 和 CitizenChain 总架构口径。
  - 边界：只更新文档，不改协议或代码。
  - 类型：文档修改。
- `memory/05-modules/citizenchain/onchina/`
  - 用途：更新 OnChina 前后端模块、数据安全和验收口径。
  - 边界：只描述当前实现和规则，不新增目录。
  - 类型：文档修改。
- `memory/07-ai/`
  - 用途：登记 `workspace` 目录、字段和登录态接口契约。
  - 边界：命名与协议登记，不承载业务设计细节。
  - 类型：文档修改。
- `memory/08-tasks/open/`
  - 用途：记录本任务目标、执行过程和验收结果。
  - 边界：任务卡记录，不作为代码真源。
  - 类型：文档新增与修改。

## 影响范围

- 登录态：需要在管理员登录返回中补充当前机构工作台信息。
- 权限模型：机构权限按 `workspace_kind` 和链上管理员角色共同决定。
- 前端框架：需要把注册局从“根 UI”降为一个工作台类型。
- 文档：需要同步说明创世公权机构、链上管理员唯一真源、workspace 目录边界。
- 残留清理：需要清理把 OnChina 误写成“注册局专用操作台”的文案和注释。

## 主要风险点

- 不得改动或回退已完善的注册局功能 UI。
- 不得新增独立管理员授权真源；管理员唯一字段继续统一为 `admins`。
- 不得恢复 `backend/src/`、`frontend/api/`、`frontend/chain/` 等旧目录结构。
- 涉及接口字段时必须全仓统一命名，避免同义字段重复。
- 不涉及 `citizenchain/runtime/` 修改，避免触发 runtime 二次确认范围。
- 新增文件前必须逐项列明路径、用途、原因、是否 Git 跟踪，并得到当前任务确认。

## 分步执行

1. 创建任务卡，记录用户确认后的目标框架。
2. 列出需要新增的代码和文档文件，等待用户确认。
3. 读取 OnChina 当前登录、权限、注册局 UI、前后端目录实现。
4. 实现后端 `workspace` 框架，不改变注册局既有业务行为。
5. 实现前端 `workspace` 框架，把注册局 UI 作为一种工作台挂载。
6. 更新架构文档、模块文档、字段命名文档和接口说明。
7. 清理旧“注册局专用根操作台”残留文案和注释。
8. 执行编译、构建和可行的本地运行态验收。

## 验收标准

- 登录态能够表达当前机构的 `workspace` 信息。
- 国家司法院等非注册局机构进入的是司法类工作台框架，不再显示成注册局管理员后台。
- 注册局既有 UI 和功能不被重构或破坏。
- 新增字段、目录、文件命名符合统一命名规则。
- 文档同步说明新框架和目录边界。
- 代码中关键职责边界有中文注释。
- 完成后清理无用残留。

## 执行记录

- 后端新增 `citizenchain/onchina/src/workspace/`，用于生成机构工作台类型、三段式分区和登录态清单。
- 前端新增 `citizenchain/onchina/frontend/workspace/`，用于承载工作台路由、注册局挂载层、司法院工作台和通用机构工作台。
- 注册局 UI 仅从 `App.tsx` 搬到 `workspace/registry/RegistryWorkspace.tsx` 挂载，没有重构注册局业务组件和业务交互。
- 登录、扫码登录轮询、登录态检查和身份识别接口均返回 `workspace`。
- 修复国家司法院登录时管理员名兜底为“市注册局管理员”的残留逻辑。
- 修复节点 active binding 缓存缺少 `institution_cid_number` 时本机构管理员接口返回空或 500 的问题：读取 active binding 时用链上账号投影和 `subjects/accounts` 补齐机构元数据，并兼容本地账号无 `0x`、链上账号带 `0x` 的比较差异。
- 文档已同步更新 OnChina 总架构、前后端模块文档、数据安全文档、统一命名、统一协议和 ADR-030。

## 验收记录

- `rustfmt --edition 2021` 已对本轮改动的 OnChina Rust 文件执行。
- `cargo check --manifest-path citizenchain/onchina/Cargo.toml` 通过。
- `./node_modules/.bin/tsc -p tsconfig.json --noEmit --incremental false` 在 `citizenchain/onchina/frontend` 通过。
- `./node_modules/.bin/vite build --outDir /tmp/onchina-workspace-vite-build --emptyOutDir` 通过；仅保留 Vite 大 chunk 提示。
- 真实本地后端 `http://127.0.0.1:8965` 验收通过：使用现有 NJD 会话调用 `/api/v1/admin/auth/check` 返回司法工作台类型、标题和当时的合并姓名展示字段；该字段现已统一为 `family_name + given_name`。
- 真实本地后端连接链 RPC 后，NJD 会话调用 `/api/v1/admin/own-institution-admins` 返回 15 名国家司法院链上 active admin，不再为空或 500。
- 真实页面验收通过：国家司法院页面显示“国家司法院工作台”和“操作 / 显示 / 记录”，顶部显示“国家司法院 · 国家司法院管理员”，未显示注册局 tab 或“市注册局管理员”。
