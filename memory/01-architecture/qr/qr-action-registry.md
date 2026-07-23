# QR_V1 Action Registry

- 更新日期:2026-07-22
- 状态:人类可读登记表;代码真源为 `citizenchain/crates/qr-protocol/registry/*`
- 范围:`k=1` 签名请求的 `b.a` 数字动作码
- 依赖:
  - `memory/01-architecture/qr/qr-protocol-spec.md`
  - 唯一代码真源:`citizenchain/crates/qr-protocol/registry/actions.yaml`
  - 字段中文真源:`citizenchain/crates/qr-protocol/registry/fields.yaml`
  - 拒绝原因真源:`citizenchain/crates/qr-protocol/registry/reject_reasons.yaml`
  - Runtime 非链签名常量参考:`citizenchain/runtime/primitives/src/sign.rs`
  - 链交易动作码校验参考:当前 runtime metadata 的 pallet/call index

`k` 只表达扫码流向;`a` 才表达业务动作。任何平台新增扫码签名场景,必须先进入唯一 action registry,再生成或校验各端常量、中文标签和 decoder 映射。不得在 CitizenApp、CitizenWallet、OnChina、node 或 citizenweb 内手写第二套 action 真源。

## 0.0 账户字段目标命名

ADR-040 已冻结 QR registry 及其生成物的账户字段目标：单一账户使用 `account_id`，多个业务角色账户使用 `<role>_account_id`；签名公钥使用 `signer_public_key`，凭证签名公钥使用 `credential_signer_public_key`，展示地址只使用 `ss58_address`。账户和 32 字节公钥的文本值固定为小写 `0x` 加 64 位十六进制。

当前 registry 和后续登记表中的 `wallet_account`、`admin_account`、`owner_account`、`signer_pubkey` 等旧字段会在任务卡 `20260722-account-id-official-unify.md` 的 QR 实施步骤与代码真源、生成器和冷热钱包生成物同步删除；在此之前只代表当前协议事实，不得复制到新 action，也不构成兼容路径。

## 0. 唯一 registry schema

`actions.yaml` 每条 action 必须包含:

| 字段 | 含义 | 强制规则 |
|---|---|---|
| `action_key` | 稳定英文动作键 | 全仓唯一,不得用 UI 文案或临时缩写 |
| `action_code` | QR `b.a` 数字动作码 | 链交易必须与 runtime metadata 校验一致 |
| `action_label_zh` | 用户可见中文动作名 | 缺失即红色拒绝 |
| `kind` | `chain_call` / `offchain_sign` / `hash_only` | 决定 decoder 和签名字节规则 |
| `qr_kind` | 当前必须是 `sign_request` | 不新增登录专用 kind |
| `pallet` | 链交易 pallet 名 | `kind=chain_call` 必填 |
| `call` | 链交易 call 名 | `kind=chain_call` 必填 |
| `decoder` | payload decoder 名 | 缺失即红色拒绝 |
| `hash_only_allowed` | 是否允许 `b.d` 只携带 32B | 只有 Runtime 升级允许为 true |
| `signing_category` | 签名类别 | 用于选择统一签名字节规则 |
| `required_fields` | 必须展示字段 key | 每个字段 key 必须能在 `fields.yaml` 找到中文 |

`fields.yaml` 是扫码展示字段中文名和固定展示值唯一真源。decoder 输出的每个字段 key 都必须能查到中文字段名;查不到时红色拒绝,不得 fallback 成英文 key。固定业务展示值用 `field_value_zh` 登记,例如默认岗位、制度账户、机构费用付款账户；CitizenWallet decoder 只提供 `actor_cid_number` 等动态替换变量,不得在 decoder 里手写第二份中文值。

签名判定统一输出:

```text
Normal/正常: 绿色,允许签名
Reject/拒绝: 红色,禁止签名
```

不存在第三种状态。action 未登记、无中文动作名、无 decoder、payload 无法解码、字段无中文翻译、普通交易 hash-only,一律 `Reject/拒绝`。

