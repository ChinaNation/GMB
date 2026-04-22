# institutions/ — 机构/账户两层数据模型

## 定位

任务卡 2 引入的新模块,对齐链端 `DuoqianManagePow::register_sfid_institution(sfid_id, account_name, …)` 的
`SfidRegisteredAddress::<T>(sfid_id, account_name) → duoqian_address` DoubleMap 语义。

**2026-04-21 起**:链端按 **角色** 分流地址派生(`InstitutionAccountRole` 枚举),`register_sfid_institution` 收到的 `account_name` 由 `role_from_account_name` 翻译:

- `"主账户"`   → `Role::Main`  → `blake2_256(DUOQIAN_V1 ‖ 0x00 ‖ ss58 ‖ sfid_id)`(preimage 不含 account_name)
- `"费用账户"` → `Role::Fee`   → `blake2_256(DUOQIAN_V1 ‖ 0x01 ‖ ss58 ‖ sfid_id)`(preimage 不含 account_name)
- 其他非空    → `Role::Named(account_name)` → `blake2_256(DUOQIAN_V1 ‖ 0x05 ‖ ss58 ‖ sfid_id ‖ account_name)`

SFID 后端侧 API 仍然只传 `account_name` 字符串,链端自动路由到对应 op_tag。`SfidRegisteredAddress` 双 map 的 key 结构不变。**派生细节完全是链端内部关注点**,sfid 后端/前端只认 `account_name` 字符串。

sfid 系统内部把多签机构拆成两层:

- **层 1 机构** `MultisigInstitution`:每个 `sfid_id` 唯一,存机构展示信息(`institution_name` 等),**不进链**
- **层 2 账户** `MultisigAccount`:以 `(sfid_id, account_name)` 复合 key,`account_name` 是**链上 extrinsic 的参数**,一个机构下可挂多个

## 文件结构

```
backend/src/institutions/
├── mod.rs       — pub use 聚合
├── model.rs     — MultisigInstitution / MultisigAccount / DTO / HasProvinceCity impl
├── store.rs     — cache entry 读写层 (multisig_institutions / multisig_accounts)
├── service.rs   — 校验 / 分类 / 同 sfid 唯一性 / DEFAULT_ACCOUNT_NAMES
├── derive.rs    — DUOQIAN_V1 本地地址派生(与链端 derive_institution_address 字节对齐)
├── chain.rs     — submit_register_account,复用 sheng_admins 的 submit_register_sfid_institution_extrinsic
└── handler.rs   — HTTP handler
```

## HTTP 端点

| 方法 | 路径 | 功能 |
|---|---|---|
| POST | `/api/v1/institution/create` | 生成机构 sfid_id(**不上链**);创建时自动插入 2 条默认 Inactive 账户 |
| GET | `/api/v1/institution/search-parents?q=xxx` | 法人机构模糊搜索(FFR 选择所属法人) |
| PATCH | `/api/v1/institution/:sfid_id` | 两步式第二步:更新机构名称 / 企业类型 / 所属法人 |
| POST | `/api/v1/institution/:sfid_id/account/create` | **只登记本地 Inactive,不上链**;同 sfid 下 account_name 唯一 |
| POST | `/api/v1/institution/:sfid_id/account/:account_name/activate` | **激活(推链)**:Inactive/Failed → Pending → Registered/Failed |
| GET | `/api/v1/institution/list?category=…&province=…&city=…&q=…` | 按 scope 过滤的机构列表;q 子串匹配 sfid_id/institution_name |
| GET | `/api/v1/institution/:sfid_id` | 机构详情(含账户列表) |
| GET | `/api/v1/institution/:sfid_id/accounts` | 机构下账户列表 |
| DELETE | `/api/v1/institution/:sfid_id/account/:account_name` | 软删账户(不触链);默认账户"主账户"/"费用账户"禁止删除(409) |
| GET | `/api/v1/app/clearing-banks/search` | wuminapp 公开搜索:返回主账户已激活上链(Registered)的机构 |

