# 任务卡：CitizenApp 充值与 CitizenConsole 发币端到端验收

> 状态：安全缺陷修复和 staging 主网配置完成；真实付款在付款前被正式链节点守卫阻塞

## 任务需求

真实测试 CitizenApp「我的 → 钱包 → 我的钱包 → 钱包详情 → 充值」中的 USDC（Base）与 USDT（Base）两种充值，以及 CitizenConsole「充值发币」功能，验证完整业务是否跑通，审计安全漏洞并提出值得更新的内容。

## 所属模块

- `citizenapp`：充值页面、WalletConnect 支付、订单提交、状态轮询和到账展示。
- `citizenapp/cloudflare`：充值配置、建单、EVM 初验、待发币队列、结算回写和 D1 台账。
- `citizenconsole`：Keychain、Touch ID 会话、EVM 独立复验、链上发币、幂等和本地台账。
- `citizenchain`：仅验证既有链上转账与到账结果；本任务默认不修改 runtime。

## 输入文档

- `memory/08-tasks/open/20260717-citizenapp-stablecoin-topup.md`
- `memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`
- `memory/01-architecture/gmb/GMB_TECHNICAL.md`
- `memory/03-security/security-rules.md`
- `memory/07-ai/unified-protocols.md`
- `memory/07-ai/definition-of-done.md`
- `memory/07-ai/module-definition-of-done/citizenapp.md`

## 安全边界

- 默认只允许本地或测试环境。2026-07-25 用户另行明确授权在 staging Worker/D1
  上分别执行最低档 USDC、USDT Base 主网付款；本次因公民链阻塞在外部钱包付款前停止，
  实际未支出稳定币，也未发放公民币。
- AI 不读取、记录或输出任何私钥、Secret、Keychain 明文或会话种子。
- 不执行 GitHub 推送、PR、远端 workflow 或生产部署。
- 任何可能改变 `citizenchain/runtime/` 的操作必须先取得单独的第二次确认。
- 静态审计结论必须回到真实代码和运行输出核验，并记录证据锚点。

## 测试范围

- [ ] 核对两种充值入口、套餐、链、代币合约和收款地址来源。
- [ ] 核对建单、支付、交易哈希上报、Worker 初验和状态轮询。
- [ ] 核对 CitizenConsole 队列拉取、独立 EVM 复验、发币和结算回写。
- [ ] 验证同一 EVM 交易哈希幂等、重复执行、并发和重启后的防重复发币。
- [ ] 验证错链、错币、错合约、错收款地址、少付、超付、未确认和回滚场景。
- [ ] 验证订单与钱包绑定、参数篡改、越权调用、CSRF/Origin、会话和日志脱敏。
- [ ] 核对 Worker、CitizenConsole、本地台账和 CitizenChain 四方状态一致性。
- [ ] 使用真实本地服务、真实测试数据库、真实 HTTP 接口和真实页面记录验收结果。

## 预计修改目录

- `memory/08-tasks/open/`：仅记录测试步骤、证据、缺陷和结论；涉及文档，不改业务代码。
- 发现缺陷后，先报告影响和修复方案；未经用户进一步确认，不修改 `citizenapp/`、`citizenconsole/` 或 `citizenchain/` 业务代码。

## 输出物

- 两种充值与发币业务的端到端测试结果。
- 带代码或运行输出锚点的漏洞和风险清单。
- 值得更新的功能、体验和安全建议。
- 任务卡中的测试证据、受阻项和最终结论。

## 验收标准

- 两种充值分别得到 `PASS`、`FAIL` 或 `BLOCKED` 的明确结论。
- CitizenConsole 发币链路得到真实运行态结论。
- 每个漏洞或风险都有证据锚点，不以推测代替检查。
- 明确区分已验证、尚未验证和因环境受阻的项目。
- 未泄露 Secret，未触碰生产资产，未产生重复发币。

## 执行记录

