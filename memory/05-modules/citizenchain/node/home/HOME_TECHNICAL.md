# 首页节点管理模块 — 技术文档

## 模块结构

```
home/
├── mod.rs              # 重新导出所有子模块的 pub 接口
├── process/mod.rs      # 进程生命周期管理（自动启动、手动启停、退出清理）
├── rpc/mod.rs          # 节点 RPC 调用（链状态、指纹验证、genesis hash）
├── identity/mod.rs     # 节点身份管理（名称、PeerId、运行状态）
└── sync_guard.rs       # 本机同步守护（检测 P2P 已连但 sync peer 表为空的脱钩状态）
```

前端首页结构与后端保持一致：
- `node/frontend/home/HomeNodeSection.tsx`：首页左侧节点状态、手动启停按钮、链状态、节点身份、发行总额与永久质押展示。
- `node/frontend/home/api.ts` 与 `types.ts`：首页节点面板专用 Tauri API 与类型。
- `node/frontend/home/components/`：链状态、节点身份和发行/质押展示子组件。
- `node/frontend/transaction/onchain-transaction/`：首页右侧交易面板，承载公民钱包、矿工热钱包、转账签名与提交。

## Cargo.toml 构建依赖

核心约束：
- `citizenchain/node/Cargo.toml` 对运行时共享常量 crate `primitives` 的本地路径必须固定指向 `citizenchain/runtime/primitives`
- 如果误指向历史顶层 `primitives/` 目录，`cargo metadata` 会在桌面壳启动前直接失败，开发启动脚本也会停在 Tauri 开发环境拉起之前

当前约束说明：
1. 仓库当前真实目录结构已经统一到 `citizenchain/runtime/primitives`
2. 桌面端启动脚本依赖 `cargo tauri dev` 先成功读取 workspace 与 `node/Cargo.toml`
3. 因此 node 对 `primitives` 的路径引用必须指向当前仓库结构
4. `node/build.rs` 只负责 Substrate cargo keys 与 Tauri 构建入口，不再维护历史 sidecar 复制逻辑
5. Tauri 编译期会校验 `tauri.conf.json` 中的 `frontendDist = "frontend/dist"` 是否存在，因此前端构建产物必须由 `node/frontend` 生成

## 开发脚本语义

当前约定：
1. `citizenchain/scripts/run.sh` 负责“不清库，继续启动开发链”，固定设置 `CITIZENCHAIN_DATA_PROFILE=dev`，使用 `~/Library/Application Support/gmb.dev`，启动时使用当前源码构建 runtime，不从 GitHub CI 下载 wasm 产物
2. `citizenchain/scripts/clean-run.sh` 负责“保留节点身份和密钥、只清开发版区块数据库后重新创世启动”，仅清理 `~/Library/Application Support/gmb.dev/chains/citizenchain/db`，fresh genesis 的 runtime code 来自当前源码，不从 GitHub CI 下载 wasm 产物
3. runtime 正式升级只走链上 `System.set_code`，本地开发启动脚本不承担下载或内置最新 CI wasm 的职责
4. 桌面端数据目录由 `shared/security.rs` 统一解析：正式版默认使用 `~/Library/Application Support/gmb`，开发版默认使用 `~/Library/Application Support/gmb.dev`；节点 `base-path` 直接指向该目录，因此正式版区块库为 `~/Library/Application Support/gmb/chains/citizenchain/db/full`，开发版区块库为 `~/Library/Application Support/gmb.dev/chains/citizenchain/db/full`

WASM CI 版本规则：
- 正式创世前项目自身 runtime 版本固定为 `0`；没有已配置且可连接的正式目标链时，公民控制台「运行 WASM CI」直接停止，不得为临时测试链或空目标生成升级版本。
- 正式创世后的升级构建只能从公民控制台「CitizenChain WASM → 运行 WASM CI」进入：控制台读取充值发币页明确配置的正式目标链 `NODE_WS`，并要求 RPC 实际 genesis hash 与协议升级区保存的 `CHAIN_GENESIS_HASH` 完全相等；未保存正式链指纹时一律视为尚无正式目标链。
- 创世哈希匹配后，源码 `spec_version` 还必须与链上版本严格相等，控制台才把源码及其现有精确测试断言同步提高到 `链上版本 + 1`，随后只提交、推送 runtime 范围并触发 `citizenchain-wasm.yml`；目标链不可达、链指纹不匹配或版本漂移时 fail-closed。
- `citizenchain-wasm.yml` 始终按已提交源码原样编译，不查询链、不连接服务器、不读取 SSH/RPC Secret，也不临时改写版本；从 GitHub 或其他位置手动触发时只做普通源码构建，不提高 `spec_version`。
- CI 会校验控制台升级构建满足“源码版本 = 目标链版本 + 1”，并在任务摘要记录构建用途、目标 genesis hash、升级前/后版本与源码提交 SHA。
- 生成的 `citizenchain-wasm` artifact 只用于显式的开发升级、下载脚本或链上 `System.set_code` 流程；本地启动脚本不下载、不内置该 artifact
- 三端桌面安装包 CI 不再由 WASM CI 自动触发，也不再下载/内置最新 WASM；现有链要使用最新 runtime 仍必须走 runtime 升级
- GitHub workflow 只保留 CI 与正式 Release，不再持有固定服务器清单或批量部署入口。Linux 服务器部署统一从本机 `citizenconsole/` 控制台选择44个权威节点之一，下载当前提交已成功 CI 的 `公民链-Linux-amd.deb`，逐节点写入匹配的身份/GRANDPA 密钥并部署；不会清除 `/opt/citizenchain/data` 区块库。

