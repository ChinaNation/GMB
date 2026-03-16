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
}

@collection
class WalletSettingsEntity {
  Id id = 0;

  int? activeWalletIndex;
  int updatedAtMillis = 0;
}

@collection
class TxRecordEntity {
  Id id = Isar.autoIncrement;

  @Index(unique: true, replace: true)
  late String txHash;

  late String fromAddress;
  late String toAddress;
  late double amount;
  late String symbol;

  @Index()
  late int createdAtMillis;

  late String status;
  String? failureReason;

  int? usedNonce;
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
      TxRecordEntitySchema,
      AdminRoleCacheEntitySchema,
      ObservedAccountEntitySchema,
      LoginReplayEntitySchema,
      AppKvEntitySchema,
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
  static const FlutterSecureStorage _secureStorage = FlutterSecureStorage();

  static const String _kMigrationMarker = 'wallet.isar.migrated.v1';
  static const String _kSchemaVersion = 'wallet.data.schema.version';
  static const String _kAdminUpdatedAtKey = 'wallet.admin_catalog.updated_at';

  static const String _kWalletItems = 'wallet.items';
  static const String _kActiveWalletIndex = 'wallet.active_index';
  static const String _kLegacyHasWallet = 'wallet.has_wallet';
  static const String _kLegacyWalletIndex = 'wallet.index';
  static const String _kLegacyAddress = 'wallet.address';
  static const String _kLegacyPubkeyHex = 'wallet.pubkey_hex';
  static const String _kLegacyAlg = 'wallet.alg';
  static const String _kLegacySs58 = 'wallet.ss58';
  static const String _kLegacyCreatedAt = 'wallet.created_at_millis';
  static const String _kLegacySource = 'wallet.source';
  static const String _kLegacyMnemonic = 'wallet.mnemonic';

  static const String _kOnchainRecords = 'trade.onchain.records';
  static const String _kRoleMap = 'wallet.admin_catalog.role_map';
  static const String _kRoleMapUpdatedAt = 'wallet.admin_catalog.updated_at';
  static const String _kObservedAccounts = 'observe.accounts';
  static const String _kUsedRequestIds = 'login.used_request_ids';
  static const String _kLegacyAttestToken = 'attest.token';
  static const String _kLegacyAttestExpiresAt = 'attest.expires_at_millis';
  static const String _kLegacyAttestPolicy = 'attest.policy';
  static const String _kLegacyAttestLastPayload = 'attest.last_payload';

  static Future<void> ensureMigrated(Isar isar) async {
    final marker = await isar.appKvEntitys
        .filter()
        .keyEqualTo(_kMigrationMarker)
        .findFirst();
    if (marker?.boolValue == true) {
      await _ensureSettingsRow(isar);
      await _upgradeToLatestSchema(isar);
      await _cleanupLegacyPrefs();
      await _cleanupLegacySecureKeys(isar);
      return;
    }

    final prefs = await SharedPreferences.getInstance();
    final now = DateTime.now().millisecondsSinceEpoch;

    final wallets = await _readWalletProfilesFromPrefs(prefs);
    final activeWalletIndex = _resolveActiveWalletIndex(prefs, wallets);
    final txRecords = _readTxRecordsFromPrefs(prefs);
    final roleMap = _readRoleMapFromPrefs(prefs);
    final roleMapUpdatedAt = prefs.getInt(_kRoleMapUpdatedAt) ?? (now ~/ 1000);
    final observed = _readObservedFromPrefs(prefs);
    final replay = _readReplayFromPrefs(prefs);

    await isar.writeTxn(() async {
      if (wallets.isNotEmpty && await isar.walletProfileEntitys.count() == 0) {
        await isar.walletProfileEntitys.putAll(wallets);
      }

      final settings = await isar.walletSettingsEntitys.get(0) ??
          (WalletSettingsEntity()..id = 0);
      settings.activeWalletIndex = activeWalletIndex;
      settings.updatedAtMillis = now;
      await isar.walletSettingsEntitys.put(settings);

      if (txRecords.isNotEmpty && await isar.txRecordEntitys.count() == 0) {
        await isar.txRecordEntitys.putAll(txRecords);
      }

      if (roleMap.isNotEmpty && await isar.adminRoleCacheEntitys.count() == 0) {
        await isar.adminRoleCacheEntitys
            .putAll(roleMap.values.toList(growable: false));
      }

      if (observed.isNotEmpty &&
          await isar.observedAccountEntitys.count() == 0) {
        await isar.observedAccountEntitys.putAll(observed);
      }

      if (replay.isNotEmpty && await isar.loginReplayEntitys.count() == 0) {
        await isar.loginReplayEntitys.putAll(replay);
      }

      await _putKvBool(isar, _kMigrationMarker, true);
      await _putKvInt(isar, _kSchemaVersion, 5);
      await _putKvInt(isar, _kAdminUpdatedAtKey, roleMapUpdatedAt);
    });

    await _cleanupLegacyPrefs();
    await _cleanupLegacySecureKeys(isar);
  }

