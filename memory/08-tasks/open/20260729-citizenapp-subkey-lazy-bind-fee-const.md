# 任务卡：设备子钥实际缺钥初始化 + 链上费率常量下发 + 注册前余额闸

> 状态：**执行中**。跨 citizenchain runtime + CitizenApp + Cloudflare Worker 三端。
> 前置：`20260717-citizenapp-stablecoin-topup.md` 的 2026-07-29 节（充值鉴权与 CID 解耦）已完成，充值可充到任意钱包账户。

## 任务需求

本窗口原始需求是「身份页点『确认注册』时先检查钱包余额，不足则跳转该账户的链上充值页」。诊断过程中发现两个更上游的阻断，必须一并解决：

1. **新用户建不出钱包**：`WalletManager.createWallet/importWallet` 对 P-256 设备子钥注册 fail-closed，而 Worker `device/register` 自 2026-07-28 commit `9aeab195` 起要求账户已绑 CID——建钱包那一刻必然没有 CID。
2. **客户端无法拿到链上最低交易费**：`ONCHAIN_MIN_FEE` 只活在 `primitives::fee_policy`，未暴露到 metadata；客户端要预检余额就只能复刻常量，违反「交易费常量全仓只有区块链常量库一处」。

## 用户定稿（不得偏离）

- **页面进入与钱包账户私钥彻底解耦**。广场、Chat、创作者、通讯录、会员/订阅页面进入
  时只判断 CID，registered 直接放行，不增加任何设备子钥页面、按钮或门禁状态。已有 P-256 子钥
  与设备用途钥直接静默使用；只有真实登录被 Worker 明确拒绝为
  `device_not_registered`，或真实数据解密发现用途钥缺失/失效时，才鉴权一次生成全部
  所需子钥并继续原业务。相同 `account_id` 不得因 revision 变化重复读取 child。
- **交易费常量单源属于区块链常量库**。全仓库只有 `primitives::fee_policy` 一处，禁止在 App、Worker 或任何其它地方另立交易费常量。客户端只能从链上读。
- 链端一次把链上收费三项都下发，「今后其他链上交易也用得到」。
- 已授权本次修改 `citizenchain/runtime/`（runtime 二次确认硬规则已满足）。

## 方案

### 1. 链端：费率常量上 metadata

`runtime/transaction/onchain` pallet 的 Config 增加三个 `#[pallet::constant]`，**值直接转发 `primitives::fee_policy`，pallet 不新定义任何数字**：

| 常量 | 转发自 | 用途 |
|---|---|---|
| `OnchainMinFee` | `ONCHAIN_MIN_FEE` | 最低链上交易费 |
| `OnchainFeeRate` | `ONCHAIN_FEE_RATE` | 费率，客户端算 `max(amount×rate, min)` |
| `VoteFlatFee` | `VOTE_FLAT_FEE` | 投票统一费 |

`ExistentialDeposit` 不加——已是 `pallet_balances` 的 `#[pallet::constant]`，metadata 里现成。

边界：只加 Config 关联常量 + `configs.rs` 的绑定；**不动 storage、不动 call 索引、不动 genesis、不动收费逻辑**。

### 2. 客户端：读链上常量，Dart 侧零费率常量

`ChainRpc` 加 metadata 常量读取入口（`metadata.chainInfo.constants[pallet][name]`，`Constant.type` 解 `Constant.bytes`；metadata 已按 `_cachedMetadata` 缓存，零新网络往返、零新依赖）。

顺带清掉既有违规副本 `square_publish_service.dart` 的 `publishFeeFen = 10` / `accountExistentialDepositFen = 111`，改为读链。

### 3. 设备子钥与设备数据钥按实际缺钥初始化

- 删 `WalletManager._registerDeviceSubkey` 及 `createWallet`/`importWallet` 两处调用；建钱包回到纯本地，不依赖 Worker。
- `IdentityRegistrationGate` 只判断 CID；registered 直接渲染功能页，不检查设备子钥。
- 广场/Chat 的真实登录先用现有 P-256 子钥静默签名；只有 Worker 返回
  `device_not_registered` 才调用 `registerDeviceSubkeyForBinding`，鉴权一次后登记并重试。
