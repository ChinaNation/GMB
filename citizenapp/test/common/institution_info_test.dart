import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/governance/shared/institution_info.dart';
import 'package:citizenapp/governance/organization-manage/institution_registry.dart';

void main() {
  test('内置机构身份编码为 mainAccount AccountId', () {
    final mainAccount = 'aa' * 32;
    final id = institutionIdentityToAccountId(
      'LN001-GCB05-944805165-2026',
      mainAccount: mainAccount,
    );

    expect(id.length, 32);
    expect(id, List<int>.filled(32, 0xaa));
  });

  test('注册机构账户身份编码为机构 AccountId', () {
    final address = '11' * 32;
    final identity = registeredAccountIdentity(address);
    final id = institutionIdentityToAccountId(identity);

    expect(id.length, 32);
    expect(id, List<int>.filled(32, 0x11));
    expect(findInstitutionByAccountId(id)?.account, address);
  });

  test('个人多签身份编码为个人多签 AccountId', () {
    final address = '22' * 32;
    final id = institutionIdentityToAccountId('personal-account:$address');

    expect(id.length, 32);
    expect(id, List<int>.filled(32, 0x22));
    expect(
        findInstitutionByAccountId(id)?.cidNumber, 'personal-account:$address');
  });
}
