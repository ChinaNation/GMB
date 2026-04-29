# Device Password 模块技术文档

## 0. 功能需求

- settings 与 home 中所有需要设备开机密码校验的操作，应复用同一套校验入口。
- 模块需要按操作系统分别调用本机认证能力完成非交互密码校验。
- 模块不能依赖 `USER/USERNAME` 环境变量判断当前用户身份，而应优先使用 OS API 获取当前登录账户。
- 模块需要对连续失败尝试进行限速和退避，降低暴力尝试风险。
- 模块需要把认证相关的本地状态单独落盘，避免和业务配置混杂。

## 1. 模块位置

- 路径：`node/src/settings/device-password/mod.rs`
- 对外接口：
  - `verify_device_login_password`

## 2. 模块职责

- 统一提供设备开机密码校验能力。
- 维护认证失败限速状态（`auth-rate-limit.json`）。
- 在 macOS / Linux / Windows 上分别接入本机认证机制。

## 3. 平台实现

- macOS
  - 通过 `geteuid + getpwuid_r` 读取当前系统账户。
  - 使用 `dscl -authonly` 做非交互密码校验。
- Linux
  - 通过 `geteuid + getpwuid_r` 读取当前系统账户。
  - 使用 PAM (`login/system-auth/common-auth`) 做认证。
- Windows
  - 通过 `GetUserNameExW/GetUserNameW` 读取当前系统账户。
  - 使用 `LogonUserW` 做交互式登录校验。

## 4. 安全规则

- 连续失败超过窗口阈值后拒绝继续尝试：滑动窗口 300 秒内最多允许 5 次失败，超限后返回限速错误。
- 失败退避在锁外执行，避免全局认证串行阻塞；初始退避 800ms，按超限次数线性增长，上限 5000ms。
- Linux PAM 交互密码缓冲区使用 `Zeroizing<Vec<u8>>` RAII 包装，确保 drop 时自动清零，即使发生 panic 或提前返回也不会泄露。
- Windows `LogonUserW` 传入的 UTF-16 密码缓冲区使用 `Zeroizing<Vec<u16>>` 包装，确保认证完成后自动清零。
- 返回前端的错误消息使用 `security::sanitize_path` 脱敏，仅保留文件名，不暴露完整路径。
