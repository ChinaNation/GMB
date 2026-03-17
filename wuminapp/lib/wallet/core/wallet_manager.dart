import 'dart:convert';
import 'dart:typed_data';

import 'package:bip39/bip39.dart' as bip39;
import 'package:bip39_mnemonic/bip39_mnemonic.dart' as bip39m;
import 'package:flutter/services.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:isar/isar.dart';
import 'package:local_auth/local_auth.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:substrate_bip39/crypto_scheme.dart';
import 'package:wuminapp_mobile/Isar/wallet_isar.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_secure_keys.dart';

class WalletProfile {
  const WalletProfile({
    required this.walletIndex,
    required this.walletName,
    required this.walletIcon,
    required this.balance,
    required this.address,
    required this.pubkeyHex,
    required this.alg,
    required this.ss58,
    required this.createdAtMillis,
    required this.source,
    required this.signMode,
  });

  final int walletIndex;
  final String walletName;
  final String walletIcon;
  final double balance;
  final String address;
  final String pubkeyHex;
  final String alg;
  final int ss58;
  final int createdAtMillis;
  final String source;

  /// 签名模式：`local`（热钱包）或 `external`（冷钱包）。
  final String signMode;

  bool get isHotWallet => signMode == 'local';
  bool get isColdWallet => signMode == 'external';
}

class WalletCreationResult {
  const WalletCreationResult({
    required this.profile,
    required this.mnemonic,
  });

  final WalletProfile profile;

  /// 助记词仅在创建时一次性展示，不会持久化。
  final String mnemonic;
}

class WalletSecret {
  const WalletSecret({required this.profile, required this.seedHex});

  final WalletProfile profile;

  /// 32 字节 mini-secret，以 64 个 hex 字符表示。
  final String seedHex;
}

/// [WalletManager.signUtf8WithWallet] 的返回值。
class WalletSignResult {
  const WalletSignResult({
    required this.account,
    required this.pubkeyHex,
    required this.sigAlg,
    required this.signatureHex,
  });

  final String account;
  final String pubkeyHex;
  final String sigAlg;
  final String signatureHex;
}

class WalletAuthException implements Exception {
  const WalletAuthException(this.message);
  final String message;

  @override
  String toString() => 'WalletAuthException: $message';
}

class WalletManager {
  static const int _ss58Format = 2027;
  static const FlutterSecureStorage _secureStorage = FlutterSecureStorage();
  static final LocalAuthentication _localAuth = LocalAuthentication();

  // ---------------------------------------------------------------------------
  // 查询
  // ---------------------------------------------------------------------------

  Future<List<WalletProfile>> getWallets() async {
    final isar = await WalletIsar.instance.db();
    final rows =
        await isar.walletProfileEntitys.where().sortByWalletIndex().findAll();
    return rows.map(_toProfile).toList(growable: false);
  }

  Future<WalletProfile?> getWallet() async {
    final isar = await WalletIsar.instance.db();
    final wallets =
        await isar.walletProfileEntitys.where().sortByWalletIndex().findAll();
    if (wallets.isEmpty) {
      return null;
    }

    final settings = await _getSettings(isar);
    final activeIndex = settings.activeWalletIndex;

    WalletProfileEntity selected = wallets.last;
    if (activeIndex != null) {
      for (final wallet in wallets) {
        if (wallet.walletIndex == activeIndex) {
          selected = wallet;
          break;
        }
      }
    } else {
      await isar.writeTxn(() async {
        settings.activeWalletIndex = selected.walletIndex;
        settings.updatedAtMillis = DateTime.now().millisecondsSinceEpoch;
        await isar.walletSettingsEntitys.put(settings);
      });
    }

    return _toProfile(selected);
  }

