# 2026-06-23 node 冷签统一与激活记录清理

## 任务需求

- 修复 citizenchain 本机冷签交易 `InvalidTransaction::BadProof`。
- 修复管理员激活读取旧本地记录时报 `missing field institution_code`。
- 更新文档、完善中文注释、清理签名扫码残留实现。

## 改动范围

- `citizenchain/node/src/governance/`：统一冷签 payload、签名验签、extrinsic 构造路径。
- `citizenchain/node/src/admins/admin_management/`：清理旧 `org` 激活记录格式，不做旧格式兼容。
- `memory/01-architecture/qr/`：记录 QR_V1 生成、签名、验签唯一口径。
- `memory/05-modules/citizenchain/node/`：记录 node 侧治理签名和激活存储事实。

## 风险点

- 冷签二维码与链端交易构造必须使用同一套 runtime 类型，否则仍会出现 BadProof。
- 本地旧激活记录清理后需要用户重新扫码激活管理员身份。
- runtime 目录不在本次改动范围内。

## 验收

- `cargo check -p node`
- 检查残留手写 extrinsic/signing payload 拼接。
- 检查文档中的旧激活记录描述和 QR 签名口径。

## 完成记录

- 2026-06-23：node 冷签链交易改为 runtime `TxExtension + SignedPayload + UncheckedExtrinsic` 类型构造。
- 2026-06-23：链交易和管理员激活生成 QR 时写入后端内存 session，签名响应提交时按 request_id 一次性消费。
- 2026-06-23：签名响应提交前补齐 QR 过期、session payload hash、sr25519 本地验签。
- 2026-06-23：管理员激活旧 `org` 本地记录检测后清空，不做迁移兼容。
- 2026-06-23：已执行 `cargo check -p node`、`cargo test -p node governance::signing::tests`、`cargo test -p node admins_change`。