CitizenApp / CitizenWallet 已按本规则接入生成产物:`citizenapp/lib/qr/generated/qr_action_registry.g.dart` 与 `citizenwallet/lib/qr/generated/qr_action_registry.g.dart`。钱包内 `action_labels.dart` / `field_labels.dart` 和公民端 `qr_protocols.dart` / `square_action_payload.dart` 只能消费生成产物;未登记 action 或字段不得显示英文/数字兜底,必须红色拒绝。

## 1. 非链动作码

| a | 名称 | payload | 签名字节 | 生成方 | 扫码/签名方 | 注释 |
|---:|---|---|---|---|---|---|
| 1 | `login` | `system|system_signature` UTF-8 | 原文 | OnChina | CitizenWallet | 登录签名确认 |
| 2 | `citizen_identity` | `VotingIdentityPayload` SCALE bytes | `blake2_256(GMB || 0x10 || payload)` | OnChina | CitizenWallet / CitizenApp(电子护照扫码签名页) | 公民本人确认链上投票身份载荷;CitizenApp 侧解码器在 `my/myid/voting_identity_payload.dart` |
| 3 | `onchina_admin_action` | `onchina_admin_governance` canonical JSON UTF-8 | 原文 | OnChina | CitizenWallet | 链上中国平台管理员治理冷钱包确认 |
| 5 | `activate_admin_account` | `GMB || 0x18` 二进制 payload | 原文 | citizenchain node / CitizenApp | CitizenWallet | 管理员激活 |
| 6 | `decrypt_admin` | `GMB || 0x19` 二进制 payload | 原文 | citizenchain node | CitizenWallet | 清算行管理员解密 |
| 7 | `runtime_upgrade_hash` | 32B WASM hash | 原文 32B | citizenchain node / CitizenApp | CitizenWallet | Runtime 升级哈希直签 |
| 9 | `square_account_action` | 广场账户动作 SCALE bytes（`action‖owner‖challenge_id[‖level]‖u64(expires)`） | `signing_message(OP_SIGN_SQUARE_ACTION, payload)` | 官网 citizenweb / CitizenApp | CitizenApp（交易 tab「扫一扫」，owner 主钥+生物识别） | 会员订阅/取消等账户动作链下签名；owner 由 QR `u` 在本机定位钱包，两色解码 `signer/square_action_payload.dart`；Worker `account/action_challenge.ts` 构造/验签 |

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
| `0x0700` | `PersonalManage.propose_create` | `propose_create_personal` | `account_name`, `admins(admin_account + family_name + given_name)`, `admins_len`, `regular_threshold`, `create_threshold`, `amount_yuan` | CitizenApp |
| `0x0701` | `PersonalManage.propose_close` | `propose_close_personal` | `account`, `beneficiary` | CitizenApp |
| `0x1d00` | `PersonalAdmins.propose_admin_set_change` | `propose_personal_admin_set_change` | `institution_code`, `account`, `admins`, `new_threshold` | CitizenApp |
| `0x0800` | `ResolutionIssuance.propose_issuance` | `propose_issuance` | `actor_cid_number`, `proposer_role_code`, `reason`, `amount_yuan`, `allocation_count`, `eligible_total`, `province_name`, `signer_pubkey` | citizenchain node / CitizenApp |
| `0x0903` | `VotingEngine.finalize_proposal` | `finalize_proposal` | `proposal_id` | citizenchain node / CitizenApp |
| `0x0904` | `VotingEngine.retry_passed_proposal` | `retry_passed_proposal` | `proposal_id` | citizenchain node / CitizenApp |
| `0x0905` | `VotingEngine.cancel_passed_proposal` | `cancel_passed_proposal` | `proposal_id`, `reason` | citizenchain node / CitizenApp |
| `0x0a00` | `CitizenIdentity.register_voting_identity` | `register_voting_identity` | `registrar_account`, `cid_number`, `wallet_account`, `citizen_age_years`, `valid_range`, `citizen_status`, `residence` | OnChina |
| `0x0c00` | `RuntimeUpgrade.propose_runtime_upgrade` | `propose_runtime_upgrade` | `wasm_hash` | citizenchain node / CitizenApp |
| `0x0c02` | `RuntimeUpgrade.developer_direct_upgrade` | `developer_direct_upgrade` | `wasm_hash` | citizenchain node / CitizenApp |
| `0x0d00` | `ResolutionDestro.propose_destroy` | `propose_destroy` | `actor_cid_number`, `proposer_role_code`, `institution_account`, `amount_yuan` | CitizenApp |
| `0x0f00` | `GrandpaKeyChange.propose_replace_grandpa_key` | `propose_replace_grandpa_key` | `actor_cid_number`, `proposer_role_code`, `new_key` | citizenchain node |
| `0x1602` | `ElectionVote.cast_popular_vote` | `cast_popular_vote` | `proposal_id`, `cid_number`, `wallet_account` | 未来具体公权选举业务模块 / CitizenApp |
| `0x1603` | `ElectionVote.cast_mutual_vote` | `cast_mutual_vote` | `proposal_id`, `voter_role_code`, `cid_number`, `wallet_account` | 未来具体公权选举业务模块 / CitizenApp |
| `0x1e01` | `PublicManage.propose_close_public_institution` | `propose_close_public_institution` | `actor_cid_number`, `proposer_role_code`, `institution_account`, `beneficiary`, `credential_issuer_cid_number`, `credential_signer_pubkey` | CitizenApp |
| `0x1e06` | `PublicManage.update_institution_info` | `update_public_institution_info` | `cid_number`, `cid_full_name`, `cid_short_name`, `actor_cid_number`, `credential_signer_pubkey`, `scope_*` | OnChina（注册局登记管理） |
| `0x1e07` | `PublicManage.add_institution_account` | `add_public_institution_account` | `cid_number`, `account_names`, `account_count`, `actor_cid_number`, `credential_signer_pubkey`, `scope_*` | OnChina（注册局登记管理） |
| `0x1e08` | `PublicManage.propose_institution_governance` | `propose_public_institution_governance` | `cid_number`, `governance_action`, `governance_detail`, `actor_cid_number`, `proposer_role_code`, `fee_payer` | OnChina（本机构治理） |
| `0x1e09` | `PublicManage.register_institution_admins` | `register_public_institution_admins` | `cid_number`, `admins(admin_account + cid_number + family_name + given_name)`, `actor_cid_number`, `fee_payer` | OnChina（注册局直接登记管理员） |
| `0x1f01` | `PrivateManage.propose_close_private_institution` | `propose_close_private_institution` | `actor_cid_number`, `proposer_role_code`, `institution_account`, `beneficiary`, `credential_issuer_cid_number`, `credential_signer_pubkey` | CitizenApp |
| `0x1f06` | `PrivateManage.update_institution_info` | `update_private_institution_info` | `cid_number`, `cid_full_name`, `cid_short_name`, `actor_cid_number`, `credential_signer_pubkey`, `scope_*` | OnChina（注册局登记管理） |
| `0x1f07` | `PrivateManage.add_institution_account` | `add_private_institution_account` | `cid_number`, `account_names`, `account_count`, `actor_cid_number`, `credential_signer_pubkey`, `scope_*` | OnChina（注册局登记管理） |
| `0x1f08` | `PrivateManage.propose_institution_governance` | `propose_private_institution_governance` | `cid_number`, `governance_action`, `governance_detail`, `actor_cid_number`, `proposer_role_code`, `fee_payer` | OnChina（本机构治理） |
| `0x1f09` | `PrivateManage.register_institution_admins` | `register_private_institution_admins` | `cid_number`, `admins(admin_account + family_name + given_name)`, `actor_cid_number`, `fee_payer` | OnChina（注册局直接登记管理员） |
| `0x2100` | `AddressRegistry.set_catalog_version` | `set_address_catalog_version` | `registrar_account`, `catalog_version`, `catalog_hash` | onchina |
| `0x2101` | `AddressRegistry.set_address_name` | `set_address_name` | `registrar_account`, `province_code`, `city_code`, `town_code`, `address_name_code`, `address_name` | onchina |
| `0x2102` | `AddressRegistry.remove_address_name` | `remove_address_name` | `registrar_account`, `province_code`, `city_code`, `town_code`, `address_name_code` | onchina |
| `0x2103` | `AddressRegistry.set_address` | `set_address` | `registrar_account`, `province_code`, `city_code`, `town_code`, `address_name_code`, `address_local_no`, `address_detail` | onchina |
| `0x2104` | `AddressRegistry.remove_address` | `remove_address` | `registrar_account`, `province_code`, `city_code`, `town_code`, `address_name_code`, `address_local_no`, `address_detail` | onchina |
| `0x1100` | `MultisigTransfer.propose_transfer` | `propose_transfer` | `actor_cid_number`, `proposer_role_code`, `institution`, `beneficiary`, `amount_yuan`, `remark` | citizenchain node / CitizenApp |
| `0x1101` | `MultisigTransfer.propose_safety_fund` | `propose_safety_fund_transfer` | `actor_cid_number`, `proposer_role_code`, `institution`, `beneficiary`, `amount_yuan`, `remark` | citizenchain node / CitizenApp |
| `0x1102` | `MultisigTransfer.propose_sweep` | `propose_sweep_to_main` | `actor_cid_number`, `proposer_role_code`, `institution`, `amount_yuan` | citizenchain node / CitizenApp |
| `0x2205` | `SquarePost.propose_set_platform_price` | `propose_set_platform_price` | `actor_cid_number`, `proposer_role_code`, `membership_level`, `new_price_fen` | OnChina |

