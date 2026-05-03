# SHENG_ADMINS_TECHNICAL

- 最后更新:2026-05-02
- 任务卡:
  - `memory/08-tasks/done/20260502-sfid-cpms-sheng目录整改.md`

## 0. 模块边界

`sfid/backend/sheng_admins/` 只负责省管理员和市管理员治理:

- 省管理员目录查询。
- 注册局-省级管理员页面的一主两备展示。
- 省管理员本人 signing seed 的自动加载、手动生成、更换。
- 市管理员 CRUD、启用/停用、删除。
- 43 省内置主管理员公钥与省份归属基线。

CPMS 系统安装、QR2 注册、QR3 匿名证书、站点状态治理已经迁到
`sfid/backend/cpms/`。不得再把 CPMS handler 放回 `sheng_admins`。

省管理员当前没有活跃 `chain_*.rs` 文件。后续只有“更换省管理员/主备交换”
需要和区块链交互时,才允许新增 `sheng_admins/chain_replace_admin.rs`。

## 1. 当前目录

```text
sfid/backend/sheng_admins/
├── mod.rs                    # 模块导出
├── catalog.rs                # 省管理员目录查询
├── operators.rs              # 市管理员 CRUD 与状态治理
├── province_admins.rs        # 43 省主管理员公钥、槽位模型、省份归属
├── roster.rs                 # 注册局-省级管理员页面一主两备展示
├── signing_cache.rs          # 省管理员 signing keypair 进程缓存
├── signing_keys.rs           # 本人 signing seed 自动加载、生成、更换接口
└── signing_seed_store.rs     # signing seed 加密持久化
```

## 2. API

| 端点 | 代码 | 说明 |
|---|---|---|
| `GET /api/v1/admin/sheng-admins` | `catalog::list_sheng_admins` | 省管理员目录查询,按登录作用域过滤 |
| `GET /api/v1/admin/sheng-admin/roster` | `roster::list_roster_admin` | 注册局页面一主两备展示 |
| `POST /api/v1/admin/sheng-signer/prepare` | `signing_keys::prepare` | 生成本人 signing seed 操作的扫码签名 payload |
| `POST /api/v1/admin/sheng-signer/submit` | `signing_keys::submit` | 校验本人签名后生成/更换本地 signing seed |
| `GET /api/v1/admin/operators` | `operators::list_operators` | 市管理员列表 |
| `POST /api/v1/admin/operators` | `operators::create_operator` | 新增市管理员 |
| `PUT /api/v1/admin/operators/:id` | `operators::update_operator` | 修改市管理员 |
| `DELETE /api/v1/admin/operators/:id` | `operators::delete_operator` | 删除市管理员 |
| `PUT /api/v1/admin/operators/:id/status` | `operators::update_operator_status` | 启用/停用市管理员 |

旧 `PUT /api/v1/admin/sheng-admins/:province` 本地替换入口已下架。正式更换省管理员
必须等待链上“更换省管理员/主备交换”能力对齐后,走本人签名和链上状态。

## 3. 权限规则

- `SHENG_ADMIN` 只能管理本省市管理员和本省页面展示。
- `SHI_ADMIN` 只能读取自己作用域内的省/市管理员信息。
- 省管理员 signing seed 只能由本人登录自动加载或本人手动生成/更换。
- 主管理员不得替备用管理员生成或更换 signing seed。
- 后端不得用本地 signing keypair 代替管理员账户私钥发起省管理员名册链上操作。

## 4. 状态来源

- 省管理员主管理员公钥来自 `province_admins.rs` 的内置 43 省基线。
- 备用管理员槽位当前等待后续链上更换/主备交换能力接入。
- signing seed 真私钥只落在 `signing_seed_store.rs` 的加密文件中。
- 页面展示用的 `signing_pubkey/signing_created_at` 由 `signing_keys.rs`
  写回管理员索引。

## 5. 已删除残留

- `chain_add_backup.rs`
- `chain_remove_backup.rs`
- `chain_activate_signer.rs`
- `chain_rotate_signer.rs`
- `chain_pending_signs.rs`
- `chain_roster_handler.rs`
- `chain_roster_query.rs`
- `bootstrap.rs`
- `signing_metadata.rs`
- `multisig.rs`
- `institutions.rs`
