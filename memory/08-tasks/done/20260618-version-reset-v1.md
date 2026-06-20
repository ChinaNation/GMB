# 任务卡：重新创世后把仓库内自有软件、协议和数据发布版本恢复到版本1，保持协议基线 V1 不改名，同步更新文档、清理残留并完成验证

- 任务编号：20260618-200542
- 状态：done
- 所属模块：cross
- 当前负责人：Codex
- 创建时间：2026-06-18 20:05:42

## 任务需求

重新创世后把仓库内自有软件、协议和数据发布版本恢复到版本1，保持协议基线 V1 不改名，同步更新文档、清理残留并完成验证

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/07-ai/workflow.md
- memory/07-ai/document-boundaries.md
- memory/07-ai/definition-of-done.md
- memory/07-ai/pre-submit-checklist.md
- memory/07-ai/unified-protocols.md
- memory/07-ai/unified-naming.md
- memory/05-modules/citizenchain/node/NODE_TECHNICAL.md
- memory/07-ai/chainspec-frozen.md
- memory/05-modules/sfid/backend/china/CHINA_TECHNICAL.md
- memory/04-decisions/ADR-021-admin-district-single-source.md
- memory/01-architecture/sfid/SFID_TECHNICAL.md
- memory/05-modules/sfid/backend/BACKEND_LAYOUT.md
- memory/01-architecture/cpms/CPMS_TECHNICAL.md
- memory/01-architecture/wuminapp/WUMINAPP_TECHNICAL.md
- memory/07-ai/module-checklists/citizenchain.md
- memory/07-ai/module-checklists/sfid.md
- memory/07-ai/module-checklists/cpms.md
- memory/07-ai/module-checklists/wuminapp.md
- memory/07-ai/module-definition-of-done/citizenchain.md
- memory/07-ai/module-definition-of-done/sfid.md
- memory/07-ai/module-definition-of-done/cpms.md
- memory/07-ai/module-definition-of-done/wuminapp.md

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

## 版本边界

- 用户确认“归零”含义是恢复到版本 1。
- `WUMIN_QR_V1`、`/api/v1`、`SFID_CPMS_V1`、`sfid-cpms-v1` 保持版本 1 基线，不改名。
- 第三方依赖、vendored smoldot/libp2p/GRANDPA 版本不归入本次自有版本恢复范围。
- `citizenchain/runtime` 当前 `spec_version` 已是 1，本任务不产生 runtime diff。
- 行政区与公权机构派生数据包恢复为重新创世基线版本 1。

## 实施记录

- 任务卡已创建
- 2026-06-18:修正任务卡文件名为短 slug `20260618-version-reset-v1.md`。
- 2026-06-19:自有软件包版本恢复到 `1.0.0` / `1.0.0+1`;第三方依赖、vendored smoldot/libp2p/GRANDPA 版本不纳入本次恢复范围。
- 2026-06-19:`sfid/backend/china/china.sqlite` 重新创世基线恢复为行政区版本 1;`admin_division_change_log` 已清空,`admin_division_versions` 仅保留版本 1。
- 2026-06-19:曾重新生成行政区包并执行 SFID 公权机构 reconcile/strict;该批统计已在 2026-06-20 行政区重新创世清理后作废,不再作为当前验收口径。
- 2026-06-20:后续行政区重新创世清理再次覆盖本任务资产包结果:当前 wuminapp 行政区包为 43 省、2872 市、39227 镇;当前 SFID 公权机构 strict check 为 `target_count=245716 active_count=245716 missing=0 mismatched=0 missing_accounts=0 obsolete=0`;当前 wuminapp 公权机构包为 43 省、245716 条,包含 `CITY_POLICE=2872`、`JY=2873`,code 交叉检查 `bad_count=0`。

## 验收记录

- `python3 sfid/backend/china/check_code_immutable.py`:通过。
- `cargo check --manifest-path cpms/backend/Cargo.toml`:通过,`cpms-backend v1.0.0`。
- `cargo check --manifest-path sfid/backend/Cargo.toml`:通过,`sfid-backend v1.0.0`。
- `cargo check --manifest-path wuminapp/rust/Cargo.toml`:通过,`smoldot_ffi v1.0.0`。
- `npm run build` in `cpms/frontend`:通过,`cpms-web@1.0.0`。
- `npm run build` in `sfid/frontend`:通过,`sfid-desktop@1.0.0`。
- `npm run build` in `website`:通过,`website@1.0.0`。
- `flutter test test/citizen/public/admin_division_test.dart test/citizen/public/public_institution_bundle_loader_test.dart test/citizen/public/public_provinces_test.dart`:通过。
- `scripts/check-chainspec-frozen.sh`:通过,wuminapp chainspec 创世部分等于链端 SSOT。
- SFID 真实 HTTP 接口 `GET /api/v1/app/public-institutions?province=伊犁省&page_size=1`:返回 `catalog_status=OK`,manifest_version 与公开包伊犁省分片一致。
- 2026-06-20 当前资产包已再次重生成,以 245716 条记录和 `bad_count=0` 为当前验收口径。
- `git diff --check`:通过。
- 残留扫描:自有配置未再命中 `0.1.0` / `0.0.0` / 旧行政区版本号;剩余 `0.1.0` 命中均为第三方 Cargo/npm 依赖,剩余 `TS` 命中均为 TypeScript 文义。
