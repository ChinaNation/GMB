# 任务卡：Cloudflare 创作者会员 BFF（步骤 1）

> 状态：**完成并通过类型检查**（2026-07-17）。`cd citizenapp/cloudflare && npx tsc --noEmit` exit 0。落地 App 创作者页 + 订阅按钮依赖的全部端点。创作者档全链下（Cloudflare 存），链上是订阅关系真源。

## 需求
- 创作者档位定义（名称/档种类≤10/月季年价）读写：写入复用现有广场账户动作统一签名 `OP_SIGN_SQUARE_ACTION`(0x1D)，**零新签名协议**。
- 概览：订阅人数 + **本月已收入**（当月真实扣款合计，**非摊算/预计**）+ 档位数。
- 订阅态（按钮双态）+ 订阅/取消上链后镜像（幂等）。
- 门禁 `requireCreatorSubscription`。金额一律**分**。

## as-built（已落地，tsc 绿）
```
migrations/0004_creator.sql          [新] square_creator_plans + square_creator_subscriptions
account/action_challenge.ts          [改] SignedAction 加 'set_creator_plan'（context=tiers 规范化 sha256）
membership/creator.ts                [新] 7 端点 handler + requireCreatorSubscription 门禁
routes.ts                            [改] 注册 7 路由（exact 在 prefix 前）
limits/catalog.ts                    [改] routeLimits 白名单加创作者路由（否则 assertKnownRoute 404）
```

## 端点（与 App `creator_api.dart`/`creator_subscribe_service` 逐字段一致）
| 方法·路由 | 行为 |
|---|---|
| `GET /v1/square/creator/plan` | 我的档位 → `{plan\|null}` |
| `GET /v1/square/creator/plan/:account` | 他人档位（订阅者选档）→ `{plan\|null}` |
| `GET /v1/square/creator/overview` | `{overview:{subscriber_count, month_income_fen, tier_count}}` |
| `GET /v1/square/creator/subscription/:account` | 我对该创作者订阅态 → `{status\|null}` |
| `POST /v1/square/creator/plan/challenge` | `{tiers[]}`→`issueActionChallenge('set_creator_plan', sha256(canonicalTiers))`→`{signing_payload_hex,challenge_id,expires_at}` |
| `POST /v1/square/creator/plan` | `{challenge_id,signature,tiers[]}`→`consumeActionSignature`(context=同一哈希)→覆盖写 `square_creator_plans` |
| `POST /v1/square/creator/subscription/confirm` | `{tx_hash,creator_account,tier_id?,period?}`→带 tier+period=订阅active/缺=取消cancelled，幂等镜像 |

## 关键不变量
- **零新签名协议**：设档走现有 0x1D action_challenge，仅加 `set_creator_plan` 动作 + `buildActionScalePayload` 复用（context=tiers 规范化 sha256 防替换）。
- **owner 由 session 派生**（`session.owner_account`），body 不携带 owner（防冒领）。
- `canonicalTiers`：档位按 tier_id 排序 + 只取正整数分周期价 + `JSON.stringify` → 挑战与保存两次哈希一致才放行。
- **本月已收入** = `SUM(price_fen)` where creator=owner 且 status='active' 且 `last_charged_at >= 当月UTC起点`（真实，非摊算）。校验/上限：≤10 档、档名非空、id 唯一、每档≥1 正价周期。

## 已知开口（硬化 TODO）
- `creatorSubscriptionConfirmRoute` 当前**信任 App 已完成的上链 tx**（subscriber 仍由 session 派生防冒领）；未来加**链读 `Subscriptions[(subscriber,Creator(creator))]` 核实**再镜像（chain/rpc.ts + storage_key.ts 已具备读链能力）。
- 本月已收入按 **UTC 月**；如需 UTC+8 对齐（护照/投票口径）后续调 `monthStartMs`。
- 单轨清 Stripe（prepaid/webhook 删）不在本卡范围（另事）。

## 验收
`npx tsc --noEmit` exit 0；7 路由注册 + 白名单齐；App `CreatorApiHttp`/`creator_subscribe_service` 从 `FakeCreatorApi` 切真 Worker 契约对齐（字段名/路径逐一核对一致）。

影响范围：`citizenapp/cloudflare`（新增 creator.ts + migration + 路由/白名单/action 扩展）。链端/ App 不在本卡。
