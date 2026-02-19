# wuminapp mobile (Flutter)

## Prerequisites
- Install Flutter SDK (stable)
- Run `flutter doctor` and complete iOS/Android toolchains

## Quick Start
```bash
cd /Users/rhett/GMB/wuminapp/backend
cargo run

cd /Users/rhett/GMB/wuminapp/mobile
flutter pub get
flutter run
```

## Wallet Note
- `我的`页面钱包流程使用 `sr25519 + SS58` 地址派生（Flutter 端本地生成/导入）。

## Test
```bash
flutter test
```
