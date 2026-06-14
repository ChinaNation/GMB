// 卡C 详情页:账户派生 + 余额/管理员/提案展示 + 订阅切换。

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

  testWidgets('详情页展示名称/ID/账户余额/管理员/提案', (tester) async {
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
    expect(find.text('12.50 元'), findsWidgets); // 余额
    expect(find.text('更多账户(3)'), findsOneWidget); // 主+费+1自定义
    expect(find.text('提案 #7'), findsOneWidget);
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
