// R2:治理详情按 cid 反查公权目录库拿 省/市/法定代表人(与公权详情统一)。

import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/citizen/public/data/public_institution_dto.dart';
import 'package:citizenapp/citizen/public/data/cid_directory_lookup.dart';

import 'public_nav_harness.dart';

void main() {
  const cid = 'GD001-GCB08-067440774-2026';

  Future<CidDirectoryLookup> seedLookup(List<PublicInstitutionDto> rows) async {
    final repo = await buildSeededRepo(
      provinceOrder: const ['GD'],
      institutions: rows,
      cityNames: const {'GD|001': '广州市'},
    );
    return CidDirectoryLookup(repository: repo);
  }

  test('命中:反查出省/市/法定代表人(省名链上常量、市名字典 join)', () async {
    final lookup = await seedLookup([
      PublicInstitutionDto.fromJson(<String, dynamic>{
        'cid_number': cid,
        'cid_full_name': '广东省公民储备委员会',
        'province_code': 'GD',
        'city_code': '001',
        'institution_code': 'GCB',
        'account_count': 2,
        'legal_representative': {
          'family_name': '程',
          'given_name': '伟',
          'cid_number': 'CID-1',
          'account': '11',
        },
      }),
    ]);
    final info = await lookup.lookup(cid);
    expect(info, isNotNull);
    expect(info!.provinceName, '广东省');
    expect(info.cityName, '广州市');
    expect(info.familyName, '程');
    expect(info.givenName, '伟');
  });

  test('命中但无法定代表人:familyName/givenName 为 null', () async {
    final lookup = await seedLookup([
      PublicInstitutionDto.fromJson(<String, dynamic>{
        'cid_number': cid,
        'cid_full_name': '广东省公民储备委员会',
        'province_code': 'GD',
        'city_code': '001',
        'institution_code': 'GCB',
        'account_count': 2,
      }),
    ]);
    final info = await lookup.lookup(cid);
    expect(info?.familyName, isNull);
    expect(info?.givenName, isNull);
    expect(info?.provinceName, '广东省');
  });

  test('反查不到未知机构 CID 时返回 null', () async {
    final lookup = await seedLookup([
      PublicInstitutionDto.fromJson(<String, dynamic>{
        'cid_number': cid,
        'province_code': 'GD',
        'city_code': '001',
        'institution_code': 'GCB',
        'account_count': 2,
      }),
    ]);
    final info = await lookup.lookup('GD001-CGOV0-000000000-2026');
    expect(info, isNull);
  });
}
