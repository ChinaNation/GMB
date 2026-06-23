# 任务卡：修复公民宪法目录白屏并同步第五十二条英文

## 任务需求

区块链软件“公民宪法” tab 在 Tauri/WebView 的 sandbox iframe 中展示 runtime 宪法 HTML。点击左侧目录中的文章条目后，页面会变成白屏。需要修复目录点击逻辑，同时保持 iframe 不开放同源权限。

第五十二条中文已调整为“联邦安全局、联邦情报局、联邦特勤局、联邦注册局和联邦人事局”，需要将对应英文同步为当前中文含义和顺序。

## 修改范围

- `citizenchain/runtime/primitives/src/CitizenConstitution.html`
  - 修复目录点击逻辑，禁止在 sandbox `srcDoc` 环境中走默认 hash 导航。
  - 同步第五十二条第一段英文内容。
- `memory/05-modules/citizenchain/node/other/other-tabs/OTHER_TABS_TECHNICAL.md`
  - 补充公民宪法 iframe 目录点击安全约束。

## 目标状态

- 点击左侧目录中的章节、节和文章条目时，不再触发 WebView 白屏导航。
- 公民宪法 HTML 仍只在 `sandbox="allow-scripts"` iframe 中运行，不增加 `allow-same-origin`。
- 第五十二条英文与当前中文机构全称/简称保持一致。
- 技术文档说明 runtime 宪法 HTML 在 sandbox iframe 内必须用脚本滚动处理目录锚点。

## 风险与边界

- 公民宪法 HTML 编入 runtime，正式链生效需要发布 runtime 升级或重新构建对应 runtime。
- 本任务不修改 Tauri CSP，不放宽 iframe sandbox 安全边界。
- 本任务不触碰 CID 当前未提交改动。

## 执行记录

- 已修改 `CitizenConstitution.html` 第五十二条第一段英文，将旧的 National Bureau 表述同步为 Federal Security / Intelligence / Special Service / Registry / Personnel Bureau。
- 已修改 `CitizenConstitution.html` 左侧目录点击脚本：
  - 所有有效 `#id` 目录链接统一先 `preventDefault()`。
  - 章节/节分支只展开和高亮，不触发默认 hash 导航。
  - 文章条目通过 `document.getElementById(id).scrollIntoView()` 在 iframe 内部滚动。
- 已保留 `RuntimeConstitutionViewer.tsx` 的 `sandbox="allow-scripts"` 安全边界，未加入 `allow-same-origin`。
- 已更新 `OTHER_TABS_TECHNICAL.md`，记录 sandbox `srcDoc` 环境下目录链接不得依赖默认 hash 导航。

## 验证记录

- `npm run build`（`citizenchain/node/frontend`）：通过；仅保留既有 Vite 大 chunk 提示。
- `git diff --check`（本任务涉及文件）：通过。
- Node 语法检查：`CitizenConstitution.html` 内联脚本可被 `new Function()` 正常解析。
- 残留扫描确认第 52 条不再保留旧的 `National Security Bureau` / `National Intelligence Bureau` / `National Registration Bureau` / `National News Bureau` / `National Personnel Bureau` 表述。
- 浏览器打开顶层 `CitizenConstitution.html` 后，展开“第二章 政府 / 第一节 总统府”并点击“第五十二条”：
  - 页面未白屏。
  - URL 未追加 hash，说明默认 hash 导航已被拦截。
  - 页面最终滚动到第 52 条附近，并显示新的英文机构名。
- in-app 浏览器无法直接向 `sandbox="allow-scripts"` 且无 `allow-same-origin` 的 iframe 内部发送点击事件，验证工具报不可访问 iframe 限制；该限制与当前安全边界一致，不作为页面失败处理。
