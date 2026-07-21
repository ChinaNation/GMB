# 公民币订阅与税务架构索引

> 状态：原组合草案已拆分；本文件只保留导航，不再承载可执行方案
> 修订：2026-07-18

原文把平台订阅、创作者订阅、外部支付、收入台账和税务分期写在同一份预研方案中，包含已经废弃的双轨付款、弱交易镜像、旧目录和未确认税务设计。为避免它继续成为第二套实现真源，订阅部分已全部收敛到以下当前文件：

- 订阅技术架构：`memory/01-architecture/gmb/subscription-part1-tech.md`
- 订阅决策：`memory/04-decisions/ADR-037-citizen-coin-native-membership.md`
- 订阅执行任务：`memory/08-tasks/open/20260716-citizen-coin-subscription.md`
- 跨端协议：`memory/07-ai/unified-protocols.md` 的 P-TX-014 与 P-STORAGE-006

当前订阅边界固定为：

- 平台与创作者订阅统一使用链上公民币；CitizenChain 是价格、扣款、状态和真实公历到期时间的唯一真源。
- CitizenApp 只对订阅、取消、换套餐和创作者设置套餐分别签名一次；自动续费由 runtime 内部执行。
- Cloudflare 只保存 finalized 完整交易证明、可重建订阅镜像和创作者展示资料，不计算日期、不触发扣款、不保存未来扣款价格真源。
- OnChina 与 CitizenWallet 只承接公民链基金会平台调价提案的统一签名流程：OnChina 展示请求二维码，CitizenWallet 只签名一次并显示响应二维码，OnChina 回扫响应后提交；投票流程完全复用统一投票引擎。

税务仍属于独立业务范围，只能以 ADR-038 及其后续经用户确认的独立任务为准。不得从本文件恢复税务 pallet、收入台账、税率、目录、索引、回调或分期安排，也不得因为订阅已经实施而推断税务方案获得授权。
