# CPMS 公民资料库

## 任务需求

- 公民档案详情页下半部分恢复为真实功能区，命名为“公民资料库”。
- 资料库用于显示和管理公民照片、出生纸/出生证明复印件、普通复印件、视频和其他资料。
- 主体业务必须放在 `cpms/backend/src/dangan` 下实现，不能放进 `operator_admin` 作为核心功能。
- 完成后更新文档、补中文注释、清理旧占位残留。

## 预计修改目录

- `cpms/backend/src/dangan`：新增资料库模块，负责元数据、文件存储、上传下载删除、硬删除联动。
- `cpms/backend/db`：新增 `archive_materials` 表结构，保存资料元数据，不保存文件正文。
- `cpms/backend/src/main.rs`：挂载 dangan 资料库路由并登记错误码。
- `cpms/frontend/operator_admin`：档案详情页新增“公民资料库”区域、资料上传、显示、下载和删除。
- `cpms/frontend/assets/styles`：新增资料卡片样式。
- `cpms/CPMS_TECHNICAL.md` 与 `memory/05-modules/cpms`：同步资料库边界、接口、存储和审计规则。
- `memory/08-tasks`：记录本次执行和验证结果。

## 执行清单

- [x] 创建任务卡。
- [x] 新建 `dangan/materials.rs`。
- [x] 新增资料库数据库表。
- [x] 挂载资料库接口。
- [x] 实现前端“公民资料库”区域。
- [x] 更新文档和错误码。
- [x] 运行构建、测试和残留扫描。

## 验证结果

- `cargo test --manifest-path cpms/backend/Cargo.toml`：通过，28 个测试通过。
- `cargo clippy --manifest-path cpms/backend/Cargo.toml --all-targets -- -D warnings`：通过。
- `npm run build`（`cpms/frontend`）：通过。
- `git diff --check`：通过。
- 浏览器烟测：使用本地 mock API 打开 `/admin/archives/mock-archive`，确认“公民资料库”、照片、出生纸资料和上传控件可渲染；同时修复窄屏详情页二维码压到公民信息的问题。
- 打印回归修复：窄屏响应式规则已限定为 `screen`，避免打印介质误用移动端上下布局导致二维码跑到第二页。
- 宽度回归修复：移除“公民资料库”独立 `max-width`，与上方“公民档案详情”使用同一卡片宽度。
- 残留扫描：未发现旧“出生纸待开发”、`CPMS_INSTALL_FILE`、`archive_wallet_challenges` 等残留；`full_name` 仅保留在技术文档“不再使用”说明中。

## 验收标准

- 档案详情页下半部分显示“公民资料库”。
- 可上传照片、出生证明、复印件、视频和其他资料。
- 图片和视频能在资料库中直接预览，其他文件可下载。
- 资料上传、下载、删除写入审计。
- 软删除档案不允许新增或删除资料；100 年硬删除时清理资料文件。
