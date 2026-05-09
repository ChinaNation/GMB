# GMB 统一协议文件

## 1. 定位

本文件是 GMB AI 编程系统的统一协议入口。

以后任何设计、修改、删除下列内容之前，必须先查本文件：

- 扫码协议
- 二维码 `kind` / `body` / `payload` 结构
- 链上交易 call data 字段顺序
- SCALE 编码载荷格式
- SFID / CPMS / wuminapp / citizenchain 之间的 API 契约
- 签名、验签、防重放、nonce、era、fixture 规则
- storage key、subject id、action、pallet/call index 等跨端字段契约

本文件负责统一“协议名称、边界、字段、规则、真源、测试”。详细技术文档可以继续放在 `memory/01-architecture/` 或 `memory/05-modules/`，但必须从本文件登记和跳转。

## 2. 强制规则

1. 不允许在代码、文档、测试里直接发明新协议名。新协议名必须先登记到本文件。
2. 不允许把“内层交易载荷格式”说成“新增扫码协议”。扫码协议和载荷格式必须分层命名。
3. 修改字段顺序、字段名、编码类型、签名 payload、nonce、era、pallet/call index 前，必须先更新本文件对应条目。
4. 每个协议条目必须写清楚：名称、类型、唯一真源、生产者、消费者、字段、编码、验收测试。
5. 详细协议文档自称“唯一事实源”时，必须在本文件有对应登记；否则不得自称唯一事实源。
6. 废弃协议不得直接删除，必须先在本文件标记 `废弃`，写清替代协议和清理范围。

## 3. 统一术语

| 术语 | 含义 | 是否扫码协议 |
|---|---|---|
| 扫码协议 | 二维码外层 envelope 和 kind 规则 | 是 |
| 签名请求 | `WUMIN_QR_V1` 下的 `kind = sign_request` | 否，属于扫码协议中的一种业务 kind |
| 交易载荷格式 | `payload_hex` 中某个链上 call data 的字段顺序和编码 | 否 |
| 接口契约 | HTTP / Tauri command / app API 的路径、字段和错误规则 | 否 |
| 凭证载荷 | SFID / CPMS 等系统签发给链端验签的 payload 字段 | 否 |
| storage 契约 | pallet storage 名称、key 类型、读取方和写入方规则 | 否 |

死规则：

```text
扫码协议只有一个：WUMIN_QR_V1。
payload_hex 里可以有很多不同交易载荷格式，但它们都不是新的扫码协议。
```

## 4. 协议登记模板

新增或修改协议时，按这个模板登记：

```text
### 编号：协议名称

- 状态：当前 / 草案 / 废弃
- 类型：扫码协议 / 交易载荷格式 / 接口契约 / 凭证载荷 / storage 契约
- 唯一真源：
- 详细文档：
- 生产者：
- 消费者：
- 字段：
- 编码：
- 签名/验签规则：
- 兼容策略：
- 禁止事项：
- 必跑测试：
```

## 5. 当前协议登记

### P-QR-001：WUMIN_QR_V1

- 状态：当前
- 类型：扫码协议
- 唯一真源：`memory/01-architecture/qr/qr-protocol-spec.md`
- 详细文档：
  - `memory/01-architecture/qr/qr-protocol-spec.md`
  - `memory/01-architecture/qr/qr-signing-recognition.md`
  - `memory/01-architecture/qr/qr-action-registry.md`
- 生产者：`wuminapp`、`citizenchain/node`、`sfid`、`cpms`
- 消费者：`wumin`、`wuminapp`、`sfid`、`cpms`
- 字段：以 `qr-protocol-spec.md` 为准
- 编码：JSON envelope
- 签名/验签规则：按各 `kind` 的 body 规则执行
- 兼容策略：开发期不做旧协议兼容
- 禁止事项：
  - 禁止新增 `WUMIN_QR_V2`
  - 禁止新增第二套扫码协议字符串
  - 禁止把某个 `payload_hex` 的交易载荷格式称为新扫码协议
