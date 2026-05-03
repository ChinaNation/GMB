# SFID 删除省管理员链功能顶层 Tab

- 创建时间:2026-05-02
- 状态:已完成

## 需求

省管理员登录后不再显示顶部 `省管理员名册`、`激活签名`、`rotate 签名` 三个 Tab。
注册局页面已经有省级管理员入口,省管理员相关链功能后续统一显示到注册局里,本任务不实现注册局新功能。

## 边界规则

- 只删除顶层 Tab 入口和 `App.tsx` 路由分支。
- 不删除 `sheng_admins/chain_*` 底层页面、API 和类型文件,作为后续注册局整合使用。
- 不新增注册局里的省管理员链功能 UI。
- 更新文档、补充中文注释、清理 App 未使用 import/type 分支。

## 预计修改目录

- `sfid/frontend/`
  - 中文注释:修改 App 顶层导航和路由分支,删除三个省管理员链功能顶层入口。
- `memory/05-modules/sfid/frontend/`
  - 中文注释:更新前端目录文档,说明这些链功能不再作为顶部 Tab 暴露,后续并入注册局。
- `memory/08-tasks/`
  - 中文注释:记录任务执行、构建和残留扫描结果。

## 验收

- 登录后顶部不再出现 `省管理员名册`、`激活签名`、`rotate 签名`。
- `App.tsx` 不再导入/渲染 `RosterPage`、`ActivationPage`、`RotatePage`。
- `npm run build` 通过。
- 文档、中文注释、残留清理完成。

## 执行记录

- 已删除 `sfid/frontend/App.tsx` 中三个省管理员链功能顶层 Tab。
- 已删除 `App.tsx` 中 `RosterPage`、`ActivationPage`、`RotatePage` 的 import、`ActiveView` 分支和渲染分支。
- 已保留 `sfid/frontend/sheng_admins/chain_*` 文件,等待后续统一并入注册局页面。
- 2026-05-02 后续任务 `20260502-sfid-sheng-admin-3slot-signer.md` 已把独立
  `chain_RosterPage.tsx`、`chain_ActivationPage.tsx`、`chain_RotatePage.tsx`
  删除,注册局页面改由 `SuperAdminSubTab.tsx` 统一承接。
- 2026-05-02 后续任务 `20260502-sfid-cpms-sheng目录整改.md` 已进一步删除
  省管理员前端旧 `chain_sheng_admins*.ts` 命名,改为 `roster_api.ts`、
  `signing_keys_api.ts`、`types.ts`。
- 已更新 `memory/05-modules/sfid/frontend/FRONTEND_LAYOUT.md`,补充这些链功能不再作为顶层 Tab 暴露的中文说明。
- 已运行 `npm run build`,构建通过;仅 Vite 输出既有 chunk 体积提示。
- 已扫描 `App.tsx` 残留,三个 Tab key/标签/import/渲染分支已清理。

- 状态：done

## 完成信息

- 完成时间：2026-05-02 15:09:40
- 完成摘要：已删除省管理员名册/激活签名/rotate签名三个顶层 Tab,后续注册局整合任务已删除底层独立页面文件,前端构建通过。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
