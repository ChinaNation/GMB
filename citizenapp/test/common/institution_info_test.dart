import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/citizen/proposal/proposal_registry.dart';
import 'package:citizenapp/citizen/shared/institution_info.dart';
import 'package:citizenapp/transaction/organization-manage/institution_registry.dart';

void main() {
  test('内置机构身份编码为 mainAccount AccountId', () {
    final mainAccount = 'aa' * 32;
    final id = institutionIdentityToAccountId(
      'LN001-NRC0G-944805165-2026',
      mainAccount: mainAccount,
    );

    expect(id.length, 32);
    expect(id, List<int>.filled(32, 0xaa));
  });

  test('注册机构账户身份编码为机构 AccountId', () {
    final address = '11' * 32;
    final identity = registeredAccountIdentity(address);
    final id = institutionIdentityToAccountId(identity);

    expect(id.length, 32);
    expect(id, List<int>.filled(32, 0x11));
    expect(findInstitutionByAccountId(id)?.account, address);
  });

  test('个人多签身份编码为个人多签 AccountId', () {
    final address = '22' * 32;
    final id = institutionIdentityToAccountId('personal-account:$address');

    expect(id.length, 32);
    expect(id, List<int>.filled(32, 0x22));
    expect(
        findInstitutionByAccountId(id)?.cidNumber, 'personal-account:$address');
  });

  group('ProposalCapabilityRegistry', () {
    InstitutionInfo info({
      required String code,
      required String account,
      int orgType = OrgType.account,
      String? cidNumber,
    }) {
      return InstitutionInfo(
        cidFullName: code,
        cidShortName: code,
        cidFullNameEn: code,
        cidShortNameEn: code,
        cidNumber: cidNumber ?? registeredAccountIdentity(account),
        orgType: orgType,
        adminAccountCode: code,
        account: account,
      );
    }

    Set<ProposalKind> kinds(ProposalSubject subject) {
      return ProposalCapabilityRegistry.capabilitiesForSubject(subject)
          .map((capability) => capability.kind)
          .toSet();
    }

    test('NRC exposes governance-only proposal capabilities', () {
      final subject = ProposalSubject.fromInstitution(
        institution: info(
          code: 'NRC',
          account: '33' * 32,
          orgType: OrgType.nrc,
          cidNumber: 'LN001-NRC0G-944805165-2026',
        ),
        institutionCode: 'NRC',
      );
      final result = kinds(subject);
      expect(result, contains(ProposalKind.transfer));
      expect(result, contains(ProposalKind.feeTransfer));
      expect(result, contains(ProposalKind.safetyFundTransfer));
      expect(result, contains(ProposalKind.resolutionIssuance));
      expect(result, contains(ProposalKind.runtimeUpgrade));
      expect(result, contains(ProposalKind.adminsChange));
    });

    test('city registry is public institution, not governance', () {
      final subject = ProposalSubject.fromInstitution(
        institution: info(code: 'CREG', account: '44' * 32),
        institutionCode: 'CREG',
      );
      final result = kinds(subject);
      expect(result, contains(ProposalKind.transfer));
      expect(result, contains(ProposalKind.adminsChange));
      expect(result, isNot(contains(ProposalKind.feeTransfer)));
      expect(result, isNot(contains(ProposalKind.runtimeUpgrade)));
    });

    test('private institution gets only generic active-account capabilities',
        () {
      final subject = ProposalSubject.fromInstitution(
        institution: info(code: 'SFGQ', account: '55' * 32),
        institutionCode: 'SFGQ',
      );
      final result = kinds(subject);
      expect(result, contains(ProposalKind.transfer));
      expect(result, contains(ProposalKind.adminsChange));
      expect(result, isNot(contains(ProposalKind.resolutionIssuance)));
    });

    test('unincorporated code does not auto-enable admins change', () {
      final subject = ProposalSubject.fromInstitution(
        institution: info(code: 'UNIN', account: '66' * 32),
        institutionCode: 'UNIN',
      );
      final result = kinds(subject);
      expect(result, contains(ProposalKind.transfer));
      expect(result, isNot(contains(ProposalKind.adminsChange)));
    });
  });
}