永久留洞：`0x0702`、`0x0a05`、`0x1502`、`0x1a00`、`0x1e04`、`0x1e05`、`0x1f04`、`0x1f05`。
这些位置不进入 action registry，不允许 CitizenApp/CitizenWallet 构造、解码或兼容；
人口快照与拒绝终态清理都由 runtime 投票引擎内部完成。
| `0x1332` | `OffchainTransaction.register_clearing_bank` | `register_clearing_bank` | `cid_number`, `peer_id`, `rpc_domain`, `rpc_port` | citizenchain node |
| `0x1333` | `OffchainTransaction.update_clearing_bank_endpoint` | `update_clearing_bank_endpoint` | `cid_number`, `new_domain`, `new_port` | citizenchain node |
| `0x1334` | `OffchainTransaction.unregister_clearing_bank` | `unregister_clearing_bank` | `cid_number` | citizenchain node |
| `0x1400` | `InternalVote.cast` | `internal_vote` | `proposal_id`, `approve` | citizenchain node / CitizenApp |
| `0x1500` | `JointVote.cast_admin` | `joint_vote` | `proposal_id`, `approve` | citizenchain node / CitizenApp |
| `0x1501` | `JointVote.cast_referendum` | `cast_referendum` | `proposal_id`, `approve`, `province_name`, `signer_pubkey` | CitizenApp |
| `0x1602` | `ElectionVote.cast_popular_vote` | `cast_popular_vote` | `proposal_id`, `cid_number`, `wallet_account` | CitizenApp |
| `0x1603` | `ElectionVote.cast_mutual_vote` | `cast_mutual_vote` | `proposal_id`, `voter_role_code`, `cid_number`, `wallet_account` | CitizenApp |

