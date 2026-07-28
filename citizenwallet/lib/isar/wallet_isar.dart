import 'dart:ffi';
import 'dart:io';
import 'dart:isolate';

import 'package:isar/isar.dart';
import 'package:path_provider/path_provider.dart';

part 'wallet_isar.g.dart';

/// 钱包（master）：一套助记词 = 一个种子 = 一个 master。其下派生多个账户。
@collection
class WalletEntity {
  Id id = Isar.autoIncrement;

  @Index(unique: true)
  late int walletIndex;

  late String walletName;

  /// 主指纹 = 账户0（`//0`）的 accountId，唯一标识一套助记词。
  /// 冷钱包按 master(此指纹)加密存 master 种子 + 助记词（见 WalletSecureKeys）;
  /// 账户不单独持久化密钥,签名/私钥导出时从种子按 accountIndex 现场派生。
  @Index(unique: true)
  late String masterId;

  late int createdAtMillis;
  late String source;

  /// 排列顺序（越小越靠前）。
  int sortOrder = 0;
}

/// 账户：钱包（master）下按派生序号展开的一对公私钥。
/// 全部 `//index` 硬派生（含账户0 = `//0`，无 bare 根）；每账户密钥独立、单向。
@collection
class AccountEntity {
  Id id = Isar.autoIncrement;

  /// 所属钱包（master）指纹。按此过滤取某钱包下全部账户。
  @Index(
    composite: [CompositeIndex('accountIndex')],
    unique: true,
  )
  late String masterId;

  /// 派生序号：N → `//N`（含账户0 = `//0`）。
  late int accountIndex;

  /// Substrate 账户唯一标识，小写 `0x` 加 64 位十六进制（= 派生公钥原字节）。
  @Index(unique: true)
  late String accountId;

  /// SS58（前缀 2027），仅展示 / 二维码用，不作授权主键。
  @Index(unique: true)
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
    final name = _isFlutterTest()
        ? 'citizenwallet_${Isolate.current.hashCode}'
        : 'citizenwallet';
    final isar = await Isar.open(schemas, name: name, directory: dir);
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

/// 已签 QR 请求持久化防重放仓库。
///
/// 请求 id 在到期前只能被一个签名流程原子占用；到期记录在每次占用时清理。
class SignedQrRequestStore {
  const SignedQrRequestStore._();

  static const String _keyPrefix = 'qr.signed_request.';

  static Future<bool> claim({
    required String requestId,
    required int expiresAt,
  }) async {
    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    if (expiresAt <= now) return false;

    final isar = await WalletIsar.instance.db();
    late bool claimed;
    await isar.writeTxn(() async {
      final records =
          await isar.appKvEntitys.filter().keyStartsWith(_keyPrefix).findAll();
      for (final record in records) {
        if ((record.intValue ?? 0) <= now) {
          await isar.appKvEntitys.delete(record.id);
        }
      }

      final key = '$_keyPrefix$requestId';
      final existing =
          await isar.appKvEntitys.filter().keyEqualTo(key).findFirst();
      if (existing != null) {
        claimed = false;
        return;
      }
      await isar.appKvEntitys.put(
        AppKvEntity()
          ..key = key
          ..intValue = expiresAt,
      );
      claimed = true;
    });
    return claimed;
  }

  /// 密钥调用前失败时释放占位；签名一旦成功则保留到请求到期。
  static Future<void> release(String requestId) async {
    final isar = await WalletIsar.instance.db();
    await isar.writeTxn(() async {
      await isar.appKvEntitys
          .filter()
          .keyEqualTo('$_keyPrefix$requestId')
          .deleteFirst();
    });
  }
}