- Chat/MLS/附件/通讯录先静默解封现有用途钥；只有真实数据访问发现缺钥或封装失效时，
  才调用独立的 `ensureDeviceDataKeysForBinding` 鉴权一次生成并重试。
- 两条入口各自拥有静态 single-flight 和失败回滚，精确键均为
  `(genesis_hash, cid_number, account_id)`；P-256 登记绝不生成或删除本地数据钥，本地数据钥
  生成绝不调用设备登记、Turnstile 或 Worker；已有对应材料时读取 child 次数为 0。
- CID 注册/换绑 finalized 只激活公开绑定并完成已授权的数据交接，不预生成数据钥，也不
  登记 P-256 子钥。
- CID 换绑目标必须是不同 `account_id`；相同账户在读取私钥、构造交易或数据交接前拒绝。
- 后台推送预热只补已有 device_id/public_key 的登记，不创建 MLS、不解密数据、不读 child。
- Worker `device/register` 读到「无 CID」时旁路 45 秒身份缓存强制回源链读一次（用户可能刚注册完 CID 就进广场，缓存里还是注册前的旧值）。

### 4. 注册前余额闸

`myid_page.dart::_onRegister` 拿到 `RegisterChoice` 后、提交前：门槛 = 链上 `OnchainMinFee + ExistentialDeposit`（当前 121 分，代码不写死）→ `fetchFinalizedBalance(bindAccountId, forceFresh: true)` → 读失败提示重试（fail-closed，不跳转不提交）／低于门槛跳 `OnchainTopupPage(accountId: bindAccountId)`／达标走原提交。

### 5. 充值页币轨卡片换真实币标

`_RailCard` 按 `rail.token` 渲染 USDC/USDT 真实 SVG（自带官方圆底色），替换现在两卡相同的 `Icons.paid_outlined`。

## 预计修改目录

- `citizenchain/runtime/transaction/onchain/`（代码，runtime 已授权）：pallet Config 加三个费率常量，仅转发 `fee_policy`。
- `citizenchain/runtime/src/`（代码，runtime 已授权）：`configs.rs` 的 `impl onchain::Config` 绑定三常量。
- `citizenapp/lib/rpc/`（代码）：`chain_rpc.dart` 加 metadata 常量读取入口。
- `citizenapp/lib/wallet/core/`（代码+残留清理）：`wallet_manager.dart` 删建钱包时子钥注册、方法改名。
- `citizenapp/lib/wallet/pages/`（代码+残留清理）：`create_wallet_onboarding_page.dart` fail-closed 注释与文案同步。
- `citizenapp/lib/my/myid/`（代码+残留清理）：`identity_registration_gate.dart` 只保留 CID 判定并直接放行；`myid_page.dart` 余额闸与充值跳转。
- `citizenapp/lib/8964/services/`（代码+残留清理）：`square_publish_service.dart` 删硬编码费率常量改读链。
- `citizenapp/lib/transaction/onchain-topup/`（代码）：`_RailCard` 换真实币标。
- `citizenapp/assets/icons/`（资源）：新增 `usdc.svg`、`usdt.svg`。
- `citizenapp/pubspec.yaml`（配置）：注册两枚 SVG。
- `citizenapp/cloudflare/src/{auth,chain}/`（代码）：子钥注册端点「无 CID」时强制回源链读。
- `citizenchain/.../tests/`、`citizenapp/cloudflare/test/`、`citizenapp/test/`（测试）：常量暴露、缓存旁路、页面零子钥初始化、真实缺钥一次初始化、余额闸三分支。
- `memory/07-ai/`、`memory/08-tasks/`、`memory/01-architecture/`（文档）：更新死契约文字、本卡、架构文档。

## 死契约变更（已获用户确认）

`square-session-never-lazy-register` 固定为：后台推送预热绝不注册设备、绝不读取账户 child；
前台真实 Session 只有在 Worker 明确返回 `device_not_registered` 时才鉴权一次生成子钥。

