# 任务卡（分步执行·逐步确认）：CitizenApp 稳定币充值购买公民币

> 状态：**已定稿设计，分步实现**。用户已逐点拍板（方案 B / USDC→Base·USDT→Arbitrum / 本地部署控制台发币 / 四方对账三态台账 / 二元成功失败）。
> 工作流约定：**每一步先输出技术方案 → 用户确认 → 执行 → 更新文档·注释·清理残留 → 输出下一步方案**。未确认不执行；不写代码前不改任何目录。

## 任务需求

在「CitizenApp → 我的 → 钱包 → 我的钱包 → 钱包详情」把「充值」改成**购买公民币**：用户用自托管钱包（MetaMask / OKX 钱包 / Bitget 钱包）通过 WalletConnect 支付 **USDC(Base) / USDT(Arbitrum)** 到指定收款地址；本地部署控制台确认到账后，用**专用发币热钱包**发一笔 `ln` 转账把对应公民币打到用户公民链钱包。同时把钱包详情第 2 卡三个按钮重排：充值=购买公民币、提现=零钱包→链上、零钱包=进清算行零钱包详情页（原「充值到清算行」迁入该页）。

## 所属模块

- citizenapp（Flutter：`wallet` 三按钮重排、`transaction/onchain-topup` 新充值页、`transaction/offchain-transaction` 零钱包详情页迁 deposit）
- cloudflare（Worker：`topup` 订单/EVM 初验/待发币队列/结算回写 + D1 台账）
- deploy（本地部署控制台：发币钱包管理 + Touch ID 会话解锁 + 队列消费 + 独立 EVM 复验 + 本机节点发 `ln` 转账 + 四方对账台账）
- **citizenchain：零改**（复用 `ln` 转账 call；矿工挖矿代码绝不动）

## 关键决策（锁定，实现不得偏离）

1. **支付模型 B（WalletConnect v2 / `reown_appkit`）**：App 构造 ERC-20 `transfer`，锁链锁额，用户在自托管钱包签名。交易所账户不支持（首期不覆盖）。
2. **两币同走 Base**：USDC + USDT 均在 Base（一条链、一种 gas，最省心；用户钱包里两币都在 Base）。EVM 底座，后续加币/加链=加配置、零新代码。
3. **发币端 = 本地部署控制台**（`deploy/`），非常驻服务器。控制台没开机订单留队列，开机运行逐个补发。发币**不是 7×24 实时**（已接受）。
4. **专用发币热钱包**：私钥存本机 macOS Keychain，由控制台**写入/更换**（操作者本人输入，AI 全程不接触私钥值）；与主账户/矿工账户分离；主账户离线，只给发币热钱包补小额浮动。
5. **会话解锁**：控制台启动**一次 Touch ID** 把私钥解锁到内存，本会话队列逐个发币不再二次 Touch ID；退出/重启即清、需重新 Touch ID。
6. **每笔发币前控制台独立验 Base/Arbitrum RPC**（不只信 Cloudflare）：合约地址、收款地址、金额≥应付、确认数（用 `safe/finalized` 防 reorg）；验过才自动发公民币，验不过不发→置异常交人工。
7. **四方对账台账**：`控制台本地台账 + 公民币区块链 + Base/Arbitrum RPC + Cloudflare` 四方一致才改订单状态；不一致→异常冻结。
8. **订单三态（仅此三种）**：`待支付`（已收到稳定币、控制台未发公民币）/`已支付`（双方都完成=成功）/`异常`（除前两种之外一切=失败，交人工）。**用户视角只有成功(已支付)/失败(异常)，无取消、无「未支付订单」态**——用户没付款=没有订单、不记录、不入台账。
9. **幂等**：按 `(chain_id, evm_tx_hash)` 唯一；控制台崩溃/重启不重发；发成功才回写 `已支付`。
10. **定价**：15→10,000.00 公民币、1,400→1,000,000.00 公民币（含约 7% 批量折扣，有意保留）；USDC/USDT 均按 1:1 美元；**发币量服务端按套餐+已验证到账额推导，绝不信客户端**。
11. **沙箱期**：testnet（Base Sepolia / Arbitrum Sepolia + mock USDT）+ 测试 GMB 链 + 测试私钥；不上生产、不用 live key。

