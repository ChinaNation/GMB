# 20260615 IM 钱包地址聊天号与通信节点模式收口

## 状态

done

## 任务需求

按用户最新确认的产品模型改造公民 IM：钱包地址就是聊天号；发送账号使用“我的-用户资料-设置通信账户”；我的通讯录联系人详情里的“消息”按钮是正式聊天入口；信息 Tab 只展示会话列表，不再暴露扫码、我的码、节点、新增联系人等工程入口；电脑端全节点设置 Tab 中的通信节点模式是独立能力开关，不改变普通全节点或归档全节点身份；通信节点支持一台电脑服务多个 wuminapp、多个钱包聊天账号，并支持 IPv6。

## 边界

- IM 路由记录只属于 IM 系统，和链业务无关。
- 通讯录“转账”继续跳现有交易页面，不纳入 IM 发送流程。
- 通信节点只服务自己的手机和钱包地址，不做公共中继、不替第三方存消息。
- 钱包私钥只用于设置通信账户后的设备授权签名，不用于聊天加密。
- 本任务优先完成入口、运行态、node 多账号模型、IPv6 与文档残留收口；不实现新的公共发现服务。

## 预计修改目录

- `wuminapp/lib/im/`：收口信息 Tab、改造运行态发送账号读取逻辑、删除工程入口，保留会话列表和聊天详情；涉及代码、中文注释和残留清理。
- `wuminapp/lib/my/user/`：接通联系人详情“消息”按钮，使用现有通讯录作为唯一联系人入口；涉及代码。
- `wuminapp/lib/isar/`：将 IM 联系人语义调整为 IM 路由缓存，避免第二套联系人体系；涉及 schema、生成文件和测试。
- `citizenchain/node/src/im/`：将单 owner mailbox / KeyPackage 池改为多钱包、多设备模型，并保留 IPv6 endpoint 支持；涉及核心代码和测试。
- `citizenchain/node/frontend/settings/`：检查并记录通信节点模式作为独立能力开关的设置边界；如现有页面已具备则只更新文档。
- `memory/05-modules/wuminapp/im/`：更新 IM 技术文档，明确通信账户、通讯录入口、精简发送流程和 IM 路由记录边界；涉及文档清理。
- `memory/05-modules/citizenchain/node/`：更新全节点通信模式说明，明确不影响普通/归档全节点；涉及文档。
- `memory/07-ai/`：更新统一协议和命名，清理旧的扫码/我的码/新增联系人/单 owner 描述；涉及文档残留清理。
- `memory/08-tasks/`：记录本任务执行与验收结果；涉及任务卡文档。

## 验收

- `flutter analyze`：通过。
- IM 相关 Flutter 测试：通过，包含信息 Tab、路由缓存、Protobuf、Isar、OpenMLS native、OpenMLS 会话和聊天 UI 适配。
- `cargo test -p node im::`：通过，19 个 IM 测试通过。
- `cargo test -p node settings::node_mode`：通过，4 个全节点模式设置测试通过。
- `cargo test`（`wuminapp/rust`）：通过，2 个 OpenMLS Rust FFI 测试通过。
- `cargo build --release`（`wuminapp/rust`）：通过，生成 macOS `libsmoldot.dylib` 供 Flutter VM 原生测试加载。
- `citizenchain/scripts/im-two-node-smoke.sh`：通过，验证两个真实 headless 节点的 KeyPackage、直连投递、重启恢复、授权设备拉取、ack 和第三方 mailbox 拒绝。
- `npm run build`（`citizenchain/node/frontend`）：通过。
- `git diff --check`：通过。

## 执行结果

- 公民“信息”Tab 已收口为会话列表，不再承载工程入口。
- “我的通讯录 -> 联系人详情 -> 消息”已接入 `ImChatPage` 和 `ImRuntime.sendText`。
- 发送账号改为读取“我的 -> 用户资料 -> 设置通信账户”；未设置时阻止发送。
- IM 本地联系人语义已改为 `ImRouteRecord` / `ImRouteCacheEntity` 路由缓存，避免第二套通讯录。
- IM 支付提示模型和 `payment_notice` 残留已删除；转账继续归既有交易页面。
- 通信节点 mailbox / KeyPackage 池已支持同一电脑节点服务多个钱包聊天号和多个授权手机设备。
- 桌面设置页“通信全节点”已改为可选择的 IM 能力开关，普通全节点仍保持待完成。
- 通信模式设置旧手机节点接口已在 2026-06-15 删除；通信节点不得恢复节点 RPC。
- IPv6 endpoint 验收继续覆盖在 node IM endpoint 测试与 Protobuf route record 测试中。
- `IM_TECHNICAL.md`、`NODE_TECHNICAL.md`、`unified-protocols.md`、`unified-naming.md` 已更新到当前口径。
