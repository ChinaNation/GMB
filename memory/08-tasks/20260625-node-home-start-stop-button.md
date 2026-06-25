# 任务卡：首页状态区节点启停按钮

## 任务需求

在区块链节点软件首页 tab 的左侧节点状态区域增加手动启停按钮：

- 节点运行中时按钮显示“关闭”，点击后停止节点。
- 节点停止时按钮显示“启动”，点击后手动启动节点。
- 按钮显示在“状态: 运行中/已停止”的右侧，点击后先弹出二次确认。
- 保留打开软件自动启动节点；首页按钮只作为手动启停入口，两者不冲突。

## 预计修改目录

- `citizenchain/node/frontend/home/`
  - 用途：首页节点状态区增加启停按钮，并接入专用 Tauri API。
  - 边界：只处理首页节点状态与链信息面板，不调整交易面板业务。
  - 类型：前端代码修改。

- `citizenchain/node/frontend/app/styles/`
  - 用途：补充状态区启停按钮样式。
  - 边界：只新增按钮局部样式，不重做整体主题。
  - 类型：前端样式修改。

- `citizenchain/node/src/home/`
  - 用途：复用已有节点生命周期函数，暴露前端可调用的手动启动/停止命令。
  - 边界：不改变节点自动启动、设置页保存即重启、更新前停节点等既有流程。
  - 类型：后端代码修改。

- `citizenchain/node/src/desktop/`
  - 用途：注册新的节点手动启停 Tauri command。
  - 边界：只调整桌面命令注册与说明，不改 runtime。
  - 类型：后端代码修改。

- `memory/05-modules/citizenchain/node/`
  - 用途：更新首页节点生命周期技术文档，清理“前端无启停按钮”等旧口径。
  - 边界：只同步当前目标态和残留说明。
  - 类型：文档维护。

- `memory/08-tasks/`
  - 用途：记录本任务执行、验收和结论。
  - 边界：只更新本任务卡。
  - 类型：任务文档。

## 验收要求

- `npm run build`（`citizenchain/node/frontend`）
- `cargo fmt --manifest-path citizenchain/node/Cargo.toml --check`
- `cargo check --manifest-path citizenchain/node/Cargo.toml`
- 真实桌面运行态验收：打开软件自动启动节点，首页按钮可关闭节点并再次手动启动节点。

## 进度

- [x] 任务卡创建
- [x] 后端手动启停命令完成
- [x] 首页启停按钮完成
- [x] 文档残留清理完成
- [x] 验收通过

## 完成摘要

- 首页状态行调整为“状态点 + 状态文字 + 启动/关闭按钮”，按钮位于状态文字右侧。
- 点击按钮后先打开二次确认弹窗，用户确认后才调用 `start_node` 或 `stop_node`。
- 后端复用原节点生命周期锁和启停流程；打开软件仍自动启动节点，手动启动只在节点停止时启动，自动启动已完成时直接返回当前状态。
- 手动停止节点后清空链状态展示，避免把预期中的 RPC 不可用显示成首页刷新错误。
- 清理了首页生命周期、前端目录和交易目录相关旧文档口径。
- 后续修复：同步守卫不再在 Tauri 进程内自动重启节点，避免 Substrate/RocksDB 释放滞后时触发 `lock hold by current process`。
- 后续修复：节点生命周期状态新增 `lock_held`、`exited` 等可见状态，首页会显示“数据库锁未释放”或“异常退出”，不再把半死状态伪装成普通停止。
- 后续修复：启动失败线程会 join 后返回错误；启动路径对 RocksDB 同进程锁错误只做有限重试，仍失败时给出完全退出软件后重开的明确提示。

## 验收结果

- `npm run build`（`citizenchain/node/frontend`）：通过，保留既有 chunk size warning。
- `cargo fmt --manifest-path citizenchain/node/Cargo.toml --check`：通过。
- `cargo check --manifest-path citizenchain/node/Cargo.toml`：通过。
- `cargo test --manifest-path citizenchain/node/Cargo.toml home::sync_guard -- --nocapture`：通过，6 个同步守卫判定测试全部通过。
- 真实桌面开发版运行：`cargo tauri dev` 打开后自动启动节点，首页状态显示“运行中”，按钮显示在状态文字右侧并显示“关闭”。
- 后续真实运行态验收：开发版自动启动后 PID `55647` 监听 `127.0.0.1:9944`，`system_health` 返回正常；等待超过旧同步守卫自动重启窗口后，审计日志新增 `sync_guard/degraded` 而不是 `restart_attempt`，同一 PID 继续运行且 RPC 正常。
- 停止开发版后复查：`127.0.0.1:9944` 无监听，`/Users/rhett/Library/Application Support/gmb.dev/chains/citizenchain/db/full/LOCK` 无进程占用。
- 自动点击弹窗验证：macOS 辅助功能返回 `-25208`，脚本无法点击 WebView；弹窗逻辑已通过前端源码和构建验证。
