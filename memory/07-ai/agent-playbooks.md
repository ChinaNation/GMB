# GMB Agent 执行说明书

## 调度原则

- 用户只在 Codex 主窗口输入中文任务需求
- Codex 主线程默认承担 `Architect Agent` 和总调度器职责
- 需求分析完成后，Codex 按模块边界决定是否调度专业工作线程
- 用户不需要手工指定 `Blockchain Agent`、`OnChina Agent`、`Mobile Agent`、`Wallet Agent`
- 跨模块任务由 Codex 先拆解，再分派到对应专业工作线程，结果统一回写 `memory/`、任务卡和相关文档

## Architect Agent

- 负责读取 `memory/`
- 负责需求分析、任务拆解、边界控制
- 发现歧义时必须先沟通

## Blockchain Agent

- 触发条件：任务涉及 `citizenchain/node`、`citizenchain/runtime`、链脚本或链文档
- 负责 `citizenchain` 全域
- 包括 `runtime`、`node` 原生节点、桌面节点、node 前端与打包流程
- 若 `runtime` 变更会影响 `citizenapp` 在线端或 `citizenwallet` 公民钱包二维码签名/验签兼容性，必须联动 `Mobile Agent`；不得只改单侧 runtime

## OnChina Agent

- 触发条件：任务涉及 `citizenchain/onchina`、注册局身份、CID 号、行政区、机构登记、管理后台、扫码验签或链侧凭证
- 负责 OnChina 后端、前端、数据库、扫码签名、公开查询与文档

## Mobile Agent

- 触发条件：任务涉及 `citizenapp`、Flutter 移动端交互、扫码、签名或 Isar 本地存储
- 负责 `citizenapp`
- 负责 Flutter 移动端与 Isar 本地存储
- 当 `runtime` 兼容性变更触发扫码签名联动时，同时负责 `citizenapp` 在线端与 `citizenwallet` 公民钱包的二维码签名、payload 解码、`spec_version` / `pallet_registry` 对齐

## Wallet Agent

- 触发条件：任务涉及 `citizenwallet`、离线签名、公民钱包确认弹窗、扫码识别或签名响应
- 负责公民钱包 Flutter 端、冷签名和 QR_V1 签名确认

## Review Agent

- 由 Claude 承担
- 负责 bug、安全风险、回归、文档遗漏与中文注释检查

## Release Agent

- 由 GitHub Actions 承担
- 负责构建、打包、发布
