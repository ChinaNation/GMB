# 公民钱包全面审计、扫码入口与产品手册设计

状态:completed(用户于 2026-07-27 确认全部修复、核心权限、彻底删除、P2/P3 清理及方案2)
所属模块:Mobile(citizenwallet 冷钱包)

## 用户需求

1. 全面审计公民钱包的字段统一、安全漏洞、残留遗漏和可改进项。
2. 后续在钱包详情页标题右上角增加与钱包列表一致的扫码入口,仍只扫描当前钱包。
3. 后续在设置页“安全”和“关于”之间增加“产品手册”入口。
4. 当前阶段先给产品手册 UI 设计,第一部分用流程图说明助记词、种子、私钥、公钥、
   AccountId 与 SS58 地址的关系。

## 执行边界

- 用户已确认实施全部审计修复、核心权限声明、彻底删除、7 项 P2、6 项 P3 清理、
  钱包详情扫码入口和产品手册方案2。
- 用户单独批准新增并纳入 Git 的文件只有:
  `citizenwallet/lib/ui/product_manual_page.dart`、
  `citizenwallet/test/ui/product_manual_page_test.dart`、
  `citizenwallet/design-qa.md`。
- 未修改 `citizenchain/runtime/`;未提交、推送或触发远端 CI。

## 审计范围

- 字段和命名:`AccountId/account_id`、`public_key`、`signer_public_key`、
  `ss58_address`、钱包/账户主键与 Dart/QR/Isar/文档对齐。
- 密钥安全:助记词、主种子、`//index` 子私钥、AEK、AES-256-GCM、
  SecureStorage、内存清理、生物识别、应用锁、防截屏和生命周期。
- 扫码签名:QR_V1 envelope、请求时效、防重放、action registry、字段中文展示、
  当前钱包限定、签名响应和拒绝状态。
- 数据可靠性:Isar 唯一索引、创建/删除原子性、重复导入、排序和密钥/业务行一致性。
- 平台与供应链:Android/iOS 权限、备份、导出面、依赖、构建和测试覆盖。
- UI/UX/可访问性:入口发现、层级、文案、触控目标、读屏语义、字号缩放和错误恢复。
- 残留:旧产品名、旧字段、旧协议、TODO/FIXME、死代码、未使用依赖和历史文档边界。

## 设计约束

- 继承 `citizenwallet/lib/ui/app_theme.dart` 的深色、冰蓝、圆角卡片设计。
- 手机画布按 390×844;不使用设备外框、系统状态栏或浏览器外壳。
- 产品手册保持单页纵向滚动、内容简短;第一部分优先展示密钥派生关系。
- 流程必须表达:
  `助记词 → 钱包种子 → //index 硬派生 → 账户私钥 → sr25519 公钥 →
  AccountId → SS58 地址(仅展示)`。
- 不展示真实密钥样例;敏感信息与公开信息必须有文字说明,不能只靠颜色区分。
- 生成三套独立视觉方案,用户选定前不进入 Flutter 实现。

## 预计修改目录

- `citizenwallet/lib/wallet/`:AEK、密文绑定、派生签名、原子创建与彻底删除;涉及安全代码
  和残留清理。
- `citizenwallet/lib/isar/`:唯一约束和签名请求持久化防重放;涉及存储代码与生成文件。
- `citizenwallet/lib/qr/`、`lib/signer/`、`lib/login/`:严格 QR 解析、时效、防重放与死字段
  清理;涉及协议消费代码。
- `citizenwallet/lib/ui/`:详情扫码入口、产品手册、签名页防截屏和可访问性;涉及 UI 代码。
- `citizenwallet/lib/security/`:安全存储升级配置、认证 API 与注释清理;涉及安全代码。
- `citizenwallet/android/`、`citizenwallet/ios/`:核心权限、最低平台版本和安全插件配置;
  涉及平台配置。
