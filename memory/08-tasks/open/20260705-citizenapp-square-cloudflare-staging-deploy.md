# CitizenApp 广场 Cloudflare staging 预发布部署

## 任务需求

在阶段 8 已完成生产化部署前准备后，进入阶段 9：创建或绑定 Cloudflare staging 资源，部署广场 Worker 到 staging 环境，并完成远端 smoke 验收。

本阶段只处理 staging，不部署 production，不触碰 GitHub 远端，不修改 `citizenchain/runtime/`。Cloudflare token、R2 access key、R2 secret key、链 RPC 私密地址只允许保存在本机 Wrangler/Cloudflare 远端变量或 secret 中，不写入仓库。

## 建议模块

- CitizenApp 广场 Worker：`citizenapp/cloudflare/`
- 架构与任务文档：`memory/01-architecture/citizenapp/`、`memory/08-tasks/`

## 影响范围

- Cloudflare staging 资源：R2 bucket、D1 database、KV namespace。
- Worker staging 配置：`wrangler.toml` 的 staging D1/KV 真实资源 ID。
- Worker staging 远端变量/secret：链 RPC、R2 预签名变量。
- 文档：记录 staging 部署结果、远端资源名、验收结果和 production 前置条件。

## 主要风险点

- 未登录 Wrangler 或当前账号无 Cloudflare 权限。
- 误部署 production 或误改 production 数据。
- 把 Cloudflare token、R2 key、链 RPC 私密地址写入仓库。
- staging 资源 ID 占位值未替换导致部署失败。
- staging 未配置 R2 预签名变量时，真实上传授权不可用。

## 是否需要先沟通

- 否。用户已回复“执行”并单独确认允许创建本任务卡。若本机 Cloudflare 登录态或权限不足，则当前阶段只能输出阻塞结果。

## 预计修改目录

- `citizenapp/cloudflare/`：替换 staging 真实资源 ID、执行远端迁移和 staging deploy；涉及配置，不写密钥。
- `memory/01-architecture/citizenapp/`：记录 staging 部署结果和 production 前置条件；涉及文档。
- `memory/08-tasks/`：记录阶段 9 方案、执行、验收和残留清理；涉及文档。

## 分步骤技术方案

### 步骤 1：Wrangler 登录态和账号审计

- 恢复 Worker 本地依赖以运行 Wrangler。
- 执行 Wrangler 账号检查，确认当前本机是否已登录 Cloudflare。
- 不输出 token，不写入任何 secret。

### 步骤 2：创建或确认 staging 资源

- 创建或确认 R2 bucket：`citizenapp-square-media-staging`。
- 创建或确认 D1 database：`citizenapp-square-db-staging`。
- 创建或确认 KV namespace：`FEED_CACHE` staging 绑定。
- 只把公开资源 ID 写入 `wrangler.toml`；secret 不写仓库。

### 步骤 3：配置 staging 远端变量/secret

- 确认 `SQUARE_DEV_UPLOAD_PROXY=0`。
- 配置或确认 `CHAIN_URL`、两项 `CHAIN_ID / CHAIN_SECRET`、`R2_ACCOUNT_ID`、`R2_ACCESS_KEY_ID`、`R2_SECRET_ACCESS_KEY`、`R2_BUCKET_NAME` 只在 Cloudflare 远端 Secret/变量；链 RPC 三项必须成套配置。
- 如果缺少私密值且本机无法读取，记录阻塞，不伪造。

### 步骤 4：远端迁移和 staging 部署

- 执行 `npm --prefix citizenapp/cloudflare run migrate:staging`。
- 执行 `npm --prefix citizenapp/cloudflare run deploy:staging`。
- 获取 staging Worker URL。

### 步骤 5：远端 smoke 验收和清理

- `GET /health` 返回 `content_on_chain=false`。
- 未登录访问会员/上传接口按预期拒绝。
- 如 staging 已配置链 RPC 和 R2 预签名变量，执行 prepare 上传授权 smoke；否则记录配置缺失边界。
- 清理本地 `node_modules`、`.wrangler` 等生成目录。
- 执行 `git diff --check`。

## 当前执行状态

- [x] 阶段 9 任务卡创建
- [x] 步骤 1：Wrangler 登录态和账号审计
- [x] 步骤 2：创建或确认 staging 资源
- [x] 步骤 3：配置 staging 远端变量/secret
- [x] 步骤 4：远端迁移和 staging 部署
- [x] 步骤 5：远端 smoke 验收和清理

## 执行记录

