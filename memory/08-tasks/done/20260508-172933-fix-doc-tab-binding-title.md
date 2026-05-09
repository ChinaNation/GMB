# 修复本地文档 tab 绑定与白皮书标题

## 任务需求

- 修复公民宪法 tab 错误显示白皮书内容的问题。
- 修正白皮书页面顶部标题为“公民区块链白皮书”。

## 影响范围

- `citizenchain/node/frontend/other/other-tabs/OtherTabsSection.tsx`：修复本地文档 tab 与内置文档的绑定逻辑。
- `citizenchain/node/frontend/other/other-tabs/LocalDocViewer.tsx`：调整白皮书页面顶部显示标题。

## 验收项

- 公民宪法 tab 只能显示 `constitution` 文档，不能再静默回退到白皮书。
- 文档 tab 不再维护第二套文档映射字段，避免 key 与文档源不一致。
- 白皮书页面顶部标题显示为“公民区块链白皮书”。
- `npm --prefix citizenchain/node/frontend run build` 通过。
- `WASM_FILE=.../citizenchain.compact.compressed.wasm cargo check --manifest-path citizenchain/node/Cargo.toml` 通过。

## 完成记录

- 已定位根因：`OtherTabsSection` 在文档查找失败时使用第一个本地文档作为兜底，导致公民宪法 tab 在字段缺失或不匹配时静默显示白皮书。
- 已改为以当前 tab key 绑定本地文档，查找失败时显示配置错误，不再回退到白皮书。
- 已删除文档 tab 的额外文档映射字段，后端协议、前端类型和文档统一改为只按 tab key 绑定。
- 已将白皮书页面顶部标题改为“公民区块链白皮书”。
- 已执行 `npm --prefix citizenchain/node/frontend run build`，构建通过。
- 已执行带 `WASM_FILE` 的 `cargo check --manifest-path citizenchain/node/Cargo.toml`，构建通过。
