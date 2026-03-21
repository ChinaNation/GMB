# SFID 系统 README

* 每个模块下面都有一个技术文档；
* 每次编写代码后都要更新技术文档；
* 需要其他模块和仓库联测的同步更新对应的技术文档；
* 编写代码的同时必须完善中文注释；
* 产品级文档位于 `memory/01-architecture/sfid/`，模块级文档位于 `memory/05-modules/sfid/`。

SFID 是在线身份绑定系统，用于接收线下二维码并服务区块链身份校验。
安全基线（2026-03）：后端已移除默认开发密钥/Token，关键环境变量缺失时会拒绝启动。

核心能力：
- 管理员在 Web 前端扫码线下工作人员提供的 CPMS 签名二维码完成绑定。
- 管理员线下受理后在系统执行解绑。
- 自动响应区块链查询（可投票人数、绑定有效性）。
- 投票资格以 CPMS 二维码状态字段为准（`NORMAL`/`ABNORMAL`）。
- 绑定/解绑结果可回传区块链。
- 公开身份查询需携带 `x-public-search-token`。
- 支持扫码登录：仅 SFID 管理员允许登录，非管理员扫码直接拒绝。
- 前端 UI 体系与 CPMS、CitizenNode 保持统一，当前实现基于 Ant Design。

## 管理员体系
- 密钥管理员（`KEY_ADMIN`）：固定 3 个（一主两备槽位映射）。
- 超级管理员（`SUPER_ADMIN`）：固定 43 个（每省 1 个）。
- 操作管理员（`OPERATOR_ADMIN`）：数量不限，由超级管理员增删改查。
- 权限边界：
1. 密钥管理员：密钥轮换管理、超级管理员替换、全局管理能力。
2. 超级管理员：可管理操作管理员，且可执行绑定/解绑/查询。
3. 操作管理员：仅可执行绑定/解绑/查询，不可管理管理员账号。
4. 三类管理员使用同一套前端页面与登录流程；菜单按角色显示，权限以后端 RBAC 为准。
5. 非管理员扫码登录会被拒绝，只有 SFID 管理员可登录。

## 登录机制（无用户名密码）
1. 用户点击“生成登录二维码”，系统展示一次性 challenge 登录二维码。
2. 手机扫码后完成签名，并将 `admin_pubkey + signature` 回传 SFID。
3. 系统校验公钥与签名：
   - 管理员公钥：进入管理员模式。
   - 非管理员公钥：拒绝登录。
   - 禁用管理员或签名失败：拒绝登录。
4. 登录页轮询 challenge 结果，成功后自动登录。
5. 权限校验以后端 RBAC 为准，前端菜单控制仅用于展示。

## 仓库结构
- `frontend/`：管理员前端网站（React + TypeScript + Vite + Ant Design）
- `backend/`：后端 API（Rust + Axum）
- `backend/db/`：数据库迁移与初始化数据
- `backend/scripts/`：后端脚本（联调/冒烟）
- `backend/tests/`：后端测试（integration/e2e）
- `deploy/`：环境部署配置（dev/staging/prod）
- `memory/01-architecture/sfid/SFID_TECHNICAL.md`：完整技术开发文档（流程、架构、数据、接口、安全、验收）

## 快速启动
### 0) 启动 PostgreSQL（本地开发）
```bash
docker run -d --name sfid-pg \
  -e POSTGRES_USER=sfid \
  -e POSTGRES_PASSWORD=sfid_dev_pwd \
  -e POSTGRES_DB=sfid_dev \
  -p 5432:5432 \
  postgres:16
```

