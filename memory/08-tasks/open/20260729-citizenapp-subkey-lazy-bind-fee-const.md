# 任务卡：设备子钥懒绑定 + 链上费率常量下发 + 注册前余额闸

> 状态：**执行中**。跨 citizenchain runtime + CitizenApp + Cloudflare Worker 三端。
> 前置：`20260717-citizenapp-stablecoin-topup.md` 的 2026-07-29 节（充值鉴权与 CID 解耦）已完成，充值可充到任意钱包账户。

## 任务需求

本窗口原始需求是「身份页点『确认注册』时先检查钱包余额，不足则跳转该账户的链上充值页」。诊断过程中发现两个更上游的阻断，必须一并解决：

1. **新用户建不出钱包**：`WalletManager.createWallet/importWallet` 对 P-256 设备子钥注册 fail-closed，而 Worker `device/register` 自 2026-07-28 commit `9aeab195` 起要求账户已绑 CID——建钱包那一刻必然没有 CID。
2. **客户端无法拿到链上最低交易费**：`ONCHAIN_MIN_FEE` 只活在 `primitives::fee_policy`，未暴露到 metadata；客户端要预检余额就只能复刻常量，违反「交易费常量全仓只有区块链常量库一处」。

## 用户定稿（不得偏离）

- **子钥绑定与 CID 注册解耦，改懒绑定**。理由：用户可以只用 CitizenApp 的钱包、交易等功能，这些不需要 CID。注册 CID 时签名是因为**要扣钱**；注册子钥是因为**要用广场/聊天/通讯录等需 CID 的场景**。所以用户注册 CID 之后并不马上注册子钥，而是**初次进入需要 CID 的页面时**才注册。两次签名分属两个不同的用户动作，天然分开。
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

### 3. 设备子钥懒绑定

- 删 `WalletManager._registerDeviceSubkey` 及 `createWallet`/`importWallet` 两处调用；建钱包回到纯本地，不依赖 Worker。
- `rebindDeviceSubkeyToAccountId` 改名 `bindDeviceSubkeyToAccountId`（同时承担首次绑定与换绑）。
- 触发点 = `IdentityRegistrationGate`：判定 `registered` 准备放行时，若本机子钥尚未绑到当前身份账户，先绑（弹一次生物识别）再放行。五处 gate（广场/聊天/通讯录/创作者/会员）共用。
- 多 gate 并发挂载 → in-flight Future 去重。
- 绑定失败不放行（与 gate 现有 fail-closed 一致），给重试；不影响钱包/交易 tab。
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
- `citizenapp/lib/my/myid/`（代码）：`identity_registration_gate.dart` 懒绑定+并发去重；`myid_page.dart` 余额闸与充值跳转。
- `citizenapp/lib/8964/services/`（代码+残留清理）：`square_publish_service.dart` 删硬编码费率常量改读链。
- `citizenapp/lib/transaction/onchain-topup/`（代码）：`_RailCard` 换真实币标。
- `citizenapp/assets/icons/`（资源）：新增 `usdc.svg`、`usdt.svg`。
- `citizenapp/pubspec.yaml`（配置）：注册两枚 SVG。
- `citizenapp/cloudflare/src/{auth,chain}/`（代码）：子钥注册端点「无 CID」时强制回源链读。
- `citizenchain/.../tests/`、`citizenapp/cloudflare/test/`、`citizenapp/test/`（测试）：常量暴露、缓存旁路、懒绑定、余额闸三分支。
- `memory/07-ai/`、`memory/08-tasks/`、`memory/01-architecture/`（文档）：更新死契约文字、本卡、架构文档。

## 死契约变更（已获用户确认）

`square-session-never-lazy-register` 原文「注册只在钱包创建」需更新为「子钥注册在初次进入需 CID 页面时」。该契约的立契理由是**后台 `ensureSession` 路径懒注册会 ANR**；本次触发点是前台、用户可见、用户主动进页面，不碰后台路径，立契理由不受影响。

## 执行记录

