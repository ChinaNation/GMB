// 测试隔离:每个测试文件(独立 isolate)用唯一临时目录开 Isar,从物理上根除跨文件
// 共享 `${systemTemp}/citizenapp.isar` 导致的并发锁竞争(30 秒超时)与磁盘残留污染。
//
// 打开真库的测试文件在 main() 顶部调一次 `useIsolatedIsar();` 即可,不再各自手写
// setUpAll(ensureTestCoreInitialized) / setUp / tearDown(resetForTest) 样板。

import 'dart:io';
import 'dart:typed_data';

import 'package:citizenapp/chat/storage/chat_crypto.dart';
import 'package:citizenapp/isar/app_isar.dart';
import 'package:citizenapp/security/local_data_key.dart';
import 'package:flutter_test/flutter_test.dart';

/// 聊天本地加密的测试密钥（固定 32 字节）。
///
/// 单测没有平台通道，真实路径会走
/// `WalletManager → 硬件金库 → flutter_secure_storage` 而抛 binding 错误；
/// 这里注入固定 CID 数据根，让 `ChatStore` 在测试中走**真实加解密**（不是绕过加密），
/// 只是密钥来源换成确定值。
final CidDataRoot debugChatCidDataRoot =
    CidDataRoot(Uint8List.fromList(List<int>.generate(32, (i) => i * 3 % 256)));

/// 为当前测试文件挂上隔离的 Isar 生命周期:
/// - setUpAll:建本文件专属临时目录 + 指向它 + 初始化 IsarCore + 注入聊天测试密钥
/// - setUp / tearDown:复位(防入 + 清出)
/// - tearDownAll:复位并删除临时目录
void useIsolatedIsar() {
  late Directory dir;
  setUpAll(() async {
    dir = Directory.systemTemp.createTempSync('citizenapp_test_');
    WalletIsar.debugTestDirectoryOverride = dir.path;
    ChatCrypto.debugFixedCidDataRoot = debugChatCidDataRoot;
    await WalletIsar.instance.ensureTestCoreInitialized();
  });
  setUp(() async {
    await WalletIsar.instance.resetForTest();
  });
  tearDown(() async {
    await WalletIsar.instance.resetForTest();
  });
  tearDownAll(() async {
    await WalletIsar.instance.resetForTest();
    WalletIsar.debugTestDirectoryOverride = null;
    ChatCrypto.debugFixedCidDataRoot = null;
    if (dir.existsSync()) {
      dir.deleteSync(recursive: true);
    }
  });
}
