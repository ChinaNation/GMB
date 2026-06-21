# CID 市管理员单市上限

## 任务目标

CID 市管理员按市限制数量。每个省内每个市最多 30 个市管理员；前端像联邦管理员 `本省联邦管理员：2 / 5` 一样展示 `本市市管理员：x / 30`，并在达到上限时禁止继续新增。

## 修改范围

- `citizencode/backend/admins/`：新增市管理员时按省市统计并校验 30 人上限。
- `citizencode/backend/main.rs`：补充市管理员城市上限错误码映射。
- `citizencode/frontend/admins/`：市列表、市管理员列表、新增弹窗展示和拦截 30 人上限。
- `memory/05-modules/citizencode/`：更新 CID 管理员数量规则、错误码和前后端文档。

## 验收标准

- 同一省同一市市管理员少于 30 人时可以新增。
- 同一省同一市市管理员达到 30 人时，后端拒绝新增并返回稳定错误码。
- 前端市管理员列表显示 `本市市管理员：x / 30`。
- 满 30 人的市禁用新增入口和新增弹窗里的市选项。
- 后端检查、测试、前端构建和残留扫描通过。

## 完成情况

- 已在 CID 后端新增 `MAX_CITY_ADMINS_PER_CITY = 30`，新增市管理员时按 `省份 + 市名` 统计同市 `CITY_ADMIN` 数量。
- 已在 `CREATE_CITY_ADMIN` 的 prepare/commit 校验链路中加入单市上限判断，达到 30 人时返回 `shi admin city limit reached`。
- 已新增稳定错误码 `CID_ADMIN_CITY_ADMIN_CITY_LIMIT_REACHED`，前端按 `ApiError.errorCode` 展示 `本市市管理员已满 30 人，不能继续新增`。
- 已在市列表卡片显示 `x / 30`，在市管理员列表显示 `本市市管理员：x / 30`，满员后禁用新增按钮。
- 已在新增市管理员弹窗中显示每个市的 `x/30`，满员城市选项不可选，当前市满员时确认按钮不可用。
- 已更新 CID 管理员体系、错误码、前后端目录文档，并清理旧的“数量不限/不设上限”和旧前端错误处理命名残留。

## 验证结果

- `cd citizencode/backend && cargo fmt --check` 通过。
- `cd citizencode/backend && cargo check` 通过。
- `cd citizencode/backend && cargo test` 通过，73 个测试全部通过。
- `cd citizencode/frontend && npm run build` 通过，仅保留 Vite chunk 体积提示。
- `git diff --check` 通过。
- `rg` 检查指定 CID 代码和文档范围，没有旧的“数量不限/不设上限”、旧 `formatAdminPubkeyConflict` 和旧“增删改查”表述残留。
