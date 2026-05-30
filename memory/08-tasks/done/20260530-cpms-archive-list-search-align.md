# 任务卡：CPMS 档案列表统一精确检索与居中显示

## 任务需求

删除档案列表检索字段选择器，改为一个统一精确检索输入框；输入姓名、护照号或档案号时直接精确检索对应档案。表格表头与内容居中显示，输入框占用原选择器加输入框总宽度。

## 建议模块

- CPMS 前端 `dangan`
- CPMS 后端 `dangan`
- CPMS 技术文档

## 影响范围

- `cpms/frontend/dangan/ArchiveList.tsx`：删除字段选择器，统一输入框、表格居中。
- `cpms/frontend/dangan/api.ts`：列表参数改为 `search`。
- `cpms/backend/src/dangan/routes.rs`：列表接口用 `search` 精确匹配档案号、护照号或姓名。
- `cpms/CPMS_TECHNICAL.md` 与 dangan 模块文档：同步检索规则。

## 主要风险点

- 不能恢复模糊搜索，必须保持百万级列表的精确检索。
- 不保留旧 `archive_no/passport_no/name` 查询参数，避免选择器式旧接口残留。

## 是否需要先沟通

- 否。用户已明确交互和显示要求。

## 执行清单

- [x] 前端删除选择器并加长统一输入框。
- [x] 后端列表接口改为 `search` 精确 OR 检索。
- [x] 表格表头与内容居中。
- [x] 更新文档并清理残留。
- [x] 运行后端测试、clippy、前端构建和残留扫描。

## 完成记录

- 2026-05-30：创建任务卡，开始执行。
- 2026-05-30：完成档案列表统一精确检索；前端删除字段选择器，输入框 placeholder 改为“请输入姓名、护照号、档案号检索”，宽度调整为原选择器加输入框合计宽度。
- 2026-05-30：后端 `GET /api/v1/archives` 改为 `search` 参数，精确匹配档案号、护照号或姓名，并拒绝旧选择器式字段参数。
- 2026-05-30：表格表头和内容居中；验证通过 `cargo fmt --check`、`cargo test`、`cargo clippy --all-targets -- -D warnings`、`npm run build`、`git diff --check` 和残留扫描。
