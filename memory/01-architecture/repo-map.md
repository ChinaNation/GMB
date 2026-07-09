# GMB 仓库映射

## 1. 仓库原则

GMB 使用唯一仓库模式，所有核心系统、文档、自动化流程和 AI 永久记忆统一放在同一个仓库中管理。

仓库根目录固定如下。这里登记的是 tracked 主目录和入口文件；本地工具私有目录、构建产物和缓存目录不属于固定结构。

```text
GMB/
  .github/
  .githooks/
  .vscode/
  memory/
  citizenchain/
  citizenwallet/
  citizenapp/
  citizenweb/
  docs/
  scripts/
  AGENTS.md
  CODEX.md
  CLAUDE.md
  README.md
  Cargo.toml
  Dockerfile
```

## 2. 目录职责

- `.github/`：GitHub Actions、PR 自动化、构建发布流程
- `.githooks/`：仓库级 Git hook 脚本
- `.vscode/`：共享编辑器设置
- `memory/`：AI 编程系统、项目长期记忆、产品文档与模块文档真源
- `citizenchain/`：公民链产品，包含区块链 runtime、节点程序、节点桌面 UI、打包发布和 OnChina 内置注册局能力
- `citizenwallet/`：公民钱包，负责离线签名、扫码识别和钱包 UI
- `citizenapp/`：公民，负责公民端钱包、治理、投票和链上状态展示
- `citizenweb/`：GMB 官网前端工程，当前使用 React + TypeScript + Vite 构建静态站点
- `docs/`：静态发布文档和展示资产，不承载系统权威记忆
- `scripts/`：仓库级脚本、生成器和自动化工具

根入口文件职责：

- `AGENTS.md`：GMB AI 编程系统新线程最高优先级启动协议
- `CODEX.md`：Codex 入口说明，必须与 `memory/CODEX.md` 同步
- `CLAUDE.md`：Claude 入口说明，必须与 `memory/CLAUDE.md` 同步
- `README.md`：仓库说明
- `Cargo.toml` / `Cargo.lock`：Rust workspace 根配置
- `Dockerfile` / `.dockerignore`：容器构建配置

## 3. 文档集中管理规则

正式文档统一收口到 `memory/`：

- `memory/00-vision/`：白皮书、愿景、总目标
- `memory/01-architecture/`：仓库级与产品级架构文档
- `memory/03-security/`：安全规则、边界和风险要求
- `memory/04-decisions/`：ADR 和重要设计决策
- `memory/05-modules/`：模块级技术文档
- `memory/06-quality/`：测试、缺陷、变更记录模板和跨端 fixture
- `memory/07-ai/`：AI 编程系统规则
- `memory/08-tasks/`：任务卡、执行记录与归档
- `scripts/`：memory 自检和入口验收脚本

以下旧目录不再属于当前结构，不得新建或恢复：

- `memory/05-architecture/`
- `memory/tasks/`

产品目录默认只保留：

- 源代码
- 配置
- 测试
- 构建与部署脚本
- 数据库迁移
- 运行所需资源文件

## 4. citizenchain 目标结构

`citizenchain` 作为一个完整区块链桌面产品进行管理，目标结构如下：

```text
citizenchain/
  crates/
    chain-signing/
  node/
  runtime/
    governance/
    issuance/
    otherpallet/
    transaction/
    primitives/
  onchina/
  packaging/
  docs/
```

## 5. citizenchain 当前结构

当前仓库已经按目标结构落地：

- `citizenchain/runtime/governance`
- `citizenchain/runtime/issuance`
- `citizenchain/runtime/otherpallet`
- `citizenchain/runtime/transaction`
- `citizenchain/runtime/primitives`
- `citizenchain/crates/chain-signing`
- `citizenchain/node`
- `citizenchain/onchina`

其中：

- `citizenchain/crates/chain-signing` 是 Rust host 端链交易签名材料唯一真源，node 与 OnChina 不再各自拼 `SignedPayload`
- 四类 runtime 业务目录已经统一收敛到 `citizenchain/runtime/`
- 原仓库根目录 `primitives/` 已迁入 `citizenchain/runtime/primitives`
- 原生节点、桌面节点 UI、Tauri 壳与打包入口统一收口到 `citizenchain/node`
- OnChina 多机构工作台、注册局业务、行政区、机构登记、管理后台和链侧凭证能力统一收口到 `citizenchain/onchina`
- 桌面端 Rust 后端模块已经扁平化到 `citizenchain/node/src/<功能名>`，不再保留 `src/ui` 目录层
- 历史旧目录 `citizenchain/node` 与独立 `citizenchain/node` 均不再作为当前实现

## 6. 当前落地策略

