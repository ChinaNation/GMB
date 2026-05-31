# SFID 确定性元数据缓存与公安局本地分页

## 任务需求

- 公安局 tab 显示 `共 N 条 / 第 X 页 / 上一页 / 下一页`,每页 20 条。
- 公安局列表首次请求后缓存,再次进入直接显示缓存并本地分页。
- 注册局 tab 的确定性市列表增加缓存。
- 公权机构 tab、私权机构 tab 的确定性市列表增加缓存。
- 机构类 tab 的省份元数据增加缓存,避免每次点击都请求后端。
- 完成后更新文档、完善中文注释、清理残留。

## 修改范围

- `sfid/frontend/sfid`
- `sfid/frontend/common`
- `sfid/frontend/admins`
- `sfid/frontend/institutions`
- `sfid/frontend/App.tsx`
- `memory/05-modules/sfid/frontend`

## 约束

- 缓存只用于确定性元数据和公安局展示加速,不得作为业务真源。
- 普通公权机构和私权机构列表仍必须输入精确条件后查询。
- 公安局本地分页不得触发后端 cursor 翻页。

## 验收

- 公安局 tab 每页 20 条,显示总数和当前页。
- 公安局本地翻页不请求后端。
- 同一省城市列表第二次读取命中缓存。
- 机构类省份元数据第二次读取命中缓存。
- 前端构建通过。

## 完成记录

- 新增 `sfid/frontend/sfid/metaCache.ts`,统一承接省份元数据、城市列表和公安局确定性展示缓存。
- 公安局 Tab 改为本地 20 条分页,展示 `共 N 条 / 第 X 页`,上一页/下一页只切本地缓存切片。
- 注册局市列表、公权机构市列表、私权机构市列表、机构新增弹窗和通用元数据 hook 已改为读取缓存工具。
- 已更新 `memory/05-modules/sfid/frontend/FRONTEND_LAYOUT.md` 的前端缓存与公安局分页规则。
- 已执行 `npm --prefix sfid/frontend run build`,构建通过;Vite 仍提示单包超过 500 kB,非本次改动引入。
