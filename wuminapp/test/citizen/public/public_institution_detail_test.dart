// 卡C 详情页:账户派生 + 五段版式(机构信息/机构账户入口/提案发起/管理员入口/提案列表)
// + 订阅切换。余额只在「全部账户页」展示,不在详情页。

import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/citizen/public/data/public_institution_accounts.dart';
import 'package:wuminapp_mobile/citizen/public/data/public_institution_chain_data.dart';
import 'package:wuminapp_mobile/citizen/public/data/public_institution_dto.dart';
import 'package:wuminapp_mobile/citizen/public/public_institution_detail_page.dart';
import 'package:wuminapp_mobile/governance/shared/account_derivation.dart';
import 'package:wuminapp_mobile/isar/wallet_isar.dart';

import 'public_nav_harness.dart';

const _nrcSfid = 'LN001-GCB05-944805165-2026';
const _nrcMainHex =
    '39936ebd8564c61f315662ff859d8fb5470ac3f1b4bfbf86746aff391d14db3d';

class _FakeChainData implements PublicInstitutionChainData {
  _FakeChainData({this.adminList = const [], this.proposalList = const []});
  final List<String> adminList;
  final List<PublicProposalSummary> proposalList;

  @override
  Future<Map<String, double>> balances(List<String> pubkeyHexes) async =>
      {for (final h in pubkeyHexes) h: 12.5};

  @override
  Future<List<String>> admins({
    required String mainAccountHex,
    required String displayName,
  }) async =>
      adminList;

  @override
  Future<List<PublicProposalSummary>> proposals(
    Uint8List mainAccountId,
  ) async =>
      proposalList;
}

PublicInstitutionEntity _entity() => PublicInstitutionDto.fromJson(
      <String, dynamic>{
        'sfid_number': _nrcSfid,
        'institution_name': '国家公民储备委员会',
        'province': '岭南',
        'city': '中央',
        'institution_code': 'ZF',
        'account_count': 4,
        'custom_account_names': ['业务专户'],
      },
    ).toEntity(catalogVersion: 'v', updatedAtMillis: 0);

Widget _wrap(Widget child) => MaterialApp(home: child);

void main() {
  group('deriveAccountRows', () {
    test('主/费/自定义三行,地址与链上派生吻合', () {
      final rows = deriveAccountRows(_entity());
      expect(rows.map((r) => r.label), ['主账户', '费用账户', '业务专户']);
      expect(rows.first.addressHex, _nrcMainHex);
      // 自定义地址 = 卡0 派生
      expect(
        rows.last.addressHex,
        hexFromAccountId(deriveInstitutionCustomAccountId(_nrcSfid, '业务专户')),
      );
    });
  });

  testWidgets('详情页五段:名称/ID/法定代表人/机构账户/提案发起/管理员入口/提案列表', (tester) async {
    final repo = await buildSeededRepo(
      provinceOrder: const ['岭南'],
      institutions: [
        // 复用 harness seedDto 不便带 custom,这里直接用 entity 的 dto。
        PublicInstitutionDto.fromJson(<String, dynamic>{
          'sfid_number': _nrcSfid,
          'institution_name': '国家公民储备委员会',
          'province': '岭南',
          'city': '中央',
          'institution_code': 'ZF',
          'account_count': 4,
          'legal_rep_name': '王法人',
          'custom_account_names': ['业务专户'],
        }),
      ],
    );
    final chain = _FakeChainData(
      adminList: const ['0xadminpubkey001'],
      proposalList: const [PublicProposalSummary(idLabel: '提案 #7', status: 1)],
    );
    await tester.pumpWidget(_wrap(PublicInstitutionDetailPage(
      sfidNumber: _nrcSfid,
      repository: repo,
      chainData: chain,
      walletPubkeyProvider: () async => 'aa',
    )));
    await tester.pumpAndSettle();

    expect(find.text('国家公民储备委员会'), findsWidgets);
    expect(find.text(_nrcSfid), findsOneWidget);
    // ① 机构信息:法定代表人 + 所属地(行间分隔线)。
    expect(find.text('法定代表人'), findsOneWidget);
    expect(find.text('王法人'), findsOneWidget);
    expect(find.text('所属地'), findsOneWidget);
    // ② 机构账户入口:主+费+1自定义。
    expect(find.text('机构账户(3)'), findsOneWidget);
    // ③ 提案发起入口(占位)。
    expect(find.text('发起提案'), findsOneWidget);
    // ④ 管理员入口:只显条数,地址在列表页。
    expect(find.text('管理员(1)'), findsOneWidget);
    // ⑤ 提案列表。
    expect(find.text('提案列表'), findsOneWidget);
    expect(find.text('提案 #7'), findsOneWidget);
  });

  testWidgets('管理员入口点击进入管理员列表页', (tester) async {
    final repo = await buildSeededRepo(
      provinceOrder: const ['岭南'],
      institutions: [
        PublicInstitutionDto.fromJson(<String, dynamic>{
          'sfid_number': _nrcSfid,
          'institution_name': '国家公民储备委员会',
          'province': '岭南',
          'city': '中央',
          'institution_code': 'ZF',
          'account_count': 2,
        }),
      ],
    );
    await tester.pumpWidget(_wrap(PublicInstitutionDetailPage(
      sfidNumber: _nrcSfid,
      repository: repo,
      chainData: _FakeChainData(adminList: const ['0xadminpubkey001']),
      walletPubkeyProvider: () async => 'aa',
    )));
    await tester.pumpAndSettle();

    await tester.tap(find.text('管理员(1)'));
    await tester.pumpAndSettle();
    // 管理员列表页:非法 hex 兜底原样展示,地址可见。
    expect(find.text('管理员列表'), findsOneWidget);
    expect(find.text('0xadminpubkey001'), findsOneWidget);
  });

  testWidgets('订阅按钮切换写入 store', (tester) async {
    final repo = await buildSeededRepo(
      provinceOrder: const ['岭南'],
      institutions: [
        PublicInstitutionDto.fromJson(<String, dynamic>{
          'sfid_number': _nrcSfid,
          'institution_name': '国家公民储备委员会',
          'province': '岭南',
          'city': '中央',
          'institution_code': 'ZF',
          'account_count': 2,
        }),
      ],
    );
    await tester.pumpWidget(_wrap(PublicInstitutionDetailPage(
      sfidNumber: _nrcSfid,
      repository: repo,
      chainData: _FakeChainData(),
      walletPubkeyProvider: () async => 'aa',
    )));
    await tester.pumpAndSettle();

    expect(await repo.isSubscribed('aa', _nrcSfid), isFalse);
    await tester.tap(find.byIcon(Icons.bookmark_border));
    await tester.pumpAndSettle();
    expect(await repo.isSubscribed('aa', _nrcSfid), isTrue);

    await tester.tap(find.byIcon(Icons.bookmark));
    await tester.pumpAndSettle();
    expect(await repo.isSubscribed('aa', _nrcSfid), isFalse);
  });
}
