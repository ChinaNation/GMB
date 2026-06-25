// 治理 tab 视图(ADR-028 P2)测试 —— 替代旧 governance_list_page_test。
// 机构改由统一目录按机构码加载(注入 seeded fake 仓库),分组/折叠/拖拽 UI 保持。

import 'package:flutter/material.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'package:citizenapp/citizen/governance/governance_tab.dart';
import 'package:citizenapp/citizen/institution/institution.dart';
import 'package:citizenapp/citizen/institution/institution_repository.dart';
import 'package:citizenapp/citizen/public/data/public_institution_dto.dart';
import 'package:citizenapp/citizen/public/data/public_institution_repository.dart';

import '../public/fake_public_institution_store.dart';

/// 构造统一机构(helper 纯函数测试用)。
Institution _inst(String name, String cid, String code) => Institution(
      cidNumber: cid,
      cidFullName: name,
      cidShortName: name,
      institutionCode: code,
    );

PublicInstitutionDto _dto(String name, String cid, String code) =>
    PublicInstitutionDto.fromJson(<String, dynamic>{
      'cid_number': cid,
      'cid_full_name': name,
      'cid_short_name': name,
      'institution_code': code,
      'province_code': '',
      'city_code': '',
      'account_count': 1,
    });

/// seeded fake 仓库:目录按机构码返回 NRC/PRC/PRB 测试机构。
Future<InstitutionRepository> _buildRepo({
  required List<({String name, String cid})> councils,
  required List<({String name, String cid})> banks,
}) async {
  final store = FakePublicInstitutionStore();
  await store.upsertInstitutions(
    [
      _dto('国家储备委员会', 'nrc', 'NRC'),
      for (final c in councils) _dto(c.name, c.cid, 'PRC'),
      for (final b in banks) _dto(b.name, b.cid, 'PRB'),
    ],
    catalogVersion: 'v',
  );
  return InstitutionRepository(
    directory: PublicInstitutionRepository(store: store),
  );
}

Future<void> _pumpPage(
  WidgetTester tester, {
  required List<({String name, String cid})> councils,
  required List<({String name, String cid})> banks,
}) async {
  final repo = await _buildRepo(councils: councils, banks: banks);
  await tester.pumpWidget(
    MaterialApp(
      home: Scaffold(
        body: SizedBox(
          width: 420,
          height: 900,
          child: GovernanceTab(repository: repo),
        ),
      ),
    ),
  );
  await tester.pumpAndSettle();
}

void main() {
  late List<({String name, String cid})> councils;
  late List<({String name, String cid})> banks;

  setUp(() {
    councils = const [
      (name: '甲省储会', cid: 'prc-a'),
      (name: '乙省储会', cid: 'prc-b'),
      (name: '丙省储会', cid: 'prc-c'),
    ];
    banks = const [
      (name: '甲省储行', cid: 'prb-a'),
      (name: '乙省储行', cid: 'prb-b'),
    ];
    SharedPreferences.setMockInitialValues(<String, Object>{});
  });

  test('applyGovernanceInstitutionOrder 使用本机顺序并把新增机构补到末尾', () {
    final source = [
      _inst('甲省储会', 'prc-a', 'PRC'),
      _inst('乙省储会', 'prc-b', 'PRC'),
      _inst('丙省储会', 'prc-c', 'PRC'),
    ];
    final ordered = applyGovernanceInstitutionOrder(
      source,
      const ['prc-b', 'missing', 'prc-a', 'prc-b'],
    );
    expect(ordered.map((i) => i.cidNumber), ['prc-b', 'prc-a', 'prc-c']);
  });

  test('reorderGovernanceInstitutions 按拖拽目标位置重排', () {
    final source = [
      _inst('甲省储会', 'prc-a', 'PRC'),
      _inst('乙省储会', 'prc-b', 'PRC'),
      _inst('丙省储会', 'prc-c', 'PRC'),
    ];
    final reordered = reorderGovernanceInstitutions(source, 0, 2);
    expect(reordered.map((i) => i.cidNumber), ['prc-b', 'prc-c', 'prc-a']);
  });

  testWidgets('省储会和省储行默认折叠，国储会保持展示', (tester) async {
    await _pumpPage(tester, councils: councils, banks: banks);
    expect(find.text('国家储备委员会'), findsOneWidget);
    expect(find.text('甲省储会'), findsNothing);
    expect(find.text('甲省储行'), findsNothing);
    expect(find.byIcon(Icons.chevron_right), findsNWidgets(3));
    expect(find.byIcon(Icons.keyboard_arrow_down), findsNothing);
  });

  testWidgets('国储会卡片横跨整行且高度对齐省级卡片', (tester) async {
    await _pumpPage(tester, councils: councils, banks: banks);
    final cardSize = tester.getSize(
      find.byKey(const ValueKey('governance_national_card_nrc')),
    );
    const expectedProvincialCardHeight = ((420 - 32 - 8) / 2) / 2.9;
    expect(cardSize.height, closeTo(expectedProvincialCardHeight, 0.01));
  });

  testWidgets('点击右侧箭头后只展开对应分组', (tester) async {
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
    expect(find.byIcon(Icons.keyboard_arrow_down), findsOneWidget);
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
    await _pumpPage(
      tester,
      councils: councils.take(2).toList(),
      banks: banks,
    );
    await tester.tap(
      find.byKey(
        const ValueKey('governance_section_toggle_provincialCouncil'),
      ),
    );
    await tester.pumpAndSettle();

    final gesture =
        await tester.startGesture(tester.getCenter(find.text('甲省储会')));
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
