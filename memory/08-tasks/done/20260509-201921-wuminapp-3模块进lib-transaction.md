任务需求：
wuminapp 3 个模块统一收编进 lib/transaction/,与链端 runtime/transaction/ 对齐:
- lib/duoqian-transfer/ → lib/transaction/duoqian-transfer/(名字不变)
- lib/onchain/ → lib/transaction/onchain-transaction/(改名,与链端 onchain-transaction pallet 一致)
- lib/offchain/ → lib/transaction/offchain-transaction/(改名,与链端 offchain-transaction pallet 一致)
- 删除 lib/offchain/offchain.dart 0 引用 barrel
- test/ 同步建空目录占位
- memory/05-modules/wuminapp/onchain/ 与 offchain/ 文档目录同步改名

不做:lib/asset/(它对应链端 issuance/onchain-issuance,与 transaction/institution-asset 无关)

所属模块：
- wuminapp
- 受影响:lib/transaction/(新建)、lib/{duoqian-transfer,onchain,offchain}/(全删)、外部 12 文件 import 替换、memory 8 文档同步

输入文档：
- memory/AGENTS.md
- memory/07-ai/chat-protocol.md
- 用户在本对话中确认的 Q1-Q4 决策

必须遵守：
- 0 行为变化:文件搬家 + 路径替换,禁止动业务逻辑
- 不动 lib/asset/
- 不动链端
- offchain.dart barrel 0 引用确认后才能删

输出物：
- 代码:
  1. 3 lib 目录搬入 lib/transaction/ + 2 改名(onchain→onchain-transaction、offchain→offchain-transaction)
  2. 删除 lib/offchain/offchain.dart barrel
  3. test/transaction/{duoqian-transfer,onchain-transaction,offchain-transaction}/ 空目录占位
  4. import 路径全工程替换(3 sed pattern)
  5. memory/05-modules/wuminapp/{onchain,offchain}/ → transaction/{onchain-transaction,offchain-transaction}/ 目录改名
  6. 8 active 文档内 lib 路径引用同步
- 测试:flutter analyze 0 + flutter test 全过(允许 personal_pending_create_lookup_test Isar flake)
- 文档更新:任务卡迁入 done/、追加 memory 项目记录
- 残留清理:grep `wuminapp_mobile/duoqian-transfer/` `wuminapp_mobile/onchain/` `wuminapp_mobile/offchain/` 全为 0

验收标准：
- flutter analyze 0 issue
- flutter test 全过
- 残留 grep 0
- lib/transaction/ 下含 3 子目录(duoqian-transfer/onchain-transaction/offchain-transaction)
- lib/{duoqian-transfer,onchain,offchain}/ 全部消失
- memory/05-modules/wuminapp/transaction/ 下含 2 子目录(onchain-transaction/offchain-transaction)
