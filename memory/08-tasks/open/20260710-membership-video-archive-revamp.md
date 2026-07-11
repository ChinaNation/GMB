# 会员体系改造：移除总储存上限 + 视频时长/体积按档 + 退订视频冷归档

- 状态：**已实现**（Worker + App 落地，typecheck 0 error / 103 vitest 全绿 / dart analyze 无问题；归档 Cron 默认关，待灰度+部署）
- 创建：2026-07-10
- 取代：~~20260710-honor-membership-tier~~（荣誉/紫色档作废，已删卡）
- 触达面：citizenapp/cloudflare（Worker·真源）、citizenapp/lib（App）、citizenweb（官网，可能零改）；**链零改动**（会员纯链下）

## 0. 决策汇总（用户逐项确认，锁定）

1. **保留 3 档**（visitor/voting/candidate），荣誉/紫色档作废。
2. **移除「账户总储存上限」维度**（全 3 档不设，删 D1 两列）。对齐 YouTube/推特（储存不作限制维度）。
3. **视频时长/体积按档（Plan B·1080p）**：长度沿用现值，体积新设。
   - 访客：**60s / 512 MB / 标清**
   - 投票：**30min(1800s) / 2 GB / 高清(1080p)**
   - 竞选：**3h(10800s) / 10 GB / 高清(1080p)**
4. **退订(subscription lapse)**：**不删数据**；**徽章勾消失**（退回仅身份）；文本/图片/文章**永久公开保留**。
5. **视频冷归档**：会员失效满 **3 个月** → 视频从 Stream 导出到 R2 冷存(IA) + 删 Stream → **对所有人「已归档」不可播** → **仅作者重新订阅才解冻**（异步回灌 Stream）。**统一归档**（竞选/治理视频不豁免）。数据只在**注销**硬删。

## 1. 价格事实依据（联网核实 2026-07）

- R2 标准 $0.015/GB·月（取回/出网免费）；R2 IA $0.01/GB·月 + 取回 $0.01/GB + 30 天最短计费。
- Cloudflare Stream $5/1000分钟·月存储（**无冷层**）+ $1/1000分钟传输。→ 3h 视频 Stream ≈ $0.90/月；同片 R2-IA ≈ $0.10/月，**省 ~85%**。成本几乎全在视频。
- Stream `/downloads` API 可生成**编码版 MP4**（非原始文件）；导出→存 R2→删 Stream 可行；回灌=二次转码，轻微画质损失（可接受）。

## 2. 数据模型（D1 迁移，migration 000X）

- `square_memberships`：**DROP COLUMN** `storage_quota_bytes`, `storage_used_bytes`（D1/SQLite≥3.35 支持 DROP COLUMN；均为普通列可删）。
- `square_memberships`：**ADD** `entitlement_lapsed_at INTEGER`（权益首次失效时刻；重新 active 清空）——归档 3 月时钟起点，避免从 expires_at 推导的歧义。
- `square_media_assets`（仅 video 行用）：**ADD**
  - `archive_state TEXT NOT NULL DEFAULT 'live'`  — `live | archived | restoring`
  - `archived_at INTEGER`
  - `r2_archive_key TEXT`  — R2 冷存对象键 `archive/{owner}/{stream_uid}.mp4`
  - image 行恒 `live`。

## 3. tier 配置（plans.ts）

- `MembershipLevel` 保持 3 档（不加 honor）。
- `DynamicQuota` **新增 `max_video_bytes: number`**。
- 三档数值（Plan B）：
  - visitor：`max_video_seconds 60`、`max_video_bytes 512*mib`、`video_quality 'sd'`
  - voting：`1800`、`2*gib`、`'hd'`
  - candidate：`10800`、`10*gib`、`'hd'`（时长/清晰度=现值不变，仅确认体积）
- **删 `legacy_storage_quota_bytes`**（interface + 3 档），及唯一读点 `service.ts:233`。

## 4. 移除总储存闸

- `membership/service.ts requireActiveMembership`：删 `remainingBytes`/`storage_quota_exceeded` 校验，退化为「仅校验会员有效」（保 `membership_required` + 非 active 拒）；去掉 `requiredBytes` 参数。
- 删 `addStorageUsage`；删 `uploads/service.ts` completeUpload 的 `storage_used_bytes` 累加；删 `posts/confirm.ts` 删帖回退。
- `uploads/service.ts prepareUpload`：不再用 `estimatedBytes` 做容量校验（`estimateUploadBytes` 账户总量用途下线；`square_uploads.estimated_bytes` 可留作统计或后续清理）。

## 5. 视频体积按档强制

