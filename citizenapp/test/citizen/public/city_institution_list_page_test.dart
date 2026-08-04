import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/citizen/public/city_institution_list_page.dart';
import 'package:citizenapp/citizen/public/data/public_institution_repository.dart';
import 'package:citizenapp/isar/app_isar.dart';

import 'fake_public_institution_store.dart';

class _PendingCityRepository extends PublicInstitutionRepository {
  _PendingCityRepository() : super(store: FakePublicInstitutionStore());

  final Completer<List<PublicInstitutionEntity>> completer =
      Completer<List<PublicInstitutionEntity>>();

  @override
  Future<List<PublicInstitutionEntity>> listInstitutionsByCity(
    String provinceCode,
    String cityCode,
  ) =>
      completer.future;
}

void main() {
  testWidgets('城市目录未返回时直接显示列表页且不使用整页转圈', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        home: CityInstitutionListPage(
          provinceCode: 'ZS',
          cityCode: '001',
          cityName: '中央',
          repository: _PendingCityRepository(),
        ),
      ),
    );
    await tester.pump();

    expect(find.text('中央公权机构'), findsOneWidget);
    expect(find.text('正在读取公权机构'), findsOneWidget);
    expect(
      find.byKey(const ValueKey('city-institution-load-progress')),
      findsOneWidget,
    );
    expect(find.byType(CircularProgressIndicator), findsNothing);
  });
}