- `citizenwallet/test/`:安全、并发、严格解析、防重放、导航和产品手册回归测试;涉及测试。
- `citizenwallet/README.md`、`citizenwallet/design-qa.md`:更新安全边界和视觉验收;只涉及文档。
- `memory/08-tasks/open/`:记录授权、实施、验收和已知供应链边界;只涉及任务文档。

## 验收口径

- 正式 finding 必须包含严重级别、`文件:行号` 证据、影响、修复建议和边界依据。
- 历史任务卡只作线索,不得替代当前源码核验。
- 单列误判撤回与无法通过静态检查确认的限制。
- 三张视觉方案必须基于现有设计系统,并在对话中可见后等待用户选择。

## 进度

- 2026-07-27:首轮需求分析完成;用户确认创建任务卡,并要求当前只做审计、方案和 UI。
- 2026-07-27:完成当前源码静态审计、依赖检查、Flutter 全量测试、Android Debug 构建和
  QR 协议 registry/仓库守卫测试;未修改 `citizenwallet` 源码、测试或配置。
- 2026-07-27:已生成并在对话中展示三套 390×844 产品手册首屏方向:
  纵向步骤流、密钥地图、学习卡片。生成物位于 Codex 会话目录,不纳入仓库和 Git。

## 当前审计结论

### P1 高优先级

1. AEK 读取、格式或写入失败时会静默生成/使用新密钥:
   `citizenwallet/lib/wallet/secret_cipher.dart:100-129`。这会覆盖原 AEK 或生成仅当前
   会话有效的 AEK,导致既有种子和助记词密文永久不可解;首次并发初始化还缺少互斥。
   现有测试在 `test/wallet/secret_cipher_test.dart:13-20` 明确不覆盖持久化,并在并发
   测试前预热缓存,没有覆盖首次并发竞争。目标状态必须 fail-closed,只在“明确不存在”
   时创建一次,写入失败不得继续建钱包。
2. iOS 权限声明遗漏:
   `citizenwallet/ios/Runner/Info.plist:4-69` 没有 `NSCameraUsageDescription`、
   `NSPhotoLibraryUsageDescription`、`NSFaceIDUsageDescription`,但扫码、相册扫码和
   Face ID 均已使用。iOS 对应核心流程不可可靠运行。
3. 同一 QR 请求 id 的“一次密钥签名”只在单个页面实例内防重复:
   `lib/ui/offline_sign_page.dart:94-118`、`lib/ui/login_sign_page.dart:87-116`。
   退出页面后重扫同一未过期 `i` 可再次调用密钥,违反
   `memory/01-architecture/qr/qr-protocol-spec.md:138`。需要进程级、可持久化且按过期
   时间清理的已签请求记录,并在调用生物识别/私钥前原子占位。
4. 删除钱包先删 Isar 事实行,随后吞掉种子和助记词删除异常:
   `lib/wallet/wallet_manager.dart:301-323`。UI 会表现为已删除,但 SecureStorage 可能
   残留根机密。目标状态应可验证清理并把失败显式报告/进入可重试清理状态。

### P2 中优先级

1. QR 解析器未严格拒绝未知字段,且 `k` 接受数字字符串:
   `lib/qr/envelope.dart:55-99`、`lib/qr/qr_protocols.dart:35-40`、
   `lib/qr/bodies/sign_request_body.dart:45-74`、
   `lib/qr/bodies/sign_response_body.dart:32-46`;base64url 解析也接受带填充形式。
   这与协议 `qr-protocol-spec.md:17,41,77-78` 的严格类型、无填充和未知字段拒绝不符,
   会形成跨端解析歧义。
2. 登录页签名前只检查缓存倒计时和目标账户,没有用当前墙钟重新校验过期:
   `lib/ui/login_sign_page.dart:72-105`。App 后台冻结计时器时可对实际已过期请求调用
   私钥;应复用离线签名服务的签名前时效校验。
3. 登录响应二维码页未启用 `ScreenshotGuard`,而离线签名页从初始化即启用:
   `lib/ui/login_sign_page.dart:36-45,245-280` 对比
   `lib/ui/offline_sign_page.dart:46-58`。登录响应属于短期认证凭证,保护口径不一致。
