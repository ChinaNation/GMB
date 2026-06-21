# 任务卡：CPMS 档案列表新增市镇列

## 任务需求

在 CPMS 公民档案列表中，在“年龄”和“公民状态”之间新增“市镇”列，只显示公民所属镇/街道名称，例如“茅台镇”。

## 建议模块

- CPMS 前端 `dangan`
- CPMS 前端 `address`
- CPMS 技术文档

## 影响范围

- `citizenpassport/frontend/dangan/ArchiveList.tsx`：加载当前市镇列表并按 `town_code` 显示镇名。
- `citizenpassport/frontend/dangan/types.ts`：清理操作管理员命名残留。
- `citizenpassport/CITIZENPASSPORT_TECHNICAL.md`：记录档案列表显示“市镇”列。

## 主要风险点

- 镇名来自当前 CPMS 实例的地址字典，接口异常时列表不能崩溃。
- 表格新增列后不能把操作列挤乱。

## 是否需要先沟通

- 否。用户已明确列名和显示内容。

## 执行清单

- [x] 档案列表加载镇/街道字典。
- [x] 新增“市镇”列并显示镇名。
- [x] 清理命名残留并更新文档。
- [x] 运行前端构建和残留扫描。

## 完成记录

- 2026-05-30：创建任务卡，开始执行。
- 2026-05-30：完成前端档案列表“市镇”列，显示 `town_code` 对应镇/街道名称；更新文档并清理操作管理员命名残留。
- 2026-05-30：验证通过 `npm run build`、`git diff --check` 和残留扫描。
