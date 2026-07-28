import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/my/myid/citizen_identity_chain_reader.dart';
import 'package:citizenapp/my/myid/identity_account_resolver.dart';
import 'package:citizenapp/my/myid/widgets/identity_registration_gate.dart';

/// 恒返回「已注册匿名 CID」的判据 resolver(不触发 smoldot)。
class _RegisteredResolver extends IdentityAccountResolver {
  @override
  Future<ResolvedIdentity?> resolve() async => ResolvedIdentity(
        accountId: '0x${'11' * 32}',
        ss58Address: 'ss58',
        accountIndex: 0,
        snapshot: CitizenIdentityChainSnapshot(
          cidNumber: 'GD-CTZN1-8F3A2B',
          accountId: Uint8List(32),
          votingIdentity: null,
          candidateIdentity: null,
        ),
      );
}

/// 让 [IdentityRegistrationGate] 在**页面** widget 测试中直接放行(注入已注册判据,
/// 不触发真 smoldot 链读)。在被 gate 包裹的页面测试 `main()` 顶部调用一次即可。
/// gate 本身的门控行为由 `identity_registration_gate_test.dart` 独立覆盖。
void useRegisteredIdentityGate() {
  setUp(() => IdentityRegistrationGate.debugResolver = _RegisteredResolver());
  tearDown(() => IdentityRegistrationGate.debugResolver = null);
}
