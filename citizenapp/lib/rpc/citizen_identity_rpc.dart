import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/foundation.dart' show visibleForTesting;
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:polkadart/scale_codec.dart' show CompactBigIntCodec, ByteOutput;

import 'package:citizenapp/citizen/shared/account_derivation.dart'
    show isAccountIdText;
import 'package:citizenapp/signer/signing.dart'
    show kOpSignCidRebind, signingMessage;
import 'package:citizenapp/wallet/core/wallet_manager.dart' show WalletManager;

import 'chain_rpc.dart';
import 'pallet_registry.dart';
import 'signed_extrinsic_builder.dart';

/// 匿名 CID 自助换绑在同一 finalized 区块读取出的防重放上下文。
class SelfRebindAuthorizationContext {
  const SelfRebindAuthorizationContext({
    required this.genesisHash,
    required this.expectedOldAccountId,
    required this.expectedBindingRevision,
    required this.expiresAt,
  });

  final Uint8List genesisHash;
  final String expectedOldAccountId;
  final BigInt expectedBindingRevision;

  /// Unix 秒。由 finalized `Timestamp.Now` 推导，不信任设备墙钟。
  final BigInt expiresAt;
}

/// CitizenIdentity(pallet 10)匿名 CID 自助占号 / 换绑的 extrinsic 构造与提交。
///
/// 两条 call 均由用户本人钱包自签、自付最低链上费(immortal、tip 0),不经注册局:
/// - `self_occupy_cid`(call 5):一笔自签占一个 CN 前缀匿名 CID + 占即绑本账户。
///   account_id 由 origin 派生,commitment 链上算,client 只送 cid_number。
/// - `self_rebind_cid_account_id`(call 9):把 CID 从旧账户换绑到新账户。origin = 新账户
///   (自签即证新账户受控),另附旧账户对创世、当前绑定、绑定修订号与时效的域分离授权签名。
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

  /// 链端允许上限为 600 秒；客户端取 300 秒，留出 finalized 提交时间且不贴上限。
  static const int _authorizationTtlSeconds = 300;

  static final BigInt _u64Max = (BigInt.one << 64) - BigInt.one;

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
  }) async {
    final callData = buildSelfOccupyCidCall(cidNumber);
    final result = await SignedExtrinsicBuilder(
      chainRpc: _rpc,
      logLabel: 'CitizenIdentityRpc',
    ).signAndSubmitInBlock(
      callData: callData,
      fromSs58Address: fromSs58Address,
      signerPublicKey: _accountId32(accountId),
      sign: (payload) => _wallet.signForAccountId(accountId, payload),
      waitForFinalized: true,
    );
    // finalized inclusion 不等于 Dispatch Success；只有目标绑定和首次 revision 均已落链，
    // 上层才可广播身份变化或继续本地凭证初始化。
    await verifyFinalizedBindingState(
      cidNumber: cidNumber,
      expectedAccountId: accountId,
      expectedBindingRevision: BigInt.one,
      finalizedBlockHashHex: result.blockHashHex,
    );
    return result;
  }

  /// 提交 `self_rebind_cid_account_id`:匿名 CID 从旧账户换绑到新账户。
  ///
  /// 先在同一 finalized 区块读取当前绑定账户、`BindingRevisionByCid` 与链上时间，
  /// 并读取 block 0 创世哈希；只有读出的旧账户仍等于 [oldAccountId] 才签名。旧账户签
  /// `CidRebindAuthorization`，新账户作为 origin 自签提交。修订号、旧账户、创世与过期
  /// 时间共同阻断旧授权在后续 A→B→A 或跨链场景重放。
  Future<({String txHash, int usedNonce, String blockHashHex})>
      selfRebindCidAccount({
    required String cidNumber,
    required String newAccountId,
    required String oldAccountId,
    required String newFromSs58Address,
  }) async {
    final context = await fetchSelfRebindAuthorizationContext(cidNumber);
    if (context.expectedOldAccountId != oldAccountId) {
      throw StateError('CID 当前绑定账户已经变化，请刷新身份后重试');
    }
    // 旧账户对冻结防重放上下文的授权：sr25519 签 32 字节 signing_message 摘要。
    final digest = buildRebindSigningDigest(
      genesisHash: context.genesisHash,
      cidNumber: cidNumber,
      expectedOldAccountId: context.expectedOldAccountId,
      newAccountId: newAccountId,
      expectedBindingRevision: context.expectedBindingRevision,
      expiresAt: context.expiresAt,
    );
    final oldSignature = await _wallet.signForAccountId(oldAccountId, digest);
    if (oldSignature.length != _signatureBytes) {
      throw StateError(
        '旧账户换绑授权签名长度必须为 $_signatureBytes 字节,当前 ${oldSignature.length}',
      );
    }
    final callData = buildSelfRebindCidAccountCall(
      cidNumber: cidNumber,
      expectedBindingRevision: context.expectedBindingRevision,
      expiresAt: context.expiresAt,
      oldAccountSignature: oldSignature,
    );
    final result = await SignedExtrinsicBuilder(
      chainRpc: _rpc,
      logLabel: 'CitizenIdentityRpc',
    ).signAndSubmitInBlock(
      callData: callData,
      fromSs58Address: newFromSs58Address,
      signerPublicKey: _accountId32(newAccountId),
      sign: (payload) => _wallet.signForAccountId(newAccountId, payload),
      waitForFinalized: true,
    );
    // 交易即使 Dispatch Failed 也会进入 finalized 区块；必须按该精确区块的存储确认
    // 新账户与 revision+1 已生效，MyIdService 才能让当前新账户接管 CID 数据根与设备子钥。
    await verifyFinalizedBindingState(
      cidNumber: cidNumber,
      expectedAccountId: newAccountId,
      expectedBindingRevision: context.expectedBindingRevision + BigInt.one,
      finalizedBlockHashHex: result.blockHashHex,
    );
    return result;
  }

  /// 在一个 finalized 锚点读取自助换绑授权所需真值。
  ///
  /// `AccountIdByCid`、`BindingRevisionByCid`、`Timestamp.Now` 任一缺失或 SCALE
  /// 非法都 fail-closed；禁止回退本地缓存、设备时间或调用方传入修订号。
  Future<SelfRebindAuthorizationContext> fetchSelfRebindAuthorizationContext(
      String cidNumber) async {
    final finalized = await _rpc.fetchFinalizedBlock();
    final finalizedHash =
        '0x${SignedExtrinsicBuilder.hexEncode(finalized.blockHash)}';
    final cidScale = _encodedCid(cidNumber);
    final accountKey = _storageMapKey(
      'CitizenIdentity',
      'AccountIdByCid',
      cidScale,
    );
    final revisionKey = _storageMapKey(
      'CitizenIdentity',
      'BindingRevisionByCid',
      cidScale,
    );
    final timestampKey = _storageValueKey('Timestamp', 'Now');
    final rows = await Future.wait<Object?>([
      _rpc.fetchGenesisHash(),
      _rpc.fetchStorageAtBlock(_hex(accountKey), finalizedHash),
      _rpc.fetchStorageAtBlock(_hex(revisionKey), finalizedHash),
      _rpc.fetchStorageAtBlock(_hex(timestampKey), finalizedHash),
    ]);
    final genesisHash = rows[0] as Uint8List;
    final oldAccountId = rows[1] as Uint8List?;
    final revisionRaw = rows[2] as Uint8List?;
    final timestampRaw = rows[3] as Uint8List?;
    if (genesisHash.length != 32) {
      throw const FormatException('创世块哈希必须为 32 字节');
    }
    if (oldAccountId == null || oldAccountId.length != 32) {
      throw StateError('CID 当前没有有效绑定账户');
    }
    final revision = _decodeU64(revisionRaw, 'BindingRevisionByCid');
    if (revision <= BigInt.zero) {
      throw const FormatException('BindingRevisionByCid 必须大于 0');
    }
    if (revision == _u64Max) {
      throw StateError('CID 绑定 revision 已达 u64 上限，禁止继续签署换绑授权');
    }
    final timestampMillis = _decodeU64(timestampRaw, 'Timestamp.Now');
    final expiresAt = timestampMillis ~/ BigInt.from(1000) +
        BigInt.from(_authorizationTtlSeconds);
    return SelfRebindAuthorizationContext(
      genesisHash: Uint8List.fromList(genesisHash),
      expectedOldAccountId:
          '0x${SignedExtrinsicBuilder.hexEncode(oldAccountId)}',
      expectedBindingRevision: revision,
      expiresAt: expiresAt,
    );
  }

  /// 在提交结果给业务层前，核验 inclusion 所在 finalized 区块的精确目标绑定状态。
  ///
  /// 该检查不依赖 extrinsic 事件解码；Dispatch Failed、目标账户错误、revision 未推进或
  /// 存储缺失都统一 fail-closed，禁止上层把“已进 finalized 区块”误当业务成功。
  @visibleForTesting
  Future<void> verifyFinalizedBindingState({
    required String cidNumber,
    required String expectedAccountId,
    required BigInt expectedBindingRevision,
    required String finalizedBlockHashHex,
  }) async {
    final expectedAccount = _accountId32(expectedAccountId);
    if (expectedBindingRevision <= BigInt.zero ||
        expectedBindingRevision > _u64Max) {
      throw ArgumentError('目标绑定 revision 必须是合法的非零 u64');
    }
    final cidScale = _encodedCid(cidNumber);
    final accountKey = _storageMapKey(
      'CitizenIdentity',
      'AccountIdByCid',
      cidScale,
    );
    final revisionKey = _storageMapKey(
      'CitizenIdentity',
      'BindingRevisionByCid',
      cidScale,
    );
    final rows = await Future.wait<Uint8List?>([
      _rpc.fetchStorageAtBlock(_hex(accountKey), finalizedBlockHashHex),
      _rpc.fetchStorageAtBlock(_hex(revisionKey), finalizedBlockHashHex),
    ]);
    final actualAccount = rows[0];
    final actualRevision =
        rows[1] == null ? null : _decodeU64(rows[1], 'BindingRevisionByCid');
    if (actualAccount == null ||
        actualAccount.length != expectedAccount.length ||
        !_sameBytes(actualAccount, expectedAccount) ||
        actualRevision != expectedBindingRevision) {
      throw StateError('交易已 finalized，但 CID 目标账户或绑定 revision 未生效');
    }
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

  /// `self_rebind_cid_account_id` call data:
  /// `[10][9][BoundedVec(cid)][revision:u64 LE][expires_at:u64 LE]`
  /// `[BoundedVec(old_signature=64B)]`。
  @visibleForTesting
  static Uint8List buildSelfRebindCidAccountCall({
    required String cidNumber,
    required BigInt expectedBindingRevision,
    required BigInt expiresAt,
    required Uint8List oldAccountSignature,
  }) {
    if (oldAccountSignature.length != _signatureBytes) {
      throw ArgumentError(
        'sr25519 旧账户签名必须为 $_signatureBytes 字节,当前 ${oldAccountSignature.length}',
      );
    }
    final output = ByteOutput()
      ..pushByte(_palletIndex)
      ..pushByte(_selfRebindCidAccountCallIndex);
    _writeCidBoundedVec(output, cidNumber);
    output.write(_u64LittleEndian(expectedBindingRevision));
    output.write(_u64LittleEndian(expiresAt));
    // SignatureOf = BoundedVec<u8, MaxCitizenSignatureLength> = compact(len) ++ 字节。
    output.write(CompactBigIntCodec.codec
        .encode(BigInt.from(oldAccountSignature.length)));
    output.write(oldAccountSignature);
    return output.toBytes();
  }

  /// 旧账户在自助换绑时需要签名的 32 字节摘要。
  ///
  /// `payload = SCALE(CidRebindAuthorization)`，字段顺序固定为：
  /// `genesis_hash(H256 raw32) ++ cid(BoundedVec) ++ expected_old(AccountId32)`
  /// `++ new(AccountId32) ++ expected_revision(u64 LE) ++ expires_at(u64 LE)`；
  /// `digest  = signing_message(OP_SIGN_CID_REBIND, payload)`
  ///         `= blake2_256( GMB(3B) ++ [0x11] ++ payload )`。
  /// 逐字节对齐链端 `CidRebindAuthorization::encode()` 与 `verify_rebind_signature`。
  @visibleForTesting
  static Uint8List buildRebindSigningDigest({
    required Uint8List genesisHash,
    required String cidNumber,
    required String expectedOldAccountId,
    required String newAccountId,
    required BigInt expectedBindingRevision,
    required BigInt expiresAt,
  }) {
    if (genesisHash.length != 32) {
      throw ArgumentError('genesis_hash 必须为 32 字节');
    }
    final payload = <int>[
      ...genesisHash,
      ..._encodedCid(cidNumber),
      ..._accountId32(expectedOldAccountId),
      ..._accountId32(newAccountId),
      ..._u64LittleEndian(expectedBindingRevision),
      ..._u64LittleEndian(expiresAt),
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

  static Uint8List _encodedCid(String cidNumber) {
    final bytes = _cidBytes(cidNumber);
    return Uint8List.fromList([
      ...CompactBigIntCodec.codec.encode(BigInt.from(bytes.length)),
      ...bytes,
    ]);
  }

  static Uint8List _storageValueKey(String pallet, String storage) =>
      Uint8List.fromList([
        ...Hasher.twoxx128.hashString(pallet),
        ...Hasher.twoxx128.hashString(storage),
      ]);

  static Uint8List _storageMapKey(
    String pallet,
    String storage,
    Uint8List keyData,
  ) =>
      Uint8List.fromList([
        ..._storageValueKey(pallet, storage),
        ...Hasher.blake2b128.hash(keyData),
        ...keyData,
      ]);

  static String _hex(Uint8List value) =>
      '0x${SignedExtrinsicBuilder.hexEncode(value)}';

  static BigInt _decodeU64(Uint8List? value, String fieldName) {
    if (value == null || value.length != 8) {
      throw FormatException('$fieldName 必须为 8 字节 u64 SCALE');
    }
    var result = BigInt.zero;
    for (var index = value.length - 1; index >= 0; index--) {
      result = (result << 8) | BigInt.from(value[index]);
    }
    return result;
  }

  static Uint8List _u64LittleEndian(BigInt value) {
    if (value < BigInt.zero || value > _u64Max) {
      throw ArgumentError('u64 字段超出范围');
    }
    final result = Uint8List(8);
    var remaining = value;
    for (var index = 0; index < result.length; index++) {
      result[index] = (remaining & BigInt.from(0xff)).toInt();
      remaining >>= 8;
    }
    return result;
  }

  static bool _sameBytes(Uint8List left, Uint8List right) {
    if (left.length != right.length) return false;
    for (var index = 0; index < left.length; index++) {
      if (left[index] != right[index]) return false;
    }
    return true;
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
