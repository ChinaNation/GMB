# smoldot 轻节点稳定性与连接性改进技术方案

> 状态：FIX-1~7 已完成，FIX-8 不需要，FIX-9 远期待排  
> 日期：2026-04-12  
> 模块：smoldotpow（Rust）+ citizenapp/lib/rpc/smoldot_client.dart（Dart）  

---

## 问题描述

citizenapp 内嵌 smoldot 轻节点频繁出现 `No node available for storage query`，链上读操作大面积失败。根因分析如下：

| 层级 | 根因 | 影响 |
|------|------|------|
| Rust·sync_service | 空 peer 列表时零重试，立即返回错误 | 每次读链直接失败 |
| Rust·standalone | peers_assumed_know_blocks 对 PoW 链过滤过严 | 有 peer 却选不出来 |
| Rust·network_service | num_out_slots=4，发现循环只能问已连接节点 | peer 池太小 |
| Rust·kademlia | K-buckets 609 行已实现但未集成，无本地 DHT 路由表 | 发现退化为单跳查询 |
| Dart·smoldot_client | _waitForPeer 未覆盖所有入口；重试次数/间隔不足 | Dart 层容错不够 |
| Dart·smoldot_client | _synced 标志 degraded 后不重置 | 恢复后仍报"未同步" |

---

## 执行结果

| 编号 | 优先级 | 状态 | 改动摘要 |
|------|--------|------|----------|
| FIX-1 | P0 | ✅ 完成 | `sync_service.rs:653` — 空 peer 列表 sleep 2s × 3 次重试（6 秒窗口） |
| FIX-2 | P0 | ✅ 完成 | `standalone.rs:977` — peer 选择三级递进 + 终极兜底返回所有 source |
| FIX-3 | P1 | ✅ 完成 | `lib.rs:2008` — num_out_slots 4→8 |
| FIX-4 | P1 | ✅ 完成 | `network_service.rs:1792` — 发现循环 `.next()` → `.choose(&mut randomness)` 随机选 peer |
| FIX-5 | P1 | ✅ 完成 | `smoldot_client.dart` — 5 个方法补 `_waitForPeer()`，覆盖率 3/8→8/8 |
| FIX-6 | P1 | ✅ 完成 | `smoldot_client.dart:50-51` — 重试 2→4 次，间隔 1→2s，容错窗口 2s→8s |
| FIX-7 | P2 | ✅ 完成 | `smoldot_client.dart:92` — degraded 时重置 `_synced=false` + `_syncFuture=null` |
| FIX-8 | P0 | ⏭️ 不需要 | chainspec 已有 44 个 WSS bootnode，前提条件不成立 |
| FIX-9 | P3 | 📋 远期待排 | 集成 K-buckets 多跳 DHT 发现（见下方独立任务卡） |

---

## FIX-1~7 改动文件清单

**Rust 侧（smoldotpow）：**
- `light-base/src/sync_service.rs` — FIX-1
- `light-base/src/sync_service/standalone.rs` — FIX-2
- `light-base/src/lib.rs` — FIX-3
- `light-base/src/network_service.rs` — FIX-4

**Dart 侧（citizenapp）：**
- `lib/rpc/smoldot_client.dart` — FIX-5 + FIX-6 + FIX-7

全部编译通过，零错误。

---

## 预期效果

| 指标 | 改进前 | 改进后 |
|------|--------|--------|
| peer 列表为空时行为 | 立即失败 | 等待 6 秒重试 3 次 |
| peer 选择策略 | 严格过滤，PoW 链频繁返回空 | 三级递进 + 终极兜底 |
| 出站连接上限 | 4 | 8 |
| 发现循环覆盖 | 固定问第一个 peer | 随机选取 peer |
| Dart 层容错窗口 | 2 秒 | 8 秒 |
| degraded 恢复 | 跳过同步检查 | 强制重新确认同步 |
| _waitForPeer 覆盖率 | 3/8 读写方法 | 8/8 |

---

## FIX-8 原生 SIGABRT 闪退：warp 构建步骤被重置后 panic（2026-08-05）

### 现象

交易签名提交后、**链上最终性确认那一秒** App 整个闪退（非 ANR）。tombstone：

```
pid: <app>, tid: <n>, name: smoldot-light-1  >>> org.citizenapp <<<
signal 6 (SIGABRT), Abort message: 'internal error: entered unreachable code'
```

### 根因

1. 新块最终化 → `lib/src/sync/all.rs` 的 `FinalityProofVerify::perform` 命中
   `NewFinalized` → 调 `warp_sync.set_chain_information(...)`；
2. `lib/src/sync/warp_sync.rs` 的 `set_chain_information` **就地重置** warp 状态机：
   `runtime_download = NotStarted`、`runtime_calls` 换回未下载默认值（其文档自述
   “只重置状态机，不含 fragment 下载与验证队列”），**但不作废已排队的构建步骤**；
