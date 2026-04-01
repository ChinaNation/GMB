# smoldot Dart 包本地 fork 说明

## 1. 目录用途

本目录保存 `wuminapp` 当前使用的 `smoldot` Dart FFI 包本地 fork。

目的：

- 不再依赖 pub.dev 上的只读包形态
- 后续可在本目录内扩展 typed capability 的 Dart 绑定
- 与 `wuminapp/rust` 和 `wuminapp/smoldot-pow` 一起纳入同一版本治理

## 2. 收编来源

- pub.dev 包名：`smoldot`
- 收编版本：`0.1.1`
- 原缓存来源：`/Users/rhett/.pub-cache/hosted/pub.dev/smoldot-0.1.1`

## 3. 当前关系

- `wuminapp/pubspec.yaml` 已改为 path 依赖本目录
- 本目录当前仍保持上游包结构，尚未开始 typed capability 改造
- App 实际 native 库构建仍以 `wuminapp/rust` 为准，不以本目录自带 `rust/` 为准

## 4. 后续规则

1. 对 Dart FFI 绑定的自定义改动，统一在本目录进行。
2. 若需要同步上游包版本，必须显式记录版本变化与冲突处理。
3. Typed capability 改造完成后，应进一步梳理本目录自带 `rust/`、`native/` 与 App 自有实现的关系，避免长期重复维护。
