# 20260614 IM 通信节点独立开关与公民配对设置

## 状态

done

## 任务需求

按用户确认的产品模型落地通信节点配对：区块链软件中“通信全节点”必须从归档/普通全节点模式中拆出，作为独立通信功能开关；公民在“我的 -> 设置”的“安全”和“关于”之间新增“设置通信节点”单行入口，点击进入设置通信节点页面，通过扫码区块链软件通信节点二维码配对或更换配对节点。

## 边界

- 本任务不得修改 `citizenchain/runtime/`，任何 runtime 变更必须另行二次确认。
- 信息 Tab 不承载通信节点设置入口。
- 通信全节点不是归档/普通全节点的互斥模式，而是独立 IM 能力开关。
- 扫码配对只保存自己的电脑通信节点配置，不添加联系人。
- 转账流程仍归既有交易页面，不纳入 IM。

## 预计修改目录

- `wuminapp/lib/my/`：在“我的 -> 设置”中新增“设置通信节点”入口；涉及代码和中文注释。
- `wuminapp/lib/im/`：新增设置通信节点页面、扫码保存配对配置、显示节点简要信息；涉及代码和测试。
- `wuminapp/lib/qr/`：新增 `im_node_pairing` 二维码 body 解析；涉及扫码协议代码。
- `citizenchain/node/src/settings/`：新增通信节点独立开关和配对信息命令；涉及代码和测试。
- `citizenchain/node/frontend/settings/`：拆分全节点模式与通信节点功能，显示开关和配对二维码；涉及代码和页面构建。
- `memory/05-modules/`：更新 wuminapp IM 与 node 技术文档；涉及文档。
- `memory/07-ai/`：登记配对协议和命名；涉及文档。
- `memory/08-tasks/`：记录执行与验收结果；涉及任务卡。

## 验收

- `flutter analyze`
- IM/QR/设置通信节点相关 Flutter 测试
- `cargo test -p node settings::`
- `cargo test -p node im::`
- `npm run build`
- `git diff --check`
- 残留扫描：不得出现通信全节点与归档/普通模式二选一口径

## 实施结果

- 已将区块链软件设置页的“通信节点功能”从“全节点模式”中拆出：`归档全节点/普通全节点` 只表示链数据模式，通信节点功能由独立开关控制。
- 已新增 `citizenchain/node/src/settings/communication-node/`，持久化 `<app_data>/communication-node.json`，生成 `WUMIN_QR_V1 / im_node_pairing` 临时二维码，二维码 body 为 `GMB_IM_NODE_PAIRING_V1`。
- 已新增桌面前端 `frontend/settings/communication-node/CommunicationNodeSection.tsx`，开启后显示 PeerId、RPC、multiaddr、二维码和刷新按钮。
- 已在公民“我的 -> 设置”的“安全”和“关于”之间新增“设置通信节点”单行入口；页面未设置时引导扫码，已设置时展示节点概要，右上角扫码可更换节点。
- 已新增 `wuminapp/lib/qr/bodies/im_node_pairing_body.dart`，扫码协议支持 IPv4、IPv6、dns4、dnsaddr 端点校验。
- 已清理钱包页扫码分派：通信节点二维码不会被当成钱包地址、转账码或联系人码。
- 已更新 wuminapp IM、node 技术文档、统一协议和统一命名登记。
- 本任务未修改 `citizenchain/runtime/`。

## 验收结果

- `flutter analyze`：通过。
- `flutter test test/qr/im_node_pairing_body_test.dart test/im/im_node_settings_page_test.dart`：通过。
- `cargo test -p node settings::node_mode`：通过。
- `cargo test -p node settings::communication_node`：通过。
- `cargo test -p node im::`：通过。
- `npm run build`：通过。
- `git diff --check`：通过。
- 残留扫描：通过，未发现通信节点与归档/普通全节点模式二选一口径。
