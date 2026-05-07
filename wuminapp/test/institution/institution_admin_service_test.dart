import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/duoqian/shared/duoqian_storage_codec.dart';
import 'package:wuminapp_mobile/institution/institution_admin_service.dart';
import 'package:wuminapp_mobile/institution/institution_data.dart';
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

  Uint8List adminSubjectBytes({
    required int threshold,
    required List<int> admin,
  }) {
    return Uint8List.fromList([
      3,
      1,
      (1 << 2) & 0xff,
      ...admin,
      ...u32Le(threshold),
    ]);
  }

  test(
      'registered institution routes through AddressRegisteredSfid then subject',
      () async {
    final rpc = FakeChainRpc();
    final service = InstitutionAdminService(chainRpc: rpc);
    final address = '11' * 32;
    final sfidNumber = Uint8List.fromList(utf8.encode('SFR-AH001-20260507'));
    final refKey =
        '0x${hexOf(DuoqianStorageCodec.addressRegisteredSfidKey(address))}';
    final subjectKey = '0x${hexOf(DuoqianStorageCodec.adminSubjectKey(
      DuoqianStorageCodec.subjectIdFromSfidBytes(sfidNumber),
    ))}';
    rpc.responses[refKey] = Uint8List.fromList([
      ...compactVec('SFR-AH001-20260507'),
      ...compactVec('主账户'),
    ]);
    rpc.responses[subjectKey] = adminSubjectBytes(
      threshold: 1,
      admin: List<int>.filled(32, 0xaa),
    );

    final admins =
        await service.fetchAdmins(registeredDuoqianIdentity(address));
    final threshold =
        await service.fetchThreshold(registeredDuoqianIdentity(address));

    expect(admins, ['aa' * 32]);
    expect(threshold, 1);
    expect(rpc.requestedKeys, [refKey, subjectKey]);
  });

  test('personal institution routes directly to personal subject', () async {
    final rpc = FakeChainRpc();
    final service = InstitutionAdminService(chainRpc: rpc);
    final address = '22' * 32;
    final subjectKey = '0x${hexOf(DuoqianStorageCodec.adminSubjectKey(
      DuoqianStorageCodec.subjectIdFromAccountHex(address),
    ))}';
    rpc.responses[subjectKey] = adminSubjectBytes(
      threshold: 1,
      admin: List<int>.filled(32, 0xbb),
    );

    final admins = await service.fetchAdmins('personal:$address');
    final threshold = await service.fetchThreshold('personal:$address');

    expect(admins, ['bb' * 32]);
    expect(threshold, 1);
    expect(rpc.requestedKeys, [subjectKey]);
  });
}
