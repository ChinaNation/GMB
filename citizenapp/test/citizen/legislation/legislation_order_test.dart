import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/citizen/institution/institution.dart';
import 'package:citizenapp/citizen/institution/institution_repository.dart';
import 'package:citizenapp/citizen/legislation/legislation_tab.dart';

import '../public/public_nav_harness.dart';

Institution _inst(String code, {String cityCode = ''}) => Institution(
      cidNumber: 'cid-$code-$cityCode',
      cidFullName: '$code$cityCode',
      institutionCode: code,
      cityCode: cityCode,
    );

void main() {
  testWidgets('宪法卡标题使用书名号并在整张卡片水平居中', (tester) async {
    tester.view.physicalSize = const Size(1200, 2400);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final directory = await buildSeededRepo(
      provinceOrder: const [],
      institutions: const [],
    );

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: LegislationTab(
            repository: InstitutionRepository(directory: directory),
          ),
        ),
      ),
    );
    await tester.pump();

    expect(find.text('《公民宪法》'), findsOneWidget);
    expect(find.text('公民宪法'), findsNothing);
    final cardRect = tester.getRect(
      find.byKey(const ValueKey<String>('citizen-constitution-card')),
    );
    final titleRect = tester.getRect(
      find.byKey(const ValueKey<String>('citizen-constitution-title')),
    );
    final title = tester.widget<Text>(
      find.byKey(const ValueKey<String>('citizen-constitution-title')),
    );
    expect(titleRect.center.dx, closeTo(cardRect.center.dx, 0.01));
    expect(title.style?.fontSize, 18);
  });

  test('省级机构按 立法院→参议会→众议会，市立法会按市代码升序', () {
    // 乱序输入（含 众议会在参议会前、市代码乱序），排序后应统一顺序。
    final rows = [
      _inst('CLEG', cityCode: '003'),
      _inst('PRP'), // 省众议会
      _inst('CLEG', cityCode: '001'),
      _inst('PSN'), // 省参议会
      _inst('PLG'), // 省立法院
      _inst('CLEG', cityCode: '002'),
    ];

    final sorted = sortProvinceLegislationRows(rows);

    expect(
      sorted.map((e) => '${e.institutionCode}${e.cityCode}').toList(),
      ['PLG', 'PSN', 'PRP', 'CLEG001', 'CLEG002', 'CLEG003'],
    );
  });

  test('市代码升序按数值而非字符串（防未补零时 2 < 10）', () {
    final sorted = sortProvinceLegislationRows([
      _inst('CLEG', cityCode: '10'),
      _inst('CLEG', cityCode: '2'),
      _inst('CLEG', cityCode: '1'),
    ]);

    expect(sorted.map((e) => e.cityCode).toList(), ['1', '2', '10']);
  });
}
