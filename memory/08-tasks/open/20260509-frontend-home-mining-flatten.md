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

## 2026-07-02 追加执行：删除挖矿页资源监控

- [x] 删除挖矿页“资源监控”分组下 4 个卡片：CPU 哈希率、GPU 哈希率、内存占用、节点数据大小。
- [x] 删除 `MiningDashboard.resources` 前端类型字段和默认状态，挖矿页不再读取或展示资源监控数据。
- [x] 删除后端 dashboard 的资源采样模型、TTL 缓存、进程内存采样、CPU/GPU 哈希率读取和节点数据目录递归统计逻辑。
- [x] 更新节点挖矿 dashboard 技术文档和节点总技术文档，明确挖矿页不再提供资源监控。
- [x] 本地节点数据已删除：`~/Library/Application Support/gmb.dev/chains/citizenchain`；未删除链上中国数据库 `registry-pgdata`。
- [x] 使用冻结 chainspec `citizenchain/node/chainspecs/citizenchain.raw.json` 启动本地节点验证，RPC `chain_getBlockHash(0)` 返回 `0x968c7eaf68a5f138fc1eef1dbe0f2b398274216d15d06805dc1d801904cad154`。
- [x] 启动日志显示远端 bootnodes 仍在旧 genesis（例如 `0xa353…0043`），与本机冻结创世 `0x968c…d154` 不同，当前不会形成同一网络。
- [x] 验证通过：`npm --prefix citizenchain/node/frontend run build`、`cargo check --manifest-path citizenchain/Cargo.toml -p node`、`git diff --check`。
