-- 通讯录密文已由 account_id 关系索引彻底切换为 CID 永久关系索引。
-- 旧密文无法按新 AAD、HMAC 与载荷契约解密，迁移时必须一次性清空，禁止兼容。
DELETE FROM square_contacts;
