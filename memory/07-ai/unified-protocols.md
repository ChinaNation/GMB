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
  - `wuminapp/lib/duoqian/shared/duoqian_manage_service.dart`
  - `citizenchain/node/src/offchain/organization_manage/signing.rs`
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
  - `wuminapp/test/duoqian/duoqian_manage_service_test.dart`
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
  - `memory/08-tasks/open/20260507-p0-5-step2d-fixture.md`
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
  - `citizenchain/node/src/offchain/organization_manage/sfid.rs`
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
  - `memory/08-tasks/open/20260507-p0-4-immortal-era.md`
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
  - `flutter test test/duoqian test/proposal test/trade`

## 6. 待纳入登记

以下已有协议/契约还需要后续逐项登记到本文件：

- `InternalVote` / `JointVote` 投票载荷格式
- `PopulationSnapshot` 凭证载荷
- `ResolutionIssuance` 凭证载荷
- `DuoqianTransfer` 交易载荷格式
- `PersonalManage` 个人多签创建/关闭载荷格式
- `AdminsChange::Subjects` storage 契约
- `OrganizationManage::InstitutionAccounts` storage 契约
- `PersonalManage::PersonalDuoqians` storage 契约
- CPMS 安装 4 码契约
