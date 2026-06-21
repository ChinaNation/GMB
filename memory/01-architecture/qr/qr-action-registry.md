# QR Action Registry

- 版本:2026-05-08
- 状态:当前详细事实源,由 `memory/07-ai/unified-protocols.md` 统一管辖
- 范围:`kind = sign_request` 的 `body.display.action` 与 `body.display.fields`
- 依赖:
  - `memory/01-architecture/qr/qr-signing-recognition.md`
  - `citizenwallet/lib/signer/pallet_registry.dart`
  - `citizenwallet/lib/signer/payload_decoder.dart`

任何一端(`citizenwallet/lib/signer/payload_decoder.dart` / `citizenchain/node/src/` / `citizenapp/lib/`)新增或修改 action / field key,必须先改本文件,再改代码。字段 key、字段顺序、渲染值必须逐字对齐。

## 一、Action 清单

### 1.1 Balances(pallet_index = 2)

| action | call_index | call | fields(顺序固定) | 签发方 |
|---|---|---|---|---|
| `transfer` | 3 | `transfer_keep_alive` | `to`, `amount_yuan` | node_ui, citizenapp |

### 1.2 PersonalManage(pallet_index = 7)

| action | call_index | call | fields(顺序固定) | 签发方 |
|---|---|---|---|---|
| `propose_create_personal` | 0 | `propose_create` | `account_name`, `admins_len`, `regular_threshold`, `create_threshold`, `amount_yuan` | citizenapp |
| `propose_close_personal` | 1 | `propose_close` | `duoqian_account`, `beneficiary` | citizenapp |
| `cleanup_rejected_personal_proposal` | 2 | `cleanup_rejected_proposal` | `proposal_id` | citizenapp |

### 1.3 ResolutionIssuance(pallet_index = 8)

| action | call_index | call | fields(顺序固定) | 签发方 |
|---|---|---|---|---|
| `propose_resolution_issuance` | 0 | `propose_resolution_issuance` | `reason`, `amount_yuan`, `allocation_count`, `eligible_total`, `province_name`, `signer_pubkey` | node_ui, citizenapp |

### 1.4 VotingEngine(pallet_index = 9)

| action | call_index | call | fields(顺序固定) | 签发方 |
|---|---|---|---|---|
| `finalize_proposal` | 3 | `finalize_proposal` | `proposal_id` | node_ui, citizenapp |
| `retry_passed_proposal` | 4 | `retry_passed_proposal` | `proposal_id` | node_ui, citizenapp |
| `cancel_passed_proposal` | 5 | `cancel_passed_proposal` | `proposal_id`, `reason` | node_ui, citizenapp |

`VotingEngine(9)` 只承载提案生命周期入口。管理员投票不得再登记到 `VotingEngine(9)`。

### 1.5 AdminsChange(pallet_index = 12)

| action | call_index | call | fields(顺序固定) | 签发方 |
|---|---|---|---|---|
| `propose_admin_set_change` | 0 | `propose_admin_set_change` | `org`, `subject`, `admins[]` | node_ui, citizenapp |

`call_index = 1` 已留洞不复用。手动重试统一走 `VotingEngine.retry_passed_proposal(9.4)`。

### 1.6 RuntimeUpgrade(pallet_index = 13)

| action | call_index | call | fields(顺序固定) | 签发方 |
|---|---|---|---|---|
| `propose_runtime_upgrade` | 0 | `propose_runtime_upgrade` | `reason`, `wasm_size`, `wasm_hash`, `eligible_total` | node_ui, citizenapp |
| `developer_direct_upgrade` | 2 | `developer_direct_upgrade` | `wasm_size`, `wasm_hash` | node_ui, citizenapp |

Runtime 升级 QR 中的 `payload_hex` 只允许放 32 字节 WASM payload hash;冷钱包走哈希直签例外,但必须展示 `wasm_hash` 供用户核对。

### 1.7 ResolutionDestro(pallet_index = 14)

| action | call_index | call | fields(顺序固定) | 签发方 |
|---|---|---|---|---|
| `propose_destroy` | 0 | `propose_destroy` | `org`, `amount_yuan` | citizenapp |

`call_index = 1` 已留洞不复用。手动重试统一走 `VotingEngine.retry_passed_proposal(9.4)`。

### 1.8 GrandpaKeyChange(pallet_index = 16)

| action | call_index | call | fields(顺序固定) | 签发方 |
|---|---|---|---|---|
| `propose_replace_grandpa_key` | 0 | `propose_replace_grandpa_key` | `institution`, `new_key` | node_ui |

`call_index = 1 / 2` 已留洞不复用。手动重试/取消统一走 `VotingEngine.retry_passed_proposal(9.4)` / `VotingEngine.cancel_passed_proposal(9.5)`。