  /// 获取当前活跃热钱包的密钥材料。冷钱包返回 null。
  ///
  /// **已弃用**：请使用 [signWithWallet] 或 [signUtf8WithWallet]，seed 不出类。
  @Deprecated('Use signWithWallet() instead — seed should not leave WalletManager')
  Future<WalletSecret?> getLatestWalletSecret() async {
    final active = await getWallet();
    if (active == null) {
      return null;
    }
    if (active.isColdWallet) {
      return null;
    }
    final seedHex = await _readSeedHex(active.walletIndex);
    if (seedHex == null || seedHex.isEmpty) {
      return null;
    }
    return WalletSecret(profile: active, seedHex: seedHex);
  }

  Future<int?> getActiveWalletIndex() async {
    final isar = await WalletIsar.instance.db();
    final settings = await _getSettings(isar);
    return settings.activeWalletIndex;
  }

  Future<void> setActiveWallet(int walletIndex) async {
    final isar = await WalletIsar.instance.db();
    final exists = await isar.walletProfileEntitys
        .filter()
        .walletIndexEqualTo(walletIndex)
        .findFirst();
    if (exists == null) {
      throw Exception('未找到指定钱包');
    }

    final settings = await _getSettings(isar);
    await isar.writeTxn(() async {
      settings.activeWalletIndex = walletIndex;
      settings.updatedAtMillis = DateTime.now().millisecondsSinceEpoch;
      await isar.walletSettingsEntitys.put(settings);
    });
  }

  /// 获取指定热钱包的密钥材料。冷钱包返回 null。
  ///
  /// **已弃用**：请使用 [signWithWallet] 或 [signUtf8WithWallet]，seed 不出类。
  @Deprecated('Use signWithWallet() instead — seed should not leave WalletManager')
  Future<WalletSecret?> getWalletSecretByIndex(int walletIndex) async {
    final isar = await WalletIsar.instance.db();
    final row = await isar.walletProfileEntitys
        .filter()
        .walletIndexEqualTo(walletIndex)
        .findFirst();
    if (row == null) {
      return null;
    }

    final profile = _toProfile(row);
    if (profile.isColdWallet) {
      return null;
    }

    final seedHex = await _readSeedHex(walletIndex);
    if (seedHex == null || seedHex.isEmpty) {
      return null;
    }

    return WalletSecret(profile: profile, seedHex: seedHex);
  }

  // ---------------------------------------------------------------------------
  // 热钱包创建 / 导入
  // ---------------------------------------------------------------------------

  /// 创建热钱包：生成助记词 → 派生 seed → 存 seed（不存助记词）。
  Future<WalletCreationResult> createWallet() async {
    final mnemonic = bip39.generateMnemonic();
    final seed = await _mnemonicToMiniSecret(mnemonic);
    final derived = _deriveSr25519FromSeed(seed);
    final walletIndex = await _nextWalletIndex();

    final profile = WalletProfile(
      walletIndex: walletIndex,
      walletName: _defaultWalletName(walletIndex),
      walletIcon: _defaultWalletIcon(),
      balance: 0,
      address: derived.address,
      pubkeyHex: derived.pubkeyHex,
      alg: 'sr25519',
      ss58: _ss58Format,
      createdAtMillis: DateTime.now().millisecondsSinceEpoch,
      source: 'created',
      signMode: 'local',
    );

    await _appendHotWallet(profile, _toHex(seed));
    return WalletCreationResult(profile: profile, mnemonic: mnemonic);
  }

  /// 导入热钱包：验证助记词 → 派生 seed → 存 seed（不存助记词）。
  Future<WalletProfile> importWallet(String mnemonic) async {
    final trimmed = mnemonic.trim();
    if (!bip39.validateMnemonic(trimmed)) {
      throw Exception('助记词无效，请检查拼写和空格');
    }

    final seed = await _mnemonicToMiniSecret(trimmed);
    final derived = _deriveSr25519FromSeed(seed);
    final walletIndex = await _nextWalletIndex();

    final profile = WalletProfile(
      walletIndex: walletIndex,
      walletName: _defaultWalletName(walletIndex),
      walletIcon: _defaultWalletIcon(),
      balance: 0,
      address: derived.address,
      pubkeyHex: derived.pubkeyHex,
      alg: 'sr25519',
      ss58: _ss58Format,
      createdAtMillis: DateTime.now().millisecondsSinceEpoch,
      source: 'imported',
      signMode: 'local',
    );

    await _appendHotWallet(profile, _toHex(seed));
    return profile;
  }

