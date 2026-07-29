# CitizenWallet Android 16 KB Isar clean cutover

状态：done（代码与产物完成；用户明确跳过真实 16 KB 内核运行验收，不记作通过）

## 任务需求

- 修复 CitizenWallet Android APK 中旧 `libisar.so` 的 4 KB ELF `LOAD` 段对齐问题。
- 从停止维护的 Isar 3.1 clean cutover 到仓库内 CitizenApp 已验证使用的
  `isar_community` 3.3.2，不保留旧依赖或兼容分支。
- 保持现有 CitizenWallet Isar 集合、字段、索引和业务数据可原地读取，不触碰助记词、
  seed、账户私钥或 secure storage 密钥生命周期。
- 完成自动化测试、真实 APK/AAB 产物检查、16 KB 运行环境验收、文档和残留清理。

## 已确认边界

- 只修改 CitizenWallet 本地存储依赖和直接消费点，不修改钱包派生、签名、二维码协议
  或安全存储模型。
- 不修改 `citizenchain/runtime/`。
- 不使用 `android:pageSizeCompat`、重新 zipalign 或压缩旧 `.so` 冒充 ELF 修复。
- 旧 Isar 兼容性数据库只允许写入批准的仓库外临时目录
  `/tmp/gmb-citizenwallet-isar31-compat/`，只含非敏感测试数据，验收后删除。
- 每一步执行后更新文档、完善中文注释和测试，并清理旧包名、生成物与临时数据。

## 预计修改目录

- `citizenwallet/pubspec.yaml`、`citizenwallet/pubspec.lock`：替换 Isar 依赖并清理旧包。
- `citizenwallet/lib/isar/`：更新 Isar import、平台兼容注释和现有 schema 生成物。
- `citizenwallet/lib/wallet/`：更新 Isar import，不改变钱包业务与密钥逻辑。
- `citizenwallet/test/wallet/`：在现有测试中增加 schema 指纹和旧库原地读取验证。
- `citizenwallet/android/`：预计不改代码，只验证当前 AGP、NDK 和打包结果。
- `memory/05-modules/citizenwallet/`：回写本地存储与 Android 16 KB 技术边界。
- `memory/08-tasks/open/`：记录实施过程、验收结果和未完成项。
- `citizenwallet/build/`、`citizenwallet/.dart_tool/`：仅生成 Git 忽略的构建产物，
  完成后清理无效残留。

## 已核实基线

- CitizenWallet 使用 AGP 8.11.1、Flutter 3.41.0 和 NDK 28.2.13676358。
- debug APK 的 16 KB ZIP 对齐检查通过。
- arm64-v8a 与 x86_64 中只有旧 `libisar.so` 的 ELF `LOAD` 对齐为 `0x1000`；
  其余相关原生库均为 `0x4000` 或 `0x10000`。
- 旧依赖为 `isar`、`isar_flutter_libs`、`isar_generator` 3.1.0+1。
- Isar 业务库名为 `citizenwallet`，现有集合为 `WalletEntity`、`AccountEntity`、
  `AppKvEntity`；安全存储不属于 Isar 业务库。

## 主要风险

- 生成器升级若改变集合、属性或索引 ID，可能导致旧业务库无法原地打开。
- 仅检查 APK ZIP 对齐不能证明 ELF 兼容，必须逐个核对 `LOAD` 段。
- 普通 4 KB Android 环境不能替代真实 16 KB 内核验收。
- 工作区存在其它任务的未提交改动，执行时不得覆盖或格式化无关文件。

## 完成标准

- 旧 Isar 三个包名及其 lock 记录全部清除。
- 新引擎可原地读取旧 3.1 测试库，集合、字段、索引、ID 和读写结果保持一致。
- CitizenWallet Analyze、全量测试、debug/profile/release 构建通过。
- APK/AAB ZIP 对齐通过，arm64-v8a 与 x86_64 所有 ELF `LOAD` 段达到
  `0x4000` 或更高。
- 在 `PAGE_SIZE=16384` 的真实设备或模拟器完成安装、冷启动和钱包数据库读写验收。
- 文档、中文注释、旧依赖和临时目录清理完成。

## 实施记录

