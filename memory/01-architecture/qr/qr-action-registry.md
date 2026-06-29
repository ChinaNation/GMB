# QR_V1 Action Registry

- 更新日期:2026-06-29
- 状态:当前详细事实源,由 `memory/07-ai/unified-protocols.md` 统一管辖
- 范围:`k=1` 签名请求的 `b.a` 数字动作码
- 依赖:
  - `memory/01-architecture/qr/qr-protocol-spec.md`
  - `citizenchain/runtime/primitives/src/sign.rs`
  - `citizenchain/onchina/src/core/institution_call.rs`(链交易动作码 `chain_action_code` + 机构/管理员 call 编码器)
  - `citizenchain/onchina/src/core/qr/mod.rs`(非链动作码常量)
  - `citizenwallet/lib/qr/qr_protocols.dart`
  - `citizenapp/lib/qr/qr_protocols.dart`
  - `citizenwallet/lib/signer/payload_decoder.dart`

`k` 只表达扫码流向;`a` 才表达业务动作。任何平台新增扫码签名场景,必须先在本文件登记 `a` 和 payload 解码规则。

## 1. 非链动作码

| a | 名称 | payload | 签名字节 | 生成方 | 扫码/签名方 | 注释 |
|---:|---|---|---|---|---|---|
| 1 | `login` | `system|system_signature` UTF-8 | 原文 | OnChina | CitizenWallet | 登录签名确认 |
| 2 | `citizen_bind` | `cid-citizen-bind-v1|...` UTF-8 | 原文 | OnChina | CitizenApp | 电子护照绑定 |
| 3 | `cid_admin_action` | `cid_admin_governance` canonical JSON UTF-8 | 原文 | OnChina | CitizenWallet | 注册局管理员治理冷钱包确认 |
| 5 | `activate_admin_account` | `GMB || 0x18` 二进制 payload | 原文 | citizenchain node / CitizenApp | CitizenWallet | 管理员激活 |
| 6 | `decrypt_admin` | `GMB || 0x19` 二进制 payload | 原文 | citizenchain node | CitizenWallet | 清算行管理员解密 |
| 7 | `runtime_upgrade_hash` | 32B WASM hash | 原文 32B | citizenchain node / CitizenApp | CitizenWallet | Runtime 升级哈希直签 |

## 2. 链交易动作码

链交易统一使用:

```text
a = (pallet_index << 8) | call_index
```

示例:`Balances(2).transfer_keep_alive(3)` 的 `a = 0x0203 = 515`。