  // ---------------------------------------------------------------------------
  // 冷钱包创建 / 导入
  // ---------------------------------------------------------------------------

  /// 创建冷钱包：生成助记词 → 派生地址 → 只存公钥（不存 seed）。
  /// 助记词仅一次性展示，由用户自行保管。
  Future<WalletCreationResult> createColdWallet() async {
    final mnemonic = bip39.generateMnemonic();
    final seed = await _mnemonicToMiniSecret(mnemonic);
    final derived = _deriveSr25519FromSeed(seed);
    final walletIndex = await _nextWalletIndex();

    final profile = WalletProfile(
      walletIndex: walletIndex,
      walletName: _defaultWalletName(walletIndex),
      walletIcon: _defaultWalletIcon(),
      balance: 0,
      address: derived.address,
      pubkeyHex: derived.pubkeyHex,
      alg: 'sr25519',
      ss58: _ss58Format,
      createdAtMillis: DateTime.now().millisecondsSinceEpoch,
      source: 'created',
      signMode: 'external',
    );

    await _appendColdWallet(profile);
    return WalletCreationResult(profile: profile, mnemonic: mnemonic);
  }

  /// 导入冷钱包：接受 SS58 地址 → 解码公钥 → 只存公钥。
  Future<WalletProfile> importColdWallet({required String address}) async {
    final trimmed = address.trim();
    if (trimmed.isEmpty) {
      throw Exception('地址不能为空');
    }

    // 解码 SS58 地址获取公钥。
    final List<int> pubkeyBytes;
    try {
      pubkeyBytes = Keyring().decodeAddress(trimmed);
    } catch (_) {
      throw Exception('无效的 SS58 地址，请检查格式');
    }

    final pubkeyHex = _toHex(pubkeyBytes);
    final walletIndex = await _nextWalletIndex();

    final profile = WalletProfile(
      walletIndex: walletIndex,
      walletName: _defaultWalletName(walletIndex),
      walletIcon: _defaultWalletIcon(),
      balance: 0,
      address: trimmed,
      pubkeyHex: pubkeyHex,
      alg: 'sr25519',
      ss58: _ss58Format,
      createdAtMillis: DateTime.now().millisecondsSinceEpoch,
      source: 'imported',
      signMode: 'external',
    );

    await _appendColdWallet(profile);
    return profile;
  }

  // ---------------------------------------------------------------------------
  // 删除
  // ---------------------------------------------------------------------------

  Future<void> clearWallet() async {
    final isar = await WalletIsar.instance.db();
    final wallets = await isar.walletProfileEntitys.where().findAll();
    for (final row in wallets) {
      if (row.signMode == 'local') {
        await _deleteSeedHex(row.walletIndex);
      }
    }

    await isar.writeTxn(() async {
      await isar.walletProfileEntitys.clear();
      final settings = await _getSettings(isar);
      settings.activeWalletIndex = null;
      settings.updatedAtMillis = DateTime.now().millisecondsSinceEpoch;
      await isar.walletSettingsEntitys.put(settings);
    });
  }

  Future<void> deleteWallet(int walletIndex) async {
    final isar = await WalletIsar.instance.db();
    final target = await isar.walletProfileEntitys
        .filter()
        .walletIndexEqualTo(walletIndex)
        .findFirst();
    if (target == null) {
      throw Exception('未找到钱包');
    }

    if (target.signMode == 'local') {
      await _deleteSeedHex(walletIndex);
    }

    await isar.writeTxn(() async {
      await isar.walletProfileEntitys.delete(target.id);

      final settings = await _getSettings(isar);
      if (settings.activeWalletIndex == walletIndex) {
        final remains = await isar.walletProfileEntitys
            .where()
            .sortByWalletIndex()
            .findAll();
        settings.activeWalletIndex =
            remains.isEmpty ? null : remains.last.walletIndex;
        settings.updatedAtMillis = DateTime.now().millisecondsSinceEpoch;
        await isar.walletSettingsEntitys.put(settings);
      }
    });
  }