3. 随后那个在途的 `BuildRuntime::build` / `BuildChainInformation::build` 执行，发现状态
   不在预期分支 → `unreachable!()` → panic → `panic = "abort"` → 整个进程 SIGABRT。

**为什么本 fork 高频触发**：`all.rs` 配置 `warp_sync_minimum_gap = 0`（本链共识为
`ChainInformationConsensus::Unknown`——smoldot 无 PoW 支持，轻节点无法验证块头，只能靠
GRANDPA warp 证明前进），于是**每来一个更新的 finalized 块就重跑一次 warp**。上游 warp
只在启动跑一次、几乎碰不到这个竞态；本 fork 把它变成了常态。

### 修复（不改 warp 策略、不动 `minimum_gap = 0`、不碰链端）

`lib/src/sync/warp_sync.rs`：3 处 `unreachable!()` → 可恢复错误 `StateReset`

| 位置 | 判据 |
|------|------|
| `BuildRuntime::build` | `runtime_download` 非 `NotVerified` |
| `BuildChainInformation::build` | `body_download` 非预期 / `runtime_download` 非 `Verified` |
| 同上（call proof 循环） | 消费前整体校验，不满足则**把取走的 `runtime_calls` 原样放回**再返回，绝不留半截空 map |

`light-base/src/sync_service/standalone.rs`：`StateReset` 单列分支，只记 Debug 日志，
**不计入 `warp_last_failure`**——它是被新块打断的良性重来，计入会让状态快照把常态误报成故障。

上层既有失败路径已覆盖：不封禁 peer（`StateReset` 不是 `SourceMisbehavior`），
`task.sync` 照常回填，warp 以新 anchor 自然重来。

### 未做与理由

**重置时取消在途请求**（原计划第 2 步）已撤销：`in_progress_requests` 由 `warp_sync` 与
`all.rs` 双方共同记账（`all.rs` 持 RequestId 并在回包时索引），单方面移除会造成记账错位、
制造新缺陷；而 FIX-8 落地后重置打断已是良性路径，收益仅剩“少几次无用下载”，不值当。

### 验证

- `cargo test -p smoldot --lib citizenapp_warp_policy_tests`：3 passed，含新增回归守卫
  `build_steps_after_state_reset_return_recoverable_error_not_panic`；
- 真机：带符号 debug 包连发交易，不再闪退、交易照常确认、链状态照常自动更新。

---

## FIX-9 原生 SIGABRT 闪退（第二处）：交易校验遇链重组后 panic（2026-08-05）

### 现象与定位

FIX-8 之后仍在**交易最终确认那一秒**闪退。这次 APK 带符号，栈直接反解到函数：

```
tid: smoldot-light-3   Abort message: 'internal error: entered unreachable code'
#12/#14  smoldot_light::transactions_service::background_task ...
```

日志同刻可见重组：`inBlock 0x08e1… → retracted → inBlock 0x38fd… → finalized`。

### 根因

`light-base/src/transactions_service.rs` 的校验 future（612 行 `async move` →
`validate_transaction`）：校验对着某块启动时该块被 pin；随后链重组把该块挤掉、pin 被释放；
future 才跑到 `pin_pinned_block_runtime`，拿到 `PinPinnedBlockRuntimeError::BlockNotPinned`
→ 撞 `unreachable!()` → panic → abort。

与 FIX-8 同类：**上游假设"不可能发生"的状态，在这条每块都可能重组的 PoW 链上是常态**。
旁证：紧邻的 `ObsoleteSubscription` 分支本就是正常返回错误、1084 行"块已不在池中"本就是
`continue`——"校验途中环境变了"本属预期，只是漏了 `BlockNotPinned` 这一种。

### 修复（3 处 panic → 可恢复）

| 位置 | 场景 | 改法 |
|------|------|------|
| `validate_transaction` 的 `BlockNotPinned` | 目标块被重组挤掉 | 返回新变体 `ValidationError::BlockObsolete`；上层只记 Debug、交易留池等下个块重新校验（**不重建通道、不判失败**） |
| `Some(Err(_))`（校验被取消） | 理论上不发生 | 清 `validation_in_progress` + `continue` |
| `transaction_user_data_mut(...).unwrap_or_else(\|\| unreachable!())` | 交易处理途中已被移出池 | `if let Some(tx)` 跳过状态更新 |

`transactions_service.rs` 现已零 `unreachable!()`（仅注释里保留历史说明）。

### 验证

真机连发交易：不再闪退、交易正常最终性上链（块 #37）、签名 18 ms、无 ANR。

---

### 配套：原生符号策略（便于下次反解）

`rust/Cargo.toml` 与 `smoldotpow/Cargo.toml` 恒 `strip = false` + `debug = 1`，磁盘产物始终
带符号可反解；APK 侧由 `android/app/build.gradle.kts` 分档——debug 包 `keepDebugSymbols`
保留（约 +55M），release 包仍剥离（体积与安全，2.4 万内部符号不随包外发），线上崩溃用
构建时留档的未剥离 `.so` 反解。