## 白皮书显示规则

- 桌面端白皮书由 `citizenweb/src/whitepaper.md` 经 `citizenchain/scripts/generate-local-docs.mjs` 内置到前端 bundle，显示样式由 `node/frontend/app/styles/global.css` 的 `.local-doc-shell.doc-whitepaper` 负责
- `citizenweb/src/whitepaper.md` 是白皮书排版结构唯一真源：中文标题与英文标题必须写在同一 Markdown 标题内并用 `<br>` 换行；列表项中文与对应英文必须写在同一个列表项内并用 `<br>` 换行，不能依赖桌面端 tab 或官网二次规整段落结构；白皮书图片仍沿用 `docs/assets/`，由生成脚本在桌面端内置时转成 data URI
- 白皮书正式文档必须统一使用同一套字体变量 `--doc-font`；标题、正文、英文副标题、代码块和表格不得单独指定书法体、衬线体或等宽字体

## process/mod.rs

节点生命周期与 App 进程绑定（2026-04-25 起，三平台 macOS/Windows/Linux 行为统一）：

| 用户操作 | 节点行为 | 触发机制 |
|----------|----------|----------|
| 打开 App | 自动启动节点 | `desktop::run_desktop` setup 后台线程 spawn `start_node_blocking` |
| 首页状态区点击“关闭” | App 继续运行，停止节点 | `home::process::stop_node` → `stop_node_sync` |
| 首页状态区点击“启动” | App 继续运行，手动启动节点 | `home::process::start_node` → `start_node_if_stopped_sync`；若自动启动已完成则直接返回当前运行状态 |
| 设置页点击“更新” | 先停止节点，再安装更新并重启 App | `prepare_desktop_update` 调用 `stop_node_blocking`，随后 Tauri updater 执行 `downloadAndInstall` + `relaunch` |
| 关窗（红 X / Cmd+Q / 菜单 Quit / 系统关闭） | 退出 App + 停止节点 | Tauri 默认行为 → `RunEvent::Exit` → `cleanup_on_exit`（不拦截 `CloseRequested`） |
| macOS 黄色横线 | 窗口最小化，节点继续运行 | macOS 系统原生 minimize，不触发 `CloseRequested`，无需拦截 |

Linux 端备注：UI 模式只用于桌面打包（.deb），服务器场景走 CLI `--clearing-bank` 不进 Tauri。

当前手动启停入口（2026-06-25 起）：
- `start_node` / `stop_node` Tauri command 只供首页状态区按钮调用，不恢复旧密码框、不要求设备开机密码。
- `start_node` 是幂等手动启动：如果 App 打开后的自动启动已经完成，命令直接返回当前运行状态，不重复重启节点。
- `stop_node` 只停止进程内节点，App 继续运行；同步守护在节点停止状态只等待，不会主动拉起手动关闭的节点。

仍保持删除的旧链路：
- `verify_start_unlock_password` / `unlock_password` 形参链路（启停不校验设备密码；密码校验只保留在 `set_grandpa_key` / `set_bootnode_key` / `set_reward_account` 等显式高权操作内）
- 首页密码输入框和旧的禁用 prop 链路；当前只保留状态文字右侧的单个“启动/关闭”按钮，点击后先弹二次确认。

核心职责：
- **进程内运行**：节点不再以子进程 sidecar 方式启动，由 `node_runner::start_node_in_process` 在 Tauri 进程内的后台线程跑 Substrate 服务（`task_manager.future().await`）
- **生命周期绑定**：句柄 `NodeHandle` 持有 `oneshot::Sender<()>`、`JoinHandle<()>` 和后台线程活跃标记，drop 时发 shutdown 信号 → `tokio::select!` 让 `task_manager.future()` 与 shutdown 任一退出 → 显式 `drop(task_manager)` 释放 Backend → `tokio_runtime.shutdown_timeout(30s)` → `JoinHandle::join()`，尽量等待 RocksDB LOCK 释放；启动失败线程会先 join 后返回错误，避免失败路径留下悬空线程。
- **保存即重启**：`set_grandpa_key` / `set_bootnode_key` 仍可在节点运行中调用 `stop_node_blocking` → `start_node_blocking` 让新私钥即时生效，依赖上述真停机制
- **更新前停节点**：设置页“更新”按钮触发 `settings::desktop_update::prepare_desktop_update`，只执行停止节点，不重新启动；安装完成后由 Tauri updater 重启整个 App
- **手动启停**：首页按钮位于“状态: 运行中/已停止”右侧，点击后先弹二次确认；确认后通过 `homeNodeApi.startNode` / `homeNodeApi.stopNode` 调用 Tauri command；按钮执行中禁用，停止态清空链状态展示，不把预期中的 RPC 不可用误报为首页错误
- **串行化**：`NODE_LIFECYCLE_LOCK` 互斥锁保证同一时刻只允许一个启停操作
- **状态可见**：`RuntimeState.node_state` 区分 `starting/running/stopping/restarting/failed/lock_held/exited/stopped`；`current_status` 会检查 `NodeHandle::is_alive()`，线程异常退出时取出旧 handle 并返回 `exited`，启动遇到同进程 RocksDB LOCK 时返回 `lock_held`，前端据此显示“数据库锁未释放”而不是普通“已停止”。
- **锁占用处理**：启动路径对 RocksDB 同进程 LOCK 错误只做有限重试；仍失败时保留 `lock_held` 状态并提示完全退出软件后重新打开，避免按钮反复触发无效启动。

