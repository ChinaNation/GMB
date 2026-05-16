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

`install_secret` 与 ARCHIVE 私钥使用 `CPMS_KEY_ENCRYPT_SECRET` 派生密钥加密存储；开发环境未配置时允许明文落库并打印警告。

## 5. 安全约束
- `proto` 必须为 `SFID_CPMS_V1`，`type` 必须为 `INSTALL`。
- `sfid_number` 必须能解析出省市代码；CPMS 不维护本地省市真源。
- `system_install.sfid_number` 已存在时拒绝重复初始化；本阶段按清库重装处理，不提供旧库迁移兼容。
- 超级管理员只允许绑定 1 个，`admin_pubkey` 不允许重复。

## 6. 模块边界
- 初始化相关路由与本机安装材料读取集中在 `initialize`。
- 登录认证在 `login`。
- 权限校验在 `authz`。
- 档案号、`geo_seal` 和 ARCHIVE 签名在 `dangan`。
