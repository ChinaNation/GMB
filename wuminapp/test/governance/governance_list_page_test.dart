import 'package:flutter/material.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/governance/governance_list_page.dart';
import 'package:wuminapp_mobile/governance/shared/institution_info.dart';

InstitutionInfo _institution(
  String name,
  String sfidNumber,
  int orgType,
  int hexSeed,
) {
  return InstitutionInfo(
    name: name,
    sfidNumber: sfidNumber,
    orgType: orgType,
    duoqianAddress: hexSeed.toRadixString(16).padLeft(64, '0'),
  );
}

Future<void> _pumpPage(
  WidgetTester tester, {
  required List<InstitutionInfo> councils,
  required List<InstitutionInfo> banks,
}) async {
  await tester.pumpWidget(
    MaterialApp(
      home: Scaffold(
        body: SizedBox(
          width: 420,
          height: 900,
          child: GovernanceListPage(
            nationalCouncil: [
              _institution('国家储备委员会', 'nrc', OrgType.nrc, 1),
            ],
            provincialCouncils: councils,
            provincialBanks: banks,
          ),
        ),
      ),
    ),
  );
  await tester.pumpAndSettle();
}

void main() {
  late List<InstitutionInfo> councils;
  late List<InstitutionInfo> banks;

  setUp(() {
    councils = [
      _institution('甲省储会', 'prc-a', OrgType.prc, 2),
      _institution('乙省储会', 'prc-b', OrgType.prc, 3),
      _institution('丙省储会', 'prc-c', OrgType.prc, 4),
    ];
    banks = [
      _institution('甲省储行', 'prb-a', OrgType.prb, 5),
      _institution('乙省储行', 'prb-b', OrgType.prb, 6),
    ];
    SharedPreferences.setMockInitialValues(<String, Object>{});
  });

  test('applyGovernanceInstitutionOrder 使用本机顺序并把新增机构补到末尾', () {
    final ordered = applyGovernanceInstitutionOrder(
      councils,
      const ['prc-b', 'missing', 'prc-a', 'prc-b'],
    );

    expect(
      ordered.map((institution) => institution.sfidNumber),
      ['prc-b', 'prc-a', 'prc-c'],
    );
  });

  test('reorderGovernanceInstitutions 按拖拽目标位置重排', () {
    final reordered = reorderGovernanceInstitutions(councils, 0, 2);

    expect(
      reordered.map((institution) => institution.sfidNumber),
      ['prc-b', 'prc-c', 'prc-a'],
    );
  });

  testWidgets('省储会和省储行默认折叠，国储会保持展示', (tester) async {
    await _pumpPage(tester, councils: councils, banks: banks);

    expect(find.text('国家储备委员会'), findsOneWidget);
    expect(find.text('甲省储会'), findsNothing);
    expect(find.text('甲省储行'), findsNothing);
    expect(find.byIcon(Icons.arrow_right), findsNWidgets(2));
    expect(find.byIcon(Icons.arrow_drop_down), findsNothing);
  });

  testWidgets('点击三角后只展开对应分组', (tester) async {
    await _pumpPage(tester, councils: councils, banks: banks);

    await tester.tap(
      find.byKey(
        const ValueKey('governance_section_toggle_provincialCouncil'),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('甲省储会'), findsOneWidget);
    expect(find.text('乙省储会'), findsOneWidget);
    expect(find.text('甲省储行'), findsNothing);
    expect(find.byIcon(Icons.arrow_drop_down), findsOneWidget);
  });

  testWidgets('展开后按本机保存顺序展示，不做管理员优先自动排序', (tester) async {
    SharedPreferences.setMockInitialValues({
      governanceProvincialCouncilOrderPrefsKey: ['prc-b', 'prc-a'],
    });

    await _pumpPage(tester, councils: councils, banks: banks);
    await tester.tap(
      find.byKey(
        const ValueKey('governance_section_toggle_provincialCouncil'),
      ),
    );
    await tester.pumpAndSettle();

    final first = tester.getTopLeft(find.text('乙省储会'));
    final second = tester.getTopLeft(find.text('甲省储会'));
    expect(first.dx, lessThan(second.dx));
  });

  testWidgets('长按拖拽省储会后保存本机排序', (tester) async {
    await _pumpPage(tester, councils: councils.take(2).toList(), banks: banks);
    await tester.tap(
      find.byKey(
        const ValueKey('governance_section_toggle_provincialCouncil'),
      ),
    );
    await tester.pumpAndSettle();

    final gesture = await tester.startGesture(
      tester.getCenter(find.text('甲省储会')),
    );
    await tester.pump(kLongPressTimeout + const Duration(milliseconds: 120));
    await gesture.moveTo(tester.getCenter(find.text('乙省储会')));
    await tester.pump();
    await gesture.up();
    await tester.pumpAndSettle();

    final prefs = await SharedPreferences.getInstance();
    expect(
      prefs.getStringList(governanceProvincialCouncilOrderPrefsKey),
      ['prc-b', 'prc-a'],
    );
  });
}
