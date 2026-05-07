# P0-4 统一 wuminapp 在线签名 immortal era

## 任务目标

执行重新创世前总审计 P0-4：`wuminapp` 所有在线签名 extrinsic 与 `citizenchain/node`、冷钱包提交规则统一为 immortal era，禁止继续使用 `_eraPeriod = 64` 和最新块哈希参与签名。

## 当前真源

- 统一协议入口：`memory/07-ai/unified-protocols.md`
- 链端 node 签名真源：`citizenchain/node/src/governance/signing.rs`
- polkadart 编码规则：`eraPeriod == 0` 编码为 immortal `0x00`

## 预计修改目录

- `memory/07-ai/`：登记统一签名 era 协议；只涉及文档。
- `memory/08-tasks/open/`：记录 P0-4 执行范围、结果和验收；只涉及文档。
- `wuminapp/lib/rpc/`：新增统一 signed extrinsic 构造器，修正 `onchain.dart` 和 `chain_rpc.dart` 注释；涉及 Dart 代码与中文注释。
- `wuminapp/lib/proposal/`：替换内部投票、runtime upgrade、转账提案签名路径；涉及 Dart 代码。
- `wuminapp/lib/duoqian/shared/`：替换机构多签管理提交路径；涉及 Dart 代码。
- `wuminapp/lib/offchain/rpc/`：替换清算行链上交易提交路径；涉及 Dart 代码。
- `wuminapp/test/rpc/`：新增 immortal era 构造器回归测试；涉及测试。

## 执行清单

- [x] 在统一协议文件登记 `P-SIGN-001：Citizenchain signed extrinsic era`。
- [x] 新增统一 signed extrinsic 构造器，固定 `eraPeriod = 0`、`blockNumber = 0`、`blockHash = genesisHash`。
- [x] 替换 `OnchainRpc.transferKeepAlive` 在线签名路径。
- [x] 替换 `InternalVoteService` 在线签名路径。
- [x] 替换 `RuntimeUpgradeService` 在线签名路径。
- [x] 替换 `TransferProposalService` 在线签名路径。
- [x] 替换 `DuoqianManageService` 在线签名路径。
- [x] 替换 `OnchainClearingBankRpc` 在线签名路径。
- [x] 清理 `_eraPeriod = 64`、`Mortal era`、`mortal era=64` 注释残留。
- [x] 增加回归测试并运行验收。

## 验收标准

- `rg -n "_eraPeriod\\s*=\\s*64|mortal era=64|Mortal era" wuminapp/lib wuminapp/test` 无输出。
- `rg -n "SigningPayload\\(|ExtrinsicPayload\\(" wuminapp/lib` 只命中统一构造器。
- `rg -n "fetchLatestBlock\\(\\)" wuminapp/lib/rpc wuminapp/lib/proposal wuminapp/lib/duoqian/shared wuminapp/lib/offchain/rpc` 不命中 signed extrinsic 构造路径。
- `flutter test test/rpc/signed_extrinsic_builder_test.dart` 通过。
- `flutter test test/duoqian test/proposal test/trade` 通过。
- `flutter analyze lib/rpc lib/proposal lib/duoqian/shared lib/offchain/rpc test/rpc test/duoqian test/proposal test/trade` 通过。
- `git diff --cached --check` 通过。

## 执行结果

2026-05-07 已执行：

- 在 `memory/07-ai/unified-protocols.md` 登记 `P-SIGN-001：Citizenchain signed extrinsic era`，明确热钱包在线签名、node、冷钱包提交统一 immortal era。
- 新增 `wuminapp/lib/rpc/signed_extrinsic_builder.dart`，统一负责 metadata、genesisHash、runtimeVersion、nonce、签名 payload、extrinsic body、submit 和失败 nonce 回滚。
- 统一构造器固定：
  - `eraPeriod = 0`
  - `era bytes = 0x00`
  - `blockNumber = 0`
  - `SigningPayload.blockHash = genesisHash`
  - `ExtrinsicPayload.blockNumber = 0`
- 已替换以下在线签名路径：
  - `OnchainRpc.transferKeepAlive`
  - `InternalVoteService`
  - `RuntimeUpgradeService`
  - `TransferProposalService`
  - `DuoqianManageService`
  - `OnchainClearingBankRpc`
- `ChainRpc.fetchLatestBlock()` 保留给 UI 展示、事件查询和诊断，不再用于 signed extrinsic 构造。
- 已新增 `wuminapp/test/rpc/signed_extrinsic_builder_test.dart`，覆盖 immortal signing payload 与 extrinsic payload。

验收记录：

- `flutter test test/rpc/signed_extrinsic_builder_test.dart`：通过。
- `flutter test test/duoqian test/proposal test/trade`：通过。
- `flutter analyze lib/rpc lib/proposal lib/duoqian/shared lib/offchain/rpc test/rpc test/duoqian test/proposal test/trade`：通过。
- `rg -n "_eraPeriod\\s*=\\s*64|mortal era=64|Mortal era" wuminapp/lib wuminapp/test`：无输出。
- `rg -n "SigningPayload\\(|ExtrinsicPayload\\(" wuminapp/lib`：只命中 `wuminapp/lib/rpc/signed_extrinsic_builder.dart`。
- `rg -n "fetchLatestBlock\\(\\)" wuminapp/lib/rpc wuminapp/lib/proposal wuminapp/lib/duoqian/shared wuminapp/lib/offchain/rpc`：只命中 `ChainRpc.fetchLatestBlock()` 方法定义；signed extrinsic 构造路径无命中。