RPC 暴露说明：
- `node_runner` 启动参数固定 `--rpc-methods Unsafe --rpc-cors all`，仅监听本机 `--rpc-port`
- 当前定位：单机家用场景，节点不对外暴露。如后续部署公网，需改 `--rpc-methods Safe` 并限制 CORS

## sync_guard.rs

同步守护用于处理一种本机进程内异常：底层 libp2p 仍有连接，交易/投票可以通过网络广播到其他节点，但 Substrate block sync 侧 `system_peers` 为空，导致本机不继续导入新区块。

设计边界：
- 只访问本机 `127.0.0.1` RPC，不定时请求公网参考节点，也不依赖引导节点可用性。
- 不用区块高度是否增长作为故障条件。CitizenChain 在无交易时不会持续空出块，按高度停滞判定会把正常无交易网络误判成故障。
- 不清链、不删除数据库、不切换 `ws/wss`，也不在 Tauri 进程内自动重启当前 Substrate 服务。原因是 Substrate/RocksDB 释放滞后时，同进程自动重启会触发 `lock hold by current process` 并把节点带入“RPC 已停、DB 锁仍在”的半死状态。

触发条件必须同时满足并持续多次采样：
- `system_health.shouldHavePeers == true`
- `system_health.peers == 0`
- `system_peers` 返回空数组
- `system_health.isSyncing == false`
- `system_unstable_networkState.connectedPeers` 中存在已识别 peer，且 peer 有 `versionString` 与 `latestPingTime`

上述条件刻画的是“raw network 已经连上，sync service 没有把连接纳入 block sync peer 表”的脱钩状态；它不会因为公网没交易、区块高度不增长而触发。

降级流程：
1. 守护线程在 App 启动后常驻，节点刚启动时先等待启动宽限期。
2. 命中脱钩条件达到阈值后进入 `degraded` 状态，并写入 `sync_guard/degraded` 审计日志。
3. 守护线程保持运行，后续采样恢复正常时回到 `healthy`；不再调用节点生命周期重启入口。
4. 10 分钟窗口内只保留降级事件计数，避免重复审计刷屏。

手动停止边界：
- 当 `current_status` 显示节点未运行时，守护线程只进入 `waiting_node` 状态并记录 `node is not running`，不会主动拉起手动关闭的节点。

诊断入口：
- Tauri command：`home::sync_guard::get_sync_guard_status`
- 审计动作：`sync_guard`，状态保留 `degraded`

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

## transaction/mod.rs

核心职责：
- **钱包列表**：公民钱包从 `cold-wallets.json` 读取；矿工热钱包从默认链 `powr` keystore 动态注入并置顶，不写入冷钱包文件。
- **矿工热钱包**：列表名称固定为“矿工热钱包”，不显示删除按钮；若用户尝试重复添加同一矿工地址，后端直接拒绝。
- **余额查询**：按钱包公钥读取 finalized 链上账户余额，供首页交易栏展示。
- **公民钱包签名请求**：构造二维码签名所需 payload，确保普通钱包继续由离线设备签名。
- **矿工热钱包签名提交**：前端要求设备开机密码；后端校验通过后签发进程内一次性令牌，再调用本机 `transaction_submitMinerTransfer` RPC 使用 `powr` 密钥签名 `OnchainTransaction::transfer_with_remark`，备注最多 99 UTF-8 字节。
- **签名提交**：公民钱包接收离线签名结果后提交链上转账；矿工热钱包由节点直接返回交易哈希。

安全约束：
1. `cold-wallets.json` 只持久化普通冷钱包，避免把矿工热钱包误当作可删除用户钱包。
2. `transaction_submitMinerTransfer` RPC 需要一次性令牌；令牌只由 Tauri 命令在设备密码校验通过后生成，直接访问本机 RPC 不能转出矿工余额。
3. 手续费预估复用 runtime `onchain_transaction::calculate_onchain_fee` 口径，前端展示只作为预估，最终以链上扣费为准。
4. 钱包余额、发行总额、永久质押总额等金额展示统一传入 `chain_getFinalizedHead` 对应 block hash 读取；best/latest 只用于链状态和交易进度。
