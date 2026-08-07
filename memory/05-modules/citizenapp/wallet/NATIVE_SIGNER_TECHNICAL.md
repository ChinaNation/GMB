# CitizenApp / CitizenWallet 原生 sr25519 签名（schnorrkel FFI）技术说明

> 全仓 sr25519 **唯一实现** = `citizenchain/crates/citizen-signer`（热端 CitizenApp 与
> 冷端 CitizenWallet 物理上共用这一份源码）。纯 Dart `package:sr25519` 已彻底移除，
> 禁止再引入第二套。

## 1. 为什么必须原生

纯 Dart `sr25519` 走 `BigInt` 软算标量乘，真机（OnePlus 6T）实测一次「派生 + 签名」
**8220 ms**：用户按完指纹要干等 8 秒，且该 CPU 曾留在 UI isolate 上，把主线程一次卡死
超过 Android ANR 阈值（5 s），发送交易必崩。

换成 schnorrkel（Substrate 全家官方实现）后，同一台设备实测 **14~18 ms**，约 500 倍。

| 阶段 | 纯 Dart | 原生 schnorrkel |
|------|---------|-----------------|
| 生物识别（交互，不可省） | ~2.3 s | ~2.3 s |
| **派生 + 签名** | **8220 ms** | **14~18 ms** |

热端原生库无需新增：`smoldotpow/lib` 早已依赖 schnorrkel（其源码注释还留着
`TODO: necessary for signing`），签名实现编进现成的 `libsmoldot`，**不新增第二个库**；
冷端永久离线、不需要链，单独编一个几百 KB 的纯签名库 `libcitizenwallet_signer`。

## 2. 口径（错一处，同一助记词会派生出另一个账户）

| 项 | 取值 | 对齐对象 |
|----|------|----------|
| 扩展模式 | `ExpansionMode::Ed25519` | Dart `MiniSecretKey.expandEd25519()` / Substrate `Pair::from_seed` |
| 签名上下文 | `b"substrate"` | Dart `Sr25519.sign` 的 `newSigningContext` |
| 硬派生 chaincode | **由 Dart 侧算好传入** | `SecretUri.fromStr('//index').junctions` |

junction 解析属 SCALE 编码而非密码学、不慢，且已被金标测试守着；留在 Dart 可把
**口径漂移面压到最小**——原生只接管慢的密码学部分。

## 3. 接口

核心实现在 `citizenchain/crates/citizen-signer`（rlib，含全部逻辑与单测）；FFI 外壳由
`export_citizen_signer_ffi!()` 宏在各消费方 cdylib **就地展开**——热端
`citizenapp/rust/src/lib.rs` 一行宏并入 `libsmoldot`，冷端 `citizenwallet/rust/src/lib.rs`
同一行宏出独立库。两端 Dart 绑定各自一份 `native_sr25519.dart`（热端复用
`SmoldotPlatform.loadLibrary()`，冷端自带平台分支），符号一致：

| 函数 | 语义 |
|------|------|
| `citizen_sr25519_derive_hard(seed32, cc32) → child32` | 一层硬派生；多层由调用方按 junction 顺序逐层调用 |
| `citizen_sr25519_public_key(child32) → public32` | child mini-secret → 公钥（= AccountId32） |
| `citizen_sr25519_sign(child32, msg) → sig64` | 签名 |
| `citizen_sr25519_verify(public32, sig64, msg) → i32` | 验签 |

## 4. 安全

- **私钥材料**：Rust 侧一律 `Zeroizing`（作用域结束即擦除；禁止手写清零，会被优化掉）；
  Dart 侧出入参缓冲在 `finally` 里**先清零再释放**。
- **绝不 panic 跨边界**：4 个入口全部 `catch_unwind`，内部异常只回错误码
  （`CITIZEN_SIGNER_ERR_*`）——原生 panic 会 `abort` 掉整个 App，代价与收益完全不成比例。
  **前提是两端 `[profile.release]` 必须 `panic = "unwind"`**：曾配成 `abort`，
  `catch_unwind` 在 release 下形同虚设、承诺落空，2026-08-05 已修正。未包
  `catch_unwind` 的 `smoldot_*` 入口不受影响——Rust 对 `extern "C"` 边界自带
  abort-on-unwind 兜底，行为与 `abort` 时代一致。
- **fail-closed**：空指针、长度非法一律拒绝；Dart 侧错误码一律上抛，
  **绝不静默兜底成"空签名"或"验签通过"**。
- **不改密钥来源**：仍从硬件金库 `readAccountKey` 取（生物识别、`biometricOnly` 不变）。
- **不需要 Isolate**：原生毫秒级，直接在调用线程完成；纯 Dart 时代为躲 ANR 而加的
  `Isolate.run` 已移除（换原生后它只剩开销）。

## 5. 验收（切换实现的安全闸门）

1. **金标测试** `test/wallet/derivation_golden_test.dart`：原生派生 + 原生公钥
   逐字节对拍 **Substrate 官方权威向量**（硬编码 `//Alice`、`//0 //1 //2` 的
   accountId / ss58 / childMiniSecret）——这是"生产实现 vs 外部真值"的直接对拍，
   比"两份 Dart 实现互证"更硬；
2. Rust 单测：派生确定性、签名→验签闭环、篡改必须验不过、空指针必拒；
3. 真机：交易正常签发并最终性上链，账户无漂移，`[Sign-Diag]` 显示 14~18 ms。