## 执行记录

- **2026-08-02 · 全仓协议版本标识门禁收口**
  - 清理 Worker 反向路由测试和现有技术文档中的废弃版本路由字面量；请求守卫仍只剥离
    `/api`，并用中性未知前缀验证路由白名单拒绝行为。
  - Cloudflare Durable Objects 的既有 migrations `tag` 是官方生命周期字段，不是 GMB
    业务协议标识；生产 `wrangler.toml` 保持不变，AI 门禁只对该文件中的精确官方配置豁免。
  - `BASE_REF=origin/main ./.github/scripts/check-ai-guardrails.sh` 通过；Worker 上传定向测试
    8/8 通过。未修改 runtime、Worker 生产配置或其它业务代码。

- **2026-08-02 · 无版本生产 Worker 发布完成（第 2 步）**
  - **根因与边界**：App 唯一生产根已经是 `https://www.crcfrcn.com/api`，业务请求使用
    `/square`、`/chat`、`/chain`、`/security` 无版本路径，但线上 Worker 仍停留在旧
    版本路由，所以广场显示加载失败，创作者与会员显示“接口不存在”，聊天和
    通讯录的登录失败又被上层误呈现成设备安全验证。本步骤只发布当前 Worker 契约，未修改
    第 1 步的数据钥、P-256 登记、页面门禁或钱包私钥读取逻辑。
  - **路径收敛**：Worker 入口只剥离唯一部署前缀 `/api`；业务路由、资源白名单、App
    请求和 bootstrap schema 全部使用无版本名称，不保留废弃版本兼容。bootstrap 返回的
    `square_base_url`、`chat_base_url`、`media_base_url` 会保留请求所在的 `/api` 部署前缀，
    避免下发不存在的同域根路径。
  - **Durable Object / 数据边界**：`ChatRealtimeObject` 没有新增、重命名或删除，部署沿用
    Cloudflare 已应用的既有迁移标记，不伪造第二次 migration。CitizenConsole 只执行
    production D1 连通性与表完整性只读检查、Secret 同步和 Worker 发布；没有重建或清空
    D1、KV、R2、Queue。
  - **自动化验证**：Worker 全量 **33 files / 258 tests** 通过；`npm run typecheck`、
    `npm run types:check`、Wrangler dry-run 全部通过。CitizenApp 的 bootstrap、广场、Chat、
    创作者、会员、通讯录、CID 门禁、设备子钥定向测试 **64/64** 通过；`flutter analyze`
    零问题。跨端测试现同时锁定 App 的 `/api` 生产根、无版本业务路径、Worker 路由白名单，
    并断言废弃版本路径必须 404。
  - **生产发布**：通过 CitizenConsole 两次 Touch ID 于 2026-08-02 发布成功，Cloudflare
    Worker Version ID 为 `78f87b74-4f39-4fb5-a7e0-c9f7d9b4e81d`。真实线上验收确认
    `/api/health`、`/api/chain/bootstrap`、`/api/security/config` 均为 200；广场、创作者、
    会员、通讯录、Chat 的无会话请求均命中真实接口并返回 401，不再返回 404；废弃版本
    路径明确返回 404。bootstrap schema 为 `citizenapp.chain.bootstrap`，三条
    service URL 均正确保留 `https://www.crcfrcn.com/api`。
  - **真机边界**：已连接 OnePlus 6T / Android 11 上现有 CitizenApp 可正常启动；当前选中
    钱包没有注册 CID，页面按产品规则停在“需要注册身份”，因此本轮不伪造“已注册 CID
    逐页点击”结果，也不擅自注册或切换用户钱包。已注册页面的路径命中由真实生产 HTTP
    与上述跨端、页面定向测试共同覆盖。

