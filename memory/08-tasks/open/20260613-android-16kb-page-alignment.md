# 任务卡:Android 16KB 内存页对齐兼容(与公权机构功能解耦)

状态:**主因已解决(2026-06-13)**。逐 `.so` 实测(llvm-readelf LOAD 段对齐)纠正了弹窗噪音——**真正未对齐的只有 isar 一个**:libisar.so=0x1000(4KB);其余全部达标:libsmoldot.so/libbarhopper_v3/libsurface_util_jni/libimage_processing_util_jni/libdatastore_shared_counter=0x4000(16KB,NDK r28 默认对齐),libflutter.so=0x10000(64KB,16KB 的整数倍,满足)。弹窗里那堆"未知错误"是设备检测工具误报。
- **修法**:isar 3.1(上游停更、预编译 4KB)→ 换社区维护分支 `isar_community`/`isar_community_flutter_libs`/`isar_community_generator` ^3.3.2(沿用 isar 3.x API + DB 格式,**不迁移数据**;libisar.so 已 16KB,实测下载验证 0x4000)。
- **改动**:pubspec 3 行换包;19 文件 `package:isar/`→`package:isar_community/`(sed);`wallet_isar.dart::_resolveLocalIsarCorePath` 测试核心解析器 `isar_flutter_libs-`→`isar_community_flutter_libs-`(否则测试取旧 3.1 核心报版本不匹配);build_runner 用 isar_community_generator 重生。
- **验证**:flutter analyze 0(API 完全兼容,零业务改动);`flutter test --concurrency=1` **232/232 全过**。注意:并行模式 2 个 Isar 测试因多 isolate 抢同一 DB 失败(isar_community 锁更严),**ECC 规则本就要求 --concurrency=1**,CI 按此跑即绿。
- **待办**:`flutter clean` 重建让安装包用上新 16KB libisar.so;真机 16KB 页设备验证无警告。剩余(armv7/x86 等其他 ABI 已随社区包一并 16KB,无需单独处理)。

## 背景
- Android 15+ 支持 16KB 内存页(旧设备 4KB)。App 的原生 `.so` 必须 16KB 对齐才能在 16KB 页设备上加载;Google Play 对 targetSdk=Android 15 的应用自 2025-11 起要求 16KB 兼容。
- 现象:debug 版在 16KB 页设备/模拟器启动弹"应用兼容性 / ELF 对齐检查失败"。**仅 debuggable 版本弹,release 不弹**;普通 4KB 设备无影响。**非本功能 bug,纯第三方预编译 .so 未对齐。**
- 现状版本:AGP **8.11.1**(已够新,AGP 自建产物默认对齐)、compileSdk 36、Flutter **3.41.0**、ndkVersion=flutter 默认。→ 问题在**插件随包预编译的 .so**,不是 AGP/构建配置。

## 未对齐的库与来源(真机弹窗实测)
| .so | 来源插件 | 修法 | 难度 |
|---|---|---|---|
| `libsmoldot.so` | **smoldot-dart**(本地包,自有) | 用 NDK r27+ 重编 + 链接加 `-Wl,-z,max-page-size=16384` | 中(自己可控) |
| `libisar.so` | **isar 3.1.0**(isar_flutter_libs) | isar 3 上游已停更、预编译未对齐→换 isar 社区分支/v4 对齐版,或自编对齐 .so | **高(DB 迁移风险)** |
| `libbarhopper_v3.so` / `libimage_processing_util_jni.so` / `libsurface_util_jni.so` | **mobile_scanner 7.1.4**(MLKit + CameraX) | 升 mobile_scanner 到对齐版(Google 新版 MLKit/CameraX 已对齐) | 低 |
| `libdatastore_shared_counter.so` | androidx datastore(传递) | 升相关 androidx/插件版本 | 低 |
| `libflutter.so` / `libVkLayer_khronos_validation.so` | Flutter 引擎(后者 debug Vulkan 校验层) | 3.41 引擎大概率已对齐;校验层仅 debug,release 不含 | 低/确认即可 |

## 完工清单
- [ ] smoldot-dart:NDK r27+ 重编 `libsmoldot.so`,链接器加 `max-page-size=16384`,验证 `objdump -p libsmoldot.so` 的 LOAD 段 `align 2**14`。
- [ ] isar:评估迁移到 16KB 对齐的 isar 版本(社区分支/v4),或临时自编对齐 `libisar.so`;**评估 Isar schema/数据迁移影响**(全 app DB 依赖,风险最高,先出方案再动)。
- [ ] mobile_scanner / datastore:升到对齐版本,`flutter pub upgrade` 验证扫码功能不回归。
- [ ] 确认 Flutter 3.41 `libflutter.so` 已对齐;release 包不含 VkLayer 校验层。
- [ ] 全量原生库 16KB 对齐校验脚本(遍历 `*.so` 查 `p_align>=0x4000`)。

## 验收
- [ ] `flutter build apk --release` 后逐 `.so` 校验 16KB 对齐通过(或 AGP/`zipalign -P 16` 检查)。
- [ ] 16KB 页设备/模拟器(Android 15+)真机启动无对齐警告、扫码/DB/链节点功能正常。
- [ ] 4KB 设备回归正常。

## 不做(边界)
- 不动任何业务逻辑/公权机构功能;纯依赖与原生构建对齐。
- isar 迁移若风险大,先只出评估方案,迁移单列子卡。

## 改动目录(中文注释)
- `citizenapp/smoldot-dart/`(自有 .so 构建):NDK/链接器对齐参数,代码/构建。
- `citizenapp/pubspec.yaml`:mobile_scanner / datastore / isar 版本升级,依赖。
- `citizenapp/android/`(必要时):packaging/NDK 配置,构建。
- 新增对齐校验脚本 `tools/`。
