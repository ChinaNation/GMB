// 统一机构详情页(ADR-028)widget 测试 —— 公权路径(信息卡/账户/提案占位/管理员/
// 提案列表/订阅)+ 统一账户行派生。替代旧 public_institution_detail_test。

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:citizenapp/citizen/institution/institution.dart';
import 'package:citizenapp/citizen/institution/institution_accounts.dart';
import 'package:citizenapp/citizen/institution/institution_chain_state.dart';
import 'package:citizenapp/citizen/institution/institution_detail_page.dart';
import 'package:citizenapp/citizen/institution/institution_repository.dart';
import 'package:citizenapp/citizen/public/data/public_institution_dto.dart';
import 'package:citizenapp/citizen/shared/admin_profile.dart';
import 'package:citizenapp/citizen/shared/account_derivation.dart';
import 'package:citizenapp/isar/wallet_isar.dart';

import '../public/public_nav_harness.dart';

const _cid = 'LN001-CREG0-944805165-2026';

class _FakeChainState implements InstitutionChainState {
  _FakeChainState({this.adminList = const [], this.proposalList = const []});
  final List<String> adminList;
  final List<InstitutionProposalSummary> proposalList;

  @override
  Future<Map<String, double>> balances(List<String> pubkeyHexes) async =>
      {for (final h in pubkeyHexes) h: 12.5};

  @override
  Future<List<String>> admins(Institution institution) async => adminList;

  @override
  Future<List<AdminProfile>> adminProfiles(Institution institution) async =>
      adminList.map((a) => AdminProfile(account: a)).toList();

  @override
  Future<List<InstitutionProposalSummary>> proposals(
    Institution institution,
  ) async =>
      proposalList;
}

PublicInstitutionEntity _entity() => PublicInstitutionDto.fromJson(
      <String, dynamic>{
        'cid_number': _cid,
        'cid_full_name': '辽宁省身份注册局',
        'province_code': 'LN',
        'city_code': '001',
        'institution_code': 'CREG',
        'account_count': 4,
        'custom_account_names': ['业务专户'],
      },
    ).toEntity(catalogVersion: 'v', updatedAtMillis: 0);

Widget _wrap(Widget child) => MaterialApp(home: child);

void main() {
  group('institutionAccountRows(公权派生)', () {
    test('主/费/自定义三行,地址与卡0 派生吻合', () {
      final rows =
          institutionAccountRows(Institution.fromPublicEntity(_entity()));
      expect(rows.map((r) => r.label), ['主账户', '费用账户', '业务专户']);
      expect(rows.first.accountHex,
          hexFromAccountId(deriveInstitutionMainAccountId(_cid)));
      expect(
        rows.last.accountHex,
        hexFromAccountId(deriveInstitutionCustomAccountId(_cid, '业务专户')),
      );
    });
  });

  testWidgets('详情页:全称/ID/主账户/余额/法代/所属地 + 账户/提案占位/管理员/提案列表', (tester) async {
    tester.view.physicalSize = const Size(1200, 3200);
    tester.view.devicePixelRatio = 1.0;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final repo = await buildSeededRepo(
      provinceOrder: const ['GD'],
      institutions: [
        PublicInstitutionDto.fromJson(<String, dynamic>{
          'cid_number': _cid,
          'cid_full_name': '辽宁省身份注册局',
          'province_code': 'GD',
          'city_code': '001',
          'institution_code': 'CREG',
          'account_count': 4,
          'legal_rep_name': '王法人',
          'custom_account_names': ['业务专户'],
        }),
      ],
      cityNames: const {'GD|001': '中央'},
    );
    final chain = _FakeChainState(
      adminList: const ['0xadminpubkey001'],
      proposalList: const [
        InstitutionProposalSummary(proposalId: 7, idLabel: '提案 #7', status: 1),
      ],
    );
    await tester.pumpWidget(_wrap(InstitutionDetailPage(
      cidNumber: _cid,
      repository: InstitutionRepository(directory: repo),
      chainState: chain,
      walletPubkeyProvider: () async => 'aa',
    )));
    await tester.pumpAndSettle();

    expect(find.text('辽宁省身份注册局'), findsWidgets); // AppBar 简称回退全称 + 全称行
    expect(find.text(_cid), findsOneWidget);
    expect(find.text('全称'), findsOneWidget);
    expect(find.text('主账户'), findsOneWidget);
    expect(find.text('主账户余额'), findsOneWidget);
    expect(find.text('12.50 元'), findsOneWidget);
    expect(find.text('法定代表人'), findsOneWidget);
    expect(find.text('王法人'), findsOneWidget);
    expect(find.text('所属地'), findsOneWidget);
    expect(find.text('广东省 · 中央'), findsOneWidget);
    // 机构账户入口:主+费+1自定义=3。
    expect(find.text('机构账户'), findsOneWidget);
    expect(find.text('共 3 个账户'), findsOneWidget);
    // 提案入口(公权占位)。
    expect(find.text('发起提案'), findsOneWidget);
    // 管理员入口。
    expect(find.text('管理员'), findsOneWidget);
    expect(find.text('共 1 位管理员'), findsOneWidget);
    // 提案列表。
    expect(find.text('提案列表'), findsOneWidget);
    expect(find.text('提案 #7'), findsOneWidget);
  });

  testWidgets('管理员入口点击进入可激活管理员列表页', (tester) async {
    final repo = await buildSeededRepo(
      provinceOrder: const ['LN'],
      institutions: [
        PublicInstitutionDto.fromJson(<String, dynamic>{
          'cid_number': _cid,
          'cid_full_name': '辽宁省身份注册局',
          'province_code': 'LN',
          'city_code': '001',
          'institution_code': 'CREG',
          'account_count': 2,
        }),
      ],
      cityNames: const {'LN|001': '中央'},
    );
    await tester.pumpWidget(_wrap(InstitutionDetailPage(
      cidNumber: _cid,
      repository: InstitutionRepository(directory: repo),
      chainState: _FakeChainState(adminList: const ['0xadminpubkey001']),
      walletPubkeyProvider: () async => 'aa',
    )));
    await tester.pumpAndSettle();

    await tester.ensureVisible(find.text('管理员'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('管理员'));
    await tester.pumpAndSettle();
    expect(find.text('管理员列表'), findsOneWidget);
    expect(find.textContaining('共 1 位管理员'), findsOneWidget);
  });

  testWidgets('订阅按钮切换写入 store', (tester) async {
    final repo = await buildSeededRepo(
      provinceOrder: const ['LN'],
      institutions: [
        PublicInstitutionDto.fromJson(<String, dynamic>{
          'cid_number': _cid,
          'cid_full_name': '辽宁省身份注册局',
          'province_code': 'LN',
          'city_code': '001',
          'institution_code': 'CREG',
          'account_count': 2,
        }),
      ],
      cityNames: const {'LN|001': '中央'},
    );
    await tester.pumpWidget(_wrap(InstitutionDetailPage(
      cidNumber: _cid,
      repository: InstitutionRepository(directory: repo),
      chainState: _FakeChainState(),
      walletPubkeyProvider: () async => 'aa',
    )));
    await tester.pumpAndSettle();

    expect(await repo.isSubscribed('aa', _cid), isFalse);
    await tester.tap(find.byIcon(Icons.bookmark_border));
    await tester.pumpAndSettle();
    expect(await repo.isSubscribed('aa', _cid), isTrue);

    await tester.tap(find.byIcon(Icons.bookmark));
    await tester.pumpAndSettle();
    expect(await repo.isSubscribed('aa', _cid), isFalse);
  });
}
