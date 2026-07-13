// 公权机构 DTO 解析单测。

import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/citizen/public/data/public_institution_dto.dart';

void main() {
  group('PublicInstitutionDto.fromJson', () {
    test('解析全字段(行政区只吃 code) + custom_account_names', () {
      final dto = PublicInstitutionDto.fromJson(<String, dynamic>{
        'cid_number': 'AH001-ZF000-123456789-2026',
        'cid_full_name': '安徽省人民政府',
        'cid_short_name': '皖府',
        'status': 'ACTIVE',
        'province_code': 'AH',
        'city_code': '001',
        'town_code': '',
        'institution_code': 'ZF',
        'account_count': 3,
        'legal_representative_name': '李大民',
        'custom_account_names': ['业务专户A', '业务专户B'],
      });
      expect(dto.cidNumber, 'AH001-ZF000-123456789-2026');
      expect(dto.cidFullName, '安徽省人民政府');
      expect(dto.provinceCode, 'AH');
      expect(dto.cityCode, '001');
      expect(dto.townCode, '');
      expect(dto.accountCount, 3);
      expect(dto.legalRepresentativeName, '李大民');
      expect(dto.customAccountNames, ['业务专户A', '业务专户B']);
      // 行政区 code + 法定代表人随实体落库。
      final entity = dto.toEntity(catalogVersion: 'v', updatedAtMillis: 0);
      expect(entity.provinceCode, 'AH');
      expect(entity.cityCode, '001');
      expect(entity.legalRepresentativeName, '李大民');
    });

    test('缺省行政区 code → 空串;缺省 custom/法定代表人 → 空/null(无名字 fallback)', () {
      final dto = PublicInstitutionDto.fromJson(<String, dynamic>{
        'cid_number': 'X',
        'province_code': 'ZS',
        'city_code': '001',
        'institution_code': 'ZF',
        'account_count': 2,
      });
      expect(dto.provinceCode, 'ZS');
      expect(dto.cityCode, '001');
      expect(dto.townCode, ''); // 缺省镇 code → 空串
      expect(dto.customAccountNames, isEmpty);
      expect(dto.status, 'ACTIVE');
      expect(dto.legalRepresentativeName, isNull);
    });

    test('toEntity 填 catalogVersion + 名称回退', () {
      final dto = PublicInstitutionDto.fromJson(<String, dynamic>{
        'cid_number': 'Y',
        'province_code': 'ZS',
        'city_code': '001',
        'institution_code': 'LF',
        'account_count': 2,
      });
      final e = dto.toEntity(catalogVersion: 'v9', updatedAtMillis: 123);
      expect(e.cidNumber, 'Y');
      expect(e.cidFullName, 'Y'); // 无名回退 cidNumber
      expect(e.catalogVersion, 'v9');
      expect(e.updatedAtMillis, 123);
    });
  });
}