- 必跑测试：QR fixture、wumin/wuminapp QR 解析测试

### P-QR-002：WUMIN_QR_V1 / sign_request

- 状态：当前
- 类型：扫码协议内业务 kind
- 唯一真源：`memory/01-architecture/qr/qr-signing-recognition.md`
- 详细文档：
  - `memory/01-architecture/qr/qr-signing-recognition.md`
  - `memory/01-architecture/qr/qr-action-registry.md`
- 生产者：`wuminapp`、`citizenchain/node`
- 消费者：`wumin`
- 字段：
  - `body.address`
  - `body.pubkey`
  - `body.sig_alg`
  - `body.payload_hex`
  - `body.display.action`
  - `body.display.fields`
  - `body.display.fields[*].key`
- 编码：外层 JSON；`payload_hex` 内部是具体链上 call data
- 签名/验签规则：
  - `payload_hex` 必须能被 `wumin` decoder 按对应交易载荷格式完整解码
  - `display.action` 必须和 decoder 得到的 action 字面一致
  - `display.fields[*].key/value` 必须和 decoder 得到的 fields 字面一致
- 兼容策略：开发期严格模式，不做别名兼容
- 禁止事项：
  - 禁止 display 字段和 decoder 字段不一致
  - 禁止未登记的 action 进入生产
  - 禁止恢复 `spec_version` envelope 门控
- 必跑测试：`wumin/test/signer/payload_decoder_test.dart`、QR sign request 测试

### P-TX-001：OrganizationManage.propose_create_institution

- 状态：当前
- 类型：交易载荷格式
- 唯一真源：`citizenchain/runtime/governance/organization-manage/src/lib.rs`
- 详细文档：
  - `memory/05-modules/citizenchain/node/offchain/NODE_CLEARING_BANK_TECHNICAL.md`
  - `memory/01-architecture/qr/qr-action-registry.md`
- 生产者：
  - `wuminapp/lib/organization-manage/shared/duoqian_manage_service.dart`
  - `citizenchain/node/src/governance/organization-manage/signing.rs`
- 消费者：
  - `citizenchain/runtime/governance/organization-manage`
  - `wumin/lib/signer/payload_decoder.dart`
- 字段：
  1. `sfid_number`
  2. `institution_name`
  3. `accounts`
  4. `admin_count`
  5. `duoqian_admins`
  6. `threshold`
  7. `register_nonce`
  8. `signature`
  9. `province`
  10. `signer_admin_pubkey`
- 编码：
  - SCALE call data
  - pallet index：`17`
  - call index：`5`
  - 前两个字节固定为 `[0x11, 0x05]`
- 签名/验签规则：
  - `register_nonce / signature / province / signer_admin_pubkey` 由 SFID 机构注册信息凭证提供
  - runtime 通过 `(province, signer_admin_pubkey)` 查省级签名公钥并验签
  - `accounts.account_name` 顺序必须与 SFID `/registration-info.account_names` 一致
- 兼容策略：开发期不兼容旧 `call_index=0`
- 禁止事项：
  - 禁止把本交易载荷称为新增扫码协议
  - 禁止继续使用旧 `propose_create call_index=0` 编码机构创建
  - 禁止在本载荷末尾追加 `a3 / sub_type / parent_sfid_number`
  - 禁止 wumin decoder 解码后仍有剩余字节
- 必跑测试：
  - `wuminapp/test/organization-manage/duoqian_manage_service_test.dart`
  - `wumin/test/signer/payload_decoder_test.dart`
  - `cargo check -p node`

### P-TX-002：JointVote.cast_referendum

- 状态：当前
- 类型：交易载荷格式
- 唯一真源：
  - `citizenchain/runtime/src/lib.rs`
  - `citizenchain/runtime/votingengine/joint-vote/src/lib.rs`
- 详细文档：
  - `memory/06-quality/fixtures/step2d_credential_payload.json`
  - `memory/08-tasks/done/20260507-p0-5-step2d-fixture.md`
- 生产者：
  - `wuminapp` 联合公投签名请求流程
  - Step2D fixture
