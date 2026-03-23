# 修复 wumin 安卓 release 构建并梳理 APK 打包前置条件

## 任务背景

用户希望将 `wumin` 冷钱包打包成安卓 `apk`，并确认打包前是否必须先到 GitHub 检查。

## 当前结论

- GitHub 检查不是本地打包 `apk` 的硬前置条件
- 当前 `wumin` 仓库可以直接尝试本地 `flutter build apk --release`
- 但当前安卓 `release` 仍使用 `debug` 签名，不适合正式分发
- 已修复 `isar_flutter_libs` 的旧版 Android 构建兼容问题，当前可成功产出测试 `apk`
- 当前产物路径：`wumin/build/app/outputs/flutter-apk/app-release.apk`

## 已知现象

- `wumin/android/app/build.gradle.kts` 当前使用 `compileSdk = flutter.compileSdkVersion`
- `wumin/pubspec.yaml` 依赖 `isar_flutter_libs: ^3.1.0+1`
- `isar_flutter_libs` 依赖内部仍写死 `compileSdkVersion 30`
- 初次 `flutter build apk --release` 在 `:isar_flutter_libs:verifyReleaseResources` 失败，报 `android:attr/lStar not found`

## 风险点

- 冷钱包无法稳定产出测试安装包，影响真机验证
- 若继续沿用 `debug` 签名，即使构建通过也不适合正式交付
- 若只修构建不梳理签名与版本流程，后续仍会重复踩坑

## 执行计划

1. 定位 `lStar` 资源错误的实际构建兼容性原因
2. 做最小 Android 构建修复并重新验证 `flutter build apk --release`
3. 输出当前 `apk` 打包结果、产物路径和正式发包前置条件

## 实际处理

- 在 `wumin/android/build.gradle.kts` 中为 `isar_flutter_libs` 增加 `afterEvaluate` 兼容补丁
- 保留 `namespace` 兜底
- 额外覆盖旧依赖写死的 `compileSdkVersion 30`，兼容不同 AGP 接口名

## 验证结果

- 执行：`cd /Users/rhett/GMB/wumin && flutter build apk --release`
- 结果：构建成功
- 产物：`/Users/rhett/GMB/wumin/build/app/outputs/flutter-apk/app-release.apk`
- 大小：约 `73MB`

## 剩余事项

- `wumin/android/app/build.gradle.kts` 的 `release` 仍使用 `debug` 签名，仅适合测试安装
- 如需正式分发，仍需补 `keystore`、`key.properties` 和 `release signingConfig`
