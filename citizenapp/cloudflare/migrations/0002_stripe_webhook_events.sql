-- Stripe Sandbox 全链路验收增量：事件原子占位 + 一次性付款永久去重。
-- 生产环境不在本任务执行本迁移；staging 由部署控制台在发布前显式应用。

CREATE TABLE IF NOT EXISTS square_stripe_webhook_events (
  event_id TEXT PRIMARY KEY,
  event_type TEXT NOT NULL,
  stripe_object_id TEXT,
  event_created_at INTEGER NOT NULL,
  received_at INTEGER NOT NULL,
  processed_at INTEGER
);
CREATE INDEX IF NOT EXISTS idx_square_stripe_webhook_events_processed
  ON square_stripe_webhook_events(processed_at, received_at);

CREATE TABLE IF NOT EXISTS square_stripe_payments (
  stripe_payment_intent_id TEXT PRIMARY KEY,
  checkout_session_id TEXT NOT NULL,
  owner_account TEXT NOT NULL,
  membership_level TEXT NOT NULL,
  payment_route TEXT NOT NULL,
  granted_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_square_stripe_payments_owner
  ON square_stripe_payments(owner_account, granted_at);