- 消费者：
  - `citizenchain/runtime/votingengine/joint-vote`
  - `wumin/lib/signer/payload_decoder.dart`
- 字段：
  1. `proposal_id`
  2. `binding_id`
  3. `nonce`
  4. `signature`
  5. `province`
  6. `signer_admin_pubkey`
  7. `approve`
- 编码：
  - SCALE call data
  - pallet index：`23`
  - call index：`1`
  - 前两个字节固定为 `[0x17, 0x01]`
  - Step2D fixture 中 `expected_byte_length = 166`
- 签名/验签规则：
  - runtime 用 `(province, signer_admin_pubkey)` 查省级签名公钥并验签
  - `binding_id / nonce / signature` 必须来自 SFID 绑定投票凭证
- 兼容策略：开发期不兼容旧 `VotingEngine(9).call_index=2`
- 禁止事项：
  - 禁止 Step2D fixture 中继续出现 `cast_referendum` 的 `pallet_index=9 / call_index=2`
  - 禁止 `cast_referendum` fixture 继续使用 `0x0902` 前缀
  - 禁止 `wumin` 与 `wuminapp` 各自维护重复 Step2D fixture
- 必跑测试：
  - `wumin/test/signer/payload_decoder_test.dart`
  - `wumin/test/signer/pallet_registry_test.dart`
  - `wuminapp/test/proposal/runtime_upgrade/runtime_upgrade_service_test.dart`

### P-CRED-001：SFID institution registration-info credential

- 状态：当前
- 类型：凭证载荷 / 接口契约
- 唯一真源：`sfid/backend/institutions/chain_duoqian_info.rs`
- 详细文档：`memory/05-modules/citizenchain/node/offchain/NODE_CLEARING_BANK_TECHNICAL.md`
- 生产者：`sfid/backend/institutions/chain_duoqian_info.rs`
- 消费者：
  - `citizenchain/node/src/governance/organization-manage/sfid.rs`
  - `wuminapp` 机构创建流程
  - `citizenchain/runtime/governance/organization-manage`
- 字段：
  - 外层业务字段：`sfid_number`、`institution_name`、`account_names`
  - 凭证字段：`credential.register_nonce`、`credential.province`、`credential.signer_admin_pubkey`、`credential.signature`
- 编码：
  - HTTP JSON 响应
  - runtime 验签 payload 按 SFID 后端 `build_institution_registration_info_credential` 的 SCALE tuple 顺序
- 签名/验签规则：
  - SFID 后端用省级签名密钥签发
  - 链端用 `province + signer_admin_pubkey` 查验签公钥
- 兼容策略：不把 `a3 / sub_type / parent_sfid_number` 纳入链端注册凭证
- 禁止事项：
  - 禁止用普通机构详情接口替代 `/registration-info`
  - 禁止 wuminapp 自己拼 `register_nonce / signature / province / signer_admin_pubkey`
- 必跑测试：SFID 后端 registration-info 测试、P-TX-001 双端编码/解码测试

### P-SIGN-001：Citizenchain signed extrinsic era

- 状态：当前
- 类型：签名 / extrinsic 协议
- 唯一真源：
  - `citizenchain/node/src/governance/signing.rs`
  - `wuminapp/lib/rpc/signed_extrinsic_builder.dart`
- 详细文档：
  - `memory/08-tasks/done/20260507-p0-4-immortal-era.md`
- 生产者：
  - `citizenchain/node`
  - `wuminapp`
  - `wumin` 冷钱包提交链路
- 消费者：
  - `citizenchain runtime` signed extension 验签
- 字段：
  - `eraPeriod = 0`
  - `era bytes = 0x00`
  - `blockNumber = 0`
  - `SigningPayload.blockHash = genesisHash`
  - `ExtrinsicPayload.blockNumber = 0`
- 编码：
  - signed extension `CheckEra` 使用 immortal era 单字节 `0x00`
  - `CheckEra` additional signed hash 使用创世块哈希，即 `block_hash(0)`
