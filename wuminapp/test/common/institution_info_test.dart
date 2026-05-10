import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/common/institution_info.dart';
import 'package:wuminapp_mobile/organization-manage/institution_registry.dart';

void main() {
  test('内置机构身份编码为 0x01 Builtin SubjectId', () {
    final id = institutionIdentityToPalletId('GFR-LN001-CB0X-944805165-2026');

    expect(id.length, 48);
    expect(id[0], 0x01);
    final tail = id.sublist(1);
    final end = tail.indexOf(0);
    expect(String.fromCharCodes(tail.sublist(0, end)),
        'GFR-LN001-CB0X-944805165-2026');
  });

  test('注册机构账户身份编码为 0x05 InstitutionAccount SubjectId', () {
    final address = '11' * 32;
    final identity = registeredDuoqianIdentity(address);
    final id = institutionIdentityToPalletId(identity);

    expect(id.length, 48);
    expect(id[0], 0x05);
    expect(id.sublist(1, 33), List<int>.filled(32, 0x11));
    expect(id.sublist(33), List<int>.filled(15, 0));
    expect(findInstitutionByPalletId(id)?.duoqianAddress, address);
  });

  test('个人多签身份编码为 0x03 PersonalDuoqian SubjectId', () {
    final address = '22' * 32;
    final id = institutionIdentityToPalletId('personal:$address');

    expect(id.length, 48);
    expect(id[0], 0x03);
    expect(id.sublist(1, 33), List<int>.filled(32, 0x22));
    expect(id.sublist(33), List<int>.filled(15, 0));
    expect(findInstitutionByPalletId(id)?.sfidNumber, 'personal:$address');
  });
}
