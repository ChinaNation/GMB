# GMB 统一协议文件

## 1. 定位

本文件是 GMB AI 编程系统的统一协议入口。

以后任何设计、修改、删除下列内容之前，必须先查本文件：

- 扫码协议
- 二维码 `kind` / `body` / `payload` 结构
- 链上交易 call data 字段顺序
- SCALE 编码载荷格式
- CID / CPMS / citizenapp / citizenchain 之间的 API 契约
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
| 签名请求 | `CITIZEN_QR_V1` 下的 `kind = sign_request` | 否，属于扫码协议中的一种业务 kind |
| 交易载荷格式 | `payload_hex` 中某个链上 call data 的字段顺序和编码 | 否 |
| 接口契约 | HTTP / Tauri command / app API 的路径、字段和错误规则 | 否 |
| 凭证载荷 | CID / CPMS 等系统签发给链端验签的 payload 字段 | 否 |
| storage 契约 | pallet storage 名称、key 类型、读取方和写入方规则 | 否 |

死规则：

```text
扫码协议只有一个：CITIZEN_QR_V1。
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
- 禁止兼容：
- 禁止事项：
- 必跑测试：
```

## 5. 当前协议登记

### P-CID-001：CID_NUMBER_V1

- 状态：当前
- 类型：接口契约 / 编码协议
- 唯一真源：`citizencode/backend/number/validator.rs`
- 详细文档：`memory/05-modules/citizencode/backend/number/NUMBER_TECHNICAL.md`
- 生产者：`citizencode/backend/number/generator.rs`
- 消费者：`citizencode/backend`、`citizencode/frontend`、`cpms`、`citizenapp`、`citizenchain`
- 字段：
  1. `R5`:省码 2 位 + 市码 3 位
  2. `K3`:主体属性 `K1` + 机构类型 `T2`
  3. `P1`:盈利属性
  4. `C1`:校验位
  5. `N9`:9 位稳定散列序列
  6. `D4`:年份
- 编码：`R5-K3P1C1-N9-D4`,示例 `LN001-NRC0G-944805165-2026`
- 签名/验签规则：本协议只定义身份号码格式;链上或二维码签名按对应协议条目执行。
- 禁止兼容：不兼容历史格式,不保留历史字段别名。
- 禁止事项：
  - 禁止在 CID 内部继续使用身份字段别名
  - 禁止恢复独立历史主体属性段
  - 禁止跳过 `C1` 校验
- 必跑测试：`cargo test --manifest-path citizencode/backend/Cargo.toml number::`

### P-QR-001：CITIZEN_QR_V1

- 状态：当前
- 类型：扫码协议
- 唯一真源：`memory/01-architecture/qr/qr-protocol-spec.md`
- 详细文档：
  - `memory/01-architecture/qr/qr-protocol-spec.md`
  - `memory/01-architecture/qr/qr-signing-recognition.md`
  - `memory/01-architecture/qr/qr-action-registry.md`
- 生产者：`citizenapp`、`citizenchain/node`、`cid`、`cpms`
- 消费者：`citizenwallet`、`citizenapp`、`cid`、`cpms`
- 字段：以 `qr-protocol-spec.md` 为准
- 编码：JSON envelope
- 签名/验签规则：按各 `kind` 的 body 规则执行
- 禁止兼容：开发期不做旧协议兼容
- 禁止事项：
  - 禁止新增 `CITIZEN_QR_V2`
  - 禁止新增第二套扫码协议字符串
  - 禁止把某个 `payload_hex` 的交易载荷格式称为新扫码协议
- 必跑测试：QR fixture、citizenwallet/citizenapp QR 解析测试

### P-QR-002：CITIZEN_QR_V1 / sign_request

- 状态：当前
- 类型：扫码协议内业务 kind
- 唯一真源：`memory/01-architecture/qr/qr-signing-recognition.md`
- 详细文档：
  - `memory/01-architecture/qr/qr-signing-recognition.md`
  - `memory/01-architecture/qr/qr-action-registry.md`
- 生产者：`citizenapp`、`citizenchain/node`、`cpms`
- 消费者：`citizenwallet`
- 字段：
  - `body.address`
  - `body.pubkey`
  - `body.sig_alg`
  - `body.payload_hex`
  - `body.display.action`
  - `body.display.fields`
  - `body.display.fields[*].key`
- 编码：外层 JSON；`payload_hex` 内部是具体链上 call data 或已登记的链下业务载荷
- 签名/验签规则：
  - `payload_hex` 必须能被 `citizenwallet` decoder 按对应交易载荷格式完整解码
  - `display.action` 必须和 decoder 得到的 action 字面一致
  - `display.fields[*].key/value` 若出现,必须和 decoder 得到的验真字段字面一致
  - 用户确认页只展示 decoder 产出的中文 `reviewFields`;账户字段必须为 SS58 地址
  - `cpms archive_delete` 的 payload 固定为 `CPMS_ARCHIVE_DELETE_V1|challenge_id|archive_id|archive_no|admin_account|expires_at`
  - 抗量子升级(ADR-022):`body.sig_alg` 扩为枚举 `sr25519 | ml-dsa-65`,新增 `auth_mode(normal|pqc|bootstrap-pqc)`、`key_version`、`chunk_index/chunk_total`(ML-DSA ~3.3KB 分片,最坏体积按 bootstrap);冷热两端 decoder 按 `sig_alg`/`auth_mode` 分流。未实现前 `sig_alg` 仍只接受 `sr25519`。
- 禁止兼容：开发期严格模式，不做别名兼容
- 禁止事项：
  - 禁止 display 字段和 decoder 字段不一致
  - 禁止未登记的 action 进入生产
  - 禁止把内部哈希、nonce、原始公钥 hex 当作普通用户确认字段展示
- 必跑测试：`citizenwallet/test/signer/payload_decoder_test.dart`、QR sign request 测试

### P-QR-003：CITIZEN_QR_V1 / im_node_pairing

- 状态：当前
- 类型：扫码协议内业务 kind
- 唯一真源：`citizenapp/lib/qr/bodies/im_node_pairing_body.dart`
- 详细文档：
  - `memory/05-modules/citizenapp/im/IM_TECHNICAL.md`
  - `memory/05-modules/citizenchain/node/NODE_TECHNICAL.md`
- 生产者：`citizenchain/node/src/settings/communication_node/mod.rs`
- 消费者：`citizenapp/lib/im/im_node_settings_page.dart`
- 字段：
  - `body.proto = GMB_IM_NODE_PAIRING_V1`
  - `body.node_peer_id`
  - `body.node_multiaddr`
  - `body.endpoint_kind`
- 编码：外层 `CITIZEN_QR_V1` 固定码；body 为 JSON 对象；顶层不包含 `id` / `issued_at` / `expires_at`。
- 签名/验签规则：本二维码只用于把公民手机配对到用户自己的电脑通信节点；钱包聊天账户授权仍走 `GMB_IM_WALLET_BINDING_V1` 钱包签名，不在二维码内携带钱包私钥或交易载荷。
- 禁止兼容：不兼容旧联系人码、旧 IM 联系人 bundle 或旧 `communication` 模式字段。
- 禁止事项：
  - 禁止用本二维码添加联系人。
  - 禁止把本二维码作为交易、转账、治理或 CID 身份码处理。
  - 禁止把通信节点配对做成全节点模式选项；归档/普通全节点模式与通信节点功能必须分离。
  - 禁止在本二维码中携带 RPC URL、临时 nonce 或有效期。
