# 恢复白皮书与公民宪法旧版显示样式

## 任务需求

- 认真对照旧版 `docs/GMB_README.html` 与 `docs/FRC_README.html` 的结构、CSS 和渲染行为。
- 在不恢复旧 HTML 真源、不恢复 iframe/静态 HTML 入口的前提下，把旧版文档显示样式迁移到新的 React 本地 Markdown 渲染实现。
- 该任务当时保持白皮书与公民宪法 Markdown 真源；后续公民宪法已收口到 runtime HTML 真源。

## 影响范围

- `citizenchain/node/frontend/other/other-tabs/`：本地文档渲染组件。
- `citizenchain/node/frontend/app/styles/global.css`：文档页面样式恢复。
- `memory/05-modules/citizenchain/node/other/other-tabs/OTHER_TABS_TECHNICAL.md`：记录新实现复刻旧视觉结构。

## 验收项

- 新实现的文档页 DOM 结构与旧 HTML 的 hero / layout / toc / article 主体语义一致。
- 旧版背景、纸张、目录、正文排版、表格、代码块、图片、移动端布局样式迁移到新实现。
- `npm --prefix citizenchain/node/frontend run build` 通过。

## 完成记录

- 已从 Git 中逐段核对旧 `docs/GMB_README.html` 与 `docs/FRC_README.html` 的 CSS、DOM 和 JS 渲染逻辑。
- 已把新 React 组件改为旧版 `hero / layout / toc / paper / markdown-body / to-top` 结构。
- 已恢复旧版左侧可折叠目录、正文内置目录剥离、代码块表格转真表格、横线清理、白皮书标题居中、宪法章/节标题排版。
- 已将旧版背景、纸张、目录、正文、表格、代码、图片、移动端布局样式按旧 HTML 迁移到 `global.css`。
- 已执行 `npm --prefix citizenchain/node/frontend run build`，构建通过。
