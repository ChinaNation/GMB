# 卡3 · CitizenApp 大频道/公共广播（复用广场基础设施，非 E2E，订阅 10万+）

任务需求：新增**大频道/公共广播**：一个频道由发布者（admin）向大量订阅者（10万+）单向广播帖子（文字/图片/视频）；**不强制 E2E**（内容服务端存储，与广场帖子同性质）；复用现有广场（Square）R2/Images/Stream/feeds 基础设施。与卡2（私密小群 E2E）双轨互不干扰。
所属模块：citizenapp / cloudflare（广场 posts/feeds/media/membership）+ App 频道 UI

## 定稿方向（承接 roadmap 决策，见 [[project_chat_media_group_roadmap_2026_07_15]]）

- 10万人+E2E+零存储三约束互斥；**10万+ 归公共频道/广播，复用广场 R2/Images/Stream，不强制 E2E**（Telegram 10万级公共频道亦非 E2E）。
- 进开发前先**厘清与广场边界**（本卡第一步，见下）。

## 现状事实（已核实，非推断）

- **广场帖子/流已成体系**：`cloudflare/src/posts/repository.ts` 有 `FeedKind`（campaign/following/…）；`feeds/follows.ts` 有 `square_follows`（关注/取关，`INSERT INTO square_follows`）；`feeds/service.ts`+`browse.ts` 分流。→ 频道≈**新 FeedKind + 频道表 + 订阅表**（订阅仿 follow）。
- **媒体基础设施现成**：`media/signed_urls.ts`（Images/Stream 签名 URL）、`uploads/`（R2 配额 `quota.ts`）、`membership/archive.ts`（退订视频 Stream→R2 冷归档）。→ 频道媒体**直接复用**,非 E2E 存储即广场既有路径。
- **会员四档门禁现成**：`membership/plans.ts`（freedom/democracy/voting/candidate，发帖额度按套餐 `membershipPlan(level).quota`）。→ 频道**发布**用会员/身份档门禁；订阅可放宽。
- 存储=Cloudflare D1（`env.DB`）+ R2 + Images + Stream。非零存储（本卡本就存内容）。

## 与广场边界厘清（本卡第一步，需拍板）

| 维度 | 广场(现有) | 频道(卡3) |
|---|---|---|
| 拓扑 | 多对多公共场 | 一对多广播(admin 发, 订阅者收) |
| 发布权 | 达标会员皆可发 | 仅频道 admin |
| 关系 | follow 账户 | subscribe 频道 |
| 分发 | 关注流/推荐 | 频道流 + 新帖推送 |
| 存储 | 广场 posts | **复用同表, 加 channel_id 维度** |

→ **推荐**：频道=广场之上的**广播 FeedKind + 频道实体**，复用 posts/media/uploads 存储与配额，新增频道表+订阅表+频道流+发布权限。不另起独立存储栈。起卡时确认此边界口径。

## 架构

```
Admin 发布(App)                Cloudflare(广场基础设施)              订阅者(10万+)
publishChannelPost
  └ 上传媒体(R2/Images/Stream, 复用 uploads/media 签名URL)
  └ INSERT square_posts(channel_id=…)  ── D1 ──▶ 频道流(分页, 边缘缓存)
                                                       └ 订阅者拉取(SWR/CDN 缓存)
  └ 新帖 → 批量推送(sendWake/push, 分批扇出)  ──────▶ 订阅者收通知
```

- **读扩展问题非加密问题**：10万 订阅=读放大,靠**边缘缓存频道流**(频道流公开/半公开,可 Cache API/KV 缓存分页)+ **推送分批**,不是每人一份密文。
- **发布权限**：仅频道 admin;会员/身份档 + 频道角色双门禁。
- **订阅**：`square_channel_subscriptions`(仿 square_follows);计数用计数器(避免 COUNT 全表)。

## 目录结构（新增/改）

