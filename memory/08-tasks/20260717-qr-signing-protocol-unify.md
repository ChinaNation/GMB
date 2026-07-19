# 全仓扫码签名协议统一

- 创建日期：2026-07-18
- 状态：已完成
- 范围：QR_V1、action registry、扫码签名两色判定、中文展示规则
- 当前阶段：第 8 步已完成，已修复 runtime 管理员存储升级预检、OnChina dryRun 提交边界和移动端固定展示值唯一真源

## 任务目标

全仓所有扫码签名统一为一个协议、一个 action registry、一个签名判定模型和一个中文展示规则。

目标态只允许两种签名状态：

- 正常：绿色，payload 已完整解码、动作和字段已完整中文翻译，允许签名。
- 拒绝：红色，任一校验、解码或中文翻译失败，不允许签名。

禁止第三种状态，禁止未知 action 或未翻译字段继续签名，禁止展示 `动作 7941`、`载荷 32 字节`、英文 action key 或原始 payload 作为用户确认内容。

## 第 1 步范围

本阶段只锁定标准和真源落点：

- 确认后续唯一 action registry 代码真源落点为 `citizenchain/crates/qr-protocol/registry/*`。
- 确认 QR_V1 普通签名请求的 `b.d` 是完整 `review_payload`，不是 32 字节 signing bytes。
- 确认 Runtime 升级是唯一允许 hash-only 的签名场景。
- 确认 `SignDecision` 只有 `Normal` 和 `Reject` 两种结果。
- 更新 `memory/` 文档，标记现有散落实现不得继续作为真源。

第 1 步不修改 `citizenchain/runtime/`，不修改移动端和 OnChina 业务代码。

## 后续执行边界

第 2 步已创建 `citizenchain/crates/qr-protocol/` 代码真源包，并由该包保存和校验 action 常量、中文标签、字段标签、拒绝原因和两态签名判定 schema。

## 第 2 步完成记录

已新增并接入:

- `citizenchain/crates/qr-protocol/registry/actions.yaml`
- `citizenchain/crates/qr-protocol/registry/fields.yaml`
- `citizenchain/crates/qr-protocol/registry/reject_reasons.yaml`
- `citizenchain/crates/qr-protocol/src/registry.rs`
- `citizenchain/crates/qr-protocol/src/decision.rs`
- `citizenchain/crates/qr-protocol/src/export.rs`
- `citizenchain/crates/qr-protocol/tests/registry_consistency.rs`

已完成验收:

- `cargo test --manifest-path citizenchain/crates/qr-protocol/Cargo.toml`
- `cargo check --manifest-path citizenchain/Cargo.toml -p qr-protocol`

## 第 3 步完成记录

已改造公民钱包扫码签名链路:

- `citizenwallet/lib/signer/offline_sign_service.dart`：删除 `matched/mismatched/decodeFailed` 三态，统一为 `SignDecisionStatus.normal/reject`。
- `citizenwallet/lib/ui/offline_sign_page.dart`：红色拒绝不再显示 payload 字节数，不再显示动作数字；只有 normal 绿色态可点击“确认签名”。
- `citizenwallet/lib/signer/action_labels.dart`：补 QR 数字 action 到中文动作名映射，未登记 action 返回 null 并拒绝。
- `citizenwallet/lib/signer/field_labels.dart`：未登记字段不再返回“未知字段”，签名放行前必须完整中文翻译。
- `citizenchain/crates/qr-protocol/registry/actions.yaml` 与 `fields.yaml`：补齐公民钱包当前 decoder 已支持的 action 和字段中文登记。

已完成验收:

- `flutter analyze`（`citizenwallet`）
- `flutter test`（`citizenwallet`）
- `cargo test --manifest-path citizenchain/crates/qr-protocol/Cargo.toml`
- `cargo check --manifest-path citizenchain/Cargo.toml -p qr-protocol`

## 第 4 步完成记录

已改造 OnChina / node 链交易二维码生成端:

- `citizenchain/crates/chain-signing/src/lib.rs`：统一产出 `payload=review_payload` 与 `signing_bytes`；普通链交易 QR 使用完整审阅载荷，sr25519 签名仍严格复用 Substrate `SignedPayload::using_encoded` 规则。
- `citizenchain/onchina/src/core/chain_submit.rs`：prepare 阶段明确 `payload` 是 QR `b.d` 审阅载荷，`signing_hash_hex` 只用于提交前重建校验。
- `citizenchain/onchina/src/core/qr/sign_request.rs`：注释明确普通链交易传入值必须是完整 `review_payload`，不得用 32 字节 `signing_bytes` 代替。
- `citizenchain/node/src/governance/signing.rs`：节点普通链交易签名请求同步使用完整 `review_payload` 放入 QR。
- `citizenchain/node/src/governance/runtime_upgrade/signing.rs`：保留 Runtime 升级作为唯一 hash-only 例外，并在注释中与普通链交易隔离。

