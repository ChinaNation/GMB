# 任务卡:清理 wuminapp 工作空间与占位标记

## 任务需求

按用户要求直接修复：

- 删除 `wuminapp/Cargo.toml` 空 workspace。
- 为 `wuminapp/smoldot-dart/rust` 声明独立 workspace，避免继续向上归入仓库根 workspace。
- 删除 `wuminapp/smoldot-dart/pubspec.yaml` 中不匹配当前 path 依赖模式的 Dart workspace 解析声明。
- 删除点名的待办标记、模板占位和占位假阳性文字残留。

## 预计修改目录

- `wuminapp/`：删除空 Rust workspace 壳，避免子 Rust 包误判父 workspace；涉及配置清理。
- `wuminapp/smoldot-dart/`：清理 Dart 包 workspace 解析声明，并隔离自带 Rust FFI 包；涉及配置清理。
- `citizenchain/node/src/offchain/settlement/`：删除密钥加密待办标记并修正文档注释；涉及代码注释清理。
- `wuminapp/android/`：删除 Android Gradle 模板待办标记；涉及配置注释清理。
- `wumin/android/`：删除 Android Gradle 模板待办标记；涉及配置注释清理。
- `citizenchain/runtime/primitives/`：删除测试示例中的占位假阳性；涉及测试代码字面量清理。
- `memory/07-ai/`：删除审计模板中的占位假阳性；涉及文档模板清理。
- `memory/08-tasks/`：记录任务执行过程；涉及任务卡更新。

## 验收标准

- `wuminapp/smoldot-dart/rust` 不再因父 workspace 空成员报错。
- `dart pub get --directory wuminapp` 通过。
- 点名占位残留被删除或替换为非门禁触发表达。
- `git diff --check` 通过。

## 执行记录

- [x] 删除空 Rust workspace，并隔离 smoldot-dart 自带 Rust 包。
- [x] 清理 Dart workspace 声明。
- [x] 清理待办标记和占位假阳性。
- [x] 执行验证。

## 验证记录

- `cargo metadata --no-deps --format-version 1 --manifest-path wuminapp/smoldot-dart/rust/Cargo.toml`：通过。
- `cargo metadata --no-deps --format-version 1 --manifest-path wuminapp/rust/Cargo.toml`：通过。
- `dart pub get --directory wuminapp`：通过。
- `rg` 检查点名残留：无命中。
- `git diff --check`：通过。
