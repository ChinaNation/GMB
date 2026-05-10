任务需求：
wuminapp 一致性整改 5 项：
1. 删 admins_change/admins_change.dart barrel(0 引用死代码)
2. 扁平化 admins_change/controllers/(1 文件升顶层)
3. 扁平化 admins_change/qr/(1 文件升顶层)
4. 扁平化 organization-manage/institution/(2 文件升顶层)
5. admins_change/ → admins-change/(命名统一为 kebab-case,与 organization-manage/personal-manage/duoqian-transfer/链端 admins-change pallet 对齐)

所属模块：
- wuminapp
- 受影响:lib/admins_change/(改名+2 子目录扁平+barrel 删)、lib/organization-manage/institution/(扁平)、外部模块 import 字符串

输入文档：
- memory/AGENTS.md
- memory/07-ai/chat-protocol.md
- memory/07-ai/agent-rules.md
- 用户在本对话中确认的整改方案

必须遵守：
- 0 行为变化:纯目录重命名 + 文件移动 + import 字符串替换,禁止顺手重构
- 不可绕过既有契约:链上 codec / 提案数据结构不变
- 不可改 Dart 源代码逻辑

输出物：
- 代码:
  1. 删 lib/admins_change/admins_change.dart
  2. lib/admins_change/controllers/admin_set_change_controller.dart 升到 lib/admins_change/ 顶层
  3. lib/admins_change/qr/admin_set_change_qr_adapter.dart 升到 lib/admins_change/ 顶层
  4. lib/organization-manage/institution/{create,close}_page.dart 升到 lib/organization-manage/ 顶层
  5. lib/admins_change → lib/admins-change(目录改名)+ test/admins_change → test/admins-change
  6. 外部模块全工程 import 字符串同步替换
- 测试:flutter analyze 0 + flutter test 全过
- 文档更新:任务卡迁入 done/、追加 memory 项目记录、更新 active 技术文档(unified-protocols/GOVERNANCE_TECHNICAL/PERSONAL_MANAGE/WUMINAPP_TECHNICAL/admins-change 相关)
- 残留清理:grep `admins_change/` 与 `organization-manage/institution/` 引用全为零

验收标准：
- flutter analyze 0 issue
- flutter test 全过
- 残留 grep 0
- lib/admins-change/ 内仅剩 5 子目录(codec/models/services/pages/widgets)+ 2 顶层 .dart(controller + qr adapter)
- lib/organization-manage/ 14 文件全平铺(无 institution/ 子目录)