已完成验收:

- `cargo test --manifest-path citizenchain/crates/chain-signing/Cargo.toml`
- `cargo test --manifest-path citizenchain/onchina/Cargo.toml core::chain_submit`
- `cargo check --manifest-path citizenchain/Cargo.toml -p node`

## 第 5 步完成记录

已改造两个移动端扫码签名识别与展示:

- `citizenapp/lib/qr/qr_protocols.dart`：补 action registry 镜像查询能力，签名入口可按 action code 查中文动作名；未登记返回 null。
- `citizenapp/lib/signer/square_action_payload.dart`：广场账户动作 payload 不再有英文 fallback，动作类型和字段名必须能中文展示，否则解码失败并拒签。
- `citizenapp/lib/signer/square_action_sign_service.dart`：公民端签名前先校验 action 已登记、有中文名、且仅支持广场账户动作；未知 action 和已登记但非本端签名动作均拒绝，不触发签名。
- `citizenapp/lib/qr/scan_dispatch_flow.dart` 与 `qr_sign_response_page.dart`：确认页/结果页展示中文动作名与中文字段列表。
- `citizenwallet/lib/qr/qr_protocols.dart` 与 `citizenwallet/lib/signer/action_labels.dart`：补 `square_account_action` 中文登记；公民钱包扫到该动作时显示中文动作名但因无 decoder 红色拒绝，不再落成未知数字。
- `citizenapp/lib/qr/bodies/sign_request_body.dart` 与 `citizenwallet/lib/qr/bodies/sign_request_body.dart`：注释统一为 `review_payload`，明确普通链交易不得用 32 字节 `signing_bytes` 冒充。

已完成验收:

- `flutter test test/signer/qr_signer_test.dart test/signer/square_action_payload_test.dart test/signer/square_action_sign_service_test.dart`（`citizenapp`）
- `flutter test test/common/admin_account_storage_codec_test.dart test/governance/admins-change/admins_change_codec_test.dart`（`citizenapp`，同步修正旧测试夹具为当前机构管理员 `(admin_name, admin_account)` storage 布局）
- `flutter test`（`citizenapp`，721 通过 / 5 跳过）
- `flutter test test/signer/field_labels_test.dart test/signer/offline_sign_service_test.dart`（`citizenwallet`）
- `flutter test`（`citizenwallet`）
- `flutter analyze`（`citizenapp`）
- `flutter analyze`（`citizenwallet`）
- `cargo test --manifest-path citizenchain/crates/qr-protocol/Cargo.toml`

## 第 6 步完成记录

已完成唯一 action registry 生成产物接入:

- `citizenchain/crates/qr-protocol/src/export.rs`：新增 Dart registry 导出，统一生成 action code、action key、中文动作名、中文字段名、拒绝原因和 hash-only 集合。
- `citizenchain/crates/qr-protocol/src/bin/export_registry.rs`：新增导出命令，后续两端 registry 更新只允许从该命令生成。
- `citizenchain/crates/qr-protocol/tests/registry_consistency.rs`：新增两端 Dart 生成文件一致性校验，registry 改动但移动端未同步会直接失败。
- `citizenchain/crates/qr-protocol/registry/actions.yaml`：补齐 `update_public_institution_info`、`add_public_institution_account`、`update_private_institution_info`、`add_private_institution_account` 四个机构动作。
- `citizenchain/crates/qr-protocol/registry/fields.yaml`：补齐治理详情字段，并把字段中文名统一为既有移动端展示文案。
- `citizenapp/lib/qr/generated/qr_action_registry.g.dart` 与 `citizenwallet/lib/qr/generated/qr_action_registry.g.dart`：由 qr-protocol 生成，禁止手改。
- `citizenapp/lib/qr/qr_protocols.dart`、`citizenwallet/lib/qr/qr_protocols.dart`：`fromDecodedAction` 和 hash-only 判断改为消费生成 registry，不再维护全量手写 switch / 手写 hash-only 列表。
- `citizenwallet/lib/signer/action_labels.dart`、`citizenwallet/lib/signer/field_labels.dart`：改为消费生成 registry，不再维护手写 action/字段中文表。
- `citizenapp/lib/signer/square_action_payload.dart`：广场账户动作字段中文名改为消费生成 registry，保留原 UI 样式和原中文展示。
- `citizenchain/onchina/src/core/qr/mod.rs`、`auth/actions.rs`、`domains/citizens/chain_identity.rs`：OnChina 登录、公民身份确认、管理员治理 3 个非链动作码改为从 `qr-protocol` registry 读取，不再硬编码 `1/2/3`。
- `citizenchain/onchina/Cargo.toml`：接入 `qr-protocol` 依赖，仅用于 host 端 QR action registry，不进入 runtime wasm。

