import 'dart:convert';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/login/services/login_whitelist_store.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  const secureStorageChannel =
      MethodChannel('plugins.it_nomads.com/flutter_secure_storage');
  final secureStorage = <String, String>{};

  setUp(() async {
    SharedPreferences.setMockInitialValues(<String, Object>{});
    secureStorage.clear();
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(secureStorageChannel, (call) async {
      final args = (call.arguments as Map?)?.cast<String, dynamic>() ??
          <String, dynamic>{};
      final key = args['key']?.toString();
      switch (call.method) {
        case 'read':
          return key == null ? null : secureStorage[key];
        case 'write':
          if (key != null) {
            secureStorage[key] = args['value']?.toString() ?? '';
          }
          return null;
        case 'delete':
          if (key != null) {
            secureStorage.remove(key);
          }
          return null;
        case 'deleteAll':
          secureStorage.clear();
          return null;
        case 'containsKey':
          return key != null && secureStorage.containsKey(key);
        case 'readAll':
          return Map<String, String>.from(secureStorage);
        default:
          return null;
      }
    });
  });

  tearDown(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(secureStorageChannel, null);
  });

  group('LoginWhitelistStore', () {
    test('load should return defaults when no config exists', () async {
      final store = LoginWhitelistStore();

      final config = await store.load();

      expect(config.audWhitelist['cpms'], contains('cpms-local-app'));
      expect(config.audWhitelist['sfid'], contains('sfid-local-app'));
    });

    test('save + load should keep aud whitelist', () async {
      final store = LoginWhitelistStore();
      const expected = LoginWhitelistConfig(
        audWhitelist: <String, Set<String>>{
          'cpms': <String>{'cpms-local-app', 'cpms-lab-app'},
          'sfid': <String>{'sfid-local-app'},
        },
      );

      await store.save(expected);
      final loaded = await store.load();

      expect(loaded.audWhitelist['cpms'], contains('cpms-local-app'));
      expect(loaded.audWhitelist['cpms'], contains('cpms-lab-app'));
      expect(loaded.audWhitelist['sfid'], contains('sfid-local-app'));
    });

    test('load should fallback to defaults when signature is invalid',
        () async {
      SharedPreferences.setMockInitialValues(<String, Object>{
        'login.whitelist_config.v1': jsonEncode({
          'ver': 1,
          'payload': {
            'aud_whitelist': {
              'cpms': ['tampered-app'],
            },
          },
          'sig': 'deadbeef',
        }),
      });
      final store = LoginWhitelistStore();

      final loaded = await store.load();

      expect(loaded.audWhitelist['cpms'], contains('cpms-local-app'));
      expect(loaded.audWhitelist['cpms'], isNot(contains('tampered-app')));
    });
  });
}
