import 'dart:typed_data';

import 'package:flutter/foundation.dart' show debugPrint, visibleForTesting;
import 'package:polkadart/polkadart.dart'
    show ExtrinsicPayload, SignatureType, SigningPayload;

import 'chain_rpc.dart';
import 'nonce_manager.dart';

/// 已签名 extrinsic 的统一构造结果。
class SignedExtrinsicTrace {
  const SignedExtrinsicTrace({
    required this.callData,
    required this.payloadBytes,
    required this.signature,
    required this.encoded,
    required this.signerPubkey,
    required this.genesisHash,
    required this.runtimeVersion,
    required this.registry,
    required this.nonce,
    required this.eraPeriod,
    required this.blockNumber,
  });

  final Uint8List callData;
  final Uint8List payloadBytes;
  final Uint8List signature;
  final Uint8List encoded;
  final Uint8List signerPubkey;
  final Uint8List genesisHash;
  final dynamic runtimeVersion;
  final dynamic registry;
  final int nonce;
  final int eraPeriod;
  final int blockNumber;
}

/// wuminapp 在线 signed extrinsic 统一构造器。
///
/// P-SIGN-001：Citizenchain PoW 链在线交易统一使用 immortal era。签名 payload
/// 和最终 extrinsic body 必须同时使用 `era = 0x00`，且 CheckEra 的 additional
/// signed hash 必须是创世块哈希。
class SignedExtrinsicBuilder {
  SignedExtrinsicBuilder({
    required ChainRpc chainRpc,
    required String logLabel,
  })  : _rpc = chainRpc,
        _logLabel = logLabel;

  final ChainRpc _rpc;
  final String _logLabel;

  static const int immortalEraPeriod = 0;
  static const int immortalBlockNumber = 0;

  Future<({String txHash, int usedNonce})> signAndSubmit({
    required Uint8List callData,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
    void Function(SignedExtrinsicTrace trace)? onTrace,
  }) async {
    debugPrint('[$_logLabel] 步骤1: 获取 metadata...');
    final metadata = await _rpc.fetchMetadata();
    debugPrint('[$_logLabel] 步骤2: 获取 genesisHash...');
    final genesisHash = await _rpc.fetchGenesisHash();
    final registry = metadata.chainInfo.scaleCodec.registry;

    debugPrint('[$_logLabel] 步骤3: 并行获取 runtimeVersion/nonce...');
    final results = await Future.wait([
      _rpc.fetchRuntimeVersion(),
      NonceManager.instance.getNextNonce(
        address: fromAddress,
        fetchChainNonce: _rpc.fetchNonce,
      ),
    ]);
    final runtimeVersion = results[0] as dynamic;
    final nonce = results[1] as int;
    debugPrint('[$_logLabel] nonce=$nonce, era=immortal');

    debugPrint('[$_logLabel] 步骤4: 构造 immortal 签名载荷...');
    final signingPayload = buildImmortalSigningPayload(
      callData: callData,
      specVersion: runtimeVersion.specVersion as int,
      transactionVersion: runtimeVersion.transactionVersion as int,
      genesisHash: genesisHash,
      nonce: nonce,
    );
    final payloadBytes = signingPayload.encode(registry);

    debugPrint('[$_logLabel] 步骤5: 签名 (${payloadBytes.length} bytes)...');
    final signature = await sign(payloadBytes);
    debugPrint('[$_logLabel] 签名完成 (${signature.length} bytes)');

    debugPrint('[$_logLabel] 步骤6: 编码 immortal extrinsic...');
    final extrinsicPayload = buildImmortalExtrinsicPayload(
      callData: callData,
      signerPubkey: signerPubkey,
      signature: signature,
      nonce: nonce,
    );
    final encoded = extrinsicPayload.encode(registry, SignatureType.sr25519);
    debugPrint('[$_logLabel] extrinsic 编码完成 (${encoded.length} bytes)');

    onTrace?.call(
      SignedExtrinsicTrace(
        callData: callData,
        payloadBytes: payloadBytes,
        signature: signature,
        encoded: encoded,
        signerPubkey: signerPubkey,
        genesisHash: genesisHash,
        runtimeVersion: runtimeVersion,
        registry: registry,
        nonce: nonce,
        eraPeriod: immortalEraPeriod,
        blockNumber: immortalBlockNumber,
      ),
    );

    debugPrint('[$_logLabel] 步骤7: 提交到链...');
    debugPrint('[$_logLabel] call data hex: ${hexEncode(callData)}');
    try {
      final txHash = await _rpc.submitExtrinsic(encoded);
      debugPrint('[$_logLabel] 提交成功: 0x${hexEncode(txHash)}');
      return (txHash: '0x${hexEncode(txHash)}', usedNonce: nonce);
    } catch (e) {
      NonceManager.instance.rollback(fromAddress);
      debugPrint('[$_logLabel] 提交失败，原始错误: $e');
      rethrow;
    }
  }

  @visibleForTesting
  static SigningPayload buildImmortalSigningPayload({
    required Uint8List callData,
    required int specVersion,
    required int transactionVersion,
    required Uint8List genesisHash,
    required int nonce,
  }) {
    final genesisHashHex = '0x${hexEncode(genesisHash)}';
    return SigningPayload(
      method: callData,
      specVersion: specVersion,
      transactionVersion: transactionVersion,
      genesisHash: genesisHashHex,
      blockHash: genesisHashHex,
      blockNumber: immortalBlockNumber,
      eraPeriod: immortalEraPeriod,
      nonce: nonce,
      tip: 0,
    );
  }

  @visibleForTesting
  static ExtrinsicPayload buildImmortalExtrinsicPayload({
    required Uint8List callData,
    required Uint8List signerPubkey,
    required Uint8List signature,
    required int nonce,
  }) {
    return ExtrinsicPayload(
      signer: signerPubkey,
      method: callData,
      signature: signature,
      eraPeriod: immortalEraPeriod,
      blockNumber: immortalBlockNumber,
      nonce: nonce,
      tip: 0,
    );
  }

  static String hexEncode(Uint8List bytes) {
    return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  }
}
