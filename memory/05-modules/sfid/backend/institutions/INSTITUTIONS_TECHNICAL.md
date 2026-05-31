# institutions/ — SFID 机构与账户名称模型

- 最后更新:2026-05-31
- 任务卡:
  - `memory/08-tasks/done/20260530-sfid-admin-permission-step2.md`
  - `memory/08-tasks/done/20260531-sfid-permission-closeout.md`

> 中文注释(2026-05-16):`institutions` 只保留机构身份、账户名称、资料库和
> 机构对区块链/钱包提供查询的能力。CPMS 安装授权、ARCHIVE 档案验真和
> 授权状态治理归 `sfid/backend/cpms/`,不挂在机构模块下。

## 定位

SFID 系统只负责机构身份和账户名称:

- `sfid_number`:创建后不可变。
- `institution_name`:机构展示名称,创建后可设置和修改。
- `account_name`:DUOQIAN_V1 协议定义的账户名称,与 `sfid_number` 一起派生链上地址。

SFID 系统不直接发起链上机构注册、链上注销、管理员阈值签名和账户激活。
链端需要注册机构时,应通过 `institutions::chain_duoqian_info` 公开接口 pull
`sfid_number / institution_name / account_names[]` 注册信息凭证。

后台权限:

- 查看机构、账户、文档:登录态 + 省/市 scope。
- 创建机构、修改机构、新增本地账户、上传文档:`PASSKEY` 写操作,
  必须先通过 Passkey 换取一次性 `x-sfid-security-grant`。
- 删除本地账户、删除文档:`PASSKEY_CHALLENGE` 写操作,
  必须通过 Passkey + 当前管理员冷钱包挑战签名换取一次性 `x-sfid-security-grant`。
- 省级管理员可操作本省所有机构;市级管理员只能操作本市业务机构。
- 机构资料库所有列表、上传、下载、删除入口必须先确认机构存在,再校验省/市
  scope;不存在对应机构的孤儿文档一律按机构不存在处理,不得绕过 scope。
- 公安局 CPMS 安装授权治理不在 `institutions` 内实现,统一归 `cpms` 模块且只允许省级管理员。

开发期按彻底改造执行，当前文档只描述现行接口和状态机。

## 地址规则

链端按 `account_name` 翻译账户角色:

- `"主账户"` → `Role::Main` → `DUOQIAN_V1 || 0x00 || ss58 || sfid_number`
- `"费用账户"` → `Role::Fee` → `DUOQIAN_V1 || 0x01 || ss58 || sfid_number`
- 其他账户名称 → `Role::Named(account_name)` → `DUOQIAN_V1 || 0x05 || ss58 || sfid_number || account_name`

SFID 后端只传和保存 `account_name` 字符串,不拆字段、不改名。

## 状态模型

机构链上状态 `InstitutionChainStatus`:

```text
NOT_REGISTERED      SFID 已创建,链上未注册
PENDING_REGISTER    链上注册中
REGISTERED          链上已注册
REVOKED_ON_CHAIN    链上已注销
```

账户链上状态 `MultisigChainStatus`:

```text
NOT_ON_CHAIN        SFID 已创建账户名称,未上链
PENDING_ON_CHAIN    账户上链中
ACTIVE_ON_CHAIN     账户已上链
REVOKED_ON_CHAIN    账户已链上注销
```

规则:

- 创建机构时,机构为 `NOT_REGISTERED`。
- 创建机构时自动生成 `主账户`、`费用账户`,两者为 `NOT_ON_CHAIN`。
- 新增账户创建后为 `NOT_ON_CHAIN`。
- SFID 后台没有手动激活按钮。
- 只有链上同步接口能把机构改成 `REGISTERED` / `REVOKED_ON_CHAIN`。
- 只有链上同步接口能把账户改成 `ACTIVE_ON_CHAIN` / `REVOKED_ON_CHAIN`。

## 删除规则

默认账户:

- `主账户`、`费用账户`永远不能单独删除。
- 只有删除整个 SFID 机构时,默认账户才随机构一起删除。

新增账户:

- `NOT_ON_CHAIN`:可以删除。
- `PENDING_ON_CHAIN`:不能删除、不能停用、不能归档。
- `ACTIVE_ON_CHAIN`:不能删除、不能停用、不能归档。
- `REVOKED_ON_CHAIN`:可以删除。

SFID 不能单方面删除仍在链上的账户名称。

## HTTP 端点

