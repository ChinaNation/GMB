-- 创作者会员：档位定义（全链下真源）+ 订阅镜像（链上 Subscriptions 的边缘镜像，供聚合/门禁）。
-- 金额一律「分」。档位/订阅的权威真源分别是 Cloudflare（档定义）与链上（订阅关系）。

CREATE TABLE IF NOT EXISTS square_creator_plans (
  creator_account TEXT PRIMARY KEY,
  tiers_json TEXT NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS square_creator_subscriptions (
  subscriber_account TEXT NOT NULL,
  creator_account TEXT NOT NULL,
  tier_id TEXT NOT NULL,
  period TEXT NOT NULL CHECK (period IN ('monthly', 'quarterly', 'yearly')),
  price_fen INTEGER NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('active', 'past_due', 'cancelled')),
  last_charged_at INTEGER NOT NULL,
  last_tx_hash TEXT NOT NULL,
  updated_at INTEGER NOT NULL,
  PRIMARY KEY (subscriber_account, creator_account)
);

CREATE INDEX IF NOT EXISTS idx_scs_creator
  ON square_creator_subscriptions (creator_account, status);
