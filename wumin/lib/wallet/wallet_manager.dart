import 'dart:convert';
import 'package:bip39/bip39.dart' as bip39;
import 'package:bip39_mnemonic/bip39_mnemonic.dart' as bip39m;
import 'package:flutter/services.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:isar/isar.dart';
import 'package:local_auth/local_auth.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:substrate_bip39/crypto_scheme.dart';
import 'package:wumin/isar/wallet_isar.dart';
import 'package:wumin/wallet/mnemonic_cipher.dart';
import 'package:wumin/wallet/wallet_secure_keys.dart';

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
    this.groupNames = const [],
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
  final String signMode;

  /// 所属分组列表（不含"全部"）。
  final List<String> groupNames;

  /// 是否属于指定分组（"全部"始终返回 true）。
  bool inGroup(String group) {
    if (group == '全部') return true;
    return groupNames.contains(group);
  }

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

  Future<WalletProfile?> getWalletByIndex(int walletIndex) async {
    final isar = await WalletIsar.instance.db();
    final row = await isar.walletProfileEntitys
        .filter()
        .walletIndexEqualTo(walletIndex)
        .findFirst();
    if (row == null) {
      return null;
    }
    return _toProfile(row);
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

  // ---------------------------------------------------------------------------
  // 热钱包创建 / 导入
  // ---------------------------------------------------------------------------

  /// 创建新钱包。
  ///
  /// [wordCount] 助记词个数，12（默认）或 24。
  Future<WalletCreationResult> createWallet({int wordCount = 12}) async {
    assert(wordCount == 12 || wordCount == 24);
    final strength = wordCount == 24 ? 256 : 128;
    final mnemonic = bip39.generateMnemonic(strength: strength);
    final seed = await _mnemonicToMiniSecret(mnemonic);
    try {
      final derived = _deriveSr25519FromSeed(seed);

      // walletIndex 分配和写入在同一个事务中完成，避免并发冲突。
      final profile = await _appendHotWalletAtomic(
        address: derived.address,
        pubkeyHex: derived.pubkeyHex,
        seedHex: _toHex(seed),
        source: 'created',
      );

      await _writeMnemonic(profile.walletIndex, mnemonic);
      return WalletCreationResult(profile: profile, mnemonic: mnemonic);
    } finally {
      _zeroList(seed);
    }
  }

  Future<WalletProfile> importWallet(String mnemonic) async {
    final trimmed = mnemonic.trim();
    if (!bip39.validateMnemonic(trimmed)) {
      throw Exception('助记词无效，请检查拼写和空格');
    }

    final seed = await _mnemonicToMiniSecret(trimmed);
    try {
      final derived = _deriveSr25519FromSeed(seed);

      // 检测重复：同一公钥的钱包已存在则拒绝
      await _checkDuplicatePubkey(derived.pubkeyHex);

      // walletIndex 分配和写入在同一个事务中完成，避免并发冲突。
      final profile = await _appendHotWalletAtomic(
        address: derived.address,
        pubkeyHex: derived.pubkeyHex,
        seedHex: _toHex(seed),
        source: 'imported',
      );

      await _writeMnemonic(profile.walletIndex, trimmed);
      return profile;
    } finally {
      _zeroList(seed);
    }
  }

  // ---------------------------------------------------------------------------
  // 删除
  // ---------------------------------------------------------------------------

  Future<void> deleteWallet(int walletIndex) async {
    final isar = await WalletIsar.instance.db();
    final target = await isar.walletProfileEntitys
        .filter()
        .walletIndexEqualTo(walletIndex)
        .findFirst();
    if (target == null) {
      throw Exception('未找到钱包');
    }

    await _deleteSeedHex(walletIndex);
    await _deleteMnemonic(walletIndex);

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

  /// 钱包名称最大字符数。
  static const int maxWalletNameLength = 5;

  Future<void> renameWallet(int walletIndex, String walletName) async {
    final nextName = walletName.trim();
    if (nextName.isEmpty) {
      throw Exception('钱包名称不能为空');
    }
    if (nextName.runes.length > maxWalletNameLength) {
      throw Exception('钱包名称最多$maxWalletNameLength个字');
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
      row.walletName = nextName;
      await isar.walletProfileEntitys.put(row);
    });
  }

  // ---------------------------------------------------------------------------
  // 签名（seed 不出类）
  // ---------------------------------------------------------------------------

  Future<Uint8List> signWithWallet(int walletIndex, Uint8List payload) async {
    await _authenticateIfSupported();
    final isar = await WalletIsar.instance.db();
    final row = await isar.walletProfileEntitys
        .filter()
        .walletIndexEqualTo(walletIndex)
        .findFirst();
    if (row == null) {
      throw const WalletAuthException('未找到指定钱包');
    }
    final profile = _toProfile(row);
    final seedHex = await _readSeedHexRaw(walletIndex);
    if (seedHex == null) {
      throw const WalletAuthException('密钥不可用，请重新导入钱包');
    }
    final seedBytes = Uint8List.fromList(_hexToBytes(seedHex));
    try {
      final pair = Keyring.sr25519.fromSeed(seedBytes);
      pair.ss58Format = profile.ss58;
      final localPubkeyHex = _toHex(pair.bytes().toList(growable: false));
      if (localPubkeyHex.toLowerCase() != profile.pubkeyHex.toLowerCase()) {
        throw const WalletAuthException('本地签名密钥与当前钱包不一致，请重新导入钱包');
      }
      return Uint8List.fromList(pair.sign(payload));
    } finally {
      seedBytes.fillRange(0, seedBytes.length, 0);
    }
  }

  Future<WalletSignResult> signUtf8WithWallet(
    int walletIndex,
    String message,
  ) async {
    await _authenticateIfSupported();
    final isar = await WalletIsar.instance.db();
    final row = await isar.walletProfileEntitys
        .filter()
        .walletIndexEqualTo(walletIndex)
        .findFirst();
    if (row == null) {
      throw const WalletAuthException('未找到指定钱包');
    }
    final profile = _toProfile(row);
    final seedHex = await _readSeedHexRaw(walletIndex);
    if (seedHex == null) {
      throw const WalletAuthException('密钥不可用，请重新导入钱包');
    }

    final seedBytes = Uint8List.fromList(_hexToBytes(seedHex));
    try {
      final pair = Keyring.sr25519.fromSeed(seedBytes);
      pair.ss58Format = _ss58Format;

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

  // ---------------------------------------------------------------------------
  // Seed 派生
  // ---------------------------------------------------------------------------

  Future<List<int>> _mnemonicToMiniSecret(String mnemonic) async {
    final entropy =
        bip39m.Mnemonic.fromSentence(mnemonic, bip39m.Language.english).entropy;
    return CryptoScheme.miniSecretFromEntropy(entropy);
  }

  _DerivedWallet _deriveSr25519FromSeed(List<int> seed) {
    // 拷贝一份用于 Keyring，用完即清零
    final seedBytes = Uint8List.fromList(seed);
    try {
      final pair = Keyring.sr25519.fromSeed(seedBytes);
      pair.ss58Format = _ss58Format;
      final pubkeyBytes = pair.bytes().toList(growable: false);
      final pubkeyHex = _toHex(pubkeyBytes);
      final address = pair.address;
      return _DerivedWallet(address: address, pubkeyHex: pubkeyHex);
    } finally {
      seedBytes.fillRange(0, seedBytes.length, 0);
    }
  }

  // ---------------------------------------------------------------------------
  // Secure Storage
  // ---------------------------------------------------------------------------

  String _seedKey(int walletIndex) => WalletSecureKeys.seedHexV1(walletIndex);

  Future<void> _writeSeedHex(int walletIndex, String seedHex) async {
    await _secureStorage.write(key: _seedKey(walletIndex), value: seedHex);
  }

  static final RegExp _seedHexPattern = RegExp(r'^[0-9a-fA-F]{64}$');

  Future<String?> _readSeedHexRaw(int walletIndex) async {
    final seedHex = await _secureStorage.read(key: _seedKey(walletIndex));
    if (seedHex == null) return null;
    if (!_seedHexPattern.hasMatch(seedHex)) {
      throw const WalletAuthException('钱包密钥数据异常，请重新导入钱包');
    }
    return seedHex;
  }

  Future<void> _deleteSeedHex(int walletIndex) async {
    await _secureStorage.delete(key: _seedKey(walletIndex));
  }

  String _mnemonicKey(int walletIndex) =>
      WalletSecureKeys.mnemonicV1(walletIndex);

  Future<void> _writeMnemonic(int walletIndex, String mnemonic) async {
    // AES-256-GCM 加密后存储
    final encrypted = await MnemonicCipher.encrypt(mnemonic);
    await _secureStorage.write(key: _mnemonicKey(walletIndex), value: encrypted);
  }

  Future<String?> _readMnemonic(int walletIndex) async {
    final stored = await _secureStorage.read(key: _mnemonicKey(walletIndex));
    if (stored == null) return null;

    // 兼容旧版明文格式：检测到明文则自动迁移为加密格式，
    // 迁移后从密文重新解密返回，不直接返回明文引用。
    if (!MnemonicCipher.isEncrypted(stored)) {
      final encrypted = await MnemonicCipher.encrypt(stored);
      await _secureStorage.write(
        key: _mnemonicKey(walletIndex),
        value: encrypted,
      );
      return MnemonicCipher.decrypt(encrypted);
    }

    return MnemonicCipher.decrypt(stored);
  }

  Future<void> _deleteMnemonic(int walletIndex) async {
    await _secureStorage.delete(key: _mnemonicKey(walletIndex));
  }

  /// 获取钱包私钥（seed hex），需设备密码验证。
  Future<String?> getSeedHex(int walletIndex) async {
    await _authenticateIfSupported();
    return _readSeedHexRaw(walletIndex);
  }

  /// 获取钱包助记词，需设备密码验证。
  Future<String?> getMnemonic(int walletIndex) async {
    await _authenticateIfSupported();
    return _readMnemonic(walletIndex);
  }

  /// 强制身份验证。
  ///
  /// 优先使用生物识别（面容/指纹），若不可用则退到设备密码/图案，
  /// 若设备未设置任何安全措施则抛出异常要求用户先设置。
  /// 认证服务异常同样阻止访问。
  static Future<void> _authenticateIfSupported() async {
    bool supported;
    try {
      supported = await _localAuth.isDeviceSupported();
    } on PlatformException catch (e) {
      throw WalletAuthException('认证服务异常：${e.message}，无法访问钱包');
    }

    if (!supported) {
      throw const WalletAuthException(
        '设备未设置锁屏密码或安全措施，请先在系统设置中启用锁屏保护后再使用冷钱包',
      );
    }

    try {
      // biometricOnly: false — 优先弹出生物识别，失败后自动回退到设备 PIN/图案。
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
      // 区分已知的非致命错误和真正的异常
      final code = e.code;
      if (code == 'NotAvailable' || code == 'NotEnrolled') {
        throw const WalletAuthException(
          '设备未设置锁屏密码或安全措施，请先在系统设置中启用锁屏保护后再使用冷钱包',
        );
      }
      throw WalletAuthException('认证服务异常：${e.message}，请稍后重试');
    }
  }

  // ---------------------------------------------------------------------------
  // 内部工具
  // ---------------------------------------------------------------------------

  /// 原子化创建热钱包：在同一个事务中分配 walletIndex 并写入数据库。
  Future<WalletProfile> _appendHotWalletAtomic({
    required String address,
    required String pubkeyHex,
    required String seedHex,
    required String source,
  }) async {
    final isar = await WalletIsar.instance.db();

    late int walletIndex;
    await isar.writeTxn(() async {
      final rows = await isar.walletProfileEntitys
          .where()
          .sortByWalletIndex()
          .findAll();
      final used = rows.map((e) => e.walletIndex).toSet();
      walletIndex = 1;
      while (used.contains(walletIndex)) {
        walletIndex++;
      }

      final entity = WalletProfileEntity()
        ..walletIndex = walletIndex
        ..walletName = '钱包$walletIndex'
        ..walletIcon = 'wallet'
        ..balance = 0
        ..address = address
        ..pubkeyHex = pubkeyHex
        ..alg = 'sr25519'
        ..ss58 = _ss58Format
        ..createdAtMillis = DateTime.now().millisecondsSinceEpoch
        ..source = source
        ..signMode = 'local';
      await isar.walletProfileEntitys.put(entity);

      final settings = await _getSettings(isar);
      settings.activeWalletIndex = walletIndex;
      settings.updatedAtMillis = DateTime.now().millisecondsSinceEpoch;
      await isar.walletSettingsEntitys.put(settings);
    });

    await _writeSeedHex(walletIndex, seedHex);

    return WalletProfile(
      walletIndex: walletIndex,
      walletName: '钱包$walletIndex',
      walletIcon: 'wallet',
      balance: 0,
      address: address,
      pubkeyHex: pubkeyHex,
      alg: 'sr25519',
      ss58: _ss58Format,
      createdAtMillis: DateTime.now().millisecondsSinceEpoch,
      source: source,
      signMode: 'local',
    );
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

  /// 将 seed / 密钥字节列表填零，防止内存残留。
  static void _zeroList(List<int> bytes) {
    for (var i = 0; i < bytes.length; i++) {
      bytes[i] = 0;
    }
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

  List<int> _hexToBytes(String input) {
    final text = input.startsWith('0x') ? input.substring(2) : input;
    if (text.isEmpty || text.length.isOdd) return const <int>[];
    final out = <int>[];
    for (var i = 0; i < text.length; i += 2) {
      out.add(int.parse(text.substring(i, i + 2), radix: 16));
    }
    return out;
  }

  /// 检查公钥是否已存在，重复则抛出异常。
  Future<void> _checkDuplicatePubkey(String pubkeyHex) async {
    final normalized = pubkeyHex.toLowerCase();
    final isar = await WalletIsar.instance.db();
    final rows = await isar.walletProfileEntitys.where().findAll();
    for (final row in rows) {
      if (row.pubkeyHex.toLowerCase() == normalized) {
        throw Exception('该钱包已存在（${row.walletName}），无需重复导入');
      }
    }
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
      groupNames: row.groupNames.isEmpty
          ? const []
          : row.groupNames.split(','),
    );
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