- 签名/验签规则：
  - 签名前 payload 与最终 extrinsic body 必须使用同一份 immortal era 字节
  - 使用 polkadart 时必须传 `eraPeriod: 0`
  - `SigningPayload.blockHash` 必须传 `genesisHash`，不得传最新块 hash
- 兼容策略：开发期不兼容热钱包 mortal era
- 禁止事项：
  - 禁止业务 service 自己保留 `_eraPeriod = 64`
  - 禁止 signed extrinsic 构造路径调用 `fetchLatestBlock()` 参与 era 计算
  - 禁止把最新块 hash 写入 immortal era 的 signing payload
- 必跑测试：
  - `wuminapp/test/rpc/signed_extrinsic_builder_test.dart`
  - `flutter test test/organization-manage test/proposal test/trade`

### P-TX-003：InternalVote.cast

- 状态：当前
- 类型：交易载荷格式
- 唯一真源：`citizenchain/runtime/votingengine/internal-vote/src/lib.rs`
- 详细文档：
  - `memory/01-architecture/qr/qr-action-registry.md`
  - `wumin/lib/signer/pallet_registry.dart`
- 生产者：`wuminapp`、`citizenchain/node`
- 消费者：
  - `citizenchain/runtime/votingengine/internal-vote`
  - `wumin/lib/signer/payload_decoder.dart`
- 字段：
  1. `proposal_id`
  2. `approve`
- 编码：
  - SCALE call data
  - pallet index：`22`
  - call index：`0`
  - 前两个字节固定为 `[0x16, 0x00]`
- 签名/验签规则：
  - 管理员投票统一走 `InternalVote::cast`
  - 业务 pallet 不再承载 `vote_*` wrapper
- 兼容策略：开发期不兼容旧 `VotingEngine(9)` 投票入口
- 禁止事项：
  - 禁止恢复业务 pallet 内的投票 wrapper
  - 禁止把内部投票编码回 `VotingEngine(9)`
- 必跑测试：
  - `wumin/test/signer/payload_decoder_test.dart`
  - `wumin/test/signer/pallet_registry_test.dart`

### P-TX-004：JointVote.cast_admin

- 状态：当前
- 类型：交易载荷格式
- 唯一真源：`citizenchain/runtime/votingengine/joint-vote/src/lib.rs`
- 详细文档：
  - `memory/01-architecture/qr/qr-action-registry.md`
  - `wumin/lib/signer/pallet_registry.dart`
- 生产者：`wuminapp`、`citizenchain/node`
- 消费者：
  - `citizenchain/runtime/votingengine/joint-vote`
  - `wumin/lib/signer/payload_decoder.dart`
- 字段：
  1. `proposal_id`
  2. `subject_id`
  3. `approve`
- 编码：
  - SCALE call data
  - pallet index：`23`
  - call index：`0`
  - 前两个字节固定为 `[0x17, 0x00]`
- 签名/验签规则：
  - 联合投票的机构管理员阶段走 `JointVote::cast_admin`
  - `subject_id` 底层类型为 `SubjectId`
- 兼容策略：开发期不兼容旧 `VotingEngine(9)` 投票入口
- 禁止事项：
  - 禁止恢复旧联合投票 wrapper
  - 禁止把 `subject_id` 注释成当前 `InstitutionPalletId`
- 必跑测试：
  - `wumin/test/signer/payload_decoder_test.dart`
  - `wumin/test/signer/pallet_registry_test.dart`

### P-CRED-002：PopulationSnapshot

- 状态：当前
- 类型：凭证载荷
- 唯一真源：`citizenchain/node/src/governance/sfid_api.rs`
- 详细文档：
  - `memory/05-modules/citizenchain/node/governance/GOVERNANCE_TECHNICAL.md`
- 生产者：`sfid`
- 消费者：
  - `citizenchain/node`
  - `wuminapp`
  - `citizenchain/runtime/src/configs/mod.rs::RuntimePopulationSnapshotVerifier`
- 字段：
  1. `eligible_total`
  2. `snapshot_nonce`
  3. `signature`
  4. `province`
  5. `signer_admin_pubkey`
