import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart/scale_codec.dart' show CompactBigIntCodec;

import 'package:citizenapp/8964/chain/square_chain_service.dart';
import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/my/myid/citizen_identity_chain_reader.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';

void main() {
  test('publish_post call_data 与 runtime 下标和字段顺序一致', () {
    final hashHex = List<int>.generate(32, (index) => index + 1)
        .map((byte) => byte.toRadixString(16).padLeft(2, '0'))
        .join();

    final callData = SquareChainService.buildPublishPostCallData(
      postId: 'sqp_abc',
      postCategory: SquarePostCategory.campaign,
      contentHashHex: hashHex,
      storageReceiptId: 'sqr_receipt',
      storageUntil: 123456789,
    );

    final expected = <int>[
      34,
      0,
      ...CompactBigIntCodec.codec.encode(BigInt.from(7)),
      ...utf8.encode('sqp_abc'),
      1,
      ...List<int>.generate(32, (index) => index + 1),
      ...CompactBigIntCodec.codec.encode(BigInt.from(11)),
      ...utf8.encode('sqr_receipt'),
      ...SquareChainService.u64LittleEndian(123456789),
    ];
    expect(callData, Uint8List.fromList(expected));
  });

  test('publish_post 拒绝零 content_hash', () {
    expect(
      () => SquareChainService.buildPublishPostCallData(
        postId: 'sqp_abc',
        postCategory: SquarePostCategory.normal,
        contentHashHex: '00' * 32,
        storageReceiptId: 'sqr_receipt',
        storageUntil: 123,
      ),
      throwsArgumentError,
    );
  });

  test('只把护照有效且状态正常的投票身份视为有效', () {
    final normal = _votingIdentityBytes(
      citizenStatus: 0,
    );
    final revoked = _votingIdentityBytes(
      citizenStatus: 1,
    );

    expect(
      SquareChainService.votingIdentityIsActive(normal, today: 20260722),
      isTrue,
    );
    expect(
      SquareChainService.votingIdentityIsActive(normal, today: 20400101),
      isFalse,
    );
    expect(
      SquareChainService.votingIdentityIsActive(revoked, today: 20260722),
      isFalse,
    );
    expect(
      SquareChainService.votingIdentityIsActive(
        Uint8List.sublistView(normal, 0, 9),
        today: 20260722,
      ),
      isFalse,
    );
  });

  test('广场身份使用永久 CID 闭环快照而不是旧 Account-keyed storage', () async {
    final service = SquareChainService(
      identityChainReader: _FakeIdentityReader(
        CitizenIdentityChainSnapshot(
          cidNumber: 'CN001-CTZN-000000001-2026',
          accountId: Uint8List(32),
          votingIdentity: _votingIdentityBytes(citizenStatus: 0),
          candidateIdentity: _candidateIdentityBytes(),
        ),
      ),
    );

    final identity = await service.fetchIdentity('ignored-in-reader');
    expect(identity.cidNumber, 'CN001-CTZN-000000001-2026');
    expect(identity.identityLevel, 'candidate');
  });

  test('三档平台价格通过一次 finalized storage 批量读取', () async {
    final rpc = _BatchPriceChainRpc();
    final service = SquareChainService(chainRpc: rpc);

    final prices = await service.fetchAllPlatformPrices(forceFresh: true);

    expect(prices, const {
      'freedom': 29900,
      'democracy': 99900,
      'spark': 199900,
    });
    expect(rpc.fetchCount, 1);
    expect(rpc.requestedKeyCount, 3);
    expect(rpc.lastForceFresh, isTrue);
  });
}

Uint8List _candidateIdentityBytes() {
  final out = BytesBuilder();
  for (final value in ['CN', '001', '0001', '陈', '明']) {
    final bytes = utf8.encode(value);
    out.add(CompactBigIntCodec.codec.encode(BigInt.from(bytes.length)));
    out.add(bytes);
  }
  out.add([0]);
  out.add(_u32(20000131));
  out.add(_u32(88));
  return out.toBytes();
}

Uint8List _votingIdentityBytes({
  required int citizenStatus,
}) {
  final out = BytesBuilder();
  out.add(_u32(20260101));
  out.add(_u32(20360101));
  out.add([citizenStatus]);
  for (final code in ['CN', '001', '0001']) {
    final bytes = utf8.encode(code);
    out.add(CompactBigIntCodec.codec.encode(BigInt.from(bytes.length)));
    out.add(bytes);
  }
  out.add(_u32(88));
  return out.toBytes();
}

class _FakeIdentityReader extends CitizenIdentityChainReader {
  _FakeIdentityReader(this.snapshot);

  final CitizenIdentityChainSnapshot? snapshot;

  @override
  Future<CitizenIdentityChainSnapshot?> readByAccountId(
          String ss58Address) async =>
      snapshot;
}

class _BatchPriceChainRpc extends ChainRpc {
  int fetchCount = 0;
  int requestedKeyCount = 0;
  bool lastForceFresh = false;

  @override
  Future<Map<String, Uint8List?>> fetchStorageBatch(
    List<String> storageKeyHexList, {
    bool forceFresh = false,
  }) async {
    fetchCount++;
    requestedKeyCount = storageKeyHexList.length;
    lastForceFresh = forceFresh;
    const prices = [29900, 99900, 199900];
    return {
      for (var index = 0; index < storageKeyHexList.length; index++)
        storageKeyHexList[index]: _u128(prices[index]),
    };
  }
}

List<int> _u32(int value) {
  final bytes = ByteData(4)..setUint32(0, value, Endian.little);
  return bytes.buffer.asUint8List();
}

Uint8List _u128(int value) {
  final bytes = Uint8List(16);
  ByteData.sublistView(bytes).setUint64(0, value, Endian.little);
  return bytes;
}
