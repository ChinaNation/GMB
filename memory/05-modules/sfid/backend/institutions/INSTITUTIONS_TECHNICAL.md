# institutions/ — 机构/账户两层数据模型

## 定位

任务卡 2 引入的新模块,对齐链端 `DuoqianManagePow::register_sfid_institution(sfid_id, name, …)` 的
`SfidRegisteredAddress::<T>(sfid_id, name) → duoqian_address` DoubleMap 语义。

sfid 系统内部把多签机构拆成两层:

- **层 1 机构** `MultisigInstitution`:每个 `sfid_id` 唯一,存机构展示信息(institution_name 等),**不进链**
- **层 2 账户** `MultisigAccount`:以 `(sfid_id, account_name)` 复合 key,`account_name` 是**链上 name 参数**,一个机构下可挂多个

## 文件结构

```
backend/src/institutions/
├── mod.rs       — pub use 聚合
├── model.rs     — MultisigInstitution / MultisigAccount / DTO / HasProvinceCity impl
├── store.rs     — cache entry 读写层 (multisig_institutions / multisig_accounts)
├── service.rs   — 校验 / 分类 / 同 sfid 唯一性
├── chain.rs     — submit_register_account,复用 sheng_admins 的 submit_register_sfid_institution_extrinsic
└── handler.rs   — HTTP handler (6 个端点)
```

## HTTP 端点

| 方法 | 路径 | 功能 |
|---|---|---|
| POST | `/api/v1/institution/create` | 生成机构 sfid_id(**不上链**) |
| POST | `/api/v1/institution/:sfid_id/account/create` | 创建账户并上链 register_sfid_institution |
| GET | `/api/v1/institution/list?category=…&province=…&city=…` | 按 scope 过滤的机构列表 |
| GET | `/api/v1/institution/:sfid_id` | 机构详情(含账户列表) |
| GET | `/api/v1/institution/:sfid_id/accounts` | 机构下账户列表 |
| DELETE | `/api/v1/institution/:sfid_id/account/:account_name` | 软删账户(不触链) |

所有 list API 都经 `scope::filter_by_scope` 按 KeyAdmin / ShengAdmin / ShiAdmin 角色自动过滤。

## 关键铁律

- **account_name 就是链上 name** — 同 sfid 下不能重名(链端 `SfidRegisteredAddress::insert` 有 `ensure!(!contains_key)` 硬约束)
- **institution_name 不进链** — 仅 sfid 系统展示用
- **创建账户前必须调 `service::ensure_account_name_unique`** — 避免白交链上手续费
- **类别持久化** — `InstitutionCategory` 枚举显式存(`PublicSecurity` / `GovInstitution` / `PrivateInstitution`)

详见 `feedback_institutions_two_layer.md`。

## 迁移

老 `multisig_sfid_records` 由 `app_core/runtime_ops::migrate_legacy_multisig_to_two_layer` 幂等
迁移到两层结构。老 cache key 保留,作为回滚兜底。

## 历史

- 2026-04-08 任务卡 2 落地(`08-tasks/done/20260408-sfid-机构账户两层模型-任务卡2.md`)