## 3. 扫码端展示规则

扫码端不得读取 QR 中的展示摘要。展示内容只能来自:

1. `b.a` 定位业务动作。
2. `b.d` 解码出的 payload 字段。
3. 本地 action label / field label 注册表。

上表“payload 展示字段”是内部解码键，不是 UI 文案。扫码确认页必须把字段名映射成中文标签，例如 `institution_code` 显示为“机构类型”、`amount_yuan` 显示为“金额”、`proposal_id` 显示为“提案编号”、`approve` 显示为“投票意见”。

禁止用户界面显示以下内容作为签名确认:

- `动作 <数字>`
- 英文 action key
- 英文字段 key
- 原始 hex payload
- `载荷 <N> 字节`
- `unknown` / `unsupported` / `decode failed`

如果只能显示上述内容,说明本地 action registry、decoder 或中文字段表不完整,必须红色拒绝。

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
8. action 没有中文动作名。
9. decoder 输出字段缺少中文字段名。
10. 普通链交易 `d` 只有 32B hash/signing bytes,导致无法完整中文展示。

## 5. 修改流程

1. 先改本文件和 `qr-protocol-spec.md`。
2. 创建或更新 `citizenchain/crates/qr-protocol/registry/*` 唯一代码真源。
3. 从唯一真源生成或校验 Dart/TS/Rust 的 action 常量、中文标签、字段标签和 decoder 映射；当前 Dart 生成命令为 `cargo run --manifest-path citizenchain/crates/qr-protocol/Cargo.toml --bin export_registry -- --dart <输出文件>`。
4. 同步 `PayloadDecoder` 和签发方 `review_payload/signing_bytes` 构造。
5. 补跨端 fixture 和真实扫码签名测试。
6. 如需修改 `citizenchain/runtime/` 或 `citizenchain/runtime/primitives/src/sign.rs`,必须按 runtime 二次确认硬规则单独确认后再执行。

