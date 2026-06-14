// 公权机构 DTO 解析单测。

import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/citizen/public/data/public_institution_dto.dart';

void main() {
  group('PublicInstitutionDto.fromJson', () {
    test('解析全字段 + custom_account_names', () {
      final dto = PublicInstitutionDto.fromJson(<String, dynamic>{
        'sfid_number': 'AH001-ZF000-123456789-2026',
        'institution_name': '安徽省人民政府',
        'sfid_name': '安徽省国民政府',
        'short_name': '皖府',
        'status': 'ACTIVE',
        'province': '安徽',
        'city': '合肥',
        'town': '',
        'institution_code': 'ZF',
        'account_count': 3,
        'custom_account_names': ['业务专户A', '业务专户B'],
      });
      expect(dto.sfidNumber, 'AH001-ZF000-123456789-2026');
      expect(dto.institutionName, '安徽省人民政府');
      expect(dto.accountCount, 3);
      expect(dto.customAccountNames, ['业务专户A', '业务专户B']);
    });

    test('缺省 custom_account_names → 空列表', () {
      final dto = PublicInstitutionDto.fromJson(<String, dynamic>{
        'sfid_number': 'X',
        'province': '中枢',
        'city': '中央',
        'institution_code': 'ZF',
        'account_count': 2,
      });
      expect(dto.customAccountNames, isEmpty);
      expect(dto.status, 'ACTIVE');
    });

    test('toEntity 填 catalogVersion + 名称回退', () {
      final dto = PublicInstitutionDto.fromJson(<String, dynamic>{
        'sfid_number': 'Y',
        'province': '中枢',
        'city': '中央',
        'institution_code': 'LF',
        'account_count': 2,
      });
      final e = dto.toEntity(catalogVersion: 'v9', updatedAtMillis: 123);
      expect(e.sfidNumber, 'Y');
      expect(e.institutionName, 'Y'); // 无名回退 sfidNumber
      expect(e.catalogVersion, 'v9');
      expect(e.updatedAtMillis, 123);
    });
  });

  group('PublicInstitutionPage.fromData', () {
    test('解析分页元数据', () {
      final page = PublicInstitutionPage.fromData(<String, dynamic>{
        'items': [
          {
            'sfid_number': 'A',
            'province': '中枢',
            'city': '中央',
            'institution_code': 'ZF',
            'account_count': 2,
          }
        ],
        'has_more': true,
        'next_cursor': '1',
        'manifest_version': 'mv-1',
      });
      expect(page.items, hasLength(1));
      expect(page.hasMore, isTrue);
      expect(page.manifestVersion, 'mv-1');
    });
  });
}
