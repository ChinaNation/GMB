BEGIN;

ALTER TABLE super_admin_scope
  ADD COLUMN IF NOT EXISTS scope_no INTEGER;
ALTER TABLE super_admin_scope
  DROP CONSTRAINT IF EXISTS super_admin_scope_scope_no_key;
ALTER TABLE super_admin_scope
  ADD CONSTRAINT super_admin_scope_scope_no_key UNIQUE (scope_no);

CREATE TEMP TABLE tmp_super_admin_catalog (
  scope_no INTEGER PRIMARY KEY,
  province_name TEXT NOT NULL UNIQUE,
  admin_pubkey TEXT NOT NULL UNIQUE
) ON COMMIT DROP;

INSERT INTO tmp_super_admin_catalog(scope_no, province_name, admin_pubkey) VALUES
  (1, '中枢省', '0x21bc9e12d717e4d55666501fd21f8f3fdfbf98d513d6584424f34162397ac1be'),
  (2, '岭南省', '0xfaf75f9cb6945e1f61cb42a26d8b6c35614794830ecfc06477e8ace8e1c844b8'),
  (3, '广东省', '0x86eaef716945c2080b9348b8bc9aaede939be2bd875ddb2d3043edbcf2c42ddc'),
  (4, '广西省', '0x209c88e0f516c4bef0bd7295a28065cd05ac8c2d4ed40b157a67dc55dc8d3e53'),
  (5, '福建省', '0x7a5d47815b1ae50c39ad89b68f8f7827dffedabbe0a349484a873cc0a3e94ba2'),
  (6, '海南省', '0xe242a55b40fbf6c0a29e5aa316965a93ec67d09aafc266f7e8895bdb3e06b5d5'),
  (7, '云南省', '0x49542724fe089ab265daeceee4a6c9f5028ac623b9602230664578d2c1ea8442'),
  (8, '贵州省', '0x523248ae37ea8689ca9e5a6d505e1610c67aa7e4646505b8b8750b82aeba9b6c'),
  (9, '湖南省', '0xbae6d80a94732903af02202d0ada61d7e74f25f0a8cdf32b6d73260dc418e39f'),
  (10, '江西省', '0xfead3049fa97c8fd8b07f319cfe42bef096831cec0235dda5b4ad8af93827180'),
  (11, '浙江省', '0x0f327a7b55043d17b16596f0c04729667696a666c38b419a237d8961fb343a00'),
  (12, '江苏省', '0x65c81d68fc9970b3ae2ec820ebb494f9c0e571a46154f2beec36de05f15ec71c'),
  (13, '山东省', '0x5213d43ecb0c0f9ef7156173cb4bb3310ccece5ae59259837b2238fc7ad7640e'),
  (14, '山西省', '0x76d4f36462eca50cdc3aa7f59650dd957200ba8fa88c3a9e27547cb79ec92266'),
  (15, '河南省', '0x682792dc6945be8fe5bdcdbe72f19fa40c42c39411d56a206516d4a223884d91'),
  (16, '河北省', '0xbe824a50fb2e456cc8ad0dac169c6f42818525b6de16647284c433d5a36c05a5'),
  (17, '湖北省', '0xa8c7569f9fd0eea135a453b9b2f1e32f3222c4f5a4981ba59111f5220e67d7f1'),
  (18, '陕西省', '0xa3dbf6e743d0712ff6be8d852ec2a892891dc35d5d2ef260c112afe291c49b64'),
  (19, '重庆省', '0x95f0a21e4d83326867e8ce82287e91f9606075cc77c21f1b1799a6224dc058c6'),
  (20, '四川省', '0x97c73ed34795384c351a59d81d876011805cf41a0b40da822d7d051137f9059d'),
  (21, '甘肃省', '0x37b05070ec70ad7444f208dc1b0f15432fe00bedb05f4fdb061ac83ee68800bc'),
  (22, '北平省', '0x4b724514f0c03b8289de69592687de7bd54096e6801ee1b55604b29789055e8e'),
  (23, '海滨省', '0xaf2cd60d5e63d8d1dff54c391c1567c89998ce4114fddb80d84d84a8f9e4db04'),
  (24, '松江省', '0xf897f0119dfb14035a841713a7cf889f903d262d26c187fd329bb4ba6a7b5be3'),
  (25, '龙江省', '0x41f918fd7f8ec10f6fa1e4d67094583742585aae5ec5a2ac97fd046b4d4dc48c'),
  (26, '吉林省', '0x0635c25df1cf9dd1fbe6d8e4bea1de71a0e8c1aab0d2dbf5fe2dbda39b64d798'),
  (27, '辽宁省', '0xb1684c88713edd73414eb23fb99608ba71495e2bed457da6d5b9afc1921e2fa4'),
  (28, '宁夏省', '0x2dbfa78e5e41e7593ae96be8623cbfe019d181e1c93d598239713ad5bd3f5472'),
  (29, '青海省', '0x0d71aa43a60982e9baf30e2898644701680737814ceeb724845c1ed3874cd685'),
  (30, '安徽省', '0x8029f0568a5a803d635179e1b84b1e82140670b425ff058497577f4e898b9829'),
  (31, '台湾省', '0x5d489483bb48d1d447cbdcbc8fbb4c71e37eda4cf31340fe6c4f5aa640f72fc5'),
  (32, '西藏省', '0x8f41d40cd896fa0af97c3052b6486353ab6f75d479f77ddf224af89a1c2977b6'),
  (33, '新疆省', '0x587bc76d45042bcd9ff52ceee72612e3b71a734059038c46e9234985f308c373'),
  (34, '西康省', '0xf66395985ccc73d95fb290504973b376f02d7d98da93b20d95c343da8d152de7'),
  (35, '阿里省', '0xbc789a1c394a5e485d38432bcdb59fad3411cfd4699a7983f5853997867401c4'),
  (36, '葱岭省', '0xed3f67c63bc0f2140205473c17890e73688babdf7684f9287ce7cddfcd22ca09'),
  (37, '天山省', '0xdd464573534b2eb6e0d7a6fcac549a51c63e5847b23c299135e36b4ea3e24adb'),
  (38, '河西省', '0x8f8b9105949a108774ff845ba4df0f7a921f5beae6e814717fabf9b99a1ce8bd'),
  (39, '昆仑省', '0xe7c1fa0801ca5b74c6939de99a519de7023dc602ab55231b640e3b1b7f5895f0'),
  (40, '河套省', '0x2ffbe3ed5ee5134149adec09698affcea20f5cca51dbf95cdb0616430eee84aa'),
  (41, '热河省', '0x6974ef45d1495159dbbacfc2a114e284b4538775dfd2c3e7296ed681285406f8'),
  (42, '兴安省', '0x10ac83be1c5cdc8c5e762323d037eb792bd783d362d77b6e52e4547e42174a3e'),
  (43, '合江省', '0xeb85e6981f71269f7dc22f4715119f943b675bb98a0ae5427ede0004f2bad626');

INSERT INTO provinces(province_name)
SELECT province_name FROM tmp_super_admin_catalog
ON CONFLICT (province_name) DO NOTHING;

DELETE FROM operator_admin_scope
WHERE super_admin_id IN (SELECT admin_id FROM admins WHERE role = 'SUPER_ADMIN');

DELETE FROM super_admin_scope;
DELETE FROM admins WHERE role = 'SUPER_ADMIN';

INSERT INTO admins(admin_pubkey, role, status, built_in, created_by, created_at)
SELECT admin_pubkey, 'SUPER_ADMIN', 'ACTIVE', TRUE, 'SYSTEM', now()
FROM tmp_super_admin_catalog
ORDER BY scope_no;

INSERT INTO super_admin_scope(admin_id, province_name, scope_no)
SELECT a.admin_id, c.province_name, c.scope_no
FROM tmp_super_admin_catalog c
JOIN admins a ON a.admin_pubkey = c.admin_pubkey
ORDER BY c.scope_no;

COMMIT;
