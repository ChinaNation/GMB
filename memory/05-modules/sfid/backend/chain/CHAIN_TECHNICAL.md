# SFID 后端链交互归属规则

- 最后更新:2026-05-02
- 任务卡:
  - `memory/08-tasks/open/20260502-sfid-chain目录归并功能模块.md`

## 0. 结论

`sfid/backend/chain/` 已废止。SFID 后端不再维护独立 chain 业务目录。

今后任一功能模块只要需要和区块链交互,就在所属功能模块目录中新增
`chain_*.rs` 文件。普通业务 CRUD、service、model 不混入链交互代码。

## 1. 当前代码归属

| 职责 | 当前文件 |
|---|---|
| 机构查询、注册信息凭证、清算行候选 | `sfid/backend/institutions/chain_duoqian_info.rs` |
| 机构链交互 DTO | `sfid/backend/institutions/chain_duoqian_info_dto.rs` |
| 机构链交互 handler | `sfid/backend/institutions/chain_duoqian_info_handler.rs` |
| 公民绑定推链 | `sfid/backend/citizens/chain_binding.rs` |
| 公民投票凭证 | `sfid/backend/citizens/chain_vote.rs` |
| 联合投票人口快照凭证 | `sfid/backend/citizens/chain_joint_vote.rs` |
| 省管理员三槽名册 handler | `sfid/backend/sheng_admins/chain_roster_handler.rs` |
| 省管理员三槽名册查询 | `sfid/backend/sheng_admins/chain_roster_query.rs` |
| 省管理员 backup 新增/移除 | `sfid/backend/sheng_admins/chain_add_backup.rs` / `chain_remove_backup.rs` |
| 省管理员签名公钥激活/轮换 | `sfid/backend/sheng_admins/chain_activate_signer.rs` / `chain_rotate_signer.rs` |
| 省管理员冷钱包待签缓存 | `sfid/backend/sheng_admins/chain_pending_signs.rs` |
| 通用链凭证/SCALE/genesis hash | `sfid/backend/app_core/chain_runtime.rs` |
| 通用链推送 helper | `sfid/backend/app_core/chain_client.rs` |
| 通用链 RPC URL | `sfid/backend/app_core/chain_url.rs` |

## 2. 目录铁律

- 禁止恢复 `sfid/backend/chain/`。
- 新增链交互文件必须以 `chain_` 开头。
- 链交互文件必须放在业务归属目录内,例如:
  - 机构 -> `institutions/chain_*`
  - 公民 -> `citizens/chain_*`
  - 省管理员 -> `sheng_admins/chain_*`
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
| `GET /api/v1/admin/sheng-admin/roster` | `sheng_admins::chain_roster_handler::list_roster_admin` |
| `POST /api/v1/admin/sheng-admin/roster/add-backup` | `sheng_admins::chain_add_backup::handler` |
| `POST /api/v1/admin/sheng-admin/roster/remove-backup` | `sheng_admins::chain_remove_backup::handler` |
| `POST /api/v1/admin/sheng-signer/activate` | `sheng_admins::chain_activate_signer::handler` |
| `POST /api/v1/admin/sheng-signer/rotate` | `sheng_admins::chain_rotate_signer::handler` |
| `GET /api/v1/chain/sheng-admin/list` | `sheng_admins::chain_roster_handler::list_roster_public` |

## 5. 验收

- `sfid/backend/chain/` 不存在。
- `sfid/backend/main.rs` 不存在 `mod chain;`。
- `rg "crate::chain|chain::" sfid/backend` 无活跃引用。
- `cd sfid/backend && cargo fmt && cargo check` 通过。