- `uploads/validation.ts`：现 flat 视频 ≤500MB → **改按档** `plan.dynamic.max_video_bytes`（在 `prepareUpload` 已知 plan，逐个 video item `byte_size ≤ 上限` 否则 403 `video_too_large`）。
- 时长：`createProviderUpload` 已把 `plan.dynamic.max_video_seconds` 作 Stream `maxDurationSeconds` 下发；改数字自动生效（Stream 拒超长）。
- 保留 tier 无关的 per-item 硬闸：单图 ≤20MB、manifest ≤512KB、≤101 项。

## 6. 退订 / entitlement 行为

- `cancel_membership` → `cancelStripeSubscriptionAtPeriodEnd`（现有；当期用满再终止）。
- lapse：webhook `customer.subscription.deleted` → `markStripeMembershipInactive` 时**置 `entitlement_lapsed_at=now`**；重新 active 时清空。
- `requireActiveMembership` 拒新上传（现有，去储存校验后）。
- 既有内容：文本/图片恒公开；视频归档前可播。
- 徽章：`membershipActive=false` → 勾消失（App `identity_badge` 已由 membershipActive 驱动，无需改逻辑）。

## 7. 视频冷归档子系统（核心新增）

### 7.1 状态机（per video asset）
`live` —(会员失效满3月)→ `archived` —(作者重订)→ `restoring` —(回灌完成)→ `live`

### 7.2 归档（Cron Worker）
- `wrangler.toml [triggers] crons`（如每日 `0 3 * * *`）。`scheduled()`：
  1. 选 `entitlement_lapsed_at ≤ now-90d` 且仍未 active 的账户。
  2. 该账户 `archive_state='live'` 的 video 资产，逐个：
     a. Stream `POST /downloads` → 轮询 `status=ready` → 取 MP4 url。
     b. 拉 MP4 → `PUT` 到 R2（**storage class = InfrequentAccess**），键 `archive/{owner}/{uid}.mp4`。
     c. **校验 R2 落成（size 比对）**——只有确认落成才继续。
     d. `DELETE` Stream 视频（停 $5/1000min）。
     e. `UPDATE` 资产 `archive_state='archived', archived_at, r2_archive_key`，清播放 url。
  3. **限流分批 + 幂等可续跑**（靠 archive_state 收敛）；单次限量防 Worker 超时。
- **无损铁律**：R2 未确认落成，绝不 DELETE Stream。

### 7.3 展示态（客户端读 archive_state）
- membership/post/media API 返回每个 video 的 `archive_state`。
- App + 官网：`archived` → 「已归档（作者未续订）」占位（**不要渲染成坏播放器**）；`restoring` → 「恢复中」占位。

### 7.4 解冻（restore-on-resubscribe）
- 重订 webhook（subscription 重新 active）→ 入队恢复该 owner 的 `archived` 视频。
- 恢复任务（异步 Cron/队列）：置 `restoring` → Stream 从 R2 MP4 回灌（copy-from-URL）→ 轮询 ready → 更新新 `provider_asset_id`+播放 url、`archive_state='live'`。
- **R2 冷存原片**：回灌后**保留**作冷 master（下次归档只需删 Stream，不用再走 download），成本 $0.01/GB·月，可接受。

### 7.5 注销交互
- `purgeAccount` 扩展：一并删 R2 `archive/{owner}/*` 与任何残留 Stream 视频。

### 7.6 链上
- 归档/删 Stream 不动链；`content_hash`/储存回执不可变。归档期「链上有记录、链下暂不可播」为正常态。

## 8. 客户端改动

### App（citizenapp/lib）
- 套餐 fallback 数值（视频时长/体积）更新；删 `storageQuotaBytes/storageUsedBytes` 解析。
- 媒体展示：读 `archive_state` 渲染「已归档/恢复中」占位。
- compose 视频体积客户端预检（若有）按档。
- 徽章：无需改。

### 官网（citizenweb）
- **可能零改动**：视频时长文案现为「1分钟标清/30分钟高清/3小时高清」，与 Plan B 时长/清晰度**完全一致**；体积不展示、储存维度已删（无储存行）。若要展示单视频体积再另说。

## 9. Stripe / 环境
- 3 档价格与 price id 不变（无新增 Stripe 价格）。
- 新增：Cron trigger 配置；R2 IA 写入（复用现有 R2 binding + storage class 参数，无新 secret）。

## 10. 上线顺序
1. D1 迁移（drop 储存列 + 加 archive 字段 + entitlement_lapsed_at）。
2. plans.ts + 移除储存闸 + 视频体积按档（typecheck+vitest）。
3. 归档子系统（Cron + 状态机 + API 返回 archive_state），**feature flag 后置开**。
4. 客户端读 archive_state。
5. purge 扩展删 R2 archive。

## 11. 测试
- Worker vitest：视频体积按档边界(512M/2G/10G)；移除储存闸后不再 402；归档状态机 live→archived→restoring→live；**R2 落成前绝不删 Stream**；重订触发 restore；purge 删 archive。
- App：archive_state 展示态 widget test；套餐数值。
- 端到端：订→传视频→退订→(模拟3月)Cron 归档→占位→重订→恢复→可播。