- 2026-07-18：用户确认创建本任务卡并开始测试；限定本地/测试环境，不触碰生产发币和 GitHub 远端。
- 2026-07-18：确认当前目标态是 USDC 与 USDT **都走 Base**；主实现任务卡和代码中仍有 Arbitrum 旧描述，列为残留。
- 2026-07-18：本机 CitizenConsole `127.0.0.1:8888` 页面真实打开成功，充值发币配置 9/9 显示已配置、会话未解锁、台账 0 条、页面无 console error。为避免真实发币，未点击解锁和结算。
- 2026-07-18：本机节点 RPC `127.0.0.1:9944` 返回 `peers=2`、`isSyncing=false`，创世哈希与仓库配置一致；不得把该节点推断成孤立开发分叉。
- 2026-07-18：production 公开充值配置只读请求返回 HTTP 200、`mainnet`、USDC/USDT 两轨均为 Base 8453、2 个套餐；未提交订单或交易。
- 2026-07-18：staging 公开入口由 Cloudflare Access 返回 302；Wrangler 只读检查进一步确认 staging 未具备充值验收条件：`0001/0003/0004/0005` 四个迁移全部待执行、`topup_orders` 表数量为 0、7 个 topup vars 未写入 `env.staging.vars`、Secret 列表缺少 `TOPUP_SETTLE_TOKEN`。

## 测试结果

| 检查项 | 结果 | 证据 |
|---|---|---|
| CitizenApp 充值与钱包入口测试 | PASS | `flutter test test/transaction/topup test/wallet/widgets/wallet_action_card_test.dart`：16/16 |
| CitizenApp 充值相关静态检查 | PASS | `dart analyze lib/transaction/onchain-topup lib/wallet/widgets/wallet_action_card.dart`：0 issue |
| Worker topup 测试 | PASS（仅现有覆盖） | `test/topup.test.ts`：10/10；Worker 全量 178/178；`tsc --noEmit` 通过 |
| CitizenConsole 语法/脚本检查 | PASS | `npm run check` 通过 |
| CitizenConsole 资金安全测试 | **FAIL** | 22 项中 16 通过、6 失败；结算 happy path、双幂等、交叉校验和并发锁测试全部失效 |
| CitizenConsole 真实页面 | PASS（只读） | 配置页、两币合约子行、台账页均正常渲染，无 console error |
| USDC Base 主网真实付款→发币 | **BLOCKED（付款前）** | staging 已就绪，但更新后节点守卫拒绝正式链区块 #5，无法提供 finalized 发币证明 |
| USDT Base 主网真实付款→发币 | **BLOCKED（付款前）** | 与 USDC 相同；未打开外部钱包确认页，未产生稳定币支出 |
| CitizenConsole 真实发币 | **BLOCKED（链前置条件）** | 控制台、结算令牌和发币会话均已验证；正式链观察节点无法导入区块 #5 |

## 审计发现

### H1：公开交易可被抢先绑定到攻击者公民链钱包

- `citizenapp/cloudflare/src/security/request_guard.ts:95`：全部 topup App 接口跳过钱包 Session，只做 IP 限流。
- `citizenapp/cloudflare/src/topup/orders.ts:83`：付款后才提交 `gmb_address + evm_tx_hash`；`payer_address` 可空，服务端没有验证提交者控制付款 EVM 地址或目标公民链钱包。
- `citizenapp/cloudflare/src/topup/orders.ts:103`：唯一绑定发生在首次上报；公开链上的 tx hash 被第三方观察后，可先提交并绑定自己的 `gmb_address`。
- 影响：真实付款人可能丢失对应公民币，攻击者成为首个入库者。
- 建议：改为付款前创建短期订单，订单必须绑定公民链钱包签名、EVM 付款地址、币种、套餐、金额、收款地址、nonce 和过期时间；付款后只允许该订单完成，禁止匿名事后认领公开 tx hash。

### H2：本地台账损坏或丢失时幂等会 fail-open，存在重复发币窗口

- `citizenconsole/topup/ledger.mjs:15`：读取、JSON 解析等任意异常均直接返回空对象。
- `citizenconsole/topup/routes.mjs:227`：是否已发币仅依赖本地台账的 `gmb_tx_hash/disbursing`。
- 影响：公民币已 finalized、Worker 尚未成功回写时，如 `.runtime/topup-ledger.json` 损坏、丢失或换机，下一次会把订单当作从未发币并再次转账。
- 建议：Worker 先持久化原子 claim/disbursing 租约；链上 remark 用稳定订单 ID，并在重试前按发币地址、订单 ID、收款人和金额查询已 finalized 交易；本地台账解析失败必须停止全部结算，不得返回空台账继续。

### H3：WalletConnect 支付执行代码运行时从未锁版本的第三方 CDN 加载

