# wuminapp Android 应用更新闭环

## 任务需求

- 手动 `Run workflow` 发布正式签名 `公民.apk` 后，同步生成 Android 更新清单并发布到 GitHub Release。
- `wuminapp` 启动后检查最新 GitHub Release，有新版本时在设置页“关于”区域的版本号前显示“更新”按钮。
- 用户点击“更新”后下载 APK、校验 SHA-256，并拉起 Android 系统安装器安装。
- push / PR CI 不发布 Release，不触发正式更新产物。

## 修改边界

- GitHub Actions：`.github/workflows/wuminapp-ci.yml`
- Flutter 更新模块：`wuminapp/lib/update/`
- 设置页接入：`wuminapp/lib/my/user/user.dart`
- 启动检查接入：`wuminapp/lib/main.dart`
- Android 安装通道：`wuminapp/android/app/src/main/`
- 文档：`memory/01-architecture/wuminapp/WUMINAPP_TECHNICAL.md` 与任务索引

## 验收标准

- 手动 workflow 生成 `公民.apk` 与 `wuminapp-android-update.json`，并上传到 GitHub Release。
- App 能读取本机 `versionCode`，只在远端 `version_code` 更高时显示更新。
- 下载 APK 后必须校验 SHA-256，不一致直接拒绝安装。
- 安装由 Android 系统安装器确认，不做静默安装。
- 文档、任务卡和残留扫描完成。

## 执行结果

- `.github/workflows/wuminapp-ci.yml` 已在手动发布路径生成 `wuminapp-android-update.json`，并上传 `公民.apk` 与更新清单到 GitHub Release。
- 移动端 Release 已显式设置为非 GitHub `Latest`，避免影响桌面端更新入口。
- `wuminapp/lib/update/` 已新增更新检查、清单校验、APK 下载、SHA-256 校验和安装状态管理。
- `wuminapp/lib/main.dart` 已在主界面启动后异步触发更新检查。
- `wuminapp/lib/my/user/user.dart` 已把硬编码版本改为真实版本显示，有新版时在版本号前显示“更新”按钮。
- Android 原生层已新增安装 APK 的 MethodChannel、`REQUEST_INSTALL_PACKAGES` 权限和 FileProvider。

## 验证记录

- `flutter analyze lib test` 通过。
- `ruby -e "require 'yaml'; YAML.load_file('.github/workflows/wuminapp-ci.yml')"` 通过。
- `JAVA_HOME="/Applications/Android Studio.app/Contents/jbr/Contents/Home" ./gradlew :app:assembleDebug --dry-run` 通过。
- `JAVA_HOME="/Applications/Android Studio.app/Contents/jbr/Contents/Home" ./gradlew :app:compileDebugKotlin` 通过。