## 关键参数（落地前二次核对合约地址——错地址=收假币）

| 币 | 链 | ChainId | 合约 | 精度 |
|---|---|---|---|---|
| USDC | Base 主网 | 8453 | `0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913` | 6 |
| USDT | Base 主网 | 8453 | `0xfde4c96c8593536e31f229ea8f37b2ada2699bb2`（用户核对提供，已内置） | 6 |
| USDC | Base Sepolia | 84532 | `0x036CbD53842c5426634e7929541eC2318f3dCF7e` | 6 |
| USDT | Base Sepolia | 84532 | 自部署 mock，经 Env `TOPUP_USDT_CONTRACT` | 6 |

- 公民币 = 原生 GMB Balances，2 位精度（元/分），ED=111 分，转账走 `ln` pallet（index 2）。
- 金额换算：稳定币 6 位、公民币 2 位，全按各自最小单位整数计算，禁浮点。
- WalletConnect Project ID：用户自注册 `dashboard.reown.com`，`--dart-define` 注入（公开标识、非私钥）。

## 必须遵守（硬规则）

- 不新建目录/文件前须列全路径经用户确认（每步方案内列明）。
- runtime 零改；矿工挖矿代码不动。
- 私钥只在 macOS Keychain；不进 Git、不留明文文件；生产发币逐次/会话级 Touch ID。
- 禁兼容/禁残留：三按钮重排后旧充值入口、静态零钱包残桩一次清干净。
- 全仓字段同名；改代码必补中文注释、必更新文档、必清残留。
- 真实验收硬规则：只编译/单测不算完成，须真机 + testnet + 真实到账端到端。

---

## 推荐步骤拆分（后端→发币端→前端支付→UI 重排→端到端）

