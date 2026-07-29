import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/foundation.dart' show visibleForTesting;
import 'package:polkadart/scale_codec.dart' show CompactBigIntCodec, ByteOutput;

import 'package:citizenapp/citizen/shared/account_derivation.dart'
    show isAccountIdText;
import 'package:citizenapp/signer/signing.dart'
    show kOpSignCidRebind, signingMessage;
import 'package:citizenapp/wallet/core/wallet_manager.dart' show WalletManager;

import 'chain_rpc.dart';
import 'pallet_registry.dart';
import 'signed_extrinsic_builder.dart';

/// CitizenIdentity(pallet 10)匿名 CID 自助占号 / 换绑的 extrinsic 构造与提交。
///
/// 两条 call 均由用户本人钱包自签、自付最低链上费(immortal、tip 0),不经注册局:
/// - `self_occupy_cid`(call 5):一笔自签占一个 CN 前缀匿名 CID + 占即绑本账户。
///   account_id 由 origin 派生,commitment 链上算,client 只送 cid_number。
/// - `self_rebind_cid_account`(call 9):把 CID 从旧账户换绑到新账户。origin = 新账户
///   (自签即证新账户受控),另附旧账户对 `(cid, new_account)` 的域分离授权签名。
///
/// SCALE 布局逐字节镜像 citizenchain `runtime/misc/citizen-identity/src/lib.rs`;
/// CID 编码为 `CidNumberBound = BoundedVec<u8, ConstU32<32>>`,签名编码为
/// `SignatureOf = BoundedVec<u8>`(compact(len) ++ bytes)。
class CitizenIdentityRpc {
  CitizenIdentityRpc({ChainRpc? chainRpc, WalletManager? walletManager})
      : _rpc = chainRpc ?? ChainRpc(),
        _wallet = walletManager ?? WalletManager();

  final ChainRpc _rpc;
  final WalletManager _wallet;

  static const int _palletIndex = PalletRegistry.citizenIdentityPallet; // 10
  static const int _selfOccupyCidCallIndex =
      PalletRegistry.selfOccupyCidCall; // 5
  static const int _selfRebindCidAccountCallIndex =
      PalletRegistry.selfRebindCidAccountCall; // 9

  /// CidNumberBound = BoundedVec<u8, ConstU32<32>>。
  static const int _cidMaxBytes = 32;

  /// sr25519 签名固定长度。
  static const int _signatureBytes = 64;

  // ──── 公开方法 ────

  /// 提交 `self_occupy_cid`:本人自签占一个匿名 CID 并绑本账户。
  ///
  /// [cidNumber] 由 [generateCitizenCid] 生成的首选候选号(CTZN/NATP)。
  /// [accountId] 占号并绑定的账户(= origin = 签名者,小写 `0x` + 64 hex);其私钥经
  /// `signForAccountId` 按 accountId 精确取用(**不走** interim 账户0 入口,占任意本地账户皆准)。
  /// [fromSs58Address] 该账户 SS58,供构造器查 nonce。
  Future<({String txHash, int usedNonce, String blockHashHex})> selfOccupyCid({
    required String cidNumber,
    required String accountId,
    required String fromSs58Address,
  }) {
    final callData = buildSelfOccupyCidCall(cidNumber);
    return SignedExtrinsicBuilder(
      chainRpc: _rpc,
      logLabel: 'CitizenIdentityRpc',
    ).signAndSubmitInBlock(
      callData: callData,
      fromSs58Address: fromSs58Address,
      signerPublicKey: _accountId32(accountId),
      sign: (payload) => _wallet.signForAccountId(accountId, payload),
      waitForFinalized: true,
    );
  }

  /// 提交 `self_rebind_cid_account`:匿名 CID 从旧账户换绑到新账户。
  ///
  /// 先用**旧账户**(当前绑定,[oldAccountId])对 `(cid_number, new_account_id)` 做域分离
  /// (OP_SIGN_CID_REBIND)授权签名,再由**新账户**([newAccountId])自签提交交易。二者私钥
  /// 均经 `signForAccountId` 按 accountId 精确取用(各弹一次生物识别),故新账户可为任意本地
  /// 账户(含非账户0),不受 interim 账户0 入口限制。旧账户亦从链上 `AccountIdByCid[cid]`
  /// 反查、不上送。
  Future<({String txHash, int usedNonce, String blockHashHex})>
      selfRebindCidAccount({
    required String cidNumber,
    required String newAccountId,
    required String oldAccountId,
    required String newFromSs58Address,
    Future<void> Function(String oldAccountSignature)? onOldAuthorizationReady,
  }) async {
    // 旧账户对 (cid ‖ 新账户) 的授权:sr25519 签 32 字节 signing_message 摘要。
    final digest = buildRebindSigningDigest(
      cidNumber: cidNumber,
      newAccountId: newAccountId,
    );
    final oldSignature = await _wallet.signForAccountId(oldAccountId, digest);
    if (oldSignature.length != _signatureBytes) {
      throw StateError(
        '旧账户换绑授权签名长度必须为 $_signatureBytes 字节,当前 ${oldSignature.length}',
      );
    }
    // 授权签名本身不是私钥，可在提交交易前作为安全 outbox 持久化；这样即便交易
    // finalized 后进程立即退出，新账户仍能重放同一授权完成旧账户材料清理。
    await onOldAuthorizationReady?.call(
      '0x${SignedExtrinsicBuilder.hexEncode(oldSignature)}',
    );
    final callData = buildSelfRebindCidAccountCall(cidNumber, oldSignature);
    return SignedExtrinsicBuilder(
      chainRpc: _rpc,
      logLabel: 'CitizenIdentityRpc',
    ).signAndSubmitInBlock(
      callData: callData,
      fromSs58Address: newFromSs58Address,
      signerPublicKey: _accountId32(newAccountId),
      sign: (payload) => _wallet.signForAccountId(newAccountId, payload),
      waitForFinalized: true,
    );
  }

