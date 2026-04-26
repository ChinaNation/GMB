# 首页节点管理模块 — 技术文档

## 模块结构

```
home/
├── mod.rs              # 重新导出所有子模块的 pub 接口
├── process/mod.rs      # 进程生命周期管理（启动、停止、检测、二进制校验）
├── rpc/mod.rs          # 节点 RPC 调用（链状态、指纹验证、genesis hash）
├── identity/mod.rs     # 节点身份管理（名称、PeerId、运行状态）
└── HOME_TECHNICAL.md   # 本文档
```

## backend/Cargo.toml 构建依赖

核心约束：
- `citizenchain/nodeui/backend` 对运行时共享常量 crate `primitives` 的本地路径必须固定指向 `citizenchain/runtime/primitives`
- 如果误指向历史顶层 `primitives/` 目录，`cargo metadata` 会在桌面壳启动前直接失败，`clean-dev.sh` 也会停在 Tauri 开发环境拉起之前

当前约束说明：
1. 仓库当前真实目录结构已经统一到 `citizenchain/runtime/primitives`
2. `nodeui` 启动脚本依赖 `cargo tauri dev` 先成功读取 workspace 与 backend manifest
3. 因此 backend 对 `primitives` 的路径引用必须跟随仓库结构同步维护，不能保留历史迁移前的相对路径
4. `backend/build.rs` 在复制内嵌 node sidecar 前必须主动创建 `backend/binaries/`，不能依赖仓库中预先存在该目录
5. sidecar 的带架构文件名应跟随当前 Rust `TARGET` 三元组生成，保持与打包脚本一致
6. Tauri 编译期会校验 `backend/tauri.conf.json` 中的 `frontendDist` 是否存在，因此 `backend/build.rs` 需要在构建前先确保 `frontend/dist/` 目录存在

## 开发脚本语义

当前约定：
1. `citizenchain/scripts/run.sh` 负责“不清库，继续启动开发链”，会保留现有 `node-data`，但仍使用 `dev-chain` feature 拉起 `--chain dev`
2. `citizenchain/scripts/clean-dev.sh` 负责“清库后启动开发链”，会先删除应用数据目录，再使用 `dev-chain` feature 启动全新创世的开发链
3. 两个脚本都属于开发用途，区别只在于是否先删除本地应用数据目录，而不是是否使用开发链

## process/mod.rs

节点生命周期与 App 进程绑定（2026-04-25 起，三平台 macOS/Windows/Linux 行为统一）：

| 用户操作 | 节点行为 | 触发机制 |
|----------|----------|----------|
| 打开 App | 自动启动节点 | `ui::run_desktop` setup 后台线程 spawn `start_node_blocking` |
| 关窗（红 X / Cmd+Q / 菜单 Quit / 系统关闭） | 退出 App + 停止节点 | Tauri 默认行为 → `RunEvent::Exit` → `cleanup_on_exit`（不拦截 `CloseRequested`） |
| macOS 黄色横线 | 窗口最小化，节点继续运行 | macOS 系统原生 minimize，不触发 `CloseRequested`，无需拦截 |

Linux 端备注：UI 模式只用于桌面打包（.deb），服务器场景走 CLI `--clearing-bank` 不进 Tauri。

**已删除**（2026-04-25）：
- `start_node` / `stop_node` Tauri command（前端不再有启停按钮和密码输入框）
- `verify_start_unlock_password` / `unlock_password` 形参链路（启停不再校验设备密码；密码校验只保留在 `set_grandpa_key` / `set_bootnode_key` / `set_reward_wallet` 等显式高权操作内）
- 前端 `HomeNodeSection` 启动/停止 modal 和相关 state、`api.startNode` / `api.stopNode`、`disabled` prop 全链路（启停按钮删后该 prop 失去触发源）

核心职责：
- **进程内运行**：节点不再以子进程 sidecar 方式启动，由 `node_runner::start_node_in_process` 在 Tauri 进程内的后台线程跑 Substrate 服务（`task_manager.future().await`）
- **生命周期绑定**：句柄 `NodeHandle` 持有 `oneshot::Sender<()>` 和 `JoinHandle<()>`，drop 时发 shutdown 信号 → `tokio::select!` 让 `task_manager.future()` 与 shutdown 任一退出 → 显式 `drop(task_manager)` 释放 Backend → `tokio_runtime.shutdown_timeout(10s)` → `JoinHandle::join()`，**确保 RocksDB LOCK 真正释放**（修复了之前 drop `JoinHandle` 不停线程导致的 `lock hold by current process` bug）
- **保存即重启**：`set_grandpa_key` / `set_bootnode_key` 仍可在节点运行中调用 `stop_node_blocking` → `start_node_blocking` 让新私钥即时生效，依赖上述真停机制
- **串行化**：`NODE_LIFECYCLE_LOCK` 互斥锁保证同一时刻只允许一个启停操作
- **状态可见**：`current_status` 单纯读 `state.node_handle.is_some()`，前端通过 `get_node_status` 每 3s 轮询自然刷新；启停期间为避免阻塞 `get_node_status`，`take` handle 后立即释放 state 锁再 drop

RPC 暴露说明：
- `node_runner` 启动参数固定 `--rpc-methods Unsafe --rpc-cors all`，仅监听本机 `--rpc-port`
- 当前定位：单机家用场景，节点不对外暴露。如后续部署公网，需改 `--rpc-methods Safe` 并限制 CORS

## rpc/mod.rs

核心职责：
- **RPC 指纹校验**：`ss58Format == 2027` + `system_name` 非空 + genesis hash 一致性
- **genesis hash 缓存**：首次连接缓存，后续比对（`shared::rpc::verify_genesis_hash`）
- **链同步状态**：区块高度、最终确认高度、同步标志

关键安全设计：
1. genesis hash 只有在格式满足 `0x` + 64 位十六进制时才允许写入缓存
2. 后续每次校验都会先做同样的格式验证，再与首次缓存比较，避免恶意节点用任意非空字符串污染缓存
3. 本地 RPC HTTP/WS URL 会跟随共享 RPC 端口变化，避免部分模块仍然访问旧的硬编码端口

## identity/mod.rs

核心职责：
- **节点状态查询**：`current_status`（PID、运行标志、PeerId、节点名）
- **节点身份读取**：`get_node_identity`（节点名 + PeerId 一次性返回，供前端展示）
- **PeerId 获取**：从 RPC `system_localPeerId` 获取
