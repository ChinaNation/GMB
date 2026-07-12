import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/foundation.dart' show visibleForTesting;
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:polkadart/scale_codec.dart' show ByteOutput, CompactBigIntCodec;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/rpc/signed_extrinsic_builder.dart';

class SquareChainPublishedResult {
  const SquareChainPublishedResult({
    required this.txHash,
    required this.usedNonce,
    required this.blockHashHex,
  });

  final String txHash;
  final int usedNonce;
  final String blockHashHex;
}

abstract class SquarePostChainPublisher {
  Future<SquareChainPublishedResult> publishPost({
    required String fromAddress,
    required Uint8List signerPubkey,
    required String postId,
    required SquarePostCategory postCategory,
    required String contentHashHex,
    required String storageReceiptId,
    required int storageUntil,
    required Future<Uint8List> Function(Uint8List payload) sign,
    TxPoolWatchCallback? onWatchEvent,
  });
}

class SquareChainService implements SquarePostChainPublisher {
  SquareChainService({ChainRpc? chainRpc}) : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;

  static const int palletIndex = 34;
  static const int publishPostCallIndex = 0;
  static const int maxPostIdBytes = 64;
  static const int maxStorageReceiptIdBytes = 96;

  @override
  Future<SquareChainPublishedResult> publishPost({
    required String fromAddress,
    required Uint8List signerPubkey,
    required String postId,
    required SquarePostCategory postCategory,
    required String contentHashHex,
    required String storageReceiptId,
    required int storageUntil,
    required Future<Uint8List> Function(Uint8List payload) sign,
    TxPoolWatchCallback? onWatchEvent,
  }) async {
    final callData = buildPublishPostCallData(
      postId: postId,
      postCategory: postCategory,
      contentHashHex: contentHashHex,
      storageReceiptId: storageReceiptId,
      storageUntil: storageUntil,
    );
    final result = await SignedExtrinsicBuilder(
      chainRpc: _rpc,
      logLabel: 'SquareChainService',
    ).signAndSubmitInBlock(
      callData: callData,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
      onWatchEvent: onWatchEvent,
    );

    final events = await _rpc.fetchSystemEventsAtBlock(result.blockHashHex);
    final failure =
        events == null ? null : _rpc.findExtrinsicFailureInEvents(events);
    if (failure != null && failure.moduleIndex == palletIndex) {
      throw StateError('广场发布交易已入块但执行失败：${failure.description}');
    }

    return SquareChainPublishedResult(
      txHash: result.txHash,
      usedNonce: result.usedNonce,
      blockHashHex: result.blockHashHex,
    );
  }

  Future<String?> fetchNormalCitizenCidNumber(String ownerAccount) async {
    final accountId = Uint8List.fromList(Keyring().decodeAddress(ownerAccount));
    final key = storageMapKey(
      'CitizenIdentity',
      'VotingIdentityByAccount',
      accountId,
    );
    final data = await _rpc.fetchStorage('0x${hexEncode(key)}');
    if (data == null) return null;
    return decodeNormalCitizenCidNumber(data);
  }

  /// 读链上身份档：有效投票身份的 cid + 是否竞选公民。
  /// identityLevel = visitor（无有效投票身份）/ voting / candidate（另有候选记录）。
  Future<({String? cidNumber, String identityLevel})> fetchIdentity(
    String ownerAccount,
  ) async {
    final cid = await fetchNormalCitizenCidNumber(ownerAccount);
    if (cid == null) {
      return (cidNumber: null, identityLevel: 'visitor');
    }
    final accountId = Uint8List.fromList(Keyring().decodeAddress(ownerAccount));
    final candKey = storageMapKey(
      'CitizenIdentity',
      'CandidateIdentityByAccount',
      accountId,
    );
    final candData = await _rpc.fetchStorage('0x${hexEncode(candKey)}');
    return (
      cidNumber: cid,
      identityLevel: candData != null ? 'candidate' : 'voting',
    );
  }