所有 list API 都经 `scope::filter_by_scope` 按 KeyAdmin / ShengAdmin / ShiAdmin 角色自动过滤。

## 账户生命周期(2026-04-21 统一两步激活模式)

### 状态机

`MultisigChainStatus` 四个状态(默认 `Inactive`):

```
Inactive  ──点"激活"──▶  Pending  ──成功──▶  Registered
                                  └──失败──▶  Failed ──点"重试激活"──▶  Pending
```

前端 UI 直接映射:`未激活` / `激活中` / `已激活` / `激活失败(可重试)`。

### 默认账户

`service::DEFAULT_ACCOUNT_NAMES = ["主账户", "费用账户"]`。

**所有机构**(公权 / 私权 / 公安局)创建时,后端自动插入这两条 `Inactive` 账户记录:

- `create_institution`:成功后调 `insert_default_accounts_best_effort(sfid_id, province, actor)` — 分片 + legacy 双写
- 公安局 `reconcile_public_security_for_province`:新增机构时调 `insert_default_accounts_into_legacy(store, sfid_id, actor)` — 只写 legacy(reconcile 是同步函数);**启动期** `main.rs` runtime 起来后调 `sync_public_security_to_sharded` 幂等同步到 sharded_store,同时给老记录补齐缺失的默认账户

**所有账户的 `duoqian_address` 在创建那一刻本地即刻派生**,无需等激活上链 — 链端 `DUOQIAN_V1` 公式是确定性的,`(sfid_id, account_name)` 一旦确定,地址就确定。创建时就填入 `MultisigAccount.duoqian_address`,前端账户列表立即显示 SS58 截断。

默认账户**不可删除**:`delete_account` 对这两个 `account_name` 直接返回 409。链上注销需走 `duoqian-manage-pow::propose_close` 投票流程(注销后链上 `DuoqianAccounts` 被 remove,sfid 本地记录保留作为审计)。

### 手工账户

管理员在详情页"+ 新建账户"创建其他账户,走 `create_account`,同样**只登记本地 Inactive,不自动上链**。管理员需要在账户列表点"激活"才触发 `submit_register_account` 推链。

### 激活流程(`activate_account`)

前置校验:
- `Registered` → 409 "已激活"
- `Pending` → 409 "正在激活中"
- `Inactive` / `Failed` → 继续

流程:
1. 写 `Pending` 状态(分片 + legacy 双写)
2. 调 `chain::submit_register_account(state, ctx, sfid_id, account_name)` — 链上 extrinsic 为 `register_sfid_institution(sfid_id, account_name, nonce, sig, signing_province)`
3. **一致性断言**:receipt.duoqian_address 必须等于本地 `derive::derive_duoqian_address(sfid_id, account_name)` 结果;不等即返回 500(说明 domain / op_tag / ss58 对不上)
4. 成功:写 `Registered` + `chain_tx_hash` + `chain_block_number`(地址已经在创建时就填好了,这里也会收到链上同样的值)
5. 失败:写 `Failed`,允许点"重试激活"

## 创建者反查

`list_institutions` 和 `get_institution` 会把 `MultisigInstitution.created_by`(pubkey)通过 legacy store 的 `admin_users_by_pubkey` 反查出管理员姓名 + 角色:

| 返回字段(JSON) | 来源 | 未命中/空值 |
|---|---|---|
| `created_by_name` | `AdminUser.admin_name`(trim 非空) | `null`(前端显 `未知` 或仅角色) |
| `created_by_role` | `"KEY_ADMIN"` / `"SHENG_ADMIN"` / `"SHI_ADMIN"` | `null` |

反查使用 `normalize_admin_pubkey` 统一规范化 pubkey(去 `0x` + 小写)。辅助函数:`handler::resolve_created_by`。

