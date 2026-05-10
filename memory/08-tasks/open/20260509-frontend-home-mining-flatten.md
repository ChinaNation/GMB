# 2026-05-09 前端 home/home-node 与 mining/dashboard 冗余嵌套扁平化

## 任务需求

前端 `home/` 与 `mining/` 各自只剩唯一子目录，构成冗余嵌套。把两层目录扁平化为一层：

- `frontend/home/home-node/*` → `frontend/home/`
- `frontend/mining/dashboard/*` → `frontend/mining/`

home-node 冗余是 Step 3 把 `home/transaction/` 搬走后的副产物；mining/dashboard 冗余是历史遗留（network-overview 没出现在前端）。

后端 `src/home/` 和 `src/mining/` 都有多个平级子模块，**不动**。

## 影响范围

- `citizenchain/node/frontend/home/home-node/`（搬源全删）
- `citizenchain/node/frontend/home/`（落地）
- `citizenchain/node/frontend/mining/dashboard/`（搬源全删）
- `citizenchain/node/frontend/mining/`（落地）
- `citizenchain/node/frontend/app/App.tsx`（2 处外部引用）
- `citizenchain/node/frontend/settings/settings-panel/SettingsSection.tsx`（2 处外部引用）

## 风险点

- 纯前端目录扁平化，无业务代码变更，无后端 / runtime 变更。
- 内部相对路径深度变化：home-node/ 5 文件 + components/ 3 文件需调层。
- mining/dashboard/ 5 文件需调层。

## 执行状态

- [x] git mv `home/home-node/*` 到 `home/`，含 `components/` 子目录（5 文件 + 1 子目录）
- [x] git mv `mining/dashboard/*` 到 `mining/`（5 文件）
- [x] home 内部相对路径调整（`api.ts / HomeNodeSection.tsx`: `../../core/` → `../core/`；`components/IssuanceSection.tsx`: `../../../shared/` → `../../shared/`）
- [x] mining 内部相对路径调整（`api.ts / MiningDashboardSection.tsx / NetworkInlineSection.tsx`: `../../core/` → `../core/`）
- [x] `App.tsx` 2 处外部引用改写（`'../home/home-node'` → `'../home'`、`'../mining/dashboard'` → `'../mining'`）
- [x] `SettingsSection.tsx` 2 处外部引用改写（`'../../home/home-node/api'` → `'../../home/api'`、`/types`同改）
- [x] `npx tsc --noEmit` 通过（exit 0）
- [x] 残留扫描全零（仅一条注释引用后端 `src/mining/dashboard` 是事实陈述，保留）
