# QR_V1 Action Registry

- 更新日期:2026-07-12
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
| 2 | `citizen_identity` | `VotingIdentityPayload` SCALE bytes | `blake2_256(GMB || 0x10 || payload)` | OnChina | CitizenWallet / CitizenApp(电子护照扫码签名页) | 公民本人确认链上投票身份载荷;CitizenApp 侧解码器在 `my/myid/voting_identity_payload.dart` |
| 3 | `onchina_admin_action` / `QR_ACTION_ONCHINA_ADMIN` | `onchina_admin_governance` canonical JSON UTF-8 | 原文 | OnChina | CitizenWallet | 链上中国平台管理员治理冷钱包确认 |
| 5 | `activate_admin_account` | `GMB || 0x18` 二进制 payload | 原文 | citizenchain node / CitizenApp | CitizenWallet | 管理员激活 |
| 6 | `decrypt_admin` | `GMB || 0x19` 二进制 payload | 原文 | citizenchain node | CitizenWallet | 清算行管理员解密 |
| 7 | `runtime_upgrade_hash` | 32B WASM hash | 原文 32B | citizenchain node / CitizenApp | CitizenWallet | Runtime 升级哈希直签 |
| 9 | `square_account_action` / `QR_ACTION_SQUARE_ACCOUNT` | 广场账户动作 SCALE bytes（`action‖owner‖challenge_id[‖level]‖u64(expires)`） | `signing_message(OP_SIGN_SQUARE_ACTION, payload)` | 官网 citizenweb / CitizenApp | CitizenApp（交易 tab「扫一扫」，owner 主钥+生物识别） | 会员订阅/取消等账户动作链下签名；owner 由 QR `u` 在本机定位钱包，两色解码 `signer/square_action_payload.dart`；Worker `account/action_challenge.ts` 构造/验签 |

动作码 `8` 已取消登记。Chat 设备绑定只能使用 CitizenApp 已登记的硬件 P-256 设备子钥静默签名，不得生成 QR 请求，不得交给 CitizenWallet 或钱包主私钥签名。

## 2. 链交易动作码

链交易统一使用:

```text
a = (pallet_index << 8) | call_index
```

示例:`OnchainTransaction(4).transfer_with_remark(0)` 的 `a = 0x0400 = 1024`。