后台管理:

| 方法 | 路径 | 功能 |
|---|---|---|
| POST | `/api/v1/institution/create` | 创建 SFID 机构,自动创建默认账户,不上链 |
| GET | `/api/v1/institution/search-parents?q=xxx` | 法人机构模糊搜索 |
| PATCH | `/api/v1/institution/:sfid_number` | 更新机构名称 / 企业类型 / 所属法人 |
| POST | `/api/v1/institution/:sfid_number/account/create` | 新增账户名称,不上链 |
| GET | `/api/v1/institution/list?category=...&province=...&city=...&q=...&cursor=...` | 按权限范围精确查询机构，返回游标分页对象 |
| GET | `/api/v1/institution/:sfid_number` | 机构详情,含账户列表 |
| GET | `/api/v1/institution/:sfid_number/accounts` | 机构账户列表 |
| DELETE | `/api/v1/institution/:sfid_number/account/:account_name` | 删除允许删除的新增账户名称 |
| GET | `/api/v1/institution/:sfid_number/documents` | 查看机构资料库文档 |
| POST | `/api/v1/institution/:sfid_number/documents` | 上传机构资料库文档 |
| GET | `/api/v1/institution/:sfid_number/documents/:doc_id/download` | 下载机构资料库文档 |
| DELETE | `/api/v1/institution/:sfid_number/documents/:doc_id` | 删除机构资料库文档 |

管理员端机构列表不默认返回全量数据。私权机构、公权机构和公安局列表共用
`sfid_institutions / sfid_institution_accounts` 行表；管理员输入机构 SFID 或机构名称后，
后端按当前省/市权限返回 `{ items, page_size, next_cursor, has_more }`。

区块链软件公开查询:

| 方法 | 路径 | 功能 |
|---|---|---|
| GET | `/api/v1/app/institutions/search?q=xxx&limit=20` | 按 SFID 或机构名称搜索机构 |
| GET | `/api/v1/app/institutions/:sfid_number` | 读取机构详情与最新机构名称,不带注册凭证 |
| GET | `/api/v1/app/institutions/:sfid_number/registration-info` | 链端注册信息凭证,业务字段仅 `sfid_number/institution_name/account_names[]` |
| GET | `/api/v1/app/institutions/:sfid_number/accounts` | 读取账户列表、地址、链上状态、删除许可 |
| GET | `/api/v1/app/clearing-banks/search` | wuminapp 搜索已上链且已加入清算网络的清算行 |
| GET | `/api/v1/app/clearing-banks/eligible-search` | 桌面节点搜索具备清算行资格的 SFID 机构 |

历史 `POST /api/v1/app/institutions/:sfid_number/chain-sync` 已下架,不得作为新功能入口。
后续如需回写链上事实,必须另起任务设计可信同步边界。

## 前端规则

- 账户列表只显示链上状态,不提供“激活”或“重试激活”按钮。
- 非默认账户只有 `NOT_ON_CHAIN` / `REVOKED_ON_CHAIN` 时显示删除按钮。
- `ACTIVE_ON_CHAIN` / `PENDING_ON_CHAIN` 显示不可删除。
- 机构详情显示机构链上状态。

## 文件结构

```text
backend/institutions/
├── mod.rs                  — 模块声明与聚合导出
├── model.rs                — 机构、账户、资料库、请求/响应 DTO
├── store.rs                — 机构/账户 store 读写层
├── service.rs              — 校验、分类、默认账户、删除许可规则
├── handler.rs              — 机构本地 HTTP handler
├── derive.rs               — DUOQIAN_V1 本地地址派生
└── chain_duoqian_info.rs   — 机构提供给区块链/钱包查询的 DTO 与 handler
```

## 技术方案收口项

- 文档更新:本文件记录 SFID 链上状态真源、API 和删除规则。
- 注释完善:`handler.rs` / `service.rs` / `model.rs` 已补充状态同步和删除规则中文注释。
- 残留清理:移除后台手动激活路由、前端激活按钮、后端直接推链文件和无效状态机。

## 历史

- 2026-04-08:引入机构/账户两层模型。
- 2026-04-19:私权机构两步式创建。
- 2026-04-21:DUOQIAN_V1 账户名称派生规则收口。
- 2026-04-29:SFID 状态真源改为链上同步,后台不再手动激活账户。
- 2026-05-02:机构本地管理与链端查询收口为当前 7 文件结构,历史角色说明已清理。
