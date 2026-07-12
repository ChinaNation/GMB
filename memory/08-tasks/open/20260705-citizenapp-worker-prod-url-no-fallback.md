# CitizenApp 广场/聊天 Worker 地址内置生产默认（删除本机兜底）

## 任务需求

- 部署 Worker 到 Cloudflare（账户、密钥、在线节点均已具备），拿到固定生产 HTTPS 地址。
- 把该固定地址写死为 App 的**唯一默认**；真机 / debug / release 默认全部连 Cloudflare，零 `--dart-define`。
- **彻底删除** debug 下回落 `http://127.0.0.1:8787` 的兜底逻辑——连本机不是设计意图，只是 Worker 未部署时期的开发临时物。
- 本卡覆盖 `20260705-citizenapp-square-production-deploy.md` 步骤 3“保留本地调试 127.0.0.1:8787 兜底”的旧决策。

## 建议模块

- App 地址入口：`citizenapp/lib/8964/services/square_api_client.dart`（`SquareApiConfig`）
- 聊天瞬时转发接口（复用同一 baseUri，自动跟随）：`citizenapp/lib/chat/chat_runtime.dart`
- 部署：既有 `20260705-citizenapp-square-cloudflare-staging-deploy.md` / `20260705-citizenapp-square-production-deploy.md`

> 注：若目录扁平化卡（`20260705-cloudflare-worker-flatten-dir.md`）已执行，Worker 路径以 `citizenapp/cloudflare/` 为准。

## 影响范围

- `SquareApiConfig.defaultBaseUrl`：删掉 `localDevBaseUrl` 兜底，改为内置 `kProdBaseUrl` 常量作默认。
- `normalizeBaseUrl`：默认不再放行本机；仅在显式 dev 开关（`--dart-define`）传入时才允许本机地址，不做任何隐式回落。
- 聊天瞬时转发接口经 `_squareApiClient.baseUri` 复用同地址，无需单独改。
- 现在真机聊天/广场的红色 `ClientException`（连 `127.0.0.1:8787` 失败）随部署+内置默认一起消失。

## 主要风险点

- 生产地址硬编码进仓库：仅是公开的 `workers.dev`/自定义域，不含 Cloudflare token、R2 key、私密 RPC，可接受。
- 部署未完成前先改默认会导致 debug 也连不上：必须**先部署拿到稳定地址，再改默认**。
- staging 与 production 地址切换：用编译期 flag 区分，避免 debug 包误连生产写库。
- 生产/预发布必须默认 `SQUARE_DEV_UPLOAD_PROXY=0`。
- 不改 `citizenchain/runtime/`（本卡不涉及链端）。

## 是否需要先沟通

- 否。用户已明确“必须连 Cloudflare、删本机兜底、最简自动”。

## 预计修改目录

- `citizenapp/lib/8964/`：改 base URL 默认来源，去本机兜底；涉及代码。
- `citizenapp/cloudflare/`：部署产出固定地址（部署本身在既有卡）；涉及配置。
- `memory/08-tasks/open/20260705-citizenapp-square-production-deploy.md`：更新步骤 3 决策为“内置生产默认、无本机兜底”；涉及文档。
- `memory/01-architecture/citizenapp/`：记录地址策略；涉及文档。

## 分步骤技术方案

### 步骤 1：部署 Worker

- 按既有 staging/production 卡命令部署，拿到固定 HTTPS 地址。
- 替换 `wrangler.toml` staging/production 的 D1 `database_id`、KV `id` 占位值为真实资源 ID（不写密钥）。

### 步骤 2：内置生产默认

- `SquareApiConfig` 新增 `kProdBaseUrl`（生产 HTTPS）常量。
- `defaultBaseUrl`：`--dart-define` 优先 → 否则返回 `kProdBaseUrl`。
- **删除** debug 回落 `localDevBaseUrl` 分支；`_isProductBuild` 抛错分支不再需要（默认已是生产地址）。

### 步骤 3：dev 覆盖（仅显式）

- 只有显式 `--dart-define=CITIZENAPP_SQUARE_API_BASE_URL=...` 时才覆盖；本机地址仅在此显式路径下允许，不做隐式回落。

### 步骤 4：校验

- 真机 debug 包不传任何 define，直接从联系人详情/广场进入，红条消失、能拉到 Worker。
- `flutter analyze` / `flutter test` 覆盖 `8964` 与 `im` 相关目录。

### 步骤 5：文档

- 覆盖既有部署卡步骤 3 的旧“本机兜底”决策。
- 在 CitizenApp 技术文档记录“默认即生产 Cloudflare，无本机回落”。

## 当前执行状态

已按方案 B（部署独立 production）执行：

- [x] 用户在 Cloudflare 新建 production R2 桶 `citizenapp-square-media`。
- [x] wrangler 建 production D1 `citizenapp-square-db-production`（`0c5a0924-83ef-4347-bacc-b3f6f36da460`）与 KV `citizenapp-square-production-FEED_CACHE`（`b72bbbcb36d240acb317fdaf79ce46f4`），真实 ID 写入 `wrangler.toml` env.production。
- [x] 远端 D1 迁移 `0001/0002` 应用成功；`wrangler deploy --env production` 部署成功。
- [x] production 上线并 smoke：`https://citizenapp-square-api.stews87-fawn.workers.dev/health` 返回 ok；未登录 `/v1/square/membership` 返回 401。
- [x] App 内置 `SquareApiConfig.prodBaseUrl = https://citizenapp-square-api.stews87-fawn.workers.dev` 为唯一默认；删除 `localDevBaseUrl`/`_isProductBuild` 与本机兜底；`normalizeBaseUrl` 保留（本机联调仍可显式 `--dart-define`）。
- [x] `dart analyze` 无问题；`test/8964/square_feed_service_test.dart` 全过（含 normalizeBaseUrl 契约）。
- Chat 不使用 R2；图片/视频发布分别使用 Cloudflare Images/Stream，R2 只承担广场 manifest、资料和既定归档对象。
- [ ] 待部署 Access + Tunnel，并为 production 成套设置 `CHAIN_URL`、`CHAIN_ID`、`CHAIN_SECRET`（仅链读取与签名交易受控广播使用）。

文字聊天与广场基础发帖：已可用（production 已部署，App 默认直连 Cloudflare，本机兜底已删）。
