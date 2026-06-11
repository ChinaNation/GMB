# 任务卡：节点桌面端 dry-run Future 拒绝改用户可懂提示

## 背景(根因已查实,见 2026-06-10 诊断)

桌面端转账/提案转账共用 `citizenchain/node/src/governance/signing.rs` 提交路径：
QR 生成时取 nonce 用 `system_accountNextIndex`(含交易池),提交时 `system_dryRun`
只对链上状态验(不见池)。同账户只要有一笔交易在池中未出块,下一笔必被判
`InvalidTransaction::Future` 拒绝,报错原文是技术细节:

```
交易校验失败，已拒绝提交: InvalidTransaction::Future(nonce 超前，交易会卡在 future 队列永不出块) (hex: 0x010002)
```

用户看不懂,实际含义就是"上一笔还没出块"。

## 需求(user 字面指示)

dry-run 判 Future 时,把抛给前端的报错改成:

```
上一笔交易尚未出块，请稍候再试
```

只加提示,不改守卫行为(仍拒绝提交),不动链端,不动 wumin/wuminapp,
不在本卡处理难度根因(另有 `20260608-pow-difficulty-fork-storm-txpool-desync`)。

## 方案

- `signing.rs` dry-run 拒绝分支:`result_bytes == [0x01,0x00,0x02]`(Future)时
  返回 `Err("上一笔交易尚未出块，请稍候再试")`,技术细节(hex/classify)留在 eprintln 日志;
  其余变体(Stale/BadProof/Payment 等)保持原报错格式不变。
- 补单测:Future 字节序列 → 用户提示文案;其余变体 → 原 `交易校验失败` 格式。

## 验收

- [x] `cargo test -p node` signing 模块 10/10 全过(含新用例 dry_run_reject_future_gives_user_hint / dry_run_reject_other_variants_keep_technical_reason)
- [x] 编译通过(全 crate 153/154,唯一失败 `compact_u128_big_integer` 为先前已存在的测试期望错误,与本卡无关,已另立后台任务)
- [x] 转账/提案转账两路径均生效(共用 signing.rs 同一提交函数,Future 分支唯一出口)

## 落地记录(2026-06-10)

- `signing.rs` 新增 `dry_run_reject_message(result_bytes, raw_hex)`:Future(0x01 0x00 0x02)
  → `上一笔交易尚未出块，请稍候再试`;其余变体保留原 `交易校验失败，已拒绝提交: {reason} (hex: …)` 格式。
- dry-run 拒绝时技术细节(classify + hex)改走 eprintln 日志,前端只收文案。
- 守卫行为不变:仍拒绝提交,不改链端,不动 wumin/wuminapp。
