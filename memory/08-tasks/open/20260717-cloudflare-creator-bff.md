# 任务卡：Cloudflare 创作者会员 BFF（已并入统一订阅任务）

> 状态：已合并，不再作为独立执行入口
> 统一任务：`memory/08-tasks/open/20260716-citizen-coin-subscription.md`

本卡原实现使用链下创作者套餐真源、额外动作挑战签名和弱交易引用，均已由统一订阅任务第 4 步彻底替换。当前唯一目标状态如下：

- 创作者钱包账户是主键；档位付款字段真源为 finalized `CreatorPlans`，D1 只保存可重建镜像和展示名称。
- 创作者设置套餐只签名一次 `set_creator_plans` 链上交易；镜像请求只携带 Bearer 会话和 finalized 完整交易证明，不再生成账户或设备签名。
- 创作者订阅镜像使用复合主键 `(subscriber_account,creator_account)`，严格核对 signed extrinsic、finalized 主链包含关系与同一区块 `Subscriptions`。
- 创作者必须有未陈旧 finalized 平台订阅才能设置或公开展示档位；`Active` 和未到期 `Cancelled` 有效，`Terminated`、到期、缺失和陈旧镜像 fail-closed。
- 原独立迁移和旧挑战路由已删除，不保留兼容入口、旧表或待硬化开口。

最终代码、协议、测试与验收事实只以统一任务卡、`memory/01-architecture/gmb/subscription-part1-tech.md` 和 ADR-037 为准。
