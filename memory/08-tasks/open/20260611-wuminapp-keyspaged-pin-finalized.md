# 任务卡：wuminapp 反向索引扫描钉死块哈希(二次修正:钉 best)

## 二次修正(2026-06-11,user 复测仍空后 harness 再实证)

钉 finalized 方案有两个缺陷:① 快照取在 ensureSynced 之前,追块窗口内拿旧哈希/null,等于复发原 bug;② harness 实证钉 finalized 的前缀扫描显著慢于钉 best(首轮 2 分钟未应答),真机 WAN 会超时/悬死。同时实证轻节点钉 **best** 秒回且数据正确,与详情读取的原生 chain_storage_values 同口径。

修正:helper 改名 `getKeysPagedAtBest`,顺序改为 ensureSynced → 取快照 → 钉 `bestBlockHash`,哈希缺失抛错(不再假装空列表);4 个调用点同步改名。analyze 0 + test 196/196(串行)。

## 根因(smoldot 同栈复现实证)

48 守卫修复后,广场点击可进详情,但机构详情提案列表依然为空。用 smoldot-pow 同分叉 + 同 chainspec 搭轻节点 harness 对照实验:

- 追块窗口内发起 `state_getKeysPaged`(org/机构两个前缀)→ **全部返回 `[]`,无任何错误**;同刻全节点返回 1 条。
- 同步到链头后发起同样查询 → 全部返回正确数据。

机制(smoldot `light-base/json_rpc_service/background.rs`):legacy `state_getKeysPaged` 不带 `hash` 参数时,在请求入队那一刻钉死 legacy 服务的 `current_best_block`;轻节点启动后追块的窗口期内这是旧块 → 返回**旧状态的空列表**。App 把空当真相:机构详情显示「暂无提案」并写空缓存。广场不受影响(Isar 摘要缓存兜底);点击详情不受影响(走原生 `chain_storage_values`,有 ensureSynced 把关)。

`methods.rs:432`:`state_getKeysPaged` 支持第 4 参 `hash`,显式传哈希直通 `BlockHashKnown` 精确钉块。原生绑定快照已暴露 `finalizedBlockHash`。

## 修复(纯 Dart,统一切换不留双轨)

1. `SmoldotClientManager` 新增 `getKeysPagedFinalized(prefixHex)`:ensureSynced → 取快照 finalizedBlockHash → `state_getKeysPaged(prefix, count, null, finalizedHash)`。
2. 全仓 4 个 `state_getKeysPaged` 调用点全部改走该入口:
   - `duoqian_transfer_service.dart:340`(提案反向索引,本次事主)
   - `institution_discovery_service.dart:112`
   - `institution_manage_service.dart:328`
   - `personal_manage_discovery_service.dart:90`
3. 口径统一:索引扫描从"不确定的旧 best"变为 finalized,与 20260521「确定状态=finalized」死规则一致。

## 验收

- [x] `flutter analyze` 0 issue + `flutter test --concurrency=1` 196/196 全过(并行偶发红为既有 Isar 测试并发问题,单跑/串行全绿)
- [ ] 真机:机构详情提案列表显示提案(user 验证)
- [x] 临时诊断 harness(smoldot-pow/light-base/examples/diag_keyspaged.rs)已删除

## 完工记录(2026-06-11)

- `smoldot_client.dart` 新增 `getKeysPagedFinalized(prefixHex, {count, startKey})`:取快照 finalizedBlockHash 作第 4 参显式钉块。
- 4 个调用点统一切换:duoqian_transfer_service(提案反向索引)/ institution_discovery_service / institution_manage_service / personal_manage_discovery_service,全仓不再有裸 `state_getKeysPaged`。
- 复现实验数据留档:追块窗口内 org/inst 两前缀扫描 + 提案本体读取全部空,同步到头后全部正确;全节点同刻全部有数据。
