// AdminAccountsScanService.filterMine 纯函数单测(ADR-018 §九)。
//
// filterMine 是个人多签发现的"按 kind + 本地钱包"分流逻辑,
// 纯函数无链依赖。链上扫描路径(getKeysPaged + fetchStorageBatch)受 smoldot
// 真链依赖,留给端到端校核覆盖。

import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/citizen/shared/admin_account_storage_codec.dart';
import 'package:citizenapp/citizen/shared/admin_accounts_scan_service.dart';
import 'package:citizenapp/citizen/proposal/admins-change/models/admin_account.dart';

void main() {
  final myWallet = 'aa' * 32; // 64 hex
  final otherWallet = 'bb' * 32;
  final secondWallet = 'cc' * 32;

  AdminAccountsScanResult resultOf(List<ScannedAdminAccount> accounts) =>
      AdminAccountsScanResult(
        accounts: accounts,
        totalKeys: accounts.length,
        partialFailure: false,
      );

  ScannedAdminAccount acc({
    required String key,
    required String institutionCode,
    required int kind,
    required List<String> adminAccounts,
  }) =>
      ScannedAdminAccount(
        cidNumber: kind == AdminAccountStorageCodec.kindPersonal ? null : key,
        personalAccountHex:
            kind == AdminAccountStorageCodec.kindPersonal ? key : null,
        institutionCode: institutionCode,
        kind: kind,
        admins: adminAccounts
            .map(
              (account) => AdminPerson(
                admin_account: account,
                family_name: '管理',
                given_name: '员',
              ),
            )
            .toList(growable: false),
      );

  group('AdminAccountsScanService.filterMine', () {
    test('按 kind 分流:只保留个人多签', () {
      final scan = resultOf([
        acc(
          key: 'CID-01',
          institutionCode: 'CGOV',
          kind: AdminAccountStorageCodec.kindPublicInstitution,
          adminAccounts: [myWallet],
        ),
        acc(
          key: '02',
          institutionCode: 'PMUL',
          kind: AdminAccountStorageCodec.kindPersonal,
          adminAccounts: [myWallet],
        ),
      ]);

      final personals = AdminAccountsScanService.filterMine(
        scan,
        myPubkeysHex: {myWallet},
        kind: AdminAccountStorageCodec.kindPersonal,
      );
      expect(personals.map((e) => e.personalAccountHex), ['02']);
    });

    test('institution_code 白名单:个人多签仍可按 PMUL 过滤', () {
      final scan = resultOf([
        acc(
          key: '01',
          institutionCode: 'PMUL',
          kind: AdminAccountStorageCodec.kindPersonal,
          adminAccounts: [myWallet],
        ),
        acc(
          key: '02',
          institutionCode: 'XXXX',
          kind: AdminAccountStorageCodec.kindPersonal,
          adminAccounts: [myWallet],
        ),
      ]);
      final result = AdminAccountsScanService.filterMine(
        scan,
        myPubkeysHex: {myWallet},
        kind: AdminAccountStorageCodec.kindPersonal,
        codeWhitelist: const {'PMUL'},
      );
      expect(result.map((e) => e.personalAccountHex), ['01']);
    });

    test('钱包匹配:管理员不含本地钱包的账户被排除', () {
      final scan = resultOf([
        acc(
            key: '01',
            institutionCode: 'PMUL',
            kind: AdminAccountStorageCodec.kindPersonal,
            adminAccounts: [myWallet, otherWallet]),
        acc(
            key: '02',
            institutionCode: 'PMUL',
            kind: AdminAccountStorageCodec.kindPersonal,
            adminAccounts: [otherWallet]),
      ]);
      final result = AdminAccountsScanService.filterMine(
        scan,
        myPubkeysHex: {myWallet},
        kind: AdminAccountStorageCodec.kindPersonal,
      );
      expect(result.map((e) => e.personalAccountHex), ['01']);
    });

    test('多钱包:命中任一本地钱包即保留', () {
      final scan = resultOf([
        acc(
            key: '01',
            institutionCode: 'PMUL',
            kind: AdminAccountStorageCodec.kindPersonal,
            adminAccounts: [secondWallet]),
      ]);
      final result = AdminAccountsScanService.filterMine(
        scan,
        myPubkeysHex: {myWallet, secondWallet},
        kind: AdminAccountStorageCodec.kindPersonal,
      );
      expect(result.map((e) => e.personalAccountHex), ['01']);
    });

    test('空扫描结果返回空', () {
      final result = AdminAccountsScanService.filterMine(
        AdminAccountsScanResult.empty,
        myPubkeysHex: {myWallet},
        kind: 1,
        codeWhitelist: const {'CGOV', 'UNIN'},
      );
      expect(result, isEmpty);
    });
  });
}
