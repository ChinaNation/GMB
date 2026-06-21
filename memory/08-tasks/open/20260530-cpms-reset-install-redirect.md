# CPMS 重置后初始化路由修复

## 任务需求

- 修复 `./citizenpassport.sh --reset` 后访问前端根路径被错误强制跳转 `/login` 的问题。
- 重置清库后的系统未初始化状态必须优先进入 `/install`。
- 管理员登录态失效时仍要清理前端用户镜像，并由路由守卫决定是否进入 `/login`。

## 预计修改目录

- `citizenpassport/frontend`：CPMS 前端工程；涉及通用 HTTP 401 处理、认证上下文和路由守卫联动修复。
- `cpms`：CPMS 技术文档；同步说明初始化路由和 401 处理边界。
- `memory/05-modules/citizenpassport`：CPMS 长期模块文档；更新错误码和前端登录态处理规则。
- `memory/08-tasks`：任务卡目录；记录本次修复、验证和残留清理结果。

## 执行清单

- [x] 创建任务卡。
- [x] 修复全局 401 不应直接 `window.location.href = '/login'` 的问题。
- [x] 确保登录态失效仍能通知 `AuthProvider` 清理用户状态。
- [x] 更新文档说明当前唯一规则。
- [x] 运行前端构建和残留扫描。

## 验收标准

- 重置后访问 `/` 时，未初始化系统进入 `/install`。
- 已初始化但未登录时，受保护页面进入 `/login`。
- 401 只清理本地用户镜像，不绕过初始化状态判断。
- 前端构建通过，旧强制跳转残留被清理。

## 完成记录

- 2026-05-30：前端 HTTP 封装改为 401 只派发 `cpms-auth-expired` 事件，不再强制写入 `/login`。
- 2026-05-30：`AuthProvider` 监听登录态失效事件并清理用户状态，由根路由和 `ProtectedRoute` 决定页面去向。
- 2026-05-30：更新 CPMS 技术文档和错误码文档；`npm run build` 通过，浏览器验证访问 `/` 进入 `/install`。
