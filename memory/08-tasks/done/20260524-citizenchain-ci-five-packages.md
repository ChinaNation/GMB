# 任务卡：CitizenChain CI 统一五个安装包

## 任务需求

- `push main` 只构建并上传 5 个用户安装包 artifact，不发布正式版、不通知更新、不部署服务器。
- GitHub 页面手动 `Run workflow` 构建同样 5 个用户安装包，并继续执行正式发布、updater 通知、Linux 服务器部署和旧 run 清理。
- 最终用户安装包名称固定为：
  - `公民链-macOS-Intel.dmg`
  - `公民链-macOS-apple.dmg`
  - `公民链-Windows.exe`
  - `公民链-Linux-amd.deb`
  - `公民链-Linux-arm.deb`

## 边界

- 暂时不做 macOS / Windows / Linux 系统级签名。
- 不删除 Tauri updater 签名、`citizenchain-latest.json`、GitHub Release、服务器部署链路。
- Linux 服务器部署只使用 `公民链-Linux-amd.deb`。

## 执行状态

- 状态：已完成

## 完成记录

- 已将 `citizenchain.yml` 的桌面端构建拆为 5 个 matrix 产物。
- 已保留手动发布时的 Tauri updater 签名、`citizenchain-latest.json`、GitHub Release 与 Linux 服务器部署链路。
- 已将 Linux 服务器部署固定为 `公民链-Linux-amd.deb`。
