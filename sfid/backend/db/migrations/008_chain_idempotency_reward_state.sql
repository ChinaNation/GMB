BEGIN;

-- 区块链接口幂等与防重放落库（跨进程/重启可追踪）
CREATE TABLE IF NOT EXISTS chain_idempotency_requests (
  id BIGSERIAL PRIMARY KEY,
  route_key TEXT NOT NULL,
  request_id TEXT NOT NULL,
  nonce TEXT NOT NULL,
  request_timestamp BIGINT NOT NULL,
  fingerprint TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_chain_idempotency_route_request
  ON chain_idempotency_requests(route_key, request_id);
CREATE UNIQUE INDEX IF NOT EXISTS uq_chain_idempotency_route_nonce
  ON chain_idempotency_requests(route_key, nonce);
CREATE INDEX IF NOT EXISTS idx_chain_idempotency_created_at
  ON chain_idempotency_requests(created_at DESC);

-- 绑定强约束（防并发双绑）
CREATE TABLE IF NOT EXISTS binding_unique_locks (
  account_pubkey TEXT PRIMARY KEY,
  archive_index TEXT NOT NULL UNIQUE,
  bound_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 绑定奖励状态机
CREATE TABLE IF NOT EXISTS bind_reward_states (
  account_pubkey TEXT PRIMARY KEY,
  archive_index TEXT NOT NULL,
  callback_id TEXT NOT NULL UNIQUE,
  reward_status TEXT NOT NULL,
  retry_count INTEGER NOT NULL DEFAULT 0,
  max_retries INTEGER NOT NULL DEFAULT 5,
  reward_tx_hash TEXT NULL,
  last_error TEXT NULL,
  next_retry_at TIMESTAMPTZ NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_bind_reward_states_status_next
  ON bind_reward_states(reward_status, next_retry_at NULLS LAST);

COMMIT;