  static Future<void> _upgradeToLatestSchema(Isar isar) async {
    final current = await _schemaVersion(isar);
    if (current < 4) {
      await _cleanupLegacyPrefs();
      await _cleanupLegacySecureKeys(isar);
    }
    if (current < 5) {
      // v5: TxRecordEntity 新增 usedNonce 字段（nullable，无需数据迁移）
      await isar.writeTxn(() async {
        await _putKvInt(isar, _kSchemaVersion, 5);
      });
    }
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

  static Future<List<WalletProfileEntity>> _readWalletProfilesFromPrefs(
    SharedPreferences prefs,
  ) async {
    final out = <WalletProfileEntity>[];
    final raw = prefs.getString(_kWalletItems);
    if (raw != null && raw.trim().isNotEmpty) {
      try {
        final decoded = jsonDecode(raw);
        if (decoded is List) {
          for (final item in decoded) {
            final map = _toStringKeyMap(item);
            if (map == null) {
              continue;
            }
            final entity = _walletFromMap(map);
            if (entity == null) {
              continue;
            }
            out.add(entity);
            await _migrateLegacyMnemonic(
                entity.walletIndex, map['mnemonic']?.toString());
          }
        }
      } catch (_) {
        // Ignore invalid legacy payload; keep clean fallback.
      }
    }

    if (out.isEmpty) {
      final legacy = await _readLegacySingleWallet(prefs);
      if (legacy != null) {
        out.add(legacy);
      }
    }

    out.sort((a, b) => a.walletIndex.compareTo(b.walletIndex));
    return out;
  }

  static int? _resolveActiveWalletIndex(
    SharedPreferences prefs,
    List<WalletProfileEntity> wallets,
  ) {
    final fromPrefs = prefs.getInt(_kActiveWalletIndex);
    if (fromPrefs != null && wallets.any((it) => it.walletIndex == fromPrefs)) {
      return fromPrefs;
    }
    if (wallets.isEmpty) {
      return null;
    }
    return wallets.last.walletIndex;
  }

  static Future<WalletProfileEntity?> _readLegacySingleWallet(
    SharedPreferences prefs,
  ) async {
    final hasWallet = prefs.getBool(_kLegacyHasWallet) ?? false;
    if (!hasWallet) {
      return null;
    }

    final walletIndex = prefs.getInt(_kLegacyWalletIndex);
    final address = prefs.getString(_kLegacyAddress);
    final pubkeyHex = prefs.getString(_kLegacyPubkeyHex);
    final alg = prefs.getString(_kLegacyAlg);
    final ss58 = prefs.getInt(_kLegacySs58);
    final createdAt = prefs.getInt(_kLegacyCreatedAt);
    final source = prefs.getString(_kLegacySource);
    final mnemonic = prefs.getString(_kLegacyMnemonic);

    if (walletIndex == null ||
        address == null ||
        pubkeyHex == null ||
        alg == null ||
        ss58 == null ||
        createdAt == null ||
        source == null) {
      return null;
    }

    await _migrateLegacyMnemonic(walletIndex, mnemonic);

    return WalletProfileEntity()
      ..walletIndex = walletIndex
      ..walletName = '钱包$walletIndex'
      ..walletIcon = 'wallet'
      ..balance = 0
      ..address = address
      ..pubkeyHex = _normalizeHex(pubkeyHex) ?? pubkeyHex
      ..alg = alg
      ..ss58 = ss58
      ..createdAtMillis = createdAt
      ..source = source;
  }

  static WalletProfileEntity? _walletFromMap(Map<String, dynamic> map) {
    final walletIndex = _asInt(map['walletIndex']);
    final address = map['address']?.toString().trim();
    final pubkey = _normalizeHex(map['pubkeyHex']?.toString() ?? '');
    final alg = map['alg']?.toString().trim();
    final ss58 = _asInt(map['ss58']);
    final createdAt = _asInt(map['createdAtMillis']);
    final source = map['source']?.toString().trim();

    if (walletIndex == null ||
        address == null ||
        address.isEmpty ||
        pubkey == null ||
        alg == null ||
        alg.isEmpty ||
        ss58 == null ||
        createdAt == null ||
        source == null ||
        source.isEmpty) {
      return null;
    }

    final name = map['walletName']?.toString().trim();
    final icon = map['walletIcon']?.toString().trim();

    return WalletProfileEntity()
      ..walletIndex = walletIndex
      ..walletName = (name == null || name.isEmpty) ? '钱包$walletIndex' : name
      ..walletIcon = (icon == null || icon.isEmpty) ? 'wallet' : icon
      ..balance = _asDouble(map['balance']) ?? 0
      ..address = address
      ..pubkeyHex = pubkey
      ..alg = alg
      ..ss58 = ss58
      ..createdAtMillis = createdAt
      ..source = source;
  }

  static Future<void> _migrateLegacyMnemonic(
      int walletIndex, String? mnemonic) async {
    final trimmed = mnemonic?.trim() ?? '';
    if (trimmed.isEmpty) {
      return;
    }
    try {
      final key = WalletSecureKeys.mnemonicV1(walletIndex);
      final existing = await _secureStorage.read(key: key);
      if (existing != null && existing.trim().isNotEmpty) {
        return;
      }
      await _secureStorage.write(key: key, value: trimmed);
      await _secureStorage.delete(key: _legacyMnemonicSecureKey(walletIndex));
    } on MissingPluginException {
      // Some unit tests intentionally do not mount secure storage plugin.
      return;
    }
  }

  static Future<void> _cleanupLegacySecureKeys(Isar isar) async {
    try {
      final rows =
          await isar.walletProfileEntitys.where().sortByWalletIndex().findAll();
      for (final row in rows) {
        await _secureStorage.delete(
          key: _legacyMnemonicSecureKey(row.walletIndex),
        );
      }
    } on MissingPluginException {
      return;
    }
  }

  static Future<void> _cleanupLegacyPrefs() async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove(_kWalletItems);
    await prefs.remove(_kActiveWalletIndex);
    await prefs.remove(_kLegacyHasWallet);
    await prefs.remove('wallet.counter');
    await prefs.remove(_kLegacyWalletIndex);
    await prefs.remove(_kLegacyAddress);
    await prefs.remove(_kLegacyPubkeyHex);
    await prefs.remove(_kLegacyAlg);
    await prefs.remove(_kLegacySs58);
    await prefs.remove(_kLegacyCreatedAt);
    await prefs.remove(_kLegacySource);
    await prefs.remove(_kLegacyMnemonic);
    await prefs.remove('settings.face_auth_enabled');
    await prefs.remove(_kOnchainRecords);
    await prefs.remove(_kRoleMap);
    await prefs.remove(_kRoleMapUpdatedAt);
    await prefs.remove(_kObservedAccounts);
    await prefs.remove(_kLegacyAttestToken);
    await prefs.remove(_kLegacyAttestExpiresAt);
    await prefs.remove(_kLegacyAttestPolicy);
    await prefs.remove(_kLegacyAttestLastPayload);
  }