  // ---------------------------------------------------------------------------
  // 更新
  // ---------------------------------------------------------------------------

  Future<void> renameWallet(int walletIndex, String walletName) async {
    await updateWalletDisplay(walletIndex, walletName: walletName);
  }

  Future<void> updateWalletDisplay(
    int walletIndex, {
    String? walletName,
    String? walletIcon,
  }) async {
    if (walletName == null && walletIcon == null) {
      return;
    }

    final nextName = walletName?.trim();
    if (walletName != null && (nextName == null || nextName.isEmpty)) {
      throw Exception('钱包名称不能为空');
    }
    if (walletIcon != null && walletIcon.trim().isEmpty) {
      throw Exception('钱包图标不能为空');
    }

    final isar = await WalletIsar.instance.db();
    final row = await isar.walletProfileEntitys
        .filter()
        .walletIndexEqualTo(walletIndex)
        .findFirst();
    if (row == null) {
      throw Exception('未找到钱包');
    }

    await isar.writeTxn(() async {
      if (nextName != null) {
        row.walletName = nextName;
      }
      if (walletIcon != null) {
        row.walletIcon = walletIcon.trim();
      }
      await isar.walletProfileEntitys.put(row);
    });
  }

  Future<void> setWalletBalance(int walletIndex, double balance) async {
    final isar = await WalletIsar.instance.db();
    final row = await isar.walletProfileEntitys
        .filter()
        .walletIndexEqualTo(walletIndex)
        .findFirst();
    if (row == null) {
      throw Exception('未找到钱包');
    }

    await isar.writeTxn(() async {
      row.balance = balance;
      await isar.walletProfileEntitys.put(row);
    });
  }

  // ---------------------------------------------------------------------------
  // Seed 派生
  // ---------------------------------------------------------------------------

  /// mnemonic → entropy → PBKDF2 → 64 字节 → 前 32 字节 mini-secret。
  ///
  /// 使用 Substrate 特定的 BIP39 派生（非标准 BIP32），与
  /// `polkadart_keyring` 的 `fromMnemonic` 内部逻辑一致。
  Future<List<int>> _mnemonicToMiniSecret(String mnemonic) async {
    final entropy =
        bip39m.Mnemonic.fromSentence(mnemonic, bip39m.Language.english)
            .entropy;
    return CryptoScheme.miniSecretFromEntropy(entropy);
  }

  /// 从 32 字节 mini-secret 派生 sr25519 密钥对。
  _DerivedWallet _deriveSr25519FromSeed(List<int> seed) {
    final pair = Keyring.sr25519.fromSeed(Uint8List.fromList(seed));
    pair.ss58Format = _ss58Format;
    final pubkeyBytes = pair.bytes().toList(growable: false);
    final pubkeyHex = _toHex(pubkeyBytes);
    final address = pair.address;
    return _DerivedWallet(address: address, pubkeyHex: pubkeyHex);
  }

  // ---------------------------------------------------------------------------
  // Secure Storage（seed）
  // ---------------------------------------------------------------------------

  String _seedKey(int walletIndex) =>
      WalletSecureKeys.seedHexV1(walletIndex);

  Future<void> _writeSeedHex(int walletIndex, String seedHex) async {
    await _secureStorage.write(key: _seedKey(walletIndex), value: seedHex);
  }

  static final RegExp _seedHexPattern = RegExp(r'^[0-9a-fA-F]{64}$');

  /// 读取 seed（含认证 + 格式校验）。
  Future<String?> _readSeedHex(int walletIndex) async {
    await _authenticateIfSupported();
    return _readSeedHexRaw(walletIndex);
  }

  /// 读取 seed（仅格式校验，不触发认证）。供已通过认证的内部方法调用。
  Future<String?> _readSeedHexRaw(int walletIndex) async {
    final seedHex = await _secureStorage.read(key: _seedKey(walletIndex));
    if (seedHex == null) return null;
    if (!_seedHexPattern.hasMatch(seedHex)) {
      throw const WalletAuthException('钱包密钥数据异常，请重新导入钱包');
    }
    return seedHex;
  }

