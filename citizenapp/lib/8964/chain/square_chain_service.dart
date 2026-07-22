import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/foundation.dart' show visibleForTesting;
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:polkadart/scale_codec.dart' show ByteOutput, CompactBigIntCodec;

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/my/myid/citizen_identity_chain_reader.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/rpc/signed_extrinsic_builder.dart';
import 'package:citizenapp/rpc/subscription_rpc.dart' show SubscriptionRpc;

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
  SquareChainService({
    ChainRpc? chainRpc,
    CitizenIdentityChainReader? identityChainReader,
  })  : _rpc = chainRpc ?? ChainRpc(),
        _identityChainReader = identityChainReader ??
            CitizenIdentityChainReader(chainRpc: chainRpc);

  final ChainRpc _rpc;
  final CitizenIdentityChainReader _identityChainReader;

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
    final identity = await _identityChainReader.readByWallet(ownerAccount);
    if (identity == null || !votingIdentityIsActive(identity.votingIdentity)) {
      return null;
    }
    return identity.cidNumber;
  }

  /// 读链上身份档：有效投票身份的 cid + 是否竞选公民。
  /// identityLevel = visitor（无有效投票身份）/ voting / candidate（另有候选记录）。
  Future<({String? cidNumber, String identityLevel})> fetchIdentity(
    String ownerAccount,
  ) async {
    final identity = await _identityChainReader.readByWallet(ownerAccount);
    if (identity == null || !votingIdentityIsActive(identity.votingIdentity)) {
      return (cidNumber: null, identityLevel: 'visitor');
    }
    return (
      cidNumber: identity.cidNumber,
      identityLevel:
          identity.candidateIdentity != null ? 'candidate' : 'voting',
    );
  }

  /// 读链上平台会员某档月价：`PlatformPrice[level]`（u128 分，OptionQuery）。
  /// 平台价链上单源（治理设置），未设该档返回 null，页面据此显示占位。
  Future<int?> fetchPlatformPriceFen(String level) async {
    final data = await _rpc.fetchStorage(
      '0x${hexEncode(_platformPriceKey(SubscriptionRpc.membershipLevelByte(level)))}',
    );
    return decodePlatformPriceFen(data);
  }

  /// 一次读三档平台价（自由/民主/薪火），会员页据此逐档展示；缺档不入表。
  Future<Map<String, int>> fetchAllPlatformPrices() async {
    const levels = ['freedom', 'democracy', 'spark'];
    final prices = <String, int>{};
    for (final level in levels) {
      final fen = await fetchPlatformPriceFen(level);
      if (fen != null) prices[level] = fen;
    }
    return prices;
  }

  /// `PlatformPrice` map 键：twox128(SquarePost) ++ twox128(PlatformPrice)
  /// ++ Twox64Concat(levelByte)。Twox64Concat(x) = twox64(x) ++ x。
  static Uint8List _platformPriceKey(int levelByte) {
    final palletHash = Hasher.twoxx128.hashString('SquarePost');
    final storageHash = Hasher.twoxx128.hashString('PlatformPrice');
    final levelBytes = Uint8List.fromList([levelByte]);
    final keyHash = Hasher.twoxx64.hash(levelBytes);
    final result = Uint8List(
      palletHash.length +
          storageHash.length +
          keyHash.length +
          levelBytes.length,
    );
    var offset = 0;
    result.setAll(offset, palletHash);
    offset += palletHash.length;
    result.setAll(offset, storageHash);
    offset += storageHash.length;
    result.setAll(offset, keyHash);
    offset += keyHash.length;
    result.setAll(offset, levelBytes);
    return result;
  }

  /// 解码 `PlatformPrice` 值：u128 小端 16 字节 → int（分级金额在 int 安全范围）。
  /// 未设（null）或字节不足返回 null。
  @visibleForTesting
  static int? decodePlatformPriceFen(Uint8List? data) {
    if (data == null || data.length < 16) return null;
    var value = BigInt.zero;
    for (var i = 15; i >= 0; i--) {
      value = (value << 8) | BigInt.from(data[i]);
    }
    return value.toInt();
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
  static bool votingIdentityIsActive(Uint8List data, {int? today}) {
    try {
      if (!CitizenIdentityChainReader.votingIdentityLayoutIsValid(data)) {
        return false;
      }
      var offset = 0;
      final validFrom = _readU32Le(data, offset);
      offset += 4;
      final validUntil = _readU32Le(data, offset);
      offset += 4;
      final citizenStatus = data[offset];
      if (citizenStatus != 0) return false;
      final beijingNow = DateTime.now().toUtc().add(const Duration(hours: 8));
      final currentDate = today ??
          beijingNow.year * 10000 + beijingNow.month * 100 + beijingNow.day;
      return currentDate >= validFrom && currentDate <= validUntil;
    } catch (_) {
      return false;
    }
  }

  static void writeCompactBytes(ByteOutput output, Uint8List bytes) {
    output.write(CompactBigIntCodec.codec.encode(BigInt.from(bytes.length)));
    output.write(bytes);
  }

  static int _readU32Le(Uint8List data, int offset) =>
      data[offset] |
      (data[offset + 1] << 8) |
      (data[offset + 2] << 16) |
      (data[offset + 3] << 24);

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
