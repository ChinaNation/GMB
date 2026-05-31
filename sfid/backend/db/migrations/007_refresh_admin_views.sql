BEGIN;

DROP VIEW IF EXISTS v_sheng_admins;
DROP VIEW IF EXISTS v_shi_admins;

CREATE OR REPLACE VIEW v_sheng_admins AS
SELECT a.*, s.province_name, s.scope_no
FROM admins a
JOIN sheng_admin_scope s ON s.admin_id = a.admin_id
WHERE a.role = 'SHENG_ADMIN';

CREATE OR REPLACE VIEW v_shi_admins AS
SELECT
  a.*,
  o.province_name,
  o.sheng_admin_id,
  sa.admin_pubkey AS sheng_admin_pubkey
FROM admins a
JOIN shi_admin_scope o ON o.admin_id = a.admin_id
JOIN admins sa ON sa.admin_id = o.sheng_admin_id
WHERE a.role = 'SHI_ADMIN';

COMMIT;
