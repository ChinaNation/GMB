import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/governance/shared/institution_info.dart';
import 'package:wuminapp_mobile/governance/admins-change/codec/subject_id_codec.dart';
import 'package:wuminapp_mobile/governance/admins-change/models/admin_subject.dart';
import 'package:wuminapp_mobile/governance/admins-change/services/admin_activation_service.dart';
import 'package:wuminapp_mobile/governance/admins-change/services/admin_subject_service.dart';
import 'package:wuminapp_mobile/governance/admins-change/services/institution_admin_service.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';

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

  final List<String> admins;

  @override
  Future<List<String>> fetchAdmins(AdminSubjectIdentity identity) async {
    return admins;
  }
}

void main() {
  String hexOf(List<int> bytes) =>
      bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();

  List<int> u32Le(int value) => [
        value & 0xff,
        (value >> 8) & 0xff,
        (value >> 16) & 0xff,
        (value >> 24) & 0xff,
      ];

  Uint8List adminSubjectBytes({
    required int org,
    required int kind,
    required List<int> admin,
  }) {
    return Uint8List.fromList([
      org,
      kind,
      (1 << 2) & 0xff,
      ...admin,
      ...List<int>.filled(32, 0xcc),
      ...u32Le(1),
      ...u32Le(2),
      1,
    ]);
  }

  Uint8List blake2128Concat(Uint8List data) {
    final hash = Hasher.blake2b128.hash(data);
    final result = Uint8List(hash.length + data.length);
    result.setAll(0, hash);
    result.setAll(hash.length, data);
    return result;
  }

  String dynamicThresholdKey({
    required String storageName,
    required int org,
    required Uint8List subjectId,
  }) {
    final palletHash = Hasher.twoxx128.hashString('InternalVote');
    final storageHash = Hasher.twoxx128.hashString(storageName);
    final orgKey = blake2128Concat(Uint8List.fromList([org]));
    final subjectKey = blake2128Concat(subjectId);
    final bytes = <int>[
      ...palletHash,
      ...storageHash,
      ...orgKey,
      ...subjectKey,
    ];
    return '0x${hexOf(bytes)}';
  }

  test('registered institution account routes to institution-account subject',
      () async {
    final rpc = FakeChainRpc();
    final service = InstitutionAdminService(chainRpc: rpc);
    final address = '11' * 32;
    final subjectId = AdminSubjectIdCodec.fromAccountHex(
      AdminSubjectIdCodec.institutionAccount,
      address,
    );
    final subjectKey = '0x${hexOf(AdminSubjectIdCodec.adminSubjectStorageKey(
      subjectId,
    ))}';
    final thresholdKey = dynamicThresholdKey(
      storageName: 'ActiveDynamicThresholds',
      org: 5,
      subjectId: subjectId,
    );
    rpc.responses[subjectKey] = adminSubjectBytes(
      org: 5,
      kind: 3,
      admin: List<int>.filled(32, 0xaa),
    );
    rpc.responses[thresholdKey] = Uint8List.fromList(u32Le(2));

    final identity = AdminSubjectIdentity.institutionAccount(
      accountHex: address,
      org: 5,
      displayName: '机构账户',
    );
    final admins = await service.fetchAdmins(identity);
    final threshold = await service.fetchThreshold(identity);

    expect(admins, ['aa' * 32]);
    expect(threshold, 2);
    expect(rpc.requestedKeys, [subjectKey, thresholdKey]);
  });

  test('personal institution routes directly to personal subject', () async {
    final rpc = FakeChainRpc();
    final service = InstitutionAdminService(chainRpc: rpc);
    final address = '22' * 32;
    final subjectId = AdminSubjectIdCodec.fromAccountHex(
      AdminSubjectIdCodec.personalDuoqian,
      address,
    );
    final subjectKey = '0x${hexOf(AdminSubjectIdCodec.adminSubjectStorageKey(
      subjectId,
    ))}';
    final thresholdKey = dynamicThresholdKey(
      storageName: 'ActiveDynamicThresholds',
      org: 3,
      subjectId: subjectId,
    );
    rpc.responses[subjectKey] = adminSubjectBytes(
      org: 3,
      kind: 2,
      admin: List<int>.filled(32, 0xbb),
    );
    rpc.responses[thresholdKey] = Uint8List.fromList(u32Le(2));

    final identity = AdminSubjectIdentity.personalDuoqian(
      accountHex: address,
      displayName: '个人多签',
    );
    final admins = await service.fetchAdmins(identity);
    final threshold = await service.fetchThreshold(identity);

    expect(admins, ['bb' * 32]);
    expect(threshold, 2);
    expect(rpc.requestedKeys, [subjectKey, thresholdKey]);
  });

  test('subject service cache is keyed by subject id', () async {
    final rpc = FakeChainRpc();
    final service = AdminSubjectService(chainRpc: rpc);
    final subjectId = AdminSubjectIdCodec.fromAccountHex(
      AdminSubjectIdCodec.personalDuoqian,
      '33' * 32,
    );
    final subjectKey =
        '0x${hexOf(AdminSubjectIdCodec.adminSubjectStorageKey(subjectId))}';
    final thresholdKey = dynamicThresholdKey(
      storageName: 'ActiveDynamicThresholds',
      org: 3,
      subjectId: subjectId,
    );
    rpc.responses[subjectKey] = adminSubjectBytes(
      org: 3,
      kind: 2,
      admin: List<int>.filled(32, 0xdd),
    );
    rpc.responses[thresholdKey] = Uint8List.fromList(u32Le(2));

    await service.fetchBySubjectId(subjectId);
    await service.fetchBySubjectId(subjectId);
    expect(rpc.requestedKeys, [subjectKey, thresholdKey]);

    service.clearSubjectCache(AdminSubjectIdCodec.hexEncode(subjectId));
    await service.fetchBySubjectId(subjectId);
    expect(rpc.requestedKeys, [
      subjectKey,
      thresholdKey,
      subjectKey,
      thresholdKey,
    ]);
  });

  test('institution info resolves to explicit admins-change identity', () {
    final personalAddress = '44' * 32;
    final personal = AdminSubjectIdentity.fromInstitution(InstitutionInfo(
      name: '个人账户',
      sfidNumber: 'personal:$personalAddress',
      orgType: OrgType.duoqian,
      duoqianAddress: personalAddress,
    ));
    expect(personal.type, AdminSubjectIdentityType.personalDuoqian);
    expect(personal.org, 3);
    expect(personal.kind, 2);

    final accountAddress = '55' * 32;
    final institutionAccount =
        AdminSubjectIdentity.fromInstitution(InstitutionInfo(
      name: '机构账户',
      sfidNumber: registeredDuoqianIdentity(accountAddress),
      orgType: OrgType.duoqian,
      adminSubjectOrg: 5,
      duoqianAddress: accountAddress,
    ));
    expect(
        institutionAccount.type, AdminSubjectIdentityType.institutionAccount);
    expect(institutionAccount.org, 5);
    expect(institutionAccount.kind, 3);

    final governance =
        AdminSubjectIdentity.fromInstitution(const InstitutionInfo(
      name: '省储行',
      sfidNumber: 'GFR-LN001-CB0X-944805165-2026',
      orgType: OrgType.prb,
      accounts: InstitutionAccounts(mainAddress: '66'),
    ));
    expect(governance.type, AdminSubjectIdentityType.governanceInstitution);
    expect(governance.org, OrgType.prb);
    expect(governance.kind, 0);
  });

  test('activation records use v3 subject identity without sfidNumber',
      () async {
    final identity = AdminSubjectIdentity.personalDuoqian(
      accountHex: '77' * 32,
      displayName: '个人多签',
    );
    final otherIdentity = AdminSubjectIdentity.institutionAccount(
      accountHex: '88' * 32,
      org: 5,
      displayName: '机构账户',
    );
    final active = ActivatedAdmin(
      pubkeyHex: 'aa' * 32,
      identityKey: identity.identityKey,
      subjectIdHex: identity.subjectIdHex,
      org: identity.org,
      kind: identity.kind,
      activatedAtMs: 1,
    );
    final stale = ActivatedAdmin(
      pubkeyHex: 'bb' * 32,
      identityKey: identity.identityKey,
      subjectIdHex: identity.subjectIdHex,
      org: identity.org,
      kind: identity.kind,
      activatedAtMs: 2,
    );
    final unrelated = ActivatedAdmin(
      pubkeyHex: 'cc' * 32,
      identityKey: otherIdentity.identityKey,
      subjectIdHex: otherIdentity.subjectIdHex,
      org: otherIdentity.org,
      kind: otherIdentity.kind,
      activatedAtMs: 3,
    );

    SharedPreferences.setMockInitialValues({
      'activated_admins_v2': jsonEncode([
        {
          'pubkeyHex': 'dd' * 32,
          'sfidNumber': 'personal:${'77' * 32}',
          'activatedAtMs': 0,
        }
      ]),
      'activated_admins_v3': jsonEncode(
        [active, stale, unrelated].map((item) => item.toJson()).toList(),
      ),
    });

    final service = ActivationService(
      adminService: FakeAdminService(admins: ['aa' * 32]),
    );

    final records = await service.getActivatedAdmins(identity);
    expect(records.map((item) => item.pubkeyHex).toList(), ['aa' * 32]);
    expect(records.single.toJson().containsKey('sfidNumber'), isFalse);

    final all = await service.loadAll();
    expect(all.map((item) => item.pubkeyHex).toSet(), {'aa' * 32, 'cc' * 32});
  });
}
