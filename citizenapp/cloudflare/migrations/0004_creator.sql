-- 创作者会员：档位展示镜像 + 订阅镜像（链上 CreatorPlans/Subscriptions 的边缘镜像）。
-- 金额一律「分」；tier_id、周期、价格和订阅关系的权威真源都在链上。

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
  status TEXT NOT NULL CHECK (status IN ('active', 'terminated', 'cancelled')),
  last_charged_at INTEGER NOT NULL,
  last_tx_hash TEXT NOT NULL,
  updated_at INTEGER NOT NULL,
  PRIMARY KEY (subscriber_account, creator_account)
);

CREATE INDEX IF NOT EXISTS idx_scs_creator
  ON square_creator_subscriptions (creator_account, status);
-- 对账器按 updated_at 最旧优先滚动取批（membership/reconcile.ts）。
CREATE INDEX IF NOT EXISTS idx_scs_reconcile
  ON square_creator_subscriptions (updated_at);
