# SFID 后端链交互归属规则

- 最后更新:2026-05-31
- 任务卡:
  - `memory/08-tasks/open/20260502-sfid-chain目录归并功能模块.md`
  - `memory/08-tasks/done/20260502-sfid-cpms-sheng目录整改.md`
  - `memory/08-tasks/done/20260502-sfid-institutions粗粒度整合.md`
  - `memory/08-tasks/open/20260530-sfid-province-admin-governance-passkey.md`

## 0. 结论

SFID 后端独立 chain 业务目录已废止。SFID 后端不再维护独立 chain 业务目录。

任一功能模块如需和区块链交互,必须在所属功能模块目录中新增 `chain_*.rs`
文件。普通业务 CRUD、页面展示、管理员 Passkey/冷钱包挑战、CPMS 系统注册
协议,不得放进 `chain_` 文件。

## 1. 当前代码归属

| 职责 | 当前文件 |
|---|---|
| 机构查询、注册信息凭证、账户列表、清算行候选 DTO 与 handler | `sfid/backend/institutions/chain_duoqian_info.rs` |
| 公民投票凭证 | `sfid/backend/citizens/chain_vote.rs` |
| 联合投票人口快照凭证 | `sfid/backend/citizens/chain_joint_vote.rs` |
| 通用链凭证/SCALE/genesis hash | `sfid/backend/app_core/chain_runtime.rs` |
| 通用链 RPC URL | `sfid/backend/app_core/chain_url.rs` |

管理员模块当前没有活跃 `chain_*.rs` 文件。省级管理员只负责管理员治理,
写操作统一走 `admins/actions.rs` 的安全动作入口与 `admins/passkeys.rs` 的
WebAuthn 辅助,
不使用云端省级 signer。

CPMS 系统安装授权、ARCHIVE 档案验真和站点状态治理归
`sfid/backend/cpms/handler.rs`,不属于省管理员链交互。

## 2. 目录铁律

- 禁止恢复后端独立 chain 业务目录。
- 新增链交互文件必须以 `chain_` 开头。
- 链交互文件必须放在业务归属目录内,例如:
  - 机构 -> `institutions/chain_*`
  - 公民 -> `citizens/chain_*`
  - 管理员治理链交互 -> `admins/chain_*`
- 只有跨业务复用的链底层工具允许放在 `app_core/chain_*`。
- `main.rs` 路由只能引用业务模块下的 `chain_*` 文件,不得引用独立 `chain::*`。

## 3. 机构注册信息凭证

链端正式注册机构前,调用:

```text
GET /api/v1/app/institutions/:sfid_number/registration-info
```

业务字段只包含:

```text
sfid_number
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

`a3/sub_type/parent_sfid_number`、照片、章程、许可证、股东会决议、法人授权书等
SFID 内部资料不进入链端注册信息凭证。链上管理员、阈值、金额、投票等由
`organization-manage`、`personal-manage` 和 `admins-change::Subjects` 按各自边界校验。

## 4. 端点归属

| 端点 | 代码 |
|---|---|
| `GET /api/v1/app/voters/count` | `citizens::chain_joint_vote::app_voters_count` |
| `POST /api/v1/app/vote/credential` | `citizens::chain_vote::app_vote_credential` |
| `GET /api/v1/app/institutions/search` | `institutions::chain_duoqian_info::app_search_institutions` |
| `GET /api/v1/app/institutions/:sfid_number` | `institutions::chain_duoqian_info::app_get_institution` |
| `GET /api/v1/app/institutions/:sfid_number/registration-info` | `institutions::chain_duoqian_info::app_get_institution_registration_info` |
| `GET /api/v1/app/institutions/:sfid_number/accounts` | `institutions::chain_duoqian_info::app_list_accounts` |
| `GET /api/v1/app/clearing-banks/search` | `institutions::chain_duoqian_info::app_search_clearing_banks` |
| `GET /api/v1/app/clearing-banks/eligible-search` | `institutions::chain_duoqian_info::app_search_eligible_clearing_banks` |

说明:

- `GET /api/v1/admin/sheng-admins` 是省级管理员列表接口,归
  `admins::catalog`,不是链交互。
- `POST /api/v1/admin/passkeys/register/start|confirm|complete` 是管理员 Passkey 注册流程,
  归 `admins::passkeys`,不是链交互。
- `POST /api/v1/admin/actions/*` 是管理员治理安全动作流程,归
  `admins::actions`,不是链交互。
- 省管理员云端代签端点已删除;不得恢复为替代入口。

## 5. 验收

- 后端独立 chain 业务目录不存在。
- `sfid/backend/main.rs` 不存在 `mod chain;`。
- `rg "crate::chain|chain::" sfid/backend` 无活跃引用。
- `sfid/backend/admins/` 不存在省管理员云端代签文件。
- `cd sfid/backend && cargo fmt && cargo check` 通过。