- **2026-07-29 · 全部落地**
  - **链端**（`runtime/transaction/onchain`）：pallet Config 加 `OnchainMinFee` / `OnchainFeeRate` / `VoteFlatFee` 三个 `#[pallet::constant]`；`configs.rs` 用 `ConstU128<{ primitives::fee_policy::… }>` 与 `parameter_types! RuntimeOnchainFeeRate` 绑定，**只转发不另立数字**。pallet 测试 mock 同步绑定，并新增守卫用例 `exposed_fee_constants_forward_fee_policy_exactly`（逐项 assert 等于 `fee_policy` 真源，有人在绑定处改写数字立刻红）。`cargo test -p onchain` **22/22**，`cargo check -p citizenchain` 整体通过。未动 storage / call 索引 / genesis / 收费逻辑。
  - **客户端读常量**：`chain_rpc.dart` 加 `fetchPalletConstant` / `fetchPalletConstantU128` / `fetchMinSelfPayBalanceFen`（`= OnchainMinFee + Balances.ExistentialDeposit`）。走已有 `metadata.chainInfo.constants`，零新依赖、零新网络往返（metadata 已缓存）。常量缺失抛 `StateError` 由调用方 fail-closed，不兜默认值。
  - **删掉 Dart 侧费率副本**：`square_publish_service.dart` 的 `publishFeeFen = 10` / `accountExistentialDepositFen = 111` / `minimumPublishBalanceFen` 三个常量**全删**，改为 `SquarePublishBalanceReader.fetchMinSelfPayBalanceFen()` 读链。全仓 grep 三个符号已零命中。
  - **子钥懒绑定**：删 `WalletManager._registerDeviceSubkey` 及 `createWallet`/`importWallet` 两处调用（建钱包回到纯本地、不依赖 Worker）；`rebindDeviceSubkeyToAccountId` → `bindDeviceSubkeyToAccountId`（首次绑定与换绑共用）；`MyIdService.ensureDeviceSubkeyBound()` 带 in-flight 去重，未注册 CID 直接拒（后端不收）；`IdentityRegistrationGate` 新增 `_GateStatus.bindFailed` + `subkeyBinder` 注入点 + `debugSubkeyBinder` 测试钩子，放行前按需绑定、失败不放行且不自愈（要弹生物识别，只能用户点重试）。
  - **修掉一个被时机变更打破的旧假设**：`_reconcileIdentityRebuild` 的 `synced == null` 分支原会写基线（注释称「账户0 凭证在钱包创建时已就绪」）。懒绑定后该前提不成立，写基线等于谎称已绑、会让 `ensureDeviceSubkeyBound` 永久短路。已改为该分支**不写基线**，标记只由真正完成绑定的路径推进。
  - **Worker**：`chain/identity.ts` 新增 `fetchChainIdentityStateFreshIfUnbound`（缓存无 CID 才回源，有 CID 直接采信不多打链）；`auth/service.ts` 的 `registerDeviceSubkey` 改用它。
  - **币轨图标**：新增 `assets/icons/usdc.svg`（`#2775CA`）、`usdt.svg`（`#26A17B`），手写 SVG、无外部依赖；`_RailCard` 按 `rail.token` 渲染，未登记币种回退 USDC 底图不崩页；`pubspec.yaml` 已注册。
  - **测试**：Worker `typecheck` 干净 + `vitest run` **205/205**（新增身份缓存旁路两例：无 CID 回源、有 CID 不回源）。Flutter `dart analyze lib test` **零问题**；`test/my/myid/` **65/65**（新增懒绑定三例 + 余额闸两例 + 门禁绑定两例）、`test/myid_page_test.dart` **20/20**（新增余额闸三分支）、`test/wallet/` **145/145**（原「门禁0 子钥强绑定」三例按新契约重写为「建钱包不注册子钥 + 唯一绑定入口」）、`test/8964/square_publish_service_test.dart` **8/8**。
  - **全量 Flutter 测试的既有干扰（非本次引入）**：`flutter test --concurrency=1` 全量有约 40 项失败，但同一批文件单跑全绿——包括本次**完全没碰过**的 `test/wallet/attestation_service_test.dart` + `test/rpc/smoldot_client_lifecycle_test.dart`（单跑 17/17 绿）。日志可见 `Failed to lookup symbol 'smoldot_client_init'`，属本工作区用例间状态干扰 + 测试环境无 native libsmoldot 的既有问题，与本任务无关，未在本卡范围内处理。
  - **真实验收的边界（如实记录）**：链端只做到 `cargo test` + 整体 `cargo check`；常量经 metadata 下发到客户端、余额闸与懒绑定的端到端真机验证**需要正式链可出块**，而正式链当前仍被 `20260717-citizenapp-stablecoin-topup.md` 记录的创世/节点守卫不一致阻塞，故本轮**未做真机端到端**。Worker 侧新分支由单测覆盖（`device/register` 的活体验证同样依赖链）。
  - **文档**：`memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md` §4.5 补「设备子钥懒绑定」与「注册前余额闸 + 交易费常量单源」两条；死契约 `square-session-never-lazy-register` 已更新子钥注册时机并说明为何不放松 ANR 立契理由。

## 待办（下一步）

- 正式链恢复出块后补真机端到端：① 新钱包能建成（不再被 `cid_not_bound` 阻断）；② 注册前余额闸三分支；③ 初次进广场触发子钥绑定；④ 客户端确实从 metadata 读到三个费率常量。
