import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:wuminapp_mobile/admins-change/codec/subject_id_codec.dart';
import 'package:wuminapp_mobile/admins-change/services/institution_admin_service.dart';
import 'package:wuminapp_mobile/common/institution_info.dart';
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

  List<int> u32Le(int value) => [
        value & 0xff,
        (value >> 8) & 0xff,
        (value >> 16) & 0xff,
        (value >> 24) & 0xff,
      ];

  List<int> u64Le(int value) => [
        value & 0xff,
        (value >> 8) & 0xff,
        (value >> 16) & 0xff,
        (value >> 24) & 0xff,
        (value >> 32) & 0xff,
        (value >> 40) & 0xff,
        (value >> 48) & 0xff,
        (value >> 56) & 0xff,
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
      ...List<int>.filled(32, 0xcc),
      ...u64Le(1),
      ...u64Le(2),
      1,
    ]);
  }

  test('registered institution account routes to institution-account subject',
      () async {
    final rpc = FakeChainRpc();
    final service = InstitutionAdminService(chainRpc: rpc);
    final address = '11' * 32;
    final subjectKey = '0x${hexOf(AdminSubjectIdCodec.adminSubjectStorageKey(
      AdminSubjectIdCodec.fromAccountHex(
        AdminSubjectIdCodec.institutionAccount,
        address,
      ),
    ))}';
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
    expect(rpc.requestedKeys, [subjectKey]);
  });

  test('personal institution routes directly to personal subject', () async {
    final rpc = FakeChainRpc();
    final service = InstitutionAdminService(chainRpc: rpc);
    final address = '22' * 32;
    final subjectKey = '0x${hexOf(AdminSubjectIdCodec.adminSubjectStorageKey(
      AdminSubjectIdCodec.fromAccountHex(
        AdminSubjectIdCodec.personalDuoqian,
        address,
      ),
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
