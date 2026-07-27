import 'dart:ffi';
import 'dart:io';

import 'package:isar/isar.dart';
import 'package:path_provider/path_provider.dart';

part 'wallet_isar.g.dart';

/// 钱包（master）：一套助记词 = 一个种子 = 一个 master。其下派生多个账户。
@collection
class WalletEntity {
  Id id = Isar.autoIncrement;

  @Index(unique: true, replace: true)
  late int walletIndex;

  late String walletName;

  /// 主种子指纹 = 账户0（根派生）的 accountId，唯一标识一套助记词。
  /// 同时用作 SecureStorage 里 master seed / 助记词的键（见 WalletSecureKeys）。
  @Index(unique: true, replace: true)
  late String masterId;

  late int createdAtMillis;
  late String source;

  /// 排列顺序（越小越靠前）。
  int sortOrder = 0;
}

/// 账户：钱包（master）下按派生序号展开的一对公私钥。
/// accountIndex 0 = 根派生(bare，逐字节等于历史直出)；N≥1 = `//N` 硬派生。
@collection
class AccountEntity {
  Id id = Isar.autoIncrement;

  /// 所属钱包（master）指纹。按此过滤取某钱包下全部账户。
  @Index()
  late String masterId;

  /// 派生序号：0=根派生，N≥1=//N。
  late int accountIndex;

  /// Substrate 账户唯一标识，小写 `0x` 加 64 位十六进制（= 派生公钥原字节）。
  @Index(unique: true, replace: true)
  late String accountId;

  /// SS58（前缀 2027），仅展示 / 二维码用，不作授权主键。
  @Index(unique: true, replace: true)
  late String ss58Address;

  /// 账户显示名，默认「账户$accountIndex」。
  late String accountName;

  late int createdAtMillis;
}

@collection
class AppKvEntity {
  Id id = Isar.autoIncrement;

  @Index(unique: true, replace: true)
  late String key;

  String? stringValue;
  int? intValue;
  bool? boolValue;
}

class WalletIsar {
  WalletIsar._();

  static final WalletIsar instance = WalletIsar._();

  Isar? _isar;
  Future<Isar>? _opening;
  Future<void>? _testCoreInit;

  Future<Isar> db() async {
    final current = _isar;
    if (current != null && current.isOpen) {
      return current;
    }

    final opening = _opening;
    if (opening != null) {
      return opening;
    }

    final task = _openFinalSchema();
    _opening = task;
    try {
      final opened = await task;
      _isar = opened;
      return opened;
    } finally {
      _opening = null;
    }
  }

  /// 只打开创世前最终 schema；旧业务库直接删除重建，不执行 migration。
  Future<Isar> _openFinalSchema() async {
    await ensureTestCoreInitialized();
    final dir = await _resolveDirectory();
    final schemas = [
      WalletEntitySchema,
      AccountEntitySchema,
      AppKvEntitySchema,
    ];
    final isar =
        await Isar.open(schemas, name: 'citizenwallet', directory: dir);
    return isar;
  }

  Future<void> ensureTestCoreInitialized() async {
    if (!_isFlutterTest()) {
      return;
    }

    final inflight = _testCoreInit;
    if (inflight != null) {
      return inflight;
    }

    final task = _initTestCoreInternal();
    _testCoreInit = task;
    try {
      await task;
    } finally {
      _testCoreInit = null;
    }
  }

  Future<void> _initTestCoreInternal() async {
    final localPath = _resolveLocalIsarCorePath();
    if (localPath == null) {
      throw StateError(
        'Flutter test 模式未找到 Isar Core 动态库，请先执行 flutter pub get。',
      );
    }
    await Isar.initializeIsarCore(
      libraries: <Abi, String>{Abi.current(): localPath},
    );
  }

  Future<void> resetForTest() async {
    if (!_isFlutterTest()) {
      return;
    }
    final current = _isar;
    if (current != null && current.isOpen) {
      await current.close(deleteFromDisk: true);
    }
    _isar = null;
    _opening = null;
  }

  Future<String> _resolveDirectory() async {
    if (_isFlutterTest()) {
      return Directory.systemTemp.path;
    }
    final appDir = await getApplicationSupportDirectory();
    return appDir.path;
  }

  bool _isFlutterTest() {
    return Platform.environment.containsKey('FLUTTER_TEST');
  }

  String? _resolveLocalIsarCorePath() {
    final fromEnv = Platform.environment['ISAR_CORE_LIB_PATH'];
    if (fromEnv != null && fromEnv.trim().isNotEmpty) {
      final file = File(fromEnv.trim());
      if (file.existsSync()) {
        return file.path;
      }
    }

    final home = Platform.environment['HOME'];
    if (home == null || home.isEmpty) {
      return null;
    }

    final hosted = Directory('$home/.pub-cache/hosted/pub.dev');
    if (!hosted.existsSync()) {
      return null;
    }

    final candidates = hosted
        .listSync(followLinks: false)
        .whereType<Directory>()
        .where((dir) => dir.path
            .split(Platform.pathSeparator)
            .last
            .startsWith('isar_flutter_libs-'))
        .toList(growable: false)
      ..sort((a, b) => b.path.compareTo(a.path));

    final relative = switch (Abi.current()) {
      Abi.macosArm64 || Abi.macosX64 => 'macos/libisar.dylib',
      Abi.linuxX64 => 'linux/libisar.so',
      Abi.windowsArm64 || Abi.windowsX64 => 'windows/isar.dll',
      _ => null,
    };
    if (relative == null) {
      return null;
    }

    for (final dir in candidates) {
      final path = '${dir.path}/$relative';
      if (File(path).existsSync()) {
        return path;
      }
    }
    return null;
  }

}
