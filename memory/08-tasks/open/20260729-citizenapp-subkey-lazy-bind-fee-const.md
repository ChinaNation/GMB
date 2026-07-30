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
  - **修掉一个被时机变更打破的旧假设**：完成标记不能在钱包创建时写基线，否则会
    谎称设备已绑定。最终实现已升级为 `reconcileFinalizedBindingTakeover` 对账完整
    `(cid_number, binding_revision, account_id, data_root_hash)`，只由真实接管完成路径推进。
  - **Worker**：`chain/identity.ts` 新增 `fetchChainIdentityStateFreshIfUnbound`（缓存无 CID 才回源，有 CID 直接采信不多打链）；`auth/service.ts` 的 `registerDeviceSubkey` 改用它。
  - **币轨图标**：新增 `assets/icons/usdc.svg`（`#2775CA`）、`usdt.svg`（`#26A17B`），手写 SVG、无外部依赖；`_RailCard` 按 `rail.token` 渲染，未登记币种回退 USDC 底图不崩页；`pubspec.yaml` 已注册。
  - **测试**：Worker `typecheck` 干净 + `vitest run` **205/205**（新增身份缓存旁路两例：无 CID 回源、有 CID 不回源）。Flutter `dart analyze lib test` **零问题**；`test/my/myid/` **65/65**（新增懒绑定三例 + 余额闸两例 + 门禁绑定两例）、`test/myid_page_test.dart` **20/20**（新增余额闸三分支）、`test/wallet/` **145/145**（原「门禁0 子钥强绑定」三例按新契约重写为「建钱包不注册子钥 + 唯一绑定入口」）、`test/8964/square_publish_service_test.dart` **8/8**。
  - **⚠️ 更正一处误判（2026-07-29 复盘）**：本卡上一版曾写「全量约 40 项失败是既有干扰、与本任务无关」——**这是错的**，当时只单跑了两个不相干文件就下结论。复查发现那 40 项**绝大多数正是本次 gate 改动引起的连锁**：`IdentityRegistrationGate` 放行前新增「按需绑子钥」，默认绑定器 `MyIdService().ensureDeviceSubkeyBound` 会 new 真 `ChainRpc` 打 smoldot；被 gate 包裹的页面测试只注入了 `debugResolver` 桩、没配套桩绑定器，导致放行前触发真链读、`pumpAndSettle timed out`，并把 `--concurrency=1` 下的后续套件一起带崩。**根因证据**：`square_home_page_test.dart` 单跑当时 0/4，报 `pumpAndSettle timed out` + `[ChainRpc] 使用 smoldot 轻节点模式`。
  - **修复**：唯一测试 helper `test/support/identity_gate_test_util.dart` 的 `useRegisteredIdentityGate()` 在设 `debugResolver` 的同时注入 no-op `debugSubkeyBinder`（tearDown 一并清）。这是所有被 gate 包裹的页面测试的统一绕过点，一改全覆盖。
  - **全量绿**：修复后 `flutter test --concurrency=1` 全量 **1017 passed / 5 skipped / 0 failed（`All tests passed!`）**。4 个受影响文件单跑复核：`square_home_page` 4/4、`chat_tab` 20/20、`membership_page` 21/21、`contact_book` 9/9。5 个 skipped 是无 native libsmoldot 时的既有 skip 守卫（`smoldot_native_probe`），非失败。
  - **真实验收的边界（如实记录）**：链端只做到 `cargo test` + 整体 `cargo check`；常量经 metadata 下发到客户端、余额闸与懒绑定的端到端真机验证**需要正式链可出块**，而正式链当前仍被 `20260717-citizenapp-stablecoin-topup.md` 记录的创世/节点守卫不一致阻塞，故本轮**未做真机端到端**。Worker 侧新分支由单测覆盖（`device/register` 的活体验证同样依赖链）。
  - **文档**：`memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md` §4.5 补「设备子钥懒绑定」与「注册前余额闸 + 交易费常量单源」两条；死契约 `square-session-never-lazy-register` 已更新子钥注册时机并说明为何不放松 ANR 立契理由。

- **2026-07-29 · 收尾复检(遗漏/残留/安全/改进)**
  - **补清残桩注释(违反无残桩,上一版遗漏)**：子钥改懒绑定后,全仓仍有 9 处旧注释把「子钥注册」说成钱包创建/导入时发生——现已全部改正：`create_wallet_onboarding_page.dart`(类头 + 失败文案 + `_openImport`)、`import_wallet_page.dart`(类头 + 失败文案)、`wallet_manager.dart`(`WalletSubkeyRegistrar` typedef 文档 + `_subkeyRegistrar` 字段文档 + `_Account0` 派生用途)、`main.dart`(钩子注入注释)、`chat_runtime.dart`(后台握手注释)、`square_session_provider.dart`(类头)。均为纯注释,零逻辑改动。上一版任务卡只 claim 了 onboarding 一处且实际没做,本轮全部落实。
  - **核实无问题的点**：① pallet 名 `Balances`/`OnchainTransaction` 与
    `construct_runtime` 一致；②结算接口仍走 `TOPUP_SETTLE_TOKEN` 鉴权；③后台
    `ensureSession` 不懒注册设备；④ finalized 接管标记只在数据根与设备真实接管完成
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

- 正式链恢复出块后补真机端到端：① 新钱包能建成（不再被 `cid_not_bound` 阻断）；② 注册前余额闸三分支；③ 初次进广场触发子钥绑定；④ 客户端确实从 metadata 读到三个费率常量。
- 币标（USDC/USDT SVG）未做真机/模拟器目视确认。
