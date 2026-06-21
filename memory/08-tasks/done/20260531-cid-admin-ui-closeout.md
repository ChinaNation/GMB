# CID 市管理员与机构列表 UI 收尾任务卡

## 任务目标

修复 CID 前端机构列表和市管理员管理界面的收尾问题:

- 公权机构、公安局列表不显示清算行资格列。
- 市管理员列表状态显示中文。
- 修改市管理员弹窗居中展示,去掉默认黄色提示图标,底部按钮居中。
- 修改市管理员时地址只读展示,只允许修改市归属和管理员姓名。
- 删除市管理员弹窗居中展示,去掉默认黄色提示图标,删除确认显示 SS58 地址,底部按钮居中。

## 固定约束

- 不兼容旧前端提交地址修改。
- 不恢复旧 `shi_admins` 目录。
- 不新增全局业务 API 目录。
- 改代码后更新文档、完善中文注释、清理残留。

## 范围

- `citizencode/frontend/institutions/InstitutionListTable.tsx`
- `citizencode/frontend/admins/FederalAdminsView.tsx`
- `citizencode/frontend/admins/ProvinceDetailView.tsx`
- `citizencode/backend/admins/actions.rs`
- `memory/05-modules/citizencode/frontend/FRONTEND_LAYOUT.md`

## 执行记录

- 2026-05-31:创建任务卡,开始执行。
- 2026-05-31:私权机构列表保留清算行资格列;公安局、公权机构列表不再展示该列。
- 2026-05-31:市管理员列表状态改为中文 Tag 展示。
- 2026-05-31:修改和删除市管理员确认弹窗居中展示,移除默认黄色提示图标,底部按钮居中。
- 2026-05-31:修改市管理员时账户地址改为 SS58 只读展示;前端提交只包含姓名和市归属。
- 2026-05-31:后端市管理员姓名修改拒绝 `admin_pubkey` 字段。
- 2026-05-31:更新 CID 前后端布局文档。
- 2026-05-31:验证:
  - `cd citizencode/backend && cargo fmt`
  - `cd citizencode/backend && cargo check && cargo test`
  - `cd citizencode/frontend && npm run build`
  - `git diff --check`