  @visibleForTesting
  static Uint8List buildPublishPostCallData({
    required String postId,
    required SquarePostCategory postCategory,
    required String contentHashHex,
    required String storageReceiptId,
    required int storageUntil,
  }) {
    final postIdBytes = Uint8List.fromList(utf8.encode(postId.trim()));
    if (postIdBytes.isEmpty || postIdBytes.length > maxPostIdBytes) {
      throw ArgumentError('post_id 长度需在 1..=$maxPostIdBytes 字节');
    }

    final receiptBytes =
        Uint8List.fromList(utf8.encode(storageReceiptId.trim()));
    if (receiptBytes.isEmpty ||
        receiptBytes.length > maxStorageReceiptIdBytes) {
      throw ArgumentError(
          'storage_receipt_id 长度需在 1..=$maxStorageReceiptIdBytes 字节');
    }

    final contentHash = hexDecode(contentHashHex);
    if (contentHash.length != 32 || contentHash.every((byte) => byte == 0)) {
      throw ArgumentError('content_hash 必须是非零 32 字节 sha256 hex');
    }
    if (storageUntil <= 0) {
      throw ArgumentError('storage_until 必须大于 0');
    }

    final output = ByteOutput();
    output.pushByte(palletIndex);
    output.pushByte(publishPostCallIndex);
    writeCompactBytes(output, postIdBytes);
    output.pushByte(postCategory == SquarePostCategory.normal ? 0 : 1);
    output.write(contentHash);
    writeCompactBytes(output, receiptBytes);
    output.write(u64LittleEndian(storageUntil));
    return output.toBytes();
  }

  @visibleForTesting
  static String? decodeNormalCitizenCidNumber(Uint8List data) {
    try {
      var offset = 0;
      final cid = readCompactBytes(data, offset);
      offset = cid.nextOffset;
      if (offset + 4 + 4 + 1 > data.length) return null;
      offset += 4; // passport_valid_from
      offset += 4; // passport_valid_until
      final citizenStatus = data[offset];
      if (citizenStatus != 0) return null;
      final cidText = utf8.decode(cid.value, allowMalformed: false).trim();
      return cidText.isEmpty ? null : cidText;
    } catch (_) {
      return null;
    }
  }

  @visibleForTesting
  static Uint8List storageMapKey(
    String palletName,
    String storageName,
    Uint8List keyData,
  ) {
    final palletHash = Hasher.twoxx128.hashString(palletName);
    final storageHash = Hasher.twoxx128.hashString(storageName);
    final keyHash = blake2128Concat(keyData);
    final result =
        Uint8List(palletHash.length + storageHash.length + keyHash.length);
    var offset = 0;
    result.setAll(offset, palletHash);
    offset += palletHash.length;
    result.setAll(offset, storageHash);
    offset += storageHash.length;
    result.setAll(offset, keyHash);
    return result;
  }

  static void writeCompactBytes(ByteOutput output, Uint8List bytes) {
    output.write(CompactBigIntCodec.codec.encode(BigInt.from(bytes.length)));
    output.write(bytes);
  }

  static ({Uint8List value, int nextOffset}) readCompactBytes(
    Uint8List data,
    int offset,
  ) {
    final (length, lengthSize) = readCompactU32(data, offset);
    final start = offset + lengthSize;
    final end = start + length;
    if (end > data.length) {
      throw const FormatException('Compact bytes 长度越界');
    }
    return (
      value: Uint8List.fromList(data.sublist(start, end)),
      nextOffset: end
    );
  }

  static (int, int) readCompactU32(Uint8List data, int offset) {
    if (offset >= data.length) {
      throw const FormatException('Compact<u32> offset 越界');
    }
    final first = data[offset];
    final mode = first & 0x03;
    if (mode == 0) return (first >> 2, 1);
    if (mode == 1) {
      if (offset + 1 >= data.length) {
        throw const FormatException('Compact<u32> mode1 长度不足');
      }
      return ((first >> 2) | (data[offset + 1] << 6), 2);
    }
    if (mode == 2) {
      if (offset + 3 >= data.length) {
        throw const FormatException('Compact<u32> mode2 长度不足');
      }
      return (
        (first >> 2) |
            (data[offset + 1] << 6) |
            (data[offset + 2] << 14) |
            (data[offset + 3] << 22),
        4,
      );
    }
    throw const FormatException('Compact<u32> big-integer 模式暂不支持');
  }

  static Uint8List blake2128Concat(Uint8List data) {
    final hash = Hasher.blake2b128.hash(data);
    final result = Uint8List(hash.length + data.length);
    result.setAll(0, hash);
    result.setAll(hash.length, data);
    return result;
  }

  static Uint8List u64LittleEndian(int value) {
    if (value < 0) throw ArgumentError('u64 不能为负数');
    final out = Uint8List(8);
    final bytes = ByteData.sublistView(out);
    bytes.setUint64(0, value, Endian.little);
    return out;
  }

  static String hexEncode(List<int> bytes) =>
      bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();

  static Uint8List hexDecode(String hex) {
    final h = hex.startsWith('0x') ? hex.substring(2) : hex;
    if (h.isEmpty || h.length.isOdd || !RegExp(r'^[0-9a-fA-F]+$').hasMatch(h)) {
      throw FormatException('非法 hex: $hex');
    }
    final result = Uint8List(h.length ~/ 2);
    for (var i = 0; i < result.length; i++) {
      result[i] = int.parse(h.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return result;
  }
}