- 必跑测试：`flutter test test/qr/im_node_pairing_body_test.dart test/im/im_node_settings_page_test.dart`

### P-IM-001：GMB_IM_V1

- 状态：当前
- 类型：接口契约 / 编码协议
- 唯一真源：`citizenapp/im/proto/im_envelope.proto`
- Dart 生成物：`citizenapp/lib/im/proto/im_envelope.pb.dart`、`citizenapp/lib/im/proto/im_envelope.pbenum.dart`、`citizenapp/lib/im/proto/im_envelope.pbjson.dart`
- 详细文档：
  - `memory/04-decisions/ADR-020-citizenapp-p2p-im.md`
  - `memory/05-modules/citizenapp/im/IM_TECHNICAL.md`
- 生产者：`citizenapp/lib/im/`、`citizenchain/node/src/im/`
- 消费者：`citizenapp/lib/im/`、`citizenchain/node/src/im/`
- 字段：
  - `ImEnvelope.protocol_version`
  - `ImEnvelope.envelope_id`
  - `ImEnvelope.conversation_id`
  - `ImEnvelope.sender_chat_account`
  - `ImEnvelope.recipient_chat_account`
  - `ImEnvelope.sender_device_id`
  - `ImEnvelope.mls_wire_message`
  - `ImEnvelope.encrypted_metadata`
  - `ImEnvelope.attachment_manifest_hash`
  - `ImEnvelope.chunk_refs`
  - `ImEnvelope.created_at_millis`
  - `ImEnvelope.ttl_millis`
  - `ImEnvelope.ack_policy`
  - `ImEnvelope.mls_message_kind`
  - `ImEnvelope.ratchet_tree`
  - `ImMlsWireMessageKind.IM_MLS_WIRE_MESSAGE_KIND_UNSPECIFIED`
  - `ImMlsWireMessageKind.IM_MLS_WIRE_MESSAGE_KIND_WELCOME`
  - `ImMlsWireMessageKind.IM_MLS_WIRE_MESSAGE_KIND_APPLICATION`
  - `ImRouteRecord.proto`
  - `ImRouteRecord.wallet_chat_account`
  - `ImRouteRecord.route_display_name`
  - `ImRouteRecord.im_device_id`
  - `ImRouteRecord.im_device_pubkey_hex`
  - `ImRouteRecord.safety_number`
  - `ImRouteRecord.node_peer_id`
  - `ImRouteRecord.node_multiaddr`
  - `ImRouteRecord.created_at_millis`
  - `ImRouteRecord.updated_at_millis`
  - `ImNodeEndpoint.peer_id`
  - `ImNodeEndpoint.multiaddr`
  - `ImNodeEndpoint.kind`
  - `RegisterImDeviceRequest.wallet_account`
  - `RegisterImDeviceRequest.im_device_id`
  - `RegisterImDeviceRequest.im_device_pubkey`
  - `RegisterImDeviceRequest.node_peer_id`
  - `RegisterImDeviceRequest.node_endpoints`
  - `RegisterImDeviceRequest.expires_at_millis`
  - `RegisterImDeviceRequest.nonce`
  - `RegisterImDeviceRequest.wallet_signature`
  - `SubmitImEnvelopeRequest.mailbox_owner_chat_account`
  - `SubmitImEnvelopeRequest.envelope`
  - `ImDirectDeliveryRequest.remote_endpoint`
  - `ImDirectDeliveryRequest.envelope`
  - `ImNetworkRequest.kind`
  - `ImNetworkRequest.body`
  - `ImNetworkResponse.kind`
  - `ImNetworkResponse.body`
  - `ImEnvelopeAck.envelope_id`
  - `ImEnvelopeAck.state`
  - `ImKeyPackage.protocol_version`
  - `ImKeyPackage.owner_wallet_account`
  - `ImKeyPackage.device_id`
  - `ImKeyPackage.device_public_key_hex`
  - `ImKeyPackage.key_package_id`
  - `ImKeyPackage.key_package`
  - `ImKeyPackage.cipher_suite`
  - `ImKeyPackage.created_at_millis`
  - `ImKeyPackage.expires_at_millis`
  - `ImKeyPackage.consumed_at_millis`
  - `PublishImKeyPackageRequest.owner_wallet_account`
  - `PublishImKeyPackageRequest.device_id`
  - `PublishImKeyPackageRequest.device_public_key_hex`
  - `PublishImKeyPackageRequest.key_package_id`
  - `PublishImKeyPackageRequest.key_package`
  - `PublishImKeyPackageRequest.cipher_suite`
  - `PublishImKeyPackageRequest.created_at_millis`
  - `PublishImKeyPackageRequest.expires_at_millis`
  - `FetchImKeyPackagesRequest.owner_wallet_account`
  - `FetchImKeyPackagesRequest.requester_chat_account`
  - `FetchImKeyPackagesRequest.limit`
  - `ConsumeImKeyPackageRequest.owner_wallet_account`
  - `ConsumeImKeyPackageRequest.key_package_id`
  - `ConsumeImKeyPackageRequest.requester_chat_account`
  - `ImDirectKeyPackageFetchRequest.remote_endpoint`
  - `ImDirectKeyPackageFetchRequest.fetch`
  - `ImDirectKeyPackageConsumeRequest.remote_endpoint`
  - `ImDirectKeyPackageConsumeRequest.consume`
- 验收接口：
  - 正式通信节点功能禁止通过节点 RPC 连接公民手机；手机到自己电脑通信节点必须走后续专用 IM P2P 通道。
  - IM 验收不得恢复节点 RPC；需要运行态验收时应走 `/gmb/im/1`、Tauri 内部命令或后续专用 P2P 通道测试入口。
- 编码：外层 Protobuf；OpenMLS 标准 wire bytes 放入 `mls_wire_message`；链内 SCALE 不作为 IM 主协议。
- 当前实现状态：Dart Protobuf 生成与 `ImEnvelope` / `ImKeyPackage` / `ImRouteRecord` round-trip 已通过；`ImEnvelope` 已正式承载 MLS message kind 与 Welcome ratchet tree；OpenMLS native 边界通过现有 `libsmoldot` C ABI 调用 Rust OpenMLS，可生成真实 KeyPackage、返回设备签名公钥、完成两方 round-trip smoke、创建持久化 MLS 会话、处理 Welcome、解密 application，并在 App 重启后恢复同一会话；公民端已新增 Isar 消息库、远程消息收发状态机、IM 路由缓存、联系人详情消息入口、信息 Tab 会话列表和“我的 -> 设置 -> 设置通信节点”配对页；桌面节点已把通信节点功能拆成独立开关并生成不含 RPC URL、不含有效期的 `im_node_pairing` 固定二维码；手机到自己通信节点的专用 P2P 通道仍待后续任务接入。
- 签名/验签规则：
  - `ImRouteRecord` 是 IM 模块内部路由缓存，不是第二套通讯录，不得替代“我的通讯录”联系人详情。
  - 公民端发消息必须读取用户资料中的通信账户；未设置通信账户不得发送。
  - 钱包账户对 IM 设备、公钥、通信节点 PeerId、端点、过期时间和 nonce 做绑定签名。
  - 绑定签名 payload 固定为 `GMB_IM_WALLET_BINDING_V1|wallet_account|im_device_id|im_device_pubkey|node_peer_id|node_endpoints|expires_at_millis|nonce`。
  - 钱包私钥只用于绑定证明，不作为 IM 消息加密密钥。
  - KeyPackage 由 IM 设备密钥管理，必须具备 TTL、一次性消费或租约消费、防重放和撤销清理。
  - 首次 MLS 会话发送会产生 Welcome + application 两条 wire message；Welcome 必须通过 `ImEnvelope.ratchet_tree` 伴随传递 ratchet tree bytes。
  - 通信节点只接受 `mailbox_owner_chat_account == ImEnvelope.recipient_chat_account` 的密文信封。
  - `/gmb/im/1` outbound 必须使用 IM 路由记录或用户自有配置中的显式 `PeerId + multiaddr`，先写入 sc-network 地址簿，再发 request。
  - 通信节点 mailbox 持久化路径固定为 `base-path/im/mailbox.json`，不得落入链数据库或 CID 目录。
  - 通信节点 KeyPackage 池持久化路径固定为 `base-path/im/keypackages.json`，不得落入链数据库或 CID 目录。
