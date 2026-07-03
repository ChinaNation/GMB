# GMB 统一协议文件

## 1. 定位

本文件是 GMB AI 编程系统的统一协议入口。

以后任何设计、修改、删除下列内容之前，必须先查本文件：

- 扫码协议
- 二维码 `kind` / `body` / `payload` 结构
- 链上交易 call data 字段顺序
- SCALE 编码载荷格式
- CID / CitizenApp / citizenchain 之间的 API 契约
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
| 扫码协议 | 二维码外层 envelope 和 `k` 流向规则 | 是 |
| 签名请求 | `QR_V1` 下的 `k = 1` | 否，属于扫码协议中的一种流向 |
| 交易载荷格式 | `b.d` 中某个链上 call data 的字段顺序和编码 | 否 |
| 接口契约 | HTTP / Tauri command / app API 的路径、字段和错误规则 | 否 |
| 凭证载荷 | CID 等系统签发给链端验签的 payload 字段 | 否 |
| storage 契约 | pallet storage 名称、key 类型、读取方和写入方规则 | 否 |

死规则：

```text
扫码协议只有一个：QR_V1。
b.d 里可以有很多不同交易载荷格式，但它们都不是新的扫码协议。
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
- 唯一真源：`citizenchain/onchina/src/cid/validator.rs`
- 详细文档：`memory/05-modules/citizenchain/onchina/DATA_SECURITY_TECHNICAL.md`
- 生产者：`citizenchain/onchina/src/cid/generator.rs`
- 消费者：`citizenchain/onchina`、`citizenapp`、`citizenwallet`、`citizenchain`
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
  - 禁止在 OnChina 内部继续使用身份字段别名
  - 禁止恢复独立历史主体属性段
  - 禁止跳过 `C1` 校验
- 必跑测试：`cargo test --manifest-path citizenchain/onchina/Cargo.toml number::`

### P-API-ONCHINA-001：OnChina 管理员登录态工作台契约

- 状态：当前
- 类型：接口契约
- 唯一真源：
  - 后端：`citizenchain/onchina/src/auth/login/model.rs`
  - 工作台清单：`citizenchain/onchina/src/workspace/model.rs`
  - 前端：`citizenchain/onchina/frontend/auth/types.ts`、`citizenchain/onchina/frontend/workspace/types.ts`
- 详细文档：
  - `memory/01-architecture/onchina/ONCHINA_TECHNICAL.md`
  - `memory/05-modules/citizenchain/onchina/BACKEND_TECHNICAL.md`
  - `memory/05-modules/citizenchain/onchina/FRONTEND_TECHNICAL.md`
- 生产者：OnChina 登录、扫码登录轮询、`/api/v1/admin/auth/check`、`/api/v1/admin/auth/identify`、`/api/v1/admin/own-institution`
- 消费者：OnChina 前端 `AuthContext` 和 `workspace/WorkspaceRouter`
- 字段：
  - 登录态继续携带 `institution_code`、`cid_number`、`cid_full_name`、`cid_short_name`、`admin_name`、`capabilities`
  - `workspace`
  - `workspace.workspace_kind`: `registry` / `judicial` / `legislation` / `generic`
  - `workspace.workspace_title`
  - `workspace.workspace_sections[]`
  - `workspace_sections[].workspace_section`: `operations` / `display` / `records`
  - `workspace_sections[].workspace_section_title`
  - `workspace_sections[].workspace_actions[]`
  - `workspace_actions[].workspace_action`
  - `workspace_actions[].workspace_action_title`
  - `workspace_actions[].workspace_action_enabled`
  - `/api/v1/admin/own-institution` 返回 `InstitutionDetailOutput`: `institution`、`accounts`、`created_by_name`、`created_by_role`
- 编码：HTTP JSON,字段统一 snake_case;前端类型保持 snake_case,不另造 lowerCamelCase API 别名。
- 签名/验签规则：本契约只描述登录态返回;管理员身份仍由 QR_V1 登录签名、节点绑定和链上 active admins 校验决定。
- 禁止兼容：不得恢复“注册局根 UI + 非注册局只塞一个 tab”的旧口径;不得新增第二套 `dashboard` / `console` / `tenant` 同义字段。
- 禁止事项：
  - 禁止把 `workspace` 作为管理员授权真源。
  - 禁止前端根据本地硬编码越过后端 `capabilities` 显示受限操作。
  - 禁止非注册局机构复用注册局业务 UI。
- 必跑测试：
  - `cargo check --manifest-path citizenchain/onchina/Cargo.toml`
  - `npm --prefix citizenchain/onchina/frontend run build`
  - 真实本地 OnChina 服务的 `/api/v1/admin/auth/check` 和真实页面验收

### P-QR-001：QR_V1

- 状态：当前
- 类型：扫码协议
- 唯一真源：`memory/01-architecture/qr/qr-protocol-spec.md`
- 详细文档：
  - `memory/01-architecture/qr/qr-protocol-spec.md`
  - `memory/01-architecture/qr/qr-signing-recognition.md`
  - `memory/01-architecture/qr/qr-action-registry.md`
- 生产者：`citizenapp`、`citizenchain/node`、`citizenchain/onchina`
- 消费者：`citizenwallet`、`citizenapp`、`citizenchain/onchina`
- 字段：顶层只允许 `p/k/i/e/b`;具体字段以 `qr-protocol-spec.md` 为准
- 编码：紧凑 JSON envelope
- 签名/验签规则：按 `k` 和 `b.a + b.d` 执行;签名响应只带 `u/s`
- 禁止兼容：开发期不做旧协议兼容
- 禁止事项：
  - 禁止新增 `QR_V2`
  - 禁止新增第二套扫码协议字符串
  - 禁止把某个 `b.d` 的交易载荷格式称为新扫码协议
- 必跑测试：QR fixture、citizenwallet/citizenapp QR 解析测试

### P-QR-002：QR_V1 / k=1 sign_request

- 状态：当前
- 类型：扫码协议内签名请求流向
- 唯一真源：`memory/01-architecture/qr/qr-signing-recognition.md`
- 详细文档：
  - `memory/01-architecture/qr/qr-signing-recognition.md`
  - `memory/01-architecture/qr/qr-action-registry.md`
- 生产者：`citizenapp`、`citizenchain/node`、`citizenchain/onchina`
- 消费者：`citizenwallet`
- 字段：
  - `b.a`:业务动作码
  - `b.g`:签名算法码,当前 `1 = sr25519`
  - `b.u`:32B 签名者公钥,base64url 无填充
  - `b.d`:payload bytes,base64url 无填充
- 编码：外层 JSON；`b.d` 内部是具体链上 call data 或已登记的链下业务载荷
- 签名/验签规则：
  - `b.a` 必须已登记
  - `b.d` 必须能被扫码端 decoder 按对应交易载荷格式完整解码
  - `b.a` 必须和 decoder 得到的 action 一致
  - 用户确认页只展示 decoder 产出的 `reviewFields`;左侧分类名必须由统一映射翻译为中文，禁止直接渲染机器 key
  - 用户确认页的账户字段必须展示 SS58 地址，禁止把原始公钥 hex 当作普通用户确认字段展示
  - `activate_admin_account` 载荷中的 `institution_code` 必须用共享机构码编码，禁止各端手写第二套字节映射。
  - **onchina 控制台链写动作码(`b.d`=裸 SCALE call data,冷钱包解码核对后冷签 origin 由 CitizenWallet 提交)**:链交易统一 `a=(pallet<<8)|call`(禁止扁平小整数,会撞非链动作码 1..8)。机构创建=公权 `0x2005`(PublicManage 32/call 5)/私权 `0x2105`(PrivateManage 33/call 5,见 P-TX-001);公民链上身份注册=`0x0a00`(CitizenIdentity 10/call 0,见 P-TX-011);管理员集合=CREG `0x0c01`(`federal_set_city_registry_admins`)/FRG `0x0c00`(`propose_admin_set_change`,见 P-TX-007);非链文本治理 `a=3 = ACTION_ONCHINA_ADMIN / QR_ACTION_ONCHINA_ADMIN`(onchina_admin_governance JSON);IM 钱包绑定 `a=8 = QR_ACTION_IM_WALLET_BINDING`。动作码由 `onchina/src/core/institution_call.rs::chain_action_code(pallet,call)` 与 call data 同源派生,非链常量在 `core/qr/mod.rs`,runtime 注释真源在 `primitives::sign`,均与 `qr-action-registry.md` 同步。
  - Substrate 交易 payload 长度 >256B 时必须签 `blake2_256(payload)`
- 禁止兼容：开发期严格模式，不做别名兼容
- 禁止事项：
  - 禁止恢复 `display` / `summary` / `fields`
  - 禁止未登记的 `a` 进入生产
  - 禁止把内部哈希、nonce、原始公钥 hex 当作普通用户确认字段展示
- 必跑测试：`citizenwallet/test/signer/payload_decoder_test.dart`、QR sign request 测试

### P-QR-003：QR_V1 / k=5 im_node_pairing

- 状态：当前
- 类型：扫码协议内固定码
- 唯一真源：`citizenapp/lib/qr/bodies/im_node_pairing_body.dart`
- 详细文档：
  - `memory/05-modules/citizenapp/im/IM_TECHNICAL.md`
  - `memory/05-modules/citizenchain/node/NODE_TECHNICAL.md`
- 生产者：`citizenchain/node/src/settings/communication_node/mod.rs`
- 消费者：`citizenapp/lib/im/im_node_settings_page.dart`
- 字段：
  - `b.node_peer_id`
  - `b.node_multiaddr`
  - `b.endpoint_kind`
- 编码：外层 `QR_V1` 固定码；body 为 JSON 对象；顶层不包含 `i/e`。
- 签名/验签规则：本二维码只用于把公民手机配对到用户自己的电脑通信节点；钱包聊天账户授权另走 `QR_V1/k=1/a=8 im_wallet_binding`，不在本固定码内携带钱包私钥或交易载荷。
- 禁止兼容：不兼容旧联系人码、旧 IM 联系人 bundle 或旧 `communication` 模式字段。
- 禁止事项：
  - 禁止用本二维码添加联系人。
  - 禁止把本二维码作为交易、转账、治理或 CID 身份码处理。
  - 禁止把通信节点配对做成全节点模式选项；归档/普通全节点模式与通信节点功能必须分离。
  - 禁止在本二维码中携带 RPC URL、临时 nonce 或有效期。
- 必跑测试：`flutter test test/qr/im_node_pairing_body_test.dart test/im/im_node_settings_page_test.dart`

### P-CRED-003：CitizenIdentity VotingIdentityPayload

- 状态：当前
- 类型：凭证载荷 / 交易载荷内层结构
- 唯一真源：`citizenchain/runtime/otherpallet/citizen-identity/src/lib.rs`
- 详细文档：
  - `memory/01-architecture/qr/qr-action-registry.md`
  - `memory/05-modules/citizenchain/onchina/BACKEND_TECHNICAL.md`
- 生产者：`citizenchain/onchina/src/domains/citizens/chain_identity.rs`
- 消费者：
  - `citizenwallet/lib/signer/payload_decoder.dart`
  - `citizenwallet/lib/signer/qr_signer.dart`
  - `citizenchain/runtime/otherpallet/citizen-identity`
- 字段：
  1. `cid_number`
  2. `wallet_account`
  3. `citizen_age_years`
  4. `passport_valid_from`
  5. `passport_valid_until`
  6. `citizen_status`
  7. `residence_province_code`
  8. `residence_city_code`
  9. `residence_town_code`
- 编码：SCALE `VotingIdentityPayload<AccountId>`;字符串字段为 bounded `Vec<u8>`,账户字段为 `AccountId32`。
- 签名/验签规则：
  - `QR_V1` 非链动作 `a=2 citizen_identity` 的签名字节为 `blake2_256(GMB || 0x10 || payload_bytes)`。
  - runtime 通过 `primitives::sign::OP_SIGN_CITIZEN_IDENTITY` 验证目标公民钱包签名。
  - `citizen_age_years` 必须大于等于 16;OnChina 和 runtime 都必须校验。
- 禁止兼容：不兼容旧 `citizen-identity-v1|...` 文本载荷,不保留旧签原文规则。
- 禁止事项：
  - 禁止本地新增公民阶段要求钱包账户。
  - 禁止未满 16 周岁公民推送链上身份。
  - 禁止二维码携带展示摘要或字段别名。
- 必跑测试：
  - `citizenwallet/test/signer/payload_decoder_test.dart`
  - `citizenwallet/test/signer/qr_signer_test.dart`
  - `cargo test --manifest-path citizenchain/Cargo.toml -p citizen-identity`

### P-TX-011：CitizenIdentity.register_voting_identity

- 状态：当前
- 类型：交易载荷格式
- 唯一真源：`citizenchain/runtime/otherpallet/citizen-identity/src/lib.rs`
- 详细文档：
  - `memory/01-architecture/qr/qr-action-registry.md`
  - `memory/05-modules/citizenchain/onchina/BACKEND_TECHNICAL.md`
- 生产者：`citizenchain/onchina/src/domains/citizens/chain_identity.rs`
- 消费者：
  - `citizenchain/runtime/otherpallet/citizen-identity`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 字段：
  1. `registrar_account`
  2. `VotingIdentityPayload`
  3. `citizen_signature`
- 编码：
  - SCALE call data
  - pallet index：`10`
  - call index：`0`
  - 前两个字节固定为 `[0x0a, 0x00]`
  - 动作码：`a=0x0a00`
- 签名/验签规则：
  - 外层链交易由当前注册局管理员公民钱包签名并提交。
  - 内层 `citizen_signature` 必须来自目标公民钱包对 P-CRED-003 的签名。
  - runtime 校验注册局管理范围、CID 唯一性、公民签名和 16 周岁年龄门槛。
- 禁止兼容：不兼容旧无年龄字段的 `VotingIdentityPayload`,不保留旧字段顺序。
- 禁止事项：
  - 禁止绕过 `citizen-identity` 在业务模块内自建投票身份。
  - 禁止前端或 OnChina 伪造已上链状态。
- 必跑测试：
  - `citizenwallet/test/signer/payload_decoder_test.dart`
  - `cargo test --manifest-path citizenchain/Cargo.toml -p citizen-identity`

### P-TX-012：LegislationYuan 法律案提案载荷

- 状态：当前
- 类型：交易载荷格式
- 唯一真源：`citizenchain/runtime/public/legislation-yuan/src/lib.rs`
- 详细文档：
  - `memory/04-decisions/ADR-027-legislation-yuan.md`
- 生产者：
  - `citizenchain/onchina/src/domains/legislation/law/chain_propose.rs`
- 消费者：
  - `citizenchain/runtime/public/legislation-yuan`
  - `citizenwallet` 法律案扫码 decoder
- 字段：
  - `propose_enact_law`: `[pallet=27, call=0] + tier + scope_code + houses + proposer_body + executive + legislature + vote_type + title + title_en + chapters + effective_at`
  - `propose_amend_law`: `[pallet=27, call=1] + law_id + proposer_body + executive + legislature + vote_type + title + title_en + chapters + effective_at`
  - `propose_repeal_law`: `[pallet=27, call=2] + law_id + proposer_body + executive + legislature + vote_type`
- 编码：
  - 裸 SCALE call data
  - `tier`/`vote_type` 为单字节枚举序号
  - `scope_code` 为 `u32`
  - `law_id` 为 `u64`
  - `houses` / `proposer_body` / `executive` / `legislature` 使用 `(InstitutionCode[4], AccountId32)`
  - `chapters` 为 `章 > 节 > 条 > 款` 的 SCALE 结构
  - `effective_at` 为 `u64` 毫秒时间戳，不是块号
  - 动作码：`0x1b00` / `0x1b01` / `0x1b02`
- 签名/验签规则：
  - 外层链交易由当前立法/提案机构管理员冷钱包签名并提交。
  - 业务投票、计票、签署和守卫流程统一归投票引擎与 legislation-vote，不得由 OnChina 或客户端复刻。
- 禁止兼容：不兼容旧区块高度生效载荷，不保留旧字段顺序。
- 禁止事项：
  - 禁止前端显示或让用户填写旧区块高度生效字段。
  - 禁止未登记动作码进入冷钱包 decoder。
- 必跑测试：
  - `cargo test -p onchina --manifest-path citizenchain/Cargo.toml law`
  - `cargo test -p legislation-yuan --manifest-path citizenchain/Cargo.toml`

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
  - 绑定签名请求固定为 `QR_V1/k=1/a=8 im_wallet_binding`；`b.d` 是 `wallet_account, im_device_id, im_device_pubkey, node_peer_id, node_endpoints(Vec<String>), expires_at_millis, nonce` 的 SCALE bytes。
  - 签名字节固定为 `signing_message(OP_SIGN_IM_WALLET_BINDING=0x1A, b.d)`；node 登记设备前必须用 `wallet_account` 解出的 32 字节公钥验签。
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

### P-TX-001：PublicManage/PrivateManage.propose_create_{public,private}_institution

- 状态：当前(机构管理已拆分公权/私权两 pallet,取代旧 `OrganizationManage.propose_create_institution`)
- 类型：交易载荷格式
- 唯一真源：
  - `citizenchain/runtime/entity/public-manage/src/lib.rs`(`propose_create_public_institution` call 5)
  - `citizenchain/runtime/entity/private-manage/src/lib.rs`(`propose_create_private_institution` call 5)
  - 两 call 参数形态完全相同(下 15 字段),仅 pallet 前缀不同
- 详细文档：
  - `memory/01-architecture/qr/qr-action-registry.md`
- 生产者：
  - `citizenchain/onchina/src/core/institution_call.rs`(注册局录入,按机构码路由公权/私权 pallet 前缀)
  - `citizenapp/lib/transaction/...`(机构创建,具体路径随 runtime 拆分对齐)
- 消费者：
  - `citizenchain/runtime/entity/public-manage` / `citizenchain/runtime/entity/private-manage`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 字段：
  1. `cid_number`
  2. `cid_full_name`
  3. `cid_short_name`（A1 新增；公权法人机构落库,私权机构链上留空,名称落 onchina 本地）
  4. `accounts`
  5. `institution_code`
  6. `admins_len`
  7. `admins`（A2 起 = `Vec<AdminProfile>`，逐人 account+admin_cid_number+name+admin_role+term_start+term_end+source；投票快照只取 `.account`，编码同 P-TX-007 机构布局的 AdminProfile）
  8. `threshold`
  9. `register_nonce`
  10. `signature`
  11. `issuer_cid_number`
  12. `issuer_main_account`
  13. `signer_pubkey`
  14. `scope_province_name`
  15. `scope_city_name`
- 编码：
  - SCALE call data
  - pallet index：公权机构=`32`(PublicManage),私权机构=`33`(PrivateManage);由 `institution_code` 经 `primitives::cid::code::is_private_legal_code` 派生(onchina `create_institution_pallet_index` 单源)
  - call index：`5`(两 pallet 同)
  - 前两个字节:公权=`[0x20, 0x05]`(动作码 `0x2005`)、私权=`[0x21, 0x05]`(动作码 `0x2105`)
- 签名/验签规则：
  - `register_nonce / signature / issuer_cid_number / issuer_main_account / signer_pubkey / scope_*` 由 CID 机构注册信息凭证提供
  - runtime 通过 `issuer_main_account` 查询 `admins-change::AdminAccounts`,确认 `signer_pubkey` 属于该机构 `admins` 后验签
  - `accounts.account_name` 顺序必须与 CID `/registration-info.account_names` 一致
  - 名称分档：runtime 用 `primitives::cid::code::is_public_legal_code(institution_code)` 判定;公权必须带非空 `cid_full_name`+`cid_short_name` 并上链,私权链上落空(名称在 onchina 本地)
  - **凭证缺口(仍未补)**：CID 注册凭证签名当前覆盖 `cid_full_name`,尚未覆盖 `cid_short_name`;B2 onchina 编码器已落地但凭证 `P-CRED-001` payload 尚未纳入 `cid_short_name` 签名(公权简称可由 official_name_pair 派生,runtime 暂不强制);如要防简称被篡改须改凭证 payload
- 禁止兼容：开发期不兼容旧 `call_index=0`、不兼容旧 `OrganizationManage(17).propose_create_institution`
- 禁止事项：
  - 禁止把本交易载荷称为新增扫码协议
  - 禁止继续使用已删除的 `OrganizationManage(17)` / `[0x11,0x05]` 编码机构创建
  - 禁止在本载荷末尾追加 `subject_property / private_type / partnership_kind / parent_cid_number`
  - 禁止用裸非法人机构码（`SFGT/SFGP/UNIN`）直接创建机构账户；非法人必须由 CID 上层明确归属后走对应管理员模块
  - 禁止 CitizenWallet decoder 解码后仍有剩余字节
- 必跑测试：
  - `cargo test -p onchina`(institution_call 跨真类型对拍 + 公权/私权前缀分支)
  - `citizenwallet/test/signer/payload_decoder_test.dart`
  - `cargo check -p public-manage -p private-manage`

### P-TX-010：AddressRegistry address payloads

- 状态：当前
- 类型：交易载荷格式
- 唯一真源：
  - `citizenchain/runtime/otherpallet/address-registry/src/lib.rs`
  - `citizenchain/onchina/src/domains/address/chain_call.rs`
- 详细文档：
  - `memory/05-modules/citizenchain/runtime/ADDRESS_REGISTRY_TECHNICAL.md`
  - `memory/05-modules/citizenchain/onchina/ADDRESS_TECHNICAL.md`
  - `memory/01-architecture/qr/qr-action-registry.md`
- 生产者：`citizenchain/onchina/src/domains/address/chain_call.rs`
- 消费者：`citizenchain/runtime/otherpallet/address-registry`
- 字段：
  - `set_catalog_version(35.0)`：`registrar_account`, `catalog_version`, `catalog_hash`
  - `set_address_name(35.1)`：`registrar_account`, `province_code`, `city_code`, `town_code`, `address_name_code`, `address_name`
  - `remove_address_name(35.2)`：`registrar_account`, `province_code`, `city_code`, `town_code`, `address_name_code`
  - `set_address(35.3)`：`registrar_account`, `province_code`, `city_code`, `town_code`, `address_name_code`, `address_local_no`, `address_detail`
  - `remove_address(35.4)`：`registrar_account`, `province_code`, `city_code`, `town_code`, `address_name_code`, `address_local_no`, `address_detail`
- 编码：
  - SCALE 裸 call data
  - pallet index：`35`
  - call index：`0..4`
  - 前两个字节：`[0x23, call_index]`
  - 动作码：`a=(35<<8)|call_index`,即 `0x2300..0x2304`
- 签名/验签规则：
  - `origin` 必须是 `registrar_account` 对应注册局的有效管理员。
  - FRG 省级组只能更新本省地址。
  - CREG 只能更新本市地址。
  - `catalog_version` 与 `catalog_hash` 由 OnChina 当前 `china.sqlite` 派生或由调用方显式传入。
- 禁止兼容：不兼容旧地址全量上链、旧墓碑表、旧变更日志表和旧地址字段。
- 禁止事项：
  - 禁止把地址库全量上链。
  - 禁止在链上保存旧地址历史或墓碑。
  - 禁止绕过 `AddressUpdateAuthority` 直接在 pallet 内复制 FRG/CREG 权限。
- 必跑测试：
  - `cargo check --manifest-path citizenchain/Cargo.toml -p address-registry`
  - `cargo check --manifest-path citizenchain/Cargo.toml -p citizenchain`
  - `cargo check --manifest-path citizenchain/Cargo.toml -p onchina`

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
  - `CitizenApp` 联合公投签名请求流程
  - Step2D fixture
- 消费者：
  - `citizenchain/runtime/votingengine/joint-vote`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 字段：
  1. `proposal_id`
  2. `approve`
- 编码：
  - SCALE call data
  - pallet index：`23`
  - call index：`1`
  - 前两个字节固定为 `[0x17, 0x01]`
  - call data 长度为 11 字节，后接标准签名尾部
- 签名/验签规则：
  - runtime 按交易签名账户读取链上公民身份。
  - 公民投票资格和作用域由 `citizen-identity` 判定。
- 禁止兼容：开发期不兼容旧 `VotingEngine(9).call_index=2`
- 禁止事项：
  - 禁止 Step2D fixture 中继续出现 `cast_referendum` 的 `pallet_index=9 / call_index=2`
  - 禁止 `cast_referendum` fixture 继续使用 `0x0902` 前缀
  - 禁止 `CitizenWallet` 与 `CitizenApp` 各自维护重复 Step2D fixture
- 必跑测试：
  - `citizenwallet/test/signer/payload_decoder_test.dart`
  - `citizenwallet/test/signer/pallet_registry_test.dart`
  - `citizenapp/test/proposal/runtime_upgrade/runtime_upgrade_service_test.dart`

### P-CRED-001：OnChina subject registration-info credential

- 状态：当前
- 类型：凭证载荷 / 接口契约
- 唯一真源：`citizenchain/onchina/src/subjects/chain_multisig_info.rs`
- 详细文档：`memory/05-modules/citizenchain/node/offchain-transaction/NODE_CLEARING_BANK_TECHNICAL.md`
- 生产者：`citizenchain/onchina/src/subjects/chain_multisig_info.rs`
- 消费者：
  - `citizenchain/onchina/src/institution/subjects/registration_call.rs`(注册局录入构造 call data)
  - `citizenchain/runtime/entity/public-manage` / `citizenchain/runtime/entity/private-manage`(链端验签)
- 字段：
  - 外层业务字段：`cid_number`、`cid_full_name`、`account_names`
  - 凭证字段：`credential.register_nonce`、`credential.issuer_cid_number`、`credential.issuer_main_account`、`credential.signer_pubkey`、`credential.scope_province_name`、`credential.scope_city_name`、`credential.signature`
- 编码：
  - HTTP JSON 响应
  - runtime 验签 payload 按 OnChina 后端 `build_institution_registration_info_credential` 的 SCALE tuple 顺序
- 签名/验签规则：
  - OnChina 后端用签发机构管理员密钥签发。
  - 链端用 `issuer_main_account` 读取 `admins-change::AdminAccounts`，确认 `signer_pubkey` 属于该机构 `admins` 后验签。
  - `scope_province_name / scope_city_name` 只表示业务作用域，不表示签发人身份。
- 禁止兼容：不把 `subject_property / private_type / partnership_kind / parent_cid_number` 纳入链端注册凭证
- 禁止事项：
  - 禁止用普通机构详情接口替代 `/registration-info`
  - 禁止 CitizenApp 自己拼 `register_nonce / signature / issuer_cid_number / issuer_main_account / signer_pubkey / scope_*`
- 必跑测试：OnChina 后端 registration-info 测试、P-TX-001 双端编码/解码测试

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
  - `flutter test test/transaction/multisig-transfer test/proposal test/trade`

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

### P-CRED-002：PopulationScopeSnapshot

- 状态：当前
- 类型：链上人口作用域
- 唯一真源：`citizenchain/runtime/otherpallet/citizen-identity`
- 详细文档：
  - `memory/05-modules/citizenchain/runtime/votingengine/VOTINGENGINE_TECHNICAL.md`
  - `memory/05-modules/citizenchain/runtime/otherpallet/citizen-identity/CITIZEN_IDENTITY_TECHNICAL.md`
- 生产者：链上交易调用者
- 消费者：
  - `citizenchain/runtime/votingengine`
  - `citizenapp`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 字段：
  1. `PopulationScope::Country`
  2. `PopulationScope::Province(province_code)`
  3. `PopulationScope::City(province_code, city_code)`
  4. `PopulationScope::Town(province_code, city_code, town_code)`
- 编码：
  - SCALE call data
  - `JointVote.prepare_joint_population_snapshot(scope)` 使用 pallet `23` / call `2`
  - `LegislationVote.prepare_population_snapshot(scope)` 使用 pallet `28` / call `0`
- 签名/验签规则：
  - 交易只按标准链上账户签名。
  - runtime 从 `citizen-identity` 读取作用域人口分母。
- 禁止兼容：开发期不兼容任何链下签发人口证明格式
- 禁止事项：
  - 禁止前端或 OnChina 伪造人口分母。
  - 禁止业务模块自行获取或透传人口证明；人口快照只属于投票引擎及其投票流程。
  - 禁止跳过 runtime 链上人口读取。
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
  - `citizenchain/node/src/transaction/multisig_transfer/`
  - `citizenchain/node/frontend/transaction/multisig-transfer/`
  - `citizenapp/lib/transaction/multisig-transfer/`
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
  - 本交易载荷只包含发行内容,不内嵌人口快照字段。
  - 联合提案人口快照由 `JointVote.prepare_joint_population_snapshot(scope)` 单独准备并读取链上公民身份人口。
- 禁止兼容：开发期不兼容继续把人口快照字段塞回本载荷的旧格式
- 禁止事项：
  - 禁止节点或前端把人口快照字段或旧链下人口证明字段混入本交易载荷
  - 禁止把发行金额显示口径和链端 `u128` 分单位混用
- 必跑测试：
  - `citizenwallet/test/signer/payload_decoder_test.dart`
  - `citizenchain/runtime/src/tests/cases.rs`

### P-TX-005：MultisigTransfer proposal payloads

- 状态：当前
- 类型：交易载荷格式
- 唯一真源：`citizenchain/runtime/transaction/multisig-transfer/src/lib.rs`
- 详细文档：
  - `memory/05-modules/citizenchain/runtime/transaction/multisig-transfer/MULTISIG_TRANSFER_TECHNICAL.md`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 生产者：`citizenchain/node`、`citizenapp`
- 消费者：
  - `citizenchain/runtime/transaction/multisig-transfer`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- CI 同步：
  - `.github/workflows/citizenwallet-ci.yml` 必须从 `MultisigTransfer` / `multisig-transfer` 同步 `citizenwallet/lib/signer/pallet_registry.dart`
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
  - `QR_V1 / k=1` 必须使用 `a + payload` 解码展示 `institution / beneficiary / amount_yuan / remark`，禁止 node 在 QR 中塞展示字段
- 禁止兼容：`call_index=3 / 4 / 5` 留洞不复用
- 禁止事项：
  - 禁止恢复 `execute_transfer / execute_safety_fund / execute_sweep` wrapper
  - 禁止把 `account_id` 注释成当前 `InstitutionPalletId`
  - 多签转账业务唯一归口 `citizenapp/lib/transaction/multisig-transfer/`(公私个共用);禁止在 `citizenapp/lib/citizen/institution/`(机构管理只读)、`citizenchain/node/src/governance/` 或 `citizenchain/node/src/transaction/offchain_transaction/`(链下结算)中另实现多签转账业务
- 必跑测试：
  - `citizenwallet/test/signer/payload_decoder_test.dart`
  - `cargo test --manifest-path citizenchain/runtime/transaction/multisig-transfer/Cargo.toml`

### P-TX-006：PersonalManage proposal payloads

- 状态：当前
- 类型：交易载荷格式
- 唯一真源：`citizenchain/runtime/private/personal-manage/src/lib.rs`
- 详细文档：
  - `memory/05-modules/citizenchain/runtime/private/personal-manage/PERSONAL_MANAGE_TECHNICAL.md`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 生产者：
  - `citizenapp/lib/transaction/personal-manage/personal_manage_service.dart`
- 消费者：
  - `citizenchain/runtime/private/personal-manage`
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
  - 禁止 CitizenApp / CitizenWallet 保留旧个人多签创建交易载荷解析分支
- 必跑测试：
  - `cargo test --manifest-path citizenchain/Cargo.toml -p personal-manage --lib`
  - `citizenwallet/test/signer/payload_decoder_test.dart`
  - `flutter test test/governance/personal-manage/personal_manage_service_test.dart test/governance/personal-manage/personal_manage_storage_codec_test.dart`

### P-TX-007：AdminsChange.propose_admin_set_change

- 状态：当前
- 类型：交易载荷格式
- 唯一真源：`citizenchain/runtime/admins/{personal-admins,public-admins,private-admins}/src/lib.rs`
- 详细文档：
  - `memory/05-modules/citizenchain/runtime/admins/ADMINS_TECHNICAL.md`
  - `memory/05-modules/citizenapp/admins-change/ADMINS_CHANGE_CITIZENAPP_TECHNICAL.md`
- 生产者：
  - `citizenapp/lib/citizen/proposal/admins-change/codec/admin_set_change_call_codec.dart`
- 消费者：
  - `citizenchain/runtime/admins/{personal-admins,public-admins,private-admins}`
  - `citizenwallet/lib/signer/payload_decoder.dart`
- 字段：
  1. `institution_code`
  2. `account_id`
  3. `admins`（A2 起:**机构(public/private)= `Vec<AdminProfile>`**;个人多签 personal = `Vec<AccountId32>`,不带 profile）
  4. `new_threshold`
- 编码：
  - SCALE call data
  - pallet index：个人多签 `7`，公权机构与固定治理机构 `29`，私权机构 `30`
  - call index：个人多签 `3`，公权/私权 `0`，联邦注册局省级组 `2`
  - 前两个字节按 `AdminAccount.kind` 和主体类型选择，不再固定为 `[0x0c, 0x00]`
  - 个人多签布局：`institution_code:[u8;4] + account_id:[u8;32] + Compact<Vec<AccountId32>> + new_threshold:u32_le`
  - 机构布局：`institution_code:[u8;4] + account_id:[u8;32] + Compact<Vec<AdminProfile>> + new_threshold:u32_le`；`AdminProfile = account:[u8;32] + admin_cid_number:Compact<Vec<u8>>(≤32) + name:Compact<Vec<u8>>(≤128) + admin_role:Compact<Vec<u8>>(≤128) + term_start:u32_le + term_end:u32_le + source:u8`(0..=4=创世/注册局/内部投票/互选/普选)。account_id 为 `AccountId32`=32 字节裸(onchina `institution_call.rs::encode_admin_set_call` 跨真类型对拍锁定;旧文档误记 48)
  - **关联调用 `PublicAdmins.propose_federal_registry_province_admin_set_change`(pallet 29 / call 2,前缀 `[0x1d,0x02]`)**:联邦注册局省级 5 人组管理员集合更换,布局为 `province_code + Compact<Vec<AdminProfile>> + threshold`。
- 签名/验签规则：
  - `new_threshold` 是管理员更换通过后写入投票引擎的目标动态阈值。
  - 内置治理机构只允许固定制度阈值，App 不展示阈值输入框。
  - 个人多签和机构账户阈值必须满足 `threshold * 2 > admins_len && threshold <= admins_len`。
  - 非法人机构码不能决定 public/private；必须由 CID 注册归属或链上 `AdminAccount.kind` 显式路由到 `PublicAdmins` 或 `PrivateAdmins`。
- 禁止兼容：不兼容缺少 `new_threshold` 的旧载荷。
- 禁止事项：
  - 禁止 CitizenApp 继续生成旧 `[org:u8, account_id, admins]` 载荷。
  - 禁止 CitizenWallet 公民钱包解码旧载荷或忽略尾部多余字节。
  - 禁止在 CitizenApp / CitizenWallet 内实现投票、计票或通过判定。
- 必跑测试：
  - `citizenapp/test/governance/admins-change/admins_change_codec_test.dart`
  - `citizenwallet/test/signer/payload_decoder_test.dart`

### P-STORAGE-001：Admins.AdminAccounts

- 状态：当前
- 类型：storage 契约
- 唯一真源：`citizenchain/runtime/admins/{personal-admins,public-admins,private-admins}/src/lib.rs` + `citizenchain/runtime/admins/admin-primitives/src/lib.rs`
- 详细文档：
  - `memory/05-modules/citizenchain/runtime/admins/ADMINS_TECHNICAL.md`
  - `memory/05-modules/citizenapp/admins-change/ADMINS_CHANGE_CITIZENAPP_TECHNICAL.md`
- 生产者：各管理员 pallet 生命周期接口与机构/个人创建流程
- 消费者：
  - `citizenchain/runtime`
  - `citizenchain/node`
  - `citizenapp/lib/citizen/proposal/admins-change/services/admin_account_service.dart`
  - `citizenapp/lib/citizen/shared/admin_account_storage_codec.dart`
- 字段：
  - key：`account_id`（机构=main_account=derive(cid_number,主账户);A2 不改键,main_account 即机构身份的确定性像）
  - value：`institution_code`、`kind`、`admins`、`creator`、`created_at`、`updated_at`、`status`
  - `admins`（A2 起）：**机构 public/private = `BoundedVec<AdminProfile>`**(每人 account+admin_cid_number+name+admin_role+term_start+term_end+source);**personal = `BoundedVec<AccountId32>`**(不带 profile)。`AdminAccountQuery::active_account_admins` 仍出 `Vec<AccountId>`(抽 `.account`)→投票/多签/阈值零改;`active_account_admin_profiles` 出完整资料供展示。固定治理机构 profile 由创世写入,source=Genesis。
- 编码：
  - storage key：`twox128(pallet_name) ++ twox128("AdminAccounts") ++ blake2_128_concat(account_id)`
  - `pallet_name` 按 `AdminAccount.kind` 选择：`PublicAdmins / PrivateAdmins / PersonalAdmins`
  - value：SCALE
- 签名/验签规则：storage 本身不签名；写入必须来自链上授权流程
- 禁止兼容：不兼容旧 `AdminsChange::Subjects / AdminsChange::Institutions` 当前路径
- 禁止事项：
  - 禁止恢复 `AdminsChange` 单 pallet 当前真源叙述
  - 禁止只凭 `UNIN/SFGT/SFGP` 自动选择 `PrivateAdmins`
- 必跑测试：
  - admins-change 单测
  - CitizenApp 多签发现相关测试

### P-STORAGE-002：PublicManage/PrivateManage.InstitutionAccounts

- 状态：当前(机构生命周期已拆 PublicManage(idx32)/PrivateManage(idx33),storage 名不变但前缀随 pallet 名变;取代旧 `OrganizationManage`)
- 类型：storage 契约
- 唯一真源：
  - `citizenchain/runtime/entity/public-manage/src/lib.rs`(公权机构)
  - `citizenchain/runtime/entity/private-manage/src/lib.rs`(私权机构)
- 详细文档：
  - `memory/05-modules/citizenchain/node/offchain-transaction/NODE_CLEARING_BANK_TECHNICAL.md`
- 生产者：`public-manage` / `private-manage`
- 消费者：
  - `citizenchain/node/src/transaction/offchain_transaction/institution_read/chain.rs`(按机构码路由 PublicManage/PrivateManage 前缀)
  - `citizenapp` 机构读共享核心 storage codec(C 阶段三分后,按机构码路由前缀)
- 字段：
  - key1：`cid_number`
  - key2：`account_name`
  - value：机构账户信息，以 runtime 类型为准
- 同 pallet 的 `Institutions[cid_number] → InstitutionInfo`（A1 精简,2026-06-28）：
  - value 字段仅 `cid_full_name`(公权)、`cid_short_name`(公权)、`institution_code`、`created_at`、`status`（5 项）
  - 已删 `main_account`/`fee_account`/`admins`/`admins_len`/`threshold`/`creator`/`account_count`：主/费账户由派生且在 InstitutionAccounts;管理员真源 admins 模块;阈值真源 internal-vote
  - 消费方镜像须按机构码路由 PublicManage/PrivateManage 前缀(node 已切;citizenapp 待 C 阶段)
- 编码：
  - double map storage key(前缀 = `twox_128(PublicManage|PrivateManage)` ++ `twox_128(InstitutionAccounts|Institutions)`)
  - value：SCALE
- 签名/验签规则：storage 本身不签名；写入必须来自机构创建和账户治理流程
- 禁止兼容：不兼容旧 `Accounts` mirror、不兼容旧 `OrganizationManage` 前缀
- 禁止事项：
  - 禁止活跃代码继续读取已删的 `OrganizationManage::Institutions/InstitutionAccounts`
  - 禁止把机构账户当个人多签账户读取
- 必跑测试：
  - `cargo check -p node`(institution_read 前缀路由)
  - `public-manage` / `private-manage` 单测

### P-STORAGE-003：PersonalManage.PersonalAccounts

- 状态：当前
- 类型：storage 契约
- 唯一真源：`citizenchain/runtime/private/personal-manage/src/lib.rs`
- 详细文档：
  - `memory/05-modules/citizenchain/runtime/private/personal-manage/PERSONAL_MANAGE_TECHNICAL.md`
- 生产者：`personal-manage`
- 消费者：
  - `citizenapp/lib/transaction/personal-manage/personal_manage_storage_codec.dart`
  - `citizenapp/lib/transaction/personal-manage/personal_manage_service.dart`
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
  - `flutter test test/governance/personal-manage/personal_manage_service_test.dart test/governance/personal-manage/personal_manage_storage_codec_test.dart`

### P-STORAGE-004：Account-level internal admin account

- 状态：当前（已按分类管理员 pallet 落地）
- 类型：storage 契约 / subject id 契约
- 唯一真源：`memory/04-decisions/ADR-015-account-admin-internal-vote.md`
- 详细文档：
  - `memory/04-decisions/ADR-015-account-admin-internal-vote.md`
  - `memory/05-modules/citizenchain/runtime/admins/ADMINS_TECHNICAL.md`
  - `memory/05-modules/citizenchain/runtime/votingengine/VOTINGENGINE_TECHNICAL.md`
- 生产者：
  - `citizenchain/runtime/admins/admin-primitives`
  - `citizenchain/runtime/admins/{public-admins,private-admins,personal-admins}`
  - `citizenchain/runtime/entity/personal-manage`
  - `citizenchain/runtime/entity/public-manage`
  - `citizenchain/runtime/entity/private-manage`
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
  - `cargo test -p public-manage --lib`
  - `cargo test -p private-manage --lib`

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
