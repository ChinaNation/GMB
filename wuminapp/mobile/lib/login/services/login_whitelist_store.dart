import 'dart:convert';
import 'dart:math';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:shared_preferences/shared_preferences.dart';

class LoginWhitelistConfig {
  const LoginWhitelistConfig({
    required this.audWhitelist,
    required this.originWhitelist,
  });

  final Map<String, Set<String>> audWhitelist;
  final Map<String, Set<String>> originWhitelist;

  Map<String, dynamic> toJson() {
    return {
      'aud_whitelist': audWhitelist.map(
        (k, v) => MapEntry(k, v.toList(growable: false)),
      ),
      'origin_whitelist': originWhitelist.map(
        (k, v) => MapEntry(k, v.toList(growable: false)),
      ),
    };
  }
}

class LoginWhitelistStore {
  static const String _kWhitelistConfig = 'login.whitelist_config.v1';
  static const String _kWhitelistHmacSecret = 'login.whitelist_hmac_secret.v1';
  static const FlutterSecureStorage _secureStorage = FlutterSecureStorage();

  static const Map<String, Set<String>> defaultAudWhitelist = {
    'cpms': {'cpms-local-app'},
    'sfid': {'sfid-local-app'},
    'citizenchain': {'citizenchain-front'},
  };

  static const Map<String, Set<String>> defaultOriginWhitelist = {
    'cpms': {'cpms-device-id'},
    'sfid': {'sfid-device-id'},
    'citizenchain': {'citizenchain-device-id'},
  };

  Future<LoginWhitelistConfig> load() async {
    final prefs = await SharedPreferences.getInstance();
    final raw = prefs.getString(_kWhitelistConfig);
    if (raw == null || raw.isEmpty) {
      return const LoginWhitelistConfig(
        audWhitelist: defaultAudWhitelist,
        originWhitelist: defaultOriginWhitelist,
      );
    }

    try {
      final decoded = jsonDecode(raw);
      if (decoded is! Map<String, dynamic>) {
        return const LoginWhitelistConfig(
          audWhitelist: defaultAudWhitelist,
          originWhitelist: defaultOriginWhitelist,
        );
      }
      final payload = await _extractVerifiedPayload(decoded);
      if (payload == null) {
        return const LoginWhitelistConfig(
          audWhitelist: defaultAudWhitelist,
          originWhitelist: defaultOriginWhitelist,
        );
      }
      final aud = _parseMap(payload['aud_whitelist']);
      final origin = _parseMap(payload['origin_whitelist']);
      return LoginWhitelistConfig(
        audWhitelist: aud.isEmpty ? defaultAudWhitelist : aud,
        originWhitelist: origin.isEmpty ? defaultOriginWhitelist : origin,
      );
    } catch (_) {
      return const LoginWhitelistConfig(
        audWhitelist: defaultAudWhitelist,
        originWhitelist: defaultOriginWhitelist,
      );
    }
  }

  Future<void> save(LoginWhitelistConfig config) async {
    final prefs = await SharedPreferences.getInstance();
    final payload = config.toJson();
    final sig = await _signPayload(payload);
    final envelope = {
      'ver': 1,
      'payload': payload,
      'sig': sig,
    };
    await prefs.setString(_kWhitelistConfig, jsonEncode(envelope));
  }

  Future<Map<String, dynamic>?> _extractVerifiedPayload(
    Map<String, dynamic> decoded,
  ) async {
    if (decoded.containsKey('payload') && decoded.containsKey('sig')) {
      final payload = decoded['payload'];
      final sig = decoded['sig']?.toString() ?? '';
      if (payload is! Map<String, dynamic> || sig.isEmpty) {
        return null;
      }
      final expected = await _signPayload(payload);
      if (!_constantTimeEquals(sig, expected)) {
        return null;
      }
      return payload;
    }
    // Legacy unsigned config.
    return decoded;
  }

  Future<String> _signPayload(Map<String, dynamic> payload) async {
    final secret = await _getOrCreateHmacSecret();
    final message = jsonEncode(payload);
    final hmac = Hmac(sha256, utf8.encode(secret));
    final digest = hmac.convert(utf8.encode(message));
    return digest.bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  }

  Future<String> _getOrCreateHmacSecret() async {
    final existing = await _secureStorage.read(key: _kWhitelistHmacSecret);
    if (existing != null && existing.isNotEmpty) {
      return existing;
    }
    final random = Random.secure();
    final bytes =
        Uint8List.fromList(List<int>.generate(32, (_) => random.nextInt(256)));
    final created = base64UrlEncode(bytes);
    await _secureStorage.write(key: _kWhitelistHmacSecret, value: created);
    return created;
  }

  bool _constantTimeEquals(String left, String right) {
    final leftBytes = utf8.encode(left);
    final rightBytes = utf8.encode(right);
    final maxLen = leftBytes.length > rightBytes.length
        ? leftBytes.length
        : rightBytes.length;
    var diff = leftBytes.length ^ rightBytes.length;
    for (var i = 0; i < maxLen; i++) {
      final l = i < leftBytes.length ? leftBytes[i] : 0;
      final r = i < rightBytes.length ? rightBytes[i] : 0;
      diff |= (l ^ r);
    }
    return diff == 0;
  }

  Map<String, Set<String>> _parseMap(dynamic raw) {
    if (raw is! Map) {
      return {};
    }
    final out = <String, Set<String>>{};
    for (final entry in raw.entries) {
      final key = entry.key.toString();
      final value = entry.value;
      if (value is List) {
        out[key] = value.map((e) => e.toString()).toSet();
      }
    }
    return out;
  }
}
