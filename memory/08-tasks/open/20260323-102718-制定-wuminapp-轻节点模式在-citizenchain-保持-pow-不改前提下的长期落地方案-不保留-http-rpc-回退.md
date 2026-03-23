# 任务卡：制定 wuminapp 轻节点模式在 citizenchain 保持 PoW 不改前提下的长期落地方案，不保留 HTTP RPC 回退

- 任务编号：20260323-102718
- 状态：open
- 所属模块：wuminapp
- 当前负责人：Codex
- 创建时间：2026-03-23 10:27:18

## 任务需求

制定 wuminapp 轻节点模式在 citizenchain 保持 PoW 不改前提下的长期落地方案，不保留 HTTP RPC 回退

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- wuminapp/WUMINAPP_TECHNICAL.md

## 模块模板

- 模板来源：memory/08-tasks/templates/wuminapp.md

### 默认改动范围

- `wuminapp`

### 先沟通条件

- 修改 Isar 数据结构
- 修改认证流程
- 修改关键交互路径


## 模块执行清单

- 清单来源：memory/07-ai/module-checklists/wuminapp.md

# WuMinApp 模块执行清单

- App 只是交互入口，不承担信任根职责
- Isar 结构、认证流程、关键交互变化前必须先沟通
- 关键 Flutter 交互与本地存储逻辑必须补中文注释
- 文档与残留必须一起收口


## 模块级完成标准

- 标准来源：memory/07-ai/module-definition-of-done/wuminapp.md

# WuMinApp 完成标准

- App 仍然只是交互入口
- 关键 Flutter 交互和 Isar 逻辑已补中文注释
- 文档已同步更新
- 关键交互或数据结构变化已先沟通
- 残留已清理


## 必须遵守

- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通
- 改代码后必须更新文档和清理残留

## 输出物

- 代码
- 中文注释
- 文档更新
- 残留清理

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
- 已补读启动协议要求的项目目标、信任边界、仓库映射、安全规则、Agent 规则、聊天协议、需求分析模板、多线程模型
- 已确认长期约束：`wuminapp = 轻节点模式`，`citizenchain = PoW + GRANDPA`，不保留 HTTP RPC 回退
- 已形成正式落地文档：
  - `memory/04-decisions/ADR-004-pow-light-client-without-http-fallback.md`
  - `memory/05-modules/wuminapp/rpc/POW_LIGHT_CLIENT_ROADMAP.md`
- 已补充文档边界说明：
  - `memory/04-decisions/ADR-002-smoldot-light-client.md` 增加长期方案指向
  - `memory/05-modules/wuminapp/rpc/RPC_TECHNICAL.md` 增加“当前实现态 / 目标架构”区分

## 当前结论

- `wuminapp` 不适合继续把 `smoldot` 当作 App 内嵌 legacy JSON-RPC 服务器使用
- 长期方案应转为“自有 GitHub fork + 主仓库固定快照 + Rust typed capability + Flutter 只走轻节点能力层”
- 根目录临时 `smoldot/` 必须后续迁入 `wuminapp` 模块内治理，不能继续以未跟踪嵌套仓库形态存在

## 当前进展

- 已完成第 1 步的仓库治理落地：
  - 根目录临时 `smoldot/` 已收编到 `wuminapp/third_party/smoldot-pow/`
  - 收编目录已剔除嵌套 `.git` 与 `target/` 编译残留
  - `wuminapp/rust/Cargo.toml` 已改为引用 `../third_party/smoldot-pow/light-base`
  - 已新增 `wuminapp/third_party/smoldot-pow/UPSTREAM.md` 记录基线提交与 PoW 改动文件
- 已继续推进 Dart 侧治理：
  - pub.dev `smoldot` 包已收编到 `wuminapp/third_party/smoldot-dart/`
  - `wuminapp/pubspec.yaml` 已改为 path 依赖本地 `smoldot` package
  - 已新增 `wuminapp/third_party/smoldot-dart/UPSTREAM.md` 记录来源与后续维护规则
- 已完成 typed capability 第一版脚手架：
  - 本地 `smoldot` Dart fork 已新增 `LightClientStatusSnapshot`
  - `Chain.getStatusSnapshot()` 已把 peer / syncing / best / finalized 状态收口为结构化对象
  - `SmoldotClientManager.getStatusSnapshot()` 已对上层暴露该能力
  - 当前仍属于“typed API 外壳 + 现有 JSON-RPC 底层实现”阶段，尚未进入 Rust 原生 capability
- 已完成第一批 Rust 原生 capability：
  - `wuminapp/rust/src/lib.rs` 已新增 `smoldot_get_status_snapshot`
  - `wuminapp/rust/src/lib.rs` 已新增 `smoldot_get_system_account`
  - `wuminapp/rust/src/lib.rs` 已新增 `smoldot_get_storage_value`
  - `wuminapp/rust/src/lib.rs` 已新增 `smoldot_get_storage_values`
  - `wuminapp/rust/src/lib.rs` 已新增 `smoldot_get_runtime_version`
  - `wuminapp/rust/src/lib.rs` 已新增 `smoldot_get_metadata`
  - `wuminapp/rust/src/lib.rs` 已新增 `smoldot_get_account_next_index`
  - `wuminapp/rust/src/lib.rs` 已新增 `smoldot_get_block_hash`
  - `wuminapp/rust/src/lib.rs` 已新增 `smoldot_get_block_extrinsics`
  - `wuminapp/rust/src/lib.rs` 已新增 `smoldot_submit_extrinsic`
  - Rust 内部已建立 JSON-RPC 响应转发与 native request 分流，避免原生能力与旧 Dart polling 直接抢同一 response stream
  - 本地 `smoldot` Dart fork 已新增对应 bindings 与 typed model
  - `SmoldotClientManager` 已增加 `getSystemAccountSnapshot()`
