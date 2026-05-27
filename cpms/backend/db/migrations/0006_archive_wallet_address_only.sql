BEGIN;

-- 中文注释：CPMS 只保存线下确认的钱包地址，钱包签名验证移到 SFID 绑定阶段。
DROP TABLE IF EXISTS archive_wallet_challenges;

ALTER TABLE archives
  DROP COLUMN IF EXISTS wallet_proof_payload,
  DROP COLUMN IF EXISTS wallet_signature;

COMMIT;
