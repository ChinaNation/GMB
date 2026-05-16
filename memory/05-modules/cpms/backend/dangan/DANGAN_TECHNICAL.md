# CPMS Dangan 模块技术文档

## 1. 模块定位
`backend/src/dangan/` 负责档案号生成、`SFID_CPMS_V1 / ARCHIVE` 档案二维码构建与签名、公民状态校验。

本模块不保存实名归属判断逻辑；省市归属来自 `initialize` 保存的 INSTALL 授权材料，并只写入加密 `geo_seal`。

## 2. 负责范围
- `generate_archive_no_with_retry(...)`：生成不暴露省市和机构号的档案号。
- `build_archive_qr_payload(...)`：构造 ARCHIVE 二维码。
- `validate_citizen_status(...)`：校验 `NORMAL / ABNORMAL`。

## 3. 档案号规则
- 格式：`<26位Base32>-<2位Base32校验>`。
- 示例：`K8M4ZP7W2Q1C9T6R5N3X8V2Y1A-7H`。
- 明文不包含省、市、CPMS 机构号、日期。
- 不使用固定业务前缀，避免把示例前缀固化成协议含义。
- 生成输入包含 `install_secret`、安全随机数、本机序列、终端 ID、管理员公钥。
- 本机 `archives.archive_no` 唯一索引兜底拒绝重复；SFID 录入时仍做全局唯一最终校验。

## 4. ARCHIVE 载荷

```json
{
  "proto": "SFID_CPMS_V1",
  "type": "ARCHIVE",
  "ano": "K8M4ZP7W2Q1C9T6R5N3X8V2Y1A-7H",
  "cs": "NORMAL",
  "ve": true,
  "cpms_pubkey": "0x...",
  "geo_seal": "g1.<nonce_hex>.<cipher_hex>",
  "sig": "0x..."
}
```

二维码明文字段不得出现 `sfid_number / province_code / city_code`。归属密文 `geo_seal` 只加密 `sfid_number`，由 SFID 根据安装授权中的 `install_secret` 解密。

## 5. 签名与加密
- `geo_seal` 使用 AES-256-GCM。
- `geo_seal` 密钥：`blake2b_256(install_secret)`。
- `geo_seal` AAD：`sfid-cpms-v1|geo-seal|{ano}|{cpms_pubkey}`。

- ARCHIVE 签名原文：

```text
sfid-cpms-v1|archive|{ano}|{cs}|{ve}|{cpms_pubkey}|{geo_seal_hash}
```

- ARCHIVE 签名上下文：`substrate`。

## 6. 模块边界
- 本模块只提供档案号和 ARCHIVE 算法。
- 安装材料读取由 `initialize` 提供。
- 业务权限和请求校验由 `operator_admin` / `authz` 负责。
