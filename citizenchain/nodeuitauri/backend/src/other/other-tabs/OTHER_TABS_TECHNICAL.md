# Other Tabs 模块技术文档

## 1. 模块位置

- 路径：`nodeuitauri/backend/src/other/other-tabs/mod.rs`
- 对外命令：
  - `get_other_tabs_content`

## 2. 模块职责

- 统一提供“白皮书 / 公民治理宪法 / 公民党”三个标签页的内容配置。
- 将前端展示项抽象为结构化数据，避免在前端硬编码多个来源。

## 3. 数据模型

- `OtherTabsPayload`
  - `tabs: Vec<OtherTabItem>`
- `OtherTabItem`
  - `key`: 业务标识（whitepaper/constitution/party）
  - `title`: 标题
  - `contentType`: 展示类型（iframe/text）
  - `url`: iframe 来源（可选）
  - `text`: 纯文本内容（可选）

## 4. 当前内容来源

- 白皮书：`https://chinanation.github.io/GMB/GMB_README.html`
- 公民治理宪法：`https://chinanation.github.io/GMB/FRC_README.html?v=20260310-1`
- 公民党：占位文本（待接入）

## 5. 安全策略

- 前端渲染 iframe 时应用 `sandbox="allow-scripts allow-same-origin"` 属性，限制嵌入页面的能力。
- 设置 `referrerPolicy="no-referrer"` 避免向外部页面泄漏来源信息。