- 禁止兼容：开发期不兼容未登记字段、未登记协议名或旧 Matrix / Olm / Megolm 主协议口径。
- 禁止事项：
  - 禁止把 CID 号码、实名信息、身份档案字段写入 IM 协议。
  - 禁止把 IM 路由缓存做成第二套通讯录。
  - 禁止通信节点作为第三方 Relay、公共 DHT、公共 rendezvous 或第三方 mailbox。
  - 禁止复用钱包私钥作为 IM 端到端加密密钥。
  - 禁止 `/gmb/im/1` 直连投递依赖公共发现或第三方中继。
- 必跑测试：`cargo test -p node im::`、`cargo test -p node settings::node_mode`、`cargo test -p node settings::communication_node`、`cargo test`（`citizenapp/rust`）、`flutter test --concurrency=1 test/qr/im_node_pairing_body_test.dart test/im/im_node_settings_page_test.dart`、`flutter test --concurrency=1 test/im/im_route_cache_store_test.dart`、`flutter test --concurrency=1 test/im/im_tab_page_test.dart test/im/im_envelope_proto_test.dart test/im/im_mls_native_test.dart`、`flutter test --concurrency=1 test/im/im_mls_session_test.dart test/im/im_mls_native_session_test.dart`、`flutter test --concurrency=1 test/im/im_envelope_session_test.dart test/im/im_isar_store_test.dart test/im/im_chat_ui_adapter_test.dart`、Protobuf 跨端 round-trip、OpenMLS 加解密测试、KeyPackage 防重放测试、通信节点密文落盘检查。
- 当前运行态 smoke：`citizenchain/scripts/im-two-node-smoke.sh` 必须通过，验证两个真实 headless 节点之间的 `/gmb/im/1` KeyPackage 发布/重启恢复/直连拉取/消费、直连投递、B 节点重启恢复 pending、授权设备拉取/ack、ack 后重启不重复、第三方 mailbox 拒绝和 ack 后重复投递不入队。

### P-TX-001：OrganizationManage.propose_create_institution

- 状态：当前
- 类型：交易载荷格式
- 唯一真源：`citizenchain/runtime/governance/organization-manage/src/lib.rs`
- 详细文档：
  - `memory/05-modules/citizenchain/node/offchain/NODE_CLEARING_BANK_TECHNICAL.md`
  - `memory/01-architecture/qr/qr-action-registry.md`
- 生产者：
  - `citizenapp/lib/governance/organization-manage/account_manage_service.dart`
  - `citizenchain/node/src/governance/organization_manage/signing.rs`
- 消费者：
  - `citizenchain/runtime/governance/organization-manage`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 字段：
  1. `cid_number`
  2. `cid_full_name`
  3. `accounts`
  4. `org`
  5. `admins_len`
  6. `admins`
  7. `threshold`
  8. `register_nonce`
  9. `signature`
  10. `issuer_cid_number`
  11. `issuer_main_account`
  12. `signer_pubkey`
  13. `scope_province_name`
  14. `scope_city_name`
- 编码：
  - SCALE call data
  - pallet index：`17`
  - call index：`5`
  - 前两个字节固定为 `[0x11, 0x05]`
- 签名/验签规则：
  - `register_nonce / signature / issuer_cid_number / issuer_main_account / signer_pubkey / scope_*` 由 CID 机构注册信息凭证提供
  - runtime 通过 `issuer_main_account` 查询 `admins-change::AdminAccounts`,确认 `signer_pubkey` 属于该机构 `admins` 后验签
  - `accounts.account_name` 顺序必须与 CID `/registration-info.account_names` 一致
- 禁止兼容：开发期不兼容旧 `call_index=0`
- 禁止事项：
  - 禁止把本交易载荷称为新增扫码协议
  - 禁止继续使用旧 `propose_create call_index=0` 编码机构创建
  - 禁止在本载荷末尾追加 `subject_property / private_type / partnership_kind / parent_cid_number`
  - 禁止 citizenwallet decoder 解码后仍有剩余字节
- 必跑测试：
  - `citizenapp/test/governance/organization-manage/account_manage_service_test.dart`
  - `citizenwallet/test/signer/payload_decoder_test.dart`
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
  - `citizenapp` 联合公投签名请求流程
  - Step2D fixture
- 消费者：
  - `citizenchain/runtime/votingengine/joint-vote`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 字段：
  1. `proposal_id`
  2. `binding_id`
  3. `nonce`
  4. `signature`
  5. `issuer_cid_number`
  6. `issuer_main_account`
  7. `signer_pubkey`
  8. `scope_province_name`
  9. `scope_city_name`
  10. `approve`
- 编码：
  - SCALE call data
  - pallet index：`23`
  - call index：`1`
  - 前两个字节固定为 `[0x17, 0x01]`
  - Step2D fixture 中 `expected_byte_length = 235`
- 签名/验签规则：
  - runtime 通过 `issuer_main_account` 查询 `admins-change::AdminAccounts`,确认 `signer_pubkey` 属于该机构 `admins` 后验签
  - `binding_id / nonce / signature` 必须来自 CID 绑定投票凭证
- 禁止兼容：开发期不兼容旧 `VotingEngine(9).call_index=2`
- 禁止事项：
  - 禁止 Step2D fixture 中继续出现 `cast_referendum` 的 `pallet_index=9 / call_index=2`
  - 禁止 `cast_referendum` fixture 继续使用 `0x0902` 前缀
  - 禁止 `citizenwallet` 与 `citizenapp` 各自维护重复 Step2D fixture
- 必跑测试：
  - `citizenwallet/test/signer/payload_decoder_test.dart`
  - `citizenwallet/test/signer/pallet_registry_test.dart`
  - `citizenapp/test/proposal/runtime_upgrade/runtime_upgrade_service_test.dart`

### P-CRED-001：CID subject registration-info credential

- 状态：当前
- 类型：凭证载荷 / 接口契约
- 唯一真源：`citizencode/backend/subjects/chain_duoqian_info.rs`
- 详细文档：`memory/05-modules/citizenchain/node/offchain/NODE_CLEARING_BANK_TECHNICAL.md`
- 生产者：`citizencode/backend/subjects/chain_duoqian_info.rs`
- 消费者：
  - `citizenchain/node/src/governance/organization_manage/cid.rs`
  - `citizenapp` 机构创建流程
  - `citizenchain/runtime/governance/organization-manage`
