import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/citizen/proposal/proposal_registry.dart';
import 'package:citizenapp/citizen/shared/institution_info.dart';
import 'package:citizenapp/citizen/institution/governance_registry.dart';

void main() {
  test('机构 CID 与具体账户分离，账户操作显式编码 AccountId', () {
    final mainAccount = 'aa' * 32;
    final id = institutionAccountId(mainAccount);

    expect(id.length, 32);
    expect(id, List<int>.filled(32, 0xaa));
  });

  test('注册机构以 CID 为身份，主/费账户只作为其账户集合', () {
    final address = '11' * 32;
    const cidNumber = 'GD001-CGOV0-123456789-2026';
    final id = institutionAccountId(address);

    expect(id.length, 32);
    expect(id, List<int>.filled(32, 0x11));
    final institution = InstitutionInfo(
      cidFullName: '机构',
      cidShortName: '机构',
      cidFullNameEn: 'Institution',
      cidShortNameEn: 'Institution',
      cidNumber: cidNumber,
      orgType: OrgType.institution,
      adminAccountCode: 'CGOV',
      accounts: InstitutionAccounts(
        mainAccount: address,
        feeAccount: '12' * 32,
      ),
    );
    expect(institution.cidNumber, cidNumber);
    expect(institution.mainAccount, address);
    expect(institution.isRegisteredInstitution, isTrue);
  });

  test('个人多签身份编码为个人多签 AccountId', () {
    final address = '22' * 32;
    final id = institutionAccountId(address);

    expect(id.length, 32);
    expect(id, List<int>.filled(32, 0x22));
    expect(personalMultisigFromAccountId(id)?.cidNumber,
        'personal-account:$address');
  });

  test('FRG/NJD 是固定治理机构且使用制度阈值，不误判为注册机构', () {
    InstitutionInfo fixed(String code) => InstitutionInfo(
          cidFullName: code,
          cidShortName: code,
          cidFullNameEn: code,
          cidShortNameEn: code,
          cidNumber: 'ZS001-${code}00-123456789-2026',
          orgType: OrgType.institution,
          adminAccountCode: code,
          accounts: InstitutionAccounts(
            mainAccount: '31' * 32,
            feeAccount: '32' * 32,
          ),
        );

    final frg = fixed('FRG');
    final njd = fixed('NJD');
    expect(frg.internalThreshold, 3);
    expect(njd.internalThreshold, 8);
    expect(frg.isRegisteredInstitution, isFalse);
    expect(njd.isRegisteredInstitution, isFalse);
  });

  group('ProposalCapabilityRegistry', () {
    InstitutionInfo info({
      required String code,
      required String account,
      int orgType = OrgType.institution,
      String? cidNumber,
    }) {
      final personal = code == 'PMUL';
      return InstitutionInfo(
        cidFullName: code,
        cidShortName: code,
        cidFullNameEn: code,
        cidShortNameEn: code,
        cidNumber: cidNumber ?? 'GD001-${code}0-123456789-2026',
        orgType: orgType,
        adminAccountCode: code,
        accounts: personal
            ? null
            : InstitutionAccounts(
                mainAccount: account,
                feeAccount: 'fe' * 32,
              ),
        personalAccountHex: personal ? account : null,
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
      expect(subject.cidNumber, 'LN001-NRC0G-944805165-2026');
      expect(subject.personalAccountHex, isNull);
      expect(result, contains(ProposalKind.transfer));
      expect(result, contains(ProposalKind.feeTransfer));
      expect(result, contains(ProposalKind.safetyFundTransfer));
      expect(result, contains(ProposalKind.resolutionIssuance));
      expect(result, contains(ProposalKind.runtimeUpgrade));
      expect(result, isNot(contains(ProposalKind.adminsChange)));
    });

    test('city registry is public institution, not governance', () {
      final subject = ProposalSubject.fromInstitution(
        institution: info(code: 'CREG', account: '44' * 32),
        institutionCode: 'CREG',
      );
      final result = kinds(subject);
      expect(subject.cidNumber, 'GD001-CREG0-123456789-2026');
      expect(subject.personalAccountHex, isNull);
      expect(result, contains(ProposalKind.transfer));
      expect(result, isNot(contains(ProposalKind.adminsChange)));
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
      expect(subject.cidNumber, 'GD001-SFGQ0-123456789-2026');
      expect(subject.personalAccountHex, isNull);
      expect(result, contains(ProposalKind.transfer));
      expect(result, isNot(contains(ProposalKind.adminsChange)));
      expect(result, isNot(contains(ProposalKind.resolutionIssuance)));
    });

    test('personal multisig exposes admins change capability', () {
      final account = '77' * 32;
      final subject = ProposalSubject.fromInstitution(
        institution: info(
          code: 'PMUL',
          account: account,
          cidNumber: 'personal-account:$account',
        ),
        institutionCode: 'PMUL',
      );
      final result = kinds(subject);
      expect(subject.cidNumber, isNull);
      expect(subject.personalAccountHex, account);
      expect(result, contains(ProposalKind.transfer));
      expect(result, contains(ProposalKind.adminsChange));
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