### 1.9 OrganizationManage(pallet_index = 17)

| action | call_index | call | fields(顺序固定) | 签发方 |
|---|---|---|---|---|
| `propose_close_institution` | 1 | `propose_close` | `duoqian_account`, `beneficiary` | citizenapp |
| `cleanup_rejected_proposal` | 4 | `cleanup_rejected_proposal` | `proposal_id` | citizenapp |
| `propose_create_institution` | 5 | `propose_create_institution` | `sfid_number`, `sfid_full_name`, `admins_len`, `threshold`, `total_amount_yuan`, `amount_<account_name>*`, `province_name`, `signer_pubkey` | node_ui, citizenapp |

`register_sfid_institution(call_index = 2)` 由 SFID 后端签发凭证并由链端验签,不走冷钱包扫码签名,不在本表范围。`call_index = 0 / 3` 已留洞不复用。

### 1.10 DuoqianTransfer(pallet_index = 19)

| action | call_index | call | fields(顺序固定) | 签发方 |
|---|---|---|---|---|
| `propose_transfer` | 0 | `propose_transfer` | `institution`, `beneficiary`, `amount_yuan`, `remark` | node_ui, citizenapp |
| `propose_safety_fund_transfer` | 1 | `propose_safety_fund` | `beneficiary`, `amount_yuan`, `remark` | node_ui, citizenapp |
| `propose_sweep_to_main` | 2 | `propose_sweep` | `institution`, `amount_yuan` | node_ui, citizenapp |

`call_index = 3 / 4 / 5` 已留洞不复用。手动重试统一走 `VotingEngine.retry_passed_proposal(9.4)`。

### 1.11 OffchainTransaction(pallet_index = 21)

| action | call_index | call | fields(顺序固定) | 签发方 |
|---|---|---|---|---|
| `bind_clearing_bank` | 30 | `bind_clearing_bank` | `bank_main` | node_ui, citizenapp |
| `deposit_clearing_bank` | 31 | `deposit` | `amount_yuan` | node_ui, citizenapp |
| `withdraw_clearing_bank` | 32 | `withdraw` | `amount_yuan` | node_ui, citizenapp |
| `switch_clearing_bank` | 33 | `switch_bank` | `new_bank` | node_ui, citizenapp |
| `register_clearing_bank` | 50 | `register_clearing_bank` | `sfid_number`, `peer_id`, `rpc_domain`, `rpc_port` | node_ui |
| `update_clearing_bank_endpoint` | 51 | `update_clearing_bank_endpoint` | `sfid_number`, `new_domain`, `new_port` | node_ui |
| `unregister_clearing_bank` | 52 | `unregister_clearing_bank` | `sfid_number` | node_ui |

### 1.12 InternalVote(pallet_index = 22)

| action | call_index | call | fields(顺序固定) | 签发方 |
|---|---|---|---|---|
| `internal_vote` | 0 | `cast` | `proposal_id`, `approve` | node_ui, citizenapp |

### 1.13 JointVote(pallet_index = 23)

| action | call_index | call | fields(顺序固定) | 签发方 |
|---|---|---|---|---|
| `joint_vote` | 0 | `cast_admin` | `proposal_id`, `approve` | node_ui, citizenapp |
| `cast_referendum` | 1 | `cast_referendum` | `proposal_id`, `approve`, `province_name`, `signer_pubkey` | citizenapp |

`joint_vote` 的 call data 内含 48 字节 `institution_id`,当前冷钱包 decoder 只展示 `proposal_id` / `approve`。若要展示机构身份,必须先更新本表和 decoder。

### 1.14 链下签名载荷

| action | payload 结构 | fields(顺序固定) | 签发方 | 用途 |
|---|---|---|---|---|
| `activate_admin_account` | `GMB_ACTIVATE_SUBJECT_V1`(23B) + `account_id`(48B) + `org`(u8) + `kind`(u8) + `pubkey`(32B) + `timestamp`(8B u64) + `nonce`(16B) = 130B | `org`, `subject`, `pubkey` | node_ui / citizenapp | subject 级管理员激活 |
| `decrypt_admin` | `GMB_DECRYPT_V1`(14B) + `sfid_number`(48B 右补零) + `pubkey`(32B) + `timestamp`(8B u64) + `nonce`(16B) = 118B | `sfid_number` | node_ui | 清算行管理员解密 challenge |
| `citizen_bind` | `sfid-citizen-bind-v1\|challenge_id\|mode\|archive_no\|citizen_status\|voting_eligible\|valid_from\|valid_until\|status_updated_at\|wallet_pubkey\|issued_at` | `mode`, `archive_no`, `voting_eligible`, `citizen_status`, `wallet_address` | sfid 后端 | citizenapp 电子护照绑定签名 |
| `archive_delete` | `CPMS_ARCHIVE_DELETE_V1\|challenge_id\|archive_id\|archive_no\|0x_admin_pubkey\|expires_at` | `archive_no`, `admin_pubkey`, `expires_at` | cpms | CPMS 公民档案软删除 |
| `sfid_admin_action` | `sfid_admin_governance` canonical JSON hex | `action_type`, `actor_province_name`, `actor_pubkey`, `target` | sfid 后端 | 联邦管理员治理和 Passkey 更新公民钱包确认 |

