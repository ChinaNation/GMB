# CPMS Initialize 模块技术文档

## 1. 模块定位
`backend/src/initialize/` 负责 CPMS 安装初始化、安装授权材料入库、ARCHIVE 签发密钥生成和超级管理员绑定。

本模块只消费 SFID 签发的 `SFID_CPMS_V1 / INSTALL` 安装码，不再维护旧中间注册状态。
CPMS 是离线系统，初始化阶段不要求外置验签公钥；档案码真实性由 SFID 在 ARCHIVE 验真阶段闭环确认。

## 2. 负责范围
- 安装状态查询：`GET /api/v1/install/status`
- 使用 SFID 安装码初始化：`POST /api/v1/install/initialize`
- 超级管理员绑定：`POST /api/v1/install/super-admin/bind`
- 生成 CPMS 本机 ARCHIVE 签发密钥：`qr_sign_keys.key_id = ARCHIVE`
- 解密运行时需要的 `install_secret`，供 `dangan` 构造 `geo_seal`

## 3. INSTALL 输入
`install/initialize` 接收 `sfid_init_qr_content`，内容支持 JSON 或 Base64(JSON)，载荷必须是：

```json
{
  "proto": "SFID_CPMS_V1",
  "type": "INSTALL",
  "sfid_number": "GFR-GD001-ZG0X-123456789-2026",
  "province_name": "广东省",
  "city_name": "广州市",
  "install_secret": "0x...",
  "sig": "0x..."
}
```

字段名固定使用 `sfid_number`，不得新增同义字段。INSTALL 签名原文固定为：

```text
sfid-cpms-v1|install|{sfid_number}|{province_name}|{city_name}|{install_secret_hash}
```

`install_secret_hash = blake2b_256(install_secret)`。CPMS 离线保存安装材料，不依赖外置 SFID 公钥完成初始化。

## 4. 数据落库
- `system_install`：保存 `sfid_number / province_name / city_name / install_secret / install_secret_hash / cpms_pubkey`。
- `qr_sign_keys`：保存本机 `ARCHIVE` 签发密钥。
- `admin_users`：保存超级管理员和操作员账号。
- `address_towns/address_villages`：在同一初始化事务内重建安装码对应市的镇/村路运行表。

`install_secret` 与 ARCHIVE 私钥使用 `CPMS_KEY_ENCRYPT_SECRET` 作为 32 字节 hex 主密钥，通过 AES-256-GCM 加密后落库，格式为 `enc:gcm:<nonce_hex>:<cipher_hex>`。未配置或格式错误时拒绝初始化；已初始化实例启动时如果存在加密材料，也必须能读取该主密钥。

## 5. 安全约束
- `proto` 必须为 `SFID_CPMS_V1`，`type` 必须为 `INSTALL`。
- `sfid_number` 必须能解析出省市代码，且 `province_name/city_name` 必须和 SFID 工具行政区真源一致。
- 初始化必须先完成 INSTALL 校验、主密钥校验、ARCHIVE 密钥生成、安装材料落库、地址表重建和超级管理员绑定前置校验；任何一步失败都不得提交半初始化状态。
- `system_install.sfid_number` 已存在时拒绝重复初始化；本阶段按清库重装处理，不提供旧库迁移兼容。
- 超级管理员只允许绑定 1 个，`admin_pubkey` 不允许重复。

## 6. 模块边界
- 初始化相关路由与本机安装材料读取集中在 `initialize`。
- 登录认证在 `login`。
- 权限校验在 `authz`。
- 档案号、`geo_seal` 和 ARCHIVE 签名在 `dangan`。
