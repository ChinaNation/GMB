import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:polkadart/scale_codec.dart' show CompactBigIntCodec;

import 'package:citizenapp/8964/chain/square_chain_service.dart';
import 'package:citizenapp/8964/models/square_models.dart';

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
      36,
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

  test('只把 Normal 公民身份解码为认证 CID', () {
    final normal = _votingIdentityBytes(
      cidNumber: 'CN001-CTZN-000000001-2026',
      citizenStatus: 0,
    );
    final revoked = _votingIdentityBytes(
      cidNumber: 'CN001-CTZN-000000001-2026',
      citizenStatus: 1,
    );

    expect(
      SquareChainService.decodeNormalCitizenCidNumber(normal),
      'CN001-CTZN-000000001-2026',
    );
    expect(SquareChainService.decodeNormalCitizenCidNumber(revoked), isNull);
  });
}

Uint8List _votingIdentityBytes({
  required String cidNumber,
  required int citizenStatus,
}) {
  final out = BytesBuilder();
  final cidBytes = utf8.encode(cidNumber);
  out.add(CompactBigIntCodec.codec.encode(BigInt.from(cidBytes.length)));
  out.add(cidBytes);
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

List<int> _u32(int value) {
  final bytes = ByteData(4)..setUint32(0, value, Endian.little);
  return bytes.buffer.asUint8List();
}