- 字段：
  - 外层业务字段：`cid_number`、`cid_full_name`、`account_names`
  - 凭证字段：`credential.register_nonce`、`credential.issuer_cid_number`、`credential.issuer_main_account`、`credential.signer_pubkey`、`credential.scope_province_name`、`credential.scope_city_name`、`credential.signature`
- 编码：
  - HTTP JSON 响应
  - runtime 验签 payload 按 CID 后端 `build_institution_registration_info_credential` 的 SCALE tuple 顺序
- 签名/验签规则：
  - CID 后端用签发机构管理员密钥签发。
  - 链端用 `issuer_main_account` 读取 `admins-change::AdminAccounts`，确认 `signer_pubkey` 属于该机构 `admins` 后验签。
  - `scope_province_name / scope_city_name` 只表示业务作用域，不表示签发人身份。
- 禁止兼容：不把 `subject_property / private_type / partnership_kind / parent_cid_number` 纳入链端注册凭证
- 禁止事项：
  - 禁止用普通机构详情接口替代 `/registration-info`
  - 禁止 citizenapp 自己拼 `register_nonce / signature / issuer_cid_number / issuer_main_account / signer_pubkey / scope_*`
- 必跑测试：CID 后端 registration-info 测试、P-TX-001 双端编码/解码测试

### P-CRED-002：CID_CPMS_V1 / ARCHIVE

- 状态：当前
- 类型：凭证载荷 / 接口契约
- 唯一真源：`memory/05-modules/citizencode/CID-CPMS-QR-v1.md`
- 详细文档：
  - `memory/05-modules/citizencode/CID-CPMS-QR-v1.md`
  - `memory/05-modules/citizenpassport/backend/archive/ARCHIVE_TECHNICAL.md`
- 生产者：`citizenpassport/backend/archive/mod.rs`
- 消费者：
  - `citizencode/backend/citizenpassport/handler.rs`
  - `citizencode/backend/citizens/binding.rs`
- 字段：
  - `proto`
  - `type`
  - `archive_no`
  - `citizen_status`
  - `voting_eligible`
  - `valid_from`
  - `valid_until`
  - `status_updated_at`
  - `cpms_pubkey`
  - `geo_seal`
  - `wallet_address`
  - `wallet_pubkey`
  - `wallet_sig_alg`
  - `sig`
- 编码：HTTP / QR JSON 载荷；`geo_seal` 为 AES-256-GCM 密文。
- 签名/验签规则：
  - CPMS 不生成钱包签名 challenge，不保存钱包签名；CPMS 只扫描 citizenapp 钱包地址二维码并保存 `wallet_address / wallet_pubkey`。
  - CPMS 签名原文固定为 `cid-cpms-v1|archive|{archive_no}|{citizen_status}|{voting_eligible}|{valid_from}|{valid_until}|{status_updated_at}|{cpms_pubkey}|{geo_seal_hash}|{wallet_address}|{wallet_pubkey}`。
  - CID 先用授权 `install_secret` 解 `geo_seal`，再用 `cpms_pubkey` 验 `sig`。
  - CID 绑定阶段必须要求 citizenapp 对 CID 绑定 challenge 签名，并校验签名公钥等于 ARCHIVE 中的 `wallet_pubkey`。
  - CID 绑定成功后直接形成本地电子护照绑定结果，并通过 citizenapp 状态查询接口返回；绑定公民电子护照流程不再设计额外确认步骤。
  - CID 正式绑定必须以有效 ARCHIVE 为入口，不允许先保存空钱包账户；按 `archive_no / cid_number / wallet_pubkey` 三者一对一约束拒绝重复绑定。
  - 同一 `archive_no` 已存在绑定记录时，只允许 `status_updated_at` 更新的 ARCHIVE 更新公民状态、选举资格、有效期或钱包字段；旧时间戳档案码必须拒绝，防止旧码覆盖新状态。
- 禁止兼容：开发期不兼容历史缩写字段名或历史签名原文。
- 禁止事项：
  - 禁止在 ARCHIVE 中新增 `code_id`。
  - 禁止在 ARCHIVE 中新增 `usage_limit`。
  - 禁止维护独立“已消费档案码”记录替代三者绑定唯一关系。
  - 禁止在公民电子护照绑定流程中设计、实现或描述二次确认步骤。
- 必跑测试：CPMS 后端 ARCHIVE 签名测试、CID 后端 ARCHIVE 验签测试、CID 前端公民列表构建。

### P-CRED-003：CID_CPMS_V1 / CPMS_STATUS_EXPORT

- 状态：当前
- 类型：凭证载荷 / 接口契约
- 唯一真源：`memory/05-modules/citizencode/CID-CPMS-QR-v1.md`
- 详细文档：
  - `memory/05-modules/citizencode/CID-CPMS-QR-v1.md`
  - `memory/05-modules/citizenpassport/backend/archive/ARCHIVE_TECHNICAL.md`
- 生产者：`citizenpassport/backend/archive/export.rs`
- 消费者：CID 后续导入模块
- 字段：
  - `proto`
  - `type`
  - `version`
  - `export_year`
  - `cid_number`
  - `cpms_pubkey`
  - `export_batch_id`
  - `exported_at`
  - `status_records_count`
  - `archive_release_records_count`
  - `records_hash`
  - `status_records`
  - `archive_release_records`
  - `sig`
- 编码：离线 JSON 文件。
- 签名/验签规则：
  - `records_hash = blake2b_256(json({status_records, archive_release_records}))`。
  - CPMS 签名原文固定为 `cid-cpms-v1|cpms-status-export|{cid_number}|{cpms_pubkey}|{export_batch_id}|{exported_at}|{records_hash}`。
  - CID 导入时必须先校验授权 CPMS、`records_hash` 和 CPMS 签名。
- 业务规则：
  - CPMS 从 UTC 每年 1 月 1 日起允许管理员导出上一年度更新数据；多年未导出时按最早未导出年度依次补导。
  - UTC 1 月 11 日起，如果存在已超过 1 月 10 日仍未导出的年度报告，CPMS 锁定操作员登录和已有会话。
  - `status_records` 按 `citizen_status_updated_at` 落入 `export_year` 过滤，`archive_release_records` 按 `released_at` 落入 `export_year` 过滤。
- 禁止兼容：开发期不兼容旧字段、旧状态名或无签名导出文件。
- 禁止事项：
  - 禁止导出姓名、出生日期、地址、护照号、钱包地址等实名或 CPMS 内部号码/绑定细节。
  - 禁止把硬删除释放记录当作公民状态更新。
  - 禁止 CPMS 通过联网方式向 CID 推送导出结果。
- 必跑测试：CPMS 后端状态导出构造测试、CPMS 后端 clippy、CPMS 后端 cargo test。

### P-SIGN-001：Citizenchain signed extrinsic era

- 状态：当前
- 类型：签名 / extrinsic 协议
- 唯一真源：
  - `citizenchain/node/src/governance/signing.rs`
  - `citizenapp/lib/rpc/signed_extrinsic_builder.dart`
- 详细文档：
  - `memory/08-tasks/done/20260507-p0-4-immortal-era.md`
- 生产者：
  - `citizenchain/node`
  - `citizenapp`
  - `citizenwallet` 公民钱包提交链路
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
- 适用范围：
  - 本协议仅约束 **sr25519 外层签名**的 signed extrinsic；PQC(ML-DSA-65)交易不走本协议，见下方 ADR-022 注与 P-TX-008/009。
