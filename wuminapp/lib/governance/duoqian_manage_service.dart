import 'package:flutter/foundation.dart';
import 'package:polkadart/polkadart.dart'
    show ExtrinsicPayload, Hasher, SignatureType, SigningPayload;
import 'package:polkadart/scale_codec.dart' show CompactBigIntCodec, ByteOutput;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import '../rpc/chain_rpc.dart';
import '../rpc/nonce_manager.dart';
import 'duoqian_manage_models.dart';

/// 多签账户管理链上交互服务（对应 DuoqianManage pallet 17）。
///
/// 负责 propose_create / propose_close / propose_create_personal 等
/// 提案创建类 extrinsic 的编码与提交,以及 SFID 注册状态和多签账户的
/// storage 查询。
///
/// Phase 3(2026-04-22): 本 pallet 内部的管理员投票入口已从链端物理删除,
/// 管理员投票一律走 `VotingEngine::internal_vote`(9.0),
/// 通过 [InternalVoteService] 或业务 service 的 `submitInternalVote`
/// 统一入口发送。
class DuoqianManageService {
  DuoqianManageService({ChainRpc? chainRpc}) : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;

  // ──── 常量 ────

  /// DuoqianManage pallet index（runtime pallet_index=17）。
  static const _palletIndex = 17;

  /// propose_create call_index=0。
  static const _proposeCreateCallIndex = 0;

  /// propose_close call_index=1。
  static const _proposeCloseCallIndex = 1;

  /// Mortal era 周期。
  static const _eraPeriod = 64;

  /// propose_create_personal call_index=3（Phase 2 重排,原 5）。
  static const _proposeCreatePersonalCallIndex = 3;

  /// ProposalData 中的 action 类型前缀。
  static const actionCreate = 1;
  static const actionClose = 2;

  // ──── Extrinsic 提交 ────

