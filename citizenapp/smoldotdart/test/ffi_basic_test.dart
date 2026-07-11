import 'package:test/test.dart';
import 'package:smoldot/src/bindings.dart';

void main() {
  group('FFI Basic Tests', () {
    late SmoldotBindings bindings;

    setUp(() {
      bindings = SmoldotBindings();
    });

    test('should load library and get version', () {
      final version = bindings.getVersion();
      print('Smoldot FFI version: $version');
      expect(version, isNotEmpty);
      expect(version, equals('1.0.0')); // 与 CitizenApp 实际构建的 rust/Cargo.toml 一致。
    });

    test('should initialize client with config', () {
      final configJson =
          '{"maxLogLevel":3,"systemName":"Test","systemVersion":"1.0.0"}';

      expect(() {
        final handle = bindings.initClient(configJson);
        print('Client initialized with handle: $handle');
        expect(handle, greaterThan(0));

        // Clean up
        bindings.destroyClient(handle);
      }, returnsNormally);
    });
  });
}
