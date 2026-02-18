-- CPMS seed data (offline LAN edition)
-- Depends on: cpms/schema.sql

BEGIN;

-- 初始化超级管理员（仅在不存在同名用户时插入）
-- 注意：首次登录后必须在系统内强制修改密码。
INSERT INTO users (
  user_id,
  username,
  password_hash,
  role,
  status,
  failed_login_count,
  created_by,
  created_at,
  updated_at
)
VALUES (
  'u_super_admin',
  'superadmin',
  '$argon2id$v=19$m=65536,t=3,p=1$REPLACE_SALT$REPLACE_HASH',
  'SUPER_ADMIN',
  'ACTIVE',
  0,
  NULL,
  NOW(),
  NOW()
)
ON CONFLICT (username) DO NOTHING;

-- 写入初始化审计日志（幂等）
INSERT INTO audit_logs (
  log_id,
  operator_user_id,
  action,
  target_type,
  target_id,
  result,
  trace_id,
  client_host,
  detail,
  prev_hash,
  curr_hash,
  created_at
)
SELECT
  'log_seed_superadmin_init',
  NULL,
  'SYSTEM_INIT_SUPER_ADMIN',
  'USER',
  'u_super_admin',
  'SUCCESS',
  'seed_init',
  'localhost',
  jsonb_build_object('username', 'superadmin'),
  NULL,
  encode(digest('SYSTEM_INIT_SUPER_ADMIN:u_super_admin', 'sha256'), 'hex'),
  NOW()
WHERE NOT EXISTS (
  SELECT 1 FROM audit_logs WHERE log_id = 'log_seed_superadmin_init'
);

COMMIT;