| a(hex) | pallet.call | decoder action | payload 展示字段 | 签发方 |
|---|---|---|---|---|
| `0x0400` | `OnchainTransaction.transfer_with_remark` | `transfer` | `to`, `amount_yuan`, `remark` | citizenchain node / CitizenApp |
| `0x0700` | `PersonalManage.propose_create` | `propose_create_personal` | `account_name`, `admins_len`, `regular_threshold`, `create_threshold`, `amount_yuan` | CitizenApp |
| `0x0701` | `PersonalManage.propose_close` | `propose_close_personal` | `account`, `beneficiary` | CitizenApp |
| `0x0702` | `PersonalManage.cleanup_rejected_proposal` | `cleanup_rejected_personal_proposal` | `proposal_id` | CitizenApp |
| `0x1d00` | `PersonalAdmins.propose_admin_set_change` | `propose_personal_admin_set_change` | `institution_code`, `account`, `admins`, `new_threshold` | CitizenApp |
| `0x0800` | `ResolutionIssuance.propose_resolution_issuance` | `propose_resolution_issuance` | `reason`, `amount_yuan`, `allocation_count`, `eligible_total`, `province_name`, `signer_pubkey` | citizenchain node / CitizenApp |
| `0x0903` | `VotingEngine.finalize_proposal` | `finalize_proposal` | `proposal_id` | citizenchain node / CitizenApp |
| `0x0904` | `VotingEngine.retry_passed_proposal` | `retry_passed_proposal` | `proposal_id` | citizenchain node / CitizenApp |
| `0x0905` | `VotingEngine.cancel_passed_proposal` | `cancel_passed_proposal` | `proposal_id`, `reason` | citizenchain node / CitizenApp |
| `0x0a00` | `CitizenIdentity.register_voting_identity` | `register_voting_identity` | `registrar_account`, `cid_number`, `wallet_account`, `citizen_age_years`, `valid_range`, `citizen_status`, `residence` | OnChina |
| `0x0c00` | `RuntimeUpgrade.propose_runtime_upgrade` | `propose_runtime_upgrade` | `wasm_hash` | citizenchain node / CitizenApp |
| `0x0c02` | `RuntimeUpgrade.developer_direct_upgrade` | `developer_direct_upgrade` | `wasm_hash` | citizenchain node / CitizenApp |
| `0x0d00` | `ResolutionDestro.propose_destroy` | `propose_destroy` | `institution_code`, `amount_yuan` | CitizenApp |
| `0x0f00` | `GrandpaKeyChange.propose_replace_grandpa_key` | `propose_replace_grandpa_key` | `institution`, `new_key` | citizenchain node |
| `0x1e01` | `PublicManage.propose_close_public_institution` | `propose_close_public_institution` | `account`, `beneficiary` | CitizenApp |
| `0x1e04` | `PublicManage.cleanup_rejected_public_proposal` | `cleanup_rejected_public_proposal` | `proposal_id` | CitizenApp |
| `0x1e05` | `PublicManage.propose_create_public_institution` | `propose_create_public_institution` | `cid_number`, `cid_full_name`, `cid_short_name`, `town_code`, `legal_representative_*`, `accounts`, `institution_code`, `roles`, `assignments`, `threshold`, `register_nonce`, `signature`, `issuer_*`, `signer_pubkey`, `scope_*` | OnChina（注册局录入） |
| `0x1f01` | `PrivateManage.propose_close_private_institution` | `propose_close_private_institution` | `account`, `beneficiary` | CitizenApp |
| `0x1f04` | `PrivateManage.cleanup_rejected_private_proposal` | `cleanup_rejected_private_proposal` | `proposal_id` | CitizenApp |
| `0x1f05` | `PrivateManage.propose_create_private_institution` | `propose_create_private_institution` | `cid_number`, `cid_full_name`, `cid_short_name`, `town_code`, `legal_representative_*`, `accounts`, `institution_code`, `roles`, `assignments`, `threshold`, `register_nonce`, `signature`, `issuer_*`, `signer_pubkey`, `scope_*` | OnChina（注册局录入） |
| `0x2100` | `AddressRegistry.set_catalog_version` | `set_address_catalog_version` | `registrar_account`, `catalog_version`, `catalog_hash` | onchina |
| `0x2101` | `AddressRegistry.set_address_name` | `set_address_name` | `registrar_account`, `province_code`, `city_code`, `town_code`, `address_name_code`, `address_name` | onchina |
| `0x2102` | `AddressRegistry.remove_address_name` | `remove_address_name` | `registrar_account`, `province_code`, `city_code`, `town_code`, `address_name_code` | onchina |
| `0x2103` | `AddressRegistry.set_address` | `set_address` | `registrar_account`, `province_code`, `city_code`, `town_code`, `address_name_code`, `address_local_no`, `address_detail` | onchina |
| `0x2104` | `AddressRegistry.remove_address` | `remove_address` | `registrar_account`, `province_code`, `city_code`, `town_code`, `address_name_code`, `address_local_no`, `address_detail` | onchina |
| `0x1100` | `MultisigTransfer.propose_transfer` | `propose_transfer` | `institution`, `beneficiary`, `amount_yuan`, `remark` | citizenchain node / CitizenApp |
| `0x1101` | `MultisigTransfer.propose_safety_fund` | `propose_safety_fund_transfer` | `beneficiary`, `amount_yuan`, `remark` | citizenchain node / CitizenApp |
| `0x1102` | `MultisigTransfer.propose_sweep` | `propose_sweep_to_main` | `institution`, `amount_yuan` | citizenchain node / CitizenApp |
| `0x1332` | `OffchainTransaction.register_clearing_bank` | `register_clearing_bank` | `cid_number`, `peer_id`, `rpc_domain`, `rpc_port` | citizenchain node |
| `0x1333` | `OffchainTransaction.update_clearing_bank_endpoint` | `update_clearing_bank_endpoint` | `cid_number`, `new_domain`, `new_port` | citizenchain node |
| `0x1334` | `OffchainTransaction.unregister_clearing_bank` | `unregister_clearing_bank` | `cid_number` | citizenchain node |
| `0x1400` | `InternalVote.cast` | `internal_vote` | `proposal_id`, `approve` | citizenchain node / CitizenApp |
| `0x1500` | `JointVote.cast_admin` | `joint_vote` | `proposal_id`, `approve` | citizenchain node / CitizenApp |
| `0x1501` | `JointVote.cast_referendum` | `cast_referendum` | `proposal_id`, `approve`, `province_name`, `signer_pubkey` | CitizenApp |

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
