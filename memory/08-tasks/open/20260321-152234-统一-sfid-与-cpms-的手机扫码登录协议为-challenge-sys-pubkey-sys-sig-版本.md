# 任务卡：统一 SFID 与 CPMS 的手机扫码登录协议为 challenge+sys_pubkey+sys_sig 版本

- 任务编号：20260321-152234
- 状态：open
- 所属模块：wuminapp / sfid / cpms
- 当前负责人：Codex
- 创建时间：2026-03-21 15:22:34

## 任务需求

按已确认口径统一 SFID 与 CPMS 的手机扫码登录协议：保留 `proto/system/challenge/issued_at/expires_at/sys_pubkey/sys_sig`，删除 `request_id`、`nonce` 与 CPMS 证书链字段，并同步更新 App、SFID、CPMS 三边实现、测试与文档。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- wuminapp/WUMINAPP_TECHNICAL.md
- sfid/SFID_TECHNICAL.md
- cpms/CPMS_TECHNICAL.md

## 模块模板

- 模板来源：多模块联合任务（手工建卡）

### 默认改动范围

- `wuminapp`
- `sfid`
- `cpms`

### 先沟通条件

- 修改认证流程
- 修改关键交互路径
- 修改跨系统协议

## 必须遵守

- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通
- 改代码后必须更新文档和清理残留

## 输出物

- 代码
- 中文注释
- 文档更新
- 残留清理

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
- 已先完成 `wuminapp` App 侧登录协议收敛：
  - 登录挑战字段收敛为 `proto/system/challenge/issued_at/expires_at/sys_pubkey/sys_sig`
  - 删除 `request_id`、`nonce` 与 CPMS 证书链字段
  - 登录回执改为按 `challenge` 回传与防重放
  - 系统验签改为仅校验 `sys_pubkey + sys_sig`
- 已通过 App 侧目标测试：
  - `test/wallet/sign_service_test.dart`
  - `test/signer/system_signature_verifier_test.dart`
  - `test/qr/qr_router_test.dart`
- 已完成 `sfid` 后端对齐：
  - 登录二维码改为输出 `proto/system/challenge/issued_at/expires_at/sys_pubkey/sys_sig`
  - 系统签名原文改为 `WUMIN_LOGIN_V1.0.0|system|challenge|issued_at|expires_at|sys_pubkey`
  - 登录完成接口兼容 `challenge|challenge_id|request_id` 回传
  - 已跑通 `cargo test --offline qr_login --manifest-path /Users/rhett/GMB/sfid/backend/Cargo.toml`
- 已完成 `cpms` 后端对齐：
  - 登录二维码改为输出 `proto/system/challenge/issued_at/expires_at/sys_pubkey/sys_sig`
  - 系统签名原文改为 `WUMIN_LOGIN_V1.0.0|system|challenge|issued_at|expires_at|sys_pubkey`
  - 登录完成接口兼容 `challenge|challenge_id` 回传
  - 已完成 `cargo test --offline login --manifest-path /Users/rhett/GMB/cpms/backend/Cargo.toml` 编译级验证
- 已同步更新架构/模块文档，明确：
  - 登录协议与链上签名协议分离
  - 登录协议不再使用 `request_id`、`nonce`、`sys_cert`
  - 登录采用“系统先签二维码 + 管理员钱包再签 challenge”的双层签名模型