| a(hex) | pallet.call | decoder action | payload 展示字段 | 签发方 |
|---|---|---|---|---|
| `0x0203` | `Balances.transfer_keep_alive` | `transfer` | `to`, `amount_yuan` | citizenchain node / CitizenApp |
| `0x0700` | `PersonalAdmins.propose_create` | `propose_create_personal` | `account_name`, `admins_len`, `regular_threshold`, `create_threshold`, `amount_yuan` | CitizenApp |
| `0x0701` | `PersonalAdmins.propose_close` | `propose_close_personal` | `account`, `beneficiary` | CitizenApp |
| `0x0702` | `PersonalAdmins.cleanup_rejected_proposal` | `cleanup_rejected_personal_proposal` | `proposal_id` | CitizenApp |
| `0x0703` | `PersonalAdmins.propose_admin_set_change` | `propose_personal_admin_set_change` | `institution_code`, `account`, `admins`, `new_threshold` | CitizenApp |
| `0x0800` | `ResolutionIssuance.propose_resolution_issuance` | `propose_resolution_issuance` | `reason`, `amount_yuan`, `allocation_count`, `eligible_total`, `province_name`, `signer_pubkey` | citizenchain node / CitizenApp |
| `0x0903` | `VotingEngine.finalize_proposal` | `finalize_proposal` | `proposal_id` | citizenchain node / CitizenApp |
| `0x0904` | `VotingEngine.retry_passed_proposal` | `retry_passed_proposal` | `proposal_id` | citizenchain node / CitizenApp |
| `0x0905` | `VotingEngine.cancel_passed_proposal` | `cancel_passed_proposal` | `proposal_id`, `reason` | citizenchain node / CitizenApp |
| `0x0c00` | `GenesisAdmins.propose_admin_set_change` | `propose_genesis_admin_set_change` | `institution_code`, `account`, `admins`, `new_threshold` | citizenchain node / CitizenApp / onchina(FRG 替换) |
| `0x0c01` | `GenesisAdmins.federal_set_city_registry_admins` | `federal_set_city_registry_admins` | `institution_code`, `account`, `admins`, `threshold` | onchina(联邦注册局直设市注册局) |
| `0x0d00` | `RuntimeUpgrade.propose_runtime_upgrade` | `propose_runtime_upgrade` | `wasm_hash` | citizenchain node / CitizenApp |
| `0x0d02` | `RuntimeUpgrade.developer_direct_upgrade` | `developer_direct_upgrade` | `wasm_hash` | citizenchain node / CitizenApp |
| `0x0e00` | `ResolutionDestro.propose_destroy` | `propose_destroy` | `institution_code`, `amount_yuan` | CitizenApp |
| `0x1000` | `GrandpaKeyChange.propose_replace_grandpa_key` | `propose_replace_grandpa_key` | `institution`, `new_key` | citizenchain node |
| `0x2001` | `PublicManage.propose_close_public_institution` | `propose_close_public_institution` | `account`, `beneficiary` | CitizenApp |
| `0x2004` | `PublicManage.cleanup_rejected_public_proposal` | `cleanup_rejected_public_proposal` | `proposal_id` | CitizenApp |
| `0x2005` | `PublicManage.propose_create_public_institution` | `propose_create_public_institution` | `cid_number`, `cid_full_name`, `cid_short_name`, `admins`(AdminProfile), `admins_len`, `threshold`, `amounts`, `scope`, `signer_pubkey` | CitizenApp / onchina(注册局录入) |
| `0x2101` | `PrivateManage.propose_close_private_institution` | `propose_close_private_institution` | `account`, `beneficiary` | CitizenApp |
| `0x2104` | `PrivateManage.cleanup_rejected_private_proposal` | `cleanup_rejected_private_proposal` | `proposal_id` | CitizenApp |
| `0x2105` | `PrivateManage.propose_create_private_institution` | `propose_create_private_institution` | `cid_number`, `cid_full_name`, `cid_short_name`, `admins`(AdminProfile), `admins_len`, `threshold`, `amounts`, `scope`, `signer_pubkey` | CitizenApp / onchina(注册局录入) |
| `0x1300` | `MultisigTransfer.propose_transfer` | `propose_transfer` | `institution`, `beneficiary`, `amount_yuan`, `remark` | citizenchain node / CitizenApp |
| `0x1d00` | `PublicAdmins.propose_admin_set_change` | `propose_public_admin_set_change` | `institution_code`, `account`, `admins`, `new_threshold` | citizenchain node / CitizenApp |
| `0x1e00` | `PrivateAdmins.propose_admin_set_change` | `propose_private_admin_set_change` | `institution_code`, `account`, `admins`, `new_threshold` | citizenchain node / CitizenApp |
| `0x1301` | `MultisigTransfer.propose_safety_fund` | `propose_safety_fund_transfer` | `beneficiary`, `amount_yuan`, `remark` | citizenchain node / CitizenApp |
| `0x1302` | `MultisigTransfer.propose_sweep` | `propose_sweep_to_main` | `institution`, `amount_yuan` | citizenchain node / CitizenApp |
| `0x1532` | `OffchainTransaction.register_clearing_bank` | `register_clearing_bank` | `cid_number`, `peer_id`, `rpc_domain`, `rpc_port` | citizenchain node |
| `0x1533` | `OffchainTransaction.update_clearing_bank_endpoint` | `update_clearing_bank_endpoint` | `cid_number`, `new_domain`, `new_port` | citizenchain node |
| `0x1534` | `OffchainTransaction.unregister_clearing_bank` | `unregister_clearing_bank` | `cid_number` | citizenchain node |
| `0x1600` | `InternalVote.cast` | `internal_vote` | `proposal_id`, `approve` | citizenchain node / CitizenApp |
| `0x1700` | `JointVote.cast_admin` | `joint_vote` | `proposal_id`, `approve` | citizenchain node / CitizenApp |
| `0x1701` | `JointVote.cast_referendum` | `cast_referendum` | `proposal_id`, `approve`, `province_name`, `signer_pubkey` | CitizenApp |

## 3. 扫码端展示规则

扫码端不得读取 QR 中的展示摘要。展示内容只能来自:

1. `b.a` 定位业务动作。
2. `b.d` 解码出的 payload 字段。
3. 本地 action label / field label 注册表。

上表“payload 展示字段”是内部解码键，不是 UI 文案。扫码确认页必须把字段名映射成中文标签，例如 `institution_code` 显示为“机构类型”、`amount_yuan` 显示为“金额”、`proposal_id` 显示为“提案编号”、`approve` 显示为“投票意见”。

必须展示给用户核对的内容:

| 字段类型 | 展示规则 |
|---|---|
| 账户/公钥可判定为 AccountId32 | 转 SS58,prefix=2027 |
| 金额 | 分转 GMB,保留 2 位 |
| `approve` | 展示“赞成/反对” |
| `remark` / `reason` / `memo` | 原字符串 |
| Runtime hash | 展示 `0x<64hex>` |
| OnChina 管理文本 | 展示动作类型、主体、公钥/账户、过期时间 |

机器校验字段如 nonce、block hash、payload hash、内部 challenge id 不作为普通确认字段展示,但可用于本地 session 校验。

## 4. 拒签规则

任一条件成立必须红色拒签:

1. `a` 未登记。
2. `a` 与 payload 解码出的 pallet/call 或文本 domain 不一致。
3. payload 无法解码。
4. `g != 1`。
5. `u` 不是 32B 公钥。
6. `d` 为空。
7. 链 payload >256B 但扫码端未按 Substrate 规则签 `blake2_256(payload)`。

## 5. 修改流程

1. 先改本文件。
2. 同步 `citizenchain/runtime/primitives/src/sign.rs` 常量。
3. 同步 Dart/TS/Rust 的 action 常量。
4. 同步 `PayloadDecoder` 和签发方 call_data 构造。
5. 补跨端 fixture 和真实扫码签名测试。
