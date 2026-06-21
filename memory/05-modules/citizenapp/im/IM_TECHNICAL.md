# citizenapp P2P IM 技术架构

## 1. 模块定位

`citizenapp` 的 P2P IM 是公民端独立通信功能。用户可见聊天号使用钱包地址；消息加密、设备身份、路由记录和通信节点 mailbox 都归 IM 模块管理。

底部 Tab 目标顺序：

```text
公民 / 多签 / 信息 / 交易 / 我的
```

当前产品入口：

- “我的 -> 用户资料 -> 设置通信账户”：选择默认发消息的钱包地址；未设置时不能发消息。
- “我的 -> 设置 -> 设置通信节点”：扫描区块链软件设置页通信节点二维码，保存或更换自己电脑通信节点。
- “我的 -> 我的通讯录 -> 联系人详情 -> 消息”：进入与该联系人钱包地址的聊天窗口。
- “信息”Tab：只展示已有会话列表和进入聊天详情，不提供工程入口。
- “我的通讯录 -> 联系人详情 -> 转账”：继续跳转既有交易页面，不归 IM 流程处理。

核心边界：

- 一台电脑区块链软件可以作为同一用户的通信节点，服务该用户多台手机和多个钱包聊天号。
- 通信节点只服务自己的 citizenapp 和自己的钱包聊天号，不互为节点，不做公共中继，不替第三方存消息。
- IM 路由记录只是通信模块的本地运行数据，不属于链业务数据，不进入 SFID 身份体系。
- 钱包私钥只用于证明“这个 IM 设备属于这个钱包地址”，不用于 OpenMLS 消息加密。

## 2. 总体架构

远程通信路径：

```text
我手机（公民）
  -> 我电脑通信节点（只服务我，只存我授权账号的密文队列）
  -> 对方电脑通信节点（只服务对方，只接收投递给对方钱包地址的密文）
  -> 对方手机（公民）
```

近场旁路：

```text
我手机 <-> 对方手机
```

手机端目录：

```text
citizenapp/lib/im/
├── im_tab_page.dart              信息 Tab 会话列表
├── im_chat_page.dart             聊天详情页
├── im_runtime.dart               通信账户、节点配对、OpenMLS、收发编排
├── im_message_flow.dart          远程消息收发状态机
├── proto/                        Protobuf 生成类型
├── crypto/                       OpenMLS、设备密钥、绑定 payload
├── storage/                      Isar 会话、消息、路由缓存、队列
└── transport/                    自己通信节点专用 P2P 通道与后续近场传输
```

节点端目录：

```text
citizenchain/node/src/im/
├── mailbox.rs                    多钱包账号密文 mailbox
├── keypackage.rs                 多钱包账号 KeyPackage 池
├── binding.rs                    钱包地址与 IM 设备授权
├── endpoint.rs                   IPv4 / IPv6 / dns4 / dnsaddr 端点校验
├── direct.rs                     显式端点直连投递
├── network.rs                    /gmb/im/1 request-response wire
└── commands.rs                   桌面端调试和验收命令
```

## 3. 钱包地址与 IM 身份模型

IM 身份分层：

```text
聊天账户 = 钱包地址 / SS58 地址
IM 设备身份 = 手机本地独立 OpenMLS 设备密钥
通信节点 = 用户自己电脑上的通信节点能力
```

钱包地址作为聊天号：

- 用户可以在 citizenapp 里有多个钱包。
- 用户资料里的“设置通信账户”决定默认用哪个钱包地址发消息。
- 用户可以随时切换通信账户；切换后，新发送消息使用新的钱包地址。
- 同一台电脑通信节点可以授权多个钱包地址和多台手机设备。

钱包签名只用于设备授权：

```text
GMB_IM_WALLET_BINDING_V1
| wallet_account
| im_device_id
| im_device_pubkey
| node_peer_id
| node_endpoints
| expires_at_millis
| nonce
```

禁止把钱包私钥用于：

- OpenMLS 消息加密。
- 每条聊天消息签名。
- 通信节点存储。
- 节点间投递鉴权。

## 4. 最小发送流程

文本消息发送流程必须保持短链路：

```text
联系人详情点击“消息”
-> 打开 ImChatPage
-> ImRuntime 读取用户资料通信账户
-> 查本地 ImRouteRecord 路由缓存
-> 确认本机手机已授权到自己的通信节点
-> 拉取/消费对方 KeyPackage
-> OpenMLS 生成密文 ImEnvelope
-> 自己通信节点直连对方通信节点投递
-> 对方手机从自己的通信节点拉取并解密
```

失败边界：

- 未设置通信账户：直接提示“请先在用户资料中设置通信账户”。
- 通信账户钱包不存在或地址不一致：提示用户重新设置通信账户。
- 联系人没有 IM 路由记录：提示“联系人暂未提供通信路由，暂不能发送消息”。
- 对方通信节点不可达：消息进入发送队列，后续重试。

## 5. IM 路由记录

