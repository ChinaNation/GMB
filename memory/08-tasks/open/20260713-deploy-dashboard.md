# 本地部署控制台

任务需求：
- 在仓库根目录建立 `deploy/`，集中承载全部部署网页、后端、脚本和密钥状态；源码由 Git 追踪，运行记录和私密材料精确忽略。
- 首页用图标展示 CitizenApp、CitizenWallet、CitizenChain、CitizenChain WASM、CitizenWeb 和 CitizenApp Cloudflare Worker。
- CitizenWeb 测试部署在本地构建并启动 `127.0.0.1:41732`，同时提供“关闭测试部署”；生产部署只更新现有 `citizenweb` Pages 项目。
- 测试操作无需密码；生产、Release 和服务器部署必须先完成本机 Touch ID 指纹验证。
- CitizenApp Cloudflare 模块提供“会员真实测试”：由官方 `cloudflared` 打开 Access 批准窗口，通过后才访问真实 staging API、远端 D1、sr25519 签名和 Stripe 测试环境。
- 部署密钥迁入 macOS Keychain，网页只显示状态，不显示、返回或记录密钥明文。
- 完成后删除根 `scripts/` 中旧六个部署入口和两份明文 Secret 文件，不保留旧流程。

所属模块：
- 仓库级部署与发布流程
- CitizenApp / CitizenWallet / CitizenChain / CitizenWeb

用户当前任务特批：
- `deploy/` 中不含密钥的部署脚本和本地网页进入 Git；`.runtime/`、日志、编译产物和私密文件不进入 Git。
- 用户于 2026-07-14 明确允许新增 `deploy/.runtime/cloudflared`；仅安装官方 Darwin ARM64 二进制，继续由根 `.gitignore` 排除，不进入 Git。
- 本任务不沿用“密钥相关入口必须留在根 scripts”的旧规则；根 scripts 只保留 AI 与仓库工具。

安全边界：
- 服务只监听 `127.0.0.1`，浏览器通过随机会话令牌访问。
- Keychain 密钥不返回前端；日志必须脱敏。
- 生产类操作每次执行都强制 Touch ID，不允许密码降级。
- 本轮开发验收不得真实触发 CI、Release、生产发布或服务器部署。

预计修改目录：
- `deploy/`：新增本地网页、后端、部署动作、Touch ID 与 Keychain 管理；源码由 Git 追踪，运行数据与私密材料被 Git 忽略。
- `scripts/`：删除旧部署入口和明文 Secret，清理旧流程残留；保留非部署仓库工具。
- `memory/`：更新任务卡、仓库映射、安全规则和部署入口文档；进入 Git。
- `.gitignore`：精确忽略根 `deploy/.runtime/` 与私密文件；进入 Git。

验收标准：
- 本地网页真实启动并通过浏览器检查首页、模块详情、密钥状态和按钮行为。
- 测试动作不要求认证；生产动作在命令执行前调用 Touch ID，认证失败不得执行。
- Keychain 中 staging/production 各 16 个 Worker Secret 完整且与迁移前值一致。
- 旧六个部署脚本和两份明文 Secret 文件全部删除。
- 日志、API 和前端均不泄露密钥。

当前进度：
- [x] 用户确认目录、Git 忽略、Keychain、Touch ID 和残留删除方案。
- [x] 实现本地部署控制台及全部密钥中文用途说明。
- [x] 迁移并验证 Keychain 密钥。
- [x] 删除旧部署入口和明文 Secret。
- [x] 完成 CitizenWeb 本地测试网站启动、真实页面访问和关闭验收。
- [x] 增加 CitizenApp Cloudflare“会员真实测试”按钮、Access 交互登录和逐项 PASS/BLOCKED/FAIL 日志；测试完成文案不再冒充部署成功。
- [x] 完成会员真实测试的 Access 人工批准及全矩阵运行态验收；业务断言 14 项通过、4 项仅在 Worker→Stripe 525 阻断，环境与清理 3 项通过，FAIL=0。
- [ ] 完成生产动作 Touch ID 成功路径验收；不得在验收中执行真实远端部署。

执行记录：
- 2026-07-13：staging/production 各 16 项 Secret 已写入 macOS Keychain并与迁移前值逐项比对一致，旧明文 Secret 文件随后删除。
- 2026-07-13：真实本地服务展示六个部署图标、20 项密钥中文用途、Keychain/GitHub Secrets 状态和脱敏日志；测试动作不要求密码。
- 2026-07-13：定位 CitizenWeb 测试和生产按钮未执行的原因是控制台当时由验收专用 `GMB_DEPLOY_DRY_RUN=1` 启动，不是官网源码、Wrangler 或 Pages 故障。
- 2026-07-13：CitizenWeb `npm ci`、lint、build、Wrangler 登录、现有 production Pages 项目和线上健康地址前置检查通过；未触发真实 production 部署。
- 2026-07-13：CitizenWeb 测试部署改为本地构建后启动 `http://127.0.0.1:41732`，已通过网页按钮真实启动、页面访问和关闭验收。
- 2026-07-13：部署控制台删除英文眉题，缩小主标题和刷新按钮，六张卡片改为左图标、右内容的紧凑横向布局并缩短高度；真实页面截图确认部署日志区域明显上移。
- 2026-07-13：本地构建、检查、密钥同步、远端触发、发布和健康检查阶段均增加简短中文步骤日志；网页真实执行“关闭测试部署”确认 `[开始]`、`[步骤]`、`[完成]` 日志正常显示。
- 2026-07-14：对 `deploy/` 源码执行明文密钥、私钥头、高熵长值、服务器IP和本机路径扫描，未发现密钥材料。目录改为Git追踪源码，`.runtime/`、日志、编译二进制、`.env`、`secrets/`和常见私钥文件继续忽略。
- 2026-07-13：顶部标题改为“部署控制台”并缩小字号、固定左对齐；部署模块、生产就绪、执行状态三个概览卡压缩后移入同一行，刷新按钮固定右侧；真实页面检查确认布局与文案生效。
- 2026-07-13：部署控制台增加macOS launchd socket按需唤醒：平时Node服务不常驻，浏览器访问 `127.0.0.1:41731` 时启动，空闲15分钟且没有部署任务时退出；手动启动仍保留自动打开浏览器。
- 2026-07-14：经用户明确授权，在被忽略的 `deploy/.runtime/` 安装官方 `cloudflared 2026.7.1` Darwin ARM64 二进制并校验发布 SHA256；控制台新增“会员真实测试”，首次网页点击已验证会打开可见的 Cloudflare Access `Approve` 窗口。
- 2026-07-14：用户在可见 Access 窗口批准后，从部署控制台真实执行 staging 全矩阵；修复 launchd Node PATH、Wrangler D1 登录态和 Stripe 测试夹具后，最终结果为 PASS=17、BLOCKED=4、FAIL=0。PASS 包含 14 个业务断言、真实 Stripe active 订阅夹具、Stripe 两项资源清理和 D1 零残留；B3/D2/E1/F1 均真实到达 Worker 的 Stripe 调用点并返回 525。
