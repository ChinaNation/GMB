任务需求：
修复 wuminapp 初次安装启动后的基础权限策略：网络权限只做平台声明，通知权限通过首次启动说明后申请，相机与相册权限保留在用户触发扫码、选图、保存二维码等具体功能时申请，并同步文档说明。

所属模块：
wuminapp

输入文档：
- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/workflow.md
- memory/07-ai/unified-naming.md
- memory/01-architecture/wuminapp/WUMINAPP_TECHNICAL.md
- memory/07-ai/module-checklists/wuminapp.md
- memory/07-ai/module-definition-of-done/wuminapp.md

必须遵守：
- 不可突破 wuminapp 移动端交互入口边界。
- 不把网络权限设计成运行时弹窗；Android 网络权限只在主 manifest 声明。
- 通知权限仅用于用户可理解的提醒能力，拒绝后不得阻塞进入 App。
- 相机与相册权限按功能触发，不在首启强制索取。
- 代码修改后必须补中文注释、更新文档并清理残留。

预计修改目录：
- wuminapp/android/：补齐 release 主 manifest 的网络权限与 Android 13+ 通知权限声明，属于平台配置。
- wuminapp/ios/：补齐 iOS 通知权限用途说明，属于平台配置。
- wuminapp/lib/：新增首次启动权限说明与通知申请逻辑，属于 Flutter 交互代码。
- memory/01-architecture/wuminapp/：同步权限策略和平台差异说明，属于文档更新。

输出物：
- Android/iOS 权限配置
- Flutter 首次启动权限引导
- 中文注释
- 测试/静态检查结果
- 文档更新
- 残留清理

验收标准：
- 首次安装后进入 App 时展示一次权限说明。
- Android release 包具备网络权限声明。
- Android 13+ 和 iOS 通知权限可在说明后申请；拒绝不阻塞 App。
- 相机、相册仍由扫码/选图/保存二维码等具体功能触发。
- 静态检查通过或明确记录无法运行原因。
- 文档已更新，残留已清理。

新增命名说明：
- 中文名：App 权限启动策略；English name：app_permission_bootstrap；类型：Dart 文件；使用位置：`wuminapp/lib/security/app_permission_bootstrap.dart`；简介：记录首启权限说明状态并桥接原生通知权限申请。
- 中文名：App 权限入口页；English name：app_permission_gate；类型：Dart 文件；使用位置：`wuminapp/lib/security/app_permission_gate.dart`；简介：应用锁通过后展示一次权限说明，并按用户选择申请通知权限。
- 中文名：wuminapp 权限修复任务卡；English name：wuminapp-permissions；类型：任务卡；使用位置：`memory/08-tasks/open/20260517-wuminapp-permissions.md`；简介：记录本次权限策略修复范围、输出物和验收标准。

执行记录：
- 已补 Android release 主 manifest 的 `INTERNET` 与 `POST_NOTIFICATIONS` 权限声明。
- 已新增 Android/iOS 原生通知权限 MethodChannel：`org.chinanation.citizen/permissions`。
- 已新增 Flutter 首启权限说明页，网络只说明不申请，通知按用户选择申请，相机/相册保持功能触发。
- 已将权限入口接入 `_AppLockGate`，应用锁通过后只展示一次。
- 已更新 `memory/01-architecture/wuminapp/WUMINAPP_TECHNICAL.md` 的首次启动权限策略。

验证记录：
- `dart format lib/security/app_permission_bootstrap.dart lib/security/app_permission_gate.dart lib/main.dart test/widget_test.dart`：通过。
- `dart analyze lib test`：通过。
- `flutter test test/widget_test.dart`：通过。
- `flutter build apk --debug`：通过。
- `flutter build ios --debug --no-codesign`：被本机 CocoaPods 缺失阻断，未进入 iOS 编译阶段。
- `git diff --check`：通过。
- 残留扫描：未引入 `permission_handler`、临时文件或旧权限兼容分支；现有 `debugPrint` 与历史文档词条不属于本次新增残留。