`ImRouteRecord` 是隐藏的 IM 路由缓存，不是第二套通讯录。

字段：

```text
route_id
wallet_chat_account
display_name
im_device_id
device_public_key_hex
safety_number
node_peer_id
node_multiaddr
note
created_at_millis
updated_at_millis
```

当前使用方式：

- 联系人仍由“我的通讯录”管理。
- 聊天发送时按联系人钱包地址查 `ImRouteRecord`。
- 路由记录只保存通信所需的 PeerId、multiaddr、设备公钥和安全码。
- 后续设备配对和路由交换可以用二维码，但信息 Tab 不承担配对入口。

## 6. E2EE 与 Protobuf

端到端加密主选 OpenMLS：

- v1 使用经典 MLS 密码套件。
- `suite_version` 保留 hybrid / PQC 迁移字段。
- v1 不宣称抗量子；MLS 密码敏捷性用于未来切换空间。
- KeyPackage 以设备为单位发布到自己的通信节点。
- KeyPackage 必须具备 TTL、一次性消费、防重放和撤销清理。
- 当前 native 边界已支持持久化 MLS 会话、Welcome/application wire bytes、重启恢复和真实 KeyPackage 生成。

外层协议使用 Protobuf：

```text
真源文件：citizenapp/im/proto/im_envelope.proto
生成命令：
PATH="$HOME/.pub-cache/bin:$PATH" protoc \
  --dart_out=lib/im/proto \
  -I im/proto \
  im/proto/im_envelope.proto
```

主要消息：

```text
ImEnvelope
ImRouteRecord
RegisterImDeviceRequest
SubmitImEnvelopeRequest
ImDirectDeliveryRequest
ImKeyPackage
PublishImKeyPackageRequest
FetchImKeyPackagesRequest
ConsumeImKeyPackageRequest
```

`ImEnvelope.mls_wire_message` 承载 OpenMLS 标准 wire bytes；链内 SCALE 不作为 IM 主协议。

## 7. 通信节点能力

通信节点能力放在 `citizenchain/node/src/im/`。

通信节点允许：

- 保存已授权钱包地址的密文 mailbox。
- 保存已授权钱包地址的 KeyPackage 池。
- 接收别人投递给本机授权钱包地址的密文。
- 给自己的手机提供拉取、确认和删除。
- 管理自己的手机设备授权。
- 向对方通信节点直连投递密文。

通信节点禁止：

- 不给第三方做 Relay。
- 不做公共 rendezvous。
- 不做公共 DHT 基础设施。
- 不替别人存消息。
- 不解密消息。
- 不处理钱包、治理、交易业务。

节点协议名：

```text
/gmb/im/1
```

桌面端设置边界：

- 设置 Tab 的“通信节点功能”是独立 IM 能力开关。
- 归档全节点和普通全节点都可以开启通信节点功能；开启通信节点功能不改变链数据运行边界。
- 普通全节点裁剪能力未完成时仍保持原有节点数据模式。
- 桌面端生成 `im_node_pairing` 固定二维码；公民只在“我的 -> 设置 -> 设置通信节点”扫码保存自己的通信节点信息。

## 8. 可达性与 IPv6

IM 支持 IPv4、IPv6 和用户自有域名。

支持 multiaddr：

```text
/ip4/<addr>/tcp/<port>/wss/p2p/<peer_id>
/ip6/<addr>/tcp/<port>/wss/p2p/<peer_id>
/dns4/<domain>/tcp/<port>/wss/p2p/<peer_id>
/dnsaddr/<domain>/p2p/<peer_id>
```

优先级：

1. 局域网直连。
2. IPv6 直连。
3. IPv4 端口映射 / UPnP / NAT-PMP。
4. 用户自有域名。
5. 用户自己控制的公网入口。

如果不可达，消息进入发送队列等待重试。禁止借别人通信节点中继。

## 9. 存储

citizenapp 本地：

- `ImConversationEntity`：会话索引。
- `ImMessageEntity`：消息索引和本机明文。
- `ImRouteCacheEntity`：IM 路由缓存。
- `ImOutboundQueueEntity`：发送队列。
- `ImPendingInboundEntity`：Welcome 未到前的 application 暂存。
- OpenMLS native provider storage 保存到 App 私有 MLS 状态目录。

通信节点：

- `base-path/im/mailbox.json`：多钱包账号密文 mailbox、设备授权、ack tombstone。
- `base-path/im/keypackages.json`：多钱包账号 KeyPackage 池、TTL、消费时间。
- 只保存密文 envelope、KeyPackage 和必要索引。
- 容量、TTL、附件大小必须有限额。

## 10. 当前代码状态