  // ---------------------------------------------------------------------------
  // 签名（seed 不出类）
  // ---------------------------------------------------------------------------

  /// 使用指定热钱包对 [payload] 进行 sr25519 签名。
  ///
  /// seed 仅在本方法内短暂存在，签名完成后立即清零，不对外暴露。
  /// 每次调用均触发生物/密码认证（设备不支持时跳过）。
  Future<Uint8List> signWithWallet(int walletIndex, Uint8List payload) async {
    await _authenticateIfSupported();
    final seedHex = await _readSeedHexRaw(walletIndex);
    if (seedHex == null) {
      throw const WalletAuthException('密钥不可用，请重新导入钱包');
    }
    final seedBytes = Uint8List.fromList(_hexToBytes(seedHex));
    try {
      final pair = Keyring.sr25519.fromSeed(seedBytes);
      return Uint8List.fromList(pair.sign(payload));
    } finally {
      seedBytes.fillRange(0, seedBytes.length, 0);
    }
  }

  /// 使用指定热钱包对 UTF-8 字符串进行 sr25519 签名，返回签名结果。
  ///
  /// 用于登录等场景，返回值包含公钥、签名 hex 等信息。
  Future<WalletSignResult> signUtf8WithWallet(
    int walletIndex,
    String message,
  ) async {
    await _authenticateIfSupported();
    final seedHex = await _readSeedHexRaw(walletIndex);
    if (seedHex == null) {
      throw const WalletAuthException('密钥不可用，请重新导入钱包');
    }

    // 查询钱包 profile 以获取公钥信息
    final isar = await WalletIsar.instance.db();
    final row = await isar.walletProfileEntitys
        .filter()
        .walletIndexEqualTo(walletIndex)
        .findFirst();
    if (row == null) {
      throw const WalletAuthException('未找到指定钱包');
    }
    final profile = _toProfile(row);

    final seedBytes = Uint8List.fromList(_hexToBytes(seedHex));
    try {
      final pair = Keyring.sr25519.fromSeed(seedBytes);
      pair.ss58Format = _ss58Format;

      // 校验 seed 与 profile 公钥一致
      final localPubkeyHex = _toHex(pair.bytes().toList(growable: false));
      if (localPubkeyHex.toLowerCase() != profile.pubkeyHex.toLowerCase()) {
        throw const WalletAuthException('本地签名密钥与当前钱包不一致，请重新导入钱包');
      }

      final payload = Uint8List.fromList(utf8.encode(message));
      final signature = pair.sign(payload);
      return WalletSignResult(
        account: profile.address,
        pubkeyHex: '0x${profile.pubkeyHex}',
        sigAlg: 'sr25519',
        signatureHex: '0x${_toHex(signature.toList(growable: false))}',
      );
    } finally {
      seedBytes.fillRange(0, seedBytes.length, 0);
    }
  }

  List<int> _hexToBytes(String input) {
    final text = input.startsWith('0x') ? input.substring(2) : input;
    if (text.isEmpty || text.length.isOdd) return const <int>[];
    final out = <int>[];
    for (var i = 0; i < text.length; i += 2) {
      out.add(int.parse(text.substring(i, i + 2), radix: 16));
    }
    return out;
  }

  Future<void> _deleteSeedHex(int walletIndex) async {
    await _secureStorage.delete(key: _seedKey(walletIndex));
  }

  static Future<void> _authenticateIfSupported() async {
    try {
      final supported = await _localAuth.isDeviceSupported();
      if (!supported) return;

      final ok = await _localAuth.authenticate(
        localizedReason: '请验证身份以访问钱包密钥',
        options: const AuthenticationOptions(
          biometricOnly: false,
          stickyAuth: true,
          useErrorDialogs: true,
        ),
      );
      if (!ok) {
        throw const WalletAuthException('未通过身份验证');
      }
    } on PlatformException catch (e) {
      throw WalletAuthException('身份验证不可用：${e.message ?? e.code}');
    }
  }

