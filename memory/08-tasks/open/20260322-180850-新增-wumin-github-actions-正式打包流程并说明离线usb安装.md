# 新增 wumin GitHub Actions 正式打包流程并说明离线 USB 安装

## 任务背景

用户希望：

- 为 `wumin` 配置 GitHub Actions 自动正式打包流程
- 明确如何将打好的 APK 安装到没有网络、已开启开发者模式并通过 USB 连接的手机

## 当前状态

- `wumin` 已支持本地正式签名打包
- 仓库当前只有 `wuminapp` 的 CI workflow，没有 `wumin` 的独立打包 workflow
- 用户已具备本地 keystore 和 `key.properties` 配置

## 风险点

- 若 workflow 直接依赖仓库中的本地签名文件，将导致私钥泄露风险
- 若 artifact 文件名不统一，用户下载后不易识别
- 若未说明 USB 安装路径，离线设备侧仍可能卡在安装环节

## 执行计划

1. 参考现有 workflow 风格新增 `wumin` 打包 workflow
2. 使用 GitHub Secrets 注入 keystore 与签名配置
3. 产出命名明确的 APK artifact
4. 给出离线手机通过 USB 安装 APK 的最短操作步骤

## 实际处理

- 新增 GitHub Actions workflow：`/Users/rhett/GMB/.github/workflows/wumin-release.yml`
- 支持 `push main` 和手动 `workflow_dispatch`
- 使用 GitHub Secrets 在 CI 中恢复 keystore 并生成 `key.properties`
- 自动执行 `flutter analyze`、`flutter test`、`flutter build apk --release`
- 构建后额外复制一份可读文件名产物：`公民钱包.apk`

## 需要的 Secrets

- `WUMIN_UPLOAD_KEYSTORE_BASE64`
- `WUMIN_KEYSTORE_PASSWORD`
- `WUMIN_KEY_PASSWORD`
- `WUMIN_KEY_ALIAS`

## 使用方式

- 在 GitHub 仓库配置上述 Secrets 后，推送到 `main` 即会自动打包
- 或在 GitHub Actions 页面手动运行 `Wumin Release APK`

## 离线 USB 安装

- 本机已存在 `adb`：`/Users/rhett/Library/Android/sdk/platform-tools/adb`
- 手机打开开发者模式并开启 USB 调试后，可直接通过 `adb install -r` 安装本地 APK
