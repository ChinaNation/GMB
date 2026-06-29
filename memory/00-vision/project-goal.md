# GMB 项目目标

## 1. 项目定位

GMB 是一套由公民链、公民、公民钱包、官方网站和 AI 驱动研发组成的综合系统。注册局身份、公民护照、机构登记和人口凭证归属公民链内置 OnChina 能力。

系统核心目标是：

- 建立清晰、可审计、可长期维护的信任边界
- 固化各产品当前真实技术栈，避免 AI 按过期技术方案执行
- 通过 AI 开发体系提升研发效率，同时避免架构失控
- 让项目知识长期沉淀在仓库文档中，而不是停留在聊天记录中

## 2. 核心组成

- `citizenchain`：区块链 runtime、节点程序、节点 UI、桌面安装包和 OnChina 注册局身份能力
- `citizenapp`：公民端在线钱包与业务入口
- `citizenwallet`：公民钱包离线签名、扫码识别和签名响应
- `website`：官方网站
- `memory`：AI 永久记忆中心

## 3. AI 开发目标

GMB 的 AI 开发体系必须满足以下要求：

- 支持多个等价聊天入口
- 中文自然语言输入
- 各智能体职责明确
- Codex 与 Claude 都可以承接主开发
- Codex 与 Claude 都可以承担代码检查与修复
- 代码改动后必须同步更新文档
- 代码改动后必须清理残留
- 逻辑不清时必须先与项目负责人沟通

## 4. 技术总原则

- `citizenchain/node` 使用 Rust + Substrate / Polkadot SDK + Tauri + React + TypeScript + Vite
- `citizenchain/runtime` 使用 Rust + Substrate / Polkadot SDK
- `citizenchain/onchina` 使用 React + TypeScript + Vite 前端，Rust + Axum 后端，PostgreSQL 持久化
- `citizenapp` 使用 Flutter + Dart，并继续使用 Isar 做端上本地存储
- `citizenwallet` 使用 Flutter + Dart
- 区块链使用框架自带数据库
- 业务系统使用 PostgreSQL

## 5. 项目原则

- 不让 AI 记对话，要让 AI 记项目结构
- 不先堆功能，先固化边界、契约、文档和流程
- 不让 AI 在不清楚逻辑时自行猜测关键业务行为
