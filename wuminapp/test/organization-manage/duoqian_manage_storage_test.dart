import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/organization-manage/duoqian_manage_models.dart';
import 'package:wuminapp_mobile/organization-manage/duoqian_manage_service.dart';
import 'package:wuminapp_mobile/organization-manage/duoqian_storage_codec.dart';
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

  Uint8List adminSubjectBytes({
    required List<int> admin1,
    required List<int> admin2,
  }) {
    return Uint8List.fromList([
      3,
      DuoqianStorageCodec.subjectKindInstitutionAccount,
      (2 << 2) & 0xff,
      ...admin1,
      ...admin2,
      ...u32Le(2),
    ]);
  }

  test('fetchDuoqianAccount reads registered institution current storages',
      () async {
    final rpc = FakeChainRpc();
    final service = DuoqianManageService(chainRpc: rpc);
    final address = '11' * 32;
    final sfidNumber = Uint8List.fromList(utf8.encode('SFR-AH001-20260507'));
    final accountName = Uint8List.fromList(utf8.encode('主账户'));

    final refKey =
        '0x${hexOf(DuoqianStorageCodec.addressRegisteredSfidKey(address))}';
    final accountKey = '0x${hexOf(DuoqianStorageCodec.institutionAccountKey(
      sfidNumber,
      accountName,
    ))}';
    final adminKey = '0x${hexOf(DuoqianStorageCodec.adminSubjectKey(
      DuoqianStorageCodec.subjectIdFromInstitutionAccountHex(address),
    ))}';
    rpc.responses[refKey] = Uint8List.fromList([
      ...compactVec('SFR-AH001-20260507'),
      ...compactVec('主账户'),
    ]);
    rpc.responses[adminKey] = adminSubjectBytes(
      admin1: List<int>.filled(32, 0xaa),
      admin2: List<int>.filled(32, 0xbb),
    );
    rpc.responses[accountKey] = Uint8List.fromList([
      ...List<int>.filled(32, 0xd1),
      ...u128Le(BigInt.from(111)),
      1,
      1,
      ...u32Le(100),
    ]);

    final info = await service.fetchDuoqianAccount(address);

    expect(info, isNotNull);
    expect(info!.adminCount, 2);
    expect(info.threshold, 2);
    expect(info.adminPubkeys, ['aa' * 32, 'bb' * 32]);
    expect(info.status, DuoqianStatus.active);
    expect(rpc.requestedKeys, [refKey, accountKey, adminKey]);
  });
}
