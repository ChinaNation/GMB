# CPMS CI 使用干净 Ubuntu 容器收集离线 deb

## 任务目标

修复 CPMS CI 在 `citizenpassport-ubuntu24-amd64.run` 打包阶段失败的问题。失败原因是打包脚本直接使用 GitHub runner 主机 apt 环境递归解析依赖，受到 runner 预装源和第三方源污染，解析出不可下载的虚拟包和 PGDG PostgreSQL 版本。

## 范围

- `citizenpassport/scripts/build_linux_host_installer.sh`：离线 deb 依赖闭包改为在官方 `ubuntu:24.04` Docker 容器内解析和下载。
- `citizenpassport/CITIZENPASSPORT_TECHNICAL.md`：补充 CI 打包阶段使用干净 Ubuntu 容器收集依赖的规则。
- `memory/08-tasks/index.md`：登记和归档任务卡。

## 验收标准

- 打包脚本不再读取 runner 主机 apt 源。
- 离线 deb 只从官方 `ubuntu:24.04` 容器环境解析。
- 虚拟包、无 candidate 包不会进入下载列表。
- 本地脚本语法检查通过；完整 `.run` 打包由 GitHub Actions 执行。

## 进度

- 2026-05-31：创建任务卡，开始修复。
- 2026-05-31：打包脚本已改为在官方 `ubuntu:24.04` 容器内解析和下载离线 deb 闭包。

## 完成摘要

- `build_linux_host_installer.sh` 不再使用 runner 主机 `apt-cache depends --recurse` 和 `sudo apt-get download`。
- 打包阶段强制要求 Docker，并在 `ubuntu:24.04` 容器内解析运行依赖、过滤无 candidate 的虚拟包、下载真实 deb。
- 容器下载出的 deb 会 `chown` 回宿主 runner 用户，避免后续打包和清理权限问题。
- 技术文档已写入“离线 deb 闭包必须在官方 Ubuntu 24.04 容器内解析”的规则。

## 验证

- `bash -n citizenpassport/scripts/build_linux_host_installer.sh`
- `git diff --check`
- 残留搜索确认旧的 `apt-cache depends --recurse`、`sudo apt-get download` 不再存在。
- 本机无 Docker，完整 `.run` 打包需由 GitHub Actions 运行验证。
