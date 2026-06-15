# 20260614 IM KeyPackage、Protobuf 与 MLS 边界

## 状态

done

## 任务需求

在区块链节点软件和公民中继续完善 IM P2P 通信基础能力：把 IM wire schema 放入 `wuminapp/im/proto/`，在节点 `citizenchain/node/src/im/` 内增加 KeyPackage 池，并在公民端增加 MLS 加密边界模型，为后续 OpenMLS FFI 接入做准备。

## 预计修改目录

- `wuminapp/im/proto/`：新增 IM Protobuf 真源 schema；只定义跨端 wire 结构，不生成代码。
- `citizenchain/node/src/im/`：新增 KeyPackage owner-only 池，接入现有 `/gmb/im/1` request-response、owner RPC 与 Tauri 命令。
- `wuminapp/lib/im/`：新增 MLS 边界模型，并让私人节点 transport 能发布、拉取、消费 KeyPackage。
- `wuminapp/test/im/`：新增公民端 MLS 边界单测，覆盖 KeyPackage JSON/hex 转换。
- `citizenchain/scripts/`：升级 IM 双节点 smoke，验证 KeyPackage 真实跨节点发布、拉取、消费。
- `memory/05-modules/` 与 `memory/07-ai/`：更新技术文档、协议登记和命名登记，清理旧口径残留。

## 边界

- 不在仓库根目录新建 `proto/`。
- 不复用钱包私钥、SFID 身份或链上账户密钥做 IM 设备身份。
- 钱包账户仅作为聊天账户和后续公民币转账收款账户标识。
- 私人通信全节点只服务 owner 自己，不做公共 DHT、公共 rendezvous、公共 relay 或第三方消息仓库。
- 本任务不接入 OpenMLS FFI 实现，只固定边界、schema 和 KeyPackage 分发池。

## 验收

- `cargo test -p node im::`
- `cargo check -p node`
- `flutter analyze`
- `flutter test --concurrency=1`
- `citizenchain/scripts/im-two-node-smoke.sh`
- `git diff --check`

## 完成记录

- 已新增 `wuminapp/im/proto/im_envelope.proto`，作为 GMB_IM_V1 外层 Protobuf schema 真源，不在仓库根目录放置 proto。
- 已新增 `citizenchain/node/src/im/keypackage.rs`，实现 owner-only KeyPackage 池、TTL、容量、一次性消费和 `base-path/im/keypackages.json` 持久化。
- 已把 KeyPackage 发布、本机查询/消费、直连拉取/消费接入 Tauri 命令、owner RPC、debug RPC 和 `/gmb/im/1` request-response。
- 已新增 `wuminapp/lib/im/crypto/im_mls_boundary.dart`，明确钱包账户只是聊天账户/收款账户，IM 设备身份和 KeyPackage 独立于钱包私钥。
- 已扩展 `ImPrivateNodeTransport`，支持发布自己的 KeyPackage、从对方私人节点直连拉取 KeyPackage、声明消费 KeyPackage。
- 已升级 `citizenchain/scripts/im-two-node-smoke.sh`，真实验证 KeyPackage 发布、重启恢复、直连拉取、消费、已消费不再返回，以及原密文 mailbox 投递/ack 持久化链路。
- 已更新 `memory/05-modules/wuminapp/im/IM_TECHNICAL.md`、`memory/05-modules/citizenchain/node/NODE_TECHNICAL.md`、`memory/07-ai/unified-protocols.md`、`memory/07-ai/unified-naming.md`，清理根目录 proto 旧口径残留。