- 编码：
  - HTTP JSON 响应
  - 链端验签 payload 以 runtime verifier 当前实现为准
- 签名/验签规则：
  - SFID 省级管理员签发人口快照
  - runtime 按 `(province, signer_admin_pubkey)` 查省级签名公钥并验签
- 兼容策略：开发期不兼容缺少省份和签发管理员公钥的旧人口快照
- 禁止事项：
  - 禁止前端自行伪造 `eligible_total / snapshot_nonce / signature`
  - 禁止跳过 runtime 人口快照验签
- 必跑测试：
  - `citizenchain/runtime/src/tests/cases.rs` 中 population snapshot 相关测试
  - `wuminapp/test/proposal/runtime_upgrade/runtime_upgrade_service_test.dart`

### P-CRED-003：ResolutionIssuance.propose_resolution_issuance

- 状态：当前
- 类型：交易载荷格式 / 凭证载荷
- 唯一真源：`citizenchain/runtime/issuance/resolution-issuance/src/lib.rs`
- 详细文档：
  - `memory/05-modules/citizenchain/runtime/issuance/resolution-issuance/RESOLUTIONISSUANCE_TECHNICAL.md`
  - `wumin/lib/signer/payload_decoder.dart`
- 生产者：`citizenchain/node`、`wuminapp`
  - `citizenchain/node/src/duoqian_transfer/`
  - `citizenchain/node/frontend/duoqian-transfer/`
  - `wuminapp/lib/duoqian-transfer/`
- 消费者：
  - `citizenchain/runtime/issuance/resolution-issuance`
  - `wumin/lib/signer/payload_decoder.dart`
- 字段：
  1. `reason`
  2. `total_amount`
  3. `allocations`
  4. `eligible_total`
  5. `snapshot_nonce`
  6. `signature`
  7. `province`
  8. `signer_admin_pubkey`
- 编码：
  - SCALE call data
  - pallet index：`8`
  - call index：`0`
  - 前两个字节固定为 `[0x08, 0x00]`
- 签名/验签规则：
  - 人口快照字段来自 `P-CRED-002`
  - runtime 通过 `RuntimePopulationSnapshotVerifier` 验签
- 兼容策略：开发期不兼容缺少 `province / signer_admin_pubkey` 的旧载荷
- 禁止事项：
  - 禁止节点或前端自行改写人口快照凭证字段
  - 禁止把发行金额显示口径和链端 `u128` 分单位混用
- 必跑测试：
  - `wumin/test/signer/payload_decoder_test.dart`
  - `citizenchain/runtime/src/tests/cases.rs`

### P-TX-005：DuoqianTransfer proposal payloads

- 状态：当前
- 类型：交易载荷格式
- 唯一真源：`citizenchain/runtime/transaction/duoqian-transfer/src/lib.rs`
- 详细文档：
  - `memory/05-modules/citizenchain/runtime/transaction/duoqian-transfer/DUOQIAN_TRANSFER_TECHNICAL.md`
  - `wumin/lib/signer/payload_decoder.dart`
- 生产者：`citizenchain/node`、`wuminapp`
- 消费者：
  - `citizenchain/runtime/transaction/duoqian-transfer`
  - `wumin/lib/signer/payload_decoder.dart`
- 字段：
  - `propose_transfer(19.0)`：`org`、`subject_id`、`beneficiary`、`amount`、`remark`
  - `propose_safety_fund_transfer(19.1)`：`beneficiary`、`amount`、`remark`
  - `propose_sweep_to_main(19.2)`：`subject_id`、`amount`
- 编码：
  - SCALE call data
  - pallet index：`19`
  - call index：`0 / 1 / 2`
- 签名/验签规则：
  - 业务提案创建由对应管理员签名
  - 投票不走本 pallet，统一走 `P-TX-003`
  - `WUMIN_QR_V1 / sign_request` 展示字段必须使用 `institution / beneficiary / amount_yuan / remark`，禁止 node 使用旧 `org` 展示字段