## 6. 移动端生成产物边界

当前移动端已按 registry 生成产物接入签名入口的中文动作名、字段名和拒签规则:

- 生成源:`citizenchain/crates/qr-protocol/registry/actions.yaml`、`fields.yaml`、`reject_reasons.yaml`。
- 生成器:`citizenchain/crates/qr-protocol/src/bin/export_registry.rs`。
- CitizenApp 产物:`citizenapp/lib/qr/generated/qr_action_registry.g.dart`。
- CitizenWallet 产物:`citizenwallet/lib/qr/generated/qr_action_registry.g.dart`。
- CitizenWallet 消费点:`citizenwallet/lib/signer/action_labels.dart`、`field_labels.dart`、`payload_decoder.dart`、`citizenwallet/lib/qr/qr_protocols.dart`。
- CitizenApp 消费点:`citizenapp/lib/qr/qr_protocols.dart`、`signer/square_action_payload.dart`。
- OnChina 非链动作码消费点:`citizenchain/onchina/src/core/qr/mod.rs` 通过 `qr-protocol` 读取 `login`、`citizen_identity`、`onchina_admin_action`，不得恢复 `ACTION_LOGIN = 1` 等硬编码常量。

生成文件只是移动端产物,不是第二真源。新增或调整 action 时必须先改
`citizenchain/crates/qr-protocol/registry/actions.yaml` / `fields.yaml`,再重新生成两端 Dart 文件并补测试。`qr-protocol` 的 `generated_dart_registries_are_current` 测试会校验两端生成文件没有漂移。

## 7. 防漂移 guard

`citizenchain/crates/qr-protocol/tests/repo_guard.rs` 是全仓 QR 签名协议防漂移护栏。它随
`cargo test --manifest-path citizenchain/crates/qr-protocol/Cargo.toml` 执行,扫描 CitizenApp、CitizenWallet、OnChina、node 和 `citizenchain/crates` 下的代码目录,禁止恢复:

- 移动端手写 action 中文表、action code 反查表、字段中文表。
- OnChina 非链 QR action code `1/2/3` 硬编码常量。
- 移动端手写 Runtime hash-only action 列表。
- 离线签名第三状态或可签名警告态。
- `动作 <数字>`、`载荷 N 字节` 等签名确认兜底展示。

新增扫码签名功能时,正确顺序固定为:先改 registry → 生成两端产物 → 补 decoder/签发方 → 跑 `qr-protocol` 测试。任何绕过 registry 的端侧手写分支都应被 guard 拦下。
