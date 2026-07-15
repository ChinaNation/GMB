# CitizenApp 机构/管理员显示三修复(千分位·删岗位码·统一SS58)

任务需求(用户三条指令):
1. 所有金额统一使用千分位。
2. 删掉岗位码的显示。
3. 所有钱包账户统一显示 SS58 地址。

安全边界:
- 纯展示层改动;链上数据、单位换算、账户/岗位来源均正确,不动链端、不动 codec/model 解码、不动余额 key 逻辑(仍按 hex key 查余额)。
- admin_set_editor 的「管理员公钥 hex」输入框保留 hex(那是输入不是展示)。

修复清单:
- 新增原语:`account_derivation.dart` 加 `ss58FromHex(String hex, {ss58Prefix=2027})`(hex→bytes→ss58FromAccountId),作为 hex→SS58 展示唯一便捷入口。
- ① 千分位(裸 toStringAsFixed(2)→AmountFormat.formatThousands),7 处:
  - institution_assignment_card.dart:42、institution_detail_page.dart:456、institution_accounts_page.dart:109、
    admins-change/pages/admin_account_detail_page.dart:62、widgets/admin_set_editor.dart:44、widgets/admin_set_diff_card.dart:55、
    transaction/offchain-transaction/pages/withdraw_page.dart:117
  - 另修双后缀 bug institution_account_info_page.dart:550(format 默认 symbol=GMB → "GMB GMB")改 formatThousands。
- ② 删岗位码:institution_assignment_card.dart:35 `Text('岗位码：${roleCode}')` 删除(roleName 已作标题显示;roleCode 内部标识不展示)。
- ③ 统一 SS58(裸 hex→ss58FromHex),4 处显示:
  - institution_assignment_card.dart:40(管理员账户)、admin_account_detail_page.dart:60(list title)、
    admin_set_editor.dart:42(list title)、admin_set_diff_card.dart:53(list title)

验收标准:
- `dart analyze` 无新增告警、`dart format` 通过。
- 上述金额点显示千分位;任职卡不再有「岗位码」行;四处账户显示为 SS58(prefix 2027)。
- 余额按 hex key 查询逻辑不变;编辑器 hex 输入框不变。

执行记录:
- 2026-07-15:诊断三处根因(工作流+自查坐实)后,用户下达三条修复指令;本卡执行纯展示层修复。
- 2026-07-15:落地完成。account_derivation.dart 新增 ss58FromHex;9 文件改动(千分位7处+双后缀1处+删岗位码1处+SS58 4处);import 全补齐。
- 2026-07-15:兜底确认无遗漏(AdminAccountCard 不显示 hex;institution/admins-change 无残留裸 hex 账户显示)。
- 2026-07-15:dart analyze 全 No issues;dart format 通过;新增 account_derivation_ss58_test.dart 3 用例全过(round-trip/0x大写容忍/产出合法SS58)。

结论:三条指令全部完成并验证通过(纯展示层,链端/codec/余额key逻辑未动)。roleName(如护宪大法官)仍作任职卡标题显示,仅删除了岗位码那一行。
