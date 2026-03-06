CREATE TABLE IF NOT EXISTS tx_prepared (
    prepared_id TEXT PRIMARY KEY,
    created_at BIGINT NOT NULL,
    expires_at BIGINT NOT NULL,
    signer_pubkey BYTEA NOT NULL,
    from_address TEXT NOT NULL,
    to_address TEXT NOT NULL,
    amount_fen TEXT NOT NULL,
    symbol TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_tx_prepared_expires_at ON tx_prepared (expires_at);

CREATE TABLE IF NOT EXISTS tx_runtime (
    tx_hash TEXT PRIMARY KEY,
    status TEXT NOT NULL,
    failure_reason TEXT,
    updated_at BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_tx_runtime_updated_at ON tx_runtime (updated_at DESC);

CREATE TABLE IF NOT EXISTS chain_bind_requests (
    id BIGSERIAL PRIMARY KEY,
    account_pubkey TEXT NOT NULL,
    accepted BOOLEAN NOT NULL,
    result_code INTEGER,
    requested_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_chain_bind_requests_account_pubkey
    ON chain_bind_requests (account_pubkey);
