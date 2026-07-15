# 挖矿网络卡片布局收口

状态：已完成并通过验收。

## 任务需求

- 删除挖矿 Tab 网络分组中的独立“全节点”和“轻节点”卡片。
- 保留“治理节点”与“在线节点”两张并排卡片。
- 治理节点卡片展示国家储委会、省储委会、省储行三类节点。
- 在线节点卡片展示在线节点、全节点、轻节点三类节点，并复用治理节点卡片的信息层级与三列布局。

## 所属模块

- CitizenChain 节点桌面端挖矿页面。

## 输入文档

- `memory/07-ai/unified-required-reading.md`
- `memory/01-architecture/repo-map.md`
- `memory/03-security/security-rules.md`
- `memory/05-modules/citizenchain/node/mining/network_overview/NETWORK_OVERVIEW_TECHNICAL.md`
- `memory/05-modules/citizenchain/node/mining/dashboard/MINING_DASHBOARD_TECHNICAL.md`
- `memory/07-ai/module-definition-of-done/citizenchain.md`

## 边界

- 只调整现有 React 组件、CSS 和模块技术文档。
- 不修改 Rust 后端、Tauri 命令、API 字段、统计口径或 runtime。
- 不新建代码、样式或测试文件。
- 不保留旧四卡片布局、治理专用重复样式或失效注释。

## 输出物

- 两张并排网络卡片的前端实现。
- 两张卡片共用的三列节点统计样式和中文注释。
- 网络总览技术文档更新。
- 前端构建与真实页面视觉验收记录。
- 旧布局和旧样式残留清理。

## 验收标准

- 网络分组顶层只显示“治理节点”“在线节点”两张卡片。
- 治理节点卡片依次显示国家储委会、省储委会、省储行。
- 在线节点卡片依次显示在线节点、全节点、轻节点。
- 两张卡片数字、名称、加载状态保持同一三列结构。
- 后端 `NetworkOverview` 数据结构和轮询行为不变。
- `npm run build` 通过，真实本地页面完成视觉验收。
- 文档、中文注释和旧布局残留清理完成。

## 实施结果

- `NetworkInlineSection` 顶层只保留“治理节点”“在线节点”两张卡片。
- 治理节点卡片依次展示国家储委会、省储委会、省储行；在线节点卡片依次展示在线节点、全节点、轻节点。
- 两张卡片统一使用 `network-node-*` 三列样式；旧 `governance-node-*` 专用样式和独立全节点、轻节点 JSX 已删除。
- 后端 `NetworkOverview`、Tauri 命令、轮询周期和统计口径均未修改。
- 网络总览技术文档已更新为两卡片目标布局，并修正后端源码真实路径。

## 验收结果

- `npm run build` 通过，包含 TypeScript 类型检查和 Vite 生产构建。
- 本地 Vite 页面真实打开挖矿 Tab；DOM 检测网络分组顶层卡片数为 2，标题依次为“治理节点”“在线节点”，两张卡片子列数均为 3。
- 视觉测量两张卡片位于同一行，宽度均为 594px，高度均为 97px；数字和节点类型上下对齐。
- 浏览器环境没有 Tauri `invoke` 桥接，因此页面显示预期的本地 API 错误提示；该限制不影响本次静态布局和真实渲染验收。
- `git diff --check`、旧 CSS 类、旧四卡片注释和独立全节点/轻节点卡片残留扫描通过。
- 本地 Vite 服务和验收浏览器页已停止；未修改 runtime、未推送、未部署。
