# wuminapp smoldot Android ABI 修复

## 任务需求

- 修复 `armeabi-v7a` Android 手机上 `libsmoldot.so` 找不到，导致 wuminapp 轻节点无法启动的问题。
- Android 只支持 `arm64-v8a` 与 `armeabi-v7a` 两类真实手机 ABI。
- 修复后构建 APK，并安装到当前连接的 Android 手机。

## 修改边界

- `wuminapp/scripts/build-smoldot-native.sh`：Android smoldot native 编译从单一 `arm64-v8a` 扩展为 `arm64-v8a + armeabi-v7a`。
- `wuminapp/android/app/build.gradle.kts`：限制 Android APK 只打包 `arm64-v8a` 与 `armeabi-v7a`。
- `memory/01-architecture/wuminapp/` 与 `memory/05-modules/wuminapp/rpc/`：同步 Android ABI 与 smoldot native 打包口径。

## 验收标准

- `android/app/src/main/jniLibs/arm64-v8a/libsmoldot.so` 存在。
- `android/app/src/main/jniLibs/armeabi-v7a/libsmoldot.so` 存在。
- 构建出的 APK 只包含 `arm64-v8a` 与 `armeabi-v7a` 两个 ABI。
- 当前 `armeabi-v7a` 手机安装后不再报 `libsmoldot.so not found`。

## 执行结果

- `scripts/build-smoldot-native.sh android` 已改为同时编译 `aarch64-linux-android` 与 `armv7-linux-androideabi`。
- `android/app/build.gradle.kts` 已限制 APK 只支持 `arm64-v8a` 与 `armeabi-v7a`，并排除 x86/x86_64 native 库。
- `scripts/wuminapp-run.sh` 与 `.github/workflows/wuminapp-ci.yml` 的 APK 构建命令已统一加入 `--target-platform android-arm,android-arm64`。
- `memory/01-architecture/wuminapp/WUMINAPP_TECHNICAL.md` 与 `memory/05-modules/wuminapp/rpc/RPC_TECHNICAL.md` 已同步 Android ABI 与 smoldot native 打包规则。

## 验证记录

- `bash -n scripts/build-smoldot-native.sh scripts/wuminapp-run.sh` 通过。
- `./scripts/build-smoldot-native.sh android` 通过，并生成：
  - `android/app/src/main/jniLibs/arm64-v8a/libsmoldot.so`
  - `android/app/src/main/jniLibs/armeabi-v7a/libsmoldot.so`
- `flutter build apk --debug --target-platform android-arm,android-arm64` 通过，产物为 `build/app/outputs/flutter-apk/app-debug.apk`。
- `unzip` 检查确认 APK 只包含 `lib/arm64-v8a/` 与 `lib/armeabi-v7a/`，不包含 `lib/x86/` 或 `lib/x86_64/`。
- `adb install -r build/app/outputs/flutter-apk/app-debug.apk` 已成功安装到设备 `ZY22JBH3G6`。
- 启动 `org.chinanation.citizen/.MainActivity` 后，日志显示 smoldot 已连接节点并完成区块头同步，未再出现 `libsmoldot.so not found` 或 `MdbxError`。
- `flutter analyze lib test` 通过。
- `git diff --check` 通过。
