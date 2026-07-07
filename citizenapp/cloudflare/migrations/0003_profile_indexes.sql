-- 用户主页：按作者拉帖 + 计数聚合所需索引。
-- profile.json 存 R2，不建表；这里只补 D1 查询索引。

-- 按作者 + 发布态 + 时间倒序拉帖，兼作 owner 维度帖子计数。
CREATE INDEX IF NOT EXISTS idx_square_posts_owner
  ON square_posts(owner_account, post_state, created_at);

-- 粉丝数：按被关注账户反查（square_follows 主键前缀已覆盖 owner_account 关注数）。
CREATE INDEX IF NOT EXISTS idx_square_follows_followed
  ON square_follows(followed_account);