已完成验收:

- `cargo test --manifest-path citizenchain/crates/qr-protocol/Cargo.toml`
- `flutter test test/signer/field_labels_test.dart test/signer/offline_sign_service_test.dart test/signer/payload_decoder_test.dart`（`citizenwallet`）
- `flutter analyze`（`citizenwallet`）
- `flutter test test/signer/qr_signer_test.dart test/signer/square_action_payload_test.dart test/signer/square_action_sign_service_test.dart`（`citizenapp`）
- `flutter analyze`（`citizenapp`）
- `cargo check --manifest-path citizenchain/Cargo.toml -p onchina`

残留清理:

- 已清理移动端手写全量 action label、action code 反查、字段 label 和 hash-only 列表。
- 已清理 OnChina 非链 QR action code `1/2/3` 硬编码常量。
- 已确认剩余 `matched/mismatched` 命中为签名错误码或普通“匹配”变量，不是离线签名第三状态。
- 已确认用户确认 UI 仍只显示中文动作名与中文字段列表，不恢复动作数字、英文 action key 或 `载荷 N 字节` 兜底。

## 第 7 步完成记录

已新增 QR 签名协议防漂移 guard:

- `citizenchain/crates/qr-protocol/tests/repo_guard.rs`：扫描 CitizenApp、CitizenWallet、OnChina、node 和 `citizenchain/crates` 的代码目录，禁止恢复第二 action/字段真源、OnChina 非链动作码硬编码、端侧 hash-only 手写列表、离线签名第三状态和数字/字节数确认兜底。
- `memory/01-architecture/qr/qr-action-registry.md`：新增“防漂移 guard”章节，明确新增扫码签名功能必须先改 registry、再生成两端产物、再补 decoder/签发方、最后跑 `qr-protocol` 测试。

已完成验收:

- `cargo test --manifest-path citizenchain/crates/qr-protocol/Cargo.toml`

涉及 `citizenchain/runtime/` 的任何修改必须单独二次确认。

## 第 8 步完成记录

本阶段按二次确认只修必要代码，不执行链上升级/发布动作。

已修复:

- `citizenchain/runtime/admins/public-admins/src/lib.rs` 与 `private-admins/src/lib.rs`：`AdminAccounts` 存储版本升至 v4，增加从旧 `Vec<AccountId>` 到 `Vec<InstitutionAdmin { admin_name, admin_account }>` 的一次性 migration，避免交易池 validate 阶段读取旧存储时 panic。
- `citizenchain/runtime/admins/public-admins/src/tests/mod.rs` 与 `private-admins/src/tests/mod.rs`：补旧管理员集合编码迁移回归测试。
- `citizenchain/onchina/src/core/chain_submit.rs`：`system_dryRun` RPC 层错误直接失败返回，不再继续 `author_submitExtrinsic`。
- `citizenchain/crates/chain-signing/src/lib.rs`：新增 preflight 错误中文归类，RuntimeApi / wasm trap 明确显示为“链运行时校验失败，交易未提交”。
- `citizenchain/crates/qr-protocol/registry/fields.yaml`、`src/registry.rs`、`src/export.rs`：字段 registry 增加 `field_value_zh`，固定展示值由唯一 registry 生成。
- `citizenapp/lib/qr/generated/qr_action_registry.g.dart` 与 `citizenwallet/lib/qr/generated/qr_action_registry.g.dart`：重新生成，包含 `fieldValueZhByKey`。
- `citizenwallet/lib/signer/payload_decoder.dart`：创建机构、机构治理和机构账户交易的“默认岗位/制度账户/费用付款账户”展示值只从生成 registry 读取，不再在 decoder 里保存第二份中文值。
- `citizenwallet/test/signer/payload_decoder_test.dart`：创建机构 payload decoder 测试不再手写旧固定文案，改为读取生成 registry，避免测试成为第二展示真源。

已完成验收:

- `cargo test --manifest-path citizenchain/Cargo.toml -p public-admins -p private-admins`
- `cargo test --manifest-path citizenchain/crates/qr-protocol/Cargo.toml`
- `cargo test --manifest-path citizenchain/crates/chain-signing/Cargo.toml`
- `cargo check --manifest-path citizenchain/Cargo.toml -p onchina`
- `flutter test test/signer/payload_decoder_test.dart`（`citizenwallet`）
- `flutter analyze`（`citizenwallet`）
- `flutter analyze`（`citizenapp`）

## 验收标准

- 文档中明确唯一 action registry 推荐落点。
- 文档中明确 `b.d = review_payload`。
- 文档中明确普通交易禁止 hash-only。
- 文档中明确签名状态只有正常 / 拒绝。
- 文档中明确无中文翻译一律拒绝。
- 文档中明确现有多端散落 QR/action/signing/display 实现不是协议真源。
