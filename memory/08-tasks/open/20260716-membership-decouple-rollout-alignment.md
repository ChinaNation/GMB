# 任务卡：会员解耦全端全环境对齐部署方案（生产 + 测试）

- 建卡日期：2026-07-16
- 归属：CID/后端（Worker + D1 + Stripe）+ Mobile（CitizenApp 打包）+ 官网（citizenweb）
- 前置：[[20260716-citizenapp-membership-identity-decouple]]（代码已完工，三端全绿）已完成
- 关联：ADR-036；双轨卡2 `20260715-citizenapp-chat-group-private-e2e`（共享 cloudflare/src + wrangler.toml）
- 状态：**staging + production 已对齐完成（2026-07-16，用户「继续」授权）**；剩 CitizenApp 打包发版 + sandbox Stripe（另一账户）待你侧

## production 完成记录（2026-07-16）

- 建 `citizenapp-chat-relay` 生产 R2 空桶（解 CHAT_RELAY 阻塞）。
- `wrangler deploy --env production` 成功（Version `649cc068-7ec6-4e4f-a958-9842589ae59e`；`SPARK_PRICE_ID=price_1TrrNC…` LIVE、`RELAY_ENABLED=0`）。
- D1 外科式重建 `square_memberships`（生产会员表**0 行**，无损）；新 schema 已核验。
- **【重要发现 + 已修】生产 D1 基线漂移**：生产库缺 3 张 `0001` 定义的表——`square_contacts`、`square_stripe_payments`、`square_stripe_webhook_events`（staging 缺 `square_contacts`）。**缺 `square_stripe_webhook_events` 会使订阅 webhook 无法原子占位 → 会员在生产根本授权不了**（此前未暴露因 Stripe e2e 只在 sandbox 跑）。已用 `CREATE TABLE IF NOT EXISTS`（additive 无损）在**两库补齐**，现两库均 21 张 `0001` 用户表，与基线一致。
- 官网 `wrangler pages deploy dist --project-name citizenweb --branch main` → 部署 `7bf3188a`（Production/main）；`www.crcfrcn.com/membership` 浏览器实测渲染三档卡（自由≤10MB / 民主≤100MB / 薪火≤5GB、无身份字段），bundle `index-CDFWAFcJ.js` 与本地一致。

## 2026-07-16 用户三点追加

1. **删用户可见解耦文案**（用户强调：会员页只讲会员，身份页只讲身份）：官网描述删「与身份彻底解耦，任意身份都可订阅任意档」→「三档会员按月订阅…」；App 底部提示「左右滑动切换会员档 · 任意身份都可订阅任意档」→「左右滑动切换会员档」。官网已重构建部署上线并浏览器实测确认删除。
2. **官网会员页 UI 重设计**（`Membership.tsx` 重写）：三档会员卡各带**「银行卡订阅」+「加密货币预付」**两键，点击弹出统一操作弹窗（钱包地址+扫码签名；加密货币再选季/年，异档自动换档）；最右卡**仅取消订阅**。删原「订阅/取消 tab」「支付方式段选」「USDC 操作段选」「当前选择栏」。**USDC 文案统一改「加密货币」**（后续支持 USDT 等）。tsc/eslint/build 全绿 + 本地浏览器实测（两类弹窗、时长段选、金额、无控制台报错）+ 部署 `5de64011`，`www.crcfrcn.com` 实测 3×银行卡订阅 + 3×加密货币预付 + 取消卡、解耦文案已无、薪火会员在。**App 无需改**（会员页本就打开官网操作）。
3. **sandbox Stripe**：MCP 仍绑 LIVE `acct_1Trr2f`（`get_stripe_account_info` 复测），网页登录沙盒不改变 MCP 绑定 → 我无法用当前连接改沙盒。需用户在沙盒后台手动：$99.99「竞选公民会员」改名「薪火会员」+ 投票 $9.99 产品/价归档。功能非阻塞（价 ID 映射已对）。

## 剩余

- **CitizenApp 打包发版**：代码已完工+测试；`flutter build apk`/iOS 可跑通，签名/商店上架走你的发版流程（keystore/凭证）。要我先跑 `build apk` 冒烟吗？
- **sandbox Stripe**（`acct_1Trr2q…` 测试账户）：候选产品改名薪火、投票归档——本 MCP 未连该账户，需你授权或手动。功能非阻塞。

## staging 完成记录（2026-07-16）

- 建 `citizenapp-chat-relay-staging` R2 空桶（解 CHAT_RELAY 部署阻塞；`RELAY_ENABLED=0` 休眠无害）。
- `wrangler deploy --env staging` 成功（Version `d09d7b2e-8fad-48bc-b378-51c488ee4548`；`SPARK_PRICE_ID=price_1TtCrx…`、`RELAY_ENABLED=0` 已确认）。
- D1 **外科式**只重建 `square_memberships`（DROP+CREATE+4 索引，唯一 schema 差异），**保留其它 20 表**（不清卡2 聊天测试数据，双轨互不干扰）。核验：新表仅 `entitlement_lapsed_at`/`prepaid_payment_ref`，无 `identity_level`/`frozen_at`/`collection_paused`；库仍 21 表。
- worker 路由 `www.crcfrcn.com/api-staging/*` 存活（未登录 302 = Cloudflare Access 门，符合既有口径）；深度功能验收须过 Access（App 指向 staging 或你侧 Access 登录）。
- **staging Stripe 未动**（sandbox 账户 `acct_1Trr2q…` 本 MCP 未连）：功能非阻塞（价 ID 已正确映射），仅测试 Checkout 显示旧产品名——待授权该账户或你手动改名/归档。