- 已开始迁移业务主路径：
  - `ChainRpc.fetchBalance()` 的轻节点分支已切到原生 `System.Account`
  - `ChainRpc.fetchConfirmedNonce()` 的轻节点分支已切到原生 `System.Account`
  - `ChainRpc.fetchStorage()` / `fetchStorageBatch()` 的轻节点分支已切到原生 storage proof 读取
  - `ChainRpc.fetchRuntimeVersion()` / `fetchMetadata()` 的轻节点分支已切到原生 capability
  - `ChainRpc.fetchLatestBlock()` 的轻节点分支已改为直接消费状态快照
  - `ChainRpc.fetchNonce()` / `fetchGenesisHash()` / `fetchBlockExtrinsicHashes()` / `submitExtrinsic()` 的轻节点分支已切到原生 capability
  - 钱包余额刷新后续无需再依赖 Dart 层手拼 `state_getStorage`
- 已继续治理轻节点状态读取：
  - `wuminapp/third_party/smoldot-pow/light-base/src/lib.rs` 已新增直接读取 `sync_service/runtime_service` 的 `chain_status_snapshot`
  - `smoldot_get_status_snapshot` 已不再通过 `system_health` legacy JSON-RPC 组装
  - `wuminapp/third_party/smoldot-dart/lib/src/chain.dart` 的 `getInfo()` / `getPeerCount()` / `getStatus()` / `getBestBlock*()` 已统一改走原生 status snapshot
- 已继续治理轻节点链数据读取：
  - `smoldot_get_runtime_version` / `smoldot_get_metadata` 已改为 runtime service / runtime call 主路径
  - `smoldot_get_account_next_index` 已改为 `AccountNonceApi_account_nonce` runtime call
  - `smoldot_get_block_hash` 已改为“最近块缓存 + 当前同步视图”双层原生路径，不再保留 `chain_getBlockHash`
  - `smoldot_get_block_extrinsics` 已改为只走按 block hash 下载 block body 的轻节点原生路径，不再保留 `chain_getBlock` fallback
  - `smoldot_get_storage_value` / `smoldot_get_storage_values` / `smoldot_get_system_account` 已改为只走 `sync_service.storage_query` proof 读取，不再保留 `state_getStorage` fallback
- 已继续清理 HTTP 回退入口：
  - `wuminapp/lib/rpc/chain_rpc.dart` 已改为纯 smoldot 单通道，不再保留 `HttpProvider` / `currentNodeUrl` / `WUMINAPP_RPC_URL`
  - `wuminapp/lib/rpc/chain_event_subscription.dart` 已移除 WebSocket 回退，只保留轻节点新区块订阅
  - `wuminapp/lib/main.dart` 已固定启动 smoldot 初始化，不再读取 `WUMINAPP_RPC_URL`
  - `wuminapp/scripts/app-run.sh` / `app-clean-run.sh` 已移除 HTTP RPC dart-define 透传

## 当前验证

- `cargo check --manifest-path wuminapp/rust/Cargo.toml` 已通过
- `cargo build --manifest-path wuminapp/rust/Cargo.toml` 已通过，`wuminapp/native/smoldot.h` 已包含新增导出
- `flutter analyze lib/rpc/smoldot_client.dart` 已通过
- `flutter analyze lib/rpc/chain_rpc.dart lib/rpc/smoldot_client.dart` 已通过
- `flutter analyze lib/main.dart lib/rpc/chain_rpc.dart lib/rpc/chain_event_subscription.dart lib/governance/all_proposals_view.dart lib/governance/transfer_proposal_service.dart lib/governance/runtime_upgrade_service.dart` 已通过，只有 `main.dart` 既有 3 条 info lint
- `flutter analyze lib/governance/all_proposals_view.dart lib/governance/proposal_cache.dart lib/rpc/smoldot_client.dart` 已通过
- `dart analyze lib` 在 `wuminapp/third_party/smoldot-dart` 中无新增 error，仅有上游历史 info lint
- 本地轻量探针已验证：
  - `status`、`runtimeVersion`、`metadata`、`System.Account`、单个 storage、批量 storage、`accountNextIndex`、`genesisHash`、`block_extrinsics` 都能在 smoldot 路径正常返回
  - 对拍本地全节点后，`state_getRuntimeVersion` 的 `specVersion=2`、`transactionVersion=1` 与轻节点一致
  - `state_getMetadata` 返回前缀 `0x6d6574610e5d0300`，与轻节点返回一致
  - Alice 在当前 dev 链上 `System.Account` 不存在，轻节点返回 `exists=false` 且全节点 `state_getStorage` 同样返回 `null`
  - 同一份探针现已不再触发 `system_health` / `state_getRuntimeVersion` / `chain_getBlockHash` / `chain_getBlock` / `state_getStorage` 的 legacy warning
  - 在移除 `smoldot_get_storage_value` / `smoldot_get_storage_values` / `smoldot_get_system_account` 的 `state_getStorage` 兜底后，同一份探针仍能稳定返回账户与 storage 结果
  - 当前链上读取主路径已不再保留 legacy fallback；剩余工作主要是带真实 peer 的真机验证与发布封板
