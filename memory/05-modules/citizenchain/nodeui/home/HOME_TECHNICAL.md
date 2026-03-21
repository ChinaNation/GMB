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

核心职责：
- **进程启停**：`start_node` / `stop_node`，通过 `NODE_LIFECYCLE_LOCK` 串行化
- **二进制校验**：SHA256 哈希验证 + 受信任目录校验（`is_trusted_node_process`）
- **进程检测**：使用 `sysinfo` 库替代 `ps` 外部命令，Linux 优先走 `/proc` 监听端口探测，其他 Unix 再回退到 `lsof`
- **运行时密钥**：启动时临时解密 node-key 注入，停止时清理
- **RPC 端口共享**：节点启动参数、监听进程识别、HTTP/WS RPC 访问都复用同一份本地 RPC 端口来源

关键安全设计：
1. 二进制先复制到运行时目录并在副本上再次验 hash（stage_verified_node_bin）
2. 进程信任判定只使用 `sysinfo::Process::exe()` 返回的结构化可执行路径，不再从拼接后的命令行字符串反推首个 token，避免带空格路径绕过受信任目录校验
3. 运行时密钥文件设置 0600 权限，停止后立即删除
4. `pid_alive` 不仅检测信号返回值，还通过 `sysinfo` 校验进程可执行文件名以 `citizenchain-node` 开头（`starts_with`），防止 PID 复用导致误判或误杀无关进程，同时避免 `contains` 被 `fake-citizenchain-node` 等伪造名称绕过
5. `cleanup_on_exit` 中对 `terminate_trusted_listener_nodes` 的失败不再静默丢弃，而是通过 `eprintln!` 记录错误日志，便于排查退出时的清理异常

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
- **节点名称管理**：`set_node_name`（需设备密码验证），名称经 Unicode NFC 归一化后存储
- **PeerId 获取**：从 RPC `system_localPeerId` 获取
