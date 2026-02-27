import 'dart:convert';

import 'package:bip39/bip39.dart' as bip39;
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:shared_preferences/shared_preferences.dart';

class WalletProfile {
  const WalletProfile({
    required this.walletIndex,
    required this.address,
    required this.pubkeyHex,
    required this.alg,
    required this.ss58,
    required this.createdAtMillis,
    required this.source,
  });

  final int walletIndex;
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

class WalletService {
  static const int _ss58Format = 2027;
  static const _kHasWallet = 'wallet.has_wallet';
  static const _kWalletCounter = 'wallet.counter';
  static const _kWallets = 'wallet.items';

  // Legacy single-wallet keys.
  static const _kWalletIndex = 'wallet.index';
  static const _kAddress = 'wallet.address';
  static const _kPubkeyHex = 'wallet.pubkey_hex';
  static const _kAlg = 'wallet.alg';
  static const _kSs58 = 'wallet.ss58';
  static const _kCreatedAtMillis = 'wallet.created_at_millis';
  static const _kSource = 'wallet.source';
  static const _kMnemonic = 'wallet.mnemonic';

  Future<List<WalletProfile>> getWallets() async {
    final records = await _loadWalletRecords();
    return records
        .map(
          (r) => WalletProfile(
            walletIndex: r.walletIndex,
            address: r.address,
            pubkeyHex: r.pubkeyHex,
            alg: r.alg,
            ss58: r.ss58,
            createdAtMillis: r.createdAtMillis,
            source: r.source,
          ),
        )
        .toList(growable: false);
  }

  Future<WalletProfile?> getWallet() async {
    final wallets = await getWallets();
    if (wallets.isEmpty) {
      return null;
    }
    return wallets.last;
  }

  Future<WalletSecret?> getLatestWalletSecret() async {
    final records = await _loadWalletRecords();
    if (records.isEmpty) {
      return null;
    }
    final record = records.last;
    return WalletSecret(
      profile: WalletProfile(
        walletIndex: record.walletIndex,
        address: record.address,
        pubkeyHex: record.pubkeyHex,
        alg: record.alg,
        ss58: record.ss58,
        createdAtMillis: record.createdAtMillis,
        source: record.source,
      ),
      mnemonic: record.mnemonic,
    );
  }

  Future<WalletSecret?> getWalletSecretByIndex(int walletIndex) async {
    final records = await _loadWalletRecords();
    for (final record in records) {
      if (record.walletIndex == walletIndex) {
        return WalletSecret(
          profile: WalletProfile(
            walletIndex: record.walletIndex,
            address: record.address,
            pubkeyHex: record.pubkeyHex,
            alg: record.alg,
            ss58: record.ss58,
            createdAtMillis: record.createdAtMillis,
            source: record.source,
          ),
          mnemonic: record.mnemonic,
        );
      }
    }
    return null;
  }