  // ──── 内部：extrinsic 编码 ────

  /// `self_occupy_cid` call data:`[10][5][BoundedVec<u8>(cid)]`。
  @visibleForTesting
  static Uint8List buildSelfOccupyCidCall(String cidNumber) {
    final output = ByteOutput()
      ..pushByte(_palletIndex)
      ..pushByte(_selfOccupyCidCallIndex);
    _writeCidBoundedVec(output, cidNumber);
    return output.toBytes();
  }

  /// `self_rebind_cid_account` call data:
  /// `[10][9][BoundedVec<u8>(cid)][BoundedVec<u8>(old_signature=64B)]`。
  @visibleForTesting
  static Uint8List buildSelfRebindCidAccountCall(
    String cidNumber,
    Uint8List oldSignature,
  ) {
    if (oldSignature.length != _signatureBytes) {
      throw ArgumentError(
        'sr25519 旧账户签名必须为 $_signatureBytes 字节,当前 ${oldSignature.length}',
      );
    }
    final output = ByteOutput()
      ..pushByte(_palletIndex)
      ..pushByte(_selfRebindCidAccountCallIndex);
    _writeCidBoundedVec(output, cidNumber);
    // SignatureOf = BoundedVec<u8, MaxCitizenSignatureLength> = compact(len) ++ 字节。
    output.write(
        CompactBigIntCodec.codec.encode(BigInt.from(oldSignature.length)));
    output.write(oldSignature);
    return output.toBytes();
  }

  /// 旧账户在自助换绑时需要签名的 32 字节摘要。
  ///
  /// `payload = SCALE( (cid_number: BoundedVec<u8>, new_account_id: AccountId32) )`
  ///         `= compact(len(cid)) ++ cid.utf8 ++ new_account_id(32B)`,
  /// `digest  = signing_message(OP_SIGN_CID_REBIND, payload)`
  ///         `= blake2_256( GMB(3B) ++ [0x11] ++ payload )`。
  /// 逐字节对齐链端 `self_rebind_cid_account` 的 `(cid, new_account).encode()`
  /// 与 `verify_rebind_signature`。
  @visibleForTesting
  static Uint8List buildRebindSigningDigest({
    required String cidNumber,
    required String newAccountId,
  }) {
    final cidBytes = _cidBytes(cidNumber);
    final payload = <int>[
      ...CompactBigIntCodec.codec.encode(BigInt.from(cidBytes.length)),
      ...cidBytes,
      ..._accountId32(newAccountId),
    ];
    return signingMessage(opTag: kOpSignCidRebind, scalePayload: payload);
  }

  static void _writeCidBoundedVec(ByteOutput output, String cidNumber) {
    final bytes = _cidBytes(cidNumber);
    output.write(CompactBigIntCodec.codec.encode(BigInt.from(bytes.length)));
    output.write(bytes);
  }

  static Uint8List _cidBytes(String cidNumber) {
    final bytes = Uint8List.fromList(utf8.encode(cidNumber));
    if (bytes.isEmpty || bytes.length > _cidMaxBytes) {
      throw ArgumentError(
        'cid_number 的 UTF-8 长度必须为 1..$_cidMaxBytes 字节,当前 ${bytes.length}',
      );
    }
    return bytes;
  }

  /// account_id 文本(ADR-040 小写 `0x` + 64 hex)→ 32 字节;不兼容 SS58。
  static Uint8List _accountId32(String accountId) {
    if (!isAccountIdText(accountId)) {
      throw ArgumentError('account_id 必须为小写 0x + 64 位十六进制');
    }
    return Uint8List.fromList([
      for (var index = 2; index < accountId.length; index += 2)
        int.parse(accountId.substring(index, index + 2), radix: 16),
    ]);
  }
}