- **2026-08-02 · 本地数据钥与 P-256 设备登记彻底拆分完成（第 1 步）**
  - `ensureDeviceDataKeysForBinding` 只补生成缺少的本地用途钥；不调用设备登记、Turnstile
    或 Worker。`registerDeviceSubkeyForBinding` 只在 Worker 明确返回
    `device_not_registered` 后登记 P-256 子钥；不生成、覆盖或删除本地数据钥。
  - 两类入口分别使用独立静态 single-flight、独立成功标记和独立失败回滚；相同
    `(genesis_hash, cid_number, account_id)` 的同类并发只读取一次 child。P-256 登记失败
    保留全部数据钥密文；本地数据钥生成失败保留 P-256 登记标记。
  - CID 注册/自主换绑/注册局换绑 finalized 已移除设备密钥初始化，只激活公开绑定并完成
    已授权的数据交接。广场与 Chat 只在 Worker 未登记响应后进入 P-256 登记。
  - 测试新增“本地数据钥不受登记后端失败影响、两类独立并发去重、只补缺项、双方失败
    互不回滚、相同账户伪换绑零 child 读取、finalized 零 P-256 登记”等断言。定向测试
    **49/49**；`flutter analyze` 零问题；全量 `flutter test --concurrency=1` 为
    **1098 passed / 5 skipped / 0 failed**，5 项仍是纯 Dart 宿主无原生 `libsmoldot` 守卫。
  - `flutter build apk --debug`、`:app:compileDebugAndroidTestKotlin` 均成功；连接的 OnePlus
    6T / Android 11 上 `connectedDebugAndroidTest` **2/2** 通过。debug APK 真机冷启动 12 秒
    无 `HW_SEED_VAULT`、`BiometricPrompt` 或 `FATAL EXCEPTION` 日志。此前 Pixel 8a 本轮未连接。
  - 本步骤没有修改或部署 Worker，没有新增任务卡、页面、按钮、授权状态或兼容分支；生产
    Worker 部署仍严格留给第 2 步单独确认。

- **2026-08-02 · 页面授权错误流程订正完成**
  - **页面边界**：`IdentityRegistrationGate` 已删除设备授权状态、检查、按钮和重试入口；
    广场、Chat、创作者、通讯录、会员/订阅只判断 CID，registered 直接渲染真功能。
  - **密钥边界（第二次订正）**：已有 P-256 设备子钥和设备用途钥均静默使用，读取账户
    child 次数为 0。Worker 真实返回 `device_not_registered` 时只进入
    `registerDeviceSubkeyForBinding`；数据访问真实发现用途钥缺失/失效时只进入
    `ensureDeviceDataKeysForBinding`。两条流程不再共用生成、登记或失败回滚。
  - **并发与换绑**：两类进程级 single-flight 分开保存，唯一键均为
    `(genesis_hash, cid_number, account_id)`；同一账户的同类并发只读一次 child。
    相同 `account_id` 的 revision 变化不是换绑，已在读取 child、签名、构造交易和数据交接前拒绝。
    CID finalized 不调用任一缺钥入口。
  - **测试**：`flutter analyze` 零问题；广场、Chat、通讯录、会员、创作者、CID 门禁、
    会话缺钥回调和钱包密钥状态机定向测试 **104/104**；
    `flutter test --concurrency=1` 全量 **1093 passed / 5 skipped / 0 failed**。5 项 skip 是纯 Dart 宿主无原生 `libsmoldot` 的既有守卫。
  - **Android 真实验收**：`flutter build apk --debug` 成功；
    `:app:compileDebugAndroidTestKotlin` 成功；Pixel 8a / Android 16 上
    `connectedDebugAndroidTest` **2/2** 通过，覆盖 P-256 删钥重建、设备数据钥 seal/open、
    AAD 不匹配拒绝和删钥后旧密文失效。锁屏状态的首次执行按设计返回 `DEVICE_LOCKED`；
    解锁后原实现直接全绿，未删除 `setUnlockedDeviceRequired(true)` 安全约束。
  - **运行态边界**：debug APK 已真机安装启动，首屏 12 秒无 `HW_SEED_VAULT` 读取和生物识别。
    该 debug 安装无既有钱包/CID 数据，因此不冒充已完成“真 CID 账户逐页点击”验收；
    已注册页面的直接放行由定向页面测试和全量测试覆盖。
  - **iOS 验收**：新 Swift 通道与 `AppDelegate.swift` 通过 `swiftc -frontend -parse`，
    Xcode 工程通过 `plutil -lint`；本机 `xcode-select` 只指向 CommandLineTools，无法执行完整 iOS 构建或真机验收。
  - **文档与残留**：安全规则、CitizenApp 架构、Wallet、Chat、User 技术文档和既有相关任务记录已同步；
    被否决的页面授权文案、页面授权接口和完成标记残留已清零。
    本次订正未新建任务卡。