### 1) 执行数据库迁移
```bash
docker exec -i sfid-pg psql -U sfid -d sfid_dev < backend/db/migrations/001_init_sfid.sql
docker exec -i sfid-pg psql -U sfid -d sfid_dev < backend/db/migrations/002_runtime_store.sql
docker exec -i sfid-pg psql -U sfid -d sfid_dev < backend/db/migrations/003_admin_role_partition.sql
docker exec -i sfid-pg psql -U sfid -d sfid_dev < backend/db/migrations/004_finalize_no_runtime_store.sql
docker exec -i sfid-pg psql -U sfid -d sfid_dev < backend/db/migrations/005_drop_sfid_prefix.sql
docker exec -i sfid-pg psql -U sfid -d sfid_dev < backend/db/migrations/006_super_admin_catalog.sql
docker exec -i sfid-pg psql -U sfid -d sfid_dev < backend/db/migrations/007_refresh_admin_views.sql
docker exec -i sfid-pg psql -U sfid -d sfid_dev < backend/db/migrations/008_chain_idempotency_reward_state.sql
docker exec -i sfid-pg psql -U sfid -d sfid_dev < backend/db/migrations/009_runtime_cache_and_pii_encryption.sql
docker exec -i sfid-pg psql -U sfid -d sfid_dev < backend/db/migrations/010_drop_plaintext_pii_columns.sql
```

### 2) 启动后端
```bash
cd backend
export DATABASE_URL='postgres://sfid:sfid_dev_pwd@127.0.0.1:5432/sfid_dev'
export SFID_SIGNING_SEED_HEX='<required>'
export SFID_KEY_ID='sfid-master-v1'
export SFID_CHAIN_TOKEN='<required>'
export SFID_CHAIN_SIGNING_SECRET='<required>=32chars'
export SFID_PUBLIC_SEARCH_TOKEN='<required>'
export SFID_RUNTIME_META_KEY='<required-runtime-meta-encryption-key>'
cargo run
```
默认地址：`http://127.0.0.1:8899`
说明：区块链接口必须携带请求头 `x-chain-token` 和 `x-chain-signature`。

### 2.1) 一键启动前后端（开发）
```bash
./start-dev.sh
```
说明：脚本会加载 `.env.dev.local`，并同时启动后端与前端开发服务。

### 3) 启动前端开发模式
```bash
cd frontend
npm install
npm run dev
```
默认地址：`http://127.0.0.1:5179`

### 4) 健康检查
```bash
curl http://127.0.0.1:8899/api/v1/health
```
返回 `{"code":0,...}` 表示后端正常。

## 数据库说明（当前）
- 本项目当前使用 PostgreSQL。
- 运行态拆分为：
1. `runtime_cache_entries`：按键分片运行态缓存（JSONB）。
2. `runtime_misc`：运行杂项（JSONB）。
3. `runtime_meta`：签名种子与公钥元信息（JSONB）。
- 管理员与密钥采用结构化分表：
1. `admins`
2. `provinces`
3. `super_admin_scope`
4. `operator_admin_scope`
5. `key_admin_keyring`
- 兼容视图：
1. `v_key_admins`
2. `v_super_admins`
3. `v_operator_admins`
- 链路一致性与防重放表：
1. `chain_idempotency_requests`
2. `binding_unique_locks`
3. `bind_reward_states`
- 已下线内容：
1. `backend/data/runtime_state.json`
2. `runtime_store`

## 常见故障排查
- 前端提示 `Failed to fetch` 或 `curl` 返回 `Empty reply from server`：
1. 检查后端窗口是否 panic。
2. 检查 8899 是否被旧进程占用：`lsof -nP -iTCP:8899 -sTCP:LISTEN`。
- 启动报 `AddrInUse`：
1. 杀掉旧进程：`kill -9 $(lsof -ti tcp:8899)`。
2. 重新启动后端。
- 启动报 `connect postgres failed`：
1. 检查容器是否运行：`docker ps --filter name=sfid-pg`。
2. 检查 `DATABASE_URL` 是否正确。

## 业务流程（当前口径）
1. 用户在区块链前端提交公钥到 SFID。
2. 用户在线下办理后，由工作人员从 CPMS 获取“签名 + 档案号”二维码。
3. SFID 管理员在 Web 前端先扫码验签，再用该次扫码返回的 `qr_id` 执行确认绑定（不可手工绕过）。
4. SFID 将档案号与公钥绑定成功后回传区块链。
5. 用户如需解绑，必须线下找 SFID 管理员处理。
6. 公开查询页通过服务端配置的查询 Token 对外提供档案号、身份识别码、公钥地址查询能力。

说明：CPMS 是完全离线独立系统，与 SFID 无在线接口直连。