- 已执行 `npm --prefix citizenapp/cloudflare install` 恢复本地 Wrangler 运行依赖；该依赖仅用于当前审计，验收清理时删除。
- 已执行 `npx wrangler whoami`：返回 `You are not authenticated. Please run wrangler login.`，当前本机无 Cloudflare 登录态。
- 已执行 `npx wrangler d1 list`、`npx wrangler r2 bucket list`、`npx wrangler kv namespace list`：均因非交互环境缺少 `CLOUDFLARE_API_TOKEN` 而失败。
- 2026-07-05 后续复查：用户完成 `wrangler login` 后，`npx wrangler whoami` 已成功识别 Cloudflare 账号 `ChinaNation`，本机 OAuth 登录态可用。
- 2026-07-05 后续复查：`npx wrangler d1 list` 成功执行，当前未列出已有 D1 数据库；`npx wrangler kv namespace list` 成功执行，当前返回空列表。
- 2026-07-05 后续复查：`npx wrangler r2 bucket list` 失败，Cloudflare API 返回 `Please enable R2 through the Cloudflare Dashboard. [code: 10042]`，说明当前账号尚未启用 R2。
- 2026-07-05 后续复查：用户启用 R2 后，`npx wrangler r2 bucket list` 已成功执行，R2 阻塞解除。
- 已创建 staging R2 bucket：`citizenapp-square-media-staging`。
- 已创建 staging D1 database：`citizenapp-square-db-staging`，`database_id=4ba85b05-657a-46ac-ab19-8bbd84fe850a`。
- 已创建 staging KV namespace：`staging-FEED_CACHE`，`id=91133becebc24f27bf10a00cb001f27e`。
- 已把 staging D1/KV 公开资源 ID 写入 `citizenapp/cloudflare/wrangler.toml`；R2 bucket 使用资源名绑定，不需要 secret 写仓库。
- 已在 staging vars 中显式配置 `R2_BUCKET_NAME=citizenapp-square-media-staging`，避免后续 R2 预签名误指向 production bucket。
- staging D1 已按当前 `0001_square_core.sql` 目标基线彻底重建，不保留独立 Chat 迁移链。
- 已执行 `npm --prefix citizenapp/cloudflare run deploy:staging`：staging Worker 已部署到 `https://citizenapp-square-api-staging.stews87-fawn.workers.dev`，当前版本 `d1b0f4b2-1cc8-4cd8-88f4-a5ac9a969211`。
- 已执行远端 D1 表检查：广场业务表、`chat_devices`、`chat_keypackages`、`chat_device_binding_nonces`、`chat_turn_credentials` 和 `square_browse_days` 存在，不存在聊天内容表。
- 已执行远端 smoke：`GET /health` 返回 200，`content_on_chain=false`；未登录访问 `/v1/square/membership` 返回 401 `missing_session`；`PUT /v1/square/uploads/dev-put` 返回 404 `dev_upload_proxy_disabled`，证明 staging 未开启本地开发上传代理。
- 已执行 `npm --prefix citizenapp/cloudflare run typecheck`：通过。
- 已执行 `npm --prefix citizenapp/cloudflare test`：通过，5 个测试文件、11 个测试用例通过。
- 用户已在 staging Worker 写入 R2 secrets；`npx wrangler secret list --env staging` 已确认存在 `R2_ACCOUNT_ID`、`R2_ACCESS_KEY_ID`、`R2_SECRET_ACCESS_KEY`。
- 已执行 staging 远端 R2 预签名上传 smoke：临时写入 KV session 与 D1 会员记录，调用 `uploads/prepare` 获取真实 R2 S3 预签名 URL，分别 PUT `manifest.json` 与 `media_001.png` 到 `citizenapp-square-media-staging`，再调用 `uploads/complete` 返回 `storage_state=completed` 和 `storage_receipt_id`。
- R2 smoke 测试数据已清理：删除临时 KV session、D1 会员/上传记录、远端 R2 `manifest.json` 与 `media_001.png` 测试对象；复查 `square_uploads` 对应 `post_id` 记录数为 0。
- 本阶段当时尚未配置链 RPC；因此只完成 Worker 远端部署、基础 smoke 和真实 R2 预签名上传 smoke，未执行链上确认 smoke。后续历史阶段曾用已废弃的单一 RPC Secret 验证，当前仍需按 Access 三项 Secret 重新验收。
- 已执行 `git diff --check`：通过；已清理 `citizenapp/cloudflare/node_modules` 和 `citizenapp/cloudflare/.wrangler`，未发现 `wrangler` / `workerd` 进程残留。
- 本阶段未修改 `citizenchain/runtime/`，未写入 Cloudflare token、R2 access key、R2 secret key 或链 RPC 私密地址，未触碰 GitHub 远端。

## 后续前置条件

- 若要重新执行真实远端上传发布 smoke，需要先在 Cloudflare staging Worker 配置链 RPC 三项 Access Secret 与 R2 预签名凭据；这些值只允许保存在 Cloudflare 远端 Secret，不得写入仓库。
- 若要部署 production，需要单独新建任务卡并确认 production R2/D1/KV 资源，不得复用 staging 数据。
