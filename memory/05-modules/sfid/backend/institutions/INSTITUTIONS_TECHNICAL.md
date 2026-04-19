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
| POST | `/api/v1/institution/create` | 生成机构 sfid_id(**不上链**);私权两步式下 institution_name 可省 |
| GET | `/api/v1/institution/search-parents?q=xxx` | 法人机构模糊搜索(FFR 选择所属法人) |
| PATCH | `/api/v1/institution/:sfid_id` | 两步式第二步:更新机构名称/企业类型/所属法人 |
| POST | `/api/v1/institution/:sfid_id/account/create` | 创建账户并上链 register_sfid_institution |
| GET | `/api/v1/institution/list?category=…&province=…&city=…` | 按 scope 过滤的机构列表 |
| GET | `/api/v1/institution/:sfid_id` | 机构详情(含账户列表) |
| GET | `/api/v1/institution/:sfid_id/accounts` | 机构下账户列表 |
| DELETE | `/api/v1/institution/:sfid_id/account/:account_name` | 软删账户(不触链) |

## 创建者反查(2026-04-19 列表/详情改造)

`list_institutions` 和 `get_institution` 会把 `MultisigInstitution.created_by`(pubkey)通过 legacy store 的 `admin_users_by_pubkey` 反查出 `admin_name` + 角色枚举:

| 返回字段(JSON) | 来源 | 未命中处理 |
|---|---|---|
| `created_by_name` | `AdminUser.admin_name` | `null`(前端显"未知") |
| `created_by_role` | `"KEY_ADMIN"` / `"SHENG_ADMIN"` / `"SHI_ADMIN"` | `null` |

反查使用 `normalize_admin_pubkey` 统一规范化 pubkey(去 `0x` + 小写),避免大小写/前缀不一致错过匹配。公共辅助函数:`handler::resolve_created_by(state, pubkey)`。

前端仅在**私权机构列表**(category = PRIVATE_INSTITUTION)和**私权详情页**的"SFID 信息"板块展示该列/字段;公安局和公权机构列表暂不展示(公安局由 reconcile 批量生成无人类语义;公权下一步再做)。

## 清算行设置(2026-04-19 第四轮)

**资格**(任一):
- SFR 且 `sub_type == "JOINT_STOCK"`(股份公司)
- FFR 且 `parent_sfid_id` 指向的机构是 SFR 且其 `sub_type == "JOINT_STOCK"`

**数据**:
- `MultisigInstitution.is_clearing_bank: bool`(`#[serde(default, skip_serializing_if = Not::not)]`)
- `UpdateInstitutionInput.is_clearing_bank: Option<bool>`:Some(true)=开;Some(false)=关;None=不变
- `ParentInstitutionRow` 新增 `sub_type: Option<String>`(前端判 FFR 父机构是否 JOINT_STOCK)
- `InstitutionListRow.is_clearing_bank: bool`
- `service::CLEARING_BANK_NAMES = ["主账户", "费用账户"]`

**开启流程**(`handler::create_clearing_bank_accounts`):
1. `check_clearing_bank_eligible`:校验 a3 + sub_type(FFR 额外读父机构)
2. 遍历 2 个 name:
   - 存在且 `Registered` → 幂等跳过
   - 存在且 `Pending` → 拒绝 409
   - 存在且 `Failed` 或不存在 → 写 Pending → 调 `chain::submit_register_account` → 成功写 Registered+地址,失败写 Failed 并返回错误(已成功的账户保留)
3. 2 个全部成功 → `is_clearing_bank = true`

**关闭流程**(`handler::ensure_clearing_bank_accounts_absent`):
- 前置:该机构 `multisig_accounts` 中 `"主账户"` / `"费用账户"` 两条均不存在
- 仍存在 → 拒绝 409,提示用户走链上 `duoqian-manage-pow::propose_close` 投票注销后 `DELETE /institution/:sfid_id/account/:name` 软删
- 通过 → `is_clearing_bank = false`

**锁定**:`existing.is_clearing_bank == true` 期间,`update_institution` 拒绝修改 `sub_type` / `parent_sfid_id`(必须先关清算行)

**UI**(`PrivateInstitutionLayout.tsx`):
- 仅 `eligibleForClearingBank` 为 true 时显示 Switch(SFR 依 sub_type 实时/FFR 依 selectedParent.sub_type)
- 已开启 + 账户仍存在 → Switch disabled,提示"关闭前需链上注销并软删 X 账户"
- 已开启 + 账户已全部删除 → Switch 可关
- 关闭态 → Switch 可开
- 开启期间 `sub_type` Select / `parent_sfid_id` AutoComplete 前端也 disabled

## FFR 所属法人(2026-04-19 第三轮)

**铁律**:非法人(FFR)**必须**挂在一个法人机构(SFR 私法人或 GFR 公法人)下。

- 数据模型:`MultisigInstitution.parent_sfid_id: Option<String>`
- 仅 `a3 == "FFR"` 可设置;SFR/GFR 传值报 400
- `update_institution` 校验 target sfid_id 存在且 `a3 ∈ {SFR, GFR}`
- FFR `needsCompletion` 包含 `!parent_sfid_id` → 未设置时"+ 新建账户"禁用
- 后端新增 `GET /api/v1/institution/search-parents?q=xxx` — 全国范围 SFR/GFR 模糊(匹配 sfid_id 子串 OR institution_name 子串),仅返回已命名机构,最多 20 条
- 前端详情页用 `AutoComplete` 驱动,FFR 专显示"所属法人"Form.Item,位于"机构名称"下方

## 两步式创建(2026-04-19 改造)

私权(SFR/FFR)采用两步式,SFID 即定不可变,详情可变:

- **第一步** `POST /create`:只生成 SFID,`institution_name` / `sub_type` **不接受**
- **第二步** `PATCH /:sfid_id`:详情页保存 `institution_name`(全国唯一查重) + `sub_type`(与 P1 联动)
- `sub_type` 联动:`P1=0` 必须 `NON_PROFIT`;`P1=1` 必须 `SOLE_PROPRIETORSHIP` / `PARTNERSHIP` / `LIMITED_LIABILITY` / `JOINT_STOCK`
- 未命名机构无法新建账户(前端按钮禁用 + 提示)

公权(GFR/公安局)仍走单步创建(下一步再做两步式改造)。

所有 list API 都经 `scope::filter_by_scope` 按 KeyAdmin / ShengAdmin / ShiAdmin 角色自动过滤。

## 关键铁律

- **account_name 就是链上 name** — 同 sfid 下不能重名(链端 `SfidRegisteredAddress::insert` 有 `ensure!(!contains_key)` 硬约束)
- **institution_name 不进链** — 仅 sfid 系统展示用,类型为 `Option<String>`(两步式)
- **创建账户前必须调 `service::ensure_account_name_unique`** — 避免白交链上手续费
- **类别持久化** — `InstitutionCategory` 枚举显式存(`PublicSecurity` / `GovInstitution` / `PrivateInstitution`)
- **机构代码清单** — GFR 移除 `CB`;SFR/FFR 移除 `CH`(2026-04-19 清理,不保留兼容)

详见 `feedback_institutions_two_layer.md`。

## 迁移

老 `multisig_sfid_records` 由 `app_core/runtime_ops::migrate_legacy_multisig_to_two_layer` 幂等
迁移到两层结构。老 cache key 保留,作为回滚兜底。

## 历史

- 2026-04-08 任务卡 2 落地(`08-tasks/done/20260408-sfid-机构账户两层模型-任务卡2.md`)
