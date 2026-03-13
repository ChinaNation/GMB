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

## process/mod.rs

核心职责：
- **进程启停**：`start_node` / `stop_node`，通过 `NODE_LIFECYCLE_LOCK` 串行化
- **二进制校验**：SHA256 哈希验证 + 受信任目录校验（`is_trusted_node_process`）
- **进程检测**：使用 `sysinfo` 库替代 `ps` 外部命令，`lsof` 保留用于端口监听检测
- **运行时密钥**：启动时临时解密 node-key 注入，停止时清理

关键安全设计：
1. 二进制先复制到运行时目录并在副本上再次验 hash（stage_verified_node_bin）
2. 命令行可执行文件必须位于受信任目录（binaries/ 或 app resource dir）
3. 运行时密钥文件设置 0600 权限，停止后立即删除

## rpc/mod.rs

核心职责：
- **RPC 指纹校验**：`ss58Format == 2027` + `system_name` 非空 + genesis hash 一致性
- **genesis hash 缓存**：首次连接缓存，后续比对（`shared::rpc::verify_genesis_hash`）
- **链同步状态**：区块高度、最终确认高度、同步标志

## identity/mod.rs

核心职责：
- **节点状态查询**：`current_status`（PID、运行标志、PeerId、节点名）
- **节点名称管理**：`set_node_name`（需设备密码验证）
- **PeerId 获取**：从 RPC `system_localPeerId` 获取
