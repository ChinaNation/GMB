# 任务卡：修复 CPMS 正式安装数据库权限并加入离线安装 PDF 手册

## 任务需求

一次性修复 CPMS 离线安装后后端因数据库表权限不足导致 nginx 502 的问题；正式安装数据库结构只由后端 `MIGRATOR.run()` 创建，`schema.sql` 只保留给开发和测试使用；安装包内加入独立 PDF 操作手册，说明安装、DNS、证书信任、健康检查和常见问题。

## 修改范围

- `citizenpassport/deploy/linux/`：修复安装脚本中的数据库角色、数据库、schema、旧表 owner 权限处理，并安装 PDF 手册。
- `citizenpassport/scripts/`：调整安装包打包脚本，删除正式安装 SQL payload，增加 PDF 打包。
- `citizenpassport/docs/`：新增 CitizenPassport 安装配置手册 Markdown 源文件和 PDF 文件。
- `.github/workflows/`：让手动 CI 产物包含安装手册 PDF，push / PR 仍只跑编译测试。
- `citizenpassport/CITIZENPASSPORT_TECHNICAL.md`：更新正式安装数据库初始化、证书信任和 PDF 手册说明。
- `memory/01-architecture/citizenpassport/`、`memory/05-modules/citizenpassport/`：同步模块文档，清理正式安装导入 `schema.sql` 的旧描述。

## 完成内容

- 正式 `.run` payload 不再包含 `schema.sql / seed.sql`。
- `install_host.sh` 不再执行 SQL 文件导入；只创建 PostgreSQL 角色、数据库、schema 权限和环境文件。
- `install_host.sh` 会修正旧错误安装残留表和序列的 owner / 权限，避免 `permission denied for table system_install` 导致后端启动失败。
- 安装时停止旧 `citizenpassport-backend`，避免重装过程中旧服务继续占用数据库或旧二进制。
- 安装生成证书后，CPMS 主机自动信任本机 Root CA；局域网客户端仍按 PDF 手册导入。
- 安装手册复制到 `/opt/citizenpassport/docs/CitizenPassport安装配置手册.md`，手动 CI artifact 也包含该 Markdown 手册。

## 验证

- `bash -n citizenpassport/deploy/linux/install_host.sh citizenpassport/scripts/build_linux_host_installer.sh`：通过。
- `cargo test --manifest-path citizenpassport/backend/Cargo.toml`：通过，32 个测试全部通过。
- `npm run build`（`citizenpassport/frontend`）：通过。
- `git diff --check`（本次相关文件）：通过。
- PDF 解析检查：4 页，能提取中文关键内容。

## 状态

- 2026-05-31：完成。
