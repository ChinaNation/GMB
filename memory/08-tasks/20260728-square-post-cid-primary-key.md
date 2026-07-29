# 任务卡：广场用户业务以 CID 为唯一主键

## 状态

- 已完成
- 用户已于 2026-07-28 确认方案并完成 runtime 二次确认。

## 任务目标

彻底将公民广场的帖子归属、平台会员订阅、自动续费、创作者身份和创作者套餐改为以
`cid_number` 为唯一稳定用户主键。`account_id` 只保留为当前链上签名、实际扣款、
实际收款或不可变交易审计证据。

## 已确认业务边界

- 公民用户身份域使用 `cid_number`：
  - 广场帖子归属与发布数量；
  - 平台会员订阅与续费调度；
  - 创作者身份、套餐和订阅关系。
- 清算账户域不属于本任务：
  - 禁止修改 `UserBank`、`DepositBalance`、`L3PaymentNonce`；
  - 禁止修改 `PaymentIntent`、清算节点、清算 RPC；
  - 禁止修改 `QR_V1 k=4 user_transfer`、扫码支付页面和 CitizenWallet；
  - 公民 CID 换绑不得迁移开户行、清算余额或支付 nonce。

## 目标状态

### 帖子

- `SquarePost.cid_number` 必填并作为用户归属。
- `signer_account_id` 仅记录发布交易实际签名账户。
- 发布数量按 `cid_number` 统计。
- active 匿名 CID 可以发布普通帖子。
- Campaign 继续额外要求有效投票身份。

### 订阅

- `Subscriptions`、`RenewalSchedule`、`RenewalIndex` 的用户维度全部使用
  `subscriber_cid_number`。
- 首次扣款从本次 signed origin 的当前账户扣除。
- 自动续费必须通过 `subscriber_cid_number` 解析并复核当前双向绑定账户。
- 当前绑定不存在、CID inactive 或双向绑定不一致时 fail-closed，禁止扣历史账户。

### 创作者

- `IssuerKey::Creator` 和 `CreatorPlans` 使用 `creator_cid_number`。
- 创作者收款账户每次扣款时由 `creator_cid_number` 解析当前绑定账户。
- 自订阅按 CID 比较，禁止通过换绑账户绕过。

## 禁止事项

- 禁止 storage migration、旧 storage key 读取和双轨兼容。
- 禁止保留 `IssuerKey::Creator(AccountId)` 旧分支。
- 禁止增加协议版本。
- 禁止修改现有 UI 布局。
- 禁止触碰清算行和 CitizenWallet。
- 开发期使用重新创世后的新 storage。

## 预计修改目录

- `citizenchain/runtime/misc/square-post/`
  - 广场链上 storage、调用、事件、续费、benchmark、权重、测试和 fixture。
- `citizenchain/runtime/src/`
  - SquarePost 身份适配，只复用现有 CitizenIdentity 双向绑定和 active 状态。
- `citizenapp/lib/`
  - 订阅调用、创作者套餐和 storage key 改用 CID，不改变 UI。
- `citizenapp/test/`
  - 更新 Dart 编码、storage key、创作者和发布测试。
- `citizenapp/cloudflare/src/`
  - 链上订阅读取、确认和 reconcile 改用 CID；D1 CID 主键保持不变。
- `citizenapp/cloudflare/test/`
  - 更新链订阅、会员和创作者对账测试。
- `memory/`
  - 更新统一协议、架构、ADR、本任务卡和残留说明。

## runtime 二次确认文件

- `citizenchain/runtime/misc/square-post/src/lib.rs`
- `citizenchain/runtime/misc/square-post/src/subscription.rs`
- `citizenchain/runtime/misc/square-post/src/billing.rs`
- `citizenchain/runtime/misc/square-post/src/benchmarking.rs`
- `citizenchain/runtime/misc/square-post/src/weights.rs`
- `citizenchain/runtime/misc/square-post/src/tests/mod.rs`
- `citizenchain/runtime/misc/square-post/tests/fixtures/subscription_scale_vectors.json`
- `citizenchain/runtime/src/configs.rs`

### P0 续费时序修复二次确认（2026-07-29）

用户另行确认以下 4 个 Runtime 路径，用于把自动续费移出 finalize 后阶段并补齐隔离测试：