## 0. 对齐矩阵（端 × 环境）

| 端 | 本地 dev | staging（测试） | production（生产） |
|---|---|---|---|
| Worker 代码 | dev 跑 | `deploy --env staging` | `deploy --env production` |
| D1 schema | 本地重建 | 远端重建（清空+0001） | 远端重建（清空+0001） |
| Stripe | 空价 + dev-proxy | 账户 `acct_1Trr2q…`（测试）**未对齐** | 账户 `acct_1Trr2f…`（LIVE）**✅已对齐** |
| 官网 citizenweb | dev | （无独立 staging，同 prod 域） | Cloudflare Pages `www.crcfrcn.com` |
| CitizenApp | — | — | 重新打包 APK/iOS 并发版 |
| CitizenWallet | — | — | **不涉会员，无需改**（见 §3.6） |

## 1. 本次变更回顾（对齐依据）

- Worker：会员三档 `freedom/democracy/spark`；删身份冻结子系统；发帖分类按身份档（`assertIdentityCanPublishCategory`，竞选须 candidate）、用量按会员档；`SPARK_PRICE_ID`（删 `VOTING_PRICE_ID`）；`square_video_spark`。
- D1：`square_memberships` 删 4 列（`identity_level`/`identity_checked_at`/`frozen_at`/`collection_paused`）。**这是 staging/production 现库与新基线的唯一 schema 差异**。
- App：会员页三档订阅卡；聊天文件上限按档（`chat_file_max_bytes` + `applyMembershipLevel`）；竞选发布收紧为 candidate 身份；星火→薪火 显示名。签名器（op_tag 0x1D）档位无关，无需改。
- 官网：三档并列订阅卡，无身份字段。
- Stripe LIVE：薪火会员 改名、voting 归档、三价 nickname/metadata 统一。

## 2. 依赖与阻塞（先解，否则部署失败）

1. **【硬阻塞】CHAT_RELAY R2 桶必须先存在**：卡2 已在共享 `wrangler.toml` 加 `CHAT_RELAY` 绑定（桶 `citizenapp-chat-relay` / `-staging`，本地 `-dev`）。`wrangler deploy` 绑定到不存在的桶会**直接失败**。故任一 Worker 部署前，必须：
   - 由卡2 创建这三个 R2 桶（含桶级 24h lifecycle），或
   - 本轮先创建空桶（`RELAY_ENABLED="0"` 已把中转逻辑关掉，空桶无害），命令：
     `npx wrangler r2 bucket create citizenapp-chat-relay`（+ `-staging`、`-dev`）。
   - 与卡2 协调：谁先部署谁建桶。
2. **卡2 relay 代码随共享 src 一起发**：`cloudflare/src` 含卡2 的中转代码，但 `RELAY_ENABLED="0"`（三环境）已使其休眠，Worker 部署对卡2 安全（不激活未完成中转）。
3. **staging Stripe 属另一账户 `acct_1Trr2qQlQZ1x0Cw8`**：本会话 Stripe MCP 只连 LIVE `acct_1Trr2f…`，无法改 sandbox。staging 产品名仍是旧值（visitor/voting/candidate）。**功能不受影响**（wrangler.staging 已把 `SPARK_PRICE_ID=price_1TtCrx…`、`FREEDOM/DEMOCRACY` 正确映射，订阅走 price_id + 订阅 metadata，产品名仅测试 Checkout 显示）。若要测试环境也显示「薪火会员」，须你在 sandbox 账户手动改名/归档，或把 MCP 切到该账户后我来做。

## 3. 分端方案

### 3.1 Stripe

- **LIVE（生产）✅ 已完成**：`prod_UraY9wtqb2XjcY`→薪火会员；`prod_UraY5SJQSS3i0e`（投票）归档；`price_1TrrN8…`（voting）停用；三价 nickname/metadata=freedom/democracy/spark。最终 3 个活跃订阅价与 `wrangler.production` 一致。
- **sandbox（测试）待办**：在 `acct_1Trr2q…` 账户：候选产品（$99.99，price_1TtCrx…）改名「薪火会员」；投票产品 + price_1TtCrw…（voting $9.99）归档/停用。功能非阻塞，属显示对齐。**需你授权该账户或代改**。
- **本地**：`FREEDOM/DEMOCRACY/SPARK_PRICE_ID=""` 保持空；本地不做真实 Checkout（如需本地端到端，置 `STRIPE_DEV_PROXY="1"` 短路）。

### 3.2 Worker（Cloudflare）

