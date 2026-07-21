import 'dart:convert';
import 'dart:typed_data';

import 'package:citizenapp/citizen/proposal/admins-change/codec/account_id_codec.dart';
import 'package:citizenapp/citizen/proposal/admins-change/models/admin_account.dart';
import 'package:citizenapp/citizen/proposal/admins-change/services/admin_account_service.dart';
import 'package:citizenapp/citizen/proposal/admins-change/services/admin_activation_service.dart';
import 'package:citizenapp/citizen/proposal/admins-change/services/institution_admin_service.dart';
import 'package:citizenapp/citizen/shared/institution_info.dart';
import 'package:citizenapp/citizen/institution/institution_role_models.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:shared_preferences/shared_preferences.dart';

class FakeChainRpc extends ChainRpc {
  final Map<String, Uint8List?> responses = {};
  final List<String> requestedKeys = [];

  @override
  Future<Uint8List?> fetchStorage(String storageKeyHex) async {
    requestedKeys.add(storageKeyHex);
    return responses[storageKeyHex];
  }
}

class FakeAdminService extends InstitutionAdminService {
  FakeAdminService({required this.admins});

  final List<AdminPerson> admins;

  @override
  Future<List<AdminPerson>> fetchAdmins(AdminAccountIdentity identity) async =>
      admins;
}

