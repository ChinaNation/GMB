# SFID 签名弹窗扫码按钮 loading 修复

## 任务目标

修复管理员完成 Passkey 验证后，冷钱包签名弹窗中的“开启扫码”按钮一直转圈且灰色不可操作的问题。

## 问题原因

签名弹窗把底层业务操作的 loading 状态传给扫码按钮，导致 prepare + Passkey 完成后，业务流程仍在等待冷钱包签名 Promise，扫码按钮被错误禁用。

## 完成内容

- 已拆分 `adminActionLoading` 与 `adminActionCommitLoading`。
- 已拆分 CPMS 安装码签发的 `cpmsBusy` 与 `securityCommitLoading`。
- 已拆分 CPMS 站点管理的 `busy` 与 `securityCommitLoading`。
- 签名弹窗刚打开时，“开启扫码”按钮不再使用底层业务 loading。
- 只有识别到签名回执并提交 `commitAdminAction` 时，扫码按钮才进入 loading。
- 已更新 SFID 前端文档中的签名弹窗 loading 边界。

## 验证

- `npm run build` 已通过。
- 残留扫描确认没有继续把 `adminActionLoading / cpmsBusy / busy` 传给签名弹窗扫码按钮。