## CPMS 二维码约定（v1）
- 档案号 `archive_no` 作为唯一用户标识。
- `archive_no` 结构固定：`省2 + 市3 + 校验1 + 随机9 + 日期8(YYYYMMDD)`。
- 省市代码来源：与 CPMS 同步使用 `sheng_cities` 数据。
- 校验位算法与 SFID `sfid_code` 一致：`BLAKE2b` 摘要字节和 `mod 10`。
- `issuer_id` 固定为 `cpms`。
- 签名算法固定 `sr25519`。
- 机构初始化必须先由 SFID 超级管理员在机构页生成机构身份识别码（`site_sfid`）及 SFID 签名初始化二维码。
- CPMS 使用该初始化二维码完成首次安装初始化，再生成机构公钥登记二维码（含 `site_sfid + 3把公钥 + init_qr_payload + checksum_or_signature`）。
- SFID 超级管理员扫码录入公钥登记二维码后，该机构公钥才生效（会校验是否由 SFID 签发二维码初始化得到）。
- 可信闭环成立条件：`SFID 初始化二维码签发 -> CPMS 初始化 -> SFID 录入机构公钥成功(ACTIVE)`；闭环完成后，该机构后续出具的公民档案二维码与状态二维码才被 SFID 接受。
- 拒绝语义：若验签失败、机构未登记、机构非 `ACTIVE`、或 `init_qr_payload` 链路不一致，SFID 必须拒绝对应 CPMS 二维码。
- 公民档案二维码包含 `sign_key_id + signature`，由该机构 `sign_key_id` 对应私钥生成。
- CPMS 不保存 SFID 公钥（当前版本）。
- 用户投票资格状态由 CPMS 二维码提供：`NORMAL` 可投票，`ABNORMAL` 不可投票。
- 机构管理权限仅超级管理员开放，密钥管理员与操作管理员不可使用机构管理功能。

## CPMS 联调脚本（开发）
- 生成公民绑定二维码（含初始状态）：
```bash
./backend/scripts/gen_cpms_qr_dev.py citizen --site-sfid SITE001 --archive-no ARCHIVE001 --sign-pubkey DEMO_PUBKEY_A --status NORMAL
```
- 生成状态变更二维码（供操作管理员扫码）：
```bash
./backend/scripts/gen_cpms_qr_dev.py status --site-sfid SITE001 --archive-no ARCHIVE001 --status ABNORMAL --sign-pubkey DEMO_PUBKEY_A
```
- 生成机构公钥登记二维码：
```bash
./backend/scripts/gen_cpms_qr_dev.py register --site-sfid SITE001 --pubkey-1 DEMO_PUBKEY_A --pubkey-2 DEMO_PUBKEY_B --pubkey-3 DEMO_PUBKEY_C
```

## 区块链自动接口（无管理员参与）
- `GET /api/v1/chain/voters/count`：查询当前可投票公民数（仅统计 `NORMAL` 且绑定有效用户）。
- `GET /api/v1/chain/voters/count`：返回 `as_of` 与 `eligible_total` 同一统计快照时间点。
- `POST /api/v1/chain/binding/validate`：校验档案号-公钥绑定是否有效，并返回是否具备投票资格。
- `POST /api/v1/chain/reward/ack`：区块链回执绑定奖励处理结果（成功/失败）。
- `GET /api/v1/chain/reward/state`：查询某公钥绑定奖励状态机当前状态。
- `POST /api/v1/bind/request`：提交公钥绑定申请（支持回调字段，见下）。
- `GET /api/v1/bind/result`：查询某公钥绑定结果；绑定成功后返回持久化 Runtime 凭证，重复查询不会生成新 `nonce`。
- `GET /api/v1/bind/result`：`signature` 为 Runtime 凭证签名，`sfid_signature` 为历史兼容字段（旧 JSON 绑定证明签名）。
- `POST /api/v1/vote/verify`：查询公钥当前投票资格（`NORMAL` 可投票，`ABNORMAL` 不可投票）。
- 接口鉴权：仅允许区块链调用方访问，请求必须携带：
  - `x-chain-token`
  - `x-chain-request-id`
  - `x-chain-nonce`
  - `x-chain-timestamp`（Unix 秒，默认 5 分钟内有效）
  - `x-chain-signature`（必填）