4. sr25519 `KeyPair` 签名后不能可靠清零,仅等待 GC:
   `lib/wallet/wallet_manager.dart:434-443,459-492` 已在注释中承认当前依赖的
   `lock()` 不可用。冷钱包威胁模型下应评估直接使用可控字节缓冲的签名原语,并升级/
   修补依赖;仅清子种子副本不足以清除 `KeyPair` 内部私钥。
5. AES-GCM 未使用 AAD 绑定“钱包 masterId + 机密种类”,且读取助记词后未重新派生并
   核对 masterId:`lib/wallet/secret_cipher.dart:34-42,73-81`、
   `lib/wallet/wallet_manager.dart:615-623`。同类密文被交换时认证仍可通过;应绑定
   上下文并在展示助记词前校验归属。
6. Isar 唯一索引使用 `replace:true`,账户缺少 `(masterId, accountIndex)` 组合唯一:
   `lib/isar/wallet_isar.dart:14-23,38-51`;创建前查重和新增序号计算部分在事务外:
   `lib/wallet/wallet_manager.dart:187-205,234-283`。并发调用可把“冲突”变成静默替换,
   应改成冲突即失败并把读改写放入同一事务。
7. 依赖检查显示 11 个直接依赖被版本约束挡在可解析的新主版本前,另有
   `hashlib_codecs`、`js` 等已停止维护的传递依赖。升级必须按密码学金标、密钥清理、
   iOS/Android 权限和真实扫码回归分批验证,不能直接批量升级。

### P3 一致性、残留与 UX

1. 设置页文案称设备锁可使用“生物识别或设备密码”:
   `lib/ui/settings_page.dart:142-149`,实际 `biometricOnly:true`,应改为只写指纹/面容。
2. 应用锁注释称 SHA-256,实现实际为 100 万次 PBKDF2-HMAC-SHA256:
   `lib/security/app_lock_service.dart:11-14,127-167`。
3. QR envelope 的 `issuedAt` 从不进入线上字段,解析后也恒为 null:
   `lib/qr/envelope.dart:11-22,25-43,63-65,92-97`;`QrSigner` 的 default/max TTL 与
   clock-skew 常量未参与普通签名验证:`lib/signer/qr_signer.dart:38-45,204-213`。
4. `WalletSignResult.accountId` 与 `signerPublicKey` 始终写同一值,调用方只使用后者:
   `lib/wallet/wallet_manager.dart:82-94,445-456`,属于重复内部字段。
5. 扫码页“相册/手电筒”使用裸 `GestureDetector`,没有按钮语义和显式最小触控区域:
   `lib/ui/scan_page.dart:200-256`;账户详情的重命名和二维码入口也有同类问题:
   `lib/ui/account_detail_page.dart:286-329`。应改为 Material/语义按钮并保证至少 48×48。
6. `QrActions` 仍保留大量手写数值别名:
   `lib/qr/qr_protocols.dart:44-155`。生成 registry 和当前守卫均通过,功能未发现错误;
   但守卫只禁止手写中文表/反查表,没有校验这些别名值。建议把调用方别名也由 registry
   生成或给全部别名增加一致性守卫,降低日后漏同步风险。

### 已确认正确的边界

- `AccountId/account_id`、`public_key/signer_public_key`、`ss58_address` 语义当前一致:
  Dart 内部使用 camelCase,QR/审阅字段使用 snake_case;SS58 只用于展示和边界输入输出。
- 未发现被禁用的旧产品名、旧账户同义字段或额外扫码协议版本;唯一协议为 `QR_V1`。
- 助记词→32B 主种子→`//index` 硬派生账户私钥→sr25519 公钥/AccountId→SS58 展示地址
  的实现与金标测试一致。
