import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/features/observe_accounts/observe_accounts_model.dart';
import 'package:wuminapp_mobile/services/api_client.dart';
import 'package:wuminapp_mobile/services/wallet_type_service.dart';

class ObservedAccountService {
  static const int _ss58Format = 2027;
  static const _kObservedAccounts = 'observe.accounts';
  final Keyring _keyring = Keyring.sr25519;
  final ApiClient _apiClient = ApiClient();

  Future<List<ObservedAccount>> getObservedAccounts() async {
    final prefs = await SharedPreferences.getInstance();
    final stored = _decode(prefs.getString(_kObservedAccounts));
    if (stored.isEmpty) {
      return stored;
    }
    final refreshed = await _refreshBalances(stored);
    final changed = _hasBalanceChanged(stored, refreshed);
    if (changed) {
      await _save(refreshed);
    }
    return refreshed;
  }

  Future<void> addObservedAccount(String input) async {
    final value = _cleanInput(input);
    if (value.isEmpty) {
      throw Exception('请输入公钥或地址');
    }

    final pubkey = _normalizeInputToPubkey(value);
    if (pubkey == null) {
      throw Exception('输入格式无效，请输入 32 字节公钥或 SS58 地址');
    }

    final address = _encodeAddress(pubkey);
    final current = await getObservedAccounts();
    final exists = current.any((it) => it.address == address);
    if (exists) {
      throw Exception('该观察账户已存在');
    }

    final role = WalletTypeService.resolveWalletType(pubkey);
    final orgName = role == WalletTypeService.defaultType
        ? '自定义观察账户'
        : _extractOrgName(role);

    final next = List<ObservedAccount>.from(current)
      ..add(
        ObservedAccount(
          id: 'manual:$pubkey',
          orgName: orgName,
          publicKey: pubkey,
          address: address,
          balance: 0,
          source: 'manual',
        ),
      );
    await _save(next);
  }

  Future<void> removeObservedAccount(ObservedAccount item) async {
    final current = await getObservedAccounts();
    current.removeWhere((it) => it.id == item.id);
    await _save(current);
  }

  Future<void> renameObservedAccount(String id, String orgName) async {
    final name = orgName.trim();
    if (name.isEmpty) {
      throw Exception('观察账户名称不能为空');
    }
    final current = await getObservedAccounts();
    bool found = false;
    final updated = current
        .map((it) {
          if (it.id != id) {
            return it;
          }
          found = true;
          return it.copyWith(orgName: name);
        })
        .toList(growable: false);
    if (!found) {
      throw Exception('未找到观察账户');
    }
    await _save(updated);
  }

  String? _normalizeInputToPubkey(String input) {
    final direct = _normalizeHexPubkey(input);
    if (direct != null) {
      return direct;
    }
    try {
      final bytes = _keyring.decodeAddress(_cleanInput(input));
      return _toHex(bytes.toList(growable: false));
    } catch (_) {
      return null;
    }
  }

  String? _normalizeHexPubkey(String input) {
    var v = _cleanInput(input).toLowerCase();
    if (v.startsWith('0x')) {
      v = v.substring(2);
    }
    final ok = RegExp(r'^[0-9a-f]{64}$').hasMatch(v);
    if (!ok) {
      return null;
    }
    return v;
  }

  String _encodeAddress(String pubkeyHex) {
    final bytes = <int>[];
    for (var i = 0; i < pubkeyHex.length; i += 2) {
      bytes.add(int.parse(pubkeyHex.substring(i, i + 2), radix: 16));
    }
    return _keyring.encodeAddress(bytes, _ss58Format);
  }

  String _extractOrgName(String roleName) {
    if (roleName.endsWith('管理员')) {
      return roleName.substring(0, roleName.length - 3);
    }
    return roleName;
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

  String _cleanInput(String input) {
    var v = input.trim();
    v = v.replaceAll(' ', '');
    v = v.replaceAll('\n', '');
    v = v.replaceAll('\r', '');
    v = v.replaceAll('\t', '');
    v = v.replaceAll('"', '');
    v = v.replaceAll("'", '');
    v = v.replaceAll('`', '');
    v = v.replaceAll(',', '');
    v = v.replaceAll('，', '');
    v = v.replaceAll('。', '');
    v = v.replaceAll('；', '');
    v = v.replaceAll(';', '');
    return v;
  }

  Future<void> _save(List<ObservedAccount> items) async {
    final prefs = await SharedPreferences.getInstance();
    final data = items
        .map(
          (it) => {
            'id': it.id,
            'orgName': it.orgName,
            'publicKey': it.publicKey,
            'address': it.address,
            'balance': it.balance,
            'source': it.source,
          },
        )
        .toList(growable: false);
    await prefs.setString(_kObservedAccounts, jsonEncode(data));
  }

  Future<List<ObservedAccount>> _refreshBalances(
    List<ObservedAccount> items,
  ) async {
    final out = <ObservedAccount>[];
    for (final item in items) {
      try {
        final data = await _apiClient.fetchWalletBalance(item.address);
        out.add(item.copyWith(balance: data.balance));
      } catch (e) {
        try {
          final fallback = await _apiClient.fetchWalletBalance(
            '0x${item.publicKey}',
          );
          out.add(item.copyWith(balance: fallback.balance));
        } catch (fallbackError) {
          debugPrint(
            'observe account balance refresh failed: ${item.address} / ${item.publicKey} '
            'err=$e fallbackErr=$fallbackError',
          );
          out.add(item);
        }
      }
    }
    return out;
  }

  bool _hasBalanceChanged(
    List<ObservedAccount> previous,
    List<ObservedAccount> next,
  ) {
    if (previous.length != next.length) {
      return true;
    }
    for (var i = 0; i < previous.length; i++) {
      if (previous[i].balance != next[i].balance) {
        return true;
      }
    }
    return false;
  }

  List<ObservedAccount> _decode(String? raw) {
    if (raw == null || raw.isEmpty) {
      return <ObservedAccount>[];
    }
    final decoded = jsonDecode(raw);
    if (decoded is! List) {
      return <ObservedAccount>[];
    }
    final out = <ObservedAccount>[];
    for (final item in decoded) {
      if (item is! Map) {
        continue;
      }
      final m = item.map((k, v) => MapEntry(k.toString(), v));
      final rawBalance = m['balance'];
      final balance = switch (rawBalance) {
        num v => v.toDouble(),
        String v => double.tryParse(v) ?? 0.0,
        _ => 0.0,
      };
      var pubkey = _normalizeHexPubkey(m['publicKey']?.toString() ?? '');
      final address = (m['address']?.toString() ?? '').trim();
      if (pubkey == null && address.isNotEmpty) {
        pubkey = _normalizeInputToPubkey(address);
      }
      if (pubkey == null) {
        continue;
      }
      final normalizedAddress =
          address.isNotEmpty ? address : _encodeAddress(pubkey);
      final role = WalletTypeService.resolveWalletType(pubkey);
      out.add(
        ObservedAccount(
          id: (m['id']?.toString() ?? 'manual:$pubkey'),
          orgName: (m['orgName']?.toString() ?? '').trim().isEmpty
              ? (role == WalletTypeService.defaultType
                  ? '自定义观察账户'
                  : _extractOrgName(role))
              : m['orgName'].toString(),
          publicKey: pubkey,
          address: normalizedAddress,
          balance: balance,
          source: (m['source']?.toString() ?? 'manual'),
        ),
      );
    }
    return out;
  }
}
