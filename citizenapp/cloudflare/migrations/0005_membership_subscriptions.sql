-- 官网 Stripe 订阅与三档会员权益。
-- 旧 square_memberships 行保留 owner_account 主键；新增字段承载订阅来源、Stripe 标识和链上身份资格快照。

ALTER TABLE square_memberships ADD COLUMN subscription_source TEXT NOT NULL DEFAULT 'stripe';
ALTER TABLE square_memberships ADD COLUMN stripe_customer_id TEXT;
ALTER TABLE square_memberships ADD COLUMN stripe_subscription_id TEXT;
ALTER TABLE square_memberships ADD COLUMN stripe_price_id TEXT;
ALTER TABLE square_memberships ADD COLUMN subscription_status TEXT NOT NULL DEFAULT 'active';
ALTER TABLE square_memberships ADD COLUMN current_period_start INTEGER;
ALTER TABLE square_memberships ADD COLUMN current_period_end INTEGER;
ALTER TABLE square_memberships ADD COLUMN cancel_at_period_end INTEGER NOT NULL DEFAULT 0;
ALTER TABLE square_memberships ADD COLUMN identity_level TEXT NOT NULL DEFAULT 'visitor';
ALTER TABLE square_memberships ADD COLUMN identity_checked_at INTEGER;

CREATE UNIQUE INDEX IF NOT EXISTS idx_square_memberships_stripe_subscription
  ON square_memberships(stripe_subscription_id)
  WHERE stripe_subscription_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_square_memberships_stripe_customer
  ON square_memberships(stripe_customer_id)
  WHERE stripe_customer_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_square_memberships_status
  ON square_memberships(subscription_status, expires_at);