- 兼容策略：`call_index=3 / 4 / 5` 留洞不复用
- 禁止事项：
  - 禁止恢复 `execute_transfer / execute_safety_fund / execute_sweep` wrapper
  - 禁止把 `subject_id` 注释成当前 `InstitutionPalletId`
  - 禁止在 `wuminapp/lib/proposal/transfer/`、`wuminapp/lib/organization-manage/`、`citizenchain/node/src/governance/`、`citizenchain/node/frontend/governance/` 或 `citizenchain/node/src/offchain/` 中实现多签转账业务
- 必跑测试：
  - `wumin/test/signer/payload_decoder_test.dart`
  - `cargo test --manifest-path citizenchain/runtime/transaction/duoqian-transfer/Cargo.toml`

### P-TX-006：PersonalManage proposal payloads

- 状态：当前
- 类型：交易载荷格式
- 唯一真源：`citizenchain/runtime/governance/personal-manage/src/lib.rs`
- 详细文档：
  - `memory/05-modules/citizenchain/runtime/governance/personal-manage/PERSONAL_MANAGE_TECHNICAL.md`
  - `wumin/lib/signer/payload_decoder.dart`
- 生产者：
  - `wuminapp/lib/personal-manage/personal_manage_service.dart`
- 消费者：
  - `citizenchain/runtime/governance/personal-manage`
  - `wumin/lib/signer/payload_decoder.dart`
- 字段：
  - `propose_create(7.0)`：`account_name`、`duoqian_admins`、`amount`
  - `propose_close(7.1)`：`duoqian_address`、`beneficiary`
- 编码：
  - SCALE call data
  - pallet index：`7`
  - call index：`0 / 1`
- ProposalData：
  - `MODULE_TAG = b"per-mgmt"`
  - `ACTION_CREATE = 0`：`duoqian_address`、`proposer`、`amount`、`fee`
  - `ACTION_CLOSE = 1`：`duoqian_address`、`beneficiary`、`proposer`
- 签名/验签规则：
  - 个人多签独立使用 `PersonalManage(7)` 与 `MODULE_TAG = b"per-mgmt"`
  - 投票统一走 `P-TX-003`
- 兼容策略：不兼容旧 `OrganizationManage(17).propose_create_personal`
- 禁止事项：
  - 禁止恢复 `OrganizationManage(17).call_index=3`
  - 禁止混用机构多签和个人多签 action 编号
- 必跑测试：
  - `cargo test --manifest-path citizenchain/Cargo.toml -p personal-manage --lib`
  - `wumin/test/signer/payload_decoder_test.dart`
  - `flutter test test/personal-manage/personal_manage_service_test.dart test/personal-manage/personal_manage_storage_codec_test.dart`

### P-STORAGE-001：AdminsChange.Subjects

- 状态：当前
- 类型：storage 契约
- 唯一真源：`citizenchain/runtime/governance/admins-change/src/lib.rs`
- 详细文档：
  - `memory/05-modules/citizenchain/runtime/governance/admins-change/ADMINSCHANGE_TECHNICAL.md`
  - `memory/04-decisions/ADR-010-subject-id-protocol.md`
- 生产者：`admins-change` 与各治理模块回调
- 消费者：
  - `citizenchain/runtime`
  - `citizenchain/node`
  - `wuminapp/lib/organization-manage/shared/admin_institution_codec.dart`
- 字段：
  - key：`SubjectId`
  - value：`org`、`kind`、`admins`、`threshold`、`creator`、`created_at`、`updated_at`、`status`
- 编码：
  - storage key：`twox128("AdminsChange") ++ twox128("Subjects") ++ hasher(SubjectId)`
  - value：SCALE
- 签名/验签规则：storage 本身不签名；写入必须来自链上授权流程
- 兼容策略：不兼容旧 `AdminsChange::Institutions` 当前路径
- 禁止事项：
  - 禁止恢复 `Institutions` storage 当前真源叙述
  - 禁止把 key 继续命名为当前 `InstitutionPalletId`
