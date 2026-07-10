-- 会员改造：移除「账户总储存上限」维度 + 视频退订冷归档状态。
-- 见任务卡 20260710-membership-video-archive-revamp。

-- 1) 移除账户总储存上限维度（对齐 YouTube/推特：储存不再作为限制/售卖维度）。
--    两列无任何索引引用，可直接 DROP（D1/SQLite ≥ 3.35 支持 DROP COLUMN）。
ALTER TABLE square_memberships DROP COLUMN storage_quota_bytes;
ALTER TABLE square_memberships DROP COLUMN storage_used_bytes;

-- 2) 会员权益失效时刻：作为「退订满 N 月冷归档」的时钟起点；重新订阅时置 NULL 清零。
ALTER TABLE square_memberships ADD COLUMN entitlement_lapsed_at INTEGER;

-- 3) 视频冷归档状态（仅视频资产使用；图片恒 live）。
--    live=在 Stream 可播 / archived=已移入 R2 冷存不可播 / restoring=重订后回灌中。
ALTER TABLE square_media_assets ADD COLUMN archive_state TEXT NOT NULL DEFAULT 'live';
ALTER TABLE square_media_assets ADD COLUMN archived_at INTEGER;
ALTER TABLE square_media_assets ADD COLUMN r2_archive_key TEXT;

-- 冷归档扫描：按失效时刻定位到期账户。
CREATE INDEX IF NOT EXISTS idx_square_memberships_lapsed
  ON square_memberships(entitlement_lapsed_at)
  WHERE entitlement_lapsed_at IS NOT NULL;

-- 归档/回灌：按 owner + 归档态定位视频资产。
CREATE INDEX IF NOT EXISTS idx_square_media_archive
  ON square_media_assets(owner_account, archive_state);