- 创建、导入、读取根机密和签名前均强制 `biometricOnly:true`;账户目标与当前钱包
  `masterId` 不一致时拒签;未知/未完整中文解释的业务载荷为红色拒绝态。
- Android 禁止备份、使用 `FLAG_SECURE`,应用未声明 INTERNET 权限且源码未发现网络
  请求入口;密钥数据使用随机 IV 的 AES-256-GCM。

## 最终实施结果

### P1 与彻底删除

1. AEK 初始化改为进程内串行原子路径;只有安全存储明确返回 AEK 不存在,且 `readAll`
   再确认不存在任何钱包种子/助记词密文时才能生成。读取、格式、写入、回读校验失败
   全部失败关闭;解密路径永不创建 AEK。
2. Android 增加 `USE_BIOMETRIC`;iOS 增加相机、相册和 Face ID 用途声明;Android
   最低版本和 AppCompat 主题、iOS 最低版本同步到安全插件要求。
3. 离线签名与登录签名均在 Isar 中按请求 id 原子占位,按过期时间清理;重复请求在认证
   和私钥调用前拒绝。登录签名安全逻辑下沉到 `WalletManager` 的唯一 UTF-8 签名入口,
   不修改登录扫码签名页面 UI。
4. 删除钱包先删除并回读确认主种子与助记词,再原子删除全部账户和钱包事实行;最后一只
   钱包还会删除并回读确认 AEK。单账户删除的查找、计数和行删除位于同一事务。

### 7 项 P2

1. `QR_V1` envelope/body 严格拒绝未知字段、数字字符串 `k`、带填充或非规范
   base64url;过期边界统一为 `expiresAt <= now`。
2. 普通离线签名保留签名前墙钟校验;登录签名在服务层占位前和生物识别后分别按当前
   墙钟校验,即使认证跨越到期边界也不会读取根机密或签名,且不修改页面 UI。
3. 离线签名页继续使用 `ScreenshotGuard`;登录签名页的新增防截屏改造已按用户明确
   要求撤回。
4. sr25519 签名改为直接使用可控 `MiniSecretKey/SecretKey` 缓冲,完成后清零子种子、
   私钥、扩展私钥和 nonce,不再返回含私钥的 `KeyPair` 等待 GC。
5. AES-GCM 使用钱包机密存储键作关联数据,并在解密后重新派生账户0核对 `masterId`;
   跨钱包和跨机密类型交换均失败。
6. Isar 唯一索引取消 `replace:true`,新增 `(masterId, accountIndex)` 组合唯一;
   重复钱包和账户序号的查重、分配、派生和写入都位于同一事务。
7. 除扫码组件外的直接依赖升级到当前可解析版本并执行完整回归;扫码组件按用户要求
   严格保持任务前锁定的 `mobile_scanner 7.2.0`,避免版本变化影响相机预览外观。
   剩余停更 `js` 由 Isar 3.1.0+1 和旧 build_runner 工具链引入;移除它必须更换
   数据库/生成器,超出本任务模块边界,当前不进入运行时产物。

### 6 项 P3 清理与 UI

1. 设备锁文案与 `biometricOnly:true` 一致,只描述指纹/面容。
2. 应用锁注释修正为实际的 PBKDF2-HMAC-SHA256 100 万次派生。
3. 删除未上线的 `issuedAt`、无效 TTL/clock-skew 常量和重复
   `WalletSignResult.accountId`。
4. 账户详情改名/二维码入口补充 Material 触控目标和读屏语义。扫码页相册/手电筒
   保持原布局,不增加会改变扫码预览尺寸的 Material 外层。
5. `QrActions` 手写数字别名改为生成 registry 支撑的 getter,只保留一个 action 真源。
6. 清理旧 `bip39` 依赖和旧认证 API;助记词统一使用当前 `bip39_mnemonic` 校验与派生。
7. 钱包详情 AppBar 右侧复用列表同一 `scan-line.svg`,进入
   `ScanPage(wallet: 当前钱包)`。
