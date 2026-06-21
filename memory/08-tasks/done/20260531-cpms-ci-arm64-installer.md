# CPMS CI 增加 Ubuntu 24 arm64 离线安装包

## 任务目标

CPMS 正式离线安装包同时支持 Ubuntu Server 24.04 `amd64` 和 `arm64`：

- `citizenpassport-ubuntu24-amd64.run`
- `citizenpassport-ubuntu24-arm64.run`

push / pull_request 只执行 CI 编译与测试，不发布正式版安装包；只有手动 `workflow_dispatch`
运行成功后才上传正式版安装包产物。

## 范围

- `.github/workflows/citizenpassport-ci.yml`：增加 amd64 / arm64 matrix；手动运行才打包并上传正式安装包。
- `citizenpassport/scripts/build_linux_host_installer.sh`：增加 `--arch amd64|arm64`，按架构输出不同 `.run` 和 `.sha256`。
- `citizenpassport/deploy/linux/install_host.sh`：读取包内 `payload/manifest.env`，按包架构校验目标机。
- `citizenpassport/CITIZENPASSPORT_TECHNICAL.md` 与 CPMS 模块文档：更新双架构正式安装规则。

## 验收标准

- push 到 main 不上传正式安装包。
- 手动运行 CPMS CI 后上传 amd64 和 arm64 两个 artifact。
- 每个 artifact 包含 `.run` 和 `.sha256`。
- amd64 包只能安装到 Ubuntu 24.04 amd64，arm64 包只能安装到 Ubuntu 24.04 arm64。

## 进度

- 2026-05-31：创建任务卡，开始执行。
- 2026-05-31：完成打包脚本多架构参数、包内 manifest、安装脚本架构校验和 GitHub Actions matrix 改造。

## 完成摘要

- CPMS CI 改为 `amd64 / arm64` 双架构 matrix。
- push / pull_request 只执行编译与测试，不构建、不上传正式安装包。
- 手动 `workflow_dispatch` 才构建并上传正式版 artifact：
  - `citizenpassport-ubuntu24-amd64`
  - `citizenpassport-ubuntu24-arm64`
- 每个 artifact 包含对应 `.run` 和 `.run.sha256`。
- 安装包内新增 `payload/manifest.env`，安装脚本根据 `CPMS_PACKAGE_ARCH` 校验目标 Ubuntu 主机架构。

## 验证

- `bash -n citizenpassport/scripts/build_linux_host_installer.sh citizenpassport/deploy/linux/install_host.sh`
- `git diff --check`
- `npm run build`
- `cargo test --manifest-path citizenpassport/backend/Cargo.toml`
- 残留搜索确认旧的 amd64-only 安装脚本错误文案已清理。