- 前置：先解 §2.1（建 CHAT_RELAY 桶）+ 确认 5 项 Secret 已在（`STRIPE_API_KEY`/`STRIPE_HOOK_SECRET`/`CHAIN_URL`/`CHAIN_ID`/`CHAIN_SECRET`；本次不增删 Secret，价 ID 是 vars 随部署下发）。
- 部署：`npm run deploy:staging` → 验收 → `npm run deploy:production`。
- 价 ID 已在 `wrangler.toml` staging/production vars 就位（SPARK 复用原 $99.99 价），部署即生效，无手工 Secret 步骤。

### 3.3 D1（清空 + 基线重建，零用户）

`0001_square_core.sql` 是唯一目标基线（仅 CREATE，无 DROP）。现有 staging/production 库带旧 `square_memberships`（4 个已删列），须先清后建。零用户，直接重置（`feedback_in_development_zero_users`）：

- 每环境执行（staging 示例）：
  1. 删全部表：对 21 张表逐一 `npx wrangler d1 execute citizenapp-square-db-staging --env staging --remote --command "DROP TABLE IF EXISTS <t>;"`（或整库删除后 `d1 create` 重建同名同 id）。
  2. 重建基线：`npm run db:staging`（= `d1 execute … --file migrations/0001_square_core.sql`）。
- production 同理走 `db:production`。本地走 `db:local`。
- 注：0001 全 21 表为两轨共享，重建对卡2 安全（卡2 无 D1 新表，relay=R2、群=Isar）。

### 3.4 官网 citizenweb

- 构建：`npm run build`（tsc+vite，已过）。
- 部署：`npx wrangler pages deploy dist --project-name citizenweb --branch main`（生产 `www.crcfrcn.com`）。
- 验收：真实访问 `/membership` 确认三档并列卡（自由/民主/薪火）、聊天文件上限行、无身份字段、无控制台报错、同源 `/api` 通。

### 3.5 CitizenApp（Flutter）

- 代码已完工（analyze 干净、149 test 过）。构建期可 `--dart-define=MEMBERSHIP_URL=…` 覆盖官网会员页地址（缺省 `https://www.crcfrcn.com/membership`）。
- 打包 Android APK / iOS，走既有发版流程（GitHub release 固定资产名 `citizenapp-android.apk`）。
- 真机 e2e：会员页三档订阅/取消跳官网、聊天按档限额、竞选发布仅竞选身份可选、护照页三身份介绍。

### 3.6 CitizenWallet（无需改，列明理由）

- 会员订阅签名 = **CitizenApp 热钱包**对 `signing_message(0x1D)` 主钥签名（`square_action_payload.dart`，档位仅作 context 字符串，签名器不校验档名，故三档→改名对签名器透明）。
- CitizenWallet 冷钱包只做 **Substrate 链上 extrinsic** 的严格两色解码（`payload_decoder.dart` 只认 register/upgrade identity 等链交易），**不解码会员 op_tag**。会员非链交易，冷钱包不涉。→ 无改动、无重打包。

## 4. 部署顺序（依赖优先）

1. 建 CHAT_RELAY R2 桶（解 §2.1，与卡2 协调谁建）。
2. Stripe：LIVE 已完；sandbox 待你授权（非阻塞，可后补）。
3. Worker + D1（同环境成对，先 staging 后 production）：
   staging → `deploy:staging` + D1 重建 → 官网/临时前端指 staging `/api-staging` 验收 →
   production → `deploy:production` + D1 重建。
4. 官网 Pages 部署（生产）。
5. CitizenApp 打包发版。
6. 真机端到端全链路验收。

## 5. 分环境验证

- Worker（各环境 HTTP）：`GET /v1/square/membership` 返回三档 `plans[]`（含 `chat_file_max_bytes`）、无 `identity`/`eligible_levels`；三档订阅 challenge 出 op_tag 0x1D；非竞选身份发竞选帖 `campaign_identity_required`。
- D1：`square_memberships` 无 `identity_level`/`frozen_at`/`collection_paused` 列。
- Stripe LIVE：仅 3 活跃订阅价（薪火/民主/自由）；订阅落 webhook 写 D1。
- 官网：三档卡渲染。
- App：三档卡 + 聊天按档 + 竞选身份闸门 + 护照三身份介绍。

## 6. 回滚

- 零用户，回滚成本低。Worker：`wrangler rollback`（回上一版本）。D1：重跑旧基线（须留旧 0001 副本；本轮建议 git 保留旧文件历史）。Stripe：voting 产品/价 `active=true` 恢复、薪火产品改回旧名（人工）。官网：Pages 回滚到上一 deployment。

## 7. 待你决策 / 授权

1. **部署授权**：Worker/D1/官网部署是外向操作，需你明确「开始部署」（并确认 staging 先行）。
2. **CHAT_RELAY 桶归属**：本轮先建空桶，还是等卡2 建？（建议先建空桶解阻塞，`RELAY_ENABLED=0` 无害）。
3. **sandbox Stripe 对齐**：授权我连 `acct_1Trr2q…` 账户代改，还是你手动改测试产品名？
4. **D1 重置确认**：staging/production 库将被清空重建（零用户，符合开发期规则）——确认可清。
