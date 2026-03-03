BEGIN;

DROP VIEW IF EXISTS v_key_admins;
DROP VIEW IF EXISTS v_super_admins;
DROP VIEW IF EXISTS v_operator_admins;

CREATE OR REPLACE VIEW v_key_admins AS
SELECT a.*, k.slot, k.keyring_version, k.updated_at AS slot_updated_at
FROM admins a
JOIN key_admin_keyring k ON k.admin_id = a.admin_id
WHERE a.role = 'KEY_ADMIN';

CREATE OR REPLACE VIEW v_super_admins AS
SELECT a.*, s.province_name, s.scope_no
FROM admins a
JOIN super_admin_scope s ON s.admin_id = a.admin_id
WHERE a.role = 'SUPER_ADMIN';

CREATE OR REPLACE VIEW v_operator_admins AS
SELECT
  a.*,
  o.province_name,
  o.super_admin_id,
  sa.admin_pubkey AS super_admin_pubkey
FROM admins a
JOIN operator_admin_scope o ON o.admin_id = a.admin_id
JOIN admins sa ON sa.admin_id = o.super_admin_id
WHERE a.role = 'OPERATOR_ADMIN';

COMMIT;
