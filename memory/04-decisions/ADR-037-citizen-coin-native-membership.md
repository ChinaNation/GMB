# ADR-037 公民币原生平台订阅与创作者订阅

- 状态：Accepted
- 初始日期：2026-07-16
- 统一修订：2026-07-18
- 关联：ADR-033、ADR-036、ADR-038、ADR-018、`20260716-citizen-coin-subscription`

## 1. 决策

平台订阅与创作者订阅统一使用 CitizenChain 原生公民币付款，订阅能力并入现有 `SquarePost` pallet。2026-07-21 已确认当前正式链尚未创世，直接使用终态 storage 和 `StorageVersion = 0`，不执行开发期原地迁移、不保留旧状态或兼容双轨；正式创世后的真实升级才另行设计迁移。

订阅周期统一由真实公历决定。每个区块唯一的 `Timestamp.Now` 是 runtime 判断到期和执行扣款的时间依据；runtime 以确定性的 UTC 整数公历算法计算月、季、年，订阅期限与区块高度、固定天数和设备时间无关。

## 2. 平台与创作者付款

- 平台三档为 Freedom、Democracy、Spark，价格真源为链上 `PlatformPrice`。
- 平台收款账户由创世常量公民链基金会 CID 派生；缺失时 fail-closed。
- 创作者必须拥有当前有效的平台订阅。
- 创作者链上套餐只保存付款必需的 `tier_id`、`billing_period`、`price_fen`。
- 名称、说明、权益文案和媒体只保存在 Cloudflare/D1。
- 每次真实扣款读取当时的链上当前价格；当前已付周期不补差价。
- 创作者订阅款按 `creator_cid_number` 解析当前双向绑定账户并全额进入该账户。

## 3. runtime 自动周期流程

1. 用户在 CitizenApp 对订阅签名；runtime 立即按最新链上价格扣款。
2. runtime 以当前区块共识时间戳记录首次扣款，并计算下一个真实公历到期时间。
3. Active 订阅进入有界到期索引；每个区块在 Timestamp inherent 写入后处理已到期项目。
4. 到期时无需再次签名，runtime 由 `subscriber_cid_number` 解析当前双向绑定账户，
   从当前账户向平台费用账户或创作者当前绑定账户转账，并读取最新链上价格。
5. 停链期间不能写状态；恢复出块后按到期顺序补扣全部已到期周期，直到追上当前共识时间或余额失败。
6. 余额不足写 `Suspended(InsufficientBalance)`；CID 当前绑定不可用写
   `Suspended(IdentityBindingUnavailable)`；目标套餐失效写
   `Suspended(NeedReconsent)`；均禁止回退扣历史账户。

CitizenApp 只负责订阅、取消、换套餐签名和链上时间戳展示。设备、App 和 Cloudflare 是否在线均不影响自动续费。

## 4. 状态与换套餐

- `Active`：当前链时间戳早于 `paid_until` 时权益有效。
- `Cancelled`：停止后续续费，已付权益保留至 `paid_until`。
- `Suspended`：余额不足、改价待再授权或当前身份绑定不可用，停止自动续扣。
- `CreatorPaused`：创作者平台会员暂停，保留调度等待恢复。
- `Terminated`：仅用于不可恢复的明确终止。
- 换套餐立即按剩余权益折算，不存在延迟生效套餐字段。
- 已取消后换套餐作为新的签名授权，立即按目标计划当前价格处理。
- 不退款、不补差价、不按日折算。

## 5. Call index

`SquarePost` pallet index 固定为 `34 / 0x22`。

| index | 调用 |
|---:|---|
| `0` | `publish_post` |
| `1` | `subscribe` |
| `2` | `cancel` |
| `3` | `set_creator_plans` |
| `4` | `change_subscription_plan` |
| `5` | `propose_set_platform_price` |
旧 keeper、外部续费调用、周期确认调用和旧 SCALE 布局全部废弃，不保留兼容入口。

## 6. 真源与资源边界

