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
- 2026-06-19:重新生成 `wuminapp/assets/admin_divisions/` 行政区包,manifest `version=1`,省 43、市 2938、镇 39728,SQLite SHA-256 为 `4f3313749fc5703cea5c2de60a2e4a26ed9c080ef2100b4f7b0773b29a502087`。
- 2026-06-19:执行 SFID 公权机构运行库 reconcile:`scopes=43 inserted=12332 updated=237011 account_inserted=498729 removed=12398`。
- 2026-06-19:执行 SFID 公权机构严格校验:`ok=true manifest_current=true target_count=249343 active_count=249343 missing=0 mismatched=0 missing_accounts=0 obsolete=0 catalog_hash=99c392297706637e8cec1826ca4c517eeadab7039f072498bdbb484d94580ea5`。
- 2026-06-19:后续任务 `20260619-wuminapp-public-all-gov` 已把 `wuminapp/assets/public_institutions/` 修正为完整公权目录,manifest `version=1`,43 省,共 249343 条公民端公权机构。
- 2026-06-19:公民端完整公权目录包含 `CITY_POLICE=2938`、`JY=2939`、`PROVINCE_RESERVE_BANK=43`,与 `check-gov --strict` 的 `target_count=249343` 对齐。

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
- SFID 真实 HTTP 接口 `GET /api/v1/app/public-institutions/version?province=伊犁省`:返回 `count=1687`,manifest_version 与公开包伊犁省分片一致。
- 后续任务 `20260619-wuminapp-public-all-gov` 已重新执行公权机构资产包 code 交叉检查:249343 条记录的省、市、镇 code 均能在行政区包中定位,`bad_count=0`。
- `git diff --check`:通过。
- 残留扫描:自有配置未再命中 `0.1.0` / `0.0.0` / 旧行政区版本号;剩余 `0.1.0` 命中均为第三方 Cargo/npm 依赖,剩余 `TS` 命中均为 TypeScript 文义。