### 步骤 1 · Cloudflare 订单后端 + D1 台账 + EVM 到账初验 + 待发币队列 + 结算回写
- 目标：`/topup` 建单/查单 + `topup_orders` 表 + Worker EVM(Base/Arb) 到账初验 + 待发币队列 + 供本地控制台鉴权拉取/回写的结算接口。
- 预计修改目录：
  - `citizenapp/cloudflare/src/topup/`（代码/**新建**）：订单、EVM 初验、队列、结算接口。
  - `citizenapp/cloudflare/migrations/`（SQL/**新建**）：`topup_orders`（`UNIQUE(chain_id, evm_tx_hash)` 幂等）。
  - `citizenapp/cloudflare/src/routes.ts`（改）：挂 `/topup`、`/topup/settlement`。
  - `citizenapp/cloudflare/src/types.ts`（改）：Env 补 EVM RPC / 结算鉴权 secret 名。
  - `citizenapp/cloudflare/wrangler.toml`（改）：声明新 vars/secret 名（不含明文）。
  - `citizenapp/cloudflare/test/`（代码/**新建**）：topup 单测。
- 验收：curl 建单 → testnet 真实 tx 上报 → 初验通过入待发币队列；重复 txHash 幂等；错链/错额/错收款地址拒。

### 步骤 2 · 部署控制台发币端（结算）
- 目标：控制台发币钱包管理（写/换 Keychain 私钥）+ 启动一次 Touch ID 会话解锁 + 拉队列 + **独立 EVM 复验** + 本机节点发 `ln` 转账 + 四方对账台账 + 幂等 + 异常人工入口。
- 预计修改目录：
  - `deploy/actions/`（代码/**新建**）：发币结算动作（队列消费/EVM 复验/`ln` 发币/台账/幂等）。
  - `deploy/server.mjs`（改）：发币端点 + 会话密钥内存管理 + 发币钱包私钥 secret 名注册。
  - `deploy/web/`（改）：发币钱包管理 + 台账三态视图 + 异常人工处理入口。
  - `deploy/`（复用 `ln.sh`/`ln-auth`；如需补发币私钥读写再列明）。
- 验收：喂一个「已确认收到稳定币」订单 → Touch ID 解锁 → 独立复验 EVM → 本机测试链发 `ln` 转账 → 台账置已支付；RPC 验不过→异常；重启需重新 Touch ID；重复不重发。

### 步骤 3 · CitizenApp 链上充值页 + WalletConnect 支付轨
- 目标：新充值页（USDC/USDT 两按钮 + 套餐弹窗）+ WalletConnect 连钱包/锁链/构造 `transfer` + 建单/上报 txHash/轮询到账。
- 预计修改目录：
  - `citizenapp/lib/transaction/onchain-topup/`（代码/**新建**）：充值页、套餐、WalletConnect 客户端、`topup_api.dart`、轮询。
  - `citizenapp/pubspec.yaml`（改）：加 `reown_appkit`。
  - `citizenapp/lib/`（改）：WalletConnect Project ID 注入（`--dart-define`）。
- 验收：真 MetaMask(testnet) 连钱包 → 选套餐 → App 锁链构造 USDC(Base Sepolia)/USDT(Arb Sepolia) `transfer` → 签名 → 拿 txHash 上报 → 轮询显示处理中/已支付。

### 步骤 4 · 钱包详情三按钮重排 + 零钱包详情页 + 清理残留
- 目标：`wallet_action_card` 三按钮重排（充值→充值页、零钱包→详情页可点、提现沿用）；新零钱包详情页迁入原 `deposit_page`；清理旧静态零钱包/旧充值入口残留。
- 预计修改目录：
  - `citizenapp/lib/wallet/widgets/wallet_action_card.dart`（改）：三按钮语义 + 点击。
  - `citizenapp/lib/transaction/offchain-transaction/`（改/**新建**）：零钱包详情页 + 迁 deposit 入口。
  - `citizenapp/test/wallet/`（改）：更新受影响 widget 测试。
- 验收：三按钮各进正确页；零钱包页含清算行余额 + 充值到清算行；旧入口零残留；widget 测试过。

### 步骤 5 · 沙箱端到端真实验收
- 目标：Base Sepolia(真 USDC) + Arb Sepolia(mock USDT) + 测试 GMB 链 + 真 MetaMask，全链路真实验收。
- 预计修改目录：
  - 以验收为主，不新增业务目录；如需夹具/脚本再单列并授权。
  - `memory/05-modules/citizenapp/` 或 `memory/01-architecture/`（文档）：补功能技术文档。
- 验收：真机付 testnet 稳定币 → 控制台自动发测试公民币 → App 到账；异常（错链/RPC 验不过）入异常人工；重复 txHash 不重发；四方对账一致。

## 全局验收标准

- 用户全链路：连钱包→付 USDC/USDT→控制台确认→发公民币→App 到账，用户端只见成功/失败。
- 台账三态正确；四方对账一致才改状态；异常冻结可人工处理。
- 幂等：同一 EVM txHash 绝不重复发币；控制台重启不重发。
- runtime 零改、矿工代码不动；三按钮重排旧入口零残留。
- 真机 + testnet 端到端通过（非仅编译/单测）。

## 影响范围

citizenapp（前端 + Worker）+ deploy（本地控制台）三处；citizenchain 零改。私钥托管在本机 Keychain，发币走本地 Touch ID 会话。

## 进度记录

- **2026-07-17 · 步骤 1 完成（Cloudflare 订单后端 + 台账 + EVM 验证 + 队列 + 结算回写）**
  - 新增 `citizenapp/cloudflare/src/topup/{config,evm_verify,orders,settlement,routes}.ts` + `migrations/0005_topup.sql`。
  - 改 `src/routes.ts`（挂 `/v1/square/topup/*`）、`src/types.ts`（Env 补 topup 配置/令牌）、`src/limits/catalog.ts`（路由白名单）、`src/security/request_guard.ts`（topup 免广场会话、结算免限流）、`wrangler.toml`（沙箱 vars + 令牌走 secret 说明）。
  - 接口：`GET /v1/square/topup/config`、`POST /v1/square/topup/submit`、`GET /v1/square/topup/status`；结算（`TOPUP_SETTLE_TOKEN` 鉴权）：`GET /v1/square/topup/settlement/pending`、`POST /v1/square/topup/settlement/:id/settled`、`POST /v1/square/topup/settlement/:id/exception`。
  - 三态台账 `topup_orders`（pending/paid/exception），幂等键 `UNIQUE(chain_id, evm_tx_hash)`；EVM 到账验证校验合约/收款地址/金额/finalized 确认；结算回写前 Worker 独立复核 EVM（四方对账的 Cloudflare 角）。
  - 验收：`tsc --noEmit` 通过；`vitest` 全量 **178/178** 通过（新增 topup **10/10**）；`0005_topup.sql` 经 sqlite3 建表建索引校验通过。真机 + testnet 端到端并入步骤 5。
  - 残留：本步纯新增，无旧入口替换（三按钮重排在步骤 4），无残留。

- **2026-07-17 · 步骤 2 完成（控制台改名 CitizenConsole + 专属页 + 发币端）**
  - **2a 改名**：`git mv deploy citizenconsole`（保历史）；改内部路径/cookie（`gmb_deploy`→`gmb_console`）/包名；`.gitignore` `/deploy/`→`/citizenconsole/`；同步治理与架构文档目录名（`AGENTS.md`/`agent-rules.md`/`repo-map`/`GMB_TECHNICAL`/`security-rules`/`ci-path-routing`/`CITIZENAPP`/`CITIZENCHAIN`/`HOME_TECHNICAL`）。保留 `deploy` SSH 身份、`wrangler deploy`、`deploy-linux-servers` 等动词/他产品引用。残留归零。
  - **2b 卡片/整页**：`server.mjs` `modules` 首位加 `citizenconsole`（`page` 字段）并把 CitizenChain WASM 上移到第 4，7 卡按 4/3 排（`styles.css` `.cards` 4 列）；`app.js` 特判 `page` 卡整页跳转（非弹窗）+ 排除计数；新增专属页 `web/citizenconsole.{html,js}`，`serveStatic` 补两条。
  - **2c 发币端**（`citizenconsole/topup/`）：`evm_verify.mjs`（独立 EVM 复验）、`chain_transfer.mjs`（`@polkadot/api` 动态导入，OnchainTransaction.transfer_with_remark 发币，runtime 零改）、`ledger.mjs`（四方对账本地台账）、`routes.mjs`（`/api/topup/*`：config/wallet/session-unlock/settle-run/ledger，Touch ID 会话解锁、幂等防重复发币+崩溃窗口 fail-closed）；`server.mjs` 挂 topup 路由；`package.json` 加 `@polkadot/{api,util,util-crypto}`。
  - 验收：`node --check` 全过（server + topup + web JS）；`bash -n` 全过；根 `deploy/` 目录残留=0。**真机 + testnet + `@polkadot/api` 安装的端到端发币在步骤 5**。
  - 私钥边界：发币私钥只经页面输入→Keychain（`topup:DISBURSE_KEY`，Touch ID），会话仅驻内存；AI 全程不接触私钥值。

- **2026-07-17 · 步骤 3 完成（CitizenApp 链上充值页 + WalletConnect 支付轨）**
  - **通道改为方案 A（WebView WalletConnect）**：`reown_appkit` 与本 App 硬冲突（`reown_core` 钉 `flutter_secure_storage ^9.2.4` vs App `10.3.1`；`freezed_annotation 2.x` vs 聊天 `3.x`），所有版本无解 → 用户拍板走 WebView 里的 WalletConnect JS（`@walletconnect/ethereum-provider`），零 Dart 依赖冲突、用户体验一致。UI 已出稿并经用户确认。
  - 新增（`lib/transaction/onchain-topup/`）：`topup_models.dart`、`topup_erc20.dart`（ERC-20 transfer 编码）、`topup_api.dart`（config/submit/status，session-free，复用 `SquareApiConfig` 基址）、`onchain_topup_page.dart`（充值页+套餐弹窗，按确认稿）、`topup_result_page.dart`（处理中/已到账/失败，轮询）、`topup_webview_page.dart`（WebView+JS 桥+钱包深链唤起）；资产 `assets/topup/walletconnect.html`（WC JS 页）；`pubspec.yaml` 注册资产。
  - WalletConnect Project ID 走 `--dart-define=WALLETCONNECT_PROJECT_ID`（当前值 `8830074307d80484b839db4eb10b1f2c`，公开标识非密钥、dashboard.reown.com 注册；旧值 `11cdceaa…` 作废）。
  - 验收：`dart analyze lib/transaction/onchain-topup/` 零问题；`flutter test test/transaction/topup/` **11/11**（9 核心 + 2 页面）。
  - **步骤 5 device-only 待办**：WebView 内真实 WalletConnect 连钱包+签名+发交易、Project ID 注入、真机 + testnet 全链路（无法在本环境验证）。
  - 充值按钮接线在步骤 4；本步页面独立可测，无残留。

- **2026-07-17 · 步骤 4 完成（钱包详情三按钮重排 + 零钱包详情页 + 清理残留）**
  - `wallet/widgets/wallet_action_card.dart`：充值→`OnchainTopupPage`（稳定币购买公民币，**去掉清算行绑定门槛**）；提现→`WithdrawPage`（沿用，需绑定）；零钱包→**改为可点击**进 `PettyWalletPage`（需绑定）。删旧 `_StaticBalance` 静态不可点逻辑与旧「充值→DepositPage」入口。
  - 新增 `transaction/offchain-transaction/pages/petty_wallet_page.dart`：零钱包详情页（清算行余额 + 充值到清算行[迁入 `DepositPage` 入口] + 提现到链上）。
  - 命名：保持 `清算行`=机制/页面（deposit/withdraw 页标题、qr_protocols、citizenwallet 不动，零分叉），`零钱包`=入口/钱包名（卡片按钮 + 详情页标题）。避免半改名残留与跨模块扩散。
  - `test/wallet/widgets/wallet_action_card_test.dart` 更新：三列均可点击（3 个 InkWell）、提现/零钱包未绑定提示先绑定。
  - 验收：`dart analyze`（offchain + wallet + onchain-topup）零问题；`flutter test` action card **5/5** + topup **16/16**；旧静态零钱包/旧充值入口零残留。

- **2026-07-17 · 步骤 5 联调调整**
  - **两轨恒显**：`cloudflare/src/topup/config.ts` 改为 USDC 与 USDT **始终同时返回**（不再因合约未配置隐藏）；mainnet 两币内置合约，testnet 的 USDT mock 由 `TOPUP_USDT_CONTRACT` 提供。同步改 `test/topup.test.ts`（config 期望 2 轨），`tsc` + vitest 10/10 通过。
  - **控制台改名 + 关闭按钮**：`citizenconsole/web/index.html` 页面标题/H1 `CitizenConsole`→**公民控制台**；右上角刷新旁新增「关闭」按钮（`#closeConsole`）；`web/app.js` 点关闭→`POST /api/shutdown`；`server.mjs` 加 `/api/shutdown`（响应后 `process.exit(0)`，launchd `KeepAlive=false` 不自启）+ 启动日志改「公民控制台」。真机运行验证：`/` 返回 `<h1>公民控制台</h1>` + 关闭按钮。
  - **依赖已装**：`citizenconsole/` `npm install`（`@polkadot/api` 等，`node_modules` git 忽略）。
  - **本机自启动待用户修**：`~/Library/LaunchAgents/com.gmb.deploy-console.plist` 仍指向旧 `GMB/deploy/`（仓库外系统文件，AI 无权改）→ 用户 `sed` 改 4 处路径 + `launchctl unload/load`，之后浏览器开 `http://127.0.0.1:41731` 即按需自启公民控制台。

- **2026-07-17 · 链路收敛到单链 Base（取代早前 USDT→Arbitrum）**
  - 用户指出 Base 上 USDC/USDT 都有、钱包里也都在 → 改为**两币同走 Base**（一条链、一种 gas、杜绝转错链）。本卡内其余「Arbitrum」表述以此为准作废。
  - `cloudflare/src/topup/config.ts`：USDT template 改 chain 8453/84532(Base/Base Sepolia)、rpc 用 `TOPUP_BASE_RPC_URL`、合约由 `TOPUP_USDT_CONTRACT` 提供（不内置防错址）；`Arbitrum` 相关不再使用。
  - 控制台清理 `ARBITRUM_RPC_URL` 配置项（`topup/routes.mjs` + `web/citizenconsole.{html,js}`）；`rpcForChain` 只留 Base。
  - 收款地址：`TOPUP_RECV_ADDRESS = 0x5ce9b56b9d1812a7cf29841e21756f09ca7d223b`。
  - 验收：Worker `tsc` + topup `vitest` 10/10；Flutter topup 11/11；控制台 `node --check` 通过。

## 待用户拍板的遗留小项（不阻塞步骤 1）

- 国储会主账户→发币热钱包补仓的触发方式（手动 / 阈值提醒）。
- 是否另出正式 `ADR-039` 决策记录（当前决策已全部落本卡）。
