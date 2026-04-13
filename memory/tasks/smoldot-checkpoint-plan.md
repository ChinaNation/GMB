# smoldot 轻节点冷启动优化技术方案

> 状态：CP-1~5 全部完成  
> 日期：2026-04-12  
> 模块：wuminapp（Dart）+ scripts（Shell）+ memory（文档）+ citizenchain/node（前端文案）  
> 前置依赖：无（与 smoldot-stability-plan 的 FIX-1~7 互补，该方案解决"peer 连接层"，本方案解决"同步起点层"）

---

## 问题链路

```
chainspec.json 无 lightSyncState
    ↓
smoldot 从 genesis (#0) 冷启动，逐块同步区块头
    ↓
3 分钟超时（链已运行数千块）
    ↓
_waitForSync 抛异常 → healthStatus = degraded
    ↓
_saveDatabaseCache 不执行（只在成功后保存）
    ↓
下次重启 → 又从 genesis 冷启动 → 恶性循环
```

核心矛盾：全节点已有 `sync_state_genSyncSpec` RPC 能产出 checkpoint，但 App 从未获取过；chainspec 冻结规则把 `lightSyncState` 和 genesis 字段一起冻死了。

---

## 修复清单（5 项）

### CP-1：App 启动时从全节点拉取 lightSyncState 注入 chainspec（Dart·P0）

**文件**：`wuminapp/lib/rpc/smoldot_client.dart`

**现状**：`initialize()` 从 assets 加载 chainspec.json → 注入 localhost bootnode → 直接传给 smoldot。chainspec 无 lightSyncState → genesis 冷启动。

**改法**：在 `_injectLocalhostBootnode` 之后、`_addChain` 之前，新增 `_injectLightSyncState` 步骤：

1. 从 bootNodes 列表中提取 WSS 全节点地址（取前 3 个）
2. 依次尝试通过 HTTP RPC 调用 `sync_state_genSyncSpec`（超时 10 秒/个）
3. 从响应中提取 `lightSyncState` 字段
4. 注入到内存中的 chainspec JSON
5. 成功后缓存到 SharedPreferences（key: `smoldot_light_sync_state`），下次启动优先用缓存
6. 全部失败则跳过（降级为 genesis 冷启动 + database 恢复，保持现有行为）

```dart
/// 从全节点拉取 lightSyncState checkpoint 并注入 chainspec。
///
/// smoldot 拿到 checkpoint 后直接从 finalized block 开始同步，
/// 跳过 genesis 到 finalized 之间的全部区块头验证，冷启动从分钟级降到秒级。
///
/// 优先使用缓存（避免每次启动都请求全节点），缓存失效时重新拉取。
/// 全部失败时静默降级，不阻塞启动流程。
Future<String> _injectLightSyncState(String chainSpecJson) async {
  try {
    // 1. 优先读缓存
    final cached = await _loadCachedLightSyncState();
    if (cached != null) {
      final spec = jsonDecode(chainSpecJson) as Map<String, dynamic>;
      spec['lightSyncState'] = jsonDecode(cached);
      debugPrint('[Smoldot] 使用缓存的 lightSyncState checkpoint');
      return jsonEncode(spec);
    }

    // 2. 缓存未命中，从全节点拉取
    final spec = jsonDecode(chainSpecJson) as Map<String, dynamic>;
    final bootNodes = (spec['bootNodes'] as List?)?.cast<String>() ?? [];
    
    // 从 bootNodes 提取 WSS RPC 地址（dns4 域名 + wss 端口）
    final rpcUrls = _extractRpcUrls(bootNodes, limit: 3);
    
    for (final url in rpcUrls) {
      try {
        final response = await _fetchSyncSpec(url, timeout: const Duration(seconds: 10));
        if (response != null && response['lightSyncState'] != null) {
          final lss = response['lightSyncState'];
          spec['lightSyncState'] = lss;
          // 缓存供下次使用
          await _saveCachedLightSyncState(jsonEncode(lss));
          debugPrint('[Smoldot] 已从 $url 获取 lightSyncState checkpoint');
          return jsonEncode(spec);
        }
      } catch (e) {
        debugPrint('[Smoldot] 从 $url 获取 lightSyncState 失败: $e');
      }
    }
    
    debugPrint('[Smoldot] 未获取到 lightSyncState，降级为 genesis 冷启动');
    return chainSpecJson;
  } catch (e) {
    debugPrint('[Smoldot] _injectLightSyncState 异常: $e');
    return chainSpecJson;
  }
}
```

