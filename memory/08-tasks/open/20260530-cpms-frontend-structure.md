# CPMS 前端目录收敛与引用清理

## 任务需求

- 删除 CPMS 前端多余的 `web` 与 `src` 包装层，把 Vite 工程收口到 `citizenpassport/frontend`。
- 前后端模块命名尽量一致，前端目录按业务拆为初始化、登录、CPMS 机构管理员、operators、地址、QR、鉴权和通用模块。
- 拆分原单文件 API 与类型定义，删除不用的旧页面、旧接口和旧引用。
- 同步修复脚本、CI、文档和任务卡中写死的旧路径。

## 预计修改目录

- `citizenpassport/frontend`：CPMS 前端工程根目录；涉及代码迁移、模块重命名、API/type 拆分和残留删除。
- `citizenpassport/citizenpassport.sh`：CPMS 本地启动脚本；涉及前端启动目录硬编码修正。
- `.github/workflows`：CI 配置；涉及 CPMS 前端工作目录、缓存路径和构建产物路径修正。
- `cpms`：CPMS 技术文档；涉及当前前端目录结构说明更新。
- `memory/01-architecture`：架构与仓库地图；涉及 CPMS 前端结构说明更新。
- `memory/05-modules`：模块文档；涉及 CPMS 前端路径与职责说明同步。
- `memory/08-tasks`：任务卡与索引；涉及本任务登记和打开任务中的旧路径清理。

## 执行清单

- [x] 创建任务卡并登记索引。
- [x] 移动 CPMS 前端工程到 `citizenpassport/frontend` 根层。
- [x] 拆分 API 与类型文件。
- [x] 更新前端导入路径和 Vite/TS 入口配置。
- [x] 修正脚本、CI、文档、任务卡硬编码旧路径。
- [x] 删除不用文件和空目录。
- [x] 运行前端构建与残留扫描。

## 验收标准

- 旧 `web` 包装层不再作为工程目录存在。
- `citizenpassport/frontend/src` 不再存在。
- 前端不再引用原 `api.ts`、`types.ts` 聚合文件。
- CI、脚本和当前文档不再写死旧前端包装路径。
- `npm run build` 在 `citizenpassport/frontend` 可通过。

## 完成记录

- 2026-05-30：删除旧 `web` 与 `src` 包装层，Vite 工程收口到 `citizenpassport/frontend`。
- 2026-05-30：按业务模块拆分 `initialize / login / admins / operators / address / qr / authz / common`。
- 2026-05-30：删除前端旧聚合 `api.ts / types.ts` 和未挂载的旧公民状态页面。
- 2026-05-30：同步修正 `citizenpassport.sh`、CPMS CI、技术文档、长期记忆和打开任务卡中的旧路径。
- 2026-05-30：`npm run build` 通过；浏览器打开 `http://127.0.0.1:5175/` 可进入 CPMS 登录页，控制台无错误。