- **2026-08-02 · 页面生物识别修复第一次实现（错误，已彻底撤销）**
  - **根因**：五个需 CID 页面共用的 `IdentityRegistrationGate` 在 `registered` 分支自动执行
    设备登记；多个 Gate/MyIdService 实例会在页面进入、广播重判和链健康恢复时并发或反复
    触发，而旧设备登记/聊天与通讯录用途钥准备会读取账户 child，因此只有已注册 CID 的
    用户反复弹出生物识别，未注册用户停在注册门禁而不触发。
  - **错误点**：这一版错误增加了页面级设备状态和按钮，导致既有 CID 用户因缺少新增
    ready 标记被阻断。该产品流程不属于用户需求，现已从代码、测试和技术文档删除。
  - **设备数据钥金库**：新增 Android Keystore AES-GCM 与 iOS Secure Enclave ECIES 独立
    命名空间，封装 Chat/MLS/附件/通讯录用途钥；已有钥静默读取，真实缺钥才鉴权一次生成。
    删除钱包会同步清理设备数据钥和密文索引。
  - **全局并发去重**：`WalletManager` 用静态 single-flight，精确键为
    `(genesis_hash, cid_number, account_id)`；相同钱包账户并发只执行一次缺钥初始化。
  - **换绑约束**：相同 `account_id` 不是换绑，已在身份上下文、签名、交易构造和数据交接
    之前直接拒绝；只有不同 `AccountId` 才可进入正式换绑。
  - **后台边界**：推送预热只重登已有 device_id/public_key，不创建 MLS、不解密数据、
    不读取 child；绑定 finalized 收敛只处理正式注册/有效换绑业务。
  - **测试**：新增/完善门禁、MyId、WalletManager、设备金库与删除清理覆盖；断言页面进入
    页面进入不调用子钥初始化、已有子钥读取 child 为 0、真实缺钥只读一次、跨实例并发
    只执行一次、相同 AccountId 换绑在签名前拒绝。
  - **原生验收**：Android `flutter build apk --debug` 成功；现有 instrumented test 补入
    设备数据钥 seal/open、AAD 不匹配拒绝及删钥后旧密文失效，
    `:app:compileDebugAndroidTestKotlin` 编译成功。iOS 新 Swift 文件通过
    `swiftc -frontend -parse`，Xcode 工程通过 `plutil -lint`。本机仅安装 CommandLineTools、
    没有完整 Xcode，且项目缺 Flutter `.metadata`，因此本轮无法执行 iOS 真机构建和两端
    真机生物识别交互验收；该限制不伪装成已完成真机验收。
  - **文档与残留**：安全规则、CitizenApp 架构、钱包、Chat、用户技术文档已同步；旧
    页面级设备状态、按钮、旧初始化入口与误导注释已清零。未新建任务卡。

