# 任务卡：wuminapp-wallet-tx-records

- 任务编号：20260518-121925
- 状态：done
- 所属模块：wuminapp
- 当前负责人：Codex
- 创建时间：2026-05-18 12:19:25

## 任务需求

让 wuminapp 钱包交易记录按本机钱包进入 App 后的链上余额变化记录：不追溯历史全链，只维护本地 Isar 流水、同步游标、收入监听和删除清理，并同步更新文档、中文注释与残留清理。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- wuminapp/WUMINAPP_TECHNICAL.md

## 模块模板

- 模板来源：memory/08-tasks/templates/wuminapp.md

### 默认改动范围

- `wuminapp`

### 先沟通条件

- 修改 Isar 数据结构
- 修改认证流程
- 修改关键交互路径


## 模块执行清单

- 清单来源：memory/07-ai/module-checklists/wuminapp.md

# WuMinApp 模块执行清单

- App 只是交互入口，不承担信任根职责
- Isar 结构、认证流程、关键交互变化前必须先沟通
- 关键 Flutter 交互与本地存储逻辑必须补中文注释
- 文档与残留必须一起收口


## 模块级完成标准

- 标准来源：memory/07-ai/module-definition-of-done/wuminapp.md

# WuMinApp 完成标准

- App 仍然只是交互入口
- 关键 Flutter 交互和 Isar 逻辑已补中文注释
- 文档已同步更新
- 关键交互或数据结构变化已先沟通
- 残留已清理


## 必须遵守

- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通
- 改代码后必须更新文档和清理残留

## 输出物

- 代码
- 中文注释
- 文档更新
- 残留清理

## 待确认问题

- 暂无

## 执行范围

- `wuminapp/lib/isar/`：调整本地交易记录字段，新增钱包交易同步游标；只处理本机交易流水存储，不改变链上状态规则。
- `wuminapp/lib/transaction/shared/`：统一本地交易流水写入、去重、pending 合并和增量同步；不承担钱包密钥、签名或治理投票职责。
- `wuminapp/lib/rpc/`：复用轻节点能力读取 finalized 区块事件并解析余额变化；不引入历史全链扫描和中心化交易记录服务。
- `wuminapp/lib/wallet/`：钱包创建、导入、删除时维护本机记录起点；交易记录列表和详情展示余额变化流水。
- `memory/01-architecture/wuminapp/` 与 `memory/05-modules/wuminapp/`：同步记录交易流水字段、同步策略、Isar 边界和轻节点负载边界。

## 字段口径

- 不单独保存 `direction`；方向由带正负号的 `amountDeltaFen` 推导。
- `type` 只保存业务类型，例如 `transfer / fee / reward / interest / issuance / burn / duoqian_transfer`。
- 钱包账户唯一性使用 `walletAddress` / `walletPubkeyHex`。
- 单条流水唯一性使用 `recordKey`；confirmed 记录按 `walletPubkeyHex:blockHash:eventIndex` 去重，pending 记录按 `walletPubkeyHex:pending:txHash` 去重。
- 不追溯导入前历史，只从钱包新建或导入本机后的 finalized 区块开始记录。

## 实施记录

- 任务卡已创建
- 已确认字段口径和执行范围。
- 已调整 `LocalTxEntity` 为签名余额变化模型：以 `amountDeltaFen` 表示增加/减少，以 `type` 表示业务类型，以 `recordKey` 表示单条流水唯一性。
- 已新增 `WalletTxSyncCursorEntity`：每个钱包按 `walletPubkeyHex` 保存本机开始跟踪区块和最新同步区块。
- 已实现 `ChainTxMonitor` finalized 区块事件监听：只读取本机开始跟踪后的 `System.Events`，解析 `Balances::Transfer`，不补扫导入前历史。
- 已调整本机提交转账记录：提交成功后写 pending 记录，confirmed 链上事件命中后合并 pending，避免重复显示。
- 已删除旧 `PendingTxReconciler` nonce 轮询确认链路，避免本机 pending 先被 nonce 推成 confirmed 后又被 finalized 事件重复写入，也减少对节点的额外 RPC 请求。
- 已删除 `OnchainRpc` 中无调用的旧 nonce 确认 API，普通转账流水确认统一等待 finalized 事件。
- 已调整钱包创建、导入、删除流程：创建/导入初始化本机同步起点；删除钱包同步删除本地流水和游标；再次导入重新记录。
- 已调整钱包详情最近交易、完整列表和详情页展示字段：列表显示业务类型、带正负号余额变化、对方地址、时间和状态；详情显示金额、手续费、from/to、txHash、区块/事件定位和失败原因。
- 已补中文注释：重点说明钱包身份唯一性、流水唯一性、pending/confirmed 合并和后台同步让路规则。
- 已更新技术文档：`memory/01-architecture/wuminapp/WUMINAPP_TECHNICAL.md`、`memory/05-modules/wuminapp/wallet/WALLET_TECHNICAL.md`、`memory/05-modules/wuminapp/transaction/onchain-transaction/ONCHAIN_TECHNICAL.md`、`memory/05-modules/wuminapp/rpc/RPC_TECHNICAL.md`。
- 已执行残留扫描，旧字段/旧查询接口在本次改动路径中已清理；保留的 `direction` 字样仅用于文档/注释说明“不再保存方向字段”。

## 验证记录

- `flutter pub run build_runner build --delete-conflicting-outputs`：通过；仅有 analyzer 版本低于 SDK 的提示，不影响生成。
- `dart format`：通过。
- `flutter analyze lib test`：通过。
- `flutter test`：通过，`All tests passed!`。

## 完成信息

- 完成时间：2026-05-18 12:33:10
- 完成摘要：完成 wuminapp 钱包本机交易流水：签名余额变化字段、finalized 事件增量同步、pending 合并、删除清理、列表详情展示和技术文档同步
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
