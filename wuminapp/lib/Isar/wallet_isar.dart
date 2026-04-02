import 'dart:convert';
import 'dart:ffi';
import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:isar/isar.dart';
import 'package:path_provider/path_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_secure_keys.dart';

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

  /// 签名模式：`local`（热钱包）或 `external`（冷钱包）。
  late String signMode;
}

@collection
class WalletSettingsEntity {
  Id id = 0;

  int? activeWalletIndex;
  int updatedAtMillis = 0;
}

@collection
class AdminRoleCacheEntity {
  Id id = Isar.autoIncrement;

  @Index(unique: true, replace: true)
  late String pubkeyHex;

  late String roleName;

  @Index()
  late int updatedAt;
}

@collection
class ObservedAccountEntity {
  Id id = Isar.autoIncrement;

  @Index(unique: true, replace: true)
  late String accountId;

  late String orgName;
  late String publicKey;
  late String address;
  double? balance;
  late String source;
}

@collection
class LoginReplayEntity {
  Id id = Isar.autoIncrement;

  @Index(unique: true, replace: true)
  late String requestId;

  late int expiresAt;
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

/// 用户创建的个人多签账户（本地持久化）。
@collection
class PersonalDuoqianEntity {
  Id id = Isar.autoIncrement;

  /// 多签地址公钥 hex（32 字节，不含 0x 前缀）。
  @Index(unique: true, replace: true)
  late String duoqianAddress;

  /// 多签账户名称。
  late String name;

  /// 创建人 SS58 地址。
  late String creatorAddress;

  /// 添加时间戳（毫秒）。
  @Index()
  late int addedAtMillis;
}

/// 用户添加的多签机构（本地持久化）。
@collection
class DuoqianInstitutionEntity {
  Id id = Isar.autoIncrement;

  /// 多签地址公钥 hex（32 字节，不含 0x 前缀），唯一标识。
  @Index(unique: true, replace: true)
  late String duoqianAddress;

  /// SFID 标识（UTF-8 字符串）。
  late String sfidId;

  /// 机构名称（链上升级前暂用 sfidId 代替）。
  late String name;

  /// 添加时间戳（毫秒），用于排序。
  @Index()
  late int addedAtMillis;
}

/// 本地交易记录（持久化存储，去中心化设计，不依赖 SFID 服务器）。
@collection
class LocalTxEntity {
  Id id = Isar.autoIncrement;

  /// 交易唯一标识。
  @Index(unique: true, replace: true)
  late String txId;

  /// 所属钱包地址（SS58）。
  @Index()
  late String walletAddress;

  /// 交易类型：transfer / offchain_pay / proposal_transfer / fee_withdraw / fee_deposit /
  /// block_reward / bank_interest / gov_issuance / lightnode_reward / duoqian_create / duoqian_close / fund_destroy
  late String txType;

  /// 方向：in / out / info
  late String direction;

  String? fromAddress;
  String? toAddress;

  /// 金额（元）。
  late double amountYuan;

  /// 手续费（元）。
  double? feeYuan;

  /// 链下交易的清算省储行 shenfen_id。
  String? bankShenfenId;

  /// 状态：pending / confirmed / onchain
  late String status;

  /// 链上交易哈希。
  String? txHash;

  /// 链上区块号。
  int? blockNumber;

  /// 本地创建时间（毫秒时间戳）。
  @Index()
  late int createdAtMillis;

  /// 确认时间（毫秒时间戳）。
  int? confirmedAtMillis;
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
      AdminRoleCacheEntitySchema,
      ObservedAccountEntitySchema,
      LoginReplayEntitySchema,
      AppKvEntitySchema,
      DuoqianInstitutionEntitySchema,
      PersonalDuoqianEntitySchema,
      LocalTxEntitySchema,
    ];
    final isar =
        await Isar.open(schemas, name: 'wuminapp_wallet', directory: dir);
    await WalletIsarMigration.ensureMigrated(isar);
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
    if (kIsWeb) {
      return '.';
    }
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

class WalletIsarMigration {
  static const String _kSchemaVersion = 'wallet.data.schema.version';

  /// 当前 schema 版本。开发阶段直接覆盖，不做增量迁移。
  static const int currentSchemaVersion = 1;

  static Future<void> ensureMigrated(Isar isar) async {
    await _ensureSettingsRow(isar);
    final version = await _schemaVersion(isar);
    if (version >= currentSchemaVersion) {
      return;
    }
    await isar.writeTxn(() async {
      final entity = AppKvEntity()
        ..key = _kSchemaVersion
        ..intValue = currentSchemaVersion
        ..boolValue = null
        ..stringValue = null;
      await isar.appKvEntitys.put(entity);
    });
  }

  static Future<int> _schemaVersion(Isar isar) async {
    final row = await isar.appKvEntitys
        .filter()
        .keyEqualTo(_kSchemaVersion)
        .findFirst();
    return row?.intValue ?? 0;
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
