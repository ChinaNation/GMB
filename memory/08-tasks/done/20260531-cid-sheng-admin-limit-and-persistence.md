# CID 联邦管理员上限与持久化成功后返回

## 任务目标

CID 只保留 `sheng_admin` 表述；每省最多 5 个联邦管理员（1 个内置初始联邦管理员 + 4 个后续新增联邦管理员）。新增联邦管理员必须在数据库持久化成功后才返回成功，避免前端显示成功但刷新不显示。

## 修改范围

- `citizencode/backend/admins/`：新增联邦管理员按省统计上限，达到 5 个时拒绝新增。
- `citizencode/backend/main.rs`：修正写操作持久化失败仍返回成功的问题，并补错误码映射。
- `citizencode/backend/db/migrations/`：清理 CID 旧 `super_admin` 表述与每省唯一约束，保留 `sheng_admin` 建模。
- `citizencode/frontend/admins/`：清理 `SuperAdmin` / `super-admin` 命名残留，按上限错误码展示中文提示。
- `memory/`：更新 CID 管理员模型、错误码、前后端目录文档并清理残留。

## 验收标准

- 同一省联邦管理员总数小于 5 时可以新增。
- 同一省联邦管理员总数达到 5 时，后端拒绝新增并返回稳定错误码。
- 数据库持久化失败时，接口不得返回业务成功。
- CID 当前代码和技术文档不再出现 `super_admin / SuperAdmin / super-admin / ADMIN` 残留。
- 后端检查、测试和前端构建通过。

## 完成情况

- 已将 CID 联邦管理员新增规则固定为每省最多 5 人，后端按省统计现有 `SHENG_ADMIN` 后再允许新增。
- 已移除 `federal_admin_scope.province_name` 唯一约束，改为普通索引，避免同省新增第 2 到第 5 个联邦管理员时数据库持久化失败。
- 已让管理员安全动作、登录态姓名修改在返回成功前显式执行 Store 持久化；持久化失败返回 `CID_STORE_PERSIST_FAILED`，不再出现“前端提示成功但刷新不显示”。
- 已将 CID 管理员前端残留的 `SuperAdminSubTab / super-admin` 改为 `FederalAdminSubTab / sheng-admin`，并在界面显示本省联邦管理员 `当前人数 / 5`。
- 已更新 CID 技术文档、前端目录文档、后端目录文档、错误码文档和迁移说明。

## 验证结果

- `cd citizencode/backend && cargo fmt --check` 通过。
- `cd citizencode/backend && cargo check` 通过。
- `cd citizencode/backend && cargo test` 通过，72 个测试全部通过。
- `cd citizencode/frontend && npm run build` 通过，仅保留 Vite chunk 体积提示。
- `rg` 检查 CID 后端、管理员前端、CID 技术文档，没有 `super_admin / SuperAdmin / super-admin / ADMIN / 006_super_admin_catalog / SuperAdminSubTab` 残留。
- `rg` 检查迁移目录，没有旧角色枚举、旧每省唯一约束和旧索引名残留。
