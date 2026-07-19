# 任务卡：CitizenApp 充值与 CitizenConsole 发币端到端验收

> 状态：执行中

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

- 只允许本地或测试环境，不对生产网络、生产稳定币或生产公民币执行真实发币。
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
| USDC Base testnet 真实付款→发币 | **BLOCKED** | staging vars、D1 迁移和结算 Secret 未就绪 |
| USDT Base testnet 真实付款→发币 | **BLOCKED** | 同上，且 Base Sepolia 的 mock USDT 合约未进入 staging vars |
| CitizenConsole 真实发币 | **BLOCKED** | 当前控制台指向 production Worker；未获生产试单授权且不得使用真实资产测试 |

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

- **不能判定整个业务完全跑通。** 静态检查、CitizenApp/Worker 单测和 CitizenConsole 页面通过，但 CitizenConsole 资金安全测试失败，staging 完全未就绪，两个币轨都没有完成真机 testnet 付款→Worker→CitizenConsole→公民链到账的真实闭环。
- 在 H1、H2、H3 修复并完成 staging 真机验收前，不建议开放 production 充值。