- **2026-07-29 · 全部落地**
  - **链端**（`runtime/transaction/onchain`）：pallet Config 加 `OnchainMinFee` / `OnchainFeeRate` / `VoteFlatFee` 三个 `#[pallet::constant]`；`configs.rs` 用 `ConstU128<{ primitives::fee_policy::… }>` 与 `parameter_types! RuntimeOnchainFeeRate` 绑定，**只转发不另立数字**。pallet 测试 mock 同步绑定，并新增守卫用例 `exposed_fee_constants_forward_fee_policy_exactly`（逐项 assert 等于 `fee_policy` 真源，有人在绑定处改写数字立刻红）。`cargo test -p onchain` **22/22**，`cargo check -p citizenchain` 整体通过。未动 storage / call 索引 / genesis / 收费逻辑。
  - **客户端读常量**：`chain_rpc.dart` 加 `fetchPalletConstant` / `fetchPalletConstantU128` / `fetchMinSelfPayBalanceFen`（`= OnchainMinFee + Balances.ExistentialDeposit`）。走已有 `metadata.chainInfo.constants`，零新依赖、零新网络往返（metadata 已缓存）。常量缺失抛 `StateError` 由调用方 fail-closed，不兜默认值。
  - **删掉 Dart 侧费率副本**：`square_publish_service.dart` 的 `publishFeeFen = 10` / `accountExistentialDepositFen = 111` / `minimumPublishBalanceFen` 三个常量**全删**，改为 `SquarePublishBalanceReader.fetchMinSelfPayBalanceFen()` 读链。全仓 grep 三个符号已零命中。
  - **当时的子钥懒绑定（已由 2026-08-02 新边界替代）**：建钱包/导入钱包不注册设备；
    当时曾把设备登记放在需 CID 页面门禁的自动放行路径。该设计会让多个常驻 Gate 在页面
    进入时触发账户 child 读取，现已彻底删除，禁止恢复。
  - **修掉一个被时机变更打破的旧假设**：完成标记不能在钱包创建时写基线，否则会
    谎称设备已绑定。最终实现按 finalized
    `(cid_number, binding_revision, account_id)` 激活当前钱包派生上下文，只由真实完成路径推进。
  - **Worker**：`chain/identity.ts` 新增 `fetchChainIdentityStateFreshIfUnbound`（缓存无 CID 才回源，有 CID 直接采信不多打链）；`auth/service.ts` 的 `registerDeviceSubkey` 改用它。
  - **币轨图标**：新增 `assets/icons/usdc.svg`（`#2775CA`）、`usdt.svg`（`#26A17B`），手写 SVG、无外部依赖；`_RailCard` 按 `rail.token` 渲染，未登记币种回退 USDC 底图不崩页；`pubspec.yaml` 已注册。
  - **当时的测试**：Worker `typecheck` 干净 + `vitest run` **205/205**；Flutter `dart analyze lib test` **零问题**。当时为页面自动登记新增的用例已随错误设计于 2026-08-02 删除，由“页面直接放行 + 真实缺钥一次初始化”回归用例取代。
  - **⚠️ 更正一处误判（2026-07-29 复盘）**：本卡上一版曾写「全量约 40 项失败是既有干扰、与本任务无关」——**这是错的**，当时只单跑了两个不相干文件就下结论。复查发现那 40 项**绝大多数正是当时 Gate 自动登记设备改动引起的连锁**：被 Gate 包裹的页面测试只注入身份解析桩，自动登记路径却创建真实链客户端，导致 `pumpAndSettle timed out`。该自动登记设计已在 2026-08-02 删除。
  - **当时的测试修复**：统一 Gate 测试 helper 当时曾注入设备登记替身；
    2026-08-02 已将设备状态与替身全部删除，测试直接断言 registered 页面只渲染真功能且没有额外按钮。
  - **全量绿**：修复后 `flutter test --concurrency=1` 全量 **1017 passed / 5 skipped / 0 failed（`All tests passed!`）**。4 个受影响文件单跑复核：`square_home_page` 4/4、`chat_tab` 20/20、`membership_page` 21/21、`contact_book` 9/9。5 个 skipped 是无 native libsmoldot 时的既有 skip 守卫（`smoldot_native_probe`），非失败。
  - **真实验收的边界（如实记录）**：链端只做到 `cargo test` + 整体 `cargo check`；常量经 metadata 下发到客户端、余额闸与懒绑定的端到端真机验证**需要正式链可出块**，而正式链当前仍被 `20260717-citizenapp-stablecoin-topup.md` 记录的创世/节点守卫不一致阻塞，故本轮**未做真机端到端**。Worker 侧新分支由单测覆盖（`device/register` 的活体验证同样依赖链）。
  - **文档**：`memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md` §4.5 补「设备子钥懒绑定」与「注册前余额闸 + 交易费常量单源」两条；死契约 `square-session-never-lazy-register` 已更新子钥注册时机并说明为何不放松 ANR 立契理由。

