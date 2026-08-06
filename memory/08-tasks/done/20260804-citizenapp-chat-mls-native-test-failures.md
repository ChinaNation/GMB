# 修复 Chat MLS 原生测试既有失败：FFI 字段名断链 + 载荷断言过时

任务需求：宿主首次产出 `libsmoldot.dylib` 后，`test/chat/mls_native_test.dart` 与
`test/chat/mls_native_session_test.dart` 由长期 skip 转为真跑并暴露 2 个失败。
定位真实根因并修复，禁止用扩大 skip 掩盖；同步注释、测试与残留清理。

所属模块：citizenapp/lib/chat（crypto + flow）、citizenapp/rust/src/chat_mls.rs、
citizenapp/test/chat

输入文档：
- memory/05-modules/citizenapp/chat/
- memory/07-ai/audit-recipe.md
- memory/07-ai/definition-of-done.md

必须遵守：
- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通

## 背景：为什么失败被掩盖了这么久

`scripts/build-smoldot-native.sh` 只给 Android 交叉编译，宿主从不产出
`rust/target/release/libsmoldot.dylib`。`test/support/smoldot_native_probe.dart`
的守卫因此让这两个文件**永久 skip**——这不是守卫写错，而是守卫覆盖的失败一直
没人看见。2026-08-05 为验证原生 sr25519 签名，在宿主跑了
`cd rust && cargo build --release --lib`，库第一次存在，测试第一次真跑。

与本次 sr25519 签名改动无关：`rust/src/chat_mls.rs` 未被改动，本次只新增
`rust/src/signer.rs` 与 `lib.rs` 的 `mod signer;`。

## 问题清单与定性

| 编号 | 等级 | 问题 | 根因 |
|---|---|---|---|
| CRITICAL-1 | 严重 | `createKeyPackage` 返回的 `devicePublicKey` 恒为空串 | Rust 侧发 `device_public_key_hex`，Dart 侧读 `device_public_key`（`mls_native.dart:86`），全仓仅此一处漏 `_hex` 后缀 |
| LOW-1 | 低 | 直达测试断言 `plaintext == '瞬时直达'`，实际是载荷 JSON | 断言写于 2026-07-11，`ChatPayloadCodec` 2026-07-15 落地后 `plaintext` 改存载荷 JSON；该测试因 skip 未随之更新 |

### CRITICAL-1 不只是测试问题：它 hard-block 生产 Chat 首启

`chat_runtime.dart:1358-1363` 首次进 Chat 时：

```dart
freshKeyPackage = await crypto.createKeyPackage(identity);
if (keyPackage.devicePublicKey.isEmpty) {
  throw StateError('OpenMLS native 未返回 Chat 设备公钥，请先重编 native 库');
}
```

设备公钥恒空 ⇒ 全新安装每次都抛这条 `StateError`，Chat 起不来。错误文案
「请先重编 native 库」是当时的误判——库没问题，是 Dart 读错了键名。

连带失效的下游：
- `chat_runtime.dart:1543-1547` 的一致性守卫被 `isNotEmpty &&` 短路，形同虚设。
- `chat_cloud_transport.dart:70` `publishKeyPackage` 会把
  `device_public_key_hex: ''` 上传给 Worker，与 `registerDevice` 上传的值不一致。

### 命名口径判定

`device_public_key_hex` 为唯一正确名：Rust 发送端、
`chat_cloud_transport.dart:53/70/335` 收发两侧、
`test/chat/chat_cloud_transport_test.dart:169` 全部用它；且 FFI 全部十六进制字段
（`key_package_hex` / `plaintext_hex` / `wire_message_hex` / `ratchet_tree_hex` /
`state_key_hex`）一律带 `_hex` 后缀。改 Dart 读侧，不改 Rust。

### LOW-1 契约判定

`ChatMessage.plaintext` 存的就是 `ChatPayloadCodec` 载荷 JSON，由两处绿测与全部
消费端独立佐证：
- `test/chat/chat_envelope_session_test.dart:219` 断言落库 plaintext 含 `gmb.chat.msg`
- `chat_ui_adapter.dart:24`、`chat_page.dart:233`、`chat_search_page.dart:352`
  一律 `ChatPayloadCodec.decode(message.plaintext)`

故 `mls_native_session_test.dart` 的断言是过时的一侧，改为解码后断言
（同时断言 `kind`，比原断言更强），不是放宽。

输出物：
- 代码：`lib/chat/crypto/mls_native.dart` 读键名修正
- 测试：直达测试改断言载荷解码结果；新增 FFI 键名契约测试
- 中文注释：FFI 键名约定说明
- 文档更新：本卡 + 模块文档
- 残留清理：全仓核对无第二处 `device_public_key` 漏后缀

验收标准：
- 宿主有 `libsmoldot.dylib` 时两个文件全部真跑且通过（非 skip）
- 移走 dylib 后仍能干净 skip
- `flutter analyze` 无新增告警
- 文档已更新，残留已清理

## 执行结果（2026-08-04 完成）

| 项 | 改动 |
|---|---|
| CRITICAL-1 | `mls_native.dart:86` 读键名改 `device_public_key_hex`；Rust 侧不动 |
| 加固 | 新增 `_requireField`：`createKeyPackage` 必填字段缺失即抛，不再 `?? ''` 静默退化。可为空字段（`welcome_wire_message_hex` / Welcome 的 `plaintext_hex`）不受影响 |
| 回归钉 | Rust `creates_real_openmls_key_package` 增断言：`device_public_key_hex` 存在、非空、小写 hex |
| LOW-1 | 直达测试改 `ChatPayloadCodec.decode(...)` 后断言 `kind` + `text`（比原断言更强） |
| 注释 | `smoldot_native_probe.dart` 头注释订正：删掉「脚本只交叉编译 Android」的错误说法，写明 `./scripts/build-smoldot-native.sh macos` 可产宿主库，改 FFI 后必须真跑 |
| 文档 | `memory/05-modules/citizenapp/chat/CHAT_TECHNICAL.md` 补本条 |

### 验证

- 宿主有 dylib：`flutter test test/chat/mls_native_test.dart test/chat/mls_native_session_test.dart` → **4 通过 0 skip**
- 移走 dylib：同命令 → **4 全 skip**，无失败；dylib 已还原
- `flutter test test/chat/` → **203 通过 0 失败**
- `flutter test test/bootstrap_test.dart`（第三个受同一守卫门控的文件）→ 3 通过 1 skip，无失败
- `cargo test --release --lib chat_mls` → **11 通过**
- `flutter analyze`（改动文件）→ 无问题

### 附带发现（本次未动，待定）

1. `chat_mls.rs:573` 的响应字段 `created_new_session` 无任何消费方——Dart 用
   `welcomeMessage != null` 自行推导（`mls_session.dart:101`）。四闸已过（逐符号
   全仓 3 处均在产出侧、FFI 唯一 Dart 消费方是 `mls_native.dart`、非生成物、
   无 open 任务卡引用），属死字段，但与本次故障无关，未删。
2. `scripts/build-smoldot-native.sh` 的 `build_macos` 仍带
   `CARGO_PROFILE_RELEASE_STRIP=false` 与「release profile 的 strip=true 会让 dyld
   报 LINKEDIT 对齐错误」的注释，而 `rust/Cargo.toml:61` 现已是 `strip = false`，
   该 override 与注释均已过时（无害，未动）。
3. `rust/src/signer.rs` 有 `unused_imports: Derivation` 警告——属本次 sr25519
   原生签名在途改动，不在本卡范围。