- `citizenapp/assets/topup/walletconnect.html:32`：运行时加载 `https://esm.sh/@walletconnect/ethereum-provider@2`。
- 影响：远端内容、浮动大版本或 CDN 被污染时，可改变钱包请求内容；这是直接触达付款签名的供应链边界。
- 建议：将审核过的 WalletConnect 依赖锁定精确版本并随 App 打包，增加 CSP；原生桥只接受格式、链、合约、金额均可复核的结果。

### M1：Worker 置已支付不独立验证公民链交易

- `citizenapp/cloudflare/src/topup/settlement.ts:69`：`settled` 只校验 32 字节格式并重新检查 EVM 到账，随后直接把订单置 `paid`。
- `citizenconsole/topup/chain_transfer.mjs:28`：控制台连接配置的任意 WS 节点，没有在发币前校验预期 genesis/network。
- 影响：结算令牌泄露、控制台配置错误或连接错误分叉时，Worker 仍可接受一个未向目标钱包真实到账的哈希。
- 建议：订单与结算回写携带预期公民链 genesis；控制台发币前硬校验 genesis；Worker 在可信 finalized RPC 上复核交易、from/to/amount/remark 后再置 paid。

### M2：CitizenConsole 核心回归测试已经失效

- `citizenconsole/test/settle.test.mjs:13`：夹具仍配置旧 `TOKEN_CONTRACTS`。
- `citizenconsole/topup/routes.mjs:14`：实现已改为 `TOKEN_USDC/TOKEN_USDT`。
- 结果：结算 happy path、两层幂等、收款/合约守卫和并发锁 6 项全部返回 400 或后续断言失败。
- 建议：先恢复测试到 22/22，并新增 USDC、USDT 两条完整结算路径和配置缺失门禁。

### M3：沙箱环境尚未落地，真实端到端验收当前不可能

- `citizenapp/cloudflare/wrangler.toml:125`：staging vars 没有任何 topup 配置；Wrangler 明确警告环境 vars 不继承顶层 vars。
- staging D1 只读检查：`topup_orders` 不存在，全部迁移待应用；staging Secret 列表缺少 `TOPUP_SETTLE_TOKEN`。
- 建议：单独授权后配置 Base Sepolia 收款地址/RPC/USDC/mock USDT、部署结算令牌、应用 staging 迁移并部署 staging Worker，再用明确 `SQUARE_API_URL=/api-staging` 的真机包验收。

### M4：收到稳定币后的异常没有人工闭环

- `citizenconsole/topup/routes.mjs:296`：任何发币错误都会把订单永久置 `exception`。
- `citizenconsole/web/citizenconsole.js:204`：台账只展示，没有核链、补发、退款、重新回写或关闭异常的操作入口。
- 影响：用户已经付款，但节点抖动、余额不足或临时错误会直接进入失败，运营人员没有受控恢复流程。
- 建议：建立经过再次 EVM/GMB 核验和 Touch ID 的“补发/退款/关闭异常”唯一流程，所有动作写审计事实并保持幂等。

### L1：EVM RPC 响应实际字节缺少完整上限

- `citizenapp/cloudflare/src/topup/evm_verify.ts:159`：只检查可选的 `Content-Length`，随后直接 `response.json()`；chunked 响应可绕过上限。
- `citizenconsole/topup/evm_verify.mjs:98`：控制台侧没有响应字节上限。
- 建议：流式有界读取实际字节后再解析 JSON，与仓库 Worker 资源上限规则一致。

### L2：目标态残留和测试覆盖不足

- `citizenapp/cloudflare/src/topup/config.ts:6`、`citizenconsole/topup/evm_verify.mjs:3`、`citizenapp/lib/transaction/onchain-topup/onchain_topup_page.dart:145` 等仍写 USDT/Arbitrum；`TOPUP_ARBITRUM_RPC_URL` 仍残留在类型、配置和测试。
- Worker 的 10 项 topup 测试只用 USDC 提交；USDT 只检查 config 出现，没有提交、到账、队列和结算覆盖。
- `citizenapp/lib/transaction/onchain-topup/topup_result_page.dart:13` 仍有 `unresolved/仍在确认` 第三种用户结果，与主任务卡“用户只见成功/失败”口径不一致。

## v1 误判撤回

- 撤回“CitizenConsole 连接的是孤立开发链，因此一定跨环境发币”的初步线索。真实 RPC 复核显示该节点有 2 个 peer、已同步且 genesis 与目标配置一致；现有证据不足以认定它是孤立分叉。保留的问题是代码没有强制校验 genesis，且本次安全约束不允许对 production Worker 执行真实试单。

