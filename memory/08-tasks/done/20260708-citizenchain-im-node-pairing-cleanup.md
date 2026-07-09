# 清理区块链(citizenchain)IM 节点配对残留

> 2026-07-08 完成。用户口径:**区块链不提供 IM**,清除链本体的 IM 节点配对死代码。公民 App 聊天是核心功能,`0x1A IM 钱包绑定`、MLS、mailbox、冷钱包一律不碰。

## 任务需求

- 删除链端「IM 节点配对(node pairing)」残留:`QR_KIND_IM_NODE_PAIRING`、`IM_NODE_PAIRING_PROTO` 及配套注释、空目录、Dart 死镜像常量。
- 这套是"把 App 和链上 IM 节点配对"用的 QR body schema,链既不再提供 IM 节点即为死代码;全仓零消费者(非签名域、不在金标向量)。

## 范围边界(务必区分两个 IM)

- **删(IM 节点配对,死代码)**:`QR_KIND_IM_NODE_PAIRING=5` / `IM_NODE_PAIRING_PROTO` / `kImNodePairingProto`。
- **不动(IM 钱包绑定,live)**:`OP_SIGN_IM_WALLET_BINDING=0x1A` / `QR_ACTION_IM_WALLET_BINDING=8`。这是公民 App 聊天设备绑定 + mailbox 登录的静默签名域,冷钱包(citizenwallet)逐字节交叉验签,且在 ADR-026 金标向量内。聊天消息本体走 MLS 端到端(设备密钥),不用钱包签名。

## 改动清单

代码:
- `citizenchain/runtime/primitives/src/sign.rs`:删 `QR_KIND_IM_NODE_PAIRING`(+注释)、`IM_NODE_PAIRING_PROTO`(+注释)、op_tag 段「IM 节点配对不是签名」注释行。⚠️runtime 改动,已获用户二次确认(「确认清理」)。
- `citizenapp/lib/signer/signing.dart`:删死镜像常量 `kImNodePairingProto` 及其 schema 段注释(全仓零引用)。
- `citizenchain/node/src/im/`:删空目录(0 文件,纯残留)。

文档(改代码后同步,保持一致无残留):
- `citizenchain/runtime/primitives/tests/fixtures/signing_domain_vectors.json`:`_comment` 去掉「IM 节点配对只是 QR body schema,亦不在此」一句(仅注释,不影响向量)。
- `memory/04-decisions/ADR-026-unified-signing-protocol.md`:第 5/11 行 `IM_NODE_PAIRING` 由「保留」改标为「2026-07-08 已删除」。
- `memory/07-ai/unified-naming.md`:IM 通信节点配对二维码行补注常量已一并删除。
- 保留不动的护栏文档(已写「k=5 已删除/禁止恢复」,删除后仍准确,继续防复活):`memory/01-architecture/qr/qr-protocol-spec.md`、`memory/05-modules/citizenapp/qr/QR_TECHNICAL.md`、`memory/04-decisions/ADR-032-*`、`memory/07-ai/unified-protocols.md`、`memory/05-modules/citizenchain/node/admins-change/ADMINS_CHANGE_NODE_TECHNICAL.md`。

## 验收(真实运行态)

- `cargo check -p primitives`:干净编译。
- `cargo test -p primitives`:46 单测 + `account_derive_golden_vectors` + **`sign_golden_vectors` 全绿**(签名金标逐字节不变 → 0x1A 等 live 域完全未动,fixture JSON 仍正常解析)。
- `dart analyze lib/signer`:No issues found。
- 代码残留复核 `grep QR_KIND_IM_NODE_PAIRING|IM_NODE_PAIRING_PROTO|kImNodePairingProto|GMB_IM_NODE_PAIRING`(.rs/.dart/.ts):零命中。

## 备注

- 未 commit / 未 push(未获授权)。
- 本次不涉及重新创世判断:仅删未被任何签名域/金标引用的常量,链上行为零变化。
