# smoldot 轻节点稳定性与连接性改进技术方案

> 状态：FIX-1~7 已完成，FIX-8 不需要，FIX-9 远期待排  
> 日期：2026-04-12  
> 模块：smoldot-pow（Rust）+ wuminapp/lib/rpc/smoldot_client.dart（Dart）  

---

## 问题描述

wuminapp 内嵌 smoldot 轻节点频繁出现 `No node available for storage query`，链上读操作大面积失败。根因分析如下：

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

**Rust 侧（smoldot-pow）：**
- `light-base/src/sync_service.rs` — FIX-1
- `light-base/src/sync_service/standalone.rs` — FIX-2
- `light-base/src/lib.rs` — FIX-3
- `light-base/src/network_service.rs` — FIX-4

**Dart 侧（wuminapp）：**
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
