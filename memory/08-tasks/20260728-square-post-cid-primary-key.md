# 任务卡：广场用户业务以 CID 为唯一主键

## 状态

- 执行中
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
