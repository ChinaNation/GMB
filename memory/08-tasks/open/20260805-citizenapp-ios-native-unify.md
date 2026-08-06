# CitizenApp iOS 原生库接入 + panic 策略修复 + gmb_ 前缀统一

- 状态: 开发完成,待真机验收(用户指定所有任务完成后一起测)
- 日期: 2026-08-05
- 模块: citizenapp(iOS/rust/chat)、citizenwallet(rust)、citizenchain/crates/citizen-signer
- 前置: 原生签名统一第 1 步(热端)、第 2 步(冷端)已完成并真机装机

## 背景

citizenapp iOS 从未接入原生库:`ios/` 无 Frameworks、Podfile 无本地 pod、pbxproj 无库引用。
`SmoldotPlatform.loadLibrary()` 在 iOS 兜底到 `DynamicLibrary.process()` 后找不到符号,
链 RPC / 原生签名 / MLS 聊天在 iOS 全部不可用。

顺带查出三项(用户已裁决):

- (甲)两端 `[profile.release] panic = "abort"` 使 `citizen-signer` 的 `catch_unwind`
  在 release 下完全失效,安全承诺落空 → **改 `panic = "unwind"`**
- (乙)`oslog = "0.2"` 是死依赖(iOS target 声明,零使用)→ **删除**
- (丙)`gmb_` 前缀未跟上 `citizen_` 统一命名 → **全仓统一,不再问**
  (例外:`GMB` 作为签名域/派生域常量是协议语义,禁改)

## 范围

1. `panic = "unwind"`:citizenapp/rust + citizenwallet/rust 两处 release profile
2. 删 citizenapp/rust 的 oslog 依赖
3. citizen-signer 清 5 条告警(unused import `Derivation` + 4×unsafe-code lint,FFI crate 源码级豁免)
4. 改名(零残留、零兼容):
   - FFI 符号 `gmb_chat_mls_*` → `citizen_chat_mls_*`(11 个;rust/chat_mls.rs + lib/chat/crypto/mls_native.dart + cbindgen 重新生成 native/smoldot.h)
   - wire 标签 `gmb_chat_envelope|gmb_chat_signal` → `citizen_chat_envelope|citizen_chat_signal`(cloudflare src+test、chat_runtime.dart;worker 需重新部署)
   - 测试临时目录前缀 `gmb_group_rt_|gmb_mls_purge_|gmb_mls_atomic_` → `citizen_*`
   - 测试夹具串 `gmb_test_*` → `citizen_test_*`(3 个 dart 测试)
   - 文档与登记簿:unified-naming.md(174 行)、unified-protocols.md、CHAT_GROUP_TECHNICAL.md 等
5. iOS 接入(照冷端已验证样板):
   - build-smoldot-native.sh:`build_ios()` 改产 `libsmoldot.a` → `ios/smoldot/`,并从 .a 自动抽符号清单(手写必漂移)
   - `ios/smoldot/smoldot_ffi.podspec`:`-force_load` + 逐符号 `-Wl,-u,<sym>`(三坑:release `-dead_strip` 静默剔除、CocoaPods 合并裸 `-u`、Mach-O 用 `nm -g` 非 `-D`)
   - Podfile 挂本地 pod;smoldotdart platform.dart iOS 显式走 `DynamicLibrary.process()`
6. 重建全部产物:热端 Android .so / host dylib / iOS .a;冷端 Android .so / iOS .a(panic 变更波及)
7. 真机:热端 iOS release 装机验证(链高度/交易/聊天);冷端重装(panic 变更)

## 执行中发现

- **iOS 推送权能挡签名**:`org.citizenapp` 带 `aps-environment` entitlement
  (firebase_messaging 真实现),但个人开发者团队(7QJXLLBA6J "wei cheng")不支持
  Push Notifications 权能 → 整个 App 无法出 profile。已暂摘该 entitlement
  (Runner.entitlements 内留恢复说明;APS_ENVIRONMENT 变量未删)。
  **零功能损失**:个人团队下 APNs token 本来就永远拿不到,聊天设备注册
  (`registerDevice` 硬要求 pushToken,iOS 侧 `readToken` 无 token 即抛)在该签名
  形态下从来不可能成功。聊天 iOS 端到端要等付费 Apple Developer Program;
  钱包/链/交易/原生签名不受影响。
- **构建脚本 pipefail 坑**:`llvm-nm` 对 .a 里个别无符号表对象报警并非零退出,
  管线输出其实完整;`set -o pipefail` 下把已成功的符号抽取判死。已在脚本内
  `|| true` 吞其退出码,完整性由三族计数把关。

## 验收(2026-08-05 全部达成)

- [x] `cargo check --target aarch64-apple-ios --release` 通过
- [x] iOS Release `Runner` 二进制 `llvm-nm -g` 数出 36 个 FFI 符号(21 smoldot + 4 citizen_sr25519 + 11 citizen_chat_mls),旧名 0——`-force_load` + 逐符号 `-Wl,-u` 扛住了 `-dead_strip`
- [x] 全仓 grep 旧名零命中(源码与技术文档;本任务卡自身作为改名记录例外)。补扫多语言时又统一了 4 个:`citizen_chat_ws_ready/pong`(WS 家族)与 Android Keystore 别名 `citizen_device_subkey_/citizen_device_data_key_`(丢别名即干净重生成,启动期 registrar 兜注册)。**保留不改**:`gmb_tx_hash/gmb_block_hash/gmb_extrinsic_index/topup_gmb_signer_mismatch`——此处 gmb 指 GMB 链/公民币结算语义(与签名域 `GMB` 同类官方名),且在充值发币安全不变量区+已部署 D1 表
- [x] citizen-signer 编译零告警(unused import 已删;FFI crate 源码级 `#![allow(unsafe_code)]` 豁免 workspace lint)
- [x] 测试:热端 MLS 2/2(=基线)+金标 3/3+夹具三文件 19/19;冷端全套 282/282;cloudflare worker 33 文件 269/269
- [x] 产物全部重建并验符号:热端 Android .so(61MB,36 符号 0 旧名)/host dylib/iOS .a(69MB→exported_symbols.txt 36 行);冷端三平台(各 4 符号)
- [x] 双 App 装机 iPhone 16 Pro:CitizenApp 88.6MB / CitizenWallet 24MB(均 release,冷端含 unwind 重建)
- [x] 文档:NATIVE_SIGNER_TECHNICAL.md(冷端实建+iOS 三坑+panic 前提)、unified-naming.md 174 行、unified-protocols.md、CHAT_GROUP_TECHNICAL.md、任务卡、memory

## 遗留(真机验收时)

1. CitizenApp iOS:开 App 验链高度自动刷新、发一笔交易(smoldot_* + citizen_sr25519_* 真机链路)
2. CitizenWallet iOS:创建/导入钱包、扫码签名
3. CitizenWallet / CitizenApp Android:Pixel 8a 连上后装新 .so 回归
4. **cloudflare worker 需重新部署**:chat wire 标签已改(`citizen_chat_envelope/signal/ws_ready/ws_pong`),已部署 worker 还说旧标签,新 App 聊天连不上直到重部署
5. iOS 聊天端到端受限:push entitlement 已摘(个人团队限制,见"执行中发现"),恢复须付费 Apple Developer Program
6. Android 设备子钥 Keystore 别名已改:旧安装首次启动会重新生成子钥并经启动期 registrar 重新登记(开发期零用户,无迁移)

## 基线

- MLS native session 测试改名前状态: **2/2 全绿**(host dylib 在位真跑,2026-08-05 13:2x)
