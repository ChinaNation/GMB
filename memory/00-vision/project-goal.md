# GMB 项目目标

## 1. 项目定位

GMB 是一套由离线实名、在线身份、链上权限、移动端交互和 AI 驱动研发组成的综合系统。

系统核心目标是：

- 建立清晰、可审计、可长期维护的信任边界
- 固化各产品当前真实技术栈，避免 AI 按过期技术方案执行
- 通过 AI 开发体系提升研发效率，同时避免架构失控
- 让项目知识长期沉淀在仓库文档中，而不是停留在聊天记录中

## 2. 核心组成

- `cpms`：完全离线实名录入、审核、签发系统
- `sfid`：在线身份验证、账户绑定、permit 签发系统
- `citizenchain`：区块链 runtime、节点程序、节点 UI、桌面安装包
- `wuminapp`：手机端钱包与业务入口
- `memory`：AI 永久记忆中心

## 3. AI 开发目标

GMB 的 AI 开发体系必须满足以下要求：

- 唯一主聊天窗口
- 中文自然语言输入
- 各智能体职责明确
- Codex 负责主开发
- Claude 负责代码检查与修复建议
- 代码改动后必须同步更新文档
- 代码改动后必须清理残留
- 逻辑不清时必须先与项目负责人沟通

## 4. 技术总原则

- `citizenchain/node` 与 `citizenchain/runtime` 使用 Rust
- `citizenchain/nodeui` 使用 Rust + Tauri + React + TypeScript + Vite
- `wuminapp` 使用 Flutter + Dart，并继续使用 Isar 做端上本地存储
- `sfid` 当前使用 React + TypeScript + Vite 前端，Rust + Axum 后端，PostgreSQL 持久化
- `cpms` 当前落地代码使用 Rust + Axum + SQLx + PostgreSQL；`frontend/` 目录仅保留预留结构，尚无独立前端实现
- 区块链使用框架自带数据库
- 业务系统使用 PostgreSQL

## 5. 项目原则

- 不让 AI 记对话，要让 AI 记项目结构
- 不先堆功能，先固化边界、契约、文档和流程
- 不让 AI 在不清楚逻辑时自行猜测关键业务行为
