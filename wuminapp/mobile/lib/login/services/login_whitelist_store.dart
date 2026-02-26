import 'dart:convert';

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

      final aud = _parseMap(decoded['aud_whitelist']);
      final origin = _parseMap(decoded['origin_whitelist']);
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
    await prefs.setString(_kWhitelistConfig, jsonEncode(config.toJson()));
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
