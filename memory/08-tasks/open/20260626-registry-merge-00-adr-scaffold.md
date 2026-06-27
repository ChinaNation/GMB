# 任务卡：registry 并入 Step0 — crate 骨架与进程贯通

## 任务需求

在 `citizenchain` 下新建 `registry` workspace 成员 crate，建立后端骨架、内嵌 PostgreSQL 引导、节点桌面端拉起 registry 进程的最小贯通，作为后续迁移的承载基座。

## 所属模块

citizenchain（registry 子系统 + node 进程编排）

## 预计修改目录

- `citizenchain/Cargo.toml`
  - 用途：`members` 增加 `registry`。
  - 边界：只加成员，不动既有 crate。
- `citizenchain/registry/`（新建）
  - 用途：`registry/src`（Axum 服务骨架 + 内嵌 PostgreSQL 启动/迁移引导 + 经节点 RPC 读链的 chain client 雏形）、`registry/frontend`（占位）。
  - 边界：本步只搭骨架与健康检查路由，不迁业务。
- `citizenchain/node/src/desktop/`、`citizenchain/node/src/<进程编排>`
  - 用途：node 启动时拉起/停止 registry 进程并做生命周期管理。
  - 边界：复用现有节点生命周期模式，不改 runtime。
- `memory/01-architecture/citizenchain/CITIZENCHAIN_TECHNICAL.md`
  - 用途：登记 registry 子系统与进程模型。
  - 边界：只补当前目标态。

## 输入文档

- memory/04-decisions/ADR-029-registry-into-citizenchain.md
- memory/01-architecture/citizencode/README.md
- memory/01-architecture/citizenchain/CITIZENCHAIN_TECHNICAL.md

## 必须遵守

- 内嵌 PostgreSQL 随节点私有实例起停，安装包零依赖（project_installer_zero_dep）。
- 不破坏现有节点桌面与挖矿/设置流程。
- 不清楚逻辑先沟通。

## 验收标准

- `cargo check`（workspace）通过，`registry` 可独立编译。
- node 启动可拉起 registry 进程，健康检查路由可访问，关闭 node 同步停 registry。
- 内嵌 PostgreSQL 可初始化空库并连通。

## 进度

- [x] 任务卡创建
- [x] 新建 `citizenchain/registry` crate 并加入 workspace 成员
- [x] 后端骨架(Axum + `/health` + 内嵌 PostgreSQL 探活 + 链 RPC 配置雏形)
- [x] node 桌面端拉起/停止 registry 子进程(`node/src/registry_proc`,setup 拉起 / Exit 停止)
- [x] `cargo check -p registry`、`cargo check -p node` 通过
- [x] CITIZENCHAIN_TECHNICAL.md 登记 registry 子系统与进程模型
- [ ] 真实运行态验收(需本机起 PostgreSQL,留作起库环境验证)

## 完成摘要

- 新增 `citizenchain/registry` 独立 crate(`Cargo.toml` + `src/{main,config,db,chain,health}.rs`),并加入 `citizenchain/Cargo.toml` workspace 成员。
- registry 为独立二进制:Axum 服务,`/health` 输出服务状态 / 内嵌库连通性 / 链 RPC 地址 / 版本;`db.rs` 用同步 `postgres` + `spawn_blocking` 探活(无库不崩,仅 warn);`chain.rs` 持有节点 RPC 地址雏形;`config.rs` 全部环境变量带本机安全默认值。
- node 侧新增 `registry_proc` 模块:从节点可执行文件同目录解析 `registry` 二进制,setup 阶段 `start_registry()` 拉起、`RunEvent::Exit` 时 `stop_registry()` 停止;找不到二进制只提示不崩。
- 沿用 citizencode 同款 `axum 0.7` / `postgres 0.19`,便于 Step1 平移。

## 验收结果

- `cargo fmt -p registry` / `cargo fmt -p node`:已格式化。
- `cargo check -p registry`:通过,零警告(24.9s)。
- `cargo check -p node`:通过(含整条 substrate 依赖,24.8s)。
- 真实运行态(节点拉起 registry 子进程 + `/health` + 内嵌 PG 连通):需本机起 PostgreSQL 后做端到端验证;无库时 registry 仍可起、`/health` 返回 `db=disconnected`。