  static String _legacyMnemonicSecureKey(int walletIndex) {
    return 'wallet.mnemonic.$walletIndex';
  }

  static List<TxRecordEntity> _readTxRecordsFromPrefs(SharedPreferences prefs) {
    final out = <TxRecordEntity>[];
    final raw = prefs.getString(_kOnchainRecords);
    if (raw == null || raw.trim().isEmpty) {
      return out;
    }

    try {
      final decoded = jsonDecode(raw);
      if (decoded is! List) {
        return out;
      }

      for (final item in decoded) {
        final map = _toStringKeyMap(item);
        if (map == null) {
          continue;
        }
        final txHash = map['txHash']?.toString().trim();
        final fromAddress = map['fromAddress']?.toString().trim();
        final toAddress = map['toAddress']?.toString().trim();
        if (txHash == null ||
            txHash.isEmpty ||
            fromAddress == null ||
            fromAddress.isEmpty ||
            toAddress == null ||
            toAddress.isEmpty) {
          continue;
        }

        final createdAt = _asInt(map['createdAtMillis']) ??
            DateTime.now().millisecondsSinceEpoch;
        out.add(
          TxRecordEntity()
            ..txHash = txHash
            ..fromAddress = fromAddress
            ..toAddress = toAddress
            ..amount = _asDouble(map['amount']) ?? 0
            ..symbol = (map['symbol']?.toString().trim().isNotEmpty ?? false)
                ? map['symbol'].toString().trim()
                : 'CIT'
            ..createdAtMillis = createdAt
            ..status = (map['status']?.toString().trim().isNotEmpty ?? false)
                ? map['status'].toString().trim().toLowerCase()
                : 'pending'
            ..failureReason = map['failureReason']?.toString(),
        );
      }
    } catch (_) {
      return <TxRecordEntity>[];
    }

    return out;
  }