- 签名/验签规则：
  - 签名前 payload 与最终 extrinsic body 必须使用同一份 immortal era 字节
  - 使用 polkadart 时必须传 `eraPeriod: 0`
  - `SigningPayload.blockHash` 必须传 `genesisHash`，不得传最新块 hash
  - 抗量子升级(ADR-022):PQC 交易改走 General Transaction(无外层 sr25519 签名),由自定义 `GmbPqcAuth` TransactionExtension 携带 ML-DSA-65 签名(proof 在扩展 extra),验签后把 origin 转 `Signed(account)`;未绑定账户首次走 bootstrap(post_dispatch 写 `AccountPqcKey`)无感绑定;AccountId 仍为原 sr25519 锚点。详见 P-TX-008/009。
- 禁止兼容：开发期不兼容热钱包 mortal era
- 禁止事项：
  - 禁止业务 service 自己保留 `_eraPeriod = 64`
  - 禁止 signed extrinsic 构造路径调用 `fetchLatestBlock()` 参与 era 计算
  - 禁止把最新块 hash 写入 immortal era 的 signing payload
- 必跑测试：
  - `citizenapp/test/rpc/signed_extrinsic_builder_test.dart`
  - `flutter test test/organization-manage test/proposal test/trade`

### P-TX-003：InternalVote.cast

- 状态：当前
- 类型：交易载荷格式
- 唯一真源：`citizenchain/runtime/votingengine/internal-vote/src/lib.rs`
- 详细文档：
  - `memory/01-architecture/qr/qr-action-registry.md`
  - `citizenwallet/lib/signer/pallet_registry.dart`
- 生产者：`citizenapp`、`citizenchain/node`
- 消费者：
  - `citizenchain/runtime/votingengine/internal-vote`
  - `citizenwallet/lib/signer/payload_decoder.dart`
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
- 禁止兼容：开发期不兼容旧 `VotingEngine(9)` 投票入口
- 禁止事项：
  - 禁止恢复业务 pallet 内的投票 wrapper
  - 禁止把内部投票编码回 `VotingEngine(9)`
- 必跑测试：
  - `citizenwallet/test/signer/payload_decoder_test.dart`
  - `citizenwallet/test/signer/pallet_registry_test.dart`

### P-TX-004：JointVote.cast_admin

- 状态：当前
- 类型：交易载荷格式
- 唯一真源：`citizenchain/runtime/votingengine/joint-vote/src/lib.rs`
- 详细文档：
  - `memory/01-architecture/qr/qr-action-registry.md`
  - `citizenwallet/lib/signer/pallet_registry.dart`
- 生产者：`citizenapp`、`citizenchain/node`
- 消费者：
  - `citizenchain/runtime/votingengine/joint-vote`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 字段：
  1. `proposal_id`
  2. `account_id`
  3. `approve`
- 编码：
  - SCALE call data
  - pallet index：`23`
  - call index：`0`
  - 前两个字节固定为 `[0x17, 0x00]`
- 签名/验签规则：
  - 联合投票的机构管理员阶段走 `JointVote::cast_admin`
  - `account_id` 底层类型为 `AccountId`
- 禁止兼容：开发期不兼容旧 `VotingEngine(9)` 投票入口
- 禁止事项：
  - 禁止恢复旧联合投票 wrapper
  - 禁止把 `account_id` 注释成当前 `InstitutionPalletId`
- 必跑测试：
  - `citizenwallet/test/signer/payload_decoder_test.dart`
  - `citizenwallet/test/signer/pallet_registry_test.dart`

### P-CRED-002：PopulationSnapshot

- 状态：当前
- 类型：凭证载荷
- 唯一真源：`citizencode/backend/citizens/chain_joint_vote.rs`
- 详细文档：
  - `memory/05-modules/citizenchain/runtime/votingengine/VOTINGENGINE_TECHNICAL.md`
  - `memory/05-modules/citizenchain/runtime/otherpallet/cid-system/CID_SYSTEM_TECHNICAL.md`
- 生产者：`cid`
- 消费者：
  - `citizenchain/runtime/votingengine`
  - `citizenapp`
  - `citizenchain/runtime/src/configs/mod.rs::RuntimePopulationSnapshotVerifier`
- 字段：
  1. `eligible_total`
  2. `snapshot_nonce`
  3. `issuer_cid_number`
  4. `issuer_main_account`
  5. `signer_pubkey`
  6. `scope_province_name`
  7. `scope_city_name`
  8. `signature`
- 编码：
  - HTTP JSON 响应
  - 链端验签 payload 以 runtime verifier 当前实现为准
- 签名/验签规则：
  - CID 后端用签发机构管理员密钥签发人口快照。
  - runtime 用 `issuer_main_account` 查询 `admins-change::AdminAccounts`,确认 `signer_pubkey` 属于该机构 `admins` 后验签。
- 禁止兼容：开发期不兼容缺少签发机构、签发管理员和作用域字段的旧人口快照
- 禁止事项：
  - 禁止前端自行伪造 `eligible_total / snapshot_nonce / signature`
  - 禁止业务模块自行获取或透传人口快照；人口快照只属于投票引擎及其投票流程
  - 禁止跳过 runtime 人口快照验签
- 必跑测试：
  - `citizenchain/runtime/src/tests/cases.rs` 中 population snapshot 相关测试
  - `citizenapp/test/proposal/runtime_upgrade/runtime_upgrade_service_test.dart`

### P-TX-003：ResolutionIssuance.propose_resolution_issuance

- 状态：当前
- 类型：交易载荷格式
- 唯一真源：`citizenchain/runtime/issuance/resolution-issuance/src/lib.rs`
- 详细文档：
  - `memory/05-modules/citizenchain/runtime/issuance/resolution-issuance/RESOLUTIONISSUANCE_TECHNICAL.md`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 生产者：`citizenchain/node`、`citizenapp`
  - `citizenchain/node/src/transaction/duoqian_transfer/`
  - `citizenchain/node/frontend/transaction/duoqian-transfer/`
  - `citizenapp/lib/transaction/duoqian-transfer/`
- 消费者：
  - `citizenchain/runtime/issuance/resolution-issuance`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 字段：
  1. `reason`
  2. `total_amount`
  3. `allocations`
- 编码：
  - SCALE call data
  - pallet index：`8`
  - call index：`0`
  - 前两个字节固定为 `[0x08, 0x00]`
- 签名/验签规则：
  - 本交易载荷只包含发行内容,不内嵌人口快照凭证。
  - 联合提案人口快照由 `JointVote.prepare_joint_population_snapshot` 单独准备并由 `RuntimePopulationSnapshotVerifier` 验签。
- 禁止兼容：开发期不兼容继续把人口快照字段塞回本载荷的旧格式
- 禁止事项：
  - 禁止节点或前端把人口快照凭证字段混入本交易载荷
  - 禁止把发行金额显示口径和链端 `u128` 分单位混用
- 必跑测试：
  - `citizenwallet/test/signer/payload_decoder_test.dart`
  - `citizenchain/runtime/src/tests/cases.rs`

### P-TX-005：DuoqianTransfer proposal payloads

- 状态：当前
- 类型：交易载荷格式
- 唯一真源：`citizenchain/runtime/transaction/duoqian-transfer/src/lib.rs`
- 详细文档：
  - `memory/05-modules/citizenchain/runtime/transaction/duoqian-transfer/DUOQIAN_TRANSFER_TECHNICAL.md`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 生产者：`citizenchain/node`、`citizenapp`