- **2026-07-29 · 收尾复检(遗漏/残留/安全/改进)**
  - **补清残桩注释(违反无残桩,上一版遗漏)**：子钥改懒绑定后,全仓仍有 9 处旧注释把「子钥注册」说成钱包创建/导入时发生——现已全部改正：`create_wallet_onboarding_page.dart`(类头 + 失败文案 + `_openImport`)、`import_wallet_page.dart`(类头 + 失败文案)、`wallet_manager.dart`(`WalletSubkeyRegistrar` typedef 文档 + `_subkeyRegistrar` 字段文档 + `_Account0` 派生用途)、`main.dart`(钩子注入注释)、`chat_runtime.dart`(后台握手注释)、`square_session_provider.dart`(类头)。均为纯注释,零逻辑改动。上一版任务卡只 claim 了 onboarding 一处且实际没做,本轮全部落实。
  - **核实无问题的点**：① pallet 名 `Balances`/`OnchainTransaction` 与
    `construct_runtime` 一致；②结算接口仍走 `TOPUP_SETTLE_TOKEN` 鉴权；③后台
    `ensureSession` 不懒注册设备；④ finalized 完成标记只在当前绑定激活与设备登记完成
    后写入，换机导入不会被虚假基线短路；⑤ topup Dart/Worker 无残留 session 语义。
  - **仍存的真实缺口/风险(如实记录,未处理)**：
    1. **`fetchPalletConstant` 真实解码路径无单测**——单测用 fake 把 `fetchMinSelfPayBalanceFen` 整个跳过,`constant.type.decode(ByteInput(bytes))` 的 u128→BigInt 实解只在真机/真链能证。编译通过、用的是 polkadart 标准 API,但属未验路径。建议:补一个用真 metadata fixture 的解码测试,或真机确认。
    2. **充值 confirm 的 DoS 面变宽**——去 session 后任何人(不止已注册用户)都能触发 confirm→打外部 EVM RPC。现由 guard IP 限流 60/60s + handler 账户级 10/60s 兜底,但比原来(会话门)宽。建议:若真出现滥用,给命中 EVM RPC 的路径再加一层全局/IP 硬顶。
    3. **抢单防护为 best-effort**——共享收款地址下,攻击者可在受害者付款进 mempool、未上链的窗口内预建意图抢先 confirm。彻底解=每单 HD 派生独立收款地址,超本轮范围,已记于 `20260717` 卡。
    4. **币标未目视验证**、**链相关 E2E 未做**(建钱包不被 cid_not_bound 挡 / 余额闸三分支 / 进广场绑子钥 / metadata 真读常量)——都要正式链能出块,当前被创世/节点守卫不一致阻塞。
  - **本轮受分类器不可用影响**:9 处注释改动未跑 `dart analyze` 复核(纯注释、无逻辑,安全);未重跑受影响测试(注释改动不影响测试)。

