-- CPMS 当前基准约束：钱包公钥在档案生命周期内唯一，硬删除物理删除后自然释放。

BEGIN;

CREATE UNIQUE INDEX IF NOT EXISTS uq_archives_wallet_pubkey_lifetime
  ON archives (wallet_pubkey)
  WHERE wallet_pubkey IS NOT NULL;

COMMIT;