  Future<WalletCreationResult> createWallet() async {
    final mnemonic = bip39.generateMnemonic();
    final derived = await _deriveSr25519Ss58Address(mnemonic);
    final walletIndex = await _nextWalletIndex();

    final profile = WalletProfile(
      walletIndex: walletIndex,
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
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove(_kHasWallet);
    await prefs.remove(_kWallets);

    // Legacy cleanup.
    await prefs.remove(_kWalletIndex);
    await prefs.remove(_kAddress);
    await prefs.remove(_kPubkeyHex);
    await prefs.remove(_kAlg);
    await prefs.remove(_kSs58);
    await prefs.remove(_kCreatedAtMillis);
    await prefs.remove(_kSource);
    await prefs.remove(_kMnemonic);
  }

  Future<void> deleteWallet(int walletIndex) async {
    final prefs = await SharedPreferences.getInstance();
    final records = await _loadWalletRecords();
    records.removeWhere((r) => r.walletIndex == walletIndex);
    if (records.isEmpty) {
      await clearWallet();
      return;
    }
    await prefs.setBool(_kHasWallet, true);
    await _saveWalletRecords(records);
  }

  Future<int> _nextWalletIndex() async {
    final prefs = await SharedPreferences.getInstance();
    final current = prefs.getInt(_kWalletCounter) ?? 0;
    final next = current + 1;
    await prefs.setInt(_kWalletCounter, next);
    return next;
  }

  Future<void> _appendWallet(WalletProfile profile, String mnemonic) async {
    final prefs = await SharedPreferences.getInstance();
    final records = await _loadWalletRecords();
    records.add(
      _WalletRecord(
        walletIndex: profile.walletIndex,
        address: profile.address,
        pubkeyHex: profile.pubkeyHex,
        alg: profile.alg,
        ss58: profile.ss58,
        createdAtMillis: profile.createdAtMillis,
        source: profile.source,
        mnemonic: mnemonic,
      ),
    );

    await prefs.setBool(_kHasWallet, true);
    await _saveWalletRecords(records);
  }

  Future<List<_WalletRecord>> _loadWalletRecords() async {
    final prefs = await SharedPreferences.getInstance();
    final rawList = prefs.getString(_kWallets);
    if (rawList != null && rawList.isNotEmpty) {
      final decoded = jsonDecode(rawList);
      if (decoded is List) {
        final out = <_WalletRecord>[];
        for (final item in decoded) {
          if (item is Map<String, dynamic>) {
            out.add(_WalletRecord.fromJson(item));
          } else if (item is Map) {
            out.add(
              _WalletRecord.fromJson(
                item.map((k, v) => MapEntry(k.toString(), v)),
              ),
            );
          }
        }
        out.sort((a, b) => a.walletIndex.compareTo(b.walletIndex));
        return out;
      }
    }

    final migrated = await _migrateLegacySingleWallet();
    if (migrated != null) {
      await _saveWalletRecords([migrated]);
      return [migrated];
    }

    return <_WalletRecord>[];
  }

  Future<_WalletRecord?> _migrateLegacySingleWallet() async {
    final prefs = await SharedPreferences.getInstance();
    final hasWallet = prefs.getBool(_kHasWallet) ?? false;
    if (!hasWallet) {
      return null;
    }

    final walletIndex = prefs.getInt(_kWalletIndex);
    final address = prefs.getString(_kAddress);
    final pubkeyHex = prefs.getString(_kPubkeyHex);
    final alg = prefs.getString(_kAlg);
    final ss58 = prefs.getInt(_kSs58);
    final createdAt = prefs.getInt(_kCreatedAtMillis);
    final source = prefs.getString(_kSource);
    final mnemonic = prefs.getString(_kMnemonic);

    if (walletIndex == null ||
        address == null ||
        pubkeyHex == null ||
        alg == null ||
        ss58 == null ||
        createdAt == null ||
        source == null ||
        mnemonic == null ||
        mnemonic.isEmpty) {
      return null;
    }

    return _WalletRecord(
      walletIndex: walletIndex,
      address: address,
      pubkeyHex: pubkeyHex,
      alg: alg,
      ss58: ss58,
      createdAtMillis: createdAt,
      source: source,
      mnemonic: mnemonic,
    );
  }

  Future<void> _saveWalletRecords(List<_WalletRecord> records) async {
    final prefs = await SharedPreferences.getInstance();
    final data = records.map((e) => e.toJson()).toList(growable: false);
    await prefs.setString(_kWallets, jsonEncode(data));
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
}

class _WalletRecord {
  const _WalletRecord({
    required this.walletIndex,
    required this.address,
    required this.pubkeyHex,
    required this.alg,
    required this.ss58,
    required this.createdAtMillis,
    required this.source,
    required this.mnemonic,
  });

  final int walletIndex;
  final String address;
  final String pubkeyHex;
  final String alg;
  final int ss58;
  final int createdAtMillis;
  final String source;
  final String mnemonic;

  factory _WalletRecord.fromJson(Map<String, dynamic> json) {
    return _WalletRecord(
      walletIndex: (json['walletIndex'] as num).toInt(),
      address: json['address'] as String,
      pubkeyHex: json['pubkeyHex'] as String,
      alg: json['alg'] as String,
      ss58: (json['ss58'] as num).toInt(),
      createdAtMillis: (json['createdAtMillis'] as num).toInt(),
      source: json['source'] as String,
      mnemonic: json['mnemonic'] as String,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'walletIndex': walletIndex,
      'address': address,
      'pubkeyHex': pubkeyHex,
      'alg': alg,
      'ss58': ss58,
      'createdAtMillis': createdAtMillis,
      'source': source,
      'mnemonic': mnemonic,
    };
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
