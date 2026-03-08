import 'package:bip39/bip39.dart' as bip39;
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:isar/isar.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
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
}

class WalletCreationResult {
  const WalletCreationResult({
    required this.profile,
    required this.mnemonic,
  });

  final WalletProfile profile;
  final String mnemonic;
}

class WalletSecret {
  const WalletSecret({required this.profile, required this.mnemonic});

  final WalletProfile profile;
  final String mnemonic;
}

class WalletManager {
  static const int _ss58Format = 2027;
  static const FlutterSecureStorage _secureStorage = FlutterSecureStorage();

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

  Future<WalletSecret?> getLatestWalletSecret() async {
    final active = await getWallet();
    if (active == null) {
      return null;
    }
    final mnemonic = await _readMnemonic(active.walletIndex);
    if (mnemonic == null || mnemonic.isEmpty) {
      return null;
    }
    return WalletSecret(profile: active, mnemonic: mnemonic);
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

  Future<WalletSecret?> getWalletSecretByIndex(int walletIndex) async {
    final isar = await WalletIsar.instance.db();
    final row = await isar.walletProfileEntitys
        .filter()
        .walletIndexEqualTo(walletIndex)
        .findFirst();
    if (row == null) {
      return null;
    }

    final mnemonic = await _readMnemonic(walletIndex);
    if (mnemonic == null || mnemonic.isEmpty) {
      return null;
    }

    return WalletSecret(
      profile: _toProfile(row),
      mnemonic: mnemonic,
    );
  }

  Future<WalletCreationResult> createWallet() async {
    final mnemonic = bip39.generateMnemonic();
    final derived = await _deriveSr25519Ss58Address(mnemonic);
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
    );

    await _appendWallet(profile, mnemonic);
    return WalletCreationResult(profile: profile, mnemonic: mnemonic);
  }

  Future<WalletProfile> importWallet(String mnemonic) async {
    final trimmed = mnemonic.trim();
    if (!bip39.validateMnemonic(trimmed)) {
      throw Exception('助记词无效，请检查拼写和空格');
    }

    final derived = await _deriveSr25519Ss58Address(trimmed);
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
    );

    await _appendWallet(profile, trimmed);
    return profile;
  }

  Future<String?> getMnemonic() async {
    final latest = await getLatestWalletSecret();
    return latest?.mnemonic;
  }

  Future<void> clearWallet() async {
    final isar = await WalletIsar.instance.db();
    final wallets = await isar.walletProfileEntitys.where().findAll();
    for (final row in wallets) {
      await _deleteMnemonic(row.walletIndex);
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

  Future<void> _appendWallet(WalletProfile profile, String mnemonic) async {
    final isar = await WalletIsar.instance.db();
    final entity = _toEntity(profile);

    await _writeMnemonic(profile.walletIndex, mnemonic);
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
      ..faceAuthEnabled = true
      ..updatedAtMillis = DateTime.now().millisecondsSinceEpoch;
    await isar.writeTxn(() async {
      await isar.walletSettingsEntitys.put(created);
    });
    return created;
  }

  String _mnemonicKey(int walletIndex) =>
      WalletSecureKeys.mnemonicV1(walletIndex);

  Future<void> _writeMnemonic(int walletIndex, String mnemonic) async {
    await _secureStorage.write(key: _mnemonicKey(walletIndex), value: mnemonic);
  }

  Future<String?> _readMnemonic(int walletIndex) async {
    return _secureStorage.read(key: _mnemonicKey(walletIndex));
  }

  Future<void> _deleteMnemonic(int walletIndex) async {
    await _secureStorage.delete(key: _mnemonicKey(walletIndex));
  }

  Future<_DerivedWallet> _deriveSr25519Ss58Address(String mnemonic) async {
    final keyring = Keyring.sr25519;
    final pair = await keyring.fromMnemonic(mnemonic);
    pair.ss58Format = _ss58Format;
    final pubkeyHex = _toHex(pair.bytes().toList(growable: false));
    return _DerivedWallet(address: pair.address, pubkeyHex: pubkeyHex);
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
      ..source = profile.source;
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