- 消费者：
  - `citizenchain/runtime/transaction/duoqian-transfer`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 字段：
  - `propose_transfer(19.0)`：`org`、`account_id`、`beneficiary`、`amount`、`remark`
  - `propose_safety_fund_transfer(19.1)`：`beneficiary`、`amount`、`remark`
  - `propose_sweep_to_main(19.2)`：`account_id`、`amount`
- 编码：
  - SCALE call data
  - pallet index：`19`
  - call index：`0 / 1 / 2`
- 签名/验签规则：
  - 业务提案创建由对应管理员签名
  - 投票不走本 pallet，统一走 `P-TX-003`
  - `CITIZEN_QR_V1 / sign_request` 展示字段必须使用 `institution / beneficiary / amount_yuan / remark`，禁止 node 使用旧 `org` 展示字段
- 禁止兼容：`call_index=3 / 4 / 5` 留洞不复用
- 禁止事项：
  - 禁止恢复 `execute_transfer / execute_safety_fund / execute_sweep` wrapper
  - 禁止把 `account_id` 注释成当前 `InstitutionPalletId`
  - 禁止在 `citizenapp/lib/governance/organization-manage/`、`citizenchain/node/src/governance/`、`citizenchain/node/frontend/governance/` 或 `citizenchain/node/src/transaction/offchain_transaction/` 中实现多签转账业务
- 必跑测试：
  - `citizenwallet/test/signer/payload_decoder_test.dart`
  - `cargo test --manifest-path citizenchain/runtime/transaction/duoqian-transfer/Cargo.toml`

### P-TX-006：PersonalManage proposal payloads

- 状态：当前
- 类型：交易载荷格式
- 唯一真源：`citizenchain/runtime/governance/personal-manage/src/lib.rs`
- 详细文档：
  - `memory/05-modules/citizenchain/runtime/governance/personal-manage/PERSONAL_MANAGE_TECHNICAL.md`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 生产者：
  - `citizenapp/lib/governance/personal-manage/personal_manage_service.dart`
- 消费者：
  - `citizenchain/runtime/governance/personal-manage`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 字段：
  - `propose_create(7.0)`：`account_name`、`admins`、`regular_threshold`、`amount`
  - `propose_close(7.1)`：`account`、`beneficiary`
- 编码：
  - SCALE call data
  - pallet index：`7`
  - call index：`0 / 1`
- ProposalData：
  - `MODULE_TAG = b"per-mgmt"`
  - `ACTION_CREATE = 0`：`account`、`proposer`、`amount`、`fee`
  - `ACTION_CLOSE = 1`：`account`、`beneficiary`、`proposer`
- 签名/验签规则：
  - 个人多签独立使用 `PersonalManage(7)` 与 `MODULE_TAG = b"per-mgmt"`
  - 投票统一走 `P-TX-003`
- 禁止兼容：不兼容旧 `OrganizationManage(17).propose_create_personal`，不兼容缺少 `regular_threshold` 的旧 `PersonalManage(7).propose_create`
- 禁止事项：
  - 禁止恢复 `OrganizationManage(17).call_index=3`
  - 禁止混用机构多签和个人多签 action 编号
  - 禁止 citizenapp / citizenwallet 保留旧个人多签创建交易载荷解析分支
- 必跑测试：
  - `cargo test --manifest-path citizenchain/Cargo.toml -p personal-manage --lib`
  - `citizenwallet/test/signer/payload_decoder_test.dart`
  - `flutter test test/personal-manage/personal_manage_service_test.dart test/personal-manage/personal_manage_storage_codec_test.dart`

### P-TX-007：AdminsChange.propose_admin_set_change

- 状态：当前
- 类型：交易载荷格式
- 唯一真源：`citizenchain/runtime/governance/admins-change/src/lib.rs`
- 详细文档：
  - `memory/05-modules/citizenchain/runtime/governance/admins-change/ADMINSCHANGE_TECHNICAL.md`
  - `memory/05-modules/citizenapp/admins-change/ADMINS_CHANGE_CITIZENAPP_TECHNICAL.md`
- 生产者：
  - `citizenapp/lib/governance/admins-change/codec/admin_set_change_call_codec.dart`
- 消费者：
  - `citizenchain/runtime/governance/admins-change`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 字段：
  1. `institution_code`
  2. `account_id`
  3. `admins`
  4. `new_threshold`
- 编码：
  - SCALE call data
  - pallet index：`12`
  - call index：`0`
  - 前两个字节固定为 `[0x0c, 0x00]`
  - 布局：`institution_code:[u8;4] + account_id:48 + Compact<Vec<AccountId32>> + new_threshold:u32_le`
- 签名/验签规则：
  - `new_threshold` 是管理员更换通过后写入投票引擎的目标动态阈值。
  - 内置治理机构只允许固定制度阈值，App 不展示阈值输入框。
  - 个人多签和机构账户阈值必须满足 `threshold * 2 > admins_len && threshold <= admins_len`。
- 禁止兼容：不兼容缺少 `new_threshold` 的旧载荷。
- 禁止事项：
  - 禁止 citizenapp 继续生成旧 `[org:u8, account_id, admins]` 载荷。
  - 禁止 citizenwallet 公民钱包解码旧载荷或忽略尾部多余字节。
  - 禁止在 citizenapp / citizenwallet 内实现投票、计票或通过判定。
- 必跑测试：
  - `citizenapp/test/governance/admins-change/admins_change_codec_test.dart`
  - `citizenwallet/test/signer/payload_decoder_test.dart`

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
  - `citizenapp/lib/governance/shared/admin_institution_codec.dart`
- 字段：
  - key：`AccountId`
  - value：`org`、`kind`、`admins`、`creator`、`created_at`、`updated_at`、`status`
- 编码：
  - storage key：`twox128("AdminsChange") ++ twox128("Subjects") ++ hasher(AccountId)`
  - value：SCALE
- 签名/验签规则：storage 本身不签名；写入必须来自链上授权流程
- 禁止兼容：不兼容旧 `AdminsChange::Institutions` 当前路径
- 禁止事项：
  - 禁止恢复 `Institutions` storage 当前真源叙述
  - 禁止把 key 继续命名为当前 `InstitutionPalletId`
- 必跑测试：
  - admins-change 单测
  - citizenapp 多签发现相关测试

### P-STORAGE-002：OrganizationManage.InstitutionAccounts

- 状态：当前
- 类型：storage 契约
- 唯一真源：`citizenchain/runtime/governance/organization-manage/src/lib.rs`
- 详细文档：
  - `memory/05-modules/citizenchain/node/offchain/NODE_CLEARING_BANK_TECHNICAL.md`
- 生产者：`organization-manage`
- 消费者：
  - `citizenchain/node/src/governance/organization_manage/chain.rs`
  - `citizenapp/lib/governance/organization-manage/duoqian_storage_codec.dart`
- 字段：
  - key1：`cid_number`
  - key2：`account_name`
  - value：机构账户信息，以 runtime 类型为准
- 编码：
  - double map storage key
  - value：SCALE
- 签名/验签规则：storage 本身不签名；写入必须来自机构创建和账户治理流程
- 禁止兼容：不兼容旧 `Accounts` mirror
- 禁止事项：
  - 禁止活跃代码继续读取 `OrganizationManage::Accounts`
  - 禁止把机构账户当个人多签账户读取
- 必跑测试：
  - `citizenapp/test/governance/organization-manage/duoqian_discovery_service_test.dart`
  - organization-manage 单测

### P-STORAGE-003：PersonalManage.PersonalAccounts