> 切换期间曾有一份 `native_sr25519_parity_test.dart` 做原生 ↔ 纯 Dart 对拍；原生真机
> 验收通过、纯 Dart 移除后该文件一并删除（不保留两套实现）。

## 6. 冷端（CitizenWallet）实建

第 2 步已完成：冷端切原生，与热端**共用 citizen-signer 同一份源码**。

- **存储模型不同、密码学相同**（有意为之）：热端「无根」只存 child mini-secret；
  冷端是持根方，只存母种子 + 助记词，签名时现场硬派生。crate 只提供密码学原语，
  不介入任何一端的存储决策。
- 冷热金标互等：两端 `derivation_golden_test.dart` 向量文件 md5 一致
  （`5be6c3f0…54db`），同一助记词两端逐字节同账户。
- 产物（`citizenwallet/scripts/build-signer-native.sh`，每次构建自动验 4 个符号）：

| 平台 | 产物 | 接入方式 |
|------|------|----------|
| Android | `android/app/src/main/jniLibs/arm64-v8a/libcitizenwallet_signer.so`（~2.1 MB） | jniLibs 自动打包 |
| iOS | `ios/signer/libcitizenwallet_signer.a`（~5.7 MB） | 本地 pod `citizenwallet_signer` 静态链入 |
| macOS(host) | `rust/target/release/libcitizenwallet_signer.dylib` | 仅 `flutter test` dlopen |

## 7. iOS 静态库三坑（冷热通用，热端 `ios/smoldot` 同方案）

1. **Release `-dead_strip` 静默剔符号**：`-force_load` 整库进来了，没被引用的
   `#[no_mangle]` 符号照样被剔——Debug 正常、Release 崩。必须**逐符号** `-Wl,-u,<符号>`
   钉住;热端符号清单从 `.a` 实抽（`exported_symbols.txt`），绝不手维护。
2. **CocoaPods 合并裸 `-u`**：写 `-u <符号>` 会被去重合并成 `-u a b c d`，
   后面的被链接器当文件名（`No such file or directory`）。必须写 `-Wl,-u,<符号>`。
3. **符号检查要查对文件**：Debug 查 `Runner.app/Runner.debug.dylib`（`Runner` 只是
   ~70KB 启动壳），Release 查 `Runner.app/Runner`;Mach-O 用 `llvm-nm -g`（`-D` 是
   ELF 专用，会误判成"没链接进去"）。

## 8. 现存边界

- `polkadart_keyring` 仍保留，**仅用于 SS58 地址编解码**（base58 + 校验和，非密码学），
  全 app 统一走它；`sr25519` 因此仍是它的传递依赖，但 `pubspec.yaml` 已移除直接依赖，
  **禁止再直接 import**。
- iOS 只出 device `arm64`（两端一致）；模拟器需 `aarch64-apple-ios-sim` 且同名架构
  不能 lipo 合并、要上 XCFramework——未做，真机验收足够。

## CI 宿主库与 skip 守卫

`flutter test` 跑在 CI runner 宿主上，Dart FFI 要 dlopen 宿主平台动态库；CitizenApp 的
`NativeSr25519` 经 smoldot 加载同一份 native 库，宿主无 `libsmoldot` 时整组用例必挂。

- 依赖原生签名的用例统一挂 `skip: smoldotNativeSkipReason()`（`test/support/smoldot_native_probe.dart`），
  库不可用时带原因跳过，可用时照常全跑；`test/wallet/wallet_manager_test.dart` 的
  「实际缺钥一次生成」「统一签名」「设备私钥失效 fail-closed」三组已按此接入。
- **skip 只代表这轮没验，不代表验过**：改动 FFI 两侧任一端后，必须先产宿主库
  （`./scripts/build-smoldot-native.sh macos`）再真跑，否则跨语言字段漂移会被静默掩盖。
- CitizenWallet 走的是另一条路径：它有独立的 `libcitizenwallet_signer`，CI 在
  `flutter test` 前用 `./scripts/build-signer-native.sh host` 真编宿主库，金标派生测试
  不 skip、每轮真跑。两个产品的原生库互不共用，不要互相套用结论。

### skip 守卫的覆盖范围

`test/wallet/` 下依赖原生派生的用例已全部接入守卫：

- `derivation_golden_test.dart`：`main()` 内所有用例统一取一次 `smoldotNativeSkipReason()`
  并逐个传 `skip:`（该文件没有 group 层）
- `wallet_multi_account_test.dart`：`WalletManager 多账户` group
- `wallet_manager_test.dart`：热钱包创建/导入/删除、实际缺钥一次生成、统一签名、
  设备私钥失效 fail-closed 四个 group

**验证方式**：本地临时移走 `citizenapp/rust/target/release/libsmoldot.dylib` 跑一遍
（等价 CI 宿主环境），确认全绿；再放回去跑一遍，确认用例真跑而不是被 skip 掩盖。
两种环境都过才算验完。

### iOS workspace 必须入库

`ios/Runner.xcworkspace/` 的 **本体必须进版本库**：`contents.xcworkspacedata` 与
`xcshareddata/{IDEWorkspaceChecks.plist,WorkspaceSettings.xcsettings}` 三个文件，
与 CitizenWallet 保持同一结构。

此前 `citizenapp/.gitignore` 整目录忽略了 `ios/Runner.xcworkspace/`，CI 检出后没有
workspace，`flutter build ios` 直接报 `Xcode workspace not found`（CitizenWallet 未忽略，
所以它的 iOS job 一直是绿的）。`.gitignore` 现只忽略各人本机的 `xcuserdata/`。
