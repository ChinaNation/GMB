// R2:治理详情按 sfid 反查公权目录库拿 省/市/法定代表人(与公权详情统一)。

import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/citizen/public/data/public_institution_dto.dart';
import 'package:citizenapp/citizen/public/data/sfid_directory_lookup.dart';

import 'public_nav_harness.dart';

void main() {
  const sfid = 'GD001-GCB08-067440774-2026';

  Future<SfidDirectoryLookup> seedLookup(
      List<PublicInstitutionDto> rows) async {
    final repo = await buildSeededRepo(
      provinceOrder: const ['GD'],
      institutions: rows,
      cityNames: const {'GD|001': '广州市'},
    );
    return SfidDirectoryLookup(repository: repo);
  }

  test('命中:反查出省/市/法定代表人(省名链上常量、市名字典 join)', () async {
    final lookup = await seedLookup([
      PublicInstitutionDto.fromJson(<String, dynamic>{
        'sfid_number': sfid,
        'sfid_full_name': '广东省公民储备委员会',
        'province_code': 'GD',
        'city_code': '001',
        'institution_code': 'GCB',
        'account_count': 2,
        'legal_rep_name': '程伟',
      }),
    ]);
    final info = await lookup.lookup(sfid);
    expect(info, isNotNull);
    expect(info!.provinceName, '广东省');
    expect(info.cityName, '广州市');
    expect(info.legalRepName, '程伟');
  });

  test('命中但无法定代表人:legalRepName 为 null', () async {
    final lookup = await seedLookup([
      PublicInstitutionDto.fromJson(<String, dynamic>{
        'sfid_number': sfid,
        'sfid_full_name': '广东省公民储备委员会',
        'province_code': 'GD',
        'city_code': '001',
        'institution_code': 'GCB',
        'account_count': 2,
      }),
    ]);
    final info = await lookup.lookup(sfid);
    expect(info?.legalRepName, isNull);
    expect(info?.provinceName, '广东省');
  });

  test('反查不到(如注册机构账户身份):返回 null', () async {
    final lookup = await seedLookup([
      PublicInstitutionDto.fromJson(<String, dynamic>{
        'sfid_number': sfid,
        'province_code': 'GD',
        'city_code': '001',
        'institution_code': 'GCB',
        'account_count': 2,
      }),
    ]);
    final info = await lookup.lookup('duoqian:abc123');
    expect(info, isNull);
  });
}