## 当前结论

- H1、H2、H3、M1、M2 已于 2026-07-25 按唯一目标态修复：匿名交易认领已删除；本地台账损坏 fail-closed；WalletConnect 精确版本随 App 打包；Worker 独立核验 finalized 公民链发币事实；CitizenConsole 资金安全测试恢复并扩展到 27/27。
- 本地真实 Worker/D1/HTTP 已验证账户会话、短期付款意图、篡改拒绝和付款前零订单。**仍不能把整个业务判定为完全跑通**，因为正式链节点守卫尚未通过，真机 Base 主网付款→EVM finalized→CitizenConsole 发币→CitizenChain finalized→CitizenApp 到账尚未执行。
- 在正式链门禁和真机 staging 两币轨验收通过前，仍不开放 production 充值。

## 2026-07-25 staging 主网预验收与阻塞证据

- 已按用户确认清空并重建 staging D1，唯一基线迁移 `0001_square_core.sql`
  成功登记；`topup_orders` 为新 21 字段结构且为空，历史业务数据未保留。
- staging Worker 已配置 Base 主网：收款地址
  `0x5ce9b56b9d1812a7cf29841e21756f09ca7d223b`，USDC 合约
  `0x833589fcd6edb6e08f4c7c32d4f71b54bda02913`，USDT 合约
  `0xfde4c96c8593536e31f229ea8f37b2ada2699bb2`，RPC
  `https://mainnet.base.org`，最小 EVM 确认口径为 finalized。
- 新 `TOPUP_INTENT_SECRET` 已写入 staging Worker；`TOPUP_SETTLE_TOKEN`
  与 CitizenConsole 已配置值一致。部署版本为
  `3fc28e6e-6d5b-43d2-84ad-0ea4d87ee400`。匿名临时 Access 放行仅用于预验收，
  结束后已删除并复核接口重新返回 Access 302。
- Worker `typecheck` 通过，全量测试 174/174。后续经用户确认已把 lockfile 中的开发依赖
  `postcss` 更新到 `8.5.23`，并随其最低依赖要求把 `nanoid` 更新到 `3.3.16`；
  未新增直接依赖、未改 Worker 生产代码。重新执行 `npm ci`、typecheck、174 项测试和
  `npm audit --audit-level=low` 均通过，当前审计为 0 项漏洞。
- CitizenConsole 临时指向 staging Worker 和独立正式链观察节点时，发币钱包会话保持解锁，
  创世哈希 `0x840d5b12c541a010783e54069c9168a13d102ba63cd8f3a00263440c1803aad9`
  与正式链一致，节点连接到 1—2 个对等节点并同步到区块 #4。
- debug 与 release 两份现有节点产物都稳定拒绝同一个正式链区块 #5
  `0x15129239cf7a517832ee923108655d4552b3e46b97a2374877ffdd4c52556c7b`，
  节点守卫理由为 `RewardAuditInvalid`。启动自检同时报告管理员账户解码、
  省储行质押账户和机构存储结构与当前冻结创世状态不一致。
- 因 Worker 只接受 finalized 公民币发币证明，继续付款会形成“稳定币已扣、公民币无法
  finalized 发放”的资金风险。本次严格停止在外部钱包付款前：USDC/USDT 均未支出，
  D1 订单仍为零。
- 测试结束后已恢复 CitizenConsole 的 production Worker URL 和 `9944` 节点配置，
  删除临时链指纹、临时观察节点数据和 Access 放行策略；生产 Worker、生产 D1、
  GitHub、runtime 和正式创世均未触碰。

### 下一步门禁

必须先让更新后的节点代码、冻结 chainspec 和正式链状态采用同一套创世结构，并用至少
两个节点真实验证：启动自检全通过、区块 #5 可导入、best/finalized 持续推进、发币账户
余额可读。上述四项全部通过后，才重新执行真机 USDC/USDT 主网付款。

## 2026-07-25 修复验收

