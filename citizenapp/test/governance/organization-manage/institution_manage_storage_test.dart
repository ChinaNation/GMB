import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:citizenapp/transaction/organization-manage/institution_manage_models.dart';
import 'package:citizenapp/transaction/organization-manage/institution_manage_service.dart';
import 'package:citizenapp/transaction/organization-manage/multisig_storage_codec.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';

class FakeChainRpc extends ChainRpc {
  final Map<String, Uint8List?> responses = {};
  final List<String> requestedKeys = [];

  @override
  Future<Uint8List?> fetchStorage(String storageKeyHex) async {
    requestedKeys.add(storageKeyHex);
    return responses[storageKeyHex];
  }

  @override
  Future<Map<String, Uint8List?>> fetchStorageBatchChunked(
    Iterable<String> storageKeyHexList, {
    int chunkSize = 100,
  }) async {
    final result = <String, Uint8List?>{};
    for (final key in storageKeyHexList) {
      requestedKeys.add(key);
      result[key] = responses[key];
    }
    return result;
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

  Uint8List adminAccountBytes({
    required List<int> admin1,
    required List<int> admin2,
  }) {
    return Uint8List.fromList([
      ...codeBytes('UNIN'),
      2, // AdminAccountKind::InstitutionAccount
      (2 << 2) & 0xff,
      ...admin1,
      ...admin2,
      ...List<int>.filled(32, 0x44), // creator
      ...u32Le(100), // created_at
      ...u32Le(101), // updated_at
      1, // Active
    ]);
  }

  test('fetchAccount reads registered institution current storages', () async {
    final rpc = FakeChainRpc();
    final service = InstitutionManageService(chainRpc: rpc);
    final address = '11' * 32;
    final cidNumber =
        Uint8List.fromList(utf8.encode('AH001-SCB0H-202605070-2026'));
    final accountName = Uint8List.fromList(utf8.encode('主账户'));

    final refKey =
        '0x${hexOf(MultisigStorageCodec.accountRegisteredCidKey(address))}';
    final accountKey = '0x${hexOf(MultisigStorageCodec.institutionAccountKey(
      cidNumber,
      accountName,
    ))}';
    final adminKey = '0x${hexOf(MultisigStorageCodec.adminAccountKey(
      MultisigStorageCodec.accountIdFromAccountHex(address),
    ))}';
    final thresholdKey = '0x${hexOf(MultisigStorageCodec.dynamicThresholdKey(
      storageName: 'ActiveDynamicThresholds',
      institutionCode: 'UNIN',
      accountId: MultisigStorageCodec.accountIdFromAccountHex(
        address,
      ),
    ))}';
    rpc.responses[refKey] = Uint8List.fromList([
      ...compactVec('AH001-SCB0H-202605070-2026'),
      ...compactVec('主账户'),
    ]);
    rpc.responses[adminKey] = adminAccountBytes(
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
    rpc.responses[thresholdKey] = Uint8List.fromList(u32Le(2));

    final info = await service.fetchAccount(address);

    expect(info, isNotNull);
    expect(info!.adminsLen, 2);
    expect(info.threshold, 2);
    expect(info.admins, ['aa' * 32, 'bb' * 32]);
    expect(info.status, InstitutionStatus.active);
    expect(rpc.requestedKeys, [refKey, accountKey, adminKey, thresholdKey]);
  });

  test('fetchAccountsBatch reads institution accounts in staged batches',
      () async {
    final rpc = FakeChainRpc();
    final service = InstitutionManageService(chainRpc: rpc);
    final address = '22' * 32;
    final cidNumber =
        Uint8List.fromList(utf8.encode('AH001-SCB0E-202605180-2026'));
    final accountName = Uint8List.fromList(utf8.encode('主账户'));

    final refKey =
        '0x${hexOf(MultisigStorageCodec.accountRegisteredCidKey(address))}';
    final accountKey = '0x${hexOf(MultisigStorageCodec.institutionAccountKey(
      cidNumber,
      accountName,
    ))}';
    final adminKey = '0x${hexOf(MultisigStorageCodec.adminAccountKey(
      MultisigStorageCodec.accountIdFromAccountHex(address),
    ))}';
    final activeThresholdKey =
        '0x${hexOf(MultisigStorageCodec.dynamicThresholdKey(
      storageName: 'ActiveDynamicThresholds',
      institutionCode: 'UNIN',
      accountId: MultisigStorageCodec.accountIdFromAccountHex(
        address,
      ),
    ))}';

    rpc.responses[refKey] = Uint8List.fromList([
      ...compactVec('AH001-SCB0E-202605180-2026'),
      ...compactVec('主账户'),
    ]);
    rpc.responses[adminKey] = adminAccountBytes(
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
    rpc.responses[activeThresholdKey] = Uint8List.fromList(u32Le(2));

    final infos = await service.fetchAccountsBatch([address]);

    expect(infos[address]!.adminsLen, 2);
    expect(infos[address]!.threshold, 2);
    expect(infos[address]!.admins, ['aa' * 32, 'bb' * 32]);
    expect(infos[address]!.status, InstitutionStatus.active);
    expect(
        rpc.requestedKeys, [refKey, accountKey, adminKey, activeThresholdKey]);
  });
}
