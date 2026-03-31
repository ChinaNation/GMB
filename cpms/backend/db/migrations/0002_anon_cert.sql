-- SFID-CPMS QR v1: 匿名证书相关字段
ALTER TABLE system_install ADD COLUMN IF NOT EXISTS install_token TEXT;
ALTER TABLE system_install ADD COLUMN IF NOT EXISTS anon_pubkey TEXT;
ALTER TABLE system_install ADD COLUMN IF NOT EXISTS anon_cert TEXT;
ALTER TABLE system_install ADD COLUMN IF NOT EXISTS anon_key_encrypted TEXT;
ALTER TABLE system_install ADD COLUMN IF NOT EXISTS blinding_factor TEXT;
ALTER TABLE system_install ADD COLUMN IF NOT EXISTS rsa_public_key_pem TEXT;
