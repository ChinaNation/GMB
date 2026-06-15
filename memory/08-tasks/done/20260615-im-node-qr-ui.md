# 20260615 通信节点二维码 UI 收口

- 任务编号：20260615-im-node-qr-ui
- 状态：done
- 所属模块：citizenchain node frontend
- 当前负责人：Codex
- 创建时间：2026-06-15

## 任务需求

按用户确认收口区块链软件设置页通信节点面板：

- 删除“公民扫码配对”和“打开公民 App...”说明文字。
- 删除节点 PeerId 卡片，把二维码缩小到原节点 PeerId 卡片高度，并移动到原节点 PeerId 卡片位置。
- 开启/关闭确认弹窗不能使用浏览器原生 `window.confirm`，标题显示“确定”，按钮显示“取消 / 确认”。

## 本轮执行范围

- `citizenchain/node/frontend/settings/communication-node/`：调整通信节点设置页结构和开关确认逻辑。
- `citizenchain/node/frontend/app/styles/global.css`：调整二维码小卡片和确认弹窗样式，清理说明块样式残留。
- `memory/08-tasks/`：记录本次 UI 收口。

## 必须遵守

- 不修改 `citizenchain/runtime/`。
- 不恢复通信节点 RPC。
- 不新增复杂解释文案。

## 验收标准

- 通信节点面板不再出现“公民扫码配对”和“打开公民 App...”说明。
- 原节点 PeerId 卡片位置显示小二维码，二维码高度接近原卡片高度，点击仍可放大。
- 开启/关闭弹窗标题为“确定”，按钮为“取消”和“确认”。

## 实施记录

- `CommunicationNodeSection.tsx` 移除大块扫码说明区，原 PeerId 卡片位置改为小二维码入口；节点未生成配对信息时保留简短占位。
- 开启/关闭按钮改为组件内确认弹窗，弹窗标题为“确定”，按钮为“取消 / 确认”，不再使用浏览器原生确认框。
- `global.css` 删除旧二维码说明区样式，新增小二维码入口和确认弹窗样式。

## 验收记录

- `npm run build`：通过。
- 残留扫描：通过，前端源码、生成文件、dist、node 源码、wuminapp 源码中未发现旧扫码说明文案、旧二维码说明样式或 `window.confirm`。
