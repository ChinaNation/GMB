# CitizenApp 广场 Cloudflare 生产化部署前准备

## 任务需求

在阶段 7 已完成本地真实 E2E 后，进入阶段 8：把广场从本地 Miniflare / D1 / R2 / KV 验收状态推进到生产化部署前准备状态。

本阶段不部署远端、不写入 Cloudflare 密钥、不触碰 GitHub 远端、不修改 `citizenchain/runtime/`。重点是固化生产/预发布配置边界、App 端环境切换方式、Worker 运维命令和文档口径，确保下一步真正部署时只需要在本机或 Cloudflare 控制台配置资源和密钥。

## 建议模块

- CitizenApp 广场 Worker：`citizenapp/cloudflare/square_worker/`
- CitizenApp 广场 App 端：`citizenapp/lib/8964/`
- 架构与任务文档：`memory/01-architecture/citizenapp/`、`memory/07-ai/`、`memory/08-tasks/`

## 影响范围

- Worker 配置：生产/预发布 D1、R2、KV 绑定命名与 dev proxy 边界。
- App 配置：广场 Worker base URL 的生产/开发切换方式，避免硬编码本地地址进入生产。
- 运维文档：迁移、部署、密钥、链 RPC 环境变量和本地验收命令。
- 残留清理：不得留下 `.dev.vars`、`.wrangler/`、`node_modules/`、`dist/`、`coverage/` 或临时 spec。

## 主要风险点

- 把 Cloudflare token、R2 access key、R2 secret key、生产链 RPC 私密地址写入仓库。
- 生产环境误启 `SQUARE_DEV_UPLOAD_PROXY=1`。
- App 生产包仍指向本地 Worker 地址。
- Worker 配置环境名和 D1/R2/KV 资源名不清晰，导致部署时误连测试数据。
- 未经二次确认修改 `citizenchain/runtime/`。

## 是否需要先沟通

- 否。用户已确认阶段 8，并已单独确认允许创建本任务卡。

## 预计修改目录

- `citizenapp/cloudflare/square_worker/`：整理 Worker 生产化配置和脚本边界；涉及配置/代码审查，不写密钥，不新增目录。
- `citizenapp/lib/8964/`：检查并必要时调整 App 端 Worker base URL 注入方式；涉及代码。
- `memory/01-architecture/citizenapp/`：补充生产部署前准备和运行边界；涉及文档。
- `memory/07-ai/`：如协议或统一口径变化，更新统一协议登记；涉及文档。
- `memory/08-tasks/`：记录本阶段方案、执行、验收和残留清理；涉及文档。

## 分步骤技术方案

### 步骤 1：现状审计

- 搜索 Worker 配置、package scripts、App API client base URL、环境变量读取逻辑。
- 确认是否存在硬编码本地地址、密钥占位、生产 dev proxy 风险。
- 不修改 `citizenchain/runtime/`。

### 步骤 2：Worker 生产化配置

- 在现有 `wrangler.toml` 中明确本地、预发布、生产的绑定边界。
- 生产/预发布必须默认 `SQUARE_DEV_UPLOAD_PROXY=0`。
- 只记录资源名和变量名，不写 Cloudflare token、R2 key 或私密 RPC。
- 补 package scripts 以固定本地迁移、预发布部署、生产部署命令。

### 步骤 3：App 端环境切换

- 检查 `SquareApiClient` 或同等入口的 base URL 来源。
- 如果存在硬编码本地地址，改为通过编译期 define 或集中配置读取。
- 保留默认安全行为：生产包必须显式提供 HTTPS Worker URL；本地调试才使用 `http://127.0.0.1:8787`。

### 步骤 4：文档与残留清理

- 更新 CitizenApp 技术文档，写清生产部署前置条件、资源命名、密钥边界和部署命令。
- 更新本任务卡执行记录。
- 清理本次产生的 `node_modules`、`.wrangler` 等本地残留。

### 步骤 5：验收

- Worker typecheck/test。
- Flutter analyze/test 覆盖广场相关目录。
- `git diff --check`。
- 检查无本地服务进程和生成目录残留。

## 当前执行状态

- [x] 阶段 8 任务卡创建
- [x] 步骤 1：现状审计
- [x] 步骤 2：Worker 生产化配置
- [x] 步骤 3：App 端环境切换
- [x] 步骤 4：文档与残留清理
- [x] 步骤 5：验收

## 执行记录

- 现状审计发现 `SquareApiClient` 默认 `CITIZENAPP_SQUARE_API_BASE_URL` 为 `http://127.0.0.1:8787`；该口径适合本地验收，但生产包如果忘记传 define 会误连本地地址。
- 已新增 `SquareApiConfig`，保留本地调试默认 `http://127.0.0.1:8787`，但生产构建未显式提供 `CITIZENAPP_SQUARE_API_BASE_URL` 时直接 fail-fast；显式配置只允许 HTTPS，或本地调试 `http://127.0.0.1` / `localhost` / `::1`。
- 已补充 `SquareApiConfig.normalizeBaseUrl` 测试，覆盖 HTTPS 尾斜杠归一化、本地 HTTP 允许和非本地 HTTP 拒绝。
- 已在 `package.json` 增加 `dev:local`、`migrate:staging`、`migrate:production`、`deploy:staging`、`deploy:production` 运维脚本；脚本不包含 token、R2 key 或链 RPC。
- 已在 `wrangler.toml` 增加 staging/production 绑定模板；`SQUARE_DEV_UPLOAD_PROXY` 在默认、staging、production 中均为 `0`。D1 `database_id` 和 KV `id` 当前是不可用占位值，远端部署前必须替换为 Cloudflare 实际资源 ID。
- 已更新 `memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`，记录 Worker 运维命令、App `--dart-define`、R2/链 RPC 密钥边界和远端资源 ID 占位规则。
- 已更新 `memory/07-ai/unified-protocols.md`，登记 `CITIZENAPP_SQUARE_API_BASE_URL`、R2 预签名变量和生产禁用 dev proxy 边界。
- 本阶段未修改 `citizenchain/runtime/`，未写入 Cloudflare token、R2 access key、R2 secret key 或链 RPC 私密地址，未触碰 GitHub 远端。

## 验收结果

- `dart format citizenapp/lib/8964/services/square_api_client.dart citizenapp/test/8964/square_feed_service_test.dart`：通过。
- `npm --prefix citizenapp/cloudflare/square_worker install`：通过，仅用于本地验收依赖恢复，验收后已清理 `node_modules`。
- `npm --prefix citizenapp/cloudflare/square_worker run typecheck`：通过。
- `npm --prefix citizenapp/cloudflare/square_worker test`：通过，5 个测试文件、10 个测试用例通过。
- `flutter analyze lib/8964 test/8964`：通过。
- `flutter test test/8964/square_chain_service_test.dart test/8964/square_publish_service_test.dart test/8964/square_feed_service_test.dart test/8964/square_home_page_test.dart`：通过，9 个测试通过。
- `git diff --check`：通过。
- 残留清理：已删除 `citizenapp/cloudflare/square_worker/node_modules`、`citizenapp/cloudflare/square_worker/.wrangler`；未发现 `wrangler`、`workerd` 或本地链服务进程残留。