- **2026-07-29 · 补两处遗留缺口(解码路径单测 + confirm DoS 面收窄)**
  - **`fetchPalletConstant` 真实解码路径补单测**：新建 `citizenapp/test/rpc/chain_rpc_test.dart`(用户已确认新建)。用一次性 Rust 生成器(`scratchpad/metadata_fixture_gen/`,不进仓库)以本仓库锁定同版本的 `frame-metadata=23.0.1` / `scale-info=2.11.6` / `parity-scale-codec=3.7.5` 手搭一段**格式真实**的 `RuntimeMetadataV14`(非简化/伪造格式),含 `OnchainTransaction.OnchainMinFee`(u128=10)、`Balances.ExistentialDeposit`(u128=111)、`Balances.SomeFlag`(bool,覆盖非整数类型拒绝分支)三个常量,固化成 153 字节 hex 常量。测试用 `_FixtureChainRpc extends ChainRpc` 只覆写 `fetchMetadata()` 跳过 smoldot 原生桥,`fetchPalletConstant`/`fetchPalletConstantU128`/`fetchMinSelfPayBalanceFen` 全走真实实现(真 `RuntimeMetadata.fromHex` → 真 `Constant.type.decode(ByteInput(bytes))`)。5 个用例覆盖:两个常量正确解出、pallet/常量名不存在抛 `StateError`、非整数类型被拒、`fetchMinSelfPayBalanceFen`=10+111=121。`flutter test test/rpc/chain_rpc_test.dart` **5/5**,`dart analyze`/`dart format` 干净。
  - **充值 confirm 的 DoS 面收窄**：`orders.ts` 新增 `enforceGlobalEvmRpcLimit(env, chainId)`,键 `topup_confirm_evm_rpc:chain:${chainId}`,300 次/60 秒,**不按 IP/账户维度**——因为 `account_id` 无需注册可无限换号、IP 可换代理池,两层限流都挡不住"分布式滥用把外部 Base RPC 配额/账单打爆";全局硬顶是唯一挡得住的手段。挂载点在 `topupConfirmRoute` 里 `findOrderByTx` dedupe 短路**之后**、`verifyErc20Payment` **之前**——已有订单的重复轮询不打 RPC,不消耗这个全局配额,只有真正要发起新 RPC 调用的请求才计数。新增测试:逐次换全新账户 + 全新 tx hash confirm 300 次全部放行,第 301 次 429 `request_rate_exceeded`,证明是全局硬顶生效而非账户/IP 限流(每个账户只用一次、每个 tx hash 从未出现过,dedupe 不短路)。`vitest run test/topup.test.ts` **13/13**。
  - **验收**:Worker `npm run typecheck` 干净;`npx vitest run` 单独跑本轮涉及的 3 个文件(`topup.test.ts`/`chain_identity.test.ts`/`device_subkey.test.ts`)**3 文件 28/28 全绿**;全量 `npx vitest run` 最终干净跑到 **32 文件 210 测试全绿**。
  - **⚠️ 排查记录,避免重蹈上次覆辙**:全量跑的中间几次出现 `account.test.ts`/`chat.test.ts`/`contacts.test.ts`/`media.test.ts`/`notify.test.ts` 等文件失败(次数在 49→22→1→0 之间跳变,不确定性明显)。这次**没有直接假设"与我无关"**,逐项核实:①这 5 个文件及其依赖的 `account/purge.ts`/`account/service.ts`/`rebind/service.ts` 均**不在本轮我的编辑清单里**;②`git status` 显示这些文件与另外一批(`auth.test.ts`/`profiles.test.ts`/`rebind_data_survives.test.ts` 等我完全没碰过的文件)同时处于未提交状态,是仓库里另一条并行推进的重构(聊天广场 CID 主键线程)遗留;③这 5 个文件逐一单跑**100% 绿**(15+6+5+3+5=34/34);④两次全量跑之间测试文件数从 32 跳到 38 又跳回 32,说明跑测试期间仓库正被其它并发会话修改。结论:这是外部并发改动引起的瞬时不确定性,不是本轮改动引入的回归——但仍然是如实记录、给证据链,不是重复上次"跑一两个不相干文件就下结论"的错误。
  - **本窗口范围内的三项缺口(fetchPalletConstant 单测 + confirm DoS 收窄 + 币标)现只剩币标目视验证 + 链相关真机 E2E,均已如上或既有条目记录待办。**

## 待办（下一步）

- 正式链恢复出块后补真机端到端：① 新钱包能建成（不再被 `cid_not_bound` 阻断）；② 注册前余额闸三分支；③ 已有子钥进入广场零生物识别，Worker 确认缺钥时只鉴权一次并继续原登录；④ 客户端确实从 metadata 读到三个费率常量。
- 币标（USDC/USDT SVG）未做真机/模拟器目视确认。