## 二、字段渲染规则

| key | 类型 | 渲染格式 |
|---|---|---|
| `to` / `beneficiary` / `duoqian_account` / `old_admin` / `new_admin` / `bank_main` / `new_bank` | `AccountId32` | SS58,prefix = 2027 |
| `amount_yuan` / `total_amount_yuan` / `amount_<account_name>` | `Balance` u128,单位分 | `"X.XX GMB"`;整数位可带千分位 |
| `proposal_id` / `eligible_total` / `allocation_count` / `rpc_port` / `new_port` | 整数 | 十进制字符串 |
| `approve` | bool | `"true"` / `"false"` |
| `org` | u8 / u32 机构代号 | 机构中文名;找不到时回退为 `机构<raw>` |
| `institution` | 48B sfid_number | 优先转机构中文名;找不到时回退原 sfid_number |
| `wasm_size` | u32 字节 | `"X.XX MB"` 或 `"X KB"` |
| `pubkey` / `signer_pubkey` / `admin_pubkey` / `actor_pubkey` / `old_admin` / `new_admin` | 32 字节账户/公钥 | 人机展示为 SS58,prefix = 2027 |
| `wasm_hash` / `new_key` / `payload_hash` | 32 字节哈希或非账户密钥 | `0x<64hex>` 小写;默认不进入普通确认字段 |
| `reason` / `remark` / `account_name` / `sfid_full_name` / `sfid_number` / `province_name` / `actor_province_name` / `peer_id` / `rpc_domain` / `new_domain` | UTF-8 | 原字符串;UI 可截断展示,签名原文不截断 |
| `admins_len` | u32 | 十进制字符串 |
| `threshold` | u32 | `"<threshold>/<admins_len>"` |
| `archive_no` / `archive_id` / `expires_at` | UTF-8 | 原字符串 |

## 三、字段约束

1. `display.action` 必须与 decoder 输出的 `decoded.action` 逐字相等。
2. `display.fields[*].key` 必须与本表 fields 列逐字相等;禁止大小写变体、别名和回退 key。
3. `display.fields` 中出现的 key/value 必须与 decoder 验真字段逐字一致;未出现的机器字段不展示。
4. 钱包 UI 展示顺序以 decoder 的 `reviewFields` 为准,只展示中文业务字段和 SS58 地址。
5. `amount_<account_name>` 是 `propose_create_institution` 的动态字段,`<account_name>` 必须等于 call data 中账户名称原文。
6. 未列入本表的 action 不得进入生产 `sign_request`。

## 四、新增或修改 action 流程

1. 先修改本文件,加入或调整 pallet_index / call_index / fields / 签发方。
2. 修改 `citizenwallet/lib/signer/pallet_registry.dart` 的索引常量。
3. 修改 `citizenwallet/lib/signer/payload_decoder.dart`,确保 `decoded.action` 和 `decoded.fields` 与本表逐字对齐。
4. 修改签发方代码(`citizenchain/node/src/` 或 `citizenapp/lib/`),确保 `display.action` 和 `SignDisplayField.key` 与本表逐字对齐。
5. 补 `citizenwallet/test/` 单测和 `memory/01-architecture/qr/qr-protocol-fixtures/` golden fixture。
6. 补端到端 Flutter / Rust 测试。
7. PR 标题包含 `[qr-registry]`,CI 门禁扫本文件 vs 三端实现。

## 五、已废弃且不得恢复

- `VotingEngine(9).internal_vote`
- `VotingEngine(9).joint_vote`
- `VotingEngine(9).citizen_vote`
- `DuoqianManage(pallet_index = 17)` 旧命名
- `OrganizationManage(17).propose_create(call_index = 0)`
- `OrganizationManage(17).propose_create_personal(call_index = 3)`
- `DuoqianTransfer(19).execute_transfer(call_index = 3)`
- `DuoqianTransfer(19).execute_safety_fund_transfer(call_index = 4)`
- `DuoqianTransfer(19).execute_sweep_to_main(call_index = 5)`
- `AdminsChange(12).execute_admin_replacement(call_index = 1)`
- `ResolutionDestro(14).execute_destroy(call_index = 1)`
- `GrandpaKeyChange(16).execute_replace_grandpa_key(call_index = 1)`
- `GrandpaKeyChange(16).cancel_failed_replace_grandpa_key(call_index = 2)`
