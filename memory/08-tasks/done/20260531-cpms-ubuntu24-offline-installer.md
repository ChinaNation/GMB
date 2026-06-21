# CPMS Ubuntu24 离线安装包与摄像头扫码收口

## 任务目标

按 `citizenpassport-ubuntu24-amd64.run` 目标形态收口 CPMS 主机安装链路：Ubuntu Server 24.04 LTS amd64 一包安装、完全离线、局域网通过 `https://www.citizenpassport.com/` 访问，客户端 DNS 由用户自行配置。

同时把 CPMS 前端所有二维码读取入口统一为浏览器摄像头扫码，删除其他非目标录入入口残留。

## 范围

- `citizenpassport/deploy/linux/`：离线依赖安装、nginx 反代、安装时证书生成、systemd、备份脚本。
- `citizenpassport/scripts/`：现有 Linux 主机打包脚本的 payload 布局同步。
- `citizenpassport/frontend/qr/`：摄像头扫码底层工具与统一扫码组件。
- `citizenpassport/frontend/login/`、`citizenpassport/frontend/initialize/`、`citizenpassport/frontend/super_admin/`、`citizenpassport/frontend/dangan/`：替换散落扫码 UI。
- `citizenpassport/CITIZENPASSPORT_TECHNICAL.md`、`memory/05-modules/citizenpassport/`：部署、资料目录、扫码规则文档同步。

## 验收标准

- 安装脚本不执行联网安装命令，不依赖外部 apt 源。
- 后端只监听 `127.0.0.1:8080`，局域网入口统一为 nginx HTTPS `https://www.citizenpassport.com/`。
- 安装时生成本机 CPMS Root CA 和 `www.citizenpassport.com` 服务端证书，并写入固定目录。
- 资料文件目录统一为 `/var/lib/citizenpassport/materials`，备份同时包含数据库、runtime、materials。
- 前端所有二维码读取入口统一使用摄像头组件；初始化页只保留摄像头扫码。
- 文档、注释、残留搜索与构建检查完成。

## 进度

- 2026-05-31：创建任务卡，开始执行。
- 2026-05-31：完成离线安装脚本、nginx、证书、systemd、备份脚本与摄像头扫码组件改造。
- 2026-05-31：完成文档同步、残留搜索和验证。

## 完成摘要

- CPMS 正式安装包收口为 `citizenpassport-ubuntu24-amd64.run`，GitHub Actions 使用 Ubuntu 24.04 runner 构建。
- 安装脚本只使用 payload 内置 deb，不联网安装依赖。
- 正式入口固定为 `https://www.citizenpassport.com/`，nginx 反代到 `127.0.0.1:8080`。
- 安装时生成本机 Root CA 和 `www.citizenpassport.com` 服务端证书。
- 公民资料库正式目录固定为 `/var/lib/citizenpassport/materials`，备份包含数据库、runtime 和 materials。
- 前端二维码读取统一走 `CameraQrScanner` 摄像头组件，初始化页只保留摄像头扫码入口。

## 验证

- `npm run build`
- `cargo test --manifest-path citizenpassport/backend/Cargo.toml`
- `bash -n citizenpassport/deploy/linux/install_host.sh citizenpassport/deploy/linux/backup_to_storage.sh citizenpassport/deploy/linux/certs/generate_citizenpassport_certs.sh citizenpassport/scripts/build_linux_host_installer.sh citizenpassport/deploy/linux/uninstall_host.sh citizenpassport/deploy/linux/install_backup_timer.sh`
- `git diff --check`