- 依赖已从旧 `isar`、`isar_flutter_libs`、`isar_generator` 3.1.0+1 clean
  cutover 到 `isar_community`、`isar_community_flutter_libs`、
  `isar_community_generator` 3.3.2；`pubspec.lock` 和源码中旧包名、旧 import
  残留均为 0。
- `wallet_isar.dart` 与 `wallet_manager.dart` 已切换 community API。测试动态库解析
  只搜索 `isar_community_flutter_libs-*`，不回退旧包。
- 切换前使用旧 3.1 引擎在批准的临时目录写入三类非敏感测试记录；切换后 community
  3.3.2 直接打开同一数据库，钱包、账户、KV 记录及三个索引读取全部正确，继续写入和
  查询成功。测试没有生成或读取助记词、seed、账户私钥。
- 重新生成的 `wallet_isar.g.dart` 只有三个引擎版本字符串从 `3.1.0+1` 更新为
  `3.3.2`；Collection ID、字段和索引未变化。新增永久 schema 指纹测试钉死三个
  Collection ID、属性集合和索引集合。
- community 引擎会生成 `<name>.isar-lck`，旧 3.1 留下空
  `<name>.isar.lock`。启动路径现已幂等删除旧锁，保留 `<name>.isar` 业务库；
  自动测试覆盖存在和不存在旧锁的两次调用。Pixel 8a 覆盖安装后实查旧锁已消失，
  community 锁和业务库正常保留。
- 安全存储、`WalletSecureKeys`、`SecretCipher`、钱包派生、签名和二维码协议均无
  diff；`citizenchain/runtime/` 也未由本任务修改。
- 最终 `flutter analyze` 通过；CitizenWallet 全量 247 项测试全部通过。debug、
  profile、release APK 及 release AAB 均构建成功。
- 三个 APK 的 Build-Tools 36.1 `zipalign -c -P 16` 均通过。逐库检查
  arm64-v8a/x86_64 的 ELF `LOAD` 段，`0x1000` 失败数为 0；
  debug/profile/release 的六个 `libisar.so` 检查点均为 `0x4000`，其它原生库为
  `0x4000` 或 `0x10000`。
- release AAB 的 `BundleConfig.pb` 中
  `uncompress_native_libraries.enabled=1`、`alignment=2`；bundletool 1.18.1 枚举
  `2` 对应 `PAGE_ALIGNMENT_16K`。
- Pixel 8a（Android 16/API 36）已在不清除 App 数据的前提下覆盖安装最终 debug APK，
  community 3.3.2 成功打开既有 `citizenwallet.isar`，页面正常显示空钱包，进程无
  Flutter 崩溃、`dlopen` 或 Isar 打开失败。未创建或导入钱包，未改写安全材料。
- Pixel 8a 的真实 `PAGE_SIZE` 为 4096；现有 API 36.1 ARM64 AVD 启动后实测同样为
  4096。因此当前环境不能完成真实 16 KB 内核冷启动验收，任务保持 open，禁止用
  ZIP/ELF 静态检查或 4 KB 冷启动冒充该项完成。
- 批准的 `/tmp/gmb-citizenwallet-isar31-compat/` 非敏感兼容数据库和旧锁测试残留已
  删除；测试夹具没有进入仓库，Android App 和模拟器进程均已停止。

## 真实 16 KB 运行验收例外（2026-07-29）

- Pixel 8a 官方开发者选项存在“以 16KB 页面大小启动设备”，但系统真实弹窗要求先
  解锁 Bootloader；启用模式会清除全部用户数据，并需两次重启。恢复 4 KB 生产锁定
  状态还会再次恢复出厂设置。
- 用户随后明确要求跳过该测试，因此未解锁 Bootloader、未清除设备数据、未切换页面
  模式。设备复核保持 `PAGE_SIZE=4096`、`flash_locked=1`、
  `verifiedbootstate=green`，界面检查临时文件已删除。
- 本任务以依赖 clean cutover、旧库原地读写、全量自动化、APK/AAB 构建、ZIP 对齐、
  ELF `LOAD` 对齐和 4 KB Android 16 真机冷启动结果收口。真实
  `PAGE_SIZE=16384` 的安装、冷启动和数据库读写未执行，后续任何发布说明均不得写成
  “已通过真实 16 KB 内核运行验收”。
