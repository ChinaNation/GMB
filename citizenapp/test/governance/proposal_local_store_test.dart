import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/citizen/institution/governance_registry.dart';
import 'package:citizenapp/citizen/shared/proposal/proposal_local_store.dart';
import 'package:citizenapp/citizen/shared/proposal/proposal_models.dart';
import 'package:citizenapp/isar/wallet_isar.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  final nationalCouncil = kNationalCouncil.first;

  setUpAll(() async {
    await WalletIsar.instance.ensureTestCoreInitialized();
  });

  setUp(() async {
    await WalletIsar.instance.resetForTest();
  });

  test('提案摘要按 ID 持久化读取', () async {
    final summary = LocalProposalSummary.fromProposal(
      ProposalWithDetail(
        meta: ProposalMeta(
          proposalId: 12,
          kind: 1,
          stage: 1,
          status: 0,
          internalCode: 'NRC',
          institutionBytes: Uint8List(32),
          subjectCidNumbers: const ['LN001-NRC0G-000000001-2026'],
          displayMeta: const ProposalDisplayMeta(year: 2026, seqInYear: 3),
        ),
      ),
      institution: nationalCouncil,
    );

    await ProposalLocalStore.instance.upsertSummaries([summary]);

    final page = await ProposalLocalStore.instance.readSummariesForIds([12]);

    expect(page, hasLength(1));
    expect(page.single.proposalId, 12);
    expect(page.single.displayId, '2026000003');
    expect(page.single.cidFullName, nationalCouncil.cidFullName);
    expect(page.single.subjectCidNumbers, ['LN001-NRC0G-000000001-2026']);
  });

  test('提案摘要按机构索引持久化读取', () async {
    final summary = LocalProposalSummary.fromProposal(
      ProposalWithDetail(
        meta: ProposalMeta(
          proposalId: 33,
          kind: 0,
          stage: 0,
          status: 1,
          internalCode: 'NRC',
          institutionBytes: Uint8List(32),
          displayMeta: const ProposalDisplayMeta(year: 2026, seqInYear: 9),
        ),
      ),
      institution: nationalCouncil,
    );

    await ProposalLocalStore.instance.upsertSummaries([summary]);
    await ProposalLocalStore.instance.putInstitutionIndex(
      nationalCouncil.cidNumber,
      [33],
    );

    final list = await ProposalLocalStore.instance.readInstitutionSummaries(
      nationalCouncil.cidNumber,
    );

    expect(list, hasLength(1));
    expect(list.single.proposalId, 33);
    expect(list.single.status, 1);
    expect(
      await ProposalLocalStore.instance.isInstitutionIndexFresh(
        nationalCouncil.cidNumber,
      ),
      isTrue,
    );
  });
}
