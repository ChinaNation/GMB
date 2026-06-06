# Other Tabs 模块技术文档

## 1. 模块位置

- 路径：`node/src/other/other-tabs/mod.rs`
- 前端路径：`node/frontend/other/other-tabs/`
  - `api.ts`：其他 tab 专用 Tauri API
  - `types.ts`：白皮书/宪法/公民党内容载荷类型
  - `LocalDocViewer.tsx`：本地 Markdown 文档渲染组件
  - `RuntimeConstitutionViewer.tsx`：链上 runtime 宪法 HTML 展示组件
  - `OtherTabsSection.tsx`：document/text 展示组件
  - `generated/local-docs.generated.ts`：构建前自动生成的本地文档 bundle
- 对外命令：
  - `get_other_tabs_content`
  - `get_runtime_constitution_document`

## 2. 模块职责

- 统一提供“白皮书 / 公民宪法 / 公民党”三个标签页的内容配置。
- 将前端展示项抽象为结构化数据，避免在前端硬编码多个来源。
- 白皮书正文只允许来自仓库根目录 `docs/《白皮书》.md`。
- 公民宪法正文只允许来自 `citizenchain/runtime/primitives/src/CitizenConstitution.html`，
  并通过 runtime API 从当前链上 runtime 读取；修改宪法必须发布 runtime 升级。

## 3. 数据模型

- `OtherTabsPayload`
  - `tabs: Vec<OtherTabItem>`
- `OtherTabItem`
  - `key`: 业务标识（whitepaper/constitution/party）
  - `title`: 标题
  - `contentType`: 展示类型（document/runtimeConstitution/text）
  - `text`: 纯文本内容（可选）
- `RuntimeConstitutionDocument`
  - `html`: 当前链上 runtime 内置的完整公民宪法 HTML
  - `blake2_256`: HTML 的 blake2_256 摘要
  - `source`: 来源标识，当前固定为 `runtime`

文档 tab 的本地文档绑定只允许使用 `key`，不再额外维护第二套映射字段，
避免“tab key 与文档 key 不一致”时错误显示其他文档。

## 4. 当前内容来源

- 白皮书：`docs/《白皮书》.md`
- 公民宪法：`citizenchain/runtime/primitives/src/CitizenConstitution.html`
- 公民党：占位文本（待接入）

`npm run dev` 与 `npm run build` 都会先执行 `npm run generate:docs`，由
`scripts/generate-local-docs.mjs` 读取白皮书 Markdown 真源并生成
`generated/local-docs.generated.ts`。公民宪法不再进入该 generated 文件，而是由
node 本地 RPC `constitution_getDocument` 从当前链上 runtime 读取。

## 5. 安全策略

- 文档使用 `marked` 转 HTML 后再经 `DOMPurify` 清洗，避免 Markdown 内嵌 HTML
  直接扩大渲染面；该策略只适用于白皮书 Markdown。
- 公民宪法 HTML 在 sandbox iframe 中运行，只允许脚本执行自身目录展开逻辑，不开放同源权限。
- 公民宪法目录链接必须由 HTML 自身脚本 `preventDefault()` 后调用 `scrollIntoView()`，
  不依赖 `srcDoc` 默认 hash 导航，避免 Tauri/WebView 在 sandbox iframe 中将锚点点击处理为空白导航。
- 公民宪法 runtime HTML 自带 `<style>` 和目录 `<script>`，Tauri CSP 必须通过
  `style-src-elem 'self' 'unsafe-inline'` 与 `script-src-elem 'self' 'unsafe-inline'`
  放行元素级内联样式/脚本；同时保留 `style-src-attr 'none'` 和
  `script-src-attr 'none'`，不允许 `style` 属性或内联事件处理器扩大攻击面。
- 不再使用 GitHub Pages、CDN 或 raw URL 加载白皮书/公民宪法。
- `vite.config.ts` 关闭 `publicDir`，避免旧静态页和唯一真源并存。
