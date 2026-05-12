任务需求：
修复个人多签第 1 步：只修改 wuminapp 和 wumin 冷钱包，彻底切换到最终目标状态，不做旧格式兼容。已注销个人多签账户继续显示在账户列表中，状态显示“已注销”，不显示金额；详情页右上角增加“删除”按钮，确认后清空该账户本地全部数据；新增个人多签恢复普通阈值输入，注册/注销提案固定全员同意；扫码添加管理员使用真正扫码图标；账户列表标题、加号弹窗文案和机构图标按用户要求修正；冷钱包只解析新的个人多签创建交易格式。

所属模块：
- wuminapp
- wumin

输入文档：
- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/workflow.md
- memory/07-ai/unified-protocols.md
- memory/07-ai/unified-naming.md
- memory/05-modules/wuminapp/personal-manage/PERSONAL_MANAGE_WUMINAPP_TECHNICAL.md
- memory/05-modules/wuminapp/qr/
- memory/05-modules/wuminapp-vs-wumin.md
- wumin/README.md
- memory/07-ai/module-checklists/wuminapp.md
- memory/07-ai/module-definition-of-done/wuminapp.md

必须遵守：
- 不可突破模块边界，本任务不修改 citizenchain/runtime。
- 不允许做旧格式兼容、过渡兼容、双轨兼容或保留旧分支。
- 不允许在 wuminapp 业务模块内实现或绕过投票流程。
- 创建个人多签交易载荷直接切换为最终字段顺序：account_name、duoqian_admins、regular_threshold、amount。
- 冷钱包只解析最终新格式，废弃旧个人多签创建载荷格式。
- 代码必须补中文注释。
- 改代码后必须更新文档、完善注释、清理残留。

输出物：
- wuminapp 代码
- wumin 冷钱包代码
- 中文注释
- 测试
- 文档更新
- 残留清理

验收标准：
- 已注销个人多签账户在账户列表显示为“已注销”，不显示金额。
- 已注销个人多签详情页不显示“未找到”“100.00”“不可用”。
- 已注销个人多签详情页右上角显示“删除”，确认后清空该账户本地全部数据。
- 新增个人多签支持用户输入普通阈值，并校验过半到管理员总数。
- 注册提案显示“注册须全员同意”，注销提案显示“注销须全员同意”。
- 扫码添加管理员使用真正扫码图标。
- 多签交易入口页标题显示“账户列表”。
- 加号弹窗中新增个人多签显示“无需身份ID”，新增机构多签显示“需要身份ID”，机构入口使用建筑/机构图标。
- wumin 冷钱包只解析新个人多签创建载荷格式。
- 测试通过。
- 文档已更新。
- 残留已清理。

执行记录：
- 2026-05-11：第 1 步已修改 wuminapp 与 wumin 冷钱包，不修改 runtime。
- 2026-05-11：个人多签创建载荷已切为 `account_name / duoqian_admins / regular_threshold / amount`，不保留旧载荷兼容。
- 2026-05-11：已注销账户列表、详情、删除按钮、阈值输入、扫码图标、账户列表标题、加号弹窗副文案和机构图标已按目标状态调整。
- 2026-05-11：已执行 `flutter analyze`、`flutter test test/governance/personal-manage`、`flutter test test/signer/payload_decoder_test.dart`。

- 状态：done

## 完成信息

- 完成时间：2026-05-11 12:50:10
- 完成摘要：第1步完成：wuminapp 和 wumin 冷钱包已切到个人多签最终目标状态；不保留旧交易格式兼容；已注销账户显示、删除按钮、阈值输入、扫码图标、账户列表标题、加号弹窗文案、冷钱包新载荷解码和文档测试均已完成。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
