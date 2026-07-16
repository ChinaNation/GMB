# 20260706 FRG 省组绑定补齐机构 CID（2026-07-15 已升级）

## 状态

- 已完成

## 背景

- OnChina 本地节点绑定 FRG 省组管理员身份时，链上省组权限存在，但旧绑定缺少 FRG 机构 CID。
- 2026-07-15 起新增公民占号和身份上链统一使用注册局 `actor_cid_number + origin`，主账户不再表达注册局身份或授权。
- 本次修复只处理 OnChina 绑定候选和 active binding 元数据补齐,不修改 `citizenchain/runtime/`。

## 目标

- FRG 省组绑定继续保留省级办理范围。
- FRG 省组绑定必须从链上投影的 FRG 主体补齐 `institution_cid_number`、`cid_full_name`、`cid_short_name`；具体账户仅在账户型交易中单独携带。
- 已存在的 active binding 在读取时可以自愈并回写缺失字段。
- 新增公民占号不再因为 FRG 绑定缺机构主账户失败。
- 更新相关技术文档,补充必要中文注释,清理本次残留。

## 影响范围

- `citizenchain/onchina/src/auth/repo.rs`: active binding 元数据补齐和自愈回写。
- `citizenchain/onchina/src/auth/login/onchain_gate.rs`: 登录候选生成时补齐 FRG CID 元数据。
- `citizenchain/onchina/src/main.rs`: 稳定错误码兜底。
- `citizenchain/onchina/frontend/utils/notice.ts`: 前端错误提示兜底。
- `memory/04-decisions/ADR-030-onchina-multi-institution-console.md`: 记录 FRG 绑定规则。
- `memory/05-modules/citizenchain/onchina/BACKEND_TECHNICAL.md`: 更新 OnChina 后端技术说明。

## 验收

- `cargo check --manifest-path citizenchain/Cargo.toml -p onchina` 通过。
- `cargo build --manifest-path citizenchain/Cargo.toml -p onchina` 通过,本地 OnChina 已重启加载新二进制。
- `npm --prefix citizenchain/onchina/frontend run build` 通过。
- 本地真实服务 `https://127.0.0.1:8964/api/v1/health` 返回 `UP`;后台 OnChina 运行在独立 `screen` 会话 `onchina-frg-fix`。
- 真实 PostgreSQL active binding 当时已补齐；2026-07-15 当前表结构已删除主账户身份字段，只保留 CID 和管理员钱包：
  - `candidate_id=FRG:FRG:475a`
  - `institution_code=FRG`
  - `institution_cid_number=ZS001-FRG07-249474503-2026`
  - `frg_province_code=0x475a`
  - `scope_province_name=贵州省`
- 使用真实本地 HTTP `POST /api/v1/admin/citizens` 验收占号 prepare,请求返回 `HTTP 200` 和 `request_id=citizen-occupy-68ae64df-f608-4a2f-94cd-22affb2450de`,不再返回“当前注册局缺少机构主账户绑定”。
- 验收残留已清理:删除上述 `chain_sign_sessions` 1 条和对应 `audit` 1 条;复查 `citizens=0`、`chain_sign_sessions=0`、该验收 CID 审计残留为 `0`。
