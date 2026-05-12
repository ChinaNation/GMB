import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/governance/personal-manage/personal_manage_models.dart';
import 'package:wuminapp_mobile/governance/personal-manage/personal_manage_service.dart';
import 'package:wuminapp_mobile/governance/personal-manage/personal_manage_storage_codec.dart';
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

void main() {
  String hexOf(List<int> bytes) =>
      bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();

  List<int> compactVec(String text) {
    final bytes = utf8.encode(text);
    return [(bytes.length << 2) & 0xff, ...bytes];
  }

  List<int> compactU32(int value) {
    if (value < 64) return [(value << 2) & 0xff];
    final encoded = (value << 2) | 0x01;
    return [encoded & 0xff, (encoded >> 8) & 0xff];
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

  Uint8List personalAccountBytes() {
    return Uint8List.fromList([
      ...List<int>.filled(32, 0xc2),
      ...compactVec('家庭基金'),
      ...u32Le(100),
      1,
    ]);
  }

  Uint8List adminSubjectBytes({
    required List<int> admin1,
    required List<int> admin2,
  }) {
    return Uint8List.fromList([
      3,
      PersonalManageStorageCodec.subjectKindPersonalDuoqian,
      (2 << 2) & 0xff,
      ...admin1,
      ...admin2,
      ...u32Le(2),
    ]);
  }

  group('PersonalManageService', () {
    test('builds propose_create_personal call_data with regular_threshold', () {
      final admin1 = Uint8List.fromList(List<int>.filled(32, 0x11));
      final admin2 = Uint8List.fromList(List<int>.filled(32, 0x22));
      final accountName = Uint8List.fromList(utf8.encode('家庭基金'));

      final callData = PersonalManageService.buildProposeCreatePersonalCallData(
        accountName: accountName,
        adminPubkeys: [admin1, admin2],
        regularThreshold: 2,
        amountFen: BigInt.from(111),
      );

      final expected = <int>[
        0x07,
        0x00,
        ...compactVec('家庭基金'),
        (2 << 2) & 0xff,
        ...admin1,
        ...admin2,
        ...u32Le(2),
        ...u128Le(BigInt.from(111)),
      ];

      expect(hexOf(callData), hexOf(expected));
    });

    test('rejects regular_threshold below strict majority', () {
      final admins = List.generate(
        4,
        (i) => Uint8List.fromList(List<int>.filled(32, 0x10 + i)),
      );

      expect(
        () => PersonalManageService.buildProposeCreatePersonalCallData(
          accountName: Uint8List.fromList(utf8.encode('家庭基金')),
          adminPubkeys: admins,
          regularThreshold: 2,
          amountFen: BigInt.from(111),
        ),
        throwsArgumentError,
      );
    });

    test('decodes current PersonalManage create ProposalData', () {
      final service = PersonalManageService();
      final inner = <int>[
        ...utf8.encode('per-mgmt'),
        0x00,
        ...List<int>.filled(32, 0x33),
        ...List<int>.filled(32, 0x44),
        ...u128Le(BigInt.from(111)),
        ...u128Le(BigInt.from(10)),
      ];
      final raw = Uint8List.fromList([
        ...compactU32(inner.length),
        ...inner,
      ]);

      final decoded = service.decodePersonalProposalData(7, raw);

      expect(decoded, isA<CreateDuoqianProposalInfo>());
      final info = decoded as CreateDuoqianProposalInfo;
      expect(info.proposalId, 7);
      expect(info.duoqianAddress, '33' * 32);
      expect(info.amountFen, BigInt.from(111));
      expect(info.feeFen, BigInt.from(10));
    });

    test('fetchPersonalAccount reads PersonalManage current storage', () async {
      final rpc = FakeChainRpc();
      final service = PersonalManageService(chainRpc: rpc);
      final address = '22' * 32;
      final personalKey =
          '0x${hexOf(PersonalManageStorageCodec.personalDuoqiansKey(address))}';
      final adminKey = '0x${hexOf(PersonalManageStorageCodec.adminSubjectKey(
        PersonalManageStorageCodec.subjectIdFromAccountHex(address),
      ))}';
      rpc.responses[personalKey] = personalAccountBytes();
      rpc.responses[adminKey] = adminSubjectBytes(
        admin1: List<int>.filled(32, 0xcc),
        admin2: List<int>.filled(32, 0xdd),
      );

      final info = await service.fetchPersonalAccount(address);

      expect(info, isNotNull);
      expect(info!.adminPubkeys, ['cc' * 32, 'dd' * 32]);
      expect(info.status, DuoqianStatus.active);
      expect(rpc.requestedKeys, [personalKey, adminKey]);
    });
  });
}
