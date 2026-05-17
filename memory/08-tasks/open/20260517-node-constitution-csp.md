# 任务卡：修复 Windows 公民宪法 tab 因 CSP 拦截内联样式脚本导致页面混乱的问题

## 任务需求

Windows 端区块链软件打开“公民宪法” tab 后，页面按无样式 HTML 渲染，目录和中英文标题粘连。需要修复 Tauri/WebView2 CSP 对 runtime 公民宪法 HTML 内联 `<style>` 与 `<script>` 的拦截。

## 影响范围

- `citizenchain/node/tauri.conf.json`：调整 Tauri CSP，允许公民宪法 iframe 中的内联样式元素和脚本元素执行。
- `citizenchain/node/frontend/other/other-tabs/RuntimeConstitutionViewer.tsx`：补充 iframe sandbox 安全边界中文注释。
- `memory/05-modules/citizenchain/node/other/other-tabs/OTHER_TABS_TECHNICAL.md`：回写公民宪法 runtime HTML 与 CSP 约束。
- `memory/05-modules/citizenchain/node/CROSS_PLATFORM_BUILD.md`：补充 Windows 安装包 QA 验收项。
- 不修改 `citizenchain/runtime/primitives/src/CitizenConstitution.html` 真源，不触发 runtime 升级。

## 修复目标

- 公民宪法 tab 在 Windows WebView2 中恢复正式样式。
- 目录折叠、高亮、回到顶部脚本可运行。
- iframe 仍不开放 `allow-same-origin`，不允许访问父页面同源资源。
- CSP 继续禁止 `style` 属性和内联事件处理器，避免无边界放开。

## 验收方式

- `tauri.conf.json` JSON 语法有效。
- 前端 TypeScript 与 Vite 构建通过。
- 残留扫描确认有效配置和文档均记录 `style-src-elem` / `script-src-elem`。
- Windows 重新打包后验证公民宪法 tab 标题分行、目录不粘连、目录交互正常。

## 状态

- [x] 创建任务卡
- [x] 调整 Tauri CSP
- [x] 补充前端安全注释
- [x] 更新模块文档和打包验收文档
- [x] 残留检查与验证