**RPC 地址提取逻辑**：bootNodes 格式为 `/dns4/<domain>/tcp/30333/wss/p2p/<peer_id>`。全节点的 HTTP RPC 端口通常是 `https://<domain>:9944`（或同域名的 HTTPS 端口）。需要一个映射规则或配置项。

**方案 A（推荐）**：App 内置一个 RPC endpoint 列表常量（如 `https://nrcgch.crcfrcn.com:9944`），不依赖 bootNodes 解析。全节点已开放 RPC，只需确认端口。

**方案 B**：从 bootNodes 的 dns4 域名推导 RPC URL（`https://<domain>:9944`），约定所有全节点的 HTTP RPC 都在 9944 端口。

**缓存策略**：
- 缓存 key：`smoldot_light_sync_state`
- 缓存有效期：24 小时（`lightSyncState` 中的 finalized block 高度落后太多时 smoldot 会自动从该点追赶，不影响正确性，只影响追赶长度）
- 缓存过期后下次启动重新拉取

**影响评估**：

| 维度 | 说明 |
|------|------|
| 冷启动耗时 | 从"同步全部区块头（分钟级）"降到"从 finalized 追赶几个块（秒级）" |
| 网络请求 | 启动时增加 1 次 HTTP RPC（有缓存时 0 次） |
| 失败降级 | 静默降级为现有 genesis 冷启动，不阻塞 |
| 安全性 | smoldot 会校验 lightSyncState 中 GRANDPA authority set 的合法性，伪造的 checkpoint 无法通过验证 |

---

### CP-2：冻结规则排除 lightSyncState（Shell + 文档·P0）

**文件**：
- `scripts/check-chainspec-frozen.sh:26`
- `wuminapp/scripts/wuminapp-run.sh:49`
- `memory/07-ai/chainspec-frozen.md:48,80`

**现状**：`jq -cS 'del(.bootNodes)'` 只排除了 `bootNodes`。如果 chainspec.json 新增 `lightSyncState` 字段，sha256 会变 → pre-commit hook / CI / 启动脚本全部拦截。

**但 CP-1 方案是内存注入，不修改文件**，所以 chainspec.json 文件本身不会变。此项改动是**预防性**的——确保未来如果需要在 chainspec.json 文件中预置 lightSyncState（比如 CI 构建时注入），冻结规则不会拦截。

**改法**：

`scripts/check-chainspec-frozen.sh:26`：
```bash
# 旧
ACTUAL="$(jq -cS 'del(.bootNodes)' "$CHAINSPEC" | shasum -a 256 | awk '{print $1}')"
# 新
ACTUAL="$(jq -cS 'del(.bootNodes, .lightSyncState)' "$CHAINSPEC" | shasum -a 256 | awk '{print $1}')"
```

`wuminapp/scripts/wuminapp-run.sh:49`：
```bash
# 旧
ACTUAL_SHA="$(jq -cS 'del(.bootNodes)' "$CHAINSPEC_OUT" | shasum -a 256 | awk '{print $1}')"
# 新
ACTUAL_SHA="$(jq -cS 'del(.bootNodes, .lightSyncState)' "$CHAINSPEC_OUT" | shasum -a 256 | awk '{print $1}')"
```

`memory/07-ai/chainspec-frozen.md` 更新：
- 第 48 行：注明 sha256 排除 `bootNodes` 和 `lightSyncState`
- 第 80 行"正确的事"新增：`✅ 更新 chainspec.json 中的 lightSyncState 字段（checkpoint 不参与 genesis hash）`

