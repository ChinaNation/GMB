import 'dart:typed_data';

import 'package:polkadart/polkadart.dart'
    show ExtrinsicPayload, SignatureType, SigningPayload;
import 'package:polkadart/scale_codec.dart' show CompactBigIntCodec, ByteOutput;

import 'chain_rpc.dart';
import 'nonce_manager.dart';

/// 扫码支付清算体系:**清算行(L2)** 体系的链上 extrinsic 构造(唯一路径)。
///
/// 中文注释:
/// - 对应 `offchain-transaction-pos` pallet 的 4 个 call(call_index 30/31/32/33):
///   `bind_clearing_bank` / `deposit` / `withdraw` / `switch_bank`。原省储行
///   `bind_clearing_institution` (call_index 9) 已在 Step 2b-iv-b 随老 pallet
///   删除。
/// - Extrinsic 编码沿用现有 `OnchainRpc` 的 polkadart + SCALE 模式,确保与链上
///   验签格式一致(sr25519 签名,mortal era=64,带 nonce/tip)。
/// - 所有金额参数以**分**为单位的整数进入 SCALE 编码,与链上 `u128` 对齐。
class OnchainClearingBankRpc {
  OnchainClearingBankRpc({ChainRpc? chainRpc}) : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;

  /// Mortal era 周期(区块数),与 `OnchainRpc._eraPeriod` 对齐。
  static const int _eraPeriod = 64;

  /// `OffchainTransactionPos` pallet index(citizenchain runtime 定义)。
  static const int _palletIndex = 21;

  /// 4 个新 call_index(对应 lib.rs:586+ 中 call_index 30~33)。
  static const int _bindClearingBankCallIndex = 30;
  static const int _depositCallIndex = 31;
  static const int _withdrawCallIndex = 32;
  static const int _switchBankCallIndex = 33;

  // ──────────── 公开接口:4 个新 extrinsic ────────────

  /// `bind_clearing_bank(bank_main_address)`:L3 绑定清算行(绑定即开户,无预存)。
  ///
  /// [fromAddress]      L3 用户 SS58 地址
  /// [signerPubkey]     L3 用户公钥(32 字节)
  /// [bankMainAccount]  目标清算行**主账户**地址(32 字节,从 SFID API 拿到 hex 后解码)
  /// [sign]             签名回调
  Future<({String txHash, int usedNonce})> bindClearingBank({
    required String fromAddress,
    required Uint8List signerPubkey,
    required Uint8List bankMainAccount,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) {
    final callData = _buildBindClearingBankCall(bankMainAccount);
    return _submitExtrinsic(
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      callData: callData,
      sign: sign,
    );
  }

  /// `deposit(amount)`:L3 自持账户 → 清算行主账户充值。
  ///
  /// [amountFen] 充值金额(分,u128 范围内的正整数)。
  Future<({String txHash, int usedNonce})> deposit({
    required String fromAddress,
    required Uint8List signerPubkey,
    required BigInt amountFen,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) {
    final callData = _buildAmountOnlyCall(_depositCallIndex, amountFen);
    return _submitExtrinsic(
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      callData: callData,
      sign: sign,
    );
  }

  /// `withdraw(amount)`:清算行主账户 → L3 自持账户提现。
  Future<({String txHash, int usedNonce})> withdraw({
    required String fromAddress,
    required Uint8List signerPubkey,
    required BigInt amountFen,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) {
    final callData = _buildAmountOnlyCall(_withdrawCallIndex, amountFen);
    return _submitExtrinsic(
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      callData: callData,
      sign: sign,
    );
  }

  /// `switch_bank(new_bank)`:切换清算行(前置:旧清算行余额必须为 0)。
  Future<({String txHash, int usedNonce})> switchBank({
    required String fromAddress,
    required Uint8List signerPubkey,
    required Uint8List newBankMainAccount,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) {
    final callData = _buildAccountIdCall(
      _switchBankCallIndex,
      newBankMainAccount,
    );
    return _submitExtrinsic(
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      callData: callData,
      sign: sign,
    );
  }

  // ──────────── 内部:extrinsic 编码 ────────────

  /// `bind_clearing_bank` 与 `switch_bank` 都是接受单个 AccountId 参数,统一编码。
  ///
  /// 格式:`[pallet_index=21] [call_index] [account_id: [u8;32]]`。
  Uint8List _buildBindClearingBankCall(Uint8List bankMainAccount) {
    return _buildAccountIdCall(_bindClearingBankCallIndex, bankMainAccount);
  }

  Uint8List _buildAccountIdCall(int callIndex, Uint8List accountId) {
    if (accountId.length != 32) {
      throw ArgumentError('account_id 必须是 32 字节,实际 ${accountId.length}');
    }
    final output = ByteOutput()
      ..pushByte(_palletIndex)
      ..pushByte(callIndex)
      ..write(accountId);
    return output.toBytes();
  }

  /// 仅含 amount 参数的 extrinsic(deposit/withdraw 共用)。
  ///
  /// 格式:`[pallet_index=21] [call_index] [Compact<u128>(amount_fen)]`
  /// 与链上 `pub fn deposit(origin, amount: u128)` 严格对齐。
  Uint8List _buildAmountOnlyCall(int callIndex, BigInt amountFen) {
    if (amountFen <= BigInt.zero) {
      throw ArgumentError('amount 必须大于 0(分),实际 $amountFen');
    }
    final output = ByteOutput()
      ..pushByte(_palletIndex)
      ..pushByte(callIndex)
      ..write(CompactBigIntCodec.codec.encode(amountFen));
    return output.toBytes();
  }

  /// 通用 extrinsic 提交流程:获取 metadata + nonce + 当前块 → 构造 SigningPayload
  /// → 签名 → 编码 ExtrinsicPayload → submitExtrinsic。
  ///
  /// 复用 `OnchainRpc` 的 era/nonce/tip 风格(mortal era=64,失败时 nonce 回滚),
  /// 与其他 extrinsic 行为一致。
  Future<({String txHash, int usedNonce})> _submitExtrinsic({
    required String fromAddress,
    required Uint8List signerPubkey,
    required Uint8List callData,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    final metadata = await _rpc.fetchMetadata();
    final genesisHash = await _rpc.fetchGenesisHash();
    final registry = metadata.chainInfo.scaleCodec.registry;

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
    final latestBlock =
        results[2] as ({Uint8List blockHash, int blockNumber});

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
    final signature = await sign(payloadBytes);

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

    try {
      final txHash = await _rpc.submitExtrinsic(encoded);
      return (txHash: '0x${_hexEncode(txHash)}', usedNonce: nonce);
    } catch (e) {
      NonceManager.instance.rollback(fromAddress);
      rethrow;
    }
  }

  // ──────────── 通用工具 ────────────

  static String _hexEncode(Uint8List bytes) {
    return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  }
}
