# 任务卡：全面仔细检查 cpms 系统是否存在安全漏洞、改进点、功能需求实现偏差、中文注释/技术文档缺失，以及需要清理的残留。

- 任务编号：20260328-164533
- 状态：open
- 所属模块：cpms
- 当前负责人：Codex
- 创建时间：2026-03-28 16:45:33

## 任务需求

全面仔细检查 cpms 系统是否存在安全漏洞、改进点、功能需求实现偏差、中文注释/技术文档缺失，以及需要清理的残留。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/01-architecture/cpms/README.md
- memory/01-architecture/cpms/CPMS_TECHNICAL.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/07-ai/module-checklists/cpms.md
- memory/07-ai/module-definition-of-done/cpms.md

## 必须遵守

- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通
- 改代码后必须更新文档和清理残留

## 输出物

- 审查结论
- 风险点
- 改进建议
- 文档/残留清单

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
- 已检查系统文档、backend、数据库 schema/migration、部署脚本与模块文档
- 已执行 `cargo fmt -- --check`
- 已执行 `cargo check`
- 已执行 `cargo test`

## 审查结论

- 未发现“普通外部用户直接远程拿下系统”的单点高危漏洞
- 但系统安全与需求收口存在 4 个明显问题，尤其是“敏感数据未加密落库”和“登录密钥体系未按设计分离”

## 主要问题

1. 敏感数据未按需求加密存储
   - 系统总文档明确要求“敏感数据加密存储”
   - 当前 `archives` 表中的 `full_name`、`birth_date`、`gender_code`、`height_cm`、`passport_no` 直接明文入库
   - 初始化生成的 `qr_sign_keys.secret` 也直接明文入库

2. 登录密钥体系与二维码签名密钥未分离
   - 总文档要求“管理员登录公钥体系”与“二维码签名私钥体系”必须分离
   - 当前登录二维码里的 `sys_pubkey/sys_sig` 实际直接使用 `qr_sign_keys` 中的激活密钥生成
   - `CPMS_LOGIN_SYSTEM_KEY_* / CPMS_LOGIN_SYS_CERT` 这条设计链路没有真正落地

3. 技术文档不完整
   - `memory/05-modules/cpms/` 目前只有 `initialize/login/dangan` 三份模块技术文档
   - `super_admin`、`operator_admin`、`authz` 没有独立技术文档
   - `CPMS_TECHNICAL.md` 的模块索引直接指向源码文件，不是模块文档

4. 部署与清理还有残留
   - 安装脚本仍写入 `CPMS_INSTALL_FILE=/var/lib/cpms/runtime/cpms_install_init.json`
   - 当前仓库里没有任何代码再读取这个变量或这个文件，属于旧方案残留
   - backend 默认监听 `0.0.0.0:8080`，部署默认是纯 HTTP，只适合严格内网隔离环境，安全加固不足

## 验证结果

- `cargo fmt -- --check`：通过
- `cargo check`：通过
- `cargo test`：通过，7 个测试全部通过

## 相关证据

- 敏感数据加密要求：`memory/01-architecture/cpms/README.md`
- 敏感档案明文入库：`cpms/backend/src/operator_admin/mod.rs`
- 二维码签名私钥明文入库：`cpms/backend/src/initialize/mod.rs`、`cpms/backend/db/schema.sql`
- 登录二维码复用 `qr_sign_keys`：`cpms/backend/src/login/mod.rs`
- 模块文档缺失：`memory/01-architecture/cpms/CPMS_TECHNICAL.md`、`memory/05-modules/cpms/`
- 安装残留变量：`cpms/deploy/linux/install_host.sh`
