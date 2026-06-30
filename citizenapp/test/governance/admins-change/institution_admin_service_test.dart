import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:shared_preferences/shared_preferences.dart';
import 'package:citizenapp/citizen/shared/institution_info.dart';
import 'package:citizenapp/citizen/proposal/admins-change/codec/account_id_codec.dart';
import 'package:citizenapp/citizen/proposal/admins-change/models/admin_account.dart';
import 'package:citizenapp/citizen/proposal/admins-change/services/admin_activation_service.dart';
import 'package:citizenapp/citizen/proposal/admins-change/services/admin_account_service.dart';
import 'package:citizenapp/citizen/proposal/admins-change/services/institution_admin_service.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';

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
  Future<List<String>> fetchAdmins(AdminAccountIdentity identity) async {
    return admins;
  }
}

void main() {
  String hexOf(List<int> bytes) =>
      bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();

  List<int> codeBytes(String code) {
    final out = List<int>.filled(4, 0);
    final raw = code.codeUnits;
    for (var i = 0; i < out.length && i < raw.length; i++) {
      out[i] = raw[i];
    }
    return out;
  }

  List<int> u32Le(int value) => [
        value & 0xff,
        (value >> 8) & 0xff,
        (value >> 16) & 0xff,
        (value >> 24) & 0xff,
      ];

  // A2:`AdminAccounts.admins` = `Vec<AdminProfile>`(机构 kind≠3:account[32] + cid/name/admin_role
  // 各 Compact(0) 空 + term_start/term_end u32 + source u8);个人多签 kind==3 仍是裸 `Vec<AccountId>`。
  // 逐字节对齐 lib/citizen/shared/admin_profile.dart::decodeAdminsVec。
  Uint8List adminAccountBytes({
    required String institutionCode,
    required int kind,
    required List<int> admin,
  }) {
    final adminEntry = <int>[...admin];
    if (kind != 3) {
      adminEntry.addAll([
        0x00, // admin_cid_number: Compact(0) 空
        0x00, // name: Compact(0) 空
        0x00, // admin_role: Compact(0) 空
        ...u32Le(0), // term_start
        ...u32Le(0), // term_end
        0x00, // source: Genesis(0)
      ]);
    }
    return Uint8List.fromList([
      ...codeBytes(institutionCode),
      kind,
      (1 << 2) & 0xff, // admins Vec: Compact(1)
      ...adminEntry,
      ...List<int>.filled(32, 0xcc), // creator
      ...u32Le(1), // created_at
      ...u32Le(2), // updated_at
      1, // status: Active
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
    required String institutionCode,
    required Uint8List accountId,
  }) {
    final palletHash = Hasher.twoxx128.hashString('InternalVote');
    final storageHash = Hasher.twoxx128.hashString(storageName);
    final codeKey = blake2128Concat(Uint8List.fromList(
      codeBytes(institutionCode),
    ));
    final accountKey = blake2128Concat(accountId);
    final bytes = <int>[
      ...palletHash,
      ...storageHash,
      ...codeKey,
      ...accountKey,
    ];
    return '0x${hexOf(bytes)}';
  }

  test('private-owned unincorporated account routes by explicit admin kind',
      () async {
    final rpc = FakeChainRpc();
    final service = InstitutionAdminService(chainRpc: rpc);
    final address = '11' * 32;
    final accountId = AdminAccountIdCodec.fromAccountHex(address);
    final accountKey = '0x${hexOf(AdminAccountIdCodec.adminAccountStorageKey(
      accountId,
      institutionCode: 'UNIN',
      adminKind: 2,
    ))}';
    final thresholdKey = dynamicThresholdKey(
      storageName: 'ActiveDynamicThresholds',
      institutionCode: 'UNIN',
      accountId: accountId,
    );
    rpc.responses[accountKey] = adminAccountBytes(
      institutionCode: 'UNIN',
      kind: 2,
      admin: List<int>.filled(32, 0xaa),
    );
    rpc.responses[thresholdKey] = Uint8List.fromList(u32Le(2));

    final identity = AdminAccountIdentity.institutionAccount(
      accountHex: address,
      institutionCode: 'UNIN',
      accountLabel: '机构账户',
      kind: 2,
    );
    final admins = await service.fetchAdmins(identity);
    final threshold = await service.fetchThreshold(identity);

    expect(admins, ['aa' * 32]);
    expect(threshold, 2);
    expect(rpc.requestedKeys, [accountKey, thresholdKey]);
  });

  test('personal institution routes directly to personal account', () async {
    final rpc = FakeChainRpc();
    final service = InstitutionAdminService(chainRpc: rpc);
    final address = '22' * 32;
    final accountId = AdminAccountIdCodec.fromAccountHex(address);
    final accountKey = '0x${hexOf(AdminAccountIdCodec.adminAccountStorageKey(
      accountId,
      institutionCode: 'PMUL',
    ))}';
    final thresholdKey = dynamicThresholdKey(
      storageName: 'ActiveDynamicThresholds',
      institutionCode: 'PMUL',
      accountId: accountId,
    );
    rpc.responses[accountKey] = adminAccountBytes(
      institutionCode: 'PMUL',
      kind: 3,
      admin: List<int>.filled(32, 0xbb),
    );
    rpc.responses[thresholdKey] = Uint8List.fromList(u32Le(2));

    final identity = AdminAccountIdentity.personalAccount(
      accountHex: address,
      accountLabel: '个人多签',
    );
    final admins = await service.fetchAdmins(identity);
    final threshold = await service.fetchThreshold(identity);

    expect(admins, ['bb' * 32]);
    expect(threshold, 2);
    expect(rpc.requestedKeys, [accountKey, thresholdKey]);
  });

  test('account service cache is keyed by account id', () async {
    final rpc = FakeChainRpc();
    final service = AdminAccountService(chainRpc: rpc);
    final accountId = AdminAccountIdCodec.fromAccountHex('33' * 32);
    final accountKey = '0x${hexOf(AdminAccountIdCodec.adminAccountStorageKey(
      accountId,
      institutionCode: 'PMUL',
    ))}';
    final thresholdKey = dynamicThresholdKey(
      storageName: 'ActiveDynamicThresholds',
      institutionCode: 'PMUL',
      accountId: accountId,
    );
    rpc.responses[accountKey] = adminAccountBytes(
      institutionCode: 'PMUL',
      kind: 3,
      admin: List<int>.filled(32, 0xdd),
    );
    rpc.responses[thresholdKey] = Uint8List.fromList(u32Le(2));

    await service.fetchByAccountId(accountId, institutionCode: 'PMUL');
    await service.fetchByAccountId(accountId, institutionCode: 'PMUL');
    expect(rpc.requestedKeys, [accountKey, thresholdKey]);

    service.clearAccountCache(AdminAccountIdCodec.hexEncode(accountId));
    await service.fetchByAccountId(accountId, institutionCode: 'PMUL');
    expect(rpc.requestedKeys, [
      accountKey,
      thresholdKey,
      accountKey,
      thresholdKey,
    ]);
  });

  test('institution info resolves to explicit admins-change identity', () {
    final personalAccount = '44' * 32;
    final personal = AdminAccountIdentity.fromInstitution(InstitutionInfo(
      cidFullName: '个人账户',
      cidShortName: '个人账户',
      cidFullNameEn: 'Personal Account',
      cidShortNameEn: 'Personal Account',
      cidNumber: 'personal-account:$personalAccount',
      orgType: OrgType.account,
      account: personalAccount,
    ));
    expect(personal.type, AdminAccountIdentityType.personalAccount);
    expect(personal.institutionCode, 'PMUL');
    expect(personal.kind, 3);

    final accountAddress = '55' * 32;
    expect(
      () => AdminAccountIdentity.fromInstitution(InstitutionInfo(
        cidFullName: '机构账户',
        cidShortName: '机构账户',
        cidFullNameEn: 'Institution Account',
        cidShortNameEn: 'Institution Account',
        cidNumber: registeredAccountIdentity(accountAddress),
        orgType: OrgType.account,
        adminAccountCode: 'UNIN',
        account: accountAddress,
      )),
      throwsA(isA<ArgumentError>()),
    );

    final privateOwnedInstitution = AdminAccountIdentity.institutionAccount(
      accountHex: accountAddress,
      institutionCode: 'UNIN',
      accountLabel: '机构账户',
      kind: 2,
    );
    expect(privateOwnedInstitution.type,
        AdminAccountIdentityType.institutionAccount);
    expect(privateOwnedInstitution.institutionCode, 'UNIN');
    expect(privateOwnedInstitution.kind, 2);

    final governance =
        AdminAccountIdentity.fromInstitution(const InstitutionInfo(
      cidFullName: '中枢省公民储备银行',
      cidShortName: '中枢省储行',
      cidFullNameEn: 'Zhongshu Provincial Citizen Reserve Bank',
      cidShortNameEn: 'Zhongshu Provincial Reserve Bank',
      cidNumber: 'ZS001-PRB08-233384677-2026',
      orgType: OrgType.prb,
      accounts: InstitutionAccounts(mainAccount: '66'),
    ));
    expect(governance.type, AdminAccountIdentityType.governanceInstitution);
    expect(governance.institutionCode, 'PRB');
    expect(governance.kind, 0);
  });

  test('activation records use v3 account identity without cidNumber',
      () async {
    final identity = AdminAccountIdentity.personalAccount(
      accountHex: '77' * 32,
      accountLabel: '个人多签',
    );
    final otherIdentity = AdminAccountIdentity.institutionAccount(
      accountHex: '88' * 32,
      institutionCode: 'UNIN',
      accountLabel: '机构账户',
      kind: 2,
    );
    final active = ActivatedAdmin(
      pubkeyHex: 'aa' * 32,
      identityKey: identity.identityKey,
      accountHex: identity.accountHex,
      institutionCode: identity.institutionCode,
      kind: identity.kind,
      activatedAtMs: 1,
    );
    final stale = ActivatedAdmin(
      pubkeyHex: 'bb' * 32,
      identityKey: identity.identityKey,
      accountHex: identity.accountHex,
      institutionCode: identity.institutionCode,
      kind: identity.kind,
      activatedAtMs: 2,
    );
    final unrelated = ActivatedAdmin(
      pubkeyHex: 'cc' * 32,
      identityKey: otherIdentity.identityKey,
      accountHex: otherIdentity.accountHex,
      institutionCode: otherIdentity.institutionCode,
      kind: otherIdentity.kind,
      activatedAtMs: 3,
    );

    SharedPreferences.setMockInitialValues({
      'activated_admin_accounts_v1': jsonEncode(
        [active, stale, unrelated].map((item) => item.toJson()).toList(),
      ),
    });

    final service = ActivationService(
      adminService: FakeAdminService(admins: ['aa' * 32]),
    );

    final records = await service.getActivatedAdmins(identity);
    expect(records.map((item) => item.pubkeyHex).toList(), ['aa' * 32]);
    expect(records.single.toJson().containsKey('cidNumber'), isFalse);

    final all = await service.loadAll();
    expect(all.map((item) => item.pubkeyHex).toSet(), {'aa' * 32, 'cc' * 32});
  });
}
