# CPMS_TECHNICAL

- 最后更新:2026-06-16
- 任务卡:
  - `memory/08-tasks/open/20260516-sfid-cpms-install-archive.md`
  - `memory/08-tasks/done/20260525-sfid-cpms-archive-simplify.md`
  - `memory/08-tasks/done/20260525-sfid-cpms-store.md`
  - `memory/08-tasks/done/20260530-sfid-admin-permission-step2.md`
  - `memory/08-tasks/done/20260531-sfid-permission-closeout.md`
  - `memory/08-tasks/open/20260615-cpms-sfid-birthplace-election-scope.md`

## 0. 模块边界

`sfid/backend/cpms/` 承接 SFID 侧的 CPMS 系统管理功能:

- 生成公安局 CPMS 的 `SFID_CPMS_V1 / INSTALL` 安装授权码。
- 保存 `install_secret / sfid_number` 授权状态，省市代码由 `sfid_number` 解码。
- 验证 CPMS 提交的 `SFID_CPMS_V1 / ARCHIVE` 档案码。
- 解 `geo_seal` 后为公民绑定流程提供公安局机构 `sfid_number`、居住地投票代码、出生地参选代码和市/镇精度。
- 查询、禁用、启用、吊销、删除 CPMS 授权。
- 按机构 `sfid_number` 反查对应 CPMS 授权记录。

该模块服务于公安局机构详情页,但不属于联邦管理员目录。CPMS 安装授权、安装码
重签发、禁用、启用、吊销、删除均只允许联邦管理员操作;市管理员不得操作
CPMS 授权治理。

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

写权限分级:

- 查询与 ARCHIVE 验真:联邦管理员或市管理员登录态 + SQL 层行政范围限定。
- 安装码签发、作废、重签发、禁用、启用、吊销、删除:必须先在
  `admins/actions.rs` 完成 Passkey + 当前联邦管理员冷钱包 sr25519 签名,
  再携带一次性 `x-sfid-security-grant` 调用本模块接口。
- 前端 `cpms/api.ts` 中禁用、启用、吊销、删除、重签发等重要操作的 API
  封装必须把安全授权声明为必填参数,不得通过可选 header 旁路类型检查。

## 3. 验真链路

1. 解析 `ARCHIVE`，强制 `proto=SFID_CPMS_V1`、`type=ARCHIVE`。
2. 在当前管理员省域内用已保存的 `install_secret` 尝试解 `geo_seal`。
3. 校验 `geo_seal.sfid_number` 与授权记录一致，并从 `sfid_number` 解码 CPMS 授权分区省市。
4. 校验 `geo_seal.election_scope_level` 与 `residence / birthplace` 代码精度一致：`PROVINCE` 只允许省，`CITY` 允许省市，`TOWN` 允许省市镇；居住省市必须与 CPMS 授权分区一致。
5. 校验 CPMS 本机签名；首次成功时把 `cpms_pubkey_hash / ACTIVE / USED`
   写入 `store_cpms` 主数据，后续只接受同一公钥。
6. 返回验真结果；正式绑定必须由 `citizens::binding::citizen_bind` 在 wuminapp 签名通过后完成。
7. 绑定流程检查 `archive_no / sfid_number / wallet_pubkey` 三者唯一，不再维护独立档案导入状态。

## 4. 归属说明

- `CpmsSiteKeys` 等数据结构归 `cpms/model.rs`。
- CPMS 授权主数据写入 `store_cpms` 模块快照表；`store/` 只保留进程内省分片缓存。
- SFID 启动时必须把 `store_cpms.cpms_site_keys` 恢复到 `sharded_store`，否则 ARCHIVE
  验真无法扫描到授权记录，会误报 `geo_seal cannot be decrypted`。
- CPMS 本机公钥绑定状态不得只写 `sharded_store`；任何 `cpms_pubkey_hash / status /
  install_token_status` 更新都必须先落 `store_cpms`，再覆盖运行缓存。
- ARCHIVE 验真入口为 `cpms::verify_cpms_archive_qr`，公民绑定复用同一入口。
- `VerifiedCpmsArchive.province_code / city_code` 表示 CPMS 授权分区；`residence_* / birth_* / election_scope_level` 表示后续投票区域判断所需代码，不得混用。
- CPMS 授权省域判断归 `cpms::scope`。
- 不得从管理员治理目录引用或复刻 CPMS handler。