**理由**：`lightSyncState` 和 `bootNodes` 性质相同——不参与 genesis hash 计算，不影响链的身份标识，是纯粹的客户端辅助信息。冻结它没有安全价值，反而卡死轻节点的同步能力。

---

### CP-3：同步超时时也保存 database 部分进度（Dart·P1）

**文件**：`wuminapp/lib/rpc/smoldot_client.dart:449-466`

**现状**：`_waitForSync` 的 `_saveDatabaseCache()` 只在成功分支（line 459）调用。超时走 catch → 不保存 → 下次又从零开始。

**改法**：在 catch 分支中也尝试保存 database（smoldot 的 database 包含已同步的区块头进度和已知 peer 列表，即使没完整同步也有价值）：

```dart
Future<void> _waitForSync(Duration timeout) async {
  debugPrint('[Smoldot] 等待轻节点同步完成...');
  try {
    await _chain!.waitUntilSynced(timeout: timeout);
    _synced = true;
    _healthStatus = ChainHealthStatus.operational;
    _lastError = null;
    debugPrint('[Smoldot] 区块头同步完成');
    unawaited(_saveDatabaseCache());
  } catch (e) {
    _healthStatus = ChainHealthStatus.degraded;
    _lastError = '轻节点同步失败: $e';
    debugPrint('[Smoldot] $_lastError');
    // 即使同步超时，也保存已有的部分进度（已知 peer + 已同步区块头），
    // 下次启动恢复后从断点继续，而不是从 genesis 重来。
    unawaited(_saveDatabaseCache());
    rethrow;
  }
}
```

**影响评估**：

| 维度 | 说明 |
|------|------|
| 恢复效果 | 假设第一次冷启动同步到 #500（超时），下次从 #500 继续而非从 #0 |
| 累积效应 | 多次启动后 database 逐步追上最新，最终某次启动能在 3 分钟内完成 |
| 风险 | 无——database 内容由 smoldot 导出，格式由 smoldot 自身校验 |

---

### CP-4：同步超时后不完全放弃，允许后台继续追赶（Dart·P1）

**文件**：`wuminapp/lib/rpc/smoldot_client.dart:449-466`

**现状**：`_waitForSync` 超时后 rethrow → `ensureSynced` 中 `_syncFuture = null`（FIX-7 新增了 `_synced = false`）。下次任何读操作调用 `ensureSynced()` 时会重新发起同步等待——但同一个 smoldot 链实例在后台仍在持续同步，只是 `waitUntilSynced` 的 Dart Future 超时了。

**改法**：超时后不把 `_healthStatus` 直接设为 `degraded`，而是保持 `syncing`。新增定时重试机制：

```dart
Future<void> _waitForSync(Duration timeout) async {
  debugPrint('[Smoldot] 等待轻节点同步完成...');
  try {
    await _chain!.waitUntilSynced(timeout: timeout);
    _synced = true;
    _healthStatus = ChainHealthStatus.operational;
    _lastError = null;
    debugPrint('[Smoldot] 区块头同步完成');
    unawaited(_saveDatabaseCache());
  } catch (e) {
    // 同步超时不等于链不可用——smoldot 后台仍在追赶。
    // 保持 syncing 状态，保存部分进度，后台定时重试。
    _healthStatus = ChainHealthStatus.syncing;
    _lastError = '轻节点同步中，尚未追上最新区块: $e';
    debugPrint('[Smoldot] $_lastError');
    unawaited(_saveDatabaseCache());
    // 后台 60 秒后自动重试同步检查
    unawaited(_scheduleRetrySync());
    rethrow;
  }
}

/// 后台定时重试同步检查（最多 5 次，间隔 60 秒）。
Future<void> _scheduleRetrySync() async {
  for (var i = 0; i < 5; i++) {
    await Future<void>.delayed(const Duration(seconds: 60));
    if (_synced || !isReady) return;
    try {
      await _chain!.waitUntilSynced(timeout: const Duration(seconds: 30));
      _synced = true;
      _healthStatus = ChainHealthStatus.operational;
      _lastError = null;
      _syncFuture = null;
      debugPrint('[Smoldot] 后台重试同步成功 (第 ${i + 1} 次)');
      unawaited(_saveDatabaseCache());
      return;
    } catch (e) {
      debugPrint('[Smoldot] 后台重试同步未完成 (第 ${i + 1}/5 次): $e');
      unawaited(_saveDatabaseCache());
    }
  }
  // 5 次都没成功，标记 degraded
  if (!_synced) {
    _healthStatus = ChainHealthStatus.degraded;
    _lastError = '轻节点长时间未能同步到最新区块';
    debugPrint('[Smoldot] $_lastError');
  }
}
```

