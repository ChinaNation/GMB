# SFID 后端链交互归属规则

- 最后更新:2026-05-02
- 任务卡:
  - `memory/08-tasks/open/20260502-sfid-chain目录归并功能模块.md`
  - `memory/08-tasks/done/20260502-sfid-cpms-sheng目录整改.md`
  - `memory/08-tasks/done/20260502-sfid-institutions粗粒度整合.md`

## 0. 结论

`sfid/backend/chain/` 已废止。SFID 后端不再维护独立 chain 业务目录。

任一功能模块如需和区块链交互,必须在所属功能模块目录中新增 `chain_*.rs`
文件。普通业务 CRUD、页面展示、SFID 本地签名 seed 生命周期、CPMS 系统注册
协议,不得放进 `chain_` 文件。

## 1. 当前代码归属

| 职责 | 当前文件 |
|---|---|
| 机构查询、注册信息凭证、账户列表、清算行候选 DTO 与 handler | `sfid/backend/institutions/chain_duoqian_info.rs` |
| 公民绑定推链 | `sfid/backend/citizens/chain_binding.rs` |
| 公民投票凭证 | `sfid/backend/citizens/chain_vote.rs` |
| 联合投票人口快照凭证 | `sfid/backend/citizens/chain_joint_vote.rs` |
| 通用链凭证/SCALE/genesis hash | `sfid/backend/app_core/chain_runtime.rs` |
| 通用链推送 helper | `sfid/backend/app_core/chain_client.rs` |
| 通用链 RPC URL | `sfid/backend/app_core/chain_url.rs` |

省管理员模块当前没有活跃 `chain_*.rs` 文件。后续只有“更换省管理员/主备交换”
需要和区块链交互时,才允许在 `sfid/backend/sheng_admins/` 下新增
`chain_replace_admin.rs`。

CPMS 系统安装、QR2 注册、QR3 匿名证书、站点状态治理归
`sfid/backend/cpms/handler.rs`,不属于省管理员链交互。

## 2. 目录铁律

- 禁止恢复 `sfid/backend/chain/`。
- 新增链交互文件必须以 `chain_` 开头。
- 链交互文件必须放在业务归属目录内,例如:
  - 机构 -> `institutions/chain_*`
  - 公民 -> `citizens/chain_*`
  - 省管理员更换 -> `sheng_admins/chain_replace_admin.rs`
- 只有跨业务复用的链底层工具允许放在 `app_core/chain_*`。
- `main.rs` 路由只能引用业务模块下的 `chain_*` 文件,不得引用独立 `chain::*`。

## 3. 机构注册信息凭证

链端正式注册机构前,调用:

```text
GET /api/v1/app/institutions/:sfid_id/registration-info
```

业务字段只包含:

```text
sfid_id
institution_name
account_names[]
```

验签包装字段位于 `credential`:

- `genesis_hash`
- `register_nonce`
- `province`
- `signer_admin_pubkey`
- `signature`
- `meta`

`a3/sub_type/parent_sfid_id`、照片、章程、许可证、股东会决议、法人授权书等
SFID 内部资料不进入链端注册信息凭证。链上管理员、阈值、金额、投票等仍归
`duoqian-manage` 自己校验。

## 4. 端点归属

| 端点 | 代码 |
|---|---|
| `GET /api/v1/app/voters/count` | `citizens::chain_joint_vote::app_voters_count` |
| `POST /api/v1/app/vote/credential` | `citizens::chain_vote::app_vote_credential` |
| `GET /api/v1/app/institutions/search` | `institutions::chain_duoqian_info::app_search_institutions` |
| `GET /api/v1/app/institutions/:sfid_id` | `institutions::chain_duoqian_info::app_get_institution` |
| `GET /api/v1/app/institutions/:sfid_id/registration-info` | `institutions::chain_duoqian_info::app_get_institution_registration_info` |
| `GET /api/v1/app/institutions/:sfid_id/accounts` | `institutions::chain_duoqian_info::app_list_accounts` |
| `GET /api/v1/app/clearing-banks/search` | `institutions::chain_duoqian_info::app_search_clearing_banks` |
| `GET /api/v1/app/clearing-banks/eligible-search` | `institutions::chain_duoqian_info::app_search_eligible_clearing_banks` |

说明:

- `GET /api/v1/admin/sheng-admin/roster` 是注册局页面展示接口,归
  `sheng_admins::roster`,不是链交互。
- `POST /api/v1/admin/sheng-signer/prepare|submit` 是省管理员本人本地 signing
  seed 生成/更换流程,归 `sheng_admins::signing_keys`,不是链交互。
- 旧 `sheng-signer/activate` / `sheng-signer/rotate` / `sheng-admin/roster/add-backup`
  / `sheng-admin/roster/remove-backup` 已删除。

## 5. 验收

- `sfid/backend/chain/` 不存在。
- `sfid/backend/main.rs` 不存在 `mod chain;`。
- `rg "crate::chain|chain::" sfid/backend` 无活跃引用。
- `sfid/backend/sheng_admins/` 不存在旧省管理员 add/remove/activate/rotate 链文件。
- `cd sfid/backend && cargo fmt && cargo check` 通过。