- 状态：当前
- 类型：storage 契约
- 唯一真源：`citizenchain/runtime/governance/personal-manage/src/lib.rs`
- 详细文档：
  - `memory/05-modules/citizenchain/runtime/governance/personal-manage/PERSONAL_MANAGE_TECHNICAL.md`
- 生产者：`personal-manage`
- 消费者：
  - `citizenapp/lib/governance/personal-manage/personal_manage_storage_codec.dart`
  - `citizenapp/lib/governance/personal-manage/personal_manage_service.dart`
- 字段：
  - key：`personal_account`
  - value：`Account { creator, account_name, created_at, status }`
- 编码：
  - storage map key
  - value：SCALE
- 签名/验签规则：storage 本身不签名；创建和关闭由 `PersonalManage` 提案流程约束
- 禁止兼容：不兼容旧 `OrganizationManage` 个人多签路径
- 禁止事项：
  - 禁止恢复 `OrganizationManage(17).propose_create_personal`
  - 禁止恢复已删除的个人多签反向索引 storage
  - 禁止把个人多签查询落回机构账户 storage
- 必跑测试：
  - `cargo test --manifest-path citizenchain/Cargo.toml -p personal-manage --lib`
  - `flutter test test/personal-manage/personal_manage_service_test.dart test/personal-manage/personal_manage_storage_codec_test.dart`

### P-STORAGE-004：Account-level internal admin account

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
  - `citizenapp`
  - `citizenwallet`
- 字段：
  - `account_id`
  - `account_id`
  - `admins`
  - `admins_len`
  - `threshold`
  - `status`
- 生命周期事件：
  - `AdminAccountPendingCreated { subject, org, kind, creator, admins_len, threshold }`
  - `AdminAccountActivated { subject, org }`
  - `AdminAccountPendingRemoved { subject, org }`
  - `AdminAccountClosed { subject, org }`
- 编码：
  - 治理机构账户继续映射到既有 `AdminAccountKind::Builtin`
  - 注册个人账户继续映射到既有 `AdminAccountKind::PersonalAccount`
  - 注册机构账户使用账户级 `AdminAccountKind::InstitutionAccount = 0x05`，payload 为账户 `AccountId` 前 32 字节并右填零
  - `注册机构归属关系 = 0x02` 保留为 CID 机构归属/检索 ID，不作为新增账户级管理员主体
- 签名/验签规则：
  - 一人一票一笔交易，投票资格由创建提案时锁定的账户级管理员快照决定
  - 注册创建和注销关闭阈值为全员
  - 普通动态账户提案阈值由管理员数量派生
  - Pending 主体清理必须命中既有 Pending 主体，不存在时返回 `InvalidInstitution`
- 禁止兼容：开发期彻底切换，不保留机构级管理员旧分支
- 禁止事项：
  - 禁止省储行永久质押账户进入内部投票
  - 禁止注册机构账户继续复用机构级管理员池
  - 禁止动态账户由用户自由输入阈值
  - 禁止把管理员增加、删除、更换、改阈值拆成四套提案
- 必跑测试：
  - `cargo test -p admins-change --lib`
  - `cargo test -p primitives --lib`
  - `cargo test -p internal-vote --lib`
  - `cargo test -p personal-manage --lib`
  - `cargo test -p organization-manage --lib`

### P-CPMS-001：CID_CPMS_V1 CPMS install/archive contract

- 状态：当前
- 类型：接口契约 / 凭证载荷
- 唯一真源：
  - `memory/05-modules/citizencode/CID-CPMS-QR-v1.md`
  - `citizencode/backend/citizenpassport/model.rs`
  - `citizencode/backend/citizenpassport/handler.rs`
- 详细文档：
  - `memory/01-architecture/citizenpassport/CPMS_TECHNICAL.md`
  - `memory/05-modules/citizencode/backend/citizenpassport/CITIZENPASSPORT_TECHNICAL.md`
- 生产者：
  - `INSTALL`：`cid`
  - `ARCHIVE`：`cpms`
- 消费者：
  - `INSTALL`：`cpms`
  - `ARCHIVE`：`cid`
- 字段：
  - `INSTALL`：`proto`、`type`、`cid_number`、`province_name`、`city_name`、`install_secret`、`sig`
  - `ARCHIVE`：`proto`、`type`、`archive_no`、`citizen_status`、`voting_eligible`、`valid_from`、`valid_until`、`status_updated_at`、`cpms_pubkey`、`geo_seal`、`wallet_address`、`wallet_pubkey`、`wallet_sig_alg`、`sig`
- 编码：
  - JSON
  - `proto` 固定为 `CID_CPMS_V1`
  - `type` 固定为 `INSTALL / ARCHIVE`
  - 机构 CID 字段固定为 `cid_number`
- 签名/验签规则：
  - `INSTALL` 由 CID 主密钥签名，原文为 `cid-cpms-v1|install|{cid_number}|{province_name}|{city_name}|{install_secret_hash}`
  - `INSTALL` 不做额外加密；CPMS 初始化只做本地防误装校验，ARCHIVE 是否可信由 CID 绑定阶段验真闭环确认
  - `geo_seal` 明文仅包含 `cid_number`，AES-GCM AAD 为 `cid-cpms-v1|geo-seal|{archive_no}|{cpms_pubkey}`
  - `ARCHIVE` 由 CPMS 本机私钥签名，原文为 `cid-cpms-v1|archive|{archive_no}|{citizen_status}|{voting_eligible}|{valid_from}|{valid_until}|{status_updated_at}|{cpms_pubkey}|{geo_seal_hash}|{wallet_address}|{wallet_pubkey}`
  - CID 验收 `ARCHIVE` 时先解 `geo_seal`，再验 CPMS 本机签名和 `archive_no` 全局唯一性
- 禁止兼容：开发期不兼容其他 `proto`
- 禁止事项：
  - 禁止把 CPMS 安装/档案业务码混入 `CITIZEN_QR_V1`
  - 禁止 CID 管理员登录二维码与 CPMS 业务二维码签名密钥混用
  - 禁止新建任何派生协议名
  - 禁止对外协议字段偏离 `cid_number`
- 必跑测试：
  - `cargo check --manifest-path citizencode/backend/Cargo.toml`
  - `npm run build`（路径：`citizencode/frontend`）

### P-TX-008：GmbPqcAuth bootstrap（未绑定账户首次无感绑定+执行）

