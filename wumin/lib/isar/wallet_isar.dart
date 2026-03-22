import 'dart:ffi';
import 'dart:io';

import 'package:isar/isar.dart';
import 'package:path_provider/path_provider.dart';

part 'wallet_isar.g.dart';

@collection
class WalletProfileEntity {
  Id id = Isar.autoIncrement;

  @Index(unique: true, replace: true)
  late int walletIndex;

  late String walletName;
  late String walletIcon;
  late double balance;

  @Index(unique: true, replace: true)
  late String address;

  @Index(unique: true, replace: true)
  late String pubkeyHex;

  late String alg;
  late int ss58;
  late int createdAtMillis;
  late String source;

  /// 签名模式：固定 `local`（wumin 只有热钱包）。
  late String signMode;

  /// 所属分组名称，逗号分隔，如 '分组一,分组二'。
  /// '全部' 是虚拟分组，不存储在此字段中。
  String groupNames = '';
}

@collection
class WalletGroupEntity {
  Id id = Isar.autoIncrement;

  @Index(unique: true, replace: true)
  late String name;

  /// 排列顺序（越小越靠前）。
  int sortOrder = 0;

  /// 是否为默认分组（全部/分组一/分组二），不可删除。
  bool isDefault = false;
}

@collection
class WalletSettingsEntity {
  Id id = 0;

  int? activeWalletIndex;
  int updatedAtMillis = 0;
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

    final task = _openAndMigrate();
    _opening = task;
    try {
      final opened = await task;
      _isar = opened;
      return opened;
    } finally {
      _opening = null;
    }
  }

  Future<Isar> _openAndMigrate() async {
    await ensureTestCoreInitialized();
    final dir = await _resolveDirectory();
    final schemas = [
      WalletProfileEntitySchema,
      WalletSettingsEntitySchema,
      AppKvEntitySchema,
      WalletGroupEntitySchema,
    ];
    final isar =
        await Isar.open(schemas, name: 'wumin_wallet', directory: dir);
    await _ensureSettingsRow(isar);
    await _ensureDefaultGroups(isar);
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

  static const List<String> _defaultGroupNames = ['全部', '分组一', '分组二'];

  static Future<void> _ensureDefaultGroups(Isar isar) async {
    final count = await isar.walletGroupEntitys.count();
    if (count > 0) return;
    await isar.writeTxn(() async {
      for (var i = 0; i < _defaultGroupNames.length; i++) {
        await isar.walletGroupEntitys.put(
          WalletGroupEntity()
            ..name = _defaultGroupNames[i]
            ..sortOrder = i
            ..isDefault = true,
        );
      }
    });
  }

  static Future<void> _ensureSettingsRow(Isar isar) async {
    final settings = await isar.walletSettingsEntitys.get(0);
    if (settings != null) {
      return;
    }
    await isar.writeTxn(() async {
      await isar.walletSettingsEntitys.put(
        WalletSettingsEntity()
          ..id = 0
          ..updatedAtMillis = DateTime.now().millisecondsSinceEpoch,
      );
    });
  }
}
