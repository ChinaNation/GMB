import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/governance/organization-manage/institution_manage_service.dart';

void main() {
  String hexOf(List<int> bytes) =>
      bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();

  List<int> compactVec(String text) {
    final bytes = utf8.encode(text);
    return [(bytes.length << 2) & 0xff, ...bytes];
  }

  List<int> u32Le(int value) => [
        value & 0xff,
        (value >> 8) & 0xff,
        (value >> 16) & 0xff,
        (value >> 24) & 0xff,
      ];

  List<int> u128Le(BigInt value) {
    final out = List<int>.filled(16, 0);
    var tmp = value;
    for (var i = 0; i < 16; i++) {
      out[i] = (tmp & BigInt.from(0xff)).toInt();
      tmp = tmp >> 8;
    }
    return out;
  }

  group('InstitutionManageService', () {
    test('builds propose_create_institution call_data as P-TX-001 11 fields',
        () {
      final admin1 = Uint8List.fromList(List<int>.filled(32, 0x11));
      final admin2 = Uint8List.fromList(List<int>.filled(32, 0x22));
      final signature = List<int>.filled(64, 0xdd);
      final signerAdmin = List<int>.generate(32, (i) => 0xc0 + (i & 0x0f));

      final callData =
          InstitutionManageService.buildProposeCreateInstitutionCallData(
        sfidNumber: 'AH001-SCB0N-202605010-2026',
        institutionName: '安徽省储行',
        accounts: [
          InstitutionInitialAccountInput(
            accountName: '主账户',
            amountFen: BigInt.from(111),
          ),
          InstitutionInitialAccountInput(
            accountName: '费用账户',
            amountFen: BigInt.from(222),
          ),
        ],
        adminOrg: 5,
        adminCount: 2,
        adminPubkeys: [admin1, admin2],
        threshold: 2,
        registerNonce: 'reg-nonce-001',
        signatureHex: '0x${hexOf(signature)}',
        province: '安徽省',
        signerAdminPubkeyHex: '0x${hexOf(signerAdmin)}',
      );

      final expected = <int>[
        0x11,
        0x05,
        ...compactVec('AH001-SCB0N-202605010-2026'),
        ...compactVec('安徽省储行'),
        (2 << 2) & 0xff,
        ...compactVec('主账户'),
        ...u128Le(BigInt.from(111)),
        ...compactVec('费用账户'),
        ...u128Le(BigInt.from(222)),
        5,
        ...u32Le(2),
        (2 << 2) & 0xff,
        ...admin1,
        ...admin2,
        ...u32Le(2),
        ...compactVec('reg-nonce-001'),
        0x01,
        0x01,
        ...signature,
        ...compactVec('安徽省'),
        ...signerAdmin,
      ];

      expect(hexOf(callData), hexOf(expected));
    });
  });
}