- 状态：草案（ADR-022，待实现）
- 类型：交易载荷格式（General Transaction + `GmbPqcAuth` 扩展 `extra`）
- 唯一真源：`GmbPqcAuth` TransactionExtension + `account-keys` pallet（待实现）
- 详细文档：`memory/04-decisions/ADR-022-unified-pqc-crypto.md`
- 生产者：`citizenapp`、`citizenwallet`　消费者：`GmbPqcAuth` 扩展 + `account-keys`、`citizenwallet` decoder
- 字段（扩展 extra）：`account`、`pqc_pubkey`(ML-DSA-65,~1952B)、`alg`(0x02)、`key_version`、`nonce`、`sr25519_bootstrap_signature`、`ml_dsa_signature`（业务 call 是普通 General Transaction call）
- 编码：General Transaction + `GmbPqcAuth` 扩展 `extra`（**非 pallet call**）
- payload `GMB_PQC_BOOTSTRAP_V1`（域标签 `DOMAIN_BOOTSTRAP=b"GMB_PQC_BOOTSTRAP_MLDSA65_V1"` 进 preimage，字段集与 GMB_PQC_TX_V1 对齐）：`genesis_hash`、`spec_version`、`transaction_version`、`account`、`pqc_pubkey_hash`、`key_version`、`nonce`、`era_or_deadline`、`tip`、`call_hash`、`following_extensions_hash`
- 规则（验序钉死，hash 全 `blake2_256`）：
  - ① `blake2_256(body.pqc_pubkey) == payload.pqc_pubkey_hash`
  - ② `sr25519_bootstrap_signature = sr25519_sign(blake2_256(DOMAIN_BOOTSTRAP ++ SCALE(genesis_hash,spec_version,transaction_version,account,pqc_pubkey_hash,key_version,nonce,call_hash,following_extensions_hash)))`——**sr25519 必须覆盖 pqc_pubkey_hash**（防 body 公钥替换），`account`=sr25519 公钥派生的当前 AccountId
  - ③ `ml_dsa_signature` 验交易 payload + `call_hash==blake2_256(body.call)`，且**反向覆盖 `blake2_256(sr25519_bootstrap_signature)`**（双向交叉绑定）
  - 三验过 → origin 转 `Signed(account)` → nonce/扣费/业务 dispatch；**绑定写 `AccountPqcKey` 在 `post_dispatch`**
  - 失败语义：绑定在 post_dispatch（nonce/扣费已跑），**内层 call 失败绑定仍保留、内层失败照常收费**；🔴 **post_dispatch 绝不返回 Err**（否则作废整区块），冲突（已绑定不同值）判定前移 validate 拒
  - 🔴 bootstrap 账户须 providers/sufficients>0（否则 CheckNonce 以 Payment 先拒）；body 长度上限硬校验 + 未绑定按 (account,source) 限速
  - 已绑定账户拒绝再次 sr25519 覆盖（first-bind-wins）
  - extrinsic body 携带完整 ML-DSA 公钥（~1952B）+ sr25519 bootstrap 签名（64B）+ ML-DSA 交易签名（~3309B）
- 禁止：扩 `MultiSignature`；用 PQC 公钥/hash 派生新 AccountId；CID 托管助记词/私钥
- 必跑测试：bootstrap 双签成功/拒绝、已绑定拒覆盖、写表+派发原子性

### P-TX-009：GmbPqcAuth PQC 交易（已绑定账户）

- 状态：草案（ADR-022，待实现）
- 类型：交易载荷格式（General Transaction + `GmbPqcAuth` 扩展 `extra`）
- 唯一真源：`GmbPqcAuth` TransactionExtension（待实现）
- 详细文档：`memory/04-decisions/ADR-022-unified-pqc-crypto.md`
- 生产者：`citizenapp`、`citizenwallet`　消费者：`GmbPqcAuth` 扩展、`citizenwallet` decoder
- 字段（扩展 extra）：`account`、`sig`(ML-DSA-65；公钥由链端按 account 从 `AccountPqcKey` 读，交易不带公钥)、`auth_mode`、`key_version`（业务 call 是普通 General Transaction call）
- 编码：General Transaction + `GmbPqcAuth` 扩展 `extra`（**非 pallet call**）
- payload `GMB_PQC_TX_V1`（域标签 `DOMAIN_TX=b"GMB_PQC_TX_MLDSA65_V1"`（含算法标识）进 preimage）：`genesis_hash`、`spec_version`、`transaction_version`、`account`、`nonce`、`era_or_deadline`、`tip`、`call_hash`、`key_version`、`following_extensions_hash`（`ss58_format` 为纯展示字段，链上无对应 implicit，不参与一致性比对）
- 规则（路线 A 定稿）：
  - `GmbPqcAuth` 读 `AccountPqcKey[account].pubkey` 验 ML-DSA 签名 + `call_hash==blake2_256(body.call)` + `alg==AccountPqcKey.alg`（防降级） → **把 origin 转 `Signed(account)`** → 后续 `CheckNonce`/`ChargeTransactionPayment` 走系统标准逻辑
  - 🔴 `following_extensions_hash` = SDK `inherited_implication` **精确递归编码**（`ImplicationParts{base,explicit,implicit}`，非扁平拼接；嵌套 tuple 下与链端 `inherited_implication.encode()` 逐字节对拍 `mod.rs:712-869`），覆盖 CheckGenesis/CheckMortality(immortal→genesis)/CheckNonce/ChargeTransactionPayment/CheckMetadataHash(Disabled→None)/WeightReclaim 等 implicit
  - 🔴 **tuple 12 上限**：嵌套 `(GmbPqcAuth, AuthorizeCall)` 占第一项槽位，不加第 13 项；`GmbPqcAuth` 兼管"已绑定拒 sr25519"；`extra=None` 透明放行原 origin 给 AuthorizeCall
  - txpool `provides=(account,nonce)` 由 CheckNonce 自动产生（GmbPqcAuth 不重复设）；**era 默认 immortal**（CheckMortality.implicit 仍 genesis，纳入 hash）
  - `weight()` 按 extra 变体路由 card1 benchmark 常量（禁读 state）；PqcPolicy 缺失 fail-open（不冻结全链）；`validate` 轻量无副作用
- 禁止：跳过 `nonce`/`genesis_hash` 域隔离；decoder 解码后仍有剩余字节
- 必跑测试：authorize 成功/拒绝、nonce 防重放、`citizenwallet` decoder

### P-STORAGE-005：account-keys.AccountPqcKey

- 状态：草案（ADR-022，待实现）
- 类型：storage 契约　唯一真源：account-keys pallet（待实现）
- 详细文档：`memory/04-decisions/ADR-022-unified-pqc-crypto.md`
- 生产者：`GmbPqcAuth`（bootstrap `post_dispatch` 写）+ `account-keys`（轮换 call 写）　消费者：`GmbPqcAuth`（PQC 交易验签读）、`offchain-transaction`（批签取公钥）
- **pallet_index=27**（契约真源；当前 runtime 最高 idx=26，27 空闲）
- 字段：
  - key：`AccountId`（sr25519 锚点）
  - value：`alg:u8`(0x02)、`key_version:u32`、`pubkey:BoundedVec<u8,ConstU32<2048>>`(完整 ML-DSA-65 公钥 ~1952B)、`bound_at:BlockNumber`（**删 bootstrap_mode**）
  - 另有 `PqcPolicy` storage（phase/bootstrap_deadline/reject_sr25519_when_bound/allow_bootstrap_unbound，安全默认 phase=B/reject=false/allow=true/deadline=None）
- 编码：SCALE，`StorageMap<Blake2_128Concat, AccountId, AccountPqcKeyRecord>`
- 规则：存完整公钥（非 hash）；first-bind-wins（冲突在 validate 拒）；**轮换双签**：当前 PQC 私钥授权 + 新私钥对 `(新公钥+key_version+account+genesis)` 自签 PoP，两签过才 `key_version++`；**账户不派生 ML-KEM**（决策3）；绑定后无 sr25519 回退恢复（决策1/2）
- 禁止：存私钥；用 PQC 公钥（或其 hash）当 AccountId；给账户加签名算法 state 字段（阶段策略在链层 A/B/C/D 治理，不做 per-account 状态切换）
- 必跑测试：`account-keys` pallet 单测、`offchain-transaction` 批签集成测试

## 6. 登记维护要求

新增或修改协议时，必须在本文件按编号登记；无法确认字段时必须先向用户报告，不得把未确认字段写成当前协议。
