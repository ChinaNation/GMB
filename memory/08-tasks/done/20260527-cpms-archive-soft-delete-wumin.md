# CPMS 公民档案软删除与 wumin 签名确认

- 状态: done
- 日期: 2026-05-27
- 模块: CPMS 后端 / CPMS 前端 / wumin / 文档

## 需求

在 CPMS 公民档案详情页增加删除按钮。删除必须为软删除，并由当前登录管理员使用 wumin
扫码签名确认；删除成功后记录删除人和删除时间。档案码按钮统一调整为“更新 / 下载 / 打印”。

## 实现

- `cpms/backend/db/migrations/0009_archive_soft_delete.sql`
  - 增加 `archives.deleted_at / deleted_by / delete_reason`。
  - 增加 `archive_delete_challenges` 保存删除签名 challenge。
- `cpms/backend/src/operator_admin/mod.rs`
  - 新增 `delete/challenge` 和 `delete/complete`。
  - 校验 challenge 未过期、未消费、绑定当前档案和当前登录管理员。
  - 校验 wumin `sign_response` 的签名账户等于当前登录管理员账户。
  - 验证 payload hash 和 sr25519 签名后软删除档案。
  - 列表默认隐藏 `DELETED` 档案，已删除档案禁止编辑、更新、打印。
- `cpms/frontend/web/src/operator/ArchiveDetail.tsx`
  - 增加删除按钮和 wumin 签名弹窗。
  - 扫描 `sign_response` 后提交删除完成接口。
  - 档案码按钮改为“更新 / 下载 / 打印”。
- `wumin/lib/signer/payload_decoder.dart`
  - 增加 `CPMS_ARCHIVE_DELETE_V1` payload 识别，使 wumin 严格识别模型可以展示并签名删除档案请求。
- 文档更新
  - `cpms/CPMS_TECHNICAL.md`
  - `memory/05-modules/cpms/ARCHIVE_WALLET_PROOF.md`
  - `memory/05-modules/cpms/ERROR_CODES.md`

## 验证

- `cargo check --manifest-path cpms/backend/Cargo.toml`
- `npm run build` in `cpms/frontend/web`
- `dart analyze wumin/lib/signer/payload_decoder.dart`

## 清理

- 前端 `dist` 构建产物在最终检查后清理。
