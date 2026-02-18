# fullnode 技术方案基线（模板）

## 1. 文档目标
- 定义 `fullnode` 的统一技术方案基线。
- 保证与 `fcrcnode` 使用同一技术路线、同一工程规范。
- 明确本软件独立边界：面向全节点运维用户，不与其他软件混合部署。

## 2. 产品定位与边界
- 产品名称：`fullnode`
- 目标用户：全节点运营方、节点管理员、技术运维人员
- 核心业务域：节点监控、同步状态、交易操作、出块奖励统计
- 非目标范围：委员会治理审批流程（由 `fcrcnode` 负责）

## 3. 技术栈（与 fcrcnode 保持一致）
- 桌面端：`Tauri + React + TypeScript + Vite`
- 桌面壳：`src-tauri`（Rust）
- 服务端：`Rust`（HTTP/RPC 服务）
- 数据库：`PostgreSQL`（生产）/ `SQLite`（开发）
- 接口协议：`REST + JSON`（必要时补充 WebSocket 推送）
- 包管理与构建：
  - 前端：`npm`
  - Rust：`cargo`

## 4. 推荐目录规范
```text
fullnode/
├── backend/
│   ├── src/
│   ├── tests/
│   └── migrations/
├── desktop/
│   ├── src/
│   ├── src-tauri/
│   ├── tests/
│   └── package.json
├── docs/
│   ├── TECH-BASELINE.md
│   ├── API-SPEC.md
│   └── RELEASE.md
└── scripts/
```

## 5. 架构分层
- 表现层：React 页面、组件、状态管理
- 桌面能力层：Tauri commands（文件、进程、系统集成）
- 服务层：业务服务、鉴权、审计、调度
- 数据层：仓储、迁移、查询模型
- 外部集成层：链 RPC、节点进程控制、监控采集

## 6. 接口与错误规范
- API 前缀：`/api/v1`
- 响应结构统一：
  - 成功：`{ code: 0, message: "ok", data: ... }`
  - 失败：`{ code: <non-zero>, message: "...", trace_id: "..." }`
- 错误码分段：
  - `1xxx` 参数与校验
  - `2xxx` 鉴权与权限
  - `3xxx` 业务规则
  - `5xxx` 系统与依赖

## 7. 安全基线
- 私钥不落后端，签名默认在端侧完成。
- Tauri capability 最小权限开放，禁止全量默认授权。
- 敏感操作必须记录审计日志：时间、操作者、公钥、动作、结果。
- 节点控制命令（启停/重启）必须有角色权限校验。
- 配置中的密钥信息使用环境变量注入，不写入仓库。

## 8. 工程质量基线
- Rust：`cargo fmt`、`cargo clippy`、`cargo test`
- 前端：`npm run lint`、`npm run typecheck`、`npm run test`、`npm run build`
- 提交门禁：至少通过格式化、静态检查、单元测试。
- 分支策略：`main` 稳定分支 + 功能分支开发。

## 9. 跨平台发布基线
- 目标平台：`macOS`、`Linux`、`Windows`
- CI 矩阵：
  - `macos-latest`
  - `ubuntu-latest`
  - `windows-latest`
- 产物类型：
  - macOS：`.app` / `.dmg`
  - Linux：`.AppImage` / `.deb`
  - Windows：`.msi` / `.exe`
- 签名与更新通道独立于 `fcrcnode`，互不影响。

## 10. 配置与环境
- 环境分层：`dev` / `staging` / `prod`
- 最小配置项：
  - `CHAIN_RPC_URL`
  - `NODE_BINARY_PATH`
  - `BACKEND_BIND_ADDR`
  - `DATABASE_URL`
  - `LOG_LEVEL`
  - `AUDIT_LOG_PATH`

## 11. 观测与运维
- 日志：结构化 JSON，字段统一（`ts`、`level`、`trace_id`、`module`）
- 指标：节点在线率、同步延迟、区块高度差、API 错误率
- 告警：按严重级别分级（P1/P2/P3）
- 备份：数据库与关键配置定期备份

## 12. 版本治理
- 版本号：`MAJOR.MINOR.PATCH`
- 发布节奏：固定迭代 + 紧急修复通道
- 变更记录：每个版本必须提供 `CHANGELOG`
- 兼容性：破坏性变更需提前一个版本公告

## 13. 实施检查清单
- [ ] 技术栈与依赖版本已冻结
- [ ] 目录结构符合本基线
- [ ] API 与错误码规范已落地
- [ ] 安全与审计策略已启用
- [ ] 三平台构建流水线可用
- [ ] 发布文档与回滚预案完备

