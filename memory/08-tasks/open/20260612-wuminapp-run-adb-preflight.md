# 任务卡:wuminapp-run.sh 启动前自清理(adb 重置 + 设备检测超时)

## 背景(已查实)
脚本卡在"编译 Rust 原生库"后的设备检测 `flutter devices --machine`→`adb devices` 永久阻塞。根因:用户曾用 `Ctrl+Z` 挂起构建,留下冻结的 adb 客户端把常驻 adb server(fork-server 守护进程,脱离终端)搞坏;关终端杀不掉守护进程,每次重跑新 `adb devices` 连同一个坏 server 死等。

## 方案(只改 wuminapp/scripts/wuminapp-run.sh)
1. chainspec 校验后、清 Rust 缓存前插入"启动前自清理":`pkill -f flutter_tools.snapshot` 清残留 + `adb kill-server && adb start-server` 重置守护进程(失败不阻断)。
2. `flutter devices --machine` 用 perl alarm 包一层 60s 超时(macOS 无 GNU timeout),卡住即快速回退而非无限等。

## 验收
- [ ] bash -n 语法通过
- [ ] 真机重跑不再卡设备检测(user 验证)
