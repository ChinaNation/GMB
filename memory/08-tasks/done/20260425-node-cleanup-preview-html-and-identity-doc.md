# node 历史遗留同步：删除 preview html + 修正 identity 文档

- 时间:2026-04-25
- 状态:done
- 归属:Blockchain Agent (citizenchain/node)

## 背景

2026-04-25 节点 UI 已统一改为 App 生命周期托管（开 App = 启节点 / 退 App = 停节点 / 关窗 = 最小化），前端不再有任何启停按钮和密码输入框，`start_node` / `stop_node` Tauri command 与 `verify_start_unlock_password` 链路均已删除。

但仓库内仍有两处历史遗留与现状不符：

1. `citizenchain/node/frontend/ui-preview-a.html` / `ui-preview-b.html` / `ui-preview-c.html` 三份早期 UI 设计稿仍含 "启动节点" / "btn-start" 按钮。这些 preview html 不在 vite 入口、不影响生产，但作为设计参考已与现状不符。
2. `memory/05-modules/citizenchain/node/home/HOME_TECHNICAL.md` 第 73-78 行 identity/mod.rs 段提到 `set_node_name` Tauri command（"节点名称管理：set_node_name（需设备密码验证）"），但 `grep "set_node_name" citizenchain/node/src` 零命中——该 command 早已删除，属预先存在的文档遗留。

## 现状（执行前）

- `citizenchain/node/frontend/ui-preview-a.html`（5969 字节，含 "btn-start" 启动节点按钮）
- `citizenchain/node/frontend/ui-preview-b.html`（5885 字节，含同款按钮）
- `citizenchain/node/frontend/ui-preview-c.html`（7036 字节，含同款按钮）
- `HOME_TECHNICAL.md:73-78` identity/mod.rs 段：错误描述 `set_node_name`（需设备密码验证）

## 现状（执行后）

- 三份 ui-preview-*.html 已删除
- `HOME_TECHNICAL.md:73-78` identity/mod.rs 段已改为：
  - `current_status`（PID、运行标志、PeerId、节点名）
  - `get_node_identity`（节点名 + PeerId 一次性返回）
  - `PeerId 获取`（从 RPC `system_localPeerId`）
- `grep "ui-preview" citizenchain/node` 零命中
- `grep "get_node_identity"` 命中真实实现（`identity/mod.rs:98`）
- `grep "set_node_name"` 仍零命中（验证文档不再提及不存在的 command）

## 决策

- 项 1 选 **方案 A 删除三份 preview html**（用户原话："没有了就删掉！"）
- 项 2 直接执行（无歧义）
- 任务范围严格限定在上述两项，不扩散到其它历史文档同步

## 验收

- [x] 三份 ui-preview-*.html 文件已删除
- [x] HOME_TECHNICAL.md identity/mod.rs 段已修正为实际职责
- [x] grep 验证 ui-preview 与 set_node_name 均零命中
- [x] grep 验证 get_node_identity 在源码真实存在

## 关联

- 文档：`memory/05-modules/citizenchain/node/home/HOME_TECHNICAL.md`
- 源码：`citizenchain/node/src/ui/home/identity/mod.rs`
- 历史：`memory/08-tasks/done/20260425-083200-ui-app.md`（App 生命周期托管整改任务）
