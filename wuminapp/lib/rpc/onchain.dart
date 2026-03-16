import 'dart:typed_data';

import 'package:polkadart/polkadart.dart'
    show ExtrinsicPayload, SignatureType, SigningPayload;
import 'package:polkadart/scale_codec.dart' show CompactBigIntCodec, ByteOutput;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import 'chain_rpc.dart';

/// onchain 模块所有 RPC 功能：extrinsic 构造、转账、交易确认查询。
class OnchainRpc {
  OnchainRpc({ChainRpc? chainRpc}) : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;

  /// Mortal era 周期（区块数）。64 ≈ 约 6.4 分钟有效期。
  static const _eraPeriod = 64;

  /// Balances pallet index（citizenchain runtime 定义）。
  static const _balancesPalletIndex = 2;

  /// transfer_keep_alive call index（标准 pallet_balances）。
  static const _transferKeepAliveCallIndex = 3;

  // ──── 公开方法 ────

  /// 执行 Balances::transfer_keep_alive 转账。
  ///
  /// [fromAddress] 发送方 SS58 地址
  /// [signerPubkey] 发送方公钥 32 字节
  /// [toAddress] 接收方 SS58 地址
  /// [amountYuan] 转账金额（元），内部转为分
  /// [sign] 签名回调：接收签名载荷字节，返回 64 字节 sr25519 签名
  ///
  /// 返回交易哈希 hex（含 0x 前缀）和提交时使用的 nonce。
  Future<({String txHash, int usedNonce})> transferKeepAlive({
    required String fromAddress,
    required Uint8List signerPubkey,
    required String toAddress,
    required double amountYuan,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    // 1. 获取链常量（缓存）
    final metadata = await _rpc.fetchMetadata();
    final genesisHash = await _rpc.fetchGenesisHash();
    final registry = metadata.chainInfo.scaleCodec.registry;

    // 2. 并行获取动态参数
    final results = await Future.wait([
      _rpc.fetchRuntimeVersion(),
      _rpc.fetchNonce(fromAddress),
      _rpc.fetchLatestBlock(),
    ]);
    final runtimeVersion = results[0] as dynamic;
    final nonce = results[1] as int;
    final latestBlock =
        results[2] as ({Uint8List blockHash, int blockNumber});

    // 3. 构造 call data
    final destAccountId = Keyring().decodeAddress(toAddress);
    final amountFen = BigInt.from((amountYuan * 100).round());
    final callData = _buildTransferKeepAliveCall(destAccountId, amountFen);

    // 4. 构造签名载荷
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

    // 5. 签名
    final signature = await sign(payloadBytes);

    // 6. 构造最终 extrinsic
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

    // 7. 提交
    final txHash = await _rpc.submitExtrinsic(encoded);
    return (txHash: '0x${_hexEncode(txHash)}', usedNonce: nonce);
  }

  /// 通过链上已打包的 nonce 判断交易是否已确认。
  ///
  /// 转账时使用的 nonce 为 N，若链上已确认 nonce > N 则交易已被打包。
  /// 注意：使用 state_getStorage 读取链上状态，不含交易池中的 pending 交易。
  Future<bool> isTxConfirmed({
    required String pubkeyHex,
    required int usedNonce,
  }) async {
    final confirmedNonce = await _rpc.fetchConfirmedNonce(pubkeyHex);
    return confirmedNonce > usedNonce;
  }

  // ──── 内部：extrinsic 编码 ────

  static String _hexEncode(Uint8List bytes) {
    return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  }

  /// 构造 Balances::transfer_keep_alive 的 SCALE 编码 call data。
  ///
  /// 格式：[pallet_index] [call_index] [MultiAddress::Id(0x00) + dest_32bytes] [Compact<u128>(fen)]
  Uint8List _buildTransferKeepAliveCall(
    Uint8List destAccountId,
    BigInt amountFen,
  ) {
    final output = ByteOutput();
    // pallet index + call index
    output.pushByte(_balancesPalletIndex);
    output.pushByte(_transferKeepAliveCallIndex);
    // MultiAddress::Id = 0x00 + 32 bytes account id
    output.pushByte(0x00);
    output.write(destAccountId);
    // Compact<u128> amount in fen
    output.write(CompactBigIntCodec.codec.encode(amountFen));
    return output.toBytes();
  }
}
