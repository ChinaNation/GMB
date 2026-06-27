# ADR-029 注册局并入区块链软件，去中心化每市自治节点

## 标题

把 CID 注册局系统并入 `citizenchain`，以"每市一个市注册局自治节点 + 一个联邦注册局节点"的去中心化形态运行。

## 背景

- 现状：`citizencode` 是独立 Axum + 中心 PostgreSQL + Redis 后端 + React 前端；`citizenchain/node` 是单机桌面节点（Tauri + 内嵌 substrate 节点），仅本机 RPC、无鉴权、单操作员。
- 目标：注册局全国一个联邦 + 每市一个市注册局；机构/个人身份只在市注册局注册。每个市注册局机房装一台节点软件 = 区块链节点 + 注册局服务；管理员在自己电脑用浏览器经内网操作同一节点。
- 代码核实：`citizencode/backend/scope/rules.rs` 证实联邦管理员按省锁定（`VisibleScope::federal_registry(province)`）、市管理员按省+市锁定；省级行政区与"联邦按省管各市注册局"必须保留。
- 仓库收敛为 3 个系统：`citizenchain`（含注册局）/ `citizenapp` / `citizenwallet`。

## 决策

1. 新建 workspace 成员 crate `citizenchain/registry`：`registry/src`（后端，迁自 `citizencode/backend`）+ `registry/frontend`（前端，迁自 `citizencode/frontend`）。
2. 进程模型：node 桌面端（Tauri）启动时拉起并托管 registry 进程；registry 经节点 RPC 读写链（复用 `chain_runtime.rs` 模式）；registry 对内网 TLS + 扫码鉴权托管 API + 前端。桌面 = 节点运维台（挖矿/设置/链状态），浏览器 = 注册局管理员（录入/查询/颁发），两者并存不冲突。
3. 数据两层：链上只放最小身份 + 链下承诺哈希；链下明细（证件照/章程/股东名册等大文件 + 结构化字段）存本市**内嵌 PostgreSQL + 本地/NAS 文件仓库**，文件哈希上链验真。
4. 选择性上链：身份可上链但非强制——仅本地注册（如只办身份+护照）不上链；绑定账户/选择上链才上链最小身份+承诺哈希。
5. 省级维度保留：行政区 省/市/镇 = china.sqlite 单源（ADR-021）；联邦管理员按省 scope 管各市注册局；记录保留 `province_code/city_code/town_code`；ShardedStore 省/市维度逻辑保留（单市节点物理只持本市一片）。
6. 去中心化鉴权：市注册局管理员公钥集合上链，仅联邦（按省）origin 可写；扫码登录比对链上集合放行；删除 passkey/设备口令登录。
7. 裁撤旧授信、旧二维码导入和旧状态导入；公民改为注册局直接录入并直接发护照。Redis 删→节点内本地限流；中心 PostgreSQL 删→每节点内嵌 PostgreSQL。
8. 归档与删除：`citizencode` 整目录删除；旧公民护照备份保留在 `docs/citizenpassport/` 存档备查。
9. 链改按链开发期口径重新创世（feedback_chain_dev_never_ask_migration），零残留（feedback_no_compatibility）。
10. 延后：公民上链粒度细化（逐人最小身份 vs 仅承诺/资格）在迁移整合完成后单独细化。

## 影响

- node 形态：从单机桌面端升级为"节点运维台 + 内网注册局服务"；大市（如香港 800万公民/500万公司/100管理员）按机房服务器 + 内嵌 PG + NAS + PG 备份/PITR + 温备部署。
- runtime：扩展 `admins`（联邦/市注册局管理员集合、联邦按省可写）与 `otherpallet/cid-system`（身份注册表 + 承诺哈希），需重新创世。
- 仓库：收敛为 3 系统；`citizencode` 删除、`citizenpassport` 归档 `docs/`。

## 备选方案

- 塞进 `node/src/registry`：否，臃肿且耦合 node 生命周期；注册局是整套子系统，独立 crate 更清晰。
- 维持中心化：否，用户明确要去中心化。
- 链下库用 SQLite：否，megacity 并发与备份需求用内嵌 PostgreSQL，且复用现有 PG 代码。

## 后续动作

分 6 步实施，见 `memory/08-tasks/open/20260626-registry-merge-0X-*.md`。
