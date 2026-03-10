# Home Node 模块技术文档

## 1. 模块位置

- 路径：`nodeui/backend/src/home/home-node/mod.rs`
- 对外命令：
  - `get_node_status`
  - `start_node`
  - `stop_node`
  - `set_node_name`
  - `get_chain_status`
  - `get_node_identity`

## 2. 模块职责

- 管理首页节点生命周期（启动、停止、状态查询）。
- 管理节点身份信息（节点名称、PeerId、角色）。
- 节点名称修改与其他敏感设置统一，要求设备开机密码校验。
- 提供链高度查询。
- 负责节点启动链路中的安全与一致性控制：
  - 节点二进制路径白名单与 SHA-256 完整性校验。
  - 校验通过后二进制落地为 `runtime-secrets/node-bin-*` 临时副本并从副本启动（降低 TOCTOU 风险）。
  - 运行时 node-key 临时文件最小权限写入与清理。
  - 节点进程识别、终止与停止后状态强校验。

## 3. 状态模型

- 运行时状态：
  - `RuntimeState.local_node`: 当前 UI 会话托管的子进程。
  - `RuntimeState.node_key_file`: 当前会话写入的临时 node-key 文件路径。
  - `RuntimeState.node_bin_file`: 当前会话写入的临时 node 二进制副本路径。
- 对外状态：
  - `NodeStatus { running, state, pid }`
  - `ChainStatus { block_height, finalized_height, syncing }`
  - `NodeIdentity { node_name, peer_id, role }`

## 4. 启动流程（`start_node`）

1. 校验开机密码与敏感密钥解锁能力。
2. 清理 `runtime-secrets` 中历史残留临时文件（`node-key-*.tmp`、`node-bin-*`）。
3. 从受信目录 `backend/binaries/citizenchain-node` 定位二进制。
4. 校验二进制 SHA-256：
   - 读取 `backend/binaries/citizenchain-node.sha256`
   - 与实际文件哈希比较，不一致则拒绝启动。
5. 将校验通过的二进制复制到 `runtime-secrets/node-bin-*`，并对副本再次哈希比对。
6. 停止托管子进程并清理当前会话临时文件（node-key + node-bin）。
7. 终止“可信旧节点进程”（见第 6 节）。
8. 启动新节点：
   - 固定 `--rpc-port 9944`
   - 启动前将 `powr` 密钥同步到本地 keystore（不再注入明文 `POWR_MINER_SURI`）
   - 若配置 bootnode key，通过 `--node-key-file` 传入临时文件
   - 若存在 GRANDPA 私钥，追加 `--validator`
   - 节点二进制执行路径使用 `runtime-secrets/node-bin-*` 副本
9. 启动后补齐奖励地址链上绑定，并校验 GRANDPA 生效。
10. 返回最新状态。

## 5. 停止流程（`stop_node`）

1. 终止托管子进程并清理当前会话临时 node-key 文件：
   - Unix 下先向进程组发送 `SIGTERM`，最多等待 2 秒（20 * 100ms）用于优雅退出。
   - 若超时仍未退出，再发送 `SIGKILL` 强制回收。
2. 终止“可信旧节点进程”（见第 6 节）。
3. 重新获取状态：
   - 若仍为运行中，直接返回错误：`停止失败：节点仍在运行（pid=...）`。
   - 否则返回已停止状态。

## 6. 进程识别与终止策略

### 6.1 托管进程

- 优先处理 `RuntimeState.local_node`（本会话直接拉起的进程）。
- 终止策略与“可信旧进程”保持一致：
  - Unix 下先发进程组 `SIGTERM`。
  - 轮询 `try_wait` 最多 2 秒，给节点优雅收敛窗口。
  - 超时后再发进程组 `SIGKILL`，并调用 `child.kill()` 兜底。

### 6.2 可信旧进程

用于处理“历史会话遗留进程”与“非托管但可信”的节点进程。

识别分层：
1. `ps -ww -axo pid,command` 扫描命令行，筛选：
   - 进程命令像节点（`citizenchain-node` 或调试/发布 node 路径特征）。
   - 且包含 `--rpc-port 9944` 或本应用 `node-data` base-path 特征。
2. 若仍无法唯一定位，使用 `lsof :9944` 作为 best-effort 兜底。
3. 若监听 9944 且 RPC 指纹匹配目标链（`ss58Format == 2027`），允许按监听 PID 兜底处理。

终止策略：
- 对命中的 PID 先 `SIGTERM`，轮询存活，超时后 `SIGKILL`。

## 7. 状态判定（`current_status`）

优先级：
1. 托管子进程仍在运行 -> `running=true`。
2. 命中可信监听 PID -> `running=true` 并回传 pid。
3. 否则走 RPC 指纹探测：
   - `system_properties.ss58Format == 2027`
   - `system_name` 非空
   - 满足则视为运行中。

该策略用于降低“仅凭端口连通导致伪阳性”的风险。

## 8. RPC 实现与健壮性

- 本模块通过本地 JSON-RPC（`127.0.0.1:9944`）读取链状态。
- 健壮性控制：
  - connect/read/write timeout
  - 响应最大读取上限（4MB）
  - HTTP 状态行检查（必须 200）
  - 支持 `Transfer-Encoding: chunked` 解码
  - JSON-RPC `error` 字段显式报错

## 9. 安全控制点

- `runtime-secrets` 目录强制权限（Unix `0700`）。
- node-key 临时文件强制权限（Unix `0600`）。
- 启动/停止/退出均清理 node-key 与 node-bin 临时文件，减少异常退出残留风险。
- 节点二进制固定受信目录 + 哈希校验，降低可执行文件被替换风险。

## 10. 依赖关系

- `settings/bootnodes-address`：bootnode 私钥读取、PeerId 角色映射。
- `settings/fee-address`：矿工签名 SURI 与奖励地址绑定补齐。
- `settings/grandpa-address`：GRANDPA 启动前准备与启动后校验。
- `validation`：节点名称合法性校验。