  static Map<String, AdminRoleCacheEntity> _readRoleMapFromPrefs(
    SharedPreferences prefs,
  ) {
    final raw = prefs.getString(_kRoleMap);
    final updatedAt = prefs.getInt(_kRoleMapUpdatedAt) ??
        (DateTime.now().millisecondsSinceEpoch ~/ 1000);
    if (raw == null || raw.trim().isEmpty) {
      return <String, AdminRoleCacheEntity>{};
    }

    try {
      final decoded = jsonDecode(raw);
      if (decoded is! Map) {
        return <String, AdminRoleCacheEntity>{};
      }
      final out = <String, AdminRoleCacheEntity>{};
      for (final entry in decoded.entries) {
        final key = _normalizeHex(entry.key.toString());
        final value = entry.value?.toString().trim() ?? '';
        if (key == null || value.isEmpty) {
          continue;
        }
        out[key] = AdminRoleCacheEntity()
          ..pubkeyHex = key
          ..roleName = value
          ..updatedAt = updatedAt;
      }
      return out;
    } catch (_) {
      return <String, AdminRoleCacheEntity>{};
    }
  }

  static List<ObservedAccountEntity> _readObservedFromPrefs(
    SharedPreferences prefs,
  ) {
    final raw = prefs.getString(_kObservedAccounts);
    if (raw == null || raw.trim().isEmpty) {
      return <ObservedAccountEntity>[];
    }

    try {
      final decoded = jsonDecode(raw);
      if (decoded is! List) {
        return <ObservedAccountEntity>[];
      }
      final out = <ObservedAccountEntity>[];
      for (final item in decoded) {
        final map = _toStringKeyMap(item);
        if (map == null) {
          continue;
        }
        final publicKey = _normalizeHex(map['publicKey']?.toString() ?? '');
        if (publicKey == null) {
          continue;
        }
        final address = map['address']?.toString().trim() ?? '';
        final accountId = map['id']?.toString().trim();
        out.add(
          ObservedAccountEntity()
            ..accountId = (accountId == null || accountId.isEmpty)
                ? 'manual:$publicKey'
                : accountId
            ..orgName = map['orgName']?.toString().trim().isNotEmpty ?? false
                ? map['orgName'].toString().trim()
                : '自定义观察账户'
            ..publicKey = publicKey
            ..address = address
            ..balance = _asDouble(map['balance'])
            ..source = map['source']?.toString().trim().isNotEmpty ?? false
                ? map['source'].toString().trim()
                : 'manual',
        );
      }
      return out;
    } catch (_) {
      return <ObservedAccountEntity>[];
    }
  }

  static List<LoginReplayEntity> _readReplayFromPrefs(SharedPreferences prefs) {
    final raw = prefs.getString(_kUsedRequestIds);
    if (raw == null || raw.trim().isEmpty) {
      return <LoginReplayEntity>[];
    }

    try {
      final decoded = jsonDecode(raw);
      if (decoded is! Map) {
        return <LoginReplayEntity>[];
      }
      final out = <LoginReplayEntity>[];
      for (final entry in decoded.entries) {
        final expiresAt = _asInt(entry.value);
        final requestId = entry.key.toString().trim();
        if (expiresAt == null || requestId.isEmpty) {
          continue;
        }
        out.add(
          LoginReplayEntity()
            ..requestId = requestId
            ..expiresAt = expiresAt,
        );
      }
      return out;
    } catch (_) {
      return <LoginReplayEntity>[];
    }
  }

  static Future<void> _putKvBool(Isar isar, String key, bool value) async {
    final entity = AppKvEntity()
      ..key = key
      ..boolValue = value
      ..intValue = null
      ..stringValue = null;
    await isar.appKvEntitys.put(entity);
  }

  static Future<void> _putKvInt(Isar isar, String key, int value) async {
    final entity = AppKvEntity()
      ..key = key
      ..intValue = value
      ..boolValue = null
      ..stringValue = null;
    await isar.appKvEntitys.put(entity);
  }

  static Map<String, dynamic>? _toStringKeyMap(dynamic item) {
    if (item is Map<String, dynamic>) {
      return item;
    }
    if (item is Map) {
      return item.map((k, v) => MapEntry(k.toString(), v));
    }
    return null;
  }

  static int? _asInt(dynamic value) {
    switch (value) {
      case int v:
        return v;
      case num v:
        return v.toInt();
      case String v:
        return int.tryParse(v);
      default:
        return null;
    }
  }

  static double? _asDouble(dynamic value) {
    switch (value) {
      case double v:
        return v;
      case num v:
        return v.toDouble();
      case String v:
        return double.tryParse(v);
      default:
        return null;
    }
  }

  static String? _normalizeHex(String input) {
    var value = input.trim().toLowerCase();
    if (value.startsWith('0x')) {
      value = value.substring(2);
    }
    if (!RegExp(r'^[0-9a-f]{64}$').hasMatch(value)) {
      return null;
    }
    return value;
  }
}
