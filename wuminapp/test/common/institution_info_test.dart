import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/governance/shared/institution_info.dart';
import 'package:wuminapp_mobile/governance/organization-manage/institution_registry.dart';

void main() {
  test('内置机构身份编码为 mainAddress AccountId', () {
    final mainAddress = 'aa' * 32;
    final id = institutionIdentityToAccountId(
      'LN001-GCB05-944805165-2026',
      mainAddress: mainAddress,
    );

    expect(id.length, 32);
    expect(id, List<int>.filled(32, 0xaa));
  });

  test('注册机构账户身份编码为机构 AccountId', () {
    final address = '11' * 32;
    final identity = registeredDuoqianIdentity(address);
    final id = institutionIdentityToAccountId(identity);

    expect(id.length, 32);
    expect(id, List<int>.filled(32, 0x11));
    expect(findInstitutionByAccountId(id)?.duoqianAddress, address);
  });

  test('个人多签身份编码为个人多签 AccountId', () {
    final address = '22' * 32;
    final id = institutionIdentityToAccountId('personal:$address');

    expect(id.length, 32);
    expect(id, List<int>.filled(32, 0x22));
    expect(findInstitutionByAccountId(id)?.sfidNumber, 'personal:$address');
  });
}
