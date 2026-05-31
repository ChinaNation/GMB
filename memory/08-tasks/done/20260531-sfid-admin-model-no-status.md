# SFID 管理员无状态与无主备模型修复任务卡

## 任务目标

按最终权限模型重做 SFID 管理员治理:

- 省级管理员不再区分主管理员、备用省级管理员。
- 代码内置初始省级管理员不可删除,且不能删除自己。
- 新增省级管理员可新增省级管理员,可管理所属省市级管理员,但不能删除省级管理员。
- 管理员不存在状态字段,前后端删除 `ACTIVE / DISABLED` 与停用/启用逻辑。
- 省级管理员和市级管理员表格操作只保留编辑、删除、更新密钥。
- 编辑只允许修改姓名。
- Passkey 拆成独立工具,只允许当前登录管理员更新自己的 Passkey。

## 固定约束

- 不保留主/备省管理员描述、类型、接口和 UI。
- 不保留管理员状态字段和停用/启用动作。
- 不新增二维码协议,继续使用 `WUMIN_QR_V1`。
- 不恢复旧 `shi_admins` 目录。
- 改代码后更新文档、完善中文注释、清理残留。

## 范围

- `sfid/backend/admins/`
- `sfid/backend/login/`
- `sfid/backend/models/`
- `sfid/backend/main.rs`
- `sfid/frontend/admins/`
- `sfid/frontend/auth/`
- `sfid/frontend/App.tsx`
- `memory/05-modules/sfid/`

## 执行记录

- 2026-05-31:创建任务卡,开始执行。
- 2026-05-31:后端删除管理员状态字段依赖,移除省级管理员主/备 roster 模块和路由。
- 2026-05-31:后端新增 `CREATE_SHENG_ADMIN / UPDATE_SHENG_ADMIN / DELETE_SHENG_ADMIN` 安全动作,市级管理员编辑收敛为只改姓名。
- 2026-05-31:Passkey 更新改为当前管理员本人替换式生成,完成后只保留一个有效凭据。
- 2026-05-31:前端注册局省级管理员页改为表格,市级管理员表格删除状态列和启用/停用按钮,新增独立 `AdminPasskeyTool.tsx`。
- 2026-05-31:登录态增加 `passkey_bound`,未绑定 Passkey 的管理员强制进入注册局管理员列表更新密钥。
- 2026-05-31:更新 SFID 架构、前后端布局、模型和链交互归属文档,清理旧 roster/主备/状态残留。
- 2026-05-31:验证通过 `npm run build`、`cargo check`、`cargo test`。
