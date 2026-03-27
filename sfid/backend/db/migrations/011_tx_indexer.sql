BEGIN;

-- 交易记录表（每条代表一次余额变动）
CREATE TABLE IF NOT EXISTS tx_records (
  id              BIGSERIAL PRIMARY KEY,
  block_number    BIGINT NOT NULL,
  extrinsic_index SMALLINT,
  event_index     SMALLINT NOT NULL,
  tx_type         TEXT NOT NULL,
  from_address    TEXT,
  to_address      TEXT,
  amount_fen      BIGINT NOT NULL,
  fee_fen         BIGINT,
  block_timestamp TIMESTAMPTZ,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_tx_records_from
  ON tx_records (from_address, block_number DESC);
CREATE INDEX IF NOT EXISTS idx_tx_records_to
  ON tx_records (to_address, block_number DESC);
CREATE INDEX IF NOT EXISTS idx_tx_records_block
  ON tx_records (block_number DESC);
CREATE INDEX IF NOT EXISTS idx_tx_records_type
  ON tx_records (tx_type);

-- 索引进度表（单行，记录已扫到哪个区块）
CREATE TABLE IF NOT EXISTS tx_indexer_state (
  id                 INT PRIMARY KEY DEFAULT 1 CHECK (id = 1),
  last_indexed_block BIGINT NOT NULL DEFAULT 0,
  updated_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO tx_indexer_state (last_indexed_block)
VALUES (0)
ON CONFLICT DO NOTHING;

COMMIT;