## 12. 风险 / 边缘
- Stream download=编码版 MP4，恢复=二次转码轻微画质损失（若要零损失需上传时双写 R2 原片，额外成本，本方案不采）。
- Cron 批量归档/恢复需限流、幂等、可续跑，防 Worker CPU/时长超限。
- 归档期公开视频对所有人变暗——已确认统一归档接受。
- 大 V 重订触发海量回灌，成本/耗时集中——队列限流平滑。
- R2 IA 30 天最短计费 + 归档即删 Stream；3 月阈值已足够保守。

## 13. 实现默认（可逆，如无异议即按此）
- 归档取片：`export-on-archive`（Stream downloads），非上传时双写原片。
- 恢复后保留 R2 冷 master。
- Cron：每日一次；单次限量分批。

## 落地记录（2026-07-10）

### Worker（citizenapp/cloudflare）
- 迁移 `migrations/0009_membership_video_archive.sql`：DROP `storage_quota_bytes`/`storage_used_bytes`；ADD `entitlement_lapsed_at`（membership）、`archive_state`/`archived_at`/`r2_archive_key`（media_assets）+ 两索引。
- `membership/plans.ts`：`DynamicQuota` 加 `max_video_bytes`（访客 512MiB/投票 2GiB/竞选 10GiB）；删 `legacy_storage_quota_bytes`。
- `membership/service.ts`：`requireActiveMembership` 去容量校验+去 `requiredBytes` 参；删 `addStorageUsage`；`upsertStripeMembership` 删储存列+置 `entitlement_lapsed_at=NULL`；`markStripeMembershipInactive` 置 `entitlement_lapsed_at=COALESCE(...,now)`；两处 SELECT 改列。
- `uploads/validation.ts`：单视频绝对硬顶 500MB→10GB；`uploads/quota.ts`：`assertDynamicQuota` 加按档 `max_video_bytes` 校验（错误码 `dynamic_video_too_large`）。
- `uploads/service.ts`：prepare/complete 去容量校验与累加；`streamWebhookRoute` 加 restoring→live（ready 时）。
- `posts/confirm.ts`：删删帖储存回退；`SquareFeedMediaItem` 序列化加 `archive_state`。
- **新增 `membership/archive.ts`**：`runVideoArchiveSweep`（Cron 入口，选 lapsed≥90d 且有 live 视频的账户，导出 Stream→R2 IA→删 Stream，无损铁律）+ `restoreOwnerVideos`（重订回灌）+ 状态机 live/archived/restoring。
- `media/cloudflare_assets.ts`：加 `createStreamDownloadUrl`（downloads API，未就绪返 null 跳过）+ `copyStreamFromUrl`（回灌）。
- `storage/presigned.ts`：加 `signR2GetUrl`（回灌用 R2 只读预签名）。
- `index.ts`：加 `scheduled()`（waitUntil 跑 sweep）；`wrangler.toml`：三环境 `[triggers] crons=["0 3 * * *"]` + `VIDEO_ARCHIVE_ENABLED="0"`（默认关）/`VIDEO_ARCHIVE_LAPSE_DAYS="90"`；`types.ts Env` 加两项。
- `membership/webhook.ts`：订阅重新 active 时 `restoreOwnerVideos`；`account/purge.ts`：注销删 `archive/{owner}/` R2 前缀。
- 测试：新增 `test/archive.test.ts`（归档/跳过未到期/关关/重订恢复 4 例）；改 membership/chain_confirm/uploads_quota 夹具适配新列。

### App（citizenapp/lib）
- `square_api_client.dart`：`SquareMembershipState` 删 `storageQuotaBytes/storageUsedBytes`（死字段）；`_parseMediaItem` 读 `archive_state`。
- `models/square_models.dart`：`SquareMediaItem` 加 `archiveState` + `isArchived/isRestoring`。
- `widgets/square_media_grid.dart`：归档/恢复中视频显示「已归档/恢复中」占位（非坏播放器）。

### 官网（citizenweb）
- **零改动**：视频文案「1分钟标清/30分钟高清/3小时高清」与 Plan B 时长一致；体积不展示、储存维度已删。

### 部署与开关（用户手动）
- `wrangler d1 migrations apply`（staging/production）跑 0009。
- 冷归档默认 `VIDEO_ARCHIVE_ENABLED="0"`；灰度时置 "1" 开启每日 Cron。
- R2 需 S3 凭证（R2_ACCOUNT_ID/ACCESS_KEY/SECRET）才能回灌（signR2GetUrl）；Stream/Images token 已有。

### 保留项（非本次移除）
- `square_uploads.estimated_bytes`：单次上传体积估算，仍随响应回传客户端，属上传元数据（非账户储存残桩），保留。
