# 任务卡：OnChina 非注册局管理员卡片和本机构信息显示

## 当前状态

已完成，待人工复核后归档。

## 任务背景

OnChina 已经统一为多机构 `workspace` 工作台。注册局 UI 保持现状；国家司法院和其它非注册局机构进入自己的工作台后，本机构管理员列表不应再沿用注册局表格列结构。

当前非注册局显示页只展示机构简称、机构码和辖区，信息不够完整。需要补齐本机构信息显示，并把非注册局管理员列表改为更适合机构工作台的卡片墙。

## 任务目标

- 注册局管理员列表保持现有表格 UI。
- 非注册局机构的本机构管理员列表使用管理员卡片墙。
- 桌面端一行两张管理员卡片，小屏一行一张。
- 非注册局卡片布局不再显示“管理员信息 / 操作”两列表头。
- passkey 按钮文案改为“密钥”，移动到管理员卡片“余额”行右侧靠右。
- 显示页展示更完整的本机构信息。
- 完成后更新文档、完善注释、清理旧口径残留。

## 预计修改目录

- `citizenchain/onchina/frontend/admins/`
  - 用途：改管理员卡片组件和本机构管理员列表布局。
  - 边界：注册局默认表格不变；非注册局通过参数启用卡片布局。
  - 类型：前端代码修改。
- `citizenchain/onchina/frontend/workspace/`
  - 用途：司法院和通用机构显示页挂载卡片布局和本机构信息。
  - 边界：不改注册局业务 UI。
  - 类型：前端代码修改。
- `citizenchain/onchina/src/`
  - 用途：新增本机构信息只读接口或扩展现有管理端读取接口。
  - 边界：只读当前 active binding 对应机构，不新增权限真源。
  - 类型：后端代码修改。
- `memory/05-modules/citizenchain/onchina/`
  - 用途：更新前后端文档和验收口径。
  - 边界：只记录当前实现目标，不保留旧兼容口径。
  - 类型：文档修改。

## 验收标准

- 注册局管理员列表 UI 不变。
- 国家司法院管理员列表在桌面端一行两张卡片，小屏一列。
- 非注册局管理员列表无“管理员信息 / 操作”表头。
- passkey 按钮显示“密钥”，位于余额行右侧。
- 显示页展示完整机构信息。
- 后端编译、前端类型检查、前端构建通过。
- 使用真实本地服务和真实页面检查国家司法院工作台。

## 执行记录

- 后端新增 `GET /api/v1/admin/own-institution`，只从当前 active binding 读取本机构 `InstitutionDetailOutput`。
- 前端 `OwnInstitutionAdminsView` 新增 `layout='cards'` 非注册局布局；默认仍为表格，注册局不受影响。
- `AdminProfileCard` 新增 `actionPlacement='balance-row'`，用于把“密钥”按钮放到余额行右侧。
- 国家司法院和通用机构显示页改为“本机构信息 + 管理员卡片墙”。
- 前端文档、后端文档和统一协议已同步更新。

## 验收记录

- `cargo check --manifest-path citizenchain/onchina/Cargo.toml` 通过。
- `./node_modules/.bin/tsc -p tsconfig.json --noEmit --incremental false` 通过。
- `./node_modules/.bin/vite build --outDir /tmp/onchina-admin-cards-vite-build --emptyOutDir` 通过；仅保留 Vite 大 chunk 体积提示。
- `git diff --check` 通过。
- 真实本地服务 `http://127.0.0.1:8965` 通过国家司法院管理员会话验收：
  - `GET /api/v1/admin/own-institution` 返回国家司法院本机构详情和账户信息。
  - `GET /api/v1/admin/own-institution-admins` 返回国家司法院管理员列表。
  - 国家司法院工作台“显示”页展示完整本机构信息。
  - 国家司法院管理员列表桌面端一行两张管理员卡片，共 15 张。
  - 非注册局管理员列表不再渲染管理员表格列，卡片内 passkey 按钮文案为“密钥”，位置在余额行右侧。
