# CitizenApp iOS 硬件金库与 SecureStorage 单源加固

状态：open

## 任务需求

- CitizenApp 全部 `FlutterSecureStorage` 使用统一加固实例，对齐 CitizenWallet 的 Android/iOS 选项。
- iOS 实现硬件绑定 seed 金库和 P-256 设备子钥桥，补齐 Android 已有能力。
- seed 解密必须由 Secure Enclave/Keychain 访问控制与 Face ID/Touch ID 原子绑定。
- 后台会话只使用无生物门禁的硬件 P-256 设备子钥，不得读取 seed 或弹生物识别。

## 已确认边界

- iOS Secure Enclave 使用 P-256；不得照搬 Android RSA 实现。
- sr25519 签名仍在软件层完成，硬件金库只负责信封解密，不能宣称“私钥永不出硬件”。
- 当前机器没有完整 Xcode、CocoaPods 或真实 iPhone；可先实现，但在真实 iPhone 验收前不得标记本任务完成。
- 每一步必须先提交技术方案，用户确认后才能执行。

## 预计修改目录

- `citizenapp/lib/security/`：全 App SecureStorage 唯一实例和选项。
- `citizenapp/lib/wallet/core/`：硬件金库与 P-256 子钥 Dart 边界。
- `citizenapp/ios/Runner/`、`citizenapp/ios/Runner.xcodeproj/`：Secure Enclave、LAContext、MethodChannel 和权限。
- `citizenapp/android/`：仅在跨平台契约需要统一时调整，不改既有安全语义。
- `citizenapp/test/`、`citizenapp/ios/RunnerTests/`：Dart、Swift 和通道测试。
- `memory/03-security/`、`memory/05-modules/citizenapp/`：威胁模型和平台能力文档。

## 主要风险

- Keychain accessibility 修改必须验证升级安装不会丢失或锁死已有密文。
- 生物录入变化、设备锁变更、App 重装和 iCloud 恢复必须有明确 fail-closed 行为。
- 模拟器不能证明 Secure Enclave 绑定；最终验收必须使用真实 iPhone。

## 完成标准

- CitizenApp 不再存在业务代码直接实例化裸 `FlutterSecureStorage()`。
- iOS 创建钱包静默封装，动钱动权每次触发生物识别，后台登录不弹窗。
- 换 Face ID/Touch ID、取消、锁定、重装和密钥失效路径均有测试及真机证据。

## 实施记录

### Step 1：SecureStorage 单源加固（2026-07-28）

- 新增 `lib/security/secure_storage.dart`，通用安全存储统一为
  `appSecureStorage`。
- Android 保持 RSA-OAEP + AES-GCM，显式启用算法迁移与崩溃恢复备份。
- iOS 收口为 `first_unlock_this_device`、`synchronizable=false`；本步没有把
  通用 Keychain 存储冒充 Secure Enclave 硬件金库。
- App 启动设备锁、设置页设备锁、PIN、attestation token、硬件金库密文 blob
  已切到统一入口；`SecureStorageBlobStore` 保留测试注入。
- 新增安全选项与 blob 注入测试；iOS 原生桥和真机验收仍属后续步骤。
- `flutter analyze --no-fatal-infos` 全量通过；`flutter test --concurrency=1`
  共 916 项通过、5 项按既有宿主能力条件跳过、0 项失败。
- Android debug APK 构建成功，并在 Pixel 8a（Android 16）通过 `install -r`
  覆盖安装；安装前后的三份 FlutterSecureStorage 文件摘要一致，
  `citizenapp.isar` 仍存在，证明本步未清空既有安全存储和本地数据库。
- Pixel 8a 分别在系统锁屏和解锁状态下完成冷启动，应用进程存活；解锁后确认
  既有钱包可进入交易页，“我的”页仍显示既有钱包，设置页成功读取设备锁与
  6 位 PIN 的既有关闭状态。再次强制停止并启动后钱包仍可读取，三份
  FlutterSecureStorage 文件摘要保持不变，全程未修改 PIN 或设备锁设置。
- 真机日志没有 SecureStorage 解密异常或崩溃；同时发现与本步无关的既有
  `广场登录态响应不完整` 异常，归广场会话任务处理，不混入安全存储改造。