当前结构已经完成物理整合，后续新增桌面节点 Rust 后端功能直接放在 `citizenchain/node/src/<功能名>`，前端功能放在 `citizenchain/node/frontend/<功能名>`；runtime pallet 与运行时原语直接放在 `citizenchain/runtime/`，跨 node/OnChina 的 Rust host 共享库放在 `citizenchain/crates/`。

## 6b. OnChina 目录策略

OnChina 是 `citizenchain` 内置能力，不再作为独立产品目录存在。旧后端源码壳、旧前端源码壳、前端旧 views 壳、后端独立 chain 业务目录、前端独立 chain 业务目录、前端独立业务 API 目录均已删除。OnChina 前后端都直接以各自根目录为代码根，按业务功能展开。

- `citizenchain/onchina/src/main.rs`:后端入口,`Cargo.toml` 显式 `[[bin]] path = "main.rs"`。
- `citizenchain/onchina/src/core/`:跨业务底层工具,含 `chain_*` 通用链工具、HTTP 安全、统一响应与 QR 协议辅助。
- `citizenchain/onchina/src/workspace/`:机构工作台类型、三段式分区和登录态工作台清单。
- `citizenchain/onchina/src/citizens/`:公民身份业务和公民链交互 `chain_*`。
- `citizenchain/onchina/src/subjects/`:身份主体共享模型、公共详情、非法人能力和机构链端公开查询。
- `citizenchain/onchina/src/gov/`:公权机构确定性目录入口,CPOL 与其它市级公权机构同模板生成。
- `citizenchain/onchina/src/private/`:私权机构注册和精确查询入口。
- `citizenchain/onchina/src/accounts/`:机构账户入口。
- `citizenchain/onchina/src/docs/`:机构资料库入口。
- `citizenchain/onchina/src/cid/china/`:中国行政区划 SQLite 真源。
- `citizenchain/onchina/src/cid/`:身份 ID 编码协议、机构码、生成和校验。
- `citizenchain/onchina/src/admins/`:联邦注册局机构管理员/市注册局机构管理员治理、冷钱包扫码签名二次确认和权限上下文。
- `citizenchain/onchina/frontend/auth/`:登录、AuthContext、登录态类型和 `api.ts`。
- `citizenchain/onchina/frontend/core/`:前端通用组件、共享 UI、扫码签名面板与 QR 工具。
- `citizenchain/onchina/frontend/china/`:行政区划元数据 API 与本地缓存。
- `citizenchain/onchina/frontend/subjects/`:主体共享类型、字段标签和 `chain_multisig_info.ts`。
- `citizenchain/onchina/frontend/gov/`:公权机构页面入口。
- `citizenchain/onchina/frontend/private/`:私权机构页面入口。
- `citizenchain/onchina/frontend/accounts/`:机构账户组件。
- `citizenchain/onchina/frontend/docs/`:机构资料库组件。
- `citizenchain/onchina/frontend/admins/`:联邦注册局机构管理员/市注册局机构管理员页面、API 与冷钱包扫码签名前端流程。

同名对齐规则:

- 后端链交互文件:`citizenchain/onchina/src/<功能模块>/chain_*.rs`
- 前端链交互文件:`citizenchain/onchina/frontend/<功能模块>/chain_*`
- runtime 公民身份模块:`citizenchain/runtime/otherpallet/citizen-identity/`

## 7. GitHub Actions 路径分流原则

GMB 的自动化已经改为“每个系统 / 模块一个 workflow”：

- `citizenchain/node`
  - `.github/workflows/citizenchain.yml`
- `citizenchain/runtime/governance`
  - `.github/workflows/citizenchain-runtime-governance.yml`
- `citizenchain/runtime/issuance`
  - `.github/workflows/citizenchain-runtime-issuance.yml`
- `citizenchain/runtime/otherpallet`
  - `.github/workflows/citizenchain-runtime-otherpallet.yml`
- `citizenchain/runtime/primitives`
  - `.github/workflows/citizenchain-runtime-primitives.yml`
- `citizenchain/runtime/src`
  - `.github/workflows/citizenchain-runtime-src.yml`
- `citizenchain/runtime/transaction`
  - `.github/workflows/citizenchain-runtime-transaction.yml`
- `citizenchain/onchina`
  - 归属公民链 CI，按 OnChina 后端、前端和链交互变更执行对应本地/CI 检查；不得恢复独立 旧独立身份系统 CI
- `citizenapp`
  - `.github/workflows/citizenapp-ci.yml`
- `citizenwallet`
  - `.github/workflows/citizenwallet-ci.yml`
- `citizenweb`
  - 当前暂无专用 GitHub Actions，官网发布前需在本地执行 `npm run build` 并部署 `citizenweb/dist/`

补充说明：

- OnChina 不再保留独立发布包、独立部署 workflow 或独立产品入口。
- Pages 只在 `docs/**` 或自身 workflow 变更时触发
- 共享 Rust 根目录变更允许触发多个 citizenchain workflow，这是保留的安全边界