8. 设置页在安全和关于之间增加产品手册入口;按用户选定的方案2实现五层密钥地图:
   `助记词 → 32B 主种子 → //index 硬派生私钥 → sr25519 公钥/AccountId
   → SS58 展示地址`。

## 最终验证结果

- `flutter analyze`:通过,0 issue。
- `flutter test`:通过,239 项（含登录签名服务层新增的 3 项安全回归）。
- `flutter build apk --debug`:通过,生成
  `citizenwallet/build/app/outputs/flutter-apk/app-debug.apk`。
- `cargo test -p qr-protocol`:通过,registry consistency 7 项、repo guard 1 项均通过。
- `plutil -lint ios/Runner/Info.plist`:通过。
- `flutter pub outdated --no-dev-dependencies`:全部直接依赖已是当前可解析最新版;仍有
  Isar/build_runner 带入的停更 `js` 和若干受上游约束的传递依赖。
- 产品手册 390×867 第二轮同屏设计 QA 无溢出、重叠、裁切和 P0/P1/P2 差异;
  `citizenwallet/design-qa.md` 最终结果为 `passed`。
- Android Debug APK 使用临时 `org.citizenwallet.qa` 包名在真实 Pixel 8a 上并行安装
  并成功启动 `MainActivity`;进程、Flutter 引擎、真实 Android KeyStore 和
  SecureStorage 算法迁移均正常,无崩溃。测试包随后已卸载,没有覆盖、读取或清除正式
  `org.citizenwallet` 的钱包数据。设备当时处于系统锁屏,因此扫码、生物识别和页面
  视觉走查仍是发布前人工验收项。API 36.1 隔离模拟器另有包管理器无法解析已注册
  Activity 的环境异常,不作为应用失败。
- 2026-07-28:根据用户现场反馈,精确撤回 `lib/ui/scan_page.dart` 中相册/手电筒按钮的
  `Semantics + Material + InkWell + 72×56` 布局改造;该文件已恢复到任务前版本。
  继续核对后把扫码依赖从升级后的 7.4.0 严格恢复为任务前锁定的 7.2.0;扫码页面、
  扫码遮罩和普通扫码签名页面均与任务前源码零差异。`flutter analyze`、扫码/详情相关
  Widget 测试及 Android Debug APK 构建通过。
- 2026-07-28:用户进一步明确要求还原扫码签名页面的任何修改。现已把登录签名页面和
  Android 日间/夜间窗口主题一并恢复到任务前版本;执行 `flutter clean` 后重新解析
  原扫码依赖并干净构建。扫码页、扫码遮罩、普通签名页、登录签名页及两份 Android
  主题文件相对任务前均为零差异。
- 2026-07-28:用户要求重新修复原问题并冻结扫码签名 UI。登录请求防重放与签名前墙钟
  校验改为在 `WalletManager.signUtf8ForAccount` 服务层实现:请求 id 在生物识别前
  原子占位,成功后保留到期,认证/签名失败释放;生物识别后再次校验到期时间。新增过期
  不调用认证、成功后拒绝重放、认证失败可重试三项回归测试。`scan_page.dart`、
  `scan_overlay.dart`、`offline_sign_page.dart`、`login_sign_page.dart` 和两份
  Android 窗口样式仍相对任务前零差异,未改任何扫码签名 UI。
- 2026-07-28:重新执行 `flutter analyze`（0 问题）、`flutter test`（239 项）、
  `cargo test -p qr-protocol`（registry 7 项、repo guard 1 项）、iOS plist 校验及
  Android Debug 构建。隔离包 `org.citizenwallet.qa` 已在真实 Pixel 8a 安装并启动,
  `MainActivity` 进入前台、Flutter/Isar/SecureStorage 初始化成功且无崩溃;随后已卸载
  隔离包并恢复正式包名,未覆盖或读取正式 `org.citizenwallet` 数据。最终正式 Debug
  APK SHA-256 为 `d46e6987fe01d31ebb4fa661300cd80072742537217d6a7d7faeb729e5148ebf`。