```
cloudflare/src/channels/            # 新频道模块
  repository.ts                     # 频道 CRUD + square_channels / square_channel_subscriptions
  publish.ts                        # admin 发帖到频道(校验频道角色 + 会员额度, 复用 uploads/media)
  feed.ts                           # 频道流分页读(边缘缓存, SWR)
  subscribe.ts                      # 订阅/退订(仿 follows) + 订阅计数
  push.ts                           # 新帖批量推送订阅者(复用 chat/push 或 posts 推送, 分批)
cloudflare/src/posts/repository.ts  # FeedKind 加 'channel';posts 加 channel_id 维度(可空=广场原帖)
cloudflare 迁移                     # square_channels / square_channel_subscriptions 表(D1);square_posts 加 channel_id 列
lib/channel/                        # App 频道模块
  channel_model.dart                # Channel(id/name/owner/adminSet/subscriberCount/visibility)
  channel_api.dart                  # BFF 客户端(发布/订阅/拉流)
  views/
    channel_list_page.dart          # 频道列表/发现
    channel_detail_page.dart        # 频道流(复用广场帖子卡渲染)
    channel_publish_page.dart       # admin 发帖(复用广场发帖 UI + 媒体上传)
```

## 分阶段实现（每阶段先出细化方案确认后执行）

- **阶段0 · 边界确认**：定频道=广播 FeedKind 复用广场存储（vs 独立栈）；定发布权限档位、订阅是否需登录/会员、频道可见性（公开/半公开）。
- **阶段1 · 频道模型 + 发布 + 拉流（服务端）**：`channels/` 模块 + D1 迁移（频道表/订阅表 + posts.channel_id）；admin 发帖（复用 uploads/media/配额）；频道流分页读 + 边缘缓存；订阅/退订 + 订阅计数。
- **阶段2 · App 频道 UI**：频道列表/详情/发布页（复用广场帖子卡与发帖 UI）；订阅按钮 + 订阅态。
- **阶段3 · 新帖推送（10万 扇出）**：新帖批量推送订阅者（分批 + 限流 + 复用 push infra）；读扩展压测（边缘缓存命中率、分页游标稳定）。

## 必须遵守

- **非 E2E、内容服务端存储**（与广场帖子同性质，明确区别于卡2 的 E2E 群）——本卡不承诺零存储。
- **复用不重造**：媒体走广场 uploads/media/R2/Images/Stream 与配额,不新起媒体栈;发帖 UI 复用广场帖子组件。
- 发布权限=频道 admin + 会员/身份档;**不放宽发布**（防滥发广播）。
- 10万 分发靠缓存+推送分批,**禁每订阅者一份写**、禁 COUNT 全表(用计数器)。
- 不动卡2 的 E2E 群路径;不动 citizenchain。

## 验收标准

- 建频道、admin 发文字/图片/视频帖,订阅者拉频道流正确分页显示;媒体经广场签名 URL 正常加载。
- 订阅/退订生效,订阅计数正确(计数器,非全表 COUNT)。
- 新帖推送到订阅者(分批,含大订阅量的限流验证)。
- 非 admin 发布被拒;未达档位发布被拒。
- 频道流边缘缓存命中、分页游标在并发发帖下稳定。
- 服务端单测(频道 CRUD/发布权限/订阅/流分页/推送分批)+ App 组件测;对抗式审查每阶段跑。

## 风险

- **读扩展(10万 订阅)**：频道流热点;缓存失效风暴、分页游标漂移需设计(游标基于稳定排序键)。
- **推送扇出成本**：10万 推送分批+限流+失败重试;避免瞬时打爆 push provider。
- **与广场数据耦合**：posts 加 channel_id 后,广场原有查询须排除频道帖(或反之),边界不清会串流——迁移+查询审计要严。
- **滥发/审核**：广播放大恶意内容影响;复用 `moderation/` 审核 + 发布额度。

## 关联

[[project_chat_media_group_roadmap_2026_07_15]] 双轨决策 · [[project_public_institution_feature_2026_06_13]] 广场 infra · [[project_membership_visitor_two_tier_exact_match]] 会员四档 · 卡2=私密小群 `20260715-citizenapp-chat-group-private-e2e.md`。
