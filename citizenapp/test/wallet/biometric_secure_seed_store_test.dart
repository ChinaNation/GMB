import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/wallet/core/biometric_secure_seed_store.dart';
import 'package:citizenapp/wallet/core/secure_seed_store.dart';

/// 用内存 map + 可注入错误来伪装 `biometric_storage` 原生侧。
/// 抛出的是 `PlatformException`，让插件真实的 `_transformErrors`
/// 把 `AuthError:*` 转成 `AuthException`，从而覆盖真实错误转换路径。
class _FakeNative {
  final Map<String, String> storage = <String, String>{};
  final Map<String, Map<String, dynamic>> initOptions =
      <String, Map<String, dynamic>>{};
  String canAuthenticate = 'Success';
  PlatformException? readError;

  Future<Object?> handle(MethodCall call) async {
    switch (call.method) {
      case 'canAuthenticate':
        return canAuthenticate;
      case 'init':
        final args = (call.arguments as Map).cast<String, dynamic>();
        final name = args['name'] as String;
        initOptions[name] = (args['options'] as Map).cast<String, dynamic>();
        return true;
      case 'read':
        final err = readError;
        if (err != null) {
          throw err;
        }
        final args = (call.arguments as Map).cast<String, dynamic>();
        return storage[args['name'] as String];
      case 'write':
        final args = (call.arguments as Map).cast<String, dynamic>();
        storage[args['name'] as String] = args['content'] as String;
        return null;
      case 'delete':
        final args = (call.arguments as Map).cast<String, dynamic>();
        storage.remove(args['name'] as String);
        return true;
      default:
        return null;
    }
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  const channel = MethodChannel('biometric_storage');
  late _FakeNative native;
  late BiometricSecureSeedStore store;

  setUp(() {
    native = _FakeNative();
    store = BiometricSecureSeedStore();
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(channel, native.handle);
  });

  tearDown(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(channel, null);
  });

  group('BiometricSecureSeedStore — 金库配置（每次验证 + 允许设备密码）', () {
    test('putSeed 写入 wallet_seed_<id>，允许设备凭证、每次验证', () async {
      await store.putSeed(1, 'a' * 64);

      expect(native.storage['wallet_seed_1'], 'a' * 64);
      final opts = native.initOptions['wallet_seed_1']!;
      // validity 正数（非 0）：0 秒令牌会在加密前过期报 KEY_USER_NOT_AUTHENTICATED；
      // biometricOnly false = 允许图案/PIN，覆盖无生物识别机型。插件每次读写都
      // 重新弹验证，故仍是「每次操作一次验证」。
      expect(opts['authenticationValidityDurationSeconds'], 10);
      expect(opts['androidBiometricOnly'], false);
      expect(opts['darwinBiometricOnly'], false);
    });

    test('putMnemonic 写入 wallet_recovery_<id>，配置同 seed 金库', () async {
      await store.putMnemonic(2, 'legal winner thank year');

      expect(native.storage['wallet_recovery_2'], 'legal winner thank year');
      final opts = native.initOptions['wallet_recovery_2']!;
      expect(opts['authenticationValidityDurationSeconds'], 10);
      expect(opts['androidBiometricOnly'], false);
      expect(opts['darwinBiometricOnly'], false);
    });
  });

  group('BiometricSecureSeedStore — 读写往返与删除', () {
    test('readSeed 返回写入的值', () async {
      await store.putSeed(1, 'b' * 64);
      expect(await store.readSeed(1), 'b' * 64);
    });

    test('deleteSeed 移除条目后 readSeed 返回 null', () async {
      await store.putSeed(1, 'c' * 64);
      await store.deleteSeed(1);
      expect(native.storage.containsKey('wallet_seed_1'), isFalse);
      expect(await store.readSeed(1), isNull);
    });

    test('readMnemonic 缺失返回 null', () async {
      expect(await store.readMnemonic(9), isNull);
    });
  });

  group('BiometricSecureSeedStore — 错误分类', () {
    test('用户取消 → AuthCancelled（不自愈）', () async {
      await store.putSeed(1, 'd' * 64);
      native.readError =
          PlatformException(code: 'AuthError:UserCanceled', message: '用户取消');

      expect(
        () => store.readSeed(1),
        throwsA(isA<AuthCancelled>()),
      );
    });

    test('未映射 AuthError（unknown）严档读 → SeedKeyInvalidated（触发自愈）', () async {
      await store.putSeed(1, 'e' * 64);
      native.readError =
          PlatformException(code: 'AuthError:KeyInvalidated', message: '失效');

      expect(
        () => store.readSeed(1),
        throwsA(isA<SeedKeyInvalidated>()),
      );
    });

    test('宽档读取的 unknown 错误 → SecureStoreUnavailable（不误判自愈）', () async {
      await store.putMnemonic(1, 'seed words');
      native.readError =
          PlatformException(code: 'AuthError:KeyInvalidated', message: '失效');

      expect(
        () => store.readMnemonic(1),
        throwsA(isA<SecureStoreUnavailable>()),
      );
    });

    test('无锁屏设备 putSeed → NoDeviceCredential（D3 fail-closed）', () async {
      native.canAuthenticate = 'ErrorPasscodeNotSet';

      expect(
        () => store.putSeed(1, 'f' * 64),
        throwsA(isA<NoDeviceCredential>()),
      );
      // fail-closed：密钥绝不落盘。
      expect(native.storage.containsKey('wallet_seed_1'), isFalse);
    });
  });

  group('BiometricSecureSeedStore — authStatus 映射', () {
    test('Success → available', () async {
      native.canAuthenticate = 'Success';
      expect(await store.authStatus(), SecureAuthStatus.available);
    });

    test('ErrorPasscodeNotSet → noDeviceLock', () async {
      native.canAuthenticate = 'ErrorPasscodeNotSet';
      expect(await store.authStatus(), SecureAuthStatus.noDeviceLock);
    });

    test('ErrorNoHardware → unsupported', () async {
      native.canAuthenticate = 'ErrorNoHardware';
      expect(await store.authStatus(), SecureAuthStatus.unsupported);
    });
  });
}
