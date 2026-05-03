# CPMS_TECHNICAL

- 最后更新:2026-05-02
- 任务卡:
  - `memory/08-tasks/done/20260502-sfid-cpms-sheng目录整改.md`
  - `memory/08-tasks/done/20260502-sfid-institutions粗粒度整合.md`
  - `memory/08-tasks/done/20260502-sfid-models-scope边界整改.md`

## 0. 模块边界

`sfid/backend/cpms/` 承接 SFID 侧的 CPMS 系统管理功能:

- 生成公安局 CPMS 站点 SFID 与 QR1 安装授权二维码。
- 扫描 CPMS 设备 QR2,返回 QR3 匿名证书。
- 导入 CPMS 档案二维码。
- 查询、禁用、启用、吊销、删除 CPMS 站点。
- 按机构 SFID 反查对应 CPMS 站点。

该模块服务于公安局机构详情页,但不属于省管理员目录。省管理员只是调用这些
接口的授权角色。

## 1. 当前目录

```text
sfid/backend/cpms/
├── mod.rs        # 模块导出
├── model.rs      # CPMS 站点、安装 token、QR1/QR2/QR3/QR4、匿名证书 DTO
├── handler.rs    # CPMS 安装、注册、匿名证书、档案导入和站点状态治理
├── rsa_blind.rs  # RSA 盲签名、匿名证书签发与验签
└── scope.rs      # CPMS 站点是否属于当前管理员省域
```

## 2. API

| 端点 | 代码 |
|---|---|
| `GET /api/v1/admin/cpms-keys` | `cpms::list_cpms_keys` |
| `GET /api/v1/admin/cpms-keys/by-institution/:sfid_id` | `cpms::get_cpms_site_by_institution` |
| `POST /api/v1/admin/cpms-keys/sfid/generate` | `cpms::generate_cpms_institution_sfid_qr` |
| `POST /api/v1/admin/cpms/register` | `cpms::register_cpms` |
| `POST /api/v1/admin/cpms/archive/import` | `cpms::archive_import` |
| `DELETE /api/v1/admin/cpms-keys/:site_sfid` | `cpms::delete_cpms_keys` |
| `POST /api/v1/admin/cpms-keys/:site_sfid/revoke-token` | `cpms::revoke_install_token` |
| `POST /api/v1/admin/cpms-keys/:site_sfid/reissue` | `cpms::reissue_install_token` |
| `PUT /api/v1/admin/cpms-keys/:site_sfid/disable` | `cpms::disable_cpms_keys` |
| `PUT /api/v1/admin/cpms-keys/:site_sfid/enable` | `cpms::enable_cpms_keys` |
| `PUT /api/v1/admin/cpms-keys/:site_sfid/revoke` | `cpms::revoke_cpms_keys` |

## 3. 归属说明

- `CpmsSiteKeys` 等数据结构归 `cpms/model.rs`。
- CPMS 站点数据仍按省写入 `store_shards`。
- 匿名证书 RSA 盲签名直接归 `cpms::rsa_blind`,不再放在机构模块。
- CPMS 站点省域判断归 `cpms::scope`,不再放在通用 `scope` 目录。
- 公民绑定/状态扫码可以调用 `cpms::resolve_site_province_via_shard` 和
  `cpms::verify_sr25519_signature`。
- 不得再从 `sheng_admins::institutions` 引用 CPMS 功能。
