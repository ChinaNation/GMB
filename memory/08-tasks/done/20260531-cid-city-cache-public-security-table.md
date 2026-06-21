# CID 注册局城市缓存首帧与公安局表格分页修正

## 任务需求

- 注册局城市列表已有本地缓存时直接显示,不得短暂显示暂无城市数据。
- 公安局列表表头改为 `序号 / 身份ID / 机构名称 / 省/市 / 账户数`,表头和数据居中对齐。
- 公安局序号按市公安局排序自动生成,跨本地分页连续编号。
- 删除公安局列表刷新按钮。
- 公安局分页显示 `共 X 页 / 第 Y 页`,原刷新按钮位置显示 `共 N 条`。

## 修改范围

- `citizencode/frontend/cid`
- `citizencode/frontend/admins`
- `citizencode/frontend/institutions`
- `memory/05-modules/citizencode/frontend`

## 约束

- 缓存只用于确定性城市元数据和公安局确定性展示列表。
- 普通公权机构和私权机构仍按精确条件服务端分页。
- 不做兼容旧 UI,直接按新口径修正。

## 验收

- 注册局城市缓存存在时首帧直接显示城市列表。
- 公安局表格新增序号列,身份ID列居中。
- 公安局无刷新按钮,分页信息显示总页数、当前页和总条数。
- 前端构建通过。

## 完成记录

- `citizencode/frontend/citizencode/metaCache.ts` 新增 `readCachedCidCities`,支持城市缓存同步读取。
- 注册局城市列表进入页面时优先同步读取本省城市缓存,缓存存在时直接显示,没有缓存时才进入加载态。
- 通用城市网格也接入同步缓存读取,避免缓存存在时仍短暂显示空态。
- 公安局列表删除刷新按钮,分页文案改为 `共 X 页 / 第 Y 页` 和 `共 N 条`。
- 公安局表格新增连续序号列,`CID` 表头改为 `身份ID`,主要列居中对齐。
- 已更新 `memory/05-modules/citizencode/frontend/FRONTEND_LAYOUT.md`。
- 已执行 `npm --prefix citizencode/frontend run build`,构建通过;Vite 仍提示单包超过 500 kB,非本次改动引入。
