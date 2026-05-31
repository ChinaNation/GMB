# SFID 签名弹窗层级统一修复

## 任务目标

修复 SFID 管理后台中 Passkey 与冷钱包签名弹窗被业务编辑弹窗遮挡的问题，并清理 CPMS 安装码签发、CPMS 站点管理中仍然手写的旧签名弹窗。

## 完成内容

- 新增 `sfid/frontend/common/modalStack.ts`，统一 SFID 前端业务弹窗、扫码账户弹窗、冷钱包签名弹窗的层级。
- `WuminSignatureModal` 默认使用最高安全层级，并禁止点遮罩或 ESC 误关。
- 市级管理员新增弹窗、省级管理员新增弹窗在签名流程进行中保持打开并禁用关闭入口。
- 市级管理员编辑/删除、省级管理员编辑/删除确认框保留在业务层，签名弹窗覆盖到最前。
- CPMS 安装码签发、CPMS 站点管理重要操作已改用统一 `WuminSignatureModal`，删除页面内手写旧签名弹窗。
- 更新 SFID 前端目录文档和 SFID 技术文档，固化“业务弹窗在底层，安全签名弹窗在最前”的规则。

## 验证

- 已运行 `npm run build`（`sfid/frontend`），通过。
- 已扫描 CPMS 安装码签发与 CPMS 站点管理页面，不再存在旧的 `securityScanner/securityVideoRef` 手写签名弹窗残留。

## 结论

已完成。后续管理员重要写操作统一按“业务弹窗保留在底层 -> Passkey 原生验证 -> 冷钱包签名弹窗最高层 -> 成功后收口业务弹窗”的顺序执行。
