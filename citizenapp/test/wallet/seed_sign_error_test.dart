import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/wallet/core/secure_seed_store.dart';
import 'package:citizenapp/wallet/core/seed_sign_error.dart';

void main() {
  group('seedSignErrorMessage', () {
    const cases = <SecureSeedException>[
      AuthCancelled('cancel'),
      NoDeviceCredential('no-lock'),
      SecureStoreUnavailable('io'),
      SeedKeyInvalidated('rekey'),
    ];

    test('每个 SecureSeedException 子型都映射到非空提示（永不静默）', () {
      for (final e in cases) {
        expect(seedSignErrorMessage(e).trim(), isNotEmpty,
            reason: '${e.runtimeType} 必须有面向用户的提示');
      }
    });

    test('四类文案互不相同（用户能区分取消 / 无锁屏 / 金库错误 / 失效）', () {
      final unique = cases.map(seedSignErrorMessage).toSet();
      expect(unique.length, cases.length);
    });

    test('用户取消映射为「已取消签名」', () {
      expect(seedSignErrorMessage(const AuthCancelled('x')), '已取消签名');
    });
  });
}