**影响评估**：

| 维度 | 说明 |
|------|------|
| 用户体感 | 从"3 分钟后直接报错不可用"变为"显示同步中，后台持续追赶，追上后自动恢复" |
| 最坏情况 | 5 分钟×5 次都没追上才标记 degraded（此时链确实有问题或网络确实不通） |
| 累积效应 | 每 60 秒保存一次 database，即使本次没追上，下次启动离目标更近 |

---

### CP-5：桌面端 NodeKeySection 文案修正（前端·P2）

**文件**：`citizenchain/node/frontend/settings/node-key/NodeKeySection.tsx`

**现状**：

| 行号 | 当前文案 | 实际功能 |
|------|----------|----------|
| 54 | `区块链引导节点` | 绑定本机节点身份（使本机以某个 bootnode 的 PeerId 运行） |
| 62 | `请输入区块链引导节点私钥` | 输入的是 ed25519 node identity key |
| 70 | `请输入区块链引导节点私钥` | 同上（错误提示） |
| 80 | `上传私钥` | 写入 `node-key/secret_ed25519` |

**改法**：

| 行号 | 新文案 |
|------|--------|
| 54 | `节点身份密钥` |
| 62 | `请输入节点身份密钥（Ed25519 私钥 hex）` |
| 70 | `请输入节点身份密钥` |
| 80 | `绑定身份` |

同时更新 `<span>` 中的状态文案：`未绑定` → `未绑定`（不变），绑定后显示的机构名保持不变。

---

## 优先级与执行顺序

| 优先级 | 编号 | 层级 | 预估工时 | 依赖 |
|--------|------|------|----------|------|
| **P0** | CP-1 | Dart | 2h | 需确认全节点 RPC 端口 |
| **P0** | CP-2 | Shell+文档 | 15min | 无 |
| **P1** | CP-3 | Dart | 15min | 无 |
| **P1** | CP-4 | Dart | 1h | CP-3 完成后 |
| **P2** | CP-5 | TSX | 15min | 无 |

**执行顺序**：CP-2（先解冻规则）→ CP-1（注入 checkpoint）→ CP-3（超时保存进度）→ CP-4（后台重试）→ CP-5（文案修正）

---

## 预期效果

| 指标 | 改进前 | 改进后 |
|------|--------|--------|
| 首次冷启动同步起点 | genesis #0 | finalized block（当前最新） |
| 冷启动同步耗时 | 数分钟（追赶全部区块头） | 秒级（追赶几个块） |
| 超时后恢复路径 | 永远从 #0 重来 | 从上次断点继续 |
| 同步超时用户体感 | "区块链不可用" | "同步中，后台追赶" |
| 冻结规则对 lightSyncState | 错杀 | 放行 |
| 桌面端节点密钥文案 | "引导节点私钥"（误导） | "节点身份密钥"（准确） |

---

## 与 smoldot-stability-plan 的关系

| 方案 | 解决层面 | 核心问题 |
|------|----------|----------|
| smoldot-stability-plan（FIX-1~7） | peer 连接层 | 连上 peer 后查询失败 |
| 本方案（CP-1~5） | 同步起点层 | 同步太慢导致超时 |

两套方案互补：FIX-1~7 确保 peer 连接稳定，CP-1~5 确保有了 peer 之后能快速同步到最新块。单独实施任一套都不能完全解决"轻节点总是连不上链"的问题。