- 防重放与幂等：同一路由下重复 `nonce` 或重复 `request-id` 会被拒绝。
- 绑定凭证刷新规则：当前 signer 公钥或 `key_id/key_version/alg` 变化后，后端会自动重签发并覆盖旧 Runtime 绑定凭证。
- 链签名原文（换行拼接）：
  - `route=<route_key>`
  - `request_id=<x-chain-request-id>`
  - `nonce=<x-chain-nonce>`
  - `timestamp=<x-chain-timestamp>`
  - `fingerprint=<request_fingerprint>`
  - 签名算法：`hex(blake2b_mac_256(blake2b_256(SFID_CHAIN_SIGNING_SECRET), payload))`

### 绑定成功回调（新增）
- `POST /api/v1/bind/request` 可选字段：
  - `callback_url`：绑定成功后回调地址。
  - `client_request_id`：业务方关联 ID（可选）。
- 回调 Bearer Token 统一通过后端环境变量 `SFID_BIND_CALLBACK_AUTH_TOKEN` 配置，不再接受请求体明文传入。
- 若未传 `callback_url`，系统会尝试使用环境变量 `SFID_BIND_CALLBACK_URL`。
- `callback_url` 安全限制：
  - 默认仅允许 `https://`；仅开发联调可通过 `SFID_ALLOW_INSECURE_CALLBACK_HTTP=true` 放开 `http://`。
  - 禁止 `localhost` 与私网/本地 IP 字面量（防 SSRF）。
  - 可用 `SFID_CALLBACK_ALLOWED_HOSTS`（逗号分隔，支持 `*.example.com`）限制允许回调域名。
- 回调失败会自动重试（指数退避，最多 5 次），并记录审计日志 `BIND_CALLBACK`。
- 回调体包含 `callback_attestation`（SFID 对回调内容签名）；HTTP Header 同步返回：
  - `x-sfid-callback-signature`
  - `x-sfid-callback-key-id`
- 区块链可用 `GET /api/v1/attestor/public-key` 获取验签公钥。

### 已实现的稳定性增强
- 幂等与防重放：链路接口统一 `request_id/nonce/timestamp` 校验，并写入数据库幂等表 `chain_idempotency_requests`。
- 并发一致性：绑定确认前落库 `binding_unique_locks`（`account_pubkey`、`archive_index` 双唯一），避免双绑竞态。
- 奖励状态机：`PENDING -> RETRY_WAITING/FAILED -> REWARDED`，由 `chain/reward/ack` 驱动。
- 投票资格短缓存：`/vote/verify` 5 秒缓存，状态变更/绑定变更即时失效。
- 可观测：`/api/v1/health` 仅返回基础存活字段（`service/status/checked_at`），内部指标仅保留在审计和内部日志。

## CORS 配置
- 默认仅放行本地开发源：
  - `http://127.0.0.1:5179`
  - `http://localhost:5179`
  - `http://127.0.0.1:5173`
  - `http://localhost:5173`
- 生产请配置 `SFID_CORS_ALLOWED_ORIGINS`（逗号分隔，禁止 `*`）。

## 公开查询接口（Token 鉴权）
- `GET /api/v1/public/identity/search?archive_no=...`
- `GET /api/v1/public/identity/search?identity_code=...`
- `GET /api/v1/public/identity/search?account_pubkey=...`
- 请求头必须包含：`x-public-search-token: <SFID_PUBLIC_SEARCH_TOKEN>`
- 返回：`found`、`archive_no`、`identity_code`、`account_pubkey`

## 测试说明（真实登录）
- 已关闭 `demo-sign` 登录测试入口。
- `backend/scripts/smoke.sh` 需要外部提供真实管理员 `ADMIN_TOKEN`（通过真实钱包登录获取）。

详细设计与完整接口定义见 `SFID_TECHNICAL.md`。
优化优先级清单（P0/P1/P2）见 `SFID_TECHNICAL.md` 第 15 章。
CPMS 对齐执行清单见 `SFID_TECHNICAL.md` 第 11 章（特别是 11.4、11.5）。
生产部署（主库 + 备库 + 应用一键安装）见 `deploy/prod/README_DEPLOY.md`。
