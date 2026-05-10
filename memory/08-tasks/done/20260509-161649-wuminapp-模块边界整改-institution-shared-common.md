任务需求：
wuminapp 模块边界整改：删除顶层 `lib/institution/` 目录、删除 `lib/organization-manage/shared/` 子目录、新建 `lib/common/`。
让三个业务模块（organization-manage / personal-manage / admins_change）严格按"共用→lib/common/、机构多签→organization-manage/、个人多签→personal-manage/、管理员管理框架→admins_change/"的边界归位。

所属模块：
- wuminapp
- 受影响目录：lib/common/（新建）、lib/organization-manage/、lib/personal-manage/、lib/admins_change/、lib/institution/（删除）、lib/duoqian-transfer/、lib/proposal/、lib/ui/、lib/vote/、lib/governance/、lib/citizen_tab_page.dart

输入文档：
- memory/AGENTS.md
- memory/07-ai/chat-protocol.md
- memory/07-ai/agent-rules.md
- memory/05-modules/wuminapp/
- 用户在本对话中确认的整改方案（共用目录命名 common、单任务卡执行、5 处违规仅改路径）

必须遵守：
- 不可突破模块边界：personal-manage 文件不得回流到 organization-manage；机构私有 model/service 不得抽到 common
- 不可绕过既有契约：链上 codec / 提案数据结构不变
- 不可擅自修改安全红线：不改 isar/wallet/signer/qr 任何业务逻辑
- 0 行为变化：纯目录重构 + import 路径更新，禁止顺手重构、增删字段、改函数签名

输出物：
- 代码：
  1. 新建 `lib/common/institution_info.dart`（来自原 institution_data.dart 的通用类型部分）
  2. 新建 `lib/common/admin_institution_codec.dart`（来自原 organization-manage/shared/）
  3. 在 organization-manage/ 下安置 institution_detail_page.dart / institution_admin_list_page.dart / institution_registry.dart + governance_institution_registry.generated.dart
  4. 拍平 organization-manage/shared/ 6 文件到 organization-manage/ 顶层
  5. 删除空的 lib/institution/ 与 lib/organization-manage/shared/
- 测试：flutter analyze 0 error
- 文档更新：本任务卡迁入 done/、追加 memory 项目记录
- 残留清理：grep 确认 `wuminapp_mobile/institution/` 与 `organization-manage/shared/` 引用全为零

验收标准：
- flutter analyze 0 error 0 warning
- grep 残留：
  * `import 'package:wuminapp_mobile/institution/'` → 0 处
  * `organization-manage/shared/` → 0 处
- 三个业务模块目录清单（ls）符合预期
- common 目录仅含真共用代码（InstitutionInfo + admin_institution_codec），不含任何模块私有
- 任务卡迁入 memory/08-tasks/done/

不做：
- personal-manage 子目录化（按用户决定）
- proposal/ vote/ governance/ qr/ signer/ 等其他模块边界整改
- 任何业务逻辑、字段、行为变化
