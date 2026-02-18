# CIIC 系统技术方案（独立系统）

## 1. 方案结论
- 桌面端技术路线固定为：`Tauri + React + TypeScript + Vite`。
- CIIC 为独立联网系统，只通过标准接口与轻节点/区块链接入。
- 绑定环节人工执行，投票凭证环节自动执行。

## 2. 建设目标
- 支撑管理员人工录入文明档案索引号并绑定公钥。
- 支撑轻节点自动获取绑定凭证与投票凭证。
- 提供完整审计、权限隔离、密钥隔离、防重放能力。
- 不改区块链实现，仅对接既有链上验签能力。

## 3. 系统边界
1. 外部系统：公民档案管理系统（离线）
- 不属于 CIIC 研发范围。
- CIIC 仅依赖管理员可从该系统获取文明档案索引号。

2. CIIC 系统（本方案范围）
- 管理员桌面端：人工绑定审批、查询、审计。
- 在线后端：工单、绑定关系、凭证签发、风控。
- 签名服务：统一签发绑定/投票凭证。

3. 轻节点与区块链（外部对接）
- 轻节点调用 CIIC API 获取凭证。
- 区块链负责链上验签与防重放。

## 4. 技术栈与分层
1. 桌面端
- `Tauri`（桌面壳）
- `React + TypeScript + Vite`（管理界面）

2. 后端服务
- `Rust + Axum`
- `PostgreSQL`（主存储）
- `Redis`（可选：限流、短期缓存、任务状态）

3. 安全与密钥
- 独立 `Signer` 服务（Rust）
- 签名私钥不落业务 API 进程
- 支持 `key_id` 轮换

## 5. 模块设计
1. `ciic-desktop-admin`
- 工单列表、绑定确认、异常处理、操作审计查询。
- 录入校验：索引号格式、重复提示、公钥校验。

2. `ciic-api`
- 绑定申请、绑定确认、凭证签发、状态查询。
- 统一响应规范与错误码。

3. `ciic-signer`
- 生成 `nonce`、签名 payload、返回签名与过期时间。
- 记录签发流水与签名 key 版本。

4. `ciic-audit`
- 记录人工操作与关键自动动作。
- 支持按人、按时间、按公钥追溯。

## 6. 核心流程
### 6.1 人工绑定
1. 轻节点提交 `account_pubkey` 创建绑定申请。
2. 管理员在 CIIC 桌面端打开申请。
3. 管理员手工输入文明档案索引号并确认。
4. 系统校验唯一性后写入绑定关系。
5. 系统自动签发绑定凭证。
6. 轻节点拿凭证提交链上绑定交易。

### 6.2 自动投票凭证
1. 轻节点提交 `account_pubkey + proposal_id`。
2. 系统校验绑定状态有效。
3. 系统自动签发投票凭证（含一次性 `nonce`）。
4. 轻节点提交投票交易，链上校验并防重放。

## 7. 状态机
1. 绑定申请状态 `bind_requests.status`
- `PENDING`
- `APPROVED`
- `REJECTED`
- `EXPIRED`

2. 绑定关系状态 `archive_bindings.status`
- `ACTIVE`
- `UNBOUND`
- `SUSPENDED`（可选：风控冻结）

## 8. 数据模型
1. `bind_requests`
- `request_id`, `account_pubkey`, `status`, `created_at`, `expires_at`, `created_ip`

2. `archive_bindings`
- `binding_id`, `archive_index`, `account_pubkey`, `ciic_identity_hash`, `status`, `bound_by`, `bound_at`, `updated_at`
- 约束：`archive_index` 唯一、`account_pubkey` 唯一

3. `credential_issues`
- `issue_id`, `credential_type(BIND|VOTE)`, `account_pubkey`, `proposal_id`, `nonce_hash`, `key_id`, `issued_at`, `expired_at`

4. `audit_logs`
- `log_id`, `operator_id`, `action`, `target_type`, `target_id`, `result`, `detail`, `created_at`

## 9. API 规范（建议）
1. `POST /api/v1/bind/request`
- 入参：`account_pubkey`
- 出参：`request_id`, `status`

2. `POST /api/v1/admin/bind/confirm`
- 入参：`request_id`, `archive_index`
- 出参：`status`
- 鉴权：管理员登录 + 2FA

3. `GET /api/v1/bind/credential`
- 入参：`request_id`, `account_pubkey`
- 出参：`identity_hash`, `nonce`, `signature`, `key_id`, `expired_at`

4. `POST /api/v1/vote/credential`
- 入参：`account_pubkey`, `proposal_id`
- 出参：`identity_hash`, `nonce`, `signature`, `key_id`, `expired_at`

5. 响应统一
- 成功：`{ code: 0, message: "ok", data: ... }`
- 失败：`{ code: <non-zero>, message: "...", trace_id: "..." }`

## 10. 安全方案
1. 权限控制
- RBAC：录入员、复核员、审计员、系统管理员。
- 高风险动作可配置双人复核。

2. 输入与风控
- 索引号格式校验与长度校验。
- 同源频控、失败重试限制、异常行为告警。

3. 密钥与签名
- API 与 signer 进程隔离。
- 私钥最小暴露，支持轮换和吊销。
- 凭证短时效 + `nonce` 一次性。

4. 审计
- 人工绑定全流程日志不可删改。
- 支持导出审计报表（时间、人、动作、结果）。

## 11. 非功能要求
- 可用性：核心 API 月可用性目标 `>= 99.9%`
- 时延：凭证签发 P95 `< 300ms`（不含客户端网络）
- 可观测：日志、指标、追踪三件套
- 备份：数据库日备份 + 审计日志归档

## 12. 实施计划
1. M1（2-3 周）
- 管理员登录、绑定申请、人工确认、绑定凭证签发
- 基础审计和唯一约束落地

2. M2（2 周）
- 投票凭证自动签发
- 风控规则与限流

3. M3（2 周）
- 密钥轮换、2FA、双人复核、审计报表

## 13. 验收标准
- 管理员可手工绑定“索引号-公钥”。
- 同一索引号不可多绑，同一公钥不可多绑。
- 投票凭证签发全自动且可追溯。
- 关键操作均有审计记录。
- CIIC 对接链流程可用，且不修改链上业务逻辑。
