# Other Tabs 模块技术文档

## 1. 模块位置

- 路径：`node/src/other/other-tabs/mod.rs`
- 前端路径：`node/frontend/other/other-tabs/`
  - `api.ts`：其他 tab 专用 Tauri API
  - `types.ts`：白皮书/宪法/公民党内容载荷类型
  - `LocalDocViewer.tsx`：本地 Markdown 文档渲染组件
  - `OtherTabsSection.tsx`：document/text 展示组件
  - `generated/local-docs.generated.ts`：构建前自动生成的本地文档 bundle
- 对外命令：
  - `get_other_tabs_content`

## 2. 模块职责

- 统一提供“白皮书 / 公民宪法 / 公民党”三个标签页的内容配置。
- 将前端展示项抽象为结构化数据，避免在前端硬编码多个来源。
- 白皮书与公民宪法正文只允许来自仓库根目录 `docs/《白皮书》.md` 与
  `docs/《公民宪法》.md`。

## 3. 数据模型

- `OtherTabsPayload`
  - `tabs: Vec<OtherTabItem>`
- `OtherTabItem`
  - `key`: 业务标识（whitepaper/constitution/party）
  - `title`: 标题
  - `contentType`: 展示类型（document/text）
  - `text`: 纯文本内容（可选）

文档 tab 的本地文档绑定只允许使用 `key`，不再额外维护第二套映射字段，
避免“tab key 与文档 key 不一致”时错误显示其他文档。

## 4. 当前内容来源

- 白皮书：`docs/《白皮书》.md`
- 公民宪法：`docs/《公民宪法》.md`
- 公民党：占位文本（待接入）

`npm run dev` 与 `npm run build` 都会先执行 `npm run generate:docs`，由
`scripts/generate-local-docs.mjs` 读取上述两个 Markdown 真源并生成
`generated/local-docs.generated.ts`。桌面端三端安装包内置该 generated 文件，
用户安装后看到的就是打包时仓库中的最新文档内容。

## 5. 安全策略

- 文档使用 `marked` 转 HTML 后再经 `DOMPurify` 清洗，避免 Markdown 内嵌 HTML
  直接扩大渲染面。
- 不再使用 iframe、GitHub Pages、CDN 或 raw URL 加载白皮书/公民宪法。
- `vite.config.ts` 关闭 `publicDir`，避免旧 HTML 静态页和唯一真源 Markdown 并存。
