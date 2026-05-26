# CPMS_TECHNICAL

- 最后更新:2026-05-25
- 任务卡:
  - `memory/08-tasks/open/20260516-sfid-cpms-install-archive.md`
  - `memory/08-tasks/done/20260525-sfid-cpms-archive-simplify.md`
  - `memory/08-tasks/done/20260525-sfid-cpms-store.md`

## 0. 模块边界

`sfid/backend/cpms/` 承接 SFID 侧的 CPMS 系统管理功能:

- 生成公安局 CPMS 的 `SFID_CPMS_V1 / INSTALL` 安装授权码。
- 保存 `install_secret / sfid_number` 授权状态，省市代码由 `sfid_number` 解码。
- 验证 CPMS 提交的 `SFID_CPMS_V1 / ARCHIVE` 档案码。
- 解 `geo_seal` 后为公民绑定流程提供省、市、公安局机构 `sfid_number`。
- 查询、禁用、启用、吊销、删除 CPMS 授权。
- 按机构 `sfid_number` 反查对应 CPMS 授权记录。

该模块服务于公安局机构详情页,但不属于省管理员目录。省管理员只是调用这些接口的授权角色。

## 1. 当前目录

```text
sfid/backend/cpms/
├── mod.rs        # 模块导出
├── model.rs      # CPMS 安装授权、INSTALL/ARCHIVE DTO 和验真结果
├── handler.rs    # 安装码签发、ARCHIVE 验真和授权状态治理
└── scope.rs      # CPMS 授权是否属于当前管理员省域
```

## 2. API

| 端点 | 代码 | 说明 |
|---|---|---|
| `GET /api/v1/admin/cpms-keys` | `cpms::list_cpms_keys` | 列出本省或全局 CPMS 授权 |
| `GET /api/v1/admin/cpms-keys/by-institution/:sfid_number` | `cpms::get_cpms_site_by_institution` | 按公安局机构 SFID 查询授权 |
| `POST /api/v1/admin/cpms-keys/sfid/generate` | `cpms::generate_cpms_install_qr` | 签发 INSTALL 安装码 |
| `POST /api/v1/admin/cpms/archive/verify` | `cpms::archive_verify` | 只验真 ARCHIVE 档案码，不产生正式绑定 |
| `DELETE /api/v1/admin/cpms-keys/:sfid_number` | `cpms::delete_cpms_keys` | 删除未激活授权 |
| `POST /api/v1/admin/cpms-keys/:sfid_number/revoke-token` | `cpms::revoke_install_token` | 作废未使用安装授权 |
| `POST /api/v1/admin/cpms-keys/:sfid_number/reissue` | `cpms::reissue_install_token` | 重新签发 INSTALL |
| `PUT /api/v1/admin/cpms-keys/:sfid_number/disable` | `cpms::disable_cpms_keys` | 暂停接收该授权签发的档案码 |
| `PUT /api/v1/admin/cpms-keys/:sfid_number/enable` | `cpms::enable_cpms_keys` | 启用已禁用授权 |
| `PUT /api/v1/admin/cpms-keys/:sfid_number/revoke` | `cpms::revoke_cpms_keys` | 吊销授权 |

## 3. 验真链路

1. 解析 `ARCHIVE`，强制 `proto=SFID_CPMS_V1`、`type=ARCHIVE`。
2. 在当前管理员省域内用已保存的 `install_secret` 尝试解 `geo_seal`。
3. 校验 `geo_seal.sfid_number` 与授权记录一致，并从 `sfid_number` 解码省市。
4. 校验 CPMS 本机签名；首次成功时绑定 `cpms_pubkey_hash`，后续只接受同一公钥。
5. 返回验真结果；正式绑定必须由 `citizens::citizen_bind(bind_archive)` 在已有钱包地址的记录上完成。
6. 绑定流程检查 `ano / sfid_code / wallet_pubkey` 三者唯一，不再维护独立档案导入状态。

## 4. 归属说明

- `CpmsSiteKeys` 等数据结构归 `cpms/model.rs`。
- CPMS 授权主数据写入 `store_cpms` 模块快照表；`store_shards/` 只保留进程内省分片缓存。
- ARCHIVE 验真入口为 `cpms::verify_cpms_archive_qr`，公民绑定复用同一入口。
- CPMS 授权省域判断归 `cpms::scope`。
- 不得再从 `sheng_admins` 引用 CPMS handler。
