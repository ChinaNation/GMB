# ADR-037 公民币原生平台订阅与创作者订阅

- 状态：Accepted
- 初始日期：2026-07-16
- 统一修订：2026-07-18
- 关联：ADR-033、ADR-036、ADR-038、ADR-018、`20260716-citizen-coin-subscription`

## 1. 决策

平台订阅与创作者订阅统一使用 CitizenChain 原生公民币付款，订阅能力并入现有 `SquarePost` pallet。现有链及全部无关状态通过 StorageVersion 原地升级保留，禁止重新创世、替换 chainspec、清空链状态或建立兼容双轨。

订阅周期统一由真实公历决定。每个区块唯一的 `Timestamp.Now` 是 runtime 判断到期和执行扣款的时间依据；runtime 以确定性的 UTC 整数公历算法计算月、季、年，订阅期限与区块高度、固定天数和设备时间无关。

## 2. 平台与创作者付款

- 平台三档为 Freedom、Democracy、Spark，价格真源为链上 `PlatformPrice`。
- 平台收款账户由真实 `PlatformCidNumber` 派生；缺失时 fail-closed。
- 创作者必须拥有当前有效的平台订阅。
- 创作者链上套餐只保存付款必需的 `tier_id`、`billing_period`、`price_fen`。
- 名称、说明、权益文案和媒体只保存在 Cloudflare/D1。
- 每次真实扣款读取当时的链上当前价格；当前已付周期不补差价。
- 创作者订阅款全额进入创作者钱包。

## 3. runtime 自动周期流程

1. 用户在 CitizenApp 对订阅签名；runtime 立即按最新链上价格扣款。
2. runtime 以当前区块共识时间戳记录首次扣款，并计算下一个真实公历到期时间。
3. Active 订阅进入有界到期索引；每个区块在 Timestamp inherent 写入后处理已到期项目。
4. 到期时无需再次签名，runtime 从订阅者钱包向平台费用账户或创作者钱包转账，并读取最新链上价格。
5. 停链期间不能写状态；恢复出块后按到期顺序补扣全部已到期周期，直到追上当前共识时间或余额失败。
6. 余额不足或目标套餐失效时立即写 `Terminated`，移除调度且不重试。

CitizenApp 只负责订阅、取消、换套餐签名和链上时间戳展示。设备、App 和 Cloudflare 是否在线均不影响自动续费。

## 4. 状态与换套餐

- `Active`：当前链时间戳早于 `paid_until` 时权益有效。
- `Cancelled`：停止后续续费，已付权益保留至 `paid_until`。
- `Terminated`：自动扣款失败或套餐失效，停止续费且不重试。
- 未到期换套餐写 `pending_plan`；下一次真实续费时使用目标计划当前价格。
- 已取消或已终止后换套餐作为新的签名授权，立即按目标计划当前价格扣款并重新进入 Active。
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

Cloudflare 只承担低频镜像、展示与门禁加速，不保存第二份未来扣款价格，不计算日期，不持有扣款能力，不进行高频全链扫描。镜像中的最近扣款价格只用于审计和用量预算，不决定下一次扣款。

## 7. 安全与信任边界

- 订阅者签名订阅即建立持续自动扣款授权，直到该订阅者签名取消。
- 续费、周期推进、换档生效和失败终止均由 runtime 执行，不接受任何外部账户提交。
- UTC 公历算法只使用确定性整数运算，所有节点对同一时间戳得出相同日期。
- 到期索引按时间顺序处理并设置单块权重上限；积压在后续区块继续，不能静默跳过。
- Cloudflare 只镜像 finalized 状态，无法延长权益或触发扣款。
- 同一 finalized 交易只能首次绑定一个钱包、动作和规范化请求；完全相同的重试幂等，改写请求内容 fail-closed。
- Cloudflare 门禁只接受未陈旧的 finalized 链时钟；`Active` 与未到期的 `Cancelled` 可用，`Terminated`、过期、未知或陈旧镜像拒绝。
- OnChina 平台模块只授权链上 `PlatformCidNumber` 对应的准确机构实例；机构码、前端显示和本地数据库都不能代替 CID 与链上 `admins` 真源。
- OnChina 的请求二维码、CitizenWallet 一次签名响应二维码和 OnChina 回扫提交是所有管理员链交易的统一流程；禁止调价业务自建直接钱包提交、第二次签名或第二套提交接口。

## 8. 升级决策

正式链升级必须：

1. 在真实链数据库副本确认 SquarePost 旧 StorageVersion。
2. 断言旧订阅、退役到期索引和相关前缀为空。
3. 发现任何旧订阅数据时写阻断标志并停止启用新协议，不转换、不删除。
4. 仅在安全前提满足时精确清理退役 keeper 单值。
5. 平台价格只做“缺失回填、已有保留”。
6. 保留帖子、发布计数、账户、余额、总发行量、平台 CID、身份、机构、治理和其他无关状态。
7. 在链数据库副本完成 TryRuntime pre/post 验收后，才可进入正式升级流程。

## 9. 后果

- 订阅期限完全脱离区块高度，页面可显示真实日期与时间。
- App 离线不影响续费；停链期间到期的周期在恢复出块后补扣。
- runtime 承担有界调度和公历计算成本，Cloudflare 不承担扣款或日期计算。
- 首次订阅只有一笔签名交易；续费没有用户交易，也没有周期确认交易。
- 第三步只接入订阅、取消、换套餐和 finalized 状态展示。
- 第四步只做 finalized 证明镜像、到期候选对账和 Cloudflare 资源门禁，不增加账户签名、设备签名或链上交易。
- 第五步只增加 OnChina 平台调价入口、准确 CID 工作台隔离和 CitizenWallet 严格识别；不修改 runtime，不在业务模块实现投票，也不增加第二次签名。
- 所有端必须同步新 SCALE、call、storage 和状态，不允许单端兼容旧协议。