| 原发现 | 当前结论 | 修复证据 |
|---|---|---|
| H1 匿名 tx hash 抢绑 | RESOLVED | `intent` 的 `account_id` 只取 Bearer 会话，HMAC 绑定 payer/报价；`confirm` 后才入库，`status` 强制账户归属 |
| H2 台账损坏重复发币 | RESOLVED | 台账仅 `ENOENT` 视为空；解析/IO 错误停止结算；Worker 原子 claim 不自动过期 |
| H3 WalletConnect CDN 供应链 | RESOLVED | provider `2.23.10` 本地 bundle + CSP；HTML 无远端 module import |
| M1 Worker 不验公民链事实 | RESOLVED | 控制台提交完整 finalized 证明；Worker 验 genesis、signer、call 参数、canonical/finalized 包含关系 |
| M2 控制台回归测试失效 | RESOLVED | CitizenConsole 全量 27/27；包含完整证明回写、双幂等、claim、独立守卫和台账损坏 fail-closed |
| 真机 testnet 完整资金流 | BLOCKED | 尚未配置并授权 testnet 外部钱包、收款资产与测试发币环境 |

## 2026-07-25 正式创世前充值前置条件复核

- 当前源码 fresh block#0 与完整 NodeGuard 扫描均通过，旧正式链 block#5
  `RewardAuditInvalid` 问题不再用旧链状态判断新版源码；正式冻结仍必须以新 CI WASM、
  新 chainspec 和清空后的正式数据重新验收。
- 两个隔离节点已建立真实 P2P 连接并读取同一创世哈希；由于交易池为空、临时矿工账户
  创世余额为 0，且本机没有正式 GRANDPA 私钥，本轮未能验证非创世区块导入及
  finalized 持续推进。不得通过修改创世余额或替换正式权威来伪造该结果。
- CitizenConsole 截图中“发币地址”解码后的规范 AccountId 为
  `0x36d00d0a9701b6e860c51476ce2d7ac5f3b35b6ff067b81d958afa1b0551c303`；
  fresh 创世中该账户可读但余额为 0。`transfer_with_remark` 入口与链上交易费路由专项测试
  已通过，但余额为 0 时不能向充值用户发放公民币。
- 基金会费用账户在 fresh 创世中同样为 0，基金会机构操作在合法注资前不能支付交易费；
  该账户与 CitizenConsole 发币账户必须分别确定资金来源、金额和执行时点，不能混为一个账户。
- 因此正式创世前仍有两类硬门禁：用户先确认两类账户的合法资金安排；再在至少一名
  正式出块/终局权威可工作的隔离窗口验证 best、跨节点导入和 finalized 持续推进。两项完成前
  不恢复 USDC/USDT 主网真实付款。

## 2026-07-25 CitizenConsole 与 Worker 冻结前复验

- CitizenConsole 实际配置项以现有页面和源码为准：发币地址、`WORKER_BASE_URL`、
  `SETTLE_TOKEN`、`BASE_RPC_URL`、`RECV_ADDRESS`、USDC/USDT 合约白名单、
  `NODE_WS`、`MIN_CONFIRMATIONS`。没有把 Worker 内部的
  `TOPUP_INTENT_SECRET`、`TOPUP_DISBURSE_ACCOUNT_ID` 或 Cloudflare 绑定误加到控制台 UI。
- CitizenConsole 27 项测试全部通过，覆盖 Base EVM finalized 转账核验、收款地址与代币
  白名单交叉校验、台账原子写、损坏 fail-closed、Touch ID 回显限制、发币幂等和并发锁；
  `npm audit --audit-level=low` 为 0。
- Worker typecheck、174 项全量测试和依赖审计全部通过；USDC/USDT 同走 Base 的配置与
  CitizenConsole 当前字段映射一致。
- production Worker 的 `TOPUP_DISBURSE_ACCOUNT_ID` 已按 CitizenConsole“发币地址”
  解码后的规范 AccountId 写入
  `0x36d00d0a9701b6e860c51476ce2d7ac5f3b35b6ff067b81d958afa1b0551c303`；
  Worker 可据此独立核验最终链上转账签名者，不再依赖空配置。
- 写入公开 AccountId 不等于业务开放：正式创世时该账户仍为零余额；正式创世后必须先部署
  国储会 GRANDPA 节点并验证 finalized 持续推进，再按用户确认向该账户转入合法资金。
  两项完成前不得执行真实主网付款。
- Worker 环境类型已从手写 `@cloudflare/workers-types` 改为
  `wrangler types --env-interface CloudflareBindings` 生成
  `worker-configuration.d.ts`；普通变量和资源绑定以 `wrangler.toml` 为类型真源，
  Secret 只在源码声明名称，不写入配置或生成物。