- 平台价格、创作者付款套餐、扣款事实、订阅状态和权益截止时间：CitizenChain。
- 真实公历计算、到期调度与自动扣款：CitizenChain runtime。
- 订阅、取消、换套餐签名和日期展示：CitizenApp。
- 创作者展示资料：Cloudflare/D1。
- D1 订阅记录：finalized 链状态的可重建镜像。
- finalized 镜像证明：交易哈希、区块哈希和完整已签名 extrinsic；Worker 必须复核签名钱包、调用参数、finalized 主链包含关系与同一区块 storage。
- 平台调价：统一投票引擎。
- 平台调价入口与提交：OnChina 读取准确机构 CID 和 finalized 链上真源，CitizenWallet 只签名一次并显示响应二维码，OnChina 回扫后通过唯一链提交入口广播。
- 平台订阅、创作者身份、套餐和订阅关系的链上及 D1 业务键全部是 CID；
  `account_id` 只表示当前交易签名者、实际付款/收款账户或不可变审计事实。

Cloudflare 只承担低频镜像、展示与门禁加速，不保存第二份未来扣款价格，不计算日期，不持有扣款能力，不进行高频全链扫描。镜像中的最近扣款价格只用于审计和用量预算，不决定下一次扣款。

## 7. 安全与信任边界

- 订阅者签名订阅即建立持续自动扣款授权，直到该订阅者签名取消。
- 续费、周期推进、换档生效和失败处理均由 runtime 执行，不接受任何外部账户提交。
- 换绑不迁移订阅 storage：CID 键天然不变；续费只解析当前绑定账户，旧账户永不再扣。
- UTC 公历算法只使用确定性整数运算，所有节点对同一时间戳得出相同日期。
- 到期索引按时间顺序处理并设置单块权重上限；积压在后续区块继续，不能静默跳过。
- Cloudflare 只镜像 finalized 状态，无法延长权益或触发扣款。
- 同一 finalized 交易只能首次绑定一个钱包、动作和规范化请求；完全相同的重试幂等，改写请求内容 fail-closed。
- Cloudflare 门禁只接受未陈旧的 finalized 链时钟；`Active` 与未到期的 `Cancelled` 可用，`Terminated`、过期、未知或陈旧镜像拒绝。
- OnChina 平台模块只授权链上 `PlatformCidNumber` 对应的准确机构实例；机构码、前端显示和本地数据库都不能代替 CID 与链上 `admins` 真源。
- OnChina 的请求二维码、CitizenWallet 一次签名响应二维码和 OnChina 回扫提交是所有管理员链交易的统一流程；禁止调价业务自建直接钱包提交、第二次签名或第二套提交接口。

## 8. 创世决策

当前正式链尚未创世，SquarePost 直接以 CID 终态键重新创世：

1. 不提供 storage migration、旧键读取、旧枚举或双轨兼容。
2. `Subscriptions`、续费索引、`CreatorPlans` 和帖子发布计数全部直接使用 CID。
3. Worker D1 与 KV 随创世部署同批重建，镜像由新链 finalized 状态产生。
4. 真实验收必须覆盖订阅者和创作者换绑，证明关系不变且只使用当前账户付款/收款。

## 9. 后果

- 订阅期限完全脱离区块高度，页面可显示真实日期与时间。
- App 离线不影响续费；停链期间到期的周期在恢复出块后补扣。
- runtime 承担有界调度和公历计算成本，Cloudflare 不承担扣款或日期计算。
- 首次订阅只有一笔签名交易；续费没有用户交易，也没有周期确认交易。
- 第三步只接入订阅、取消、换套餐和 finalized 状态展示。
- 第四步只做 finalized 证明镜像、到期候选对账和 Cloudflare 资源门禁，不增加账户签名、设备签名或链上交易。
- 第五步只增加 OnChina 平台调价入口、准确 CID 工作台隔离和 CitizenWallet 严格识别；不修改 runtime，不在业务模块实现投票，也不增加第二次签名。
- 所有端必须同步新 SCALE、call、storage 和状态，不允许单端兼容旧协议。
