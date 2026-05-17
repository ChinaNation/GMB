# wuminapp Android 正式签名

## 任务需求

- wuminapp Android release 构建不得再使用 debug 签名。
- 正式发布 APK 必须使用固定 release keystore，保证后续 Android 能按同一应用更新。
- push / PR CI 只做构建检查，不需要发布密钥。
- 手动 `Run workflow` 才注入 release keystore 并产出正式签名的 `公民.apk`。

## 修改边界

- Android Gradle：`wuminapp/android/app/build.gradle.kts`
- GitHub Actions：`.github/workflows/wuminapp-ci.yml`
- 密钥忽略规则：根 `.gitignore`
- 文档：wuminapp 技术文档与任务索引

## 验收标准

- `release` 构建在没有 release keystore 时直接失败，不再回退 debug 签名。
- 手动 workflow 使用 GitHub secret 写入临时 keystore，构建正式签名 APK。
- push / PR workflow 不要求 release keystore，只构建 debug APK 做检查。
- 文档说明 signing secrets、版本号递增和 Android 更新前提。
- 清理“release 沿用 debug 签名”的旧口径。

## 执行结果

- `wuminapp/android/app/build.gradle.kts` 已改为 release 专用签名配置，缺少 keystore 时 release 构建直接失败。
- `.github/workflows/wuminapp-ci.yml` 已拆分 push/PR Debug 构建与手动正式签名 APK 构建。
- `.gitignore` 已忽略 `wuminapp/android/key.properties` 与本地 Android keystore 文件。
- wuminapp 技术文档已补充 Android 正式签名、GitHub Secrets、版本号递增与更新前提。

## 验证记录

- `git diff --check` 通过。
- `ruby -e "require 'yaml'; YAML.load_file('.github/workflows/wuminapp-ci.yml')"` 通过。
- `JAVA_HOME="/Applications/Android Studio.app/Contents/jbr/Contents/Home" ./gradlew :app:assembleDebug --dry-run` 通过。
- `JAVA_HOME="/Applications/Android Studio.app/Contents/jbr/Contents/Home" ./gradlew :app:assembleRelease --dry-run` 在缺少 release keystore 时按预期失败，并输出新的中文签名配置错误。
