# FIX-9：集成 K-buckets 实现多跳 DHT 发现

> 状态：远期待排（P3）  
> 日期：2026-04-12  
> 前置：FIX-1~7 已完成，需先观测实际连接稳定性  
> 预估工时：3-5 天  
> 模块：smoldot-pow（Rust）  

---

## 背景

当前 smoldot 的 peer 发现循环是**单跳查询**：每次向 1 个已连接的 gossip peer 发送 Kademlia FindNode，拿到结果直接存入 `BasicPeeringStrategy`，不做迭代查找。

K-buckets 数据结构（`lib/src/network/kademlia/kbuckets.rs`，609 行）已完整实现：
- SHA-256 距离计算
- 256 个桶，每桶可配置容量（典型值 20）
- Connected/Disconnected 状态追踪
- 过期淘汰机制（pending_timeout）
- `closest_entries(target)` 按距离排序查询
- `entry()` / `or_insert()` / `set_state()` 完整 CRUD API

但 `Kademlia` 主结构体是空壳：

```rust
// lib/src/network/kademlia.rs
// TODO: work in progress
// TODO: unused
pub struct Kademlia {}
```

未集成到 `network_service` 的发现循环中。

---

## 实现方案

### 第一步：填充 Kademlia 结构体

**文件**：`lib/src/network/kademlia.rs`

```rust
use crate::libp2p::peer_id::PeerId;
use kbuckets::{KBuckets, PeerState};
use core::time::Duration;

pub struct Kademlia<TNow> {
    /// 本地 DHT 路由表。
    buckets: KBuckets<Vec<u8>, PeerData, TNow, 20>,
}

struct PeerData {
    /// 该 peer 的已知地址列表。
    addresses: Vec<Vec<u8>>,
}
```

提供方法：
- `new(local_peer_id: &PeerId)` — 初始化，local_key = PeerId 的 multihash bytes
- `insert_peer(peer_id, addresses, now, connected)` — 插入/更新 peer
- `set_peer_state(peer_id, connected/disconnected, now)` — 连接状态变更
- `closest_peers(target_peer_id, count) -> Vec<(PeerId, Vec<Multiaddr>)>` — 查询距 target 最近的 N 个 peer

### 第二步：在 BackgroundTask 中持有 Kademlia

**文件**：`light-base/src/network_service.rs`

`BackgroundTask` 新增字段：
```rust
kademlia: Kademlia<TPlat::Instant>,
```

初始化时用轻节点自身的 PeerId 构造。

### 第三步：事件驱动填充 K-buckets

在以下事件处插入/更新 K-buckets：

| 事件 | 位置 | 操作 |
|------|------|------|
| `HandshakeFinished` | network_service.rs:1829 | `insert_peer(peer_id, [addr], now, Connected)` |
| `KademliaFindNode(Ok(nodes))` | network_service.rs:2343 | 对每个 `(peer_id, addrs)` 调用 `insert_peer(peer_id, addrs, now, Disconnected)` |
| `GossipDisconnected` | 现有断连处理 | `set_peer_state(peer_id, Disconnected, now)` |

### 第四步：发现循环改为迭代查找

**文件**：`light-base/src/network_service.rs` 的 `WakeUpReason::StartDiscovery` 分支

当前流程：
1. 生成随机 PeerId
2. 从 gossip_connected_peers 随机选 1 个 peer
3. 发 FindNode(random_peer_id)
4. 收到响应后存入 BasicPeeringStrategy，结束

改为迭代查找：
1. 生成随机 PeerId `target`
2. 从 K-buckets 查询 `closest_peers(target, α)` 取 α=3 个最近 peer
3. 向这 α 个 peer 并行发 FindNode(target)
4. 收到响应后，将新 peer 插入 K-buckets
5. 再从 K-buckets 查询 `closest_peers(target, α)`，如果有比之前更近的 peer，继续查询
6. 直到没有更近的 peer（收敛），或达到最大轮数（3 轮）

需要新增状态机来追踪进行中的迭代查找：
```rust
struct IterativeFind {
    target: PeerId,
    queried: HashSet<PeerId>,
    closest_so_far: Vec<PeerId>,
    pending_requests: usize,
    round: u8,
}
```

### 第五步：定期 bucket 刷新

每 120 秒检查所有 256 个 bucket，对最久未查询的 bucket 生成一个落入该距离范围的随机 PeerId，发起一次迭代查找。保持路由表新鲜度。

---

## 与 BasicPeeringStrategy 的边界

| 职责 | K-buckets | BasicPeeringStrategy |
|------|-----------|---------------------|
| 知道谁在网络上 | ✅ DHT 路由表 | ❌ |
| 决定连谁 | ❌ | ✅ 插槽分配 + 封禁 |
| 存储地址 | ✅ 每 peer 多地址 | ✅ 每 peer 多地址 |
| 发现新 peer | ✅ 迭代查找 | ❌ 被动接收 |

迭代查找发现新 peer 后，同步插入 `BasicPeeringStrategy`（现有逻辑不变），由后者决定是否分配连接插槽。

---

## 风险与注意事项

1. **改动面大**：涉及 `kademlia.rs`（重写）、`network_service.rs`（新增状态机）、所有 peer 事件处理点
2. **并发状态机**：迭代查找需要跨多个 WakeUpReason 维护状态，复杂度高
3. **测试覆盖**：`kbuckets.rs` 现有测试很少（1 个 `nodes_kicked_out` + 3 个 distance 测试），需补充
4. **收益评估**：当前 44 个 WSS bootnode 已全部在 chainspec 中，FIX-1~7 已解决核心稳定性问题。K-buckets 的价值在网络规模扩展到数百节点后才明显

---

## 触发条件

满足以下任一条件时启动 FIX-9：
1. FIX-1~7 部署后，轻节点仍频繁出现连接不稳定
2. 网络节点数量增长到 100+ 且轻节点发现效率明显不足
3. 有明确的产品需求要求轻节点在弱网环境下保持高可用
