任务需求：
wuminapp 把 4 个治理相关模块统一收编到 `lib/governance/` 下,与链端 `runtime/governance/` 对齐。
- lib/admins-change/ → lib/governance/admins-change/
- lib/organization-manage/ → lib/governance/organization-manage/
- lib/personal-manage/ → lib/governance/personal-manage/
- lib/proposal/runtime_upgrade/ → lib/governance/runtime-upgrade/(snake → kebab,与链端 pallet 名一致)

只移动 + import 路径替换,零功能改动。

所属模块：
- wuminapp
- 受影响:lib/governance/(新增 4 子目录)、lib/admins-change/(删)、lib/organization-manage/(删)、lib/personal-manage/(删)、lib/proposal/runtime_upgrade/(删)、外部模块全工程 import、test 同步、generator 脚本路径

输入文档：
- memory/AGENTS.md
- memory/07-ai/chat-protocol.md
- memory/07-ai/agent-rules.md
- 用户在本对话中确认的方案(横线 kebab-case)

必须遵守：
- 0 行为变化:纯目录移动 + 重命名(snake→kebab)+ import 字符串替换,禁止顺手改任何业务逻辑
- 不动 lib/proposal/ 其余内容(shared/、proposal_types_page.dart、resolution_destroy/、resolution_issuance/、grandpakey_change/)
- 不动 lib/governance/governance_list_page.dart
- 不动 lib/duoqian-transfer/、lib/vote/、lib/onchain/、lib/offchain/、lib/asset/、lib/common/
- 不动链端任何代码

输出物：
- 代码:
  1. 4 个目录搬家(lib + test 同步)
  2. import 路径全工程替换(4 个模式)
  3. tools/generate_wuminapp_governance_registry.mjs 输出路径更新
  4. active 技术文档路径同步(ADMINS_CHANGE_WUMINAPP_TECHNICAL/GOVERNANCE_TECHNICAL/WUMINAPP_TECHNICAL)
- 测试:flutter analyze 0 + flutter test 全过
- 文档更新:任务卡迁入 done/、追加 memory 项目记录
- 残留清理:grep `wuminapp_mobile/admins-change/` `wuminapp_mobile/organization-manage/` `wuminapp_mobile/personal-manage/` `wuminapp_mobile/proposal/runtime_upgrade/` 全为 0

验收标准：
- flutter analyze 0 issue
- flutter test 全过(允许 personal_pending_create_lookup_test Isar 状态泄漏 flake,与本次无关)
- 残留 grep 全 0
- lib/governance/ 下含 5 项:governance_list_page.dart + admins-change/ + organization-manage/ + personal-manage/ + runtime-upgrade/
- lib/proposal/ 仅剩 shared/、proposal_types_page.dart、resolution_destroy/、resolution_issuance/、grandpakey_change/

不做：
- proposal_types_page 任何 UI 改动(转账/安全基金/手续费 3 张卡留待下一轮)
- institution_detail_page 转账按钮位置(留待下一轮)
- transaction/issuance/votingengine 等其他模块对齐(留待下一轮)