- 必跑测试：
  - admins-change 单测
  - wuminapp 多签发现相关测试

### P-STORAGE-002：OrganizationManage.InstitutionAccounts

- 状态：当前
- 类型：storage 契约
- 唯一真源：`citizenchain/runtime/governance/organization-manage/src/lib.rs`
- 详细文档：
  - `memory/05-modules/citizenchain/node/offchain/NODE_CLEARING_BANK_TECHNICAL.md`
- 生产者：`organization-manage`
- 消费者：
  - `citizenchain/node/src/governance/organization-manage/chain.rs`
  - `wuminapp/lib/organization-manage/shared/duoqian_storage_codec.dart`
- 字段：
  - key1：`sfid_number`
  - key2：`account_name`
  - value：机构账户信息，以 runtime 类型为准
- 编码：
  - double map storage key
  - value：SCALE
- 签名/验签规则：storage 本身不签名；写入必须来自机构创建和账户治理流程
- 兼容策略：不兼容旧 `DuoqianAccounts` mirror
- 禁止事项：
  - 禁止活跃代码继续读取 `OrganizationManage::DuoqianAccounts`
  - 禁止把机构账户当个人多签账户读取
- 必跑测试：
  - `wuminapp/test/organization-manage/duoqian_discovery_service_test.dart`
  - organization-manage 单测

### P-STORAGE-003：PersonalManage.PersonalDuoqians

- 状态：当前
- 类型：storage 契约
- 唯一真源：`citizenchain/runtime/governance/personal-manage/src/lib.rs`
- 详细文档：
  - `memory/05-modules/citizenchain/runtime/governance/personal-manage/PERSONAL_MANAGE_TECHNICAL.md`
- 生产者：`personal-manage`
- 消费者：
  - `wuminapp/lib/personal-manage/personal_manage_storage_codec.dart`
  - `wuminapp/lib/personal-manage/personal_manage_service.dart`
- 字段：
  - key：`personal_address`
  - value：`DuoqianAccount { creator, account_name, created_at, status }`
- 编码：
  - storage map key
  - value：SCALE
- 签名/验签规则：storage 本身不签名；创建和关闭由 `PersonalManage` 提案流程约束
- 兼容策略：不兼容旧 `OrganizationManage` 个人多签路径
- 禁止事项：
  - 禁止恢复 `OrganizationManage(17).propose_create_personal`
  - 禁止恢复已删除的个人多签反向索引 storage
  - 禁止把个人多签查询落回机构账户 storage
- 必跑测试：
  - `cargo test --manifest-path citizenchain/Cargo.toml -p personal-manage --lib`
  - `flutter test test/personal-manage/personal_manage_service_test.dart test/personal-manage/personal_manage_storage_codec_test.dart`

### P-STORAGE-004：Account-level internal admin subject

- 状态：当前（第 1 步已在 primitives/admins-change 落地，organization-manage 等业务模块仍需后续接入）
- 类型：storage 契约 / subject id 契约
- 唯一真源：`memory/04-decisions/ADR-015-account-admin-internal-vote.md`
- 详细文档：
  - `memory/04-decisions/ADR-015-account-admin-internal-vote.md`
  - `memory/05-modules/citizenchain/runtime/governance/admins-change/ADMINSCHANGE_TECHNICAL.md`
  - `memory/05-modules/citizenchain/runtime/votingengine/VOTINGENGINE_TECHNICAL.md`
- 生产者：
  - `citizenchain/runtime/governance/admins-change`
  - `citizenchain/runtime/governance/personal-manage`
  - `citizenchain/runtime/governance/organization-manage`
- 消费者：
  - `citizenchain/runtime/votingengine/internal-vote`
  - `citizenchain/runtime/transaction/*`
  - `wuminapp`
  - `wumin`
- 字段：
  - `subject_id`
  - `account_id`
  - `admins`
  - `admin_count`
  - `threshold`
  - `status`