前端**仅在**私权机构列表(category=PRIVATE_INSTITUTION)和**私权详情页**的"SFID 信息"板块展示;公安局/公权暂不展示(公安局由 reconcile 批量生成无人类语义;公权下一步再做)。

## FFR 所属法人

**铁律**:非法人(FFR)必须挂在一个法人机构(SFR 私法人或 GFR 公法人)下。

- 数据:`MultisigInstitution.parent_sfid_id: Option<String>`
- 仅 `a3 == "FFR"` 可设置;SFR/GFR 传值 400
- `update_institution` 校验 target sfid_id 存在且 `a3 ∈ {SFR, GFR}`
- FFR `needsCompletion` 包含 `!parent_sfid_id` → 未设置时"+ 新建账户"按钮禁用
- `GET /institution/search-parents` 全国模糊(匹配 sfid_id 子串或 institution_name 子串),返回已命名的 SFR/GFR,最多 20 条,含 `sub_type`
- 前端 FFR 详情页 `AutoComplete` 由搜索图标触发请求

## 两步式创建

私权(SFR/FFR)采用两步式,SFID 即定不可变,详情可变:

- **第一步** `POST /create`:只生成 SFID;`institution_name` / `sub_type` 不接受(私权),自动补 2 条默认账户
- **第二步** `PATCH /:sfid_id`:详情页保存 `institution_name`(全国唯一查重)+ `sub_type`(与 P1 联动)+ `parent_sfid_id`(FFR 必填)
- `sub_type` 联动:`P1=0` 必须 `NON_PROFIT`;`P1=1` 必须 `SOLE_PROPRIETORSHIP` / `PARTNERSHIP` / `LIMITED_LIABILITY` / `JOINT_STOCK`
- 未命名机构无法新建账户(前端按钮禁用 + 提示)

公权(GFR/公安局)仍走单步创建(下一步再做两步式改造)。

## wuminapp 公开搜索

`GET /api/v1/app/clearing-banks/search?keyword=xxx&province=xxx&city=xxx&page=&size=`:
- 无鉴权
- 返回条件:**主账户已上链注册**(`chain_status == Registered`)的机构
- 关键字在 `sfid_id` / `institution_name` 子串匹配(大小写不敏感)
- 返回 `sfid_id` / `institution_name` / `a3` / `province` / `city` / `main_account` / `fee_account` 地址

## 关键铁律

- **`account_name` 就是链上 `register_sfid_institution` 的参数** — 同 sfid 下唯一(链端 `SfidRegisteredAddress::insert` 硬约束)
- **`institution_name` 不进链** — 仅 sfid 系统展示用,`Option<String>`(两步式未命名为 None)
- **默认账户不可删** — 主账户 / 费用账户每家机构必有两条
- **创建不自动上链** — 所有账户默认 `Inactive`,需要显式"激活"
- **地址派生是链端内部细节** — sfid 层面不做预览,避免和链端 `InstitutionAccountRole` 派生公式漂移
- **机构代码清单** — GFR 移除 `CB`;SFR/FFR 移除 `CH`(2026-04-19 清理,不留兼容)
- **类别持久化** — `InstitutionCategory` 枚举显式存(`PublicSecurity` / `GovInstitution` / `PrivateInstitution`)

详见 `feedback_institutions_two_layer.md`。

## 迁移

老 `multisig_sfid_records` 由 `app_core/runtime_ops::migrate_legacy_multisig_to_two_layer` 幂等迁移到两层结构。老 cache key 保留,作为回滚兜底。

## 历史

- 2026-04-08 任务卡 2 落地(`08-tasks/done/20260408-sfid-机构账户两层模型-任务卡2.md`)
- 2026-04-19 两步式私权创建、机构代码清理、创建者反查、FFR 所属法人(`08-tasks/done/20260419-sfid-机构两步式创建.md`)
- 2026-04-21 统一两步激活模式:所有机构创建时自动生成主账户/费用账户(Inactive),手工账户也走 Inactive;清算行概念彻底废弃
