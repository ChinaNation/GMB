# SFID `/api/v1/app/voters/count` 响应加 (province, signer_admin_pubkey)

- 状态:open
- 创建日期:2026-05-02
- 模块:`sfid/backend`
- 关联 ADR:`memory/04-decisions/ADR-008-sheng-admin-3tier-and-key-admin-removal.md`
- 上游:step2d(commit b4bb76e)wuminapp `PopulationSnapshotResponse` 已强校验 `province + signerAdminPubkey`

## 任务需求

step2d 后 wuminapp `ApiClient.fetchPopulationSnapshot` 解析期强制要求 `province + signerAdminPubkey` 字段。SFID 后端 `/api/v1/app/voters/count` 当前 JSON 不返回这 2 字段 → 在线端 throw 。本卡补全。

## 影响范围

- `sfid/backend/chain/joint_vote/handler.rs`(或 app_voters_count handler 实际位置,grep 找)
- 响应 DTO struct 加 2 字段
- 数据来源:登录 session 的 `admin_province` + `unlocked_admin_pubkey`
- 现有测试同步;新加 ≥ 1 测试断言响应含新字段

## 验收

- `cargo check -p sfid-backend` 全绿
- `cargo test -p sfid-backend` ≥ baseline 79 + 1 新测试
- 手动 curl `/api/v1/app/voters/count` 响应含 `province` + `signer_admin_pubkey`(0x 小写 hex)

## 工作量

~30 行 + 1 测试,~0.2 round。
