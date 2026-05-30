-- CPMS 当前基线：QR 登录挑战不再保存旧 challenge_payload 残留。

BEGIN;

ALTER TABLE login_challenges DROP COLUMN IF EXISTS challenge_payload;

COMMIT;