- `citizenapp/lib/main.dart` 已在底部多签和交易之间插入“信息”Tab。
- `citizenapp/lib/im/im_tab_page.dart` 已收口为会话列表，不再暴露工程入口。
- `citizenapp/lib/my/user/user.dart` 已在“我的 -> 设置”的“安全”和“关于”之间插入“设置通信节点”单行入口。
- `citizenapp/lib/im/im_node_settings_page.dart` 已提供通信节点未设置/已设置状态、右上角扫码更换和首次扫码配对。
- `citizenapp/lib/qr/bodies/im_node_pairing_body.dart` 已登记 `GMB_IM_NODE_PAIRING_V1`，支持 IPv4/IPv6/dns4/dnsaddr multiaddr 校验。
- `citizenapp/lib/my/user/user.dart` 的联系人详情“消息”按钮已接入 `ImChatPage`。
- `citizenapp/lib/my/user/user_service.dart` 已提供通信账户设置状态。
- `citizenapp/lib/im/im_runtime.dart` 已按用户资料通信账户发送消息，并使用 `ImPairedNodeConfig` 保存自己的通信节点配置。
- `citizenapp/lib/im/storage/im_isar_store.dart` 已将原联系人语义收口为 `ImRouteRecord` / `ImRouteCacheEntity`。
- `citizenapp/im/proto/im_envelope.proto` 已作为 Protobuf 真源放在 citizenapp 内部目录。
- `citizenapp/lib/im/proto/` 已从该真源生成 Dart 类型。
- `citizenapp/lib/im/crypto/` 已接入 OpenMLS native 边界、设备密钥、KeyPackage 和绑定 payload。
- `citizenapp/lib/im/im_message_flow.dart` 已串联 OpenMLS、Protobuf envelope、通信节点投递和 Isar 状态。
- `citizenapp/lib/im/transport/im_private_node_transport.dart` 已移除节点 RPC 客户端，仅保留通信节点端点和后续专用 P2P 通道占位。
- `citizenchain/node/src/im/mailbox.rs` 已支持一台通信节点服务多个钱包账号和多个授权设备。
- `citizenchain/node/src/im/keypackage.rs` 已支持多钱包账号 KeyPackage 池。
- `citizenchain/node/src/settings/node-mode/mod.rs` 已移除旧 `communication` 选项，全节点模式只保留归档/普通链数据模式。
- `citizenchain/node/src/settings/communication-node/mod.rs` 已提供独立通信节点功能开关、IPv4/IPv6 配对端点和不含 RPC URL / 有效期的 CITIZEN_QR_V1 固定配对二维码。
- `citizenchain/scripts/im-two-node-smoke.sh` 已覆盖双节点 KeyPackage、直连投递、重启恢复、ack 和第三方 mailbox 拒绝。

## 11. 预计修改目录

- `citizenapp/lib/im/`：公民 IM 信息 Tab、聊天页面、运行态、消息状态机和传输抽象；涉及代码、中文注释和残留清理。
- `citizenapp/im/proto/`：GMB_IM_V1 外层 Protobuf schema 真源；涉及协议文件，不放仓库根目录。
- `citizenapp/lib/im/proto/`：Dart Protobuf 生成物；涉及生成代码。
- `citizenapp/lib/im/crypto/`：OpenMLS、设备密钥、KeyPackage、安全码和钱包账户绑定；涉及代码。
- `citizenapp/lib/im/storage/`：Isar 会话、消息、路由缓存、发送队列和 pending 入站 envelope；涉及代码。
- `citizenapp/lib/im/transport/`：手机连接自己通信节点的专用 P2P 通道占位和后续近场传输抽象；涉及代码。
- `citizenapp/lib/my/user/`：用户资料通信账户和通讯录详情消息入口；涉及代码。
- `citizenapp/lib/isar/`：IM 本地数据库 schema；涉及代码生成和迁移边界。
- `citizenapp/android/im/`：Android 近场能力预留目录；涉及后续原生代码。
- `citizenapp/ios/im/`：iOS 近场能力预留目录；涉及后续原生代码。
- `citizenchain/node/src/im/`：通信节点 mailbox、KeyPackage、设备授权、端点校验和 `/gmb/im/1`；涉及代码。
- `citizenchain/node/src/settings/communication-node/`：桌面端通信节点功能独立开关和配对二维码；涉及代码。
- `citizenchain/node/frontend/settings/communication-node/`：桌面端设置页通信节点功能展示、开关和二维码；涉及代码。
- `citizenchain/node/src/settings/node-mode/`：只保留归档/普通全节点数据模式；涉及旧 `communication` 选项残留清理。
- `citizenchain/node/frontend/settings/node-mode/`：只展示归档/普通全节点数据模式；涉及旧 `communication` 选项残留清理。
- `memory/05-modules/citizenapp/im/`：citizenapp IM 技术文档；涉及文档。
- `memory/05-modules/citizenchain/node/`：通信节点边界文档；涉及文档。
- `memory/07-ai/`：`GMB_IM_V1` 协议和命名登记；涉及文档残留清理。

## 12. 验收要求

- `flutter analyze`
- IM 相关 Flutter 测试。
- `cargo test -p node im::`
- `cargo test -p node settings::node_mode`
- `citizenchain/scripts/im-two-node-smoke.sh`
- `git diff --check`
