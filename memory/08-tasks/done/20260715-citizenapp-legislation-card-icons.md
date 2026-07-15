# CitizenApp 立法页顶部五卡图标与顺序调整

任务需求：
- 立法 tab 顶部第 3 行左右对调，从「众议会｜参议会」改为「参议会｜众议会」。
- 四张国家级机构卡加前置图标（图标在前、文字在后），与宪法卡一致。
- 五卡统一采用方案二「五色语义 chip」：圆形浅底 + 深色图标，一机构一色。
- 图标归属（用户定稿）：
  - 公民宪法 → `Icons.menu_book`（书本，翠绿）
  - 国家立法院 → `Icons.account_balance`（房子，金）
  - 国家教委会 → `Icons.school`（蓝，保持）
  - 国家参议会 → `Icons.gavel`（锤子，紫）
  - 国家众议会 → `Icons.groups`（绿，保持）

安全边界：
- 纯展示层改动，不动机构数据源、跳转逻辑、链读与排序。
- 机构码 NLG/NED/NRP/NSN 与 `_openDetail`/`_openConstitution` 语义不变。

预计修改目录：
- `citizenapp/lib/citizen/legislation/legislation_tab.dart`：对调第 3 行、`_nationalCard` 加图标 chip、宪法卡换图标；新增 chip 规格常量。

验收标准：
- 第 3 行显示为「国家参议会（左）｜国家众议会（右）」。
- 五卡均带圆形色底图标，图标与机构对应正确，颜色符合方案二五色。
- 宪法卡图标为书本，立法院为房子，参议会为锤子。
- `dart analyze` 无新增告警，`dart format` 通过。

执行记录：
- 2026-07-15：用户在 5 套方案中选方案二并重排图标归属；本卡为纯 UI 调整。
- 2026-07-15：落地 `legislation_tab.dart`——对调第 3 行、`_nationalCard` 加图标 chip、宪法卡换书本、新增 `_CardIcon`/`_nationalIcons`/`_iconChip`。
- 2026-07-15：修正 chip 形状 bug（误做成 `BoxShape.circle`）→ 圆角方形 `BorderRadius.circular(10)`，与方案二渲染稿一致；教训入 memory `feedback_implement_exact_approved_visual`。
- 2026-07-15：`dart format` + `dart analyze` 通过，用户确认，归档。

结论：完成。
