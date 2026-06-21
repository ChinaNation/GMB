# 任务卡：citizenapp 转账提案批量解码 48 字节旧主体残留守卫

## 根因(链上字节级验证)

广场能显示提案、点击进不去详情、机构详情列表不显示——finalized 已推进到提案所在块后依然复现。链上 `ProposalData[0]` 实测 132 字节(7 tag + **32 机构** + 32 收款 + 16 金额 + 13 备注 + 32 提案人),而 `duoqian_transfer_service.dart:934` 批量解码 `_decodeProposalData` 的最小长度守卫仍是旧 48 字节主体时代的:

```dart
if (data.length < tag.length + 48 + 32 + 16 + 1 + 32) return null;  // 要求 ≥136
```

132 < 136 → 返回 null(走在 catch 日志之前,零日志)→ businessDetails 为空 → 广场点击 `matches()` 不命中落进"详情页面正在开发中"兜底;机构详情列表丢弃无业务详情的提案。

- 同文件 `fetchProposalAction`(:995)单查路径在 2026-06-07 机构 48B→32B 整改(84080b6a)时已改对,批量路径漏改。
- **只在备注 < 16 字节时发作**(备注够长则总长 ≥136 侥幸通过),所以此前未暴露;本次备注「转账测试」12 字节踩中。
- 全仓扫描:此 `48` 残留仅此一处(chain_tx_monitor:608 的 48 = 32+16 合法组合;citizenwallet 无残留)。

## 修复

`tag.length + 48 + 32 + 16 + 1 + 32` → `tag.length + 32 + 32 + 16 + 1 + 32`(=120,空备注合法下限)。

## 验收

- [x] 修复后用链上真实 132 字节 payload 回归:新增 `test/transaction/duoqian-transfer/duoqian_transfer_decode_test.dart` 4 用例(链上实抓字节/空备注下限 120/截断/错 tag),修复前首条必红
- [x] `flutter analyze` 0 issue + `flutter test` 196/196 全过
- [ ] 真机:广场点击进入提案详情、机构详情列表显示提案(user 验证)

## 完工记录(2026-06-11)

- `duoqian_transfer_service.dart:936` 守卫 `48`→`32`(下限 136→120);新增 `@visibleForTesting debugDecodeProposalData` 测试入口。
- 全仓扫描确认 48 字节主体残留仅此一处(citizenwallet 零残留;chain_tx_monitor:608 的 48=32+16 合法)。
- 排查过程中的伴生事实(已另行成立):finalized 滞后于提案所在块时手机按 finalized 读链同样看不到提案——跳空块 + GRANDPA 只投到 best−2 导致死水期尾块不固化,归 20260608-pow-difficulty 任务族处理。