  /// 提交 propose_create extrinsic。
  ///
  /// 参数编码：[0x11][0x00] + sfid_id(BoundedVec<u8>) + account_name(BoundedVec<u8>)
  ///   + admin_count(u32 LE) + duoqian_admins(BoundedVec<AccountId32>)
  ///   + threshold(u32 LE) + amount(u128 LE)
  Future<({String txHash, int usedNonce})> submitProposeCreate({
    required Uint8List sfidId,
    required Uint8List accountName,
    required int adminCount,
    required List<Uint8List> adminPubkeys,
    required int threshold,
    required BigInt amountFen,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    final output = ByteOutput();
    output.pushByte(_palletIndex);
    output.pushByte(_proposeCreateCallIndex);

    // sfid_id: BoundedVec<u8> = Compact<u32> length + bytes
    output.write(CompactBigIntCodec.codec.encode(BigInt.from(sfidId.length)));
    output.write(sfidId);

    // account_name: BoundedVec<u8> = Compact<u32> length + bytes
    output.write(
        CompactBigIntCodec.codec.encode(BigInt.from(accountName.length)));
    output.write(accountName);

    // admin_count: u32 little-endian
    output.write(_u32ToLeBytes(adminCount));

    // duoqian_admins: BoundedVec<AccountId32> = Compact<u32> length + N × 32 bytes
    output.write(
        CompactBigIntCodec.codec.encode(BigInt.from(adminPubkeys.length)));
    for (final pubkey in adminPubkeys) {
      output.write(pubkey);
    }

    // threshold: u32 little-endian
    output.write(_u32ToLeBytes(threshold));

    // amount: u128 little-endian（非 Compact）
    output.write(_u128ToLeBytes(amountFen));

    return _signAndSubmit(
      callData: output.toBytes(),
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
    );
  }

  /// 提交 propose_create_personal extrinsic（个人多签，无需 SFID）。
  ///
  /// 参数编码：[0x11][0x05] + account_name(BoundedVec) + admin_count(u32 LE)
  ///   + duoqian_admins(BoundedVec<AccountId32>) + threshold(u32 LE) + amount(u128 LE)
  Future<({String txHash, int usedNonce})> submitProposeCreatePersonal({
    required Uint8List accountName,
    required int adminCount,
    required List<Uint8List> adminPubkeys,
    required int threshold,
    required BigInt amountFen,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    final output = ByteOutput();
    output.pushByte(_palletIndex);
    output.pushByte(_proposeCreatePersonalCallIndex);

    // account_name: BoundedVec<u8> = Compact<u32> length + bytes
    output.write(
        CompactBigIntCodec.codec.encode(BigInt.from(accountName.length)));
    output.write(accountName);

    // admin_count: u32 little-endian
    output.write(_u32ToLeBytes(adminCount));

    // duoqian_admins: BoundedVec<AccountId32>
    output.write(
        CompactBigIntCodec.codec.encode(BigInt.from(adminPubkeys.length)));
    for (final pubkey in adminPubkeys) {
      output.write(pubkey);
    }

    // threshold: u32 little-endian
    output.write(_u32ToLeBytes(threshold));

    // amount: u128 little-endian
    output.write(_u128ToLeBytes(amountFen));

    return _signAndSubmit(
      callData: output.toBytes(),
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
    );
  }

  /// 提交 propose_close extrinsic。
  ///
  /// 参数编码：[0x11][0x01] + duoqian_address(32B) + beneficiary(32B)
  Future<({String txHash, int usedNonce})> submitProposeClose({
    required String duoqianAddress,
    required String beneficiaryAddress,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    final output = ByteOutput();
    output.pushByte(_palletIndex);
    output.pushByte(_proposeCloseCallIndex);

    // duoqian_address: AccountId32 = 32 bytes
    output.write(_hexDecode(duoqianAddress));

    // beneficiary: AccountId32 = 32 bytes
    final beneficiaryId = Keyring().decodeAddress(beneficiaryAddress);
    output.write(beneficiaryId);

    return _signAndSubmit(
      callData: output.toBytes(),
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
    );
  }

  // 投票动作已迁移到 `InternalVoteService`（Phase 3, pallet=9 call=0）。

  // ──── 链上查询 ────

  /// 查询 SFID (sfid_id + account_name) 是否已注册，返回派生的多签地址 hex（null 表示未注册）。
  Future<String?> fetchSfidRegisteredAddress(
      Uint8List sfidId, Uint8List accountName) async {
    final key = _buildDoubleMapStorageKey(
      'DuoqianManage',
      'SfidRegisteredAddress',
      sfidId,
      accountName,
    );
    final data = await _rpc.fetchStorage('0x${_hexEncode(key)}');
    if (data == null || data.length < 32) return null;
    return _hexEncode(Uint8List.fromList(data.sublist(0, 32)));
  }

  /// 查询多签账户信息（从 DuoqianAccounts 存储解码）。
  ///
  /// 链上 SCALE 布局：
  ///   admin_count: u32 + threshold: u32
  ///   + duoqian_admins: BoundedVec<AccountId32>（Compact len + N × 32B）
  ///   + creator: AccountId32(32) + created_at: BlockNumber(u32)
  ///   + status: enum(u8)（0=Pending, 1=Active）
  Future<DuoqianAccountInfo?> fetchDuoqianAccount(
      String duoqianAddressHex) async {
    final key = _buildStorageKey(
      'DuoqianManage',
      'DuoqianAccounts',
      _hexDecode(duoqianAddressHex),
    );
    final data = await _rpc.fetchStorage('0x${_hexEncode(key)}');
    if (data == null || data.length < 8) return null;

    var offset = 0;

    // admin_count: u32
    final adminCount = _decodeU32(data, offset);
    offset += 4;

    // threshold: u32
    final threshold = _decodeU32(data, offset);
    offset += 4;

    // duoqian_admins: BoundedVec<AccountId32> = Compact<u32> len + N × 32
    final (adminLen, lenSize) = _decodeCompact(data, offset);
    offset += lenSize;
    final pubkeys = <String>[];
    for (var i = 0; i < adminLen && offset + 32 <= data.length; i++) {
      pubkeys.add(
          _hexEncode(Uint8List.fromList(data.sublist(offset, offset + 32))));
      offset += 32;
    }

    // creator: AccountId32(32) + created_at: u32(4)
    offset += 32 + 4;

    // status: enum u8（0=Pending, 1=Active）
    final statusByte = offset < data.length ? data[offset] : 0;
    final status =
        statusByte == 1 ? DuoqianStatus.active : DuoqianStatus.pending;

    return DuoqianAccountInfo(
      adminCount: adminCount,
      threshold: threshold,
      adminPubkeys: pubkeys,
      status: status,
    );
  }

  /// 从 ProposalData 解码多签管理提案（创建或关闭）。
  ///
  /// ProposalData 存储为 BoundedVec<u8>，SCALE：Compact<len> + [ACTION_TYPE(1B)] + action.encode()
  /// ACTION_CREATE(1): duoqian_address(32B) + proposer(32B) + admin_count(u32) + threshold(u32) + amount(u128)
  /// ACTION_CLOSE(2): duoqian_address(32B) + beneficiary(32B) + proposer(32B)
  ///
  /// 返回 CreateDuoqianProposalInfo 或 CloseDuoqianProposalInfo，解码失败返回 null。
  /// MODULE_TAG 前缀（与链上 duoqian-manage 的 MODULE_TAG 一致）。
  static const _moduleTag = [
    0x64,
    0x71,
    0x2d,
    0x6d,
    0x67,
    0x6d,
    0x74
  ]; // "dq-mgmt"

  Object? decodeManageProposalData(int proposalId, Uint8List raw) {
    try {
      var offset = 0;

      // BoundedVec<u8> 外层：Compact<len> + bytes
      final (vecLen, lenBytes) = _decodeCompact(raw, offset);
      offset += lenBytes;
      if (offset + vecLen > raw.length) return null;
      final data = raw.sublist(offset, offset + vecLen);

      // 跳过 MODULE_TAG 前缀（"dq-mgmt", 7 bytes）
      if (data.length < _moduleTag.length + 1) return null;
      for (var i = 0; i < _moduleTag.length; i++) {
        if (data[i] != _moduleTag[i]) return null;
      }
      final actionType = data[_moduleTag.length];
      final payload = data.sublist(_moduleTag.length + 1);

      if (actionType == actionCreate) {
        return _decodeCreateAction(proposalId, payload);
      } else if (actionType == actionClose) {
        return _decodeCloseAction(proposalId, payload);
      }
      return null;
    } catch (_) {
      return null;
    }
  }

  CreateDuoqianProposalInfo? _decodeCreateAction(
      int proposalId, Uint8List data) {
    // duoqian_address(32) + proposer(32) + admin_count(u32) + threshold(u32) + amount(u128)
    if (data.length < 32 + 32 + 4 + 4 + 16) return null;
    var offset = 0;

    final duoqianAddress =
        _hexEncode(Uint8List.fromList(data.sublist(offset, offset + 32)));
    offset += 32;

    final proposerBytes = data.sublist(offset, offset + 32);
    final proposerSs58 =
        Keyring().encodeAddress(Uint8List.fromList(proposerBytes), 2027);
    offset += 32;

    final adminCount = _decodeU32(data, offset);
    offset += 4;

    final threshold = _decodeU32(data, offset);
    offset += 4;

    final amountBytes = data.sublist(offset, offset + 16);
    var amountBig = BigInt.zero;
    for (var i = 15; i >= 0; i--) {
      amountBig = (amountBig << 8) | BigInt.from(amountBytes[i]);
    }

    return CreateDuoqianProposalInfo(
      proposalId: proposalId,
      duoqianAddress: duoqianAddress,
      proposer: proposerSs58,
      adminCount: adminCount,
      threshold: threshold,
      amountFen: amountBig,
    );
  }

  CloseDuoqianProposalInfo? _decodeCloseAction(int proposalId, Uint8List data) {
    // duoqian_address(32) + beneficiary(32) + proposer(32)
    if (data.length < 32 + 32 + 32) return null;
    var offset = 0;

    final duoqianAddress =
        _hexEncode(Uint8List.fromList(data.sublist(offset, offset + 32)));
    offset += 32;

    final beneficiaryBytes = data.sublist(offset, offset + 32);
    final beneficiarySs58 =
        Keyring().encodeAddress(Uint8List.fromList(beneficiaryBytes), 2027);
    offset += 32;

    final proposerBytes = data.sublist(offset, offset + 32);
    final proposerSs58 =
        Keyring().encodeAddress(Uint8List.fromList(proposerBytes), 2027);

    return CloseDuoqianProposalInfo(
      proposalId: proposalId,
      duoqianAddress: duoqianAddress,
      beneficiary: beneficiarySs58,
      proposer: proposerSs58,
    );
  }

  // ──── 内部：签名提交 ────

  Future<({String txHash, int usedNonce})> _signAndSubmit({
    required Uint8List callData,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    debugPrint('[DuoqianManage] 步骤1: 获取 metadata...');
    final metadata = await _rpc.fetchMetadata();
    debugPrint('[DuoqianManage] 步骤2: 获取 genesisHash...');
    final genesisHash = await _rpc.fetchGenesisHash();
    final registry = metadata.chainInfo.scaleCodec.registry;

    debugPrint('[DuoqianManage] 步骤3: 并行获取 runtimeVersion/nonce/latestBlock...');
    final results = await Future.wait([
      _rpc.fetchRuntimeVersion(),
      NonceManager.instance.getNextNonce(
        address: fromAddress,
        fetchChainNonce: _rpc.fetchNonce,
      ),
      _rpc.fetchLatestBlock(),
    ]);
    final runtimeVersion = results[0] as dynamic;
    final nonce = results[1] as int;
    final latestBlock = results[2] as ({Uint8List blockHash, int blockNumber});
    debugPrint(
        '[DuoqianManage] nonce=$nonce, block=${latestBlock.blockNumber}');

    debugPrint('[DuoqianManage] 步骤4: 构造签名载荷...');
    final signingPayload = SigningPayload(
      method: callData,
      specVersion: runtimeVersion.specVersion,
      transactionVersion: runtimeVersion.transactionVersion,
      genesisHash: '0x${_hexEncode(genesisHash)}',
      blockHash: '0x${_hexEncode(latestBlock.blockHash)}',
      blockNumber: latestBlock.blockNumber,
      eraPeriod: _eraPeriod,
      nonce: nonce,
      tip: 0,
    );
    final payloadBytes = signingPayload.encode(registry);

    debugPrint('[DuoqianManage] 步骤5: 签名 (${payloadBytes.length} bytes)...');
    final signature = await sign(payloadBytes);
    debugPrint('[DuoqianManage] 签名完成 (${signature.length} bytes)');

    debugPrint('[DuoqianManage] 步骤6: 编码 extrinsic...');
    final extrinsicPayload = ExtrinsicPayload(
      signer: signerPubkey,
      method: callData,
      signature: signature,
      eraPeriod: _eraPeriod,
      blockNumber: latestBlock.blockNumber,
      nonce: nonce,
      tip: 0,
    );
    final encoded = extrinsicPayload.encode(registry, SignatureType.sr25519);
    debugPrint('[DuoqianManage] extrinsic 编码完成 (${encoded.length} bytes)');

    debugPrint('[DuoqianManage] 步骤7: 提交到链...');
    debugPrint('[DuoqianManage] call data hex: ${_hexEncode(callData)}');
    try {
      final txHash = await _rpc.submitExtrinsic(encoded);
      debugPrint('[DuoqianManage] 提交成功: 0x${_hexEncode(txHash)}');
      return (txHash: '0x${_hexEncode(txHash)}', usedNonce: nonce);
    } catch (e) {
      NonceManager.instance.rollback(fromAddress);
      debugPrint('[DuoqianManage] 提交失败，原始错误: $e');
      rethrow;
    }
  }

  // ──── 内部：storage key 构造 ────

  Uint8List _buildStorageKey(
    String palletName,
    String storageName,
    Uint8List keyData,
  ) {
    final palletHash = Hasher.twoxx128.hashString(palletName);
    final storageHash = Hasher.twoxx128.hashString(storageName);
    final keyHash = _blake2128Concat(keyData);

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

  /// StorageDoubleMap key: twox128(pallet) + twox128(storage) + blake2_128_concat(key1) + blake2_128_concat(key2)
  Uint8List _buildDoubleMapStorageKey(
    String palletName,
    String storageName,
    Uint8List key1Data,
    Uint8List key2Data,
  ) {
    final palletHash = Hasher.twoxx128.hashString(palletName);
    final storageHash = Hasher.twoxx128.hashString(storageName);
    final key1Hash = _blake2128Concat(key1Data);
    final key2Hash = _blake2128Concat(key2Data);

    final result = Uint8List(palletHash.length +
        storageHash.length +
        key1Hash.length +
        key2Hash.length);
    var offset = 0;
    result.setAll(offset, palletHash);
    offset += palletHash.length;
    result.setAll(offset, storageHash);
    offset += storageHash.length;
    result.setAll(offset, key1Hash);
    offset += key1Hash.length;
    result.setAll(offset, key2Hash);
    return result;
  }

  Uint8List _blake2128Concat(Uint8List data) {
    final hash = Hasher.blake2b128.hash(data);
    final result = Uint8List(hash.length + data.length);
    result.setAll(0, hash);
    result.setAll(hash.length, data);
    return result;
  }

  // ──── 内部：编码工具 ────

  Uint8List _u32ToLeBytes(int value) {
    final bytes = Uint8List(4);
    final bd = ByteData.sublistView(bytes);
    bd.setUint32(0, value, Endian.little);
    return bytes;
  }

  Uint8List _u128ToLeBytes(BigInt value) {
    final bytes = Uint8List(16);
    var v = value;
    for (var i = 0; i < 16; i++) {
      bytes[i] = (v & BigInt.from(0xFF)).toInt();
      v >>= 8;
    }
    return bytes;
  }

  int _decodeU32(Uint8List data, int offset) {
    final bd = ByteData.sublistView(data);
    return bd.getUint32(offset, Endian.little);
  }

  (int, int) _decodeCompact(Uint8List data, int offset) {
    final first = data[offset];
    final mode = first & 0x03;
    if (mode == 0) {
      return (first >> 2, 1);
    } else if (mode == 1) {
      final val = (data[offset] | (data[offset + 1] << 8)) >> 2;
      return (val, 2);
    } else if (mode == 2) {
      final val = (data[offset] |
              (data[offset + 1] << 8) |
              (data[offset + 2] << 16) |
              (data[offset + 3] << 24)) >>
          2;
      return (val, 4);
    } else {
      final lenBytes = (first >> 2) + 4;
      var val = 0;
      for (var i = lenBytes - 1; i >= 0; i--) {
        val = (val << 8) | data[offset + 1 + i];
      }
      return (val, 1 + lenBytes);
    }
  }

  static String _hexEncode(Uint8List bytes) {
    return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  }

  Uint8List _hexDecode(String hex) {
    final h = hex.startsWith('0x') ? hex.substring(2) : hex;
    final result = Uint8List(h.length ~/ 2);
    for (var i = 0; i < result.length; i++) {
      result[i] = int.parse(h.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return result;
  }
}
