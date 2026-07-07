-- 文章长文分类：链上仍发 Normal，文章标记只落链下 D1 + R2 manifest。
-- content_format 区分普通帖(normal)与文章(article)；title 为文章标题（普通帖为空）。

ALTER TABLE square_posts ADD COLUMN content_format TEXT NOT NULL DEFAULT 'normal';
ALTER TABLE square_posts ADD COLUMN title TEXT;

-- 按作者 + 发布态 + 内容形态 + 时间倒序拉帖（文章 Tab / 帖子 Tab 排除文章）。
CREATE INDEX IF NOT EXISTS idx_square_posts_owner_format
  ON square_posts(owner_account, post_state, content_format, created_at);
