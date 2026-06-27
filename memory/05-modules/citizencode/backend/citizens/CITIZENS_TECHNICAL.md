# CITIZENS 模块技术文档

- 最后更新:2026-06-16
- 任务卡:
  - `memory/08-tasks/done/20260530-cid-admin-permission-step2.md`

## 1. 模块定位

- 路径：`citizencode/backend/citizens`
- 职责：承载注册局直接录入公民、公民电子护照绑定状态、公民投票凭证签发和联合投票人口快照凭证签发。
- 电子护照边界：注册局直接录入 `cid_number / citizen_status / voting_eligible / valid_from / valid_until / wallet_address / wallet_pubkey / wallet_sig_alg` 以及居住地、出生地和 `election_scope_level`；CID 验签通过后直接写入本地绑定结果并向 CitizenApp 状态接口返回。

## 2. 模块结构

- `admin_entry.rs`
  - `admin_create_citizen`：注册局管理员直接录入公民并发放公民护照身份。
- `vote.rs`
  - `app_myid_status`：CitizenApp 查询电子护照绑定状态。
- `chain_vote.rs`
  - `app_vote_credential`：公民投票凭证签发接口。
- `chain_joint_vote.rs`
  - `app_voters_count`：联合投票人口快照凭证签发接口。
- `model.rs`
  - 公民电子护照记录、`bind_status`、绑定 DTO、状态扫码 QR 载荷。
- `handler.rs`
  - `admin_list_citizens`：后台公民精确查询和游标分页。
  - `admin_search_legal_representative_citizens`：机构法定代表人候选公民查询;范围由目标机构上下文传入,不按登录管理员辖区硬切。
  - `public_identity_search`：公开身份查询。
- `mod.rs`
  - 子模块注册入口。

## 3. 路由接线

- `POST /api/v1/admin/citizens` -> `citizens::admin_entry::admin_create_citizen`
- `GET  /api/v1/app/myid/status?wallet_address=<walletAddress>` -> `citizens::vote::app_myid_status`
- `POST /api/v1/app/vote/credential` -> `citizens::chain_vote::app_vote_credential`
- `GET  /api/v1/app/voters/count` -> `citizens::chain_joint_vote::app_voters_count`
- `GET  /api/v1/admin/citizens` -> `citizens::handler::admin_list_citizens`
- `GET  /api/v1/admin/citizens/legal-representatives` -> `citizens::handler::admin_search_legal_representative_citizens`
- `GET  /api/v1/public/identity/search` -> `citizens::handler::public_identity_search`

## 4. 依赖与边界

- 依赖：
  - `admins::actions::require_admin_security_grant`：确认 `PASSKEY` 写操作已经通过 Passkey。
  - `login::parse_sr25519_pubkey_bytes`：解析 CitizenApp 钱包公钥。
  - 全局公共能力：鉴权、审计、状态存储。
- 边界：
  - 电子护照身份必须由注册局管理员直接录入。
  - 绑定必须使用 CitizenApp 对 CID challenge 的 sr25519 签名。
- `citizens` 不实现投票流程；公民投票只调用投票凭证签发接口。
- 公民 DTO 放 `citizens/model.rs`，不得塞入全局 `models`。

## 5. 关键一致性约束

- 三端字段统一：`citizen_status / voting_eligible / vote_status / identity_status / valid_from / valid_until / status_updated_at / wallet_address / wallet_pubkey / wallet_sig_alg / cid_number / bind_status / residence_* / birth_* / election_scope_level`；前端展示范围时必须使用 CID 行政区真源解析出的名称，不展示行政区代码。
- `bind_status` 只表达电子护照绑定状态：`PENDING / BOUND`；`identity_status` 表达身份 ID 当前有效状态；`vote_status` 由 `citizen_status + voting_eligible` 计算。
- `citizen_status` 当前只允许 `NORMAL / REVOKED`；`REVOKED` 表示注册局注销，必须对应 `voting_eligible=false`。
- 后台公民精确查询和直接录入均按管理员省/市 scope 过滤:
  联邦注册局机构管理员只看本省,市注册局机构管理员只操作本市。
- 管理员端公民查询不默认返回任何全量列表；必须输入身份ID、投票账户地址或投票账户公钥，后端返回 `{ items, page_size, next_cursor, has_more }`。
- CID 公民模块不保存公民姓名，任何公民检索都不得按姓名匹配。
- `citizens` 是管理员浏览器查询用分区表；直接录入和绑定完成后同步写入，`cid_number / wallet_pubkey` 二者一对一由公民录入流程强制。
- 公民创建/绑定时必须写入 `province_code / city_code`;该归属来自执行创建的注册局管理员 scope。
- 公民创建/绑定时必须另写 `residence_province_code / residence_city_code / residence_town_code / birth_province_code / birth_city_code / birth_town_code / election_scope_level`。投票范围按居住地判断，参选范围按出生地判断，业务模块只提供字段给投票引擎，不自行实现投票流程。
- CID 管理端公民详情只展示 `投票范围` 和 `参选范围`：投票范围 = 全国 + 居住省 + 按 `election_scope_level` 展开的居住市/镇；参选范围 = 全国 + 出生省 + 按 `election_scope_level` 展开的出生市/镇；不得展示 `residence_*` 或 `birth_*` 代码字段。
- `election_scope_level=PROVINCE` 时市镇字段为空；`CITY` 时市有值、镇为空；`TOWN` 时市镇都有值。业务模块不得据此自行实现投票流程，只能把字段提供给投票引擎使用。
- 法定代表人候选搜索是机构创建/编辑的辅助查询,按 `subjects` 模块推导出的目标机构 scope 查询正常状态公民:
  普通私法人和挂靠私法人的机构可搜全国;公权机构、公法人教育机构和挂靠公法人的非法人机构按本省/本市收口;国家级/部级/联邦级公权机构可搜全国。
- 公民列表页仍按管理员省/市 scope 精确查询,不得因为法定代表人全国可选而放大后台公民管理列表权限。
- 完成绑定和年度报告导入属于 `PASSKEY` 写操作,必须携带 Passkey 换取的一次性
  `x-cid-security-grant`。
- 公民录入时必须规范化钱包字段；后续绑定校验必须确认签名响应公钥等于记录中的 `wallet_pubkey`。
- 生成方本地 session 的 `payload_hash` 必须等于 challenge 原文哈希；二维码响应不得携带 `payload_hash`。
- CitizenApp 扫描 `citizen_bind` 请求时必须先按 QR_V1 `b.a + b.d` 独立解析载荷，确认 action、公民状态、选举权利和钱包地址与解码展示内容一致后才签名。
- `cid_number / wallet_pubkey` 二者保持一对一唯一关系。

## 6. 审计事件

| 事件 | 触发场景 | 关键字段 |
|------|---------|---------|
| `CITIZEN_BIND` | 管理员完成电子护照绑定 | wallet_pubkey, cid_number |
| `APP_VOTE_CREDENTIAL` | CitizenApp 请求公民投票凭证 | wallet_pubkey, proposal_id |