void main() {
  String hexOf(Iterable<int> bytes) =>
      bytes.map((byte) => byte.toRadixString(16).padLeft(2, '0')).join();

  List<int> codeBytes(String code) => [
        ...code.codeUnits,
        ...List.filled(4 - code.length, 0),
      ];

  List<int> u32Le(int value) => [
        value & 0xff,
        (value >> 8) & 0xff,
        (value >> 16) & 0xff,
        (value >> 24) & 0xff,
      ];

  Uint8List institutionAdminBytes({
    required String institutionCode,
    required List<int> admin,
    bool isPublic = false,
    String cidNumber = '',
    String familyName = '管理',
    String givenName = '员',
  }) {
    final cidBytes = utf8.encode(cidNumber);
    final familyBytes = utf8.encode(familyName);
    final givenBytes = utf8.encode(givenName);
    return Uint8List.fromList([
      ...codeBytes(institutionCode),
      4,
      ...admin,
      if (isPublic) ...[cidBytes.length << 2, ...cidBytes],
      familyBytes.length << 2,
      ...familyBytes,
      givenBytes.length << 2,
      ...givenBytes,
    ]);
  }

  Uint8List personalAdminBytes({required List<int> admin}) {
    final familyBytes = utf8.encode('管理');
    final givenBytes = utf8.encode('员');
    return Uint8List.fromList([
      0,
      ...codeBytes('PMUL'),
      2,
      4,
      ...admin,
      familyBytes.length << 2,
      ...familyBytes,
      givenBytes.length << 2,
      ...givenBytes,
      ...List<int>.filled(32, 0xcc),
      ...u32Le(1),
      ...u32Le(2),
      1,
    ]);
  }

  String thresholdKey({
    required String palletName,
    required String storageName,
    required Uint8List keyData,
  }) {
    final bytes = <int>[
      ...Hasher.twoxx128.hashString(palletName),
      ...Hasher.twoxx128.hashString(storageName),
      ...AdminAccountIdCodec.blake2128Concat(keyData),
    ];
    return '0x${hexOf(bytes)}';
  }

  test('私权非法人机构按 CID 与显式 kind 路由', () async {
    const cidNumber = 'GD001-UNIN0-123456789-2026';
    final rpc = FakeChainRpc();
    final service = InstitutionAdminService(chainRpc: rpc);
    final accountKey =
        '0x${hexOf(AdminAccountIdCodec.institutionAdminStorageKey(
      cidNumber,
      institutionCode: 'UNIN',
      adminKind: 1,
    ))}';
    final thresholdStorageKey = thresholdKey(
      palletName: 'PrivateManage',
      storageName: 'InstitutionGovernanceThresholds',
      keyData: AdminAccountIdCodec.scaleBytes(utf8.encode(cidNumber)),
    );
    rpc.responses[accountKey] = institutionAdminBytes(
      institutionCode: 'UNIN',
      admin: List<int>.filled(32, 0xaa),
    );
    rpc.responses[thresholdStorageKey] = Uint8List.fromList(u32Le(2));

    final identity = AdminAccountIdentity.institution(
      cidNumber: cidNumber,
      institutionCode: 'UNIN',
      accountLabel: '机构账户',
      kind: 1,
    );
    expect(
      (await service.fetchAdmins(identity)).map((admin) => admin.admin_account),
      ['aa' * 32],
    );
    expect(await service.fetchThreshold(identity), 2);
    expect(rpc.requestedKeys, [accountKey, thresholdStorageKey]);
  });

  test('个人多签严格按 personal_account 路由', () async {
    final rpc = FakeChainRpc();
    final service = InstitutionAdminService(chainRpc: rpc);
    final accountHex = '22' * 32;
    final accountId = AdminAccountIdCodec.fromAccountHex(accountHex);
    final accountKey =
        '0x${hexOf(AdminAccountIdCodec.personalAdminStorageKey(accountId))}';
    final thresholdStorageKey = thresholdKey(
      palletName: 'InternalVote',
      storageName: 'ActivePersonalThresholds',
      keyData: accountId,
    );
    rpc.responses[accountKey] = personalAdminBytes(
      admin: List<int>.filled(32, 0xbb),
    );
    rpc.responses[thresholdStorageKey] = Uint8List.fromList(u32Le(2));

    final identity = AdminAccountIdentity.personalAccount(
      personalAccountHex: accountHex,
      accountLabel: '个人多签',
    );
    expect(
      (await service.fetchAdmins(identity)).map((admin) => admin.admin_account),
      ['bb' * 32],
    );
    expect(await service.fetchThreshold(identity), 2);
    expect(rpc.requestedKeys, [accountKey, thresholdStorageKey]);
  });

  test('公权机构从 PublicManage 读取机构阈值', () async {
    for (final entry in const {'FRG': 3, 'NJD': 8}.entries) {
      final cidNumber = 'ZS001-${entry.key}00-123456789-2026';
      final rpc = FakeChainRpc();
      final service = AdminAccountService(chainRpc: rpc);
      final identity = AdminAccountIdentity.institution(
        cidNumber: cidNumber,
        institutionCode: entry.key,
        accountLabel: entry.key,
      );
      final accountKey =
          '0x${hexOf(AdminAccountIdCodec.institutionAdminStorageKey(
        cidNumber,
        institutionCode: entry.key,
        adminKind: 0,
      ))}';
      rpc.responses[accountKey] = institutionAdminBytes(
        institutionCode: entry.key,
        admin: List<int>.filled(32, 0xee),
        isPublic: true,
        cidNumber: 'GZ000-CTZN6-198805200-2026',
        familyName: '',
        givenName: '',
      );
      final thresholdStorageKey = thresholdKey(
        palletName: 'PublicManage',
        storageName: 'InstitutionGovernanceThresholds',
        keyData: AdminAccountIdCodec.scaleBytes(utf8.encode(cidNumber)),
      );
      rpc.responses[thresholdStorageKey] =
          Uint8List.fromList(u32Le(entry.value));

      expect((await service.fetchByIdentity(identity))?.threshold, entry.value);
      final state = await service.fetchByIdentity(identity);
      expect(state?.admins.single.cid_number, 'GZ000-CTZN6-198805200-2026');
      expect(state?.admins.single.family_name, isEmpty);
      expect(rpc.requestedKeys, [accountKey, thresholdStorageKey]);
    }
  });

  test('管理员缓存按明确 identity key 隔离并可清除', () async {
    final rpc = FakeChainRpc();
    final service = AdminAccountService(chainRpc: rpc);
    final accountHex = '33' * 32;
    final accountId = AdminAccountIdCodec.fromAccountHex(accountHex);
    final identity = AdminAccountIdentity.personalAccount(
      personalAccountHex: accountHex,
      accountLabel: '个人多签',
    );
    final accountKey =
        '0x${hexOf(AdminAccountIdCodec.personalAdminStorageKey(accountId))}';
    final thresholdStorageKey = thresholdKey(
      palletName: 'InternalVote',
      storageName: 'ActivePersonalThresholds',
      keyData: accountId,
    );
    rpc.responses[accountKey] = personalAdminBytes(
      admin: List<int>.filled(32, 0xdd),
    );
    rpc.responses[thresholdStorageKey] = Uint8List.fromList(u32Le(2));

    await service.fetchByIdentity(identity);
    await service.fetchByIdentity(identity);
    expect(rpc.requestedKeys, [accountKey, thresholdStorageKey]);

    service.clearCache(identity);
    await service.fetchByIdentity(identity);
    expect(rpc.requestedKeys, [
      accountKey,
      thresholdStorageKey,
      accountKey,
      thresholdStorageKey,
    ]);
  });

  test('InstitutionInfo 解析为 CID 机构或个人多签明确身份', () {
    final personalAccount = '44' * 32;
    final personal = AdminAccountIdentity.fromInstitution(InstitutionInfo(
      cidFullName: '个人账户',
      cidShortName: '个人账户',
      cidFullNameEn: 'Personal Account',
      cidShortNameEn: 'Personal Account',
      cidNumber: 'personal-account:$personalAccount',
      orgType: OrgType.personalMultisig,
      personalAccountHex: personalAccount,
    ));
    expect(personal.type, AdminAccountIdentityType.personalAccount);
    expect(personal.personalAccountHex, personalAccount);

    const institutionCid = 'GD001-CGOV0-123456789-2026';
    final institution = AdminAccountIdentity.fromInstitution(InstitutionInfo(
      cidFullName: '机构账户',
      cidShortName: '机构账户',
      cidFullNameEn: 'Institution Account',
      cidShortNameEn: 'Institution Account',
      cidNumber: institutionCid,
      orgType: OrgType.institution,
      adminAccountCode: 'CGOV',
      accounts: InstitutionAccounts(
        mainAccount: '55' * 32,
        feeAccount: '56' * 32,
      ),
    ));
    expect(institution.type, AdminAccountIdentityType.institution);
    expect(institution.cidNumber, institutionCid);

    final privateOwnedInstitution = AdminAccountIdentity.institution(
      cidNumber: 'GD001-UNIN0-223456789-2026',
      institutionCode: 'UNIN',
      accountLabel: '私权非法人机构',
      kind: 1,
    );
    expect(privateOwnedInstitution.type, AdminAccountIdentityType.institution);
    expect(privateOwnedInstitution.kind, 1);

    final governance = AdminAccountIdentity.fromInstitution(InstitutionInfo(
      cidFullName: '中枢省公民储备银行',
      cidShortName: '中枢省储行',
      cidFullNameEn: 'Zhongshu Provincial Citizen Reserve Bank',
      cidShortNameEn: 'Zhongshu Provincial Reserve Bank',
      cidNumber: 'ZS001-PRB08-233384677-2026',
      orgType: OrgType.prb,
      accounts: InstitutionAccounts(
        mainAccount: '77' * 32,
        feeAccount: '78' * 32,
      ),
    ));
    expect(governance.type, AdminAccountIdentityType.institution);
    expect(governance.institutionCode, 'PRB');
  });

  test('管理员激活记录只以机构 CID 归属和去重', () async {
    const cidNumber = 'GD001-CGOV0-323456789-2026';
    final identity = AdminAccountIdentity.institution(
      cidNumber: cidNumber,
      institutionCode: 'CGOV',
      accountLabel: '公权机构',
    );
    const otherCidNumber = 'GD001-UNIN0-423456789-2026';
    final otherIdentity = AdminAccountIdentity.institution(
      cidNumber: otherCidNumber,
      institutionCode: 'UNIN',
      accountLabel: '私权机构',
      kind: 1,
    );
    final active = ActivatedAdmin(
      pubkeyHex: 'aa' * 32,
      cidNumber: cidNumber,
      institutionCode: identity.institutionCode,
      kind: identity.kind,
      activatedAtMs: 1,
    );
    final stale = ActivatedAdmin(
      pubkeyHex: 'bb' * 32,
      cidNumber: cidNumber,
      institutionCode: identity.institutionCode,
      kind: identity.kind,
      activatedAtMs: 2,
    );
    final unrelated = ActivatedAdmin(
      pubkeyHex: 'cc' * 32,
      cidNumber: otherCidNumber,
      institutionCode: otherIdentity.institutionCode,
      kind: otherIdentity.kind,
      activatedAtMs: 3,
    );

    SharedPreferences.setMockInitialValues({
      'activated_institution_admins_v1': jsonEncode(
        [active, stale, unrelated].map((item) => item.toJson()).toList(),
      ),
    });
    final service = ActivationService(
      adminService: FakeAdminService(
        admins: [
          AdminPerson(
            admin_account: 'aa' * 32,
            family_name: '管理',
            given_name: '员',
          ),
        ],
      ),
    );

    final records = await service.getActivatedAdmins(identity);
    expect(records.map((item) => item.pubkeyHex), ['aa' * 32]);
    expect(records.single.toJson()['cid_number'], cidNumber);
    expect((await service.loadAll()).map((item) => item.pubkeyHex).toSet(), {
      'aa' * 32,
      'cc' * 32,
    });
  });

  test('管理员人员左连接岗位且无岗位人员不丢失', () {
    final first = AdminPerson(
      admin_account: 'aa' * 32,
      family_name: '张',
      given_name: '三',
    );
    final second = AdminPerson(
      admin_account: 'bb' * 32,
      family_name: '李',
      given_name: '四',
    );
    final assignment = InstitutionAdminAssignment(
      cidNumber: 'CID-1',
      admin_account: first.admin_account,
      roleCode: 'DIRECTOR',
      roleName: '负责人',
      termStart: 0,
      termEnd: 0,
      source: InstitutionAssignmentSource.genesis,
      sourceRef: '',
      active: true,
    );

    final views = InstitutionAdminService.mergeAdminViews(
      [first, second],
      [assignment],
    );

    expect(views, hasLength(2));
    expect(views.first.assignments, [assignment]);
    expect(views.last.admin.admin_account, second.admin_account);
    expect(views.last.assignments, isEmpty);
  });
}
