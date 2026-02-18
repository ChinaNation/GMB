# fcrcnode 目录结构（第一版）

本结构按 `Tauri + React + TypeScript` 技术路线先行落地，目标是先支持桌面前端 MVP 开发并与后端并行。

## 顶层

- `backend/`：联储会后端（链外服务、报表、日志、配置）
- `desktop/`：联储会桌面前端（Tauri 壳 + React UI）
- `docs/`：设计文档、接口文档、流程说明
- `scripts/`：本地开发脚本（构建、检查、发布）

## backend

- `src/config/`：配置模型与加载
- `src/db/models/`：链外数据模型
- `src/routes/`：HTTP/RPC 路由
- `src/services/`：业务服务层
- `src/domain/`：领域对象与规则
- `src/adapters/`：外部系统适配
- `src/utils/`：通用工具
- `tests/`：后端测试
- `migrations/`：数据库迁移

## desktop

- `public/`：静态资源
- `src/app/`：应用入口与路由装配
- `src/layouts/`：布局组件
- `src/components/`：通用 UI 组件
- `src/pages/Nrc/`：国储会页面
- `src/pages/Prc/`：省储会页面
- `src/pages/Prb/`：省储行页面
- `src/features/auth/`：登录与角色鉴权
- `src/features/chain/`：链连接与链状态
- `src/features/transaction/`：交易流程
- `src/features/governance/`：治理提案/投票
- `src/features/monitor/`：余额与监控
- `src/services/rpc/`：链 RPC 封装
- `src/services/auth/`：认证服务
- `src/services/signer/`：签名服务（后续可扩展硬件钱包）
- `src/stores/`：客户端状态管理
- `src/hooks/`：复用 hooks
- `src/utils/`：前端工具
- `src/types/`：TS 类型
- `src/constants/`：常量
- `src/assets/styles/`：全局样式与设计变量
- `src-tauri/`：Tauri Rust 壳目录（预留，后续迁移）
- `tests/`：前端测试