- `citizenchain/runtime/misc/square-post/src/lib.rs`
- `citizenchain/runtime/misc/square-post/src/weights.rs`
- `citizenchain/runtime/misc/square-post/src/tests/mod.rs`
- `citizenchain/runtime/src/tests/cases.rs`

NodeGuard 只新增“initialize 阶段零和业务转账不得误判为 finalize 发行”的隔离测试，
不放宽生产校验。清算行继续属于明确禁止修改范围。

## 验收标准

- 未绑定 CID 不得发帖或订阅。
- active 匿名 CID 可以发布普通帖子，但不能发布 Campaign。
- 帖子归属和累计数量按 CID。
- 账户 A 订阅后 CID 换绑到 B，订阅关系不变，续费只扣 B。
- 创作者换绑后套餐不丢，续费收入进入创作者当前账户。
- 同一 CID 在不同历史账户下仍不得自订阅。
- CID inactive、绑定缺失或双向绑定不一致时续费 fail-closed。
- Rust、Dart、TypeScript 的调用、storage key 和 SCALE fixture 对齐。
- 编译、单元测试、静态检查通过。
- 全新本地链完成发帖、订阅、换绑、续费与余额真实验收。
- 文档、中文注释和旧 AccountId 用户主键残留同步清理。

## 执行记录

- 2026-07-28：完成只读审计、业务边界纠正、方案确认和 runtime 二次确认。
- 2026-07-29：SquarePost 帖子、订阅、续费索引、创作者套餐及跨端读写已彻底收敛为
  CID 用户主键；无 storage migration、旧键读取或双轨兼容。
- 2026-07-29：隔离双节点初验定位 P0：自动续费在 finalize 后改变账户余额时，
  NodeGuard 会按安全设计拒绝候选区块。按二次确认把续费改为 `on_initialize` 使用上一块
  已确认时间，正常出块最多延后一个区块且不提前扣款；生产 NodeGuard 规则保持不变。
- 2026-07-29：代码级验收已通过：SquarePost 27/27、runtime 51/51、NodeGuard finalize
  24/24、Worker 193/193、CitizenApp 926 通过/5 跳过；runtime benchmark feature、
  Rust 格式、TypeScript typecheck 和 Flutter analyze 均通过。
- 2026-07-29：全新导出的 `citizenchain-fresh` 规范创建两个独立 base path 节点，
  第一节点真实 PoW 出块、第二节点零挖矿同步；完成两个匿名 CID 占号、普通帖子发布、
  匿名 Campaign 拒绝、三条订阅、创作者套餐及 Alice→Bob、Charlie→Dave 换绑。
  换绑后两节点共同最佳块 #13 为
  `0x1fc18383c190ba429da0310be452c8586930e2ac70bccae9bf0fc87ceacd9e80`，
  订阅与套餐仍以 CID 存在，绑定分别为 Bob 和 Dave。
- 2026-07-29：同一链数据库以 `libfaketime +40d` 重启。#14
  `0xeadd71d37d62658bb71ee5f9a541f6f55d40c883612b304b23567ea6d207397b`
  只写入新共识时间，续费事件为 0；#15
  `0x09d654ea86aed7aa316879860d0d9a64ab7d0951ad034eb6f4d6e84dedacc3df`
  在 `on_initialize` 产生 3 条 `SubscriptionCharged` 并被 NodeGuard 接受，第二节点同步
  到同一哈希。Bob 支付平台 199900 + 创作者 10000，Dave 支付平台 199900 并收到
  创作者款 10000；含 Bob 两笔发帖费后余额变化为 Alice 0、Bob -209920、
  Charlie 0、Dave -189900。三个 `paid_until` 均推进，CID 绑定、创作者套餐不变，
  发帖计数按订阅者 CID 从 1 增至 3。
- 2026-07-29：该 fresh spec 的 GRANDPA finalized head 在本地单验证者测试配置中未越过
  创世块，故 P0 NodeGuard 验收使用“两节点共同最佳 PoW 区块哈希 + 同一状态读取”证明
  候选块被完整校验并导入，不虚报 finalized。临时链目录已移入废纸篓，可恢复；
  验收专用 `libfaketime` 与其 `coreutils` 依赖已卸载。