- 生命周期事件：
  - `AdminSubjectPendingCreated { subject, org, kind, creator, admin_count, threshold }`
  - `AdminSubjectActivated { subject, org }`
  - `AdminSubjectPendingRemoved { subject, org }`
  - `AdminSubjectClosed { subject, org }`
- 编码：
  - 治理机构账户继续映射到既有 `SubjectKind::Builtin`
  - 注册个人账户继续映射到既有 `SubjectKind::PersonalDuoqian`
  - 注册机构账户使用账户级 `SubjectKind::InstitutionAccount = 0x05`，payload 为账户 `AccountId` 前 32 字节并右填零
  - `SubjectKind::SfidInstitution = 0x02` 保留为 SFID 机构归属/检索 ID，不作为新增账户级管理员主体
- 签名/验签规则：
  - 一人一票一笔交易，投票资格由创建提案时锁定的账户级管理员快照决定
  - 注册创建和注销关闭阈值为全员
  - 普通动态账户提案阈值由管理员数量派生
  - Pending 主体清理必须命中既有 Pending 主体，不存在时返回 `InvalidInstitution`
- 兼容策略：开发期彻底切换，不保留机构级管理员旧分支
- 禁止事项：
  - 禁止省储行质押账户进入内部投票
  - 禁止注册机构账户继续复用机构级管理员池
  - 禁止动态账户由用户自由输入阈值
  - 禁止把管理员增加、删除、更换、改阈值拆成四套提案
- 必跑测试：
  - `cargo test -p admins-change --lib`
  - `cargo test -p primitives --lib`
  - `cargo test -p internal-vote --lib`
  - `cargo test -p personal-manage --lib`
  - `cargo test -p organization-manage --lib`

### P-CPMS-001：SFID_CPMS_V1 installation four-code contract

- 状态：当前
- 类型：接口契约 / 凭证载荷
- 唯一真源：
  - `cpms/backend/src/initialize/mod.rs`
  - `cpms/backend/src/dangan/mod.rs`
- 详细文档：
  - `memory/01-architecture/cpms/CPMS_TECHNICAL.md`
  - `memory/05-modules/cpms/backend/initialize/INITIALIZE_TECHNICAL.md`
  - `memory/05-modules/cpms/backend/dangan/DANGAN_TECHNICAL.md`
- 生产者：
  - QR1 / QR3：`sfid`
  - QR2 / QR4：`cpms`
- 消费者：
  - QR1 / QR3：`cpms`
  - QR2 / QR4：`sfid`
- 字段：
  - QR1 `INSTALL`：`proto`、`type`、`sfid`、`token`、`rsa`、`sig`、`province_name`、`city_name`、`institution_name`
  - QR2 `REGISTER`：`proto`、`type`、`sfid`、`token`、`blind`
  - QR3 `CERT`：`proto`、`type`、`prov`、`bsig`
  - QR4 `ARCHIVE`：`proto`、`type`、`prov`、`ano`、`cs`、`ve`、`cert`、`sig`
- 编码：
  - JSON
  - `proto` 固定为 `SFID_CPMS_V1`
  - `type` 固定为 `INSTALL / REGISTER / CERT / ARCHIVE`
- 签名/验签规则：
  - QR1 由 SFID 签发，CPMS 离线初始化阶段按当前实现不验 SFID 签名
  - QR2 包含 CPMS 盲化请求
  - QR3 承载 SFID 返回的匿名证书
  - QR4 使用 CPMS 匿名私钥签名档案业务载荷并嵌入匿名证书
- 兼容策略：开发期不兼容其他 `proto`
- 禁止事项：
  - 禁止把 CPMS 安装四码混入 `WUMIN_QR_V1`
  - 禁止省级管理员登录二维码与 CPMS 业务二维码签名密钥混用
  - 禁止 QR4 在 QR3 未完成时签发
- 必跑测试：
  - CPMS initialize / dangan / qr 相关后端测试
  - `cargo check --manifest-path cpms/backend/Cargo.toml`

## 6. 登记维护要求

新增或修改协议时，必须在本文件按编号登记；无法确认字段时必须先向用户报告，不得把未确认字段写成当前协议。
