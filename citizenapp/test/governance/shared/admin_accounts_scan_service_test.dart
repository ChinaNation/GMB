// AdminAccountsScanService.filterMine 纯函数单测(ADR-018 §九)。
//
// filterMine 是机构/个人多签共用的"按 kind + institution_code 白名单 + 本地钱包"分流逻辑,
// 纯函数无链依赖。链上扫描路径(getKeysPaged + fetchStorageBatch)受 smoldot
// 真链依赖,留给端到端校核覆盖。

import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/governance/shared/admin_account_storage_codec.dart';
import 'package:citizenapp/governance/shared/admin_accounts_scan_service.dart';

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
    required String addr,
    required String institutionCode,
    required int kind,
    required List<String> admins,
  }) =>
      ScannedAdminAccount(
        addrHex: addr,
        institutionCode: institutionCode,
        kind: kind,
        adminsHex: admins,
      );

  group('AdminAccountsScanService.filterMine', () {
    test('按 kind 分流:机构(2)与个人(1)互不串味', () {
      final scan = resultOf([
        acc(
          addr: '01',
          institutionCode: 'CGOV',
          kind: AdminAccountStorageCodec.kindInstitutionAccount,
          admins: [myWallet],
        ),
        acc(
          addr: '02',
          institutionCode: 'PMUL',
          kind: AdminAccountStorageCodec.kindPersonal,
          admins: [myWallet],
        ),
      ]);

      final institutions = AdminAccountsScanService.filterMine(
        scan,
        myPubkeysHex: {myWallet},
        kind: AdminAccountStorageCodec.kindInstitutionAccount,
        codeWhitelist: const {'CGOV', 'UNIN'},
      );
      expect(institutions.map((e) => e.addrHex), ['01']);

      final personals = AdminAccountsScanService.filterMine(
        scan,
        myPubkeysHex: {myWallet},
        kind: AdminAccountStorageCodec.kindPersonal,
      );
      expect(personals.map((e) => e.addrHex), ['02']);
    });

    test('institution_code 白名单:不在注册机构码集合的机构账户被排除', () {
      final scan = resultOf([
        acc(addr: '01', institutionCode: 'CGOV', kind: 2, admins: [myWallet]),
        acc(addr: '02', institutionCode: 'PRC', kind: 2, admins: [myWallet]),
      ]);
      final result = AdminAccountsScanService.filterMine(
        scan,
        myPubkeysHex: {myWallet},
        kind: 2,
        codeWhitelist: const {'CGOV', 'UNIN'},
      );
      expect(result.map((e) => e.addrHex), ['01']);
    });

    test('钱包匹配:管理员不含本地钱包的账户被排除', () {
      final scan = resultOf([
        acc(
            addr: '01',
            institutionCode: 'CGOV',
            kind: 2,
            admins: [myWallet, otherWallet]),
        acc(
            addr: '02',
            institutionCode: 'UNIN',
            kind: 2,
            admins: [otherWallet]),
      ]);
      final result = AdminAccountsScanService.filterMine(
        scan,
        myPubkeysHex: {myWallet},
        kind: 2,
        codeWhitelist: const {'CGOV', 'UNIN'},
      );
      expect(result.map((e) => e.addrHex), ['01']);
    });

    test('多钱包:命中任一本地钱包即保留', () {
      final scan = resultOf([
        acc(
            addr: '01',
            institutionCode: 'CGOV',
            kind: 2,
            admins: [secondWallet]),
      ]);
      final result = AdminAccountsScanService.filterMine(
        scan,
        myPubkeysHex: {myWallet, secondWallet},
        kind: 2,
        codeWhitelist: const {'CGOV', 'UNIN'},
      );
      expect(result.map((e) => e.addrHex), ['01']);
    });

    test('空扫描结果返回空', () {
      final result = AdminAccountsScanService.filterMine(
        AdminAccountsScanResult.empty,
        myPubkeysHex: {myWallet},
        kind: 2,
        codeWhitelist: const {'CGOV', 'UNIN'},
      );
      expect(result, isEmpty);
    });
  });
}