  // ---------------------------------------------------------------------------
  // 内部工具
  // ---------------------------------------------------------------------------

  Future<int> _nextWalletIndex() async {
    final isar = await WalletIsar.instance.db();
    final rows =
        await isar.walletProfileEntitys.where().sortByWalletIndex().findAll();
    if (rows.isEmpty) {
      return 1;
    }

    final used = rows.map((e) => e.walletIndex).toSet();
    var candidate = rows.length + 1;
    while (used.contains(candidate)) {
      candidate++;
    }
    return candidate;
  }

  /// 热钱包入库：写 seed + 写 Isar + 设置活跃。
  Future<void> _appendHotWallet(WalletProfile profile, String seedHex) async {
    final isar = await WalletIsar.instance.db();
    final entity = _toEntity(profile);

    await _writeSeedHex(profile.walletIndex, seedHex);
    await isar.writeTxn(() async {
      await isar.walletProfileEntitys.put(entity);

      final settings = await _getSettings(isar);
      settings.activeWalletIndex = profile.walletIndex;
      settings.updatedAtMillis = DateTime.now().millisecondsSinceEpoch;
      await isar.walletSettingsEntitys.put(settings);
    });
  }

  /// 冷钱包入库：仅写 Isar + 设置活跃（不写 secure storage）。
  Future<void> _appendColdWallet(WalletProfile profile) async {
    final isar = await WalletIsar.instance.db();
    final entity = _toEntity(profile);

    await isar.writeTxn(() async {
      await isar.walletProfileEntitys.put(entity);

      final settings = await _getSettings(isar);
      settings.activeWalletIndex = profile.walletIndex;
      settings.updatedAtMillis = DateTime.now().millisecondsSinceEpoch;
      await isar.walletSettingsEntitys.put(settings);
    });
  }

  Future<WalletSettingsEntity> _getSettings(Isar isar) async {
    final row = await isar.walletSettingsEntitys.get(0);
    if (row != null) {
      return row;
    }

    final created = WalletSettingsEntity()
      ..id = 0
      ..updatedAtMillis = DateTime.now().millisecondsSinceEpoch;
    await isar.writeTxn(() async {
      await isar.walletSettingsEntitys.put(created);
    });
    return created;
  }

  String _toHex(List<int> bytes) {
    const chars = '0123456789abcdef';
    final buf = StringBuffer();
    for (final b in bytes) {
      buf
        ..write(chars[(b >> 4) & 0x0f])
        ..write(chars[b & 0x0f]);
    }
    return buf.toString();
  }

  String _defaultWalletName(int walletIndex) {
    return '钱包$walletIndex';
  }

  String _defaultWalletIcon() {
    return 'wallet';
  }

  WalletProfile _toProfile(WalletProfileEntity row) {
    return WalletProfile(
      walletIndex: row.walletIndex,
      walletName: row.walletName,
      walletIcon: row.walletIcon,
      balance: row.balance,
      address: row.address,
      pubkeyHex: row.pubkeyHex,
      alg: row.alg,
      ss58: row.ss58,
      createdAtMillis: row.createdAtMillis,
      source: row.source,
      signMode: row.signMode,
    );
  }

  WalletProfileEntity _toEntity(WalletProfile profile) {
    return WalletProfileEntity()
      ..walletIndex = profile.walletIndex
      ..walletName = profile.walletName
      ..walletIcon = profile.walletIcon
      ..balance = profile.balance
      ..address = profile.address
      ..pubkeyHex = profile.pubkeyHex
      ..alg = profile.alg
      ..ss58 = profile.ss58
      ..createdAtMillis = profile.createdAtMillis
      ..source = profile.source
      ..signMode = profile.signMode;
  }
}

class _DerivedWallet {
  const _DerivedWallet({
    required this.address,
    required this.pubkeyHex,
  });

  final String address;
  final String pubkeyHex;
}
