# SFID 权限分级收尾修复任务卡

## 任务目标

收口省市管理员权限分级第 2 步验收中发现的两个残留问题:

- 机构资料库文档列表、下载、删除接口必须强制校验机构存在和省市 scope。
- CPMS 禁用、吊销前端 API 必须在类型层面强制携带安全授权。

## 固定约束

- 不做旧流程兼容。
- 不新增二维码协议,继续使用 `WUMIN_QR_V1`。
- 不恢复旧目录、旧权限函数或旧前端 API 目录。
- 改代码后同步更新 SFID 文档、完善必要中文注释并清理残留。

## 范围

- `sfid/backend/institutions/handler.rs`
- `sfid/frontend/cpms/api.ts`
- `memory/05-modules/sfid/backend/institutions/INSTITUTIONS_TECHNICAL.md`
- `memory/05-modules/sfid/backend/cpms/CPMS_TECHNICAL.md`

## 执行记录

- 2026-05-31:创建任务卡,开始收尾修复。
- 2026-05-31:后端机构资料库列表、上传、下载、删除统一通过机构存在性和省/市 scope 前置校验。
- 2026-05-31:前端 CPMS 禁用、吊销 API 将安全授权改为必填参数,不再允许可选 grant header。
- 2026-05-31:更新 SFID institutions / CPMS 技术文档,记录收口规则。
- 2026-05-31:验证:
  - `cd sfid/backend && cargo fmt`
  - `cd sfid/backend && cargo check && cargo test`
  - `cd sfid/frontend && npm run build`
